//! Purpose:
//! `json_encode` for the wasm32-wasi backend: scalars, arrays, hashes, and objects.
//!
//! Called from:
//! - `plan::lower_module` — `emit_json_property_table` lays out the per-class public-property
//!   metadata, and `emit_json_runtime` adds the encoder, both only for a module that calls
//!   `json_encode`.
//! - `builtins` — the shape check and the lowering.
//!
//! Key details:
//! - The encoder appends into ONE growable buffer held in globals rather than concatenating
//!   returned strings, so encoding a nested structure costs amortised appends instead of a copy
//!   per level.
//! - An object is encoded the way php does it: a class implementing `JsonSerializable` has its
//!   `jsonSerialize()` called and the RESULT encoded; any other class has its PUBLIC properties
//!   walked in declaration order. Both need per-class metadata, which is laid out here in the
//!   same shape `objects::emit_gc_desc_table` uses — a pointer table indexed by `class_id`.
//! - Strings are escaped with php's DEFAULT flags: `"` `\` `/` and the control characters get
//!   their short forms, everything else below 0x20 becomes `\u00XX`, and a non-ASCII sequence is
//!   decoded from UTF-8 and re-emitted as `\uXXXX` — with a surrogate pair above U+FFFF, which
//!   is what php produces without `JSON_UNESCAPED_UNICODE`.
//! - A FLOAT is refused by the capability audit rather than encoded: php writes one with
//!   serialize_precision (17), which is not the precision the existing formatter uses, so the
//!   two would disagree on values this backend cannot currently spell.

use std::collections::{HashMap, HashSet};

use crate::ir::Module;
use crate::types::{ClassInfo, PhpType};

use super::wat::{DataSegment, Global, ValType, WatModule};

/// One public property, as the encoder reads it.
struct JsonProperty {
    name: String,
    /// Byte offset of the property's 16-byte slot inside the object payload.
    offset: u32,
    /// The runtime tag the slot's bytes carry, matching `objects::gc_desc_tag`.
    tag: u32,
}

/// The runtime tag for a property type, mirroring `objects::gc_desc_tag` — the same numbering a
/// Mixed cell uses, so a slot can be boxed by pairing its bytes with this tag.
fn json_property_tag(ty: &PhpType) -> Option<u32> {
    match ty.codegen_repr() {
        PhpType::Int => Some(0),
        PhpType::Str => Some(1),
        PhpType::Bool | PhpType::False => Some(3),
        PhpType::Array(_) => Some(4),
        PhpType::AssocArray { .. } => Some(5),
        PhpType::Object(_) => Some(6),
        PhpType::Mixed | PhpType::Union(_) => Some(7),
        // A float would need php's serialize_precision, and anything else has no JSON form the
        // audit is willing to claim.
        _ => None,
    }
}

/// Whether a class's public properties can all be encoded.
///
/// Walks nested classes with a visited set, so a self-referential type is answered rather than
/// followed forever.
fn class_is_encodable(module: &Module, class_name: &str, seen: &mut HashSet<String>) -> bool {
    if !seen.insert(crate::names::php_symbol_key(class_name)) {
        return true;
    }
    let Some(info) = crate::ir::class_relations::lookup_class(module, class_name) else {
        return false;
    };
    // A `JsonSerializable` answers through its own method, so its properties are never read.
    if implements_json_serializable(module, class_name) {
        return jsonserialize_target(module, class_name).is_some();
    }
    public_properties(info).iter().all(|(name, ty)| {
        let _ = name;
        match json_property_tag(ty) {
            None => false,
            Some(_) => match ty.codegen_repr() {
                PhpType::Object(nested) => class_is_encodable(module, &nested, seen),
                _ => true,
            },
        }
    })
}

/// Whether `php` and everything it can contain has a JSON form this encoder produces.
pub(super) fn type_is_encodable(module: &Module, php: &PhpType) -> bool {
    let mut seen = HashSet::new();
    type_is_encodable_inner(module, php, &mut seen)
}

/// The recursive half of `type_is_encodable`.
fn type_is_encodable_inner(module: &Module, php: &PhpType, seen: &mut HashSet<String>) -> bool {
    match php.codegen_repr() {
        PhpType::Int | PhpType::Str | PhpType::Bool | PhpType::False | PhpType::Void => true,
        PhpType::Array(element) => type_is_encodable_inner(module, &element, seen),
        PhpType::AssocArray { key, value } => {
            matches!(key.codegen_repr(), PhpType::Int | PhpType::Str)
                && type_is_encodable_inner(module, &value, seen)
        }
        PhpType::Object(class_name) => class_is_encodable(module, &class_name, seen),
        // A bare `mixed` promises nothing: its runtime tag could be a float or a resource, and
        // the encoder would have to answer for a value the audit never saw.
        _ => false,
    }
}

/// The public properties of a class, in declaration order.
fn public_properties(info: &ClassInfo) -> Vec<(String, PhpType)> {
    info.properties
        .iter()
        .filter(|(name, _)| {
            matches!(
                info.property_visibilities.get(name),
                Some(crate::parser::ast::Visibility::Public) | None
            )
        })
        .cloned()
        .collect()
}

/// Whether a class implements `JsonSerializable`, following its ancestors.
fn implements_json_serializable(module: &Module, class_name: &str) -> bool {
    crate::ir::class_relations::class_implements_interface(module, class_name, "JsonSerializable")
}

/// The compiled `jsonSerialize` body for a class, and whether it dispatches virtually.
fn jsonserialize_target(module: &Module, class_name: &str) -> Option<(String, bool)> {
    let key = crate::names::php_symbol_key("jsonSerialize");
    let info = crate::ir::class_relations::lookup_class(module, class_name)?;
    let impl_class = info
        .method_impl_classes
        .get(&key)
        .cloned()
        .unwrap_or_else(|| class_name.to_string());
    let want = crate::names::php_symbol_key(&format!("{impl_class}::jsonSerialize"));
    let body = module
        .class_methods
        .iter()
        .find(|f| crate::names::php_symbol_key(&f.name) == want)?;
    let dynamic = info.vtable_slots.contains_key(&key) && !info.final_methods.contains(&key);
    Some((body.name.clone(), dynamic))
}

/// Lays out the per-class public-property metadata and declares its globals.
///
/// Same shape as the gc-descriptor table: one row per class id, reached through a pointer table,
/// so the encoder resolves a class in two loads. Returns the new data cursor.
pub(super) fn emit_json_property_table(
    wm: &mut WatModule,
    module: &Module,
    mut cursor: u32,
) -> u32 {
    cursor = (cursor + 3) & !3;
    let class_infos = &module.class_infos;
    if class_infos.is_empty() {
        for name in ["__json_prop_ptrs", "__json_prop_counts", "__json_prop_count"] {
            wm.add_global(Global {
                name: name.to_string(),
                ty: ValType::I32,
                mutable: false,
                init: 0,
            });
        }
        return cursor;
    }
    let mut ordered: Vec<_> = class_infos.iter().collect();
    ordered.sort_by(|(left, left_ci), (right, right_ci)| {
        left_ci.class_id.cmp(&right_ci.class_id).then_with(|| left.cmp(right))
    });
    let max_id = ordered.last().map(|(_, ci)| ci.class_id).unwrap_or(0);
    let count = max_id + 1;
    let mut id_to_name: HashMap<u64, &str> = HashMap::new();
    for (name, ci) in &ordered {
        id_to_name.entry(ci.class_id).or_insert(name.as_str());
    }

    // The property NAMES, laid out once each and shared by every row that mentions them.
    let mut name_offsets: HashMap<String, (u32, u32)> = HashMap::new();
    for (_, ci) in &ordered {
        for (name, _) in public_properties(ci) {
            if !name_offsets.contains_key(&name) {
                let bytes = name.as_bytes().to_vec();
                let len = bytes.len() as u32;
                wm.add_data(DataSegment { offset: cursor, bytes });
                name_offsets.insert(name.clone(), (cursor, len));
                cursor += len;
            }
        }
    }

    // One 16-byte entry per public property: name pointer, name length, slot offset, tag.
    cursor = (cursor + 3) & !3;
    let mut row_off: HashMap<u64, u32> = HashMap::new();
    let mut row_len: HashMap<u64, u32> = HashMap::new();
    for cid in 0..=max_id {
        let Some(name) = id_to_name.get(&cid) else {
            continue;
        };
        let Some(ci) = class_infos.get(*name) else {
            continue;
        };
        let properties: Vec<JsonProperty> = public_properties(ci)
            .into_iter()
            .filter_map(|(property, ty)| {
                let index = ci.properties.iter().position(|(p, _)| *p == property)?;
                let offset = ci
                    .property_offsets
                    .get(&property)
                    .copied()
                    .unwrap_or(8 + index * 16) as u32;
                Some(JsonProperty {
                    name: property,
                    offset,
                    tag: json_property_tag(&ty)?,
                })
            })
            .collect();
        let mut bytes = Vec::with_capacity(properties.len() * 16);
        for property in &properties {
            let (name_ptr, name_len) = name_offsets[&property.name];
            bytes.extend_from_slice(&name_ptr.to_le_bytes());
            bytes.extend_from_slice(&name_len.to_le_bytes());
            bytes.extend_from_slice(&property.offset.to_le_bytes());
            bytes.extend_from_slice(&property.tag.to_le_bytes());
        }
        row_len.insert(cid, properties.len() as u32);
        if bytes.is_empty() {
            row_off.insert(cid, cursor);
            continue;
        }
        row_off.insert(cid, cursor);
        let written = bytes.len() as u32;
        wm.add_data(DataSegment { offset: cursor, bytes });
        cursor += written;
    }

    cursor = (cursor + 3) & !3;
    let ptrs_off = cursor;
    let mut ptrs = Vec::with_capacity(count as usize * 4);
    for cid in 0..=max_id {
        ptrs.extend_from_slice(&row_off.get(&cid).copied().unwrap_or(0).to_le_bytes());
    }
    wm.add_data(DataSegment { offset: ptrs_off, bytes: ptrs });
    cursor += count as u32 * 4;

    let counts_off = cursor;
    let mut counts = Vec::with_capacity(count as usize * 4);
    for cid in 0..=max_id {
        counts.extend_from_slice(&row_len.get(&cid).copied().unwrap_or(0).to_le_bytes());
    }
    wm.add_data(DataSegment { offset: counts_off, bytes: counts });
    cursor += count as u32 * 4;

    wm.add_global(Global {
        name: "__json_prop_ptrs".to_string(),
        ty: ValType::I32,
        mutable: false,
        init: ptrs_off as i64,
    });
    wm.add_global(Global {
        name: "__json_prop_counts".to_string(),
        ty: ValType::I32,
        mutable: false,
        init: counts_off as i64,
    });
    wm.add_global(Global {
        name: "__json_prop_count".to_string(),
        ty: ValType::I32,
        mutable: false,
        init: count as i64,
    });
    cursor
}

/// The `(class_id, callee_symbol)` arms of the `jsonSerialize` ladder.
///
/// The dispatch decision is the one an ordinary method call makes — a method with a vtable slot
/// that is not final goes through its introducer's stub, so an overriding subclass still wins —
/// because a different rule here would answer with a different body than `$obj->jsonSerialize()`
/// would.
pub(super) fn jsonserialize_ladder(module: &Module) -> Vec<(u64, String)> {
    let key = crate::names::php_symbol_key("jsonSerialize");
    let mut arms = Vec::new();
    let mut classes: Vec<_> = module.class_infos.iter().collect();
    classes.sort_by(|left, right| left.1.class_id.cmp(&right.1.class_id));
    for (class_name, info) in classes {
        if !implements_json_serializable(module, class_name) {
            continue;
        }
        let Some((body_name, dynamic)) = jsonserialize_target(module, class_name) else {
            continue;
        };
        let symbol = if dynamic {
            let introducer = info
                .method_declaring_classes
                .get(&key)
                .cloned()
                .unwrap_or_else(|| class_name.clone());
            super::symbols::method_dispatch_symbol(&introducer, &key)
        } else {
            super::symbols::method_symbol(&body_name)
        };
        arms.push((info.class_id, symbol));
    }
    arms
}

/// Whether this module calls `json_encode`.
pub(super) fn module_uses_json_encode(module: &Module) -> bool {
    use crate::ir::{Immediate, RuntimeCallTarget, RuntimeFnId};
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .any(|function| {
            function.instructions.iter().any(|inst| {
                let Some(Immediate::RuntimeCall(target)) = inst.immediate.as_ref() else {
                    return false;
                };
                matches!(
                    target,
                    RuntimeCallTarget::Function(RuntimeFnId::JsonEncode)
                        | RuntimeCallTarget::ProfiledFunction {
                            target: RuntimeFnId::JsonEncode,
                            ..
                        }
                )
            })
        })
}

/// Adds the encoder.
///
/// `serialize_targets` are the `(class_id, callee_symbol)` arms of the `jsonSerialize` ladder,
/// which is what lets an object answer through its own method. The ladder is a class-id if-chain
/// for the same reason every other dynamic dispatch here is one: this backend has no `funcref`
/// table to index.
pub(super) fn emit_json_runtime(wm: &mut WatModule, serialize_targets: &[(u64, String)]) {
    for (name, init) in [("__json_buf", 0), ("__json_len", 0), ("__json_cap", 0)] {
        wm.add_global(Global {
            name: name.to_string(),
            ty: ValType::I32,
            mutable: true,
            init,
        });
    }
    wm.add_raw_func(RT_JSON_PUT);
    wm.add_raw_func(RT_JSON_PUT_BYTE);
    wm.add_raw_func(RT_JSON_PUT_HEX4);
    wm.add_raw_func(RT_JSON_PUT_STRING);
    wm.add_raw_func(&rt_json_serialize_call(serialize_targets));
    wm.add_raw_func(RT_JSON_OBJECT);
    wm.add_raw_func(RT_JSON_VALUE);
    wm.add_raw_func(RT_JSON_ENCODE);
}

/// `__rt_json_put`: appends bytes to the encode buffer, doubling it when full.
const RT_JSON_PUT: &str = r#"(func $__rt_json_put (param $ptr i32) (param $len i32)
  (local $need i32) (local $grown i32) (local $i i32)
  (if (i32.eqz (local.get $len))
    (then (return)))
  (local.set $need (i32.add (global.get $__json_len) (local.get $len)))
  (if (i32.gt_u (local.get $need) (global.get $__json_cap))
    (then
      (global.set $__json_cap (i32.mul (i32.add (global.get $__json_cap) (i32.const 64)) (i32.const 2)))
      (if (i32.gt_u (local.get $need) (global.get $__json_cap))
        (then (global.set $__json_cap (local.get $need))))
      (local.set $grown (call $__rt_heap_alloc (global.get $__json_cap)))
      (block $copied (loop $byte
        (br_if $copied (i32.ge_u (local.get $i) (global.get $__json_len)))
        (i32.store8 (i32.add (local.get $grown) (local.get $i))
                    (i32.load8_u (i32.add (global.get $__json_buf) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $byte)))
      (if (global.get $__json_buf)
        (then (call $__rt_heap_free (global.get $__json_buf))))
      (global.set $__json_buf (local.get $grown))))
  (local.set $i (i32.const 0))
  (block $written (loop $byte
    (br_if $written (i32.ge_u (local.get $i) (local.get $len)))
    (i32.store8 (i32.add (global.get $__json_buf) (i32.add (global.get $__json_len) (local.get $i)))
                (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $byte)))
  (global.set $__json_len (local.get $need)))
"#;

/// `__rt_json_put_byte`: one literal byte, written through the float scratch.
const RT_JSON_PUT_BYTE: &str = r#"(func $__rt_json_put_byte (param $byte i32)
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const 3900)) (local.get $byte))
  (call $__rt_json_put (i32.add (global.get $__float_scratch) (i32.const 3900)) (i32.const 1)))
"#;

/// `__rt_json_put_hex4`: `\uXXXX` for one code unit, lower-case hex as php writes it.
const RT_JSON_PUT_HEX4: &str = r#"(func $__rt_json_put_hex4 (param $unit i32)
  (local $i i32) (local $nibble i32)
  (call $__rt_json_put_byte (i32.const 92))                        ;; '\'
  (call $__rt_json_put_byte (i32.const 117))                       ;; 'u'
  (local.set $i (i32.const 3))
  (block $done (loop $digit
    (br_if $done (i32.lt_s (local.get $i) (i32.const 0)))
    (local.set $nibble (i32.and (i32.shr_u (local.get $unit) (i32.mul (local.get $i) (i32.const 4))) (i32.const 15)))
    (call $__rt_json_put_byte
      (select (i32.add (local.get $nibble) (i32.const 87))          ;; 'a'..'f'
              (i32.add (local.get $nibble) (i32.const 48))          ;; '0'..'9'
              (i32.gt_u (local.get $nibble) (i32.const 9))))
    (local.set $i (i32.sub (local.get $i) (i32.const 1)))
    (br $digit))))
"#;

/// `__rt_json_put_string`: a quoted, escaped JSON string.
///
/// php's DEFAULT flags: the short forms for `"` `\` `/` and the five named control characters,
/// `\u00XX` for the rest below 0x20, and `\uXXXX` for everything above 0x7f — decoded from UTF-8,
/// with a surrogate pair beyond U+FFFF. A byte that cannot start a valid sequence is emitted as
/// `�`, which is what php does under `JSON_INVALID_UTF8_SUBSTITUTE`; without it php answers
/// false, a divergence this cannot reach for the ASCII this target admits.
const RT_JSON_PUT_STRING: &str = r#"(func $__rt_json_put_string (param $ptr i32) (param $len i64)
  (local $i i32) (local $n i32) (local $b i32) (local $cp i32) (local $extra i32) (local $j i32)
  (local $cont i32)
  (local.set $n (i32.wrap_i64 (local.get $len)))
  (call $__rt_json_put_byte (i32.const 34))                        ;; '"'
  (block $done (loop $byte
    (br_if $done (i32.ge_u (local.get $i) (local.get $n)))
    (local.set $b (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
    (block $emitted
      ;; The short forms, in the order php writes them.
      (if (i32.eq (local.get $b) (i32.const 34))                   ;; '"'
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 34)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 92))                   ;; '\'
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 92)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 47))                   ;; '/'
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 47)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 8))                    ;; backspace
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 98)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 12))                   ;; form feed
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 102)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 10))                   ;; newline
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 110)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 13))                   ;; carriage return
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 114)) (br $emitted)))
      (if (i32.eq (local.get $b) (i32.const 9))                    ;; tab
        (then (call $__rt_json_put_byte (i32.const 92))
              (call $__rt_json_put_byte (i32.const 116)) (br $emitted)))
      (if (i32.lt_u (local.get $b) (i32.const 32))                 ;; other control byte
        (then (call $__rt_json_put_hex4 (local.get $b)) (br $emitted)))
      (if (i32.lt_u (local.get $b) (i32.const 128))                ;; plain ASCII
        (then (call $__rt_json_put_byte (local.get $b)) (br $emitted)))
      ;; A UTF-8 sequence: its length comes from the lead byte, then the continuation bytes are
      ;; gathered and the code point re-emitted as escaped UTF-16 — a surrogate pair above
      ;; U+FFFF, which is what php writes without JSON_UNESCAPED_UNICODE.
      (local.set $extra (i32.const 0))
      (local.set $cp (i32.const 65533))                            ;; U+FFFD until proven otherwise
      (if (i32.eq (i32.and (local.get $b) (i32.const 224)) (i32.const 192))
        (then (local.set $extra (i32.const 1))
              (local.set $cp (i32.and (local.get $b) (i32.const 31)))))
      (if (i32.eq (i32.and (local.get $b) (i32.const 240)) (i32.const 224))
        (then (local.set $extra (i32.const 2))
              (local.set $cp (i32.and (local.get $b) (i32.const 15)))))
      (if (i32.eq (i32.and (local.get $b) (i32.const 248)) (i32.const 240))
        (then (local.set $extra (i32.const 3))
              (local.set $cp (i32.and (local.get $b) (i32.const 7)))))
      (if (i32.gt_u (i32.add (i32.add (local.get $i) (local.get $extra)) (i32.const 1)) (local.get $n))
        (then (local.set $extra (i32.const 0)) (local.set $cp (i32.const 65533))))
      (local.set $j (i32.const 0))
      (block $gathered (loop $cbyte
        (br_if $gathered (i32.ge_u (local.get $j) (local.get $extra)))
        (local.set $cont (i32.load8_u
          (i32.add (local.get $ptr) (i32.add (i32.add (local.get $i) (local.get $j)) (i32.const 1)))))
        (if (i32.ne (i32.and (local.get $cont) (i32.const 192)) (i32.const 128))
          (then (local.set $cp (i32.const 65533)) (local.set $extra (i32.const 0)) (br $gathered)))
        (local.set $cp (i32.or (i32.shl (local.get $cp) (i32.const 6))
                               (i32.and (local.get $cont) (i32.const 63))))
        (local.set $j (i32.add (local.get $j) (i32.const 1)))
        (br $cbyte)))
      (local.set $i (i32.add (local.get $i) (local.get $extra)))
      (if (i32.gt_u (local.get $cp) (i32.const 65535))
        (then
          (local.set $cp (i32.sub (local.get $cp) (i32.const 65536)))
          (call $__rt_json_put_hex4
            (i32.add (i32.const 55296) (i32.shr_u (local.get $cp) (i32.const 10))))
          (call $__rt_json_put_hex4
            (i32.add (i32.const 56320) (i32.and (local.get $cp) (i32.const 1023)))))
        (else (call $__rt_json_put_hex4 (local.get $cp)))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $byte)))
  (call $__rt_json_put_byte (i32.const 34)))
"#;

/// `__rt_json_serialize_call`: dispatches an object to its own `jsonSerialize`.
///
/// Answers 0 for a class that does not implement `JsonSerializable`, which is how the value
/// encoder chooses between the method and the property walk.
fn rt_json_serialize_call(targets: &[(u64, String)]) -> String {
    let mut wat = String::from(
        "(func $__rt_json_serialize_call (param $obj i32) (result i32)\n  (local $cid i32)\n",
    );
    if targets.is_empty() {
        wat.push_str("  (i32.const 0))\n");
        return wat;
    }
    wat.push_str("  (local.set $cid (i32.wrap_i64 (i64.load (local.get $obj))))\n");
    for (class_id, callee) in targets {
        wat.push_str(&format!(
            "  (if (i32.eq (local.get $cid) (i32.const {class_id}))\n    (then (return (call ${callee} (local.get $obj)))))\n"
        ));
    }
    wat.push_str("  (i32.const 0))\n");
    wat
}

/// `__rt_json_object`: an object as a JSON object.
///
/// A `JsonSerializable` answers through its method and the RESULT is encoded; any other class
/// has its public properties walked in declaration order, each slot paired with the tag its
/// class table records and boxed so the value encoder sees one uniform shape.
const RT_JSON_OBJECT: &str = r#"(func $__rt_json_object (param $obj i32)
  (local $cid i32) (local $row i32) (local $n i32) (local $i i32) (local $entry i32)
  (local $tag i32) (local $slot i32) (local $cell i32) (local $res i32)
  (local.set $res (call $__rt_json_serialize_call (local.get $obj)))
  (if (local.get $res)
    (then
      (call $__rt_json_value (local.get $res))
      (call $__rt_decref_any (local.get $res))
      (return)))
  (local.set $cid (i32.wrap_i64 (i64.load (local.get $obj))))
  (call $__rt_json_put_byte (i32.const 123))                       ;; '{'
  (if (i32.lt_u (local.get $cid) (global.get $__json_prop_count))
    (then
      (local.set $row (i32.load (i32.add (global.get $__json_prop_ptrs) (i32.mul (local.get $cid) (i32.const 4)))))
      (local.set $n (i32.load (i32.add (global.get $__json_prop_counts) (i32.mul (local.get $cid) (i32.const 4)))))
      (block $walked (loop $property
        (br_if $walked (i32.ge_u (local.get $i) (local.get $n)))
        (if (local.get $i)
          (then (call $__rt_json_put_byte (i32.const 44))))        ;; ','
        (local.set $entry (i32.add (local.get $row) (i32.mul (local.get $i) (i32.const 16))))
        (call $__rt_json_put_string (i32.load (local.get $entry))
                                    (i64.extend_i32_u (i32.load offset=4 (local.get $entry))))
        (call $__rt_json_put_byte (i32.const 58))                  ;; ':'
        (local.set $slot (i32.add (local.get $obj) (i32.load offset=8 (local.get $entry))))
        (local.set $tag (i32.load offset=12 (local.get $entry)))
        (if (i32.eq (local.get $tag) (i32.const 7))
          (then
            ;; The slot already holds a boxed cell; the encoder reads it directly.
            (call $__rt_json_value (i32.wrap_i64 (i64.load (local.get $slot)))))
          (else
            (local.set $cell (call $__rt_mixed_from_value (i64.extend_i32_u (local.get $tag))
              (i64.load (local.get $slot)) (i64.load offset=8 (local.get $slot))))
            (call $__rt_json_value (local.get $cell))
            (call $__rt_decref_any (local.get $cell))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $property)))))
  (call $__rt_json_put_byte (i32.const 125)))                      ;; '}'
"#;

/// `__rt_json_value`: one value, appended to the encode buffer.
///
/// An INDEXED array becomes a JSON array and a HASH a JSON object, which is php's rule: the
/// storage decides, and a hash key is written as a string whichever kind it is.
const RT_JSON_VALUE: &str = r#"(func $__rt_json_value (param $cell i32)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $tptr i32) (local $tlen i32)
  (local $src i32) (local $kind i32) (local $cursor i64) (local $more i64) (local $next i64)
  (local $kcell i32) (local $vcell i32) (local $first i32)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eqz (local.get $tag))                                   ;; int
    (then
      (call $__rt_itoa (local.get $lo) (global.get $__float_scratch))
      (local.set $tlen)
      (local.set $tptr)
      (call $__rt_json_put (local.get $tptr) (local.get $tlen))
      (return)))
  (if (i64.eq (local.get $tag) (i64.const 1))                      ;; string
    (then
      (call $__rt_json_put_string (i32.wrap_i64 (local.get $lo)) (local.get $hi))
      (return)))
  (if (i64.eq (local.get $tag) (i64.const 3))                      ;; bool
    (then
      (if (i64.eqz (local.get $lo))
        (then (call $__rt_json_put (i32.add (global.get $__float_scratch) (i32.const 3910)) (i32.const 5)))
        (else (call $__rt_json_put (i32.add (global.get $__float_scratch) (i32.const 3920)) (i32.const 4))))
      (return)))
  (if (i64.eq (local.get $tag) (i64.const 6))                      ;; object
    (then
      (call $__rt_json_object (i32.wrap_i64 (local.get $lo)))
      (return)))
  (if (i32.or (i64.eq (local.get $tag) (i64.const 4))
              (i64.eq (local.get $tag) (i64.const 5)))
    (then
      (local.set $src (i32.wrap_i64 (local.get $lo)))
      (local.set $kind (select (i32.const 1) (i32.const 0) (i64.eq (local.get $tag) (i64.const 5))))
      (call $__rt_json_put_byte (select (i32.const 123) (i32.const 91) (local.get $kind)))
      (local.set $cursor (select (i64.const -2) (i64.const -1) (i32.eq (local.get $kind) (i32.const 1))))
      (block $walked (loop $entry
        (call $__rt_mixed_iter_next (local.get $src) (local.get $cursor) (local.get $kind))
        (local.set $more)
        (local.set $next)
        (br_if $walked (i64.eqz (local.get $more)))
        (local.set $cursor (local.get $next))
        (if (local.get $first)
          (then (call $__rt_json_put_byte (i32.const 44))))
        (local.set $first (i32.const 1))
        (if (i32.eq (local.get $kind) (i32.const 1))
          (then
            (local.set $kcell (call $__rt_mixed_iter_key (local.get $src) (local.get $cursor) (local.get $kind)))
            (call $__rt_mixed_unbox (local.get $kcell))
            (local.set $hi)
            (local.set $lo)
            (local.set $tag)
            (if (i64.eq (local.get $tag) (i64.const 1))
              (then (call $__rt_json_put_string (i32.wrap_i64 (local.get $lo)) (local.get $hi)))
              (else
                ;; An integer key is written as a STRING: a JSON object has no other kind.
                (call $__rt_itoa (local.get $lo) (global.get $__float_scratch))
                (local.set $tlen)
                (local.set $tptr)
                (call $__rt_json_put_string (local.get $tptr) (i64.extend_i32_u (local.get $tlen)))))
            (call $__rt_decref_any (local.get $kcell))
            (call $__rt_json_put_byte (i32.const 58))))
        (local.set $vcell (call $__rt_mixed_iter_value (local.get $src) (local.get $cursor) (local.get $kind)))
        (call $__rt_json_value (local.get $vcell))
        (call $__rt_decref_any (local.get $vcell))
        (br $entry)))
      (call $__rt_json_put_byte (select (i32.const 125) (i32.const 93) (local.get $kind)))
      (return)))
  ;; null, and anything the audit refused: php writes null.
  (call $__rt_json_put (i32.add (global.get $__float_scratch) (i32.const 3930)) (i32.const 4)))
"#;

/// `__rt_json_encode`: the builtin, boxed as php's `string|false`.
///
/// The literals `true`, `false` and `null` are written into the scratch here rather than laid
/// out as data, because the encoder is the only reader and three fixed words do not justify
/// moving the whole region's offsets.
const RT_JSON_ENCODE: &str = r#"(func $__rt_json_encode (param $cell i32) (result i32)
  (local $ptr i32) (local $len i64) (local $out i32)
  (i32.store8 offset=3910 (global.get $__float_scratch) (i32.const 102))   ;; 'f'
  (i32.store8 offset=3911 (global.get $__float_scratch) (i32.const 97))    ;; 'a'
  (i32.store8 offset=3912 (global.get $__float_scratch) (i32.const 108))   ;; 'l'
  (i32.store8 offset=3913 (global.get $__float_scratch) (i32.const 115))   ;; 's'
  (i32.store8 offset=3914 (global.get $__float_scratch) (i32.const 101))   ;; 'e'
  (i32.store8 offset=3920 (global.get $__float_scratch) (i32.const 116))   ;; 't'
  (i32.store8 offset=3921 (global.get $__float_scratch) (i32.const 114))   ;; 'r'
  (i32.store8 offset=3922 (global.get $__float_scratch) (i32.const 117))   ;; 'u'
  (i32.store8 offset=3923 (global.get $__float_scratch) (i32.const 101))   ;; 'e'
  (i32.store8 offset=3930 (global.get $__float_scratch) (i32.const 110))   ;; 'n'
  (i32.store8 offset=3931 (global.get $__float_scratch) (i32.const 117))   ;; 'u'
  (i32.store8 offset=3932 (global.get $__float_scratch) (i32.const 108))   ;; 'l'
  (i32.store8 offset=3933 (global.get $__float_scratch) (i32.const 108))   ;; 'l'
  (global.set $__json_len (i32.const 0))
  (call $__rt_json_value (local.get $cell))
  (call $__rt_str_persist (global.get $__json_buf) (i64.extend_i32_u (global.get $__json_len)))
  (local.set $len)
  (local.set $ptr)
  (local.set $out (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (local.get $ptr)) (local.get $len)))
  (local.get $out))
"#;

//! Purpose:
//! Computes and emits the closed-world flat method/property registry tables backing
//! dynamic `new ReflectionMethod($class, $method)` / `new ReflectionProperty($class,
//! $property)` construction and `ReflectionClass::getMethod()`/`getProperty()`.
//!
//! Called from:
//! - `crate::codegen_ir::mod::finalize_user_asm()` when
//!   `crate::codegen_ir::lower_inst::objects::reflection_members::
//!   module_needs_reflection_member_dynamic_dispatch()` is true.
//!
//! Key details:
//! - FLATTEN ALGORITHM: one row is emitted per class C (from `module.class_infos`,
//!   excluding internal synthetic helper classes) for every key already present in
//!   `C.methods`/`C.static_methods` (methods) or `C.properties`/`C.static_properties`
//!   (properties) — these maps are ALREADY flattened by the type checker's schema
//!   builder (`crate::types::checker::schema::classes`): a subclass that does not
//!   override a member keeps the SAME map entry the parent declared (same
//!   visibility/declaring-class), and a subclass that DOES override (including a
//!   private member shadowing a same-named private parent member) replaces the
//!   entry outright — matching PHP's real single-slot-per-name resolution (php -n
//!   verified: `(new ReflectionClass(B::class))->getMethod('priv')` finds `B`'s own
//!   private `priv` when `B` shadows it, or `A`'s private `priv` via
//!   `method_declaring_classes` when `B` does not shadow it at all — both cases are
//!   just ONE map entry already, so no additional inheritance walk is needed here).
//!   Trait methods/properties are already merged into the using class's own maps by
//!   trait flattening before the checker ever builds `ClassInfo`, so they need no
//!   special handling either (php -n verified: `getDeclaringClass()` for a
//!   trait-provided member reports the USING class, never the trait).
//! - K1: `getMethods()`/`getProperties()` ENUMERATION (php -n verified: a subclass
//!   omits an inherited-but-not-overridden PARENT-PRIVATE method/property, while
//!   `getMethod()`/a direct `new ReflectionMethod($sub, $name)` construction still
//!   FINDS it) is implemented, but NOT by re-walking this table at runtime:
//!   `crate::codegen_ir::lower_inst::objects::reflection::reflection_class_extra_metadata`
//!   bakes an already-ordered, already-filtered name array
//!   (`ReflectionClass::__methods_ordered`/`__properties_ordered`) at `ReflectionClass`
//!   CONSTRUCTION time using `method_decl_order_and_names`/`property_decl_order_and_names`
//!   below directly against `ClassInfo` — the SAME closed-world-switch trick the existing
//!   dynamic `ReflectionClass($runtimeName)` dispatcher already uses (every dispatch
//!   case is a compile-time-known class name, see `emit_reflection_class_dynamic_construct`)
//!   means EVERY construction path, literal or dynamic, resolves to a compile-time-known
//!   `reflected_class: &str` by the time the metadata bake runs, so no runtime table walk
//!   is needed for this consumer. `getMethods()`/`getProperties()` are then a plain PHP-level
//!   loop (`crate::types::checker::builtin_types::reflection`) over that baked array,
//!   constructing one shell per visible name through the EXISTING dynamic
//!   `ReflectionMethod`/`ReflectionProperty` dispatcher below — no new runtime assembly.
//!   `declaring_class_id`/`decl_order` are still carried on every ROW regardless (Jury
//!   Addendum #1/#3: future point-lookup-adjacent consumers may want them; kept for the ABI
//!   layout's own documentation value even though today's enumeration consumer computes its
//!   own order independently via the same Rust function, not by reading these row fields).
//! - Row layouts (see `METHOD_ROW_SIZE`/`PROPERTY_ROW_SIZE`/`INDEX_ROW_SIZE`/
//!   `CLASS_ID_ROW_SIZE` below) all start with an 8-byte-aligned `{name_ptr,
//!   name_len}` pair so `__rt_sorted_name_search` (generic, entry-size-parameterized)
//!   can binary search any of them unmodified.
//! - Method rows store TWO name strings: `name` (ASCII-lowercased, the search/sort
//!   key — methods are case-insensitive) and `real_name` (the exact declared
//!   spelling, used to bake `ReflectionMethod::getName()`). Property rows store only
//!   ONE exact-case name (properties are case-SENSITIVE in PHP, so the search key
//!   and the display spelling are the same bytes).
//! - Per-class segments are located in O(1): `_reflect_method_index`/
//!   `_reflect_property_index` are dense arrays indexed DIRECTLY by `class_id`
//!   (`crate::types::schema::ClassInfo::class_id` is a dense `0..N` counter, see
//!   `crate::types::checker::driver::mod`), each entry `{start, count}` bounding a
//!   contiguous, per-class-sorted slice of the flat row table. Resolving a class
//!   NAME to its `class_id` at runtime goes through a THIRD small table,
//!   `_reflect_class_id_table` (name-sorted, `__rt_sorted_name_search`-compatible),
//!   since `_class_table` (`const_registry.rs`) does not carry `class_id` and is a
//!   different feature's table (existence-only, no class_id/method/property payload).
//! - String bytes ARE deduplicated across rows (unlike `const_registry.rs`'s
//!   `_class_table`/`_interface_table`/`_trait_table`, which never repeat a class
//!   name): a local content-keyed label cache collapses repeated method/property
//!   names (`__construct`, `getId`, `get`, `set`, …) that legitimately recur across
//!   hundreds of classes, keeping the emitted `.data` size close to the row-array
//!   size instead of also scaling with total (row-count × average-name-length).

use std::collections::{HashMap, HashSet};

use crate::ir::Module;
use crate::names::php_symbol_key;
use crate::parser::ast::{ClassMethod, Visibility};
use crate::types::ClassInfo;

use super::instanceof::escaped_ascii;

/// Byte size of one `_reflect_class_id_table` entry: `{name_ptr:8, name_len:8, class_id:8}`.
pub(crate) const CLASS_ID_ROW_SIZE: usize = 24;
/// Byte size of one `_reflect_method_table` entry: `{name_ptr:8, name_len:8, real_name_ptr:8,
/// real_name_len:8, modifiers:4, declaring_class_id:4, decl_order:4, _pad:4}` — the trailing 4
/// bytes are unused padding, kept only to hold the row size at an 8-byte multiple (`.p2align 3`
/// covers the TABLE start, not each individual row) now that `decl_order` (Jury Addendum #1)
/// pushed the natural size to 44.
pub(crate) const METHOD_ROW_SIZE: usize = 48;
/// Byte size of one `_reflect_property_table` entry: `{name_ptr:8, name_len:8, modifiers:4,
/// declaring_class_id:4, decl_order:4, _pad:4}` — see `METHOD_ROW_SIZE`'s padding note.
pub(crate) const PROPERTY_ROW_SIZE: usize = 32;
/// Byte size of one `_reflect_method_index`/`_reflect_property_index` entry: `{start:8, count:8}`.
pub(crate) const INDEX_ROW_SIZE: usize = 16;

/// Byte offset of `real_name_ptr` within a method row.
pub(crate) const METHOD_ROW_REAL_NAME_PTR_OFFSET: usize = 16;
/// Byte offset of `real_name_len` within a method row.
pub(crate) const METHOD_ROW_REAL_NAME_LEN_OFFSET: usize = 24;
/// Byte offset of `modifiers` within a method row.
pub(crate) const METHOD_ROW_MODIFIERS_OFFSET: usize = 32;
/// Byte offset of `declaring_class_id` within a method row. Not yet read by any dispatcher (see
/// the file-level doc comment: enumeration is a documented future consumer); kept for the ABI
/// layout's own documentation value.
#[allow(dead_code)]
pub(crate) const METHOD_ROW_DECLARING_CLASS_ID_OFFSET: usize = 36;
/// Byte offset of `decl_order` within a method row (Jury Addendum #1). Not yet read by any
/// dispatcher — `ReflectionClass::getMethods()` consumes the SAME order via
/// `ReflectionClassExtraMetadata::methods_ordered`, computed directly from
/// `method_decl_order_and_names` at compile time rather than by reading this row field back out
/// of the emitted table (see `crate::codegen_ir::lower_inst::objects::reflection`); kept for the
/// ABI layout's own documentation/future-consumer value, matching `METHOD_ROW_DECLARING_CLASS_ID_OFFSET`.
#[allow(dead_code)]
pub(crate) const METHOD_ROW_DECL_ORDER_OFFSET: usize = 40;
/// Byte offset of `modifiers` within a property row.
pub(crate) const PROPERTY_ROW_MODIFIERS_OFFSET: usize = 16;
/// Byte offset of `declaring_class_id` within a property row. Not yet read (see
/// `METHOD_ROW_DECLARING_CLASS_ID_OFFSET`).
#[allow(dead_code)]
pub(crate) const PROPERTY_ROW_DECLARING_CLASS_ID_OFFSET: usize = 20;
/// Byte offset of `decl_order` within a property row. Not yet read (see
/// `METHOD_ROW_DECL_ORDER_OFFSET`).
#[allow(dead_code)]
pub(crate) const PROPERTY_ROW_DECL_ORDER_OFFSET: usize = 24;
/// Byte offset of `class_id` within a `_reflect_class_id_table` entry.
pub(crate) const CLASS_ID_ROW_CLASS_ID_OFFSET: usize = 16;
/// Byte offset of `count` within an index-table entry. Not yet read via this named constant (the
/// dispatcher reads it with a raw `+8` immediate offset); kept for the ABI layout's documentation.
#[allow(dead_code)]
pub(crate) const INDEX_ROW_COUNT_OFFSET: usize = 8;

// Jury Addendum #2: assert the manual row-layout byte math above against the actual field
// widths (`ptr`/`len` fields are 8-byte quads, `modifiers`/`declaring_class_id`/`decl_order`/
// padding are 4-byte longs) instead of trusting the doc comments alone.
const _: () = assert!(METHOD_ROW_SIZE == 8 + 8 + 8 + 8 + 4 + 4 + 4 + 4, "METHOD_ROW_SIZE must match name_ptr+name_len+real_name_ptr+real_name_len+modifiers+declaring_class_id+decl_order+pad");
const _: () = assert!(METHOD_ROW_SIZE % 8 == 0, "METHOD_ROW_SIZE must stay an 8-byte multiple so consecutive rows keep their leading pointer fields naturally aligned");
const _: () = assert!(PROPERTY_ROW_SIZE == 8 + 8 + 4 + 4 + 4 + 4, "PROPERTY_ROW_SIZE must match name_ptr+name_len+modifiers+declaring_class_id+decl_order+pad");
const _: () = assert!(PROPERTY_ROW_SIZE % 8 == 0, "PROPERTY_ROW_SIZE must stay an 8-byte multiple so consecutive rows keep their leading pointer fields naturally aligned");
const _: () = assert!(METHOD_ROW_DECL_ORDER_OFFSET + 4 <= METHOD_ROW_SIZE, "decl_order must fit inside the method row");
const _: () = assert!(PROPERTY_ROW_DECL_ORDER_OFFSET + 4 <= PROPERTY_ROW_SIZE, "decl_order must fit inside the property row");

/// One flattened method row before layout/emission.
struct MethodRow {
    /// ASCII-lowercased search key (`php_symbol_key` of the declared name).
    search_name: String,
    /// Exact declared spelling, used to bake `getName()`.
    real_name: String,
    modifiers: u32,
    declaring_class_id: u32,
    /// PHP `getMethods()` declaration-order position among the RECEIVING class's (`class_name`
    /// in `class_method_rows`) own visible rows — see `method_decl_order_and_names`. Carried on
    /// every row per Jury Addendum #1 even though no dispatcher currently reads it back out of
    /// the emitted table (`ReflectionClass::getMethods()`'s enumeration instead consumes the
    /// SAME order via `ReflectionClassExtraMetadata::methods_ordered`, computed by the same
    /// function at compile time — see `crate::codegen_ir::lower_inst::objects::reflection`) —
    /// kept for the ABI layout's own documentation/future-consumer value, matching this file's
    /// existing `declaring_class_id` precedent.
    decl_order: u32,
}

/// One flattened property row before layout/emission.
struct PropertyRow {
    /// Exact-case declared name (properties are case-sensitive).
    name: String,
    modifiers: u32,
    declaring_class_id: u32,
    /// PHP `getProperties()` declaration-order position — see `MethodRow::decl_order` and
    /// `property_decl_order_and_names`.
    decl_order: u32,
}

/// Measured byte totals for the emitted tables, returned so callers can report the
/// size gate (`crate::codegen_ir::lower_inst::objects::reflection_members`'s
/// `--web` size-report contract; see the J4 spec's `<500KB` acceptance gate).
/// Diagnostic-only: no production code path currently branches on these fields (the
/// `<500KB` gate is verified out-of-band by measuring a real compile, not by an in-process
/// assertion), so they are intentionally allowed to go unread by `cargo build`.
#[allow(dead_code)]
pub(crate) struct ReflectMemberRegistrySizes {
    pub(crate) method_row_count: usize,
    pub(crate) property_row_count: usize,
    pub(crate) method_table_bytes: usize,
    pub(crate) property_table_bytes: usize,
    pub(crate) total_bytes: usize,
}

/// Returns true for internal helper classes hidden from PHP-visible reflection.
/// Mirrors `crate::codegen_ir::is_internal_synthetic_class_name`.
fn is_internal_synthetic_class_name(name: &str) -> bool {
    php_symbol_key(name).starts_with("__elephc")
}

/// Computes the real PHP `IS_*` bitmask for a declared method (php -n verified:
/// `IS_STATIC=16, IS_PUBLIC=1, IS_PROTECTED=2, IS_PRIVATE=4, IS_ABSTRACT=64, IS_FINAL=32`).
/// Duplicated in miniature from `crate::codegen_ir::lower_inst::objects::reflection::
/// method_modifiers_bitmask` (private to that module, and this file lives in a different
/// module tree) rather than widened visibility for one shared 8-line computation —
/// matches this directory's existing precedent (`class_relation_registry.rs`'s doc
/// comment: "a parallel implementation ... rather than a shared refactor").
fn method_modifiers_bitmask(decl: &ClassMethod) -> u32 {
    let mut bits = match decl.visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 4,
    };
    if decl.is_static {
        bits |= 16;
    }
    if decl.is_abstract {
        bits |= 64;
    }
    if decl.is_final {
        bits |= 32;
    }
    bits
}

/// Locates the `ClassMethod` AST node that declares `method_key`'s real spelling and
/// modifiers, walking to the actual declaring class via `method_declaring_classes`/
/// `static_method_declaring_classes`. Mirrors `crate::codegen_ir::lower_inst::objects::
/// reflection::find_method_decl`.
fn find_method_decl<'a>(module: &'a Module, class_name: &str, method_key: &str) -> Option<&'a ClassMethod> {
    let info = module.class_infos.get(class_name)?;
    let declaring_class = info
        .method_declaring_classes
        .get(method_key)
        .or_else(|| info.static_method_declaring_classes.get(method_key))
        .cloned()
        .unwrap_or_else(|| class_name.to_string());
    let declaring_info = module.class_infos.get(&declaring_class)?;
    declaring_info
        .method_decls
        .iter()
        .find(|decl| php_symbol_key(&decl.name) == method_key)
}

/// Walks `class_name`'s ancestor chain (itself, then each `parent`, …) and assigns PHP
/// declaration-order positions to every method key VISIBLE to `class_name` (own + inherited,
/// including an unoverridden PRIVATE ancestor member — `ClassInfo.methods` still carries a row
/// for it, see the file-level "FLATTEN ALGORITHM" doc comment) — php -n VERIFIED order:
/// `class_name`'s own declared methods first, in THEIR OWN source order, then each ancestor's
/// own declared methods appended (nearest ancestor first), skipping any key already claimed by
/// a more-derived level. An override therefore keeps the POSITION of the level that
/// (re)declares it, not the original ancestor's position (verified against a 3-level A/B/C
/// hierarchy with an override and a non-overriding leaf class — see the J4 spec's Part A).
///
/// Returns `(decl_order_by_key, ordered_real_names)`: `decl_order_by_key` has an entry for
/// EVERY visible key (`class_method_rows` bakes this into every row's `decl_order` field, per
/// Jury Addendum #1, even for a parent-private key `getMethods()` itself excludes below);
/// `ordered_real_names` is already filtered to the `getMethods()`-VISIBLE subset (Jury Addendum
/// #3: an ancestor-declared PRIVATE method is excluded from ENUMERATION independently of any
/// `$filter` bitmask, though `getMethod()`/dynamic-construction POINT LOOKUP still finds it —
/// php -n verified), in final display order, using each level's real declared spelling.
/// `crate::codegen_ir::lower_inst::objects::reflection::reflection_class_extra_metadata` bakes
/// `ordered_real_names` verbatim into `ReflectionClass::__methods_ordered`.
pub(crate) fn method_decl_order_and_names(module: &Module, class_name: &str) -> (HashMap<String, u32>, Vec<String>) {
    let mut order = HashMap::new();
    let mut ordered_names = Vec::new();
    let mut counter: u32 = 0;
    let mut current = Some(class_name.to_string());
    let mut visited = HashSet::new();
    while let Some(level_name) = current {
        if !visited.insert(level_name.clone()) {
            break; // cycle guard against malformed metadata
        }
        let Some(info) = module.class_infos.get(&level_name) else {
            break;
        };
        for decl in &info.method_decls {
            let key = php_symbol_key(&decl.name);
            if order.contains_key(&key) {
                continue; // already claimed by a more-derived level
            }
            order.insert(key, counter);
            counter += 1;
            let visible = decl.visibility != Visibility::Private || level_name == class_name;
            if visible {
                ordered_names.push(decl.name.clone());
            }
        }
        current = info.parent.clone();
    }
    (order, ordered_names)
}

/// Property counterpart of `method_decl_order_and_names`: walks `ClassInfo.own_property_decl_order`
/// (instance and static combined, in source order — mirrors `method_decls` for properties, which
/// have no per-class body to also carry) instead of `method_decls`, and looks up each level's OWN
/// declared visibility via `property_visibilities`/`static_property_visibilities` (a name can
/// never be in both — the checker rejects redeclaring a static property as instance or vice
/// versa). Property names are case-sensitive, so `decl_order_by_key` is keyed by the EXACT
/// declared name (not `php_symbol_key`-folded, unlike the method version).
pub(crate) fn property_decl_order_and_names(module: &Module, class_name: &str) -> (HashMap<String, u32>, Vec<String>) {
    let mut order = HashMap::new();
    let mut ordered_names = Vec::new();
    let mut counter: u32 = 0;
    let mut current = Some(class_name.to_string());
    let mut visited = HashSet::new();
    while let Some(level_name) = current {
        if !visited.insert(level_name.clone()) {
            break; // cycle guard against malformed metadata
        }
        let Some(info) = module.class_infos.get(&level_name) else {
            break;
        };
        for name in &info.own_property_decl_order {
            if order.contains_key(name) {
                continue; // already claimed by a more-derived level
            }
            order.insert(name.clone(), counter);
            counter += 1;
            let visibility = info
                .property_visibilities
                .get(name)
                .or_else(|| info.static_property_visibilities.get(name))
                .cloned()
                .unwrap_or(Visibility::Public);
            let visible = visibility != Visibility::Private || level_name == class_name;
            if visible {
                ordered_names.push(name.clone());
            }
        }
        current = info.parent.clone();
    }
    (order, ordered_names)
}

/// Builds one class's flattened, segment-sorted method rows (both instance and
/// static methods, keyed the same way `ClassInfo::methods`/`static_methods` already
/// are — see the file-level "FLATTEN ALGORITHM" doc comment).
fn class_method_rows(module: &Module, class_name: &str, info: &ClassInfo) -> Vec<MethodRow> {
    let mut rows = Vec::with_capacity(info.methods.len() + info.static_methods.len());
    let (decl_order, _) = method_decl_order_and_names(module, class_name);
    for method_key in info.methods.keys().chain(info.static_methods.keys()) {
        let Some(decl) = find_method_decl(module, class_name, method_key) else {
            continue; // no declaring AST found (never happens for a sound ClassInfo); skip defensively
        };
        let declaring_class = info
            .method_declaring_classes
            .get(method_key)
            .or_else(|| info.static_method_declaring_classes.get(method_key))
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let declaring_class_id = module
            .class_infos
            .get(&declaring_class)
            .map(|declaring_info| declaring_info.class_id as u32)
            .unwrap_or(u32::MAX);
        rows.push(MethodRow {
            search_name: method_key.clone(),
            real_name: decl.name.clone(),
            modifiers: method_modifiers_bitmask(decl),
            declaring_class_id,
            decl_order: decl_order.get(method_key).copied().unwrap_or(u32::MAX),
        });
    }
    rows.sort_by(|a, b| a.search_name.as_bytes().cmp(b.search_name.as_bytes()));
    rows
}

/// Computes `(visibility/staticness/readonly/abstract/final bitmask)` for a
/// property declared on `info`. Mirrors `crate::codegen_ir::lower_inst::objects::
/// reflection::property_modifiers_and_type`'s bitmask half (the has-declared-type
/// half is not needed for the flat table).
fn property_modifiers_bitmask(info: &ClassInfo, property_name: &str, is_static: bool) -> u32 {
    let visibility = if is_static {
        info.static_property_visibilities.get(property_name)
    } else {
        info.property_visibilities.get(property_name)
    }
    .cloned()
    .unwrap_or(Visibility::Public);
    let mut bits = match visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 4,
    };
    if is_static {
        bits |= 16;
    }
    if info.readonly_properties.contains(property_name) {
        bits |= 128;
    }
    if info.abstract_properties.contains(property_name) {
        bits |= 64;
    }
    if info.final_properties.contains(property_name) || info.final_static_properties.contains(property_name) {
        bits |= 32;
    }
    bits
}

/// Builds one class's flattened, segment-sorted property rows (both instance and
/// static properties).
fn class_property_rows(module: &Module, class_name: &str, info: &ClassInfo) -> Vec<PropertyRow> {
    let mut rows = Vec::with_capacity(info.properties.len() + info.static_properties.len());
    let (decl_order, _) = property_decl_order_and_names(module, class_name);
    for (name, _) in &info.properties {
        let declaring_class = info
            .property_declaring_classes
            .get(name)
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let declaring_class_id = module
            .class_infos
            .get(&declaring_class)
            .map(|declaring_info| declaring_info.class_id as u32)
            .unwrap_or(u32::MAX);
        rows.push(PropertyRow {
            name: name.clone(),
            modifiers: property_modifiers_bitmask(info, name, false),
            declaring_class_id,
            decl_order: decl_order.get(name).copied().unwrap_or(u32::MAX),
        });
    }
    for (name, _) in &info.static_properties {
        let declaring_class = info
            .static_property_declaring_classes
            .get(name)
            .cloned()
            .unwrap_or_else(|| class_name.to_string());
        let declaring_class_id = module
            .class_infos
            .get(&declaring_class)
            .map(|declaring_info| declaring_info.class_id as u32)
            .unwrap_or(u32::MAX);
        rows.push(PropertyRow {
            name: name.clone(),
            modifiers: property_modifiers_bitmask(info, name, true),
            declaring_class_id,
            decl_order: decl_order.get(name).copied().unwrap_or(u32::MAX),
        });
    }
    rows.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
    rows
}

/// Content-keyed string interner: returns the SAME `.data` label for repeated byte
/// content instead of emitting duplicate `.ascii` bytes (`const_registry.rs`'s
/// `emit_name_table` does not do this, which is fine there because class/interface/
/// trait names are unique — method/property names legitimately repeat across
/// hundreds of classes, e.g. `__construct`, and this table must not scale with
/// row-count × name-length as a result).
struct StringInterner {
    labels: HashMap<Vec<u8>, String>,
    bytes: Vec<(String, Vec<u8>)>,
    counter: usize,
}

impl StringInterner {
    /// Creates an empty interner with no labels emitted yet.
    fn new() -> Self {
        Self {
            labels: HashMap::new(),
            bytes: Vec::new(),
            counter: 0,
        }
    }

    /// Interns `value` and returns its (possibly shared) `.data` label.
    fn intern(&mut self, value: &str) -> String {
        let key = value.as_bytes().to_vec();
        if let Some(label) = self.labels.get(&key) {
            return label.clone();
        }
        let label = format!("_reflectmemname_{}", self.counter);
        self.counter += 1;
        self.labels.insert(key.clone(), label.clone());
        self.bytes.push((label.clone(), key));
        label
    }

    /// Appends every interned string's `label:` + `.ascii` bytes to `out`.
    fn emit_bytes(&self, out: &mut String) {
        for (label, bytes) in &self.bytes {
            out.push_str(&format!("{}:\n", label));
            out.push_str(&format!(
                "    .ascii \"{}\"\n",
                escaped_ascii(&String::from_utf8_lossy(bytes))
            ));
        }
    }
}

/// Computes the flat, per-class-id-indexed method/property tables from `module`
/// and returns `(assembly_text, sizes)`. Emits five sibling tables:
/// `_reflect_class_id_table` (name→class_id, name-sorted), `_reflect_method_table` +
/// `_reflect_method_index` (class_id-indexed segments), and `_reflect_property_table`
/// + `_reflect_property_index` (same shape). See the file-level doc comment for the
/// flatten algorithm and row layouts.
pub(crate) fn emit_reflect_member_registry_data(module: &Module) -> (String, ReflectMemberRegistrySizes) {
    let mut class_entries: Vec<(&String, &ClassInfo)> = module
        .class_infos
        .iter()
        .filter(|(name, _)| !is_internal_synthetic_class_name(name))
        .collect();
    class_entries.sort_by_key(|(_, info)| info.class_id);

    let index_len = module
        .class_infos
        .values()
        .map(|info| info.class_id)
        .max()
        .map(|max_id| max_id as usize + 1)
        .unwrap_or(0);

    let mut method_rows: Vec<MethodRow> = Vec::new();
    let mut method_index: Vec<(usize, usize)> = vec![(0, 0); index_len];
    let mut property_rows: Vec<PropertyRow> = Vec::new();
    let mut property_index: Vec<(usize, usize)> = vec![(0, 0); index_len];
    let mut class_id_rows: Vec<(String, u64)> = Vec::new();

    for (class_name, info) in &class_entries {
        let class_id = info.class_id as usize;
        class_id_rows.push((php_symbol_key(class_name.trim_start_matches('\\')), info.class_id));

        let start = method_rows.len();
        let mut rows = class_method_rows(module, class_name, info);
        let count = rows.len();
        method_rows.append(&mut rows);
        if class_id < method_index.len() {
            method_index[class_id] = (start, count);
        }

        let pstart = property_rows.len();
        let mut prows = class_property_rows(module, class_name, info);
        let pcount = prows.len();
        property_rows.append(&mut prows);
        if class_id < property_index.len() {
            property_index[class_id] = (pstart, pcount);
        }
    }
    class_id_rows.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    // Jury Addendum #2: every index entry's segment must stay inside its row table — a bound
    // violation here would make the ARM64/x86_64 dispatchers' `mul`-computed segment address
    // walk off the end of `_reflect_method_table`/`_reflect_property_table` at runtime.
    debug_assert!(
        method_index.iter().all(|&(start, count)| start + count <= method_rows.len()),
        "a _reflect_method_index segment exceeds the method row table"
    );
    debug_assert!(
        property_index.iter().all(|&(start, count)| start + count <= property_rows.len()),
        "a _reflect_property_index segment exceeds the property row table"
    );

    let mut out = String::new();
    let mut names = StringInterner::new();
    out.push_str(".data\n");
    out.push_str(".p2align 3\n");

    // -- _reflect_class_id_table: name-sorted {name_ptr, name_len, class_id} --
    out.push_str(".globl _reflect_class_id_table_count\n_reflect_class_id_table_count:\n");
    out.push_str(&format!("    .quad {}\n", class_id_rows.len()));
    out.push_str(".globl _reflect_class_id_table\n_reflect_class_id_table:\n");
    for (name, class_id) in &class_id_rows {
        let label = names.intern(name);
        out.push_str(&format!("    .quad {}\n", label));
        out.push_str(&format!("    .quad {}\n", name.len()));
        out.push_str(&format!("    .quad {}\n", class_id));
    }

    out.push_str(".p2align 3\n");
    // -- _reflect_method_index: dense class_id-indexed {start, count} --
    out.push_str(".globl _reflect_method_index_count\n_reflect_method_index_count:\n");
    out.push_str(&format!("    .quad {}\n", method_index.len()));
    out.push_str(".globl _reflect_method_index\n_reflect_method_index:\n");
    for (start, count) in &method_index {
        out.push_str(&format!("    .quad {}\n", start));
        out.push_str(&format!("    .quad {}\n", count));
    }

    out.push_str(".p2align 3\n");
    // -- _reflect_method_table: per-class-segment-sorted method rows --
    out.push_str(".globl _reflect_method_table_count\n_reflect_method_table_count:\n");
    out.push_str(&format!("    .quad {}\n", method_rows.len()));
    out.push_str(".globl _reflect_method_table\n_reflect_method_table:\n");
    for row in &method_rows {
        let search_label = names.intern(&row.search_name);
        let real_label = names.intern(&row.real_name);
        out.push_str(&format!("    .quad {}\n", search_label));
        out.push_str(&format!("    .quad {}\n", row.search_name.len()));
        out.push_str(&format!("    .quad {}\n", real_label));
        out.push_str(&format!("    .quad {}\n", row.real_name.len()));
        out.push_str(&format!("    .long {}\n", row.modifiers));
        out.push_str(&format!("    .long {}\n", row.declaring_class_id));
        out.push_str(&format!("    .long {}\n", row.decl_order));
        out.push_str("    .long 0\n"); // padding: keeps METHOD_ROW_SIZE an 8-byte multiple
    }

    out.push_str(".p2align 3\n");
    // -- _reflect_property_index: dense class_id-indexed {start, count} --
    out.push_str(".globl _reflect_property_index_count\n_reflect_property_index_count:\n");
    out.push_str(&format!("    .quad {}\n", property_index.len()));
    out.push_str(".globl _reflect_property_index\n_reflect_property_index:\n");
    for (start, count) in &property_index {
        out.push_str(&format!("    .quad {}\n", start));
        out.push_str(&format!("    .quad {}\n", count));
    }

    out.push_str(".p2align 3\n");
    // -- _reflect_property_table: per-class-segment-sorted property rows --
    out.push_str(".globl _reflect_property_table_count\n_reflect_property_table_count:\n");
    out.push_str(&format!("    .quad {}\n", property_rows.len()));
    out.push_str(".globl _reflect_property_table\n_reflect_property_table:\n");
    for row in &property_rows {
        let label = names.intern(&row.name);
        out.push_str(&format!("    .quad {}\n", label));
        out.push_str(&format!("    .quad {}\n", row.name.len()));
        out.push_str(&format!("    .long {}\n", row.modifiers));
        out.push_str(&format!("    .long {}\n", row.declaring_class_id));
        out.push_str(&format!("    .long {}\n", row.decl_order));
        out.push_str("    .long 0\n"); // padding: keeps PROPERTY_ROW_SIZE an 8-byte multiple
    }

    out.push_str(".p2align 3\n");
    names.emit_bytes(&mut out);

    let method_table_bytes = method_rows.len() * METHOD_ROW_SIZE;
    let property_table_bytes = property_rows.len() * PROPERTY_ROW_SIZE;
    let index_bytes = method_index.len() * INDEX_ROW_SIZE + property_index.len() * INDEX_ROW_SIZE;
    let class_id_bytes = class_id_rows.len() * CLASS_ID_ROW_SIZE;
    let name_bytes: usize = names.bytes.iter().map(|(_, bytes)| bytes.len()).sum();
    let total_bytes = method_table_bytes + property_table_bytes + index_bytes + class_id_bytes + name_bytes;

    let sizes = ReflectMemberRegistrySizes {
        method_row_count: method_rows.len(),
        property_row_count: property_rows.len(),
        method_table_bytes,
        property_table_bytes,
        total_bytes,
    };
    (out, sizes)
}

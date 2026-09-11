//! Purpose:
//! Generates the per-program `__rt_web_reset` routine for `--web` builds. The
//! routine resets all process-persistent state between requests so the prefork
//! worker can serve request N+1 with the same clean state request N saw: it
//! releases and zeroes function static locals (and their init markers, so their
//! initializers re-run), releases the previous value of refcounted static class
//! properties (their initializers re-run in the handler body and restore the
//! defaults), releases and zeroes ordinary globals plus request superglobals
//! ($_SERVER/$_GET/$_POST) that survive between requests, and resets the
//! native and dynamic include-once bookkeeping, and the concat-buffer write offset.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()`, after every function and the
//!   `--web` handler body are emitted (so every static local has been recorded).
//! - The emitted handler prologue calls the label via `bl/call __rt_web_reset`.
//!
//! Key details:
//! - This is a per-program routine, not a fixed runtime helper: it needs the
//!   module's static set, so it is generated here rather than in `src/codegen/`.
//! - Refcounted release is memory-safety-critical: every release is guarded by
//!   the static's init marker / typed-property sentinel so an uninitialized slot
//!   (all-zero `.comm`) or a sentinel-as-pointer is never released.
//! - Function statics are zeroed (value + marker) so their initializers re-run.
//!   Static properties are NOT zeroed: the handler body re-runs their
//!   initializers after the reset, which rewrites both value and sentinel.
//! - The Magician include registry is reset only when the module links the eval bridge.

use crate::codegen::abi;
use crate::codegen::data_section::{DataSection, StaticLocalRecord};
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::codegen::UNINITIALIZED_TYPED_PROPERTY_SENTINEL;
use crate::ir::Module;
use crate::names::{classlike_activation_symbol, ir_global_symbol, static_property_symbol};
use std::collections::BTreeSet;
use crate::superglobals;
use crate::types::PhpType;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{AppleVariant, Platform, Target};
    use crate::ir::{Builder, Function, Immediate, IrType, Op, Ownership};

    #[test]
    fn native_include_reset_covers_catalog_cells_without_native_instructions() {
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        let path = std::path::PathBuf::from("/dynamic-only.php");
        let catalog = crate::ir::SourceCatalog::from_units([crate::resolver::SourceUnit {
            canonical_path: path.clone(), mode: crate::source::SourceMode::Php, source: "<?php".into(),
        }]).unwrap();
        let module = Module::with_source_catalog(target, catalog);
        let mut data = DataSection::new();
        let symbol = super::super::source_units::include_guard_symbol(&path);
        data.add_comm(symbol.clone(), 8);
        let mut emitter = Emitter::new(target);
        emit_web_reset(&mut emitter, &module, &data);
        let asm = emitter.output();
        assert!(asm.find(&symbol).unwrap() < asm.find("_heap_off").unwrap());
    }

    #[test]
    fn native_include_guards_reset_without_eval_bridge() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
            Target { apple_variant: AppleVariant::IOS, ..Target::new(Platform::MacOS, Arch::AArch64) },
            Target { apple_variant: AppleVariant::IOSSimulator, ..Target::new(Platform::MacOS, Arch::AArch64) },
        ] {
            let path = std::path::PathBuf::from("/fixture/module.php");
            let symbol = super::super::source_units::include_guard_symbol(&path);
            let catalog = crate::ir::SourceCatalog::from_units([crate::resolver::SourceUnit {
                canonical_path: path.clone(), mode: crate::source::SourceMode::Php, source: "<?php".into(),
            }]).unwrap();
            let id = catalog.id_for_path(&path).unwrap();
            let mut module = Module::with_source_catalog(target, catalog);
            let mut function = Function::new("included".into(), IrType::Void, PhpType::Void);
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", vec![]);
            builder.set_entry(entry);
            builder.position_at_end(entry);
            builder.emit(Op::IncludeOnceMark, vec![], Some(Immediate::Source(id)),
                IrType::Void, PhpType::Void, Ownership::NonHeap);
            // A method-only include must participate too, not just the entry body.
            module.class_methods.push(function);
            let mut data = DataSection::new();
            data.add_comm(symbol.clone(), 8);
            data.add_comm("_include_once_unrelated".into(), 8);
            let mut emitter = Emitter::new(target);
            emit_web_reset(&mut emitter, &module, &data);
            let asm = emitter.output();
            assert!(!asm.contains("__elephc_eval_include_request_reset"));
            assert!(!asm.contains("_include_once_unrelated"));
            let guard = asm.find(&symbol).expect("native guard must reset");
            assert!(guard < asm.find("_heap_off").unwrap());
        }
    }

    #[test]
    fn compiled_interface_activation_cells_reset_before_heap() {
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        let catalog = crate::ir::SourceCatalog::from_units([crate::resolver::SourceUnit {
            canonical_path: "/fixture/interface.php".into(),
            mode: crate::source::SourceMode::Php,
            source: "<?php".into(),
        }])
        .unwrap();
        let mut module = Module::with_source_catalog(target, catalog);
        let name = module.data.intern_string("Fixture\\Probe");
        let mut function = Function::new("main".into(), IrType::Void, PhpType::Void);
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        builder.emit(
            Op::ClassLikeActivate,
            vec![],
            Some(Immediate::ClassLikeActivation {
                source: crate::ir::SourceId::from_raw(0),
                site: crate::span::Span::dummy(),
                kind: crate::parser::ast::ClassLikeKind::Interface,
                name,
            }),
            IrType::Void,
            PhpType::Void,
            Ownership::NonHeap,
        );
        builder.terminate(crate::ir::Terminator::Return { value: None });
        module.add_function(function);
        let cell = crate::names::classlike_activation_symbol(
            crate::parser::ast::ClassLikeKind::Interface,
            "Fixture\\Probe",
        );
        let mut data = DataSection::new();
        data.add_comm(cell.clone(), 8);
        let mut emitter = Emitter::new(target);
        emit_web_reset(&mut emitter, &module, &data);
        let asm = emitter.output();
        assert!(asm.find(&cell).unwrap() < asm.find("_heap_off").unwrap());
    }
}

/// Minimal frame: just the x29/x30 footer (AArch64) or `push rbp` (x86_64),
/// which keeps the stack 16-byte aligned across the runtime helper calls.
const RESET_FRAME_SIZE: usize = 16;

/// Monotonic per-routine label counter so the reset's internal skip labels never
/// collide with each other across the many static slots it touches.
struct LabelGen {
    next: usize,
}

impl LabelGen {
    /// Creates a fresh label generator starting at zero.
    fn new() -> Self {
        Self { next: 0 }
    }

    /// Returns a unique `__rt_web_reset`-scoped label with the given prefix.
    fn next(&mut self, prefix: &str) -> String {
        let label = format!("__rt_web_reset_{}_{}", prefix, self.next);
        self.next += 1;
        label
    }
}

/// Emits the `__rt_web_reset` routine for the module.
///
/// Always emitted in `--web` builds (even with zero statics) so the handler's
/// `bl/call __rt_web_reset` resolves; in that case it only resets `_concat_off`.
/// Runs before the handler body's static-property/enum initializers, so it must
/// only RELEASE the previous refcounted property value, not rewrite it.
pub(super) fn emit_web_reset(emitter: &mut Emitter, module: &Module, data: &DataSection) {
    if emitter.target.arch == Arch::AArch64 {
        emitter.raw(".align 2");
    }
    emitter.blank();
    emitter.comment("--- runtime: web per-request state reset ---");
    emitter.label_global("__rt_web_reset");
    abi::emit_frame_prologue(emitter, RESET_FRAME_SIZE);

    let mut labels = LabelGen::new();
    for record in data.static_locals() {
        emit_static_local_reset(emitter, record, &mut labels);
    }
    for (symbol, php_type) in refcounted_static_properties(module) {
        emit_static_property_release(emitter, &symbol, &php_type, &mut labels);
    }
    for name in &module.data.global_names {
        let symbol = ir_global_symbol(name);
        if !superglobals::is_superglobal(name)
            && !module.extern_globals.contains_key(name)
            && data.has_comm(&symbol)
        {
            if superglobals::uses_shared_ref_cell(module, name) {
                emit_shared_superglobal_reset(emitter, &symbol, &mut labels);
            } else {
                emit_ordinary_global_reset(emitter, &symbol, &mut labels);
            }
        }
    }
    // Request superglobals ($_SERVER/$_GET/$_POST) live in `_eir_global_*` symbol
    // storage and are reassigned by the web prelude every request. Reset them here
    // so stale request arrays are gone before the next prelude builds replacements.
    for name in superglobals::SUPERGLOBALS {
        if superglobals::uses_shared_ref_cell(module, name) {
            emit_shared_superglobal_reset(emitter, &ir_global_symbol(name), &mut labels);
        } else {
            emit_superglobal_reset(emitter, &ir_global_symbol(name), &mut labels);
        }
    }

    if module.required_runtime_features.eval_bridge {
        emitter.comment("reset dynamic include_once state for the next request");
        let symbol = emitter
            .target
            .extern_symbol("__elephc_eval_include_request_reset");
        abi::emit_call_label(emitter, &symbol);
    }
    emit_native_include_resets(emitter, module, data);
    emit_native_classlike_activation_resets(emitter, module, data);

    // Clear every lazy enum case slot so request N+1 re-materializes its cases on
    // demand instead of reusing request N's object. This keeps the pre-existing
    // per-request lifecycle: the handler prologue used to re-run the eager enum
    // initializers and overwrite each slot every request, so a case object never
    // spanned two requests. Carrying the pointer over instead would be a new
    // hazard, because the per-request local cleanup can release a case that
    // reached a top-level local.
    super::enum_singletons::emit_enum_slot_resets(emitter, module);

    emit_concat_offset_reset(emitter);

    // The heap arena reset MUST be the final reset step: the static/global releases
    // above may run destructors or decref shared values, which require the arena to
    // still be valid. Wiping the allocator to pure-bump state last reclaims the whole
    // per-request arena at once. This is coupled to `--web` full-reset semantics; a
    // future persistent-statics worker-script mode (`--web-worker`, PR #456) must NOT
    // route through this routine, or its surviving statics would be freed underneath it.
    emit_heap_arena_reset(emitter);

    abi::emit_frame_restore(emitter, RESET_FRAME_SIZE);
    abi::emit_return(emitter);
}

/// Clears request-active compiled interface bindings after PHP-visible cleanup.
///
/// Discovery metadata remains immutable, but the cells consulted by existence
/// queries must not leak declaration activation from request N into request N+1.
fn emit_native_classlike_activation_resets(
    emitter: &mut Emitter,
    module: &Module,
    data: &DataSection,
) {
    let mut cells = BTreeSet::new();
    for function in module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.runtime_callable_invokers.iter())
    {
        for instruction in &function.instructions {
            let Some(crate::ir::Immediate::ClassLikeActivation { kind, name, .. }) =
                &instruction.immediate
            else {
                continue;
            };
            if *kind != crate::parser::ast::ClassLikeKind::Interface {
                continue;
            }
            let Some(name) = module.data.strings.get(name.as_raw() as usize) else {
                continue;
            };
            let cell = classlike_activation_symbol(*kind, name);
            if data.has_comm(&cell) {
                cells.insert(cell);
            }
        }
    }
    if cells.is_empty() {
        return;
    }
    emitter.comment("reset request-active compiled interface bindings");
    let zero = abi::temp_int_reg(emitter.target);
    abi::emit_load_int_immediate(emitter, zero, 0);
    for cell in cells {
        abi::emit_store_reg_to_symbol(emitter, zero, &cell, 0);
    }
}

/// Clears emitted once guards after PHP-visible cleanup, including native-only builds.
/// The catalog includes cells accessed only through the native lookup callback too.
fn emit_native_include_resets(emitter: &mut Emitter, module: &Module, data: &DataSection) {
    if let Some(catalog) = module.source_catalog() {
        for (_, source) in catalog.iter() {
            let label = super::source_units::include_guard_symbol(&source.canonical_path);
            if data.has_comm(&label) { abi::emit_store_zero_to_symbol(emitter, &label, 0); }
        }
    }
}

/// Resets the PHP heap arena to a pristine bump-only state: `_heap_off = 0`, an empty
/// ordered free list, and empty small-bin caches. Emitted as the final step of the
/// per-request `__rt_web_reset`, so the whole arena is reclaimed at once after every
/// refcounted per-request value has already been released above. Valid only under
/// `--web` full-reset semantics — nothing in the PHP arena legitimately survives a
/// request; Rust-side state (the PDO persistent-connection pool, bridge result cells)
/// lives outside `_heap_buf` and is unaffected.
fn emit_heap_arena_reset(emitter: &mut Emitter) {
    emitter.comment("reset the PHP heap arena to pure-bump allocation for the next request");
    abi::emit_store_zero_to_symbol(emitter, "_heap_off", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_free_list", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 8);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 16);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 24);
}

/// Resets one function static local: skips uninitialized slots, releases any
/// owned refcounted value, then zeroes the 16-byte value and the init marker so
/// the static's initializer re-runs on the next request.
fn emit_static_local_reset(emitter: &mut Emitter, record: &StaticLocalRecord, labels: &mut LabelGen) {
    let ty = record.php_type.codegen_repr();
    let skip_label = labels.next("skip_static");
    emitter.comment(&format!("reset static local {}", record.symbol));
    // Guard on the init marker: a zero marker means the initializer never ran
    // this process, so the value slot is still all-zero `.comm` storage and there
    // is nothing to release or zero. This also keeps us from releasing garbage.
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), &record.init_symbol, 0);
    abi::emit_branch_if_int_result_zero(emitter, &skip_label);

    emit_release_symbol_value(emitter, &record.symbol, &ty);
    // Zero the 16-byte value slot and the init marker so the initializer re-runs.
    abi::emit_store_zero_to_symbol(emitter, &record.symbol, 0);
    abi::emit_store_zero_to_symbol(emitter, &record.symbol, 8);
    abi::emit_store_zero_to_symbol(emitter, &record.init_symbol, 0);

    emitter.label(&skip_label);
}

/// Releases the previous value of one refcounted static class property without
/// zeroing it: the handler body's re-run initializer overwrites both the value
/// and the typed-property sentinel after this reset, so only the old owner needs
/// releasing to avoid a per-request leak. Skips the uninitialized sentinel so a
/// sentinel is never released as if it were a heap pointer.
fn emit_static_property_release(
    emitter: &mut Emitter,
    symbol: &str,
    php_type: &PhpType,
    labels: &mut LabelGen,
) {
    let ty = php_type.codegen_repr();
    let skip_label = labels.next("skip_prop");
    emitter.comment(&format!("release previous static property value {}", symbol));
    // Typed static properties carry an uninitialized sentinel in the high word
    // until first written. If it is still the sentinel, the value word holds no
    // owned heap pointer, so skip the release entirely.
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), symbol, 8);
    emit_branch_if_equals_sentinel(emitter, &skip_label);
    emit_release_symbol_value(emitter, symbol, &ty);
    emitter.label(&skip_label);
}

/// Releases and zeroes one referenced request superglobal's ref-cell owner.
///
/// The `_eir_global_*` symbol stores a shared cell pointer rather than the hash directly, so a
/// returned array containing `&$_SESSION` can retain the cell safely. Marker owners may keep the
/// payload alive until their containing arrays are released; this routine drops only the global
/// slot's share and then clears the symbol before the next prelude assignment.
fn emit_shared_superglobal_reset(emitter: &mut Emitter, symbol: &str, labels: &mut LabelGen) {
    let skip_label = labels.next("skip_superglobal");
    emitter.comment(&format!("reset request superglobal {}", symbol));
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), symbol, 0);
    abi::emit_branch_if_int_result_zero(emitter, &skip_label);
    abi::emit_call_label(emitter, "__rt_global_ref_cell_decref");
    abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    emitter.label(&skip_label);
}

/// Releases and zeroes one ordinary request superglobal stored as a direct hash pointer.
fn emit_superglobal_reset(emitter: &mut Emitter, symbol: &str, labels: &mut LabelGen) {
    let ty = superglobals::superglobal_type().codegen_repr();
    let skip_label = labels.next("skip_superglobal");
    emitter.comment(&format!("reset request superglobal {}", symbol));
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), symbol, 0);
    abi::emit_branch_if_int_result_zero(emitter, &skip_label);
    emit_release_symbol_value(emitter, symbol, &ty);
    abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    emitter.label(&skip_label);
}

/// Releases and zeroes one ordinary PHP global, whose storage is a boxed Mixed cell.
fn emit_ordinary_global_reset(emitter: &mut Emitter, symbol: &str, labels: &mut LabelGen) {
    let ty = PhpType::Mixed;
    let skip_label = labels.next("skip_global");
    emitter.comment(&format!("reset ordinary global {}", symbol));
    abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), symbol, 0);
    abi::emit_branch_if_int_result_zero(emitter, &skip_label);
    emit_release_symbol_value(emitter, symbol, &ty);
    abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    emitter.label(&skip_label);
}

/// Releases the owned refcounted value currently stored at `symbol` (offset 0).
///
/// Mirrors the function-epilogue cleanup shapes: strings free their payload via
/// the validating heap-free helper, callables release their descriptor, and
/// other refcounted kinds decref through the type-specific helper. Non-refcounted
/// types (int/bool/float/tagged scalar) own no heap value and are a no-op here.
fn emit_release_symbol_value(emitter: &mut Emitter, symbol: &str, ty: &PhpType) {
    match ty {
        PhpType::Str => {
            // Load the string pointer into the result register and free it. The
            // validating free safely ignores null and non-heap pointers.
            abi::emit_load_symbol_to_reg(emitter, abi::int_result_reg(emitter), symbol, 0);
            abi::emit_call_label(emitter, "__rt_heap_free_safe");
        }
        PhpType::Callable => {
            abi::emit_load_symbol_to_result(emitter, symbol, ty);
            abi::emit_decref_if_refcounted(emitter, ty);
        }
        other if other.is_refcounted() => {
            abi::emit_load_symbol_to_result(emitter, symbol, other);
            abi::emit_decref_if_refcounted(emitter, other);
        }
        _ => {}
    }
}

/// Branches to `label` when the integer result register equals the uninitialized
/// typed-property sentinel, so an unwritten typed property is skipped.
fn emit_branch_if_equals_sentinel(emitter: &mut Emitter, label: &str) {
    let scratch = abi::temp_int_reg(emitter.target);
    abi::emit_load_int_immediate(emitter, scratch, UNINITIALIZED_TYPED_PROPERTY_SENTINEL);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {}, {}", abi::int_result_reg(emitter), scratch)); // compare the property marker against the uninitialized sentinel
            emitter.instruction(&format!("b.eq {}", label));                    // skip the release when the property was never written
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {}, {}", abi::int_result_reg(emitter), scratch)); // compare the property marker against the uninitialized sentinel
            emitter.instruction(&format!("je {}", label));                      // skip the release when the property was never written
        }
    }
}

/// Resets the concat-buffer write offset to its process-start base (zero), so the
/// 64KB `_concat_buf` does not exhaust across many requests. `_concat_off` is
/// `.comm`-zero-initialized and nothing sets it nonzero at startup, so zero is the
/// correct base; the handler then captures this fresh base for its frame.
fn emit_concat_offset_reset(emitter: &mut Emitter) {
    emitter.comment("reset the concat-buffer write offset for the next request");
    abi::emit_store_zero_to_symbol(emitter, "_concat_off", 0);
}

/// Returns `(storage_symbol, php_type)` for every refcounted static class
/// property that the handler body initializes, enumerated exactly like
/// `emit_static_property_initializers` so the reset stays in lockstep with what
/// gets re-initialized each request. Non-refcounted properties are excluded:
/// their re-run initializer simply overwrites the scalar, with nothing to free.
fn refcounted_static_properties(module: &Module) -> Vec<(String, PhpType)> {
    let mut class_names = super::runtime_referenced_class_names(module)
        .into_iter()
        .collect::<Vec<_>>();
    class_names.sort();
    let mut props = Vec::new();
    for class_name in class_names {
        let Some(class_info) = module.class_infos.get(&class_name) else {
            continue;
        };
        for (property, php_type) in &class_info.static_properties {
            let declaring_class = class_info
                .static_property_declaring_classes
                .get(property)
                .map(String::as_str)
                .unwrap_or(class_name.as_str());
            if declaring_class != class_name {
                continue;
            }
            let ty = php_type.codegen_repr();
            if !(matches!(ty, PhpType::Str | PhpType::Callable) || ty.is_refcounted()) {
                continue;
            }
            props.push((static_property_symbol(&class_name, property), php_type.clone()));
        }
    }
    props
}

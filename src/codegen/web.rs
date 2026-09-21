//! Purpose:
//! Generates the per-program `__rt_web_reset` routine for `--web` builds. The
//! routine resets all process-persistent state between requests so the prefork
//! worker can serve request N+1 with the same clean state request N saw: it
//! releases and zeroes function static locals (and their init markers, so their
//! initializers re-run), releases the previous value of refcounted static class
//! properties (their initializers re-run in the handler body and restore the
//! defaults), releases and zeroes ordinary globals plus request superglobals
//! ($_SERVER/$_GET/$_POST) that survive between requests, and resets the
//! native and dynamic include-once bookkeeping, the concat-buffer write offset,
//! and the fixed runtime's request-scoped flags (`REQUEST_SCOPED_RUNTIME_FLAGS`).
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
//! - The fixed runtime's `.comm` flags are process storage that PHP scopes to a REQUEST.
//!   They are zero at process start, so only a worker serving a second request can observe
//!   one left set; `_headers_sent` is the reason this list exists.

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
    if super::context::module_uses_pcntl_signal_handlers(module) {
        abi::emit_call_label(emitter, "__rt_pcntl_release_handlers");
    }
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
    emit_deferred_class_load_resets(emitter, module, data);
    emit_native_classlike_activation_resets(emitter, module, data);

    // Clear every lazy enum case slot so request N+1 re-materializes its cases on
    // demand instead of reusing request N's object. This keeps the pre-existing
    // per-request lifecycle: the handler prologue used to re-run the eager enum
    // initializers and overwrite each slot every request, so a case object never
    // spanned two requests. Carrying the pointer over instead would be a new
    // hazard, because the per-request local cleanup can release a case that
    // reached a top-level local.
    super::enum_singletons::emit_enum_slot_resets(emitter, module);

    emit_request_scoped_flag_resets(emitter);

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

/// Clears the LOAD flag of every class the closed world carries only to answer a probe.
///
/// `class_exists($n, false)` must report the same "not loaded yet" on request 2 as on request 1;
/// a flag a previous request raised would otherwise make the next one skip the branch that the
/// probe guards.
fn emit_deferred_class_load_resets(emitter: &mut Emitter, module: &Module, data: &DataSection) {
    for name in &module.deferred_class_loads {
        let label = super::source_units::deferred_class_symbol(name);
        if data.has_comm(&label) {
            abi::emit_store_zero_to_symbol(emitter, &label, 0);
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
/// Fills the reclaimed arena with a pattern under `--heap-debug`, before the bump pointer moves.
///
/// A pointer that outlives the request boundary is not detectable on its own: the next request
/// allocates over the same addresses and the stale value keeps reading plausible bytes, so the
/// failure surfaces tens of requests later as a corrupted container with an intact header. That
/// is how the Twig generator crash presents, and it took a register dump and a whole-table read
/// to even classify. Poisoning what the boundary reclaims turns the first stale READ into the
/// failure, which is a short walk from its holder.
///
/// Off unless `--heap-debug` asked for it: this writes every live byte of the arena.
fn emit_heap_arena_poison(emitter: &mut Emitter) {
    let skip_label = "__rt_web_reset_skip_arena_poison";
    let result_reg = abi::int_result_reg(emitter);
    emitter.comment("--heap-debug: poison the arena this boundary reclaims");
    abi::emit_load_symbol_to_reg(emitter, result_reg, "_heap_debug_enabled", 0);
    abi::emit_branch_if_int_result_zero(emitter, skip_label);
    let length_reg = abi::int_arg_reg_name(emitter.target, 2);
    abi::emit_load_symbol_to_reg(emitter, length_reg, "_heap_off", 0);
    let fill_reg = abi::int_arg_reg_name(emitter.target, 1);
    abi::emit_load_int_immediate(emitter, fill_reg, 0xde);
    let base_reg = abi::int_arg_reg_name(emitter.target, 0);
    abi::emit_symbol_address(emitter, base_reg, "_heap_buf");
    let memset = emitter.target.extern_symbol("memset");
    abi::emit_call_label(emitter, &memset);
    emitter.label(skip_label);
}

/// Unbinds every heap granule from the object handle it carried, before the arena is recycled.
///
/// `_obj_handle_index` holds one u32 handle per 16-byte granule of the arena, and a granule is
/// only unbound when its block is freed. The request boundary frees nothing -- it drops the whole
/// arena -- so without this the next request inherits a table full of last request's bindings.
/// `__rt_object_handle_release` runs for EVERY block freed, object or not, reads the granule's
/// binding and pushes whatever it finds onto the released-handle stack. An ordinary string that
/// lands on a stale granule therefore hands back a handle it never minted, the same handle can
/// come back from several granules, and two live objects end up sharing one. Object identity is
/// what the interpreter keys its per-object maps on, so that is enough to make it read one object
/// where it expects another.
///
/// Only granules below the previous high-water mark can be bound, so this clears `heap_off / 4`
/// bytes -- four per sixteen of arena actually used, a few hundred KB for a Symfony request, not
/// the whole-heap wipe the old note priced it at.
fn emit_object_handle_index_reset(emitter: &mut Emitter) {
    let skip_label = "__rt_web_reset_skip_handle_index";
    emitter.comment("unbind heap granules from the object handles they carried");
    let used_reg = abi::int_result_reg(emitter);
    abi::emit_load_symbol_to_reg(emitter, used_reg, "_heap_off", 0);
    abi::emit_branch_if_int_result_zero(emitter, skip_label);
    let length_reg = abi::int_arg_reg_name(emitter.target, 2);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter
                .instruction(&format!("lsr {}, {}, #2", length_reg, used_reg));
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov {}, {}", length_reg, used_reg));
            emitter.instruction(&format!("shr {}, 2", length_reg));
        }
    }
    let fill_reg = abi::int_arg_reg_name(emitter.target, 1);
    abi::emit_load_int_immediate(emitter, fill_reg, 0);
    let base_reg = abi::int_arg_reg_name(emitter.target, 0);
    abi::emit_symbol_address(emitter, base_reg, "_obj_handle_index");
    let memset = emitter.target.extern_symbol("memset");
    abi::emit_call_label(emitter, &memset);
    emitter.label(skip_label);
}

fn emit_heap_arena_reset(emitter: &mut Emitter) {
    emitter.comment("reset the PHP heap arena to pure-bump allocation for the next request");
    emit_heap_arena_poison(emitter);
    emit_object_handle_index_reset(emitter);
    abi::emit_store_zero_to_symbol(emitter, "_heap_off", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_free_list", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 8);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 16);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 24);
}

/// Clears the runtime flags whose PHP-visible lifetime is one REQUEST, not one process.
///
/// `.comm` storage starts at zero, so a CLI build never notices that nothing puts it back; a
/// prefork worker does. `_headers_sent` is the one that showed: `__rt_stdout_write` raises it the
/// first time bytes escape the buffering stack, so from request 2 onward `headers_sent()` answered
/// true before the handler had emitted anything, Symfony's `Response::sendHeaders()` returned
/// early, and every response after the first lost its `Content-Type` and `Cache-Control` while
/// still carrying a byte-identical body -- a wrong response that no status code reports.
///
/// The rest are cleared for the same reason. A request that ends through a fatal or an uncaught
/// throw leaves its depth counters, its capture-mode flags and its exception chain set, and the
/// chain in particular points into the arena `emit_heap_arena_reset` is about to wipe.
fn emit_request_scoped_flag_resets(emitter: &mut Emitter) {
    emitter.comment("reset the runtime flags whose lifetime is one request");
    for symbol in REQUEST_SCOPED_RUNTIME_FLAGS {
        abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    }
}

/// The request-scoped runtime flags `__rt_web_reset` clears, grouped by owning subsystem.
///
/// Every entry is 8-byte `.comm` storage declared unconditionally by the fixed runtime data, so
/// the symbol always resolves and zero is always its process-start value. Tables whose slots hold
/// OS resources (stream, directory and process handles) are deliberately absent: those have to be
/// CLOSED at the request boundary, not blanked, which is a separate change.
///
/// `_obj_handle_next` is absent for a sharper reason, and putting it back would be a bug.
/// `_obj_handle_index` binds a handle to a heap GRANULE and is not cleared by the arena wipe, so
/// a granule that held an object last request still reads back that object's handle. Today the
/// cursor only climbs, so such a stale handle is always below it and re-minting it is harmless.
/// Restart the cursor at 1 and it stops being harmless: the next block to land on that granule
/// pushes the stale handle onto the free stack when it is freed, and a handle this request has
/// already handed to a live object gets handed out a second time. Restarting the numbering means
/// clearing `_obj_handle_index` too -- one `u32` per 16 heap bytes, so 128MB of stores for a
/// 512MB heap, every request -- which is why `spl_object_id()` still climbs across requests here.
const REQUEST_SCOPED_RUNTIME_FLAGS: &[&str] = &[
    // Raised by `__rt_stdout_write` once bytes reach the response sink.
    "_headers_sent",
    // Output buffering: stack depth, both re-entry guards, and the implicit-flush setting.
    "_ob_level",
    "_ob_in_handler",
    "_ob_flushing",
    "_ob_implicit_flush",
    // `print_r($value, true)` capture mode and its accumulated write offset.
    "_print_r_mode",
    "_print_r_off",
    // var_dump indentation and its recursion guard depth.
    "_vd_indent",
    "_vd_seen_n",
    // The handler chain and the pending throw, both of which point into the request arena.
    "_exc_handler_top",
    "_exc_call_frame_top",
    "_exc_value",
    // Fiber scheduling state and the saved main-fiber context.
    "_fiber_current",
    "_fiber_main_saved_sp",
    "_fiber_main_saved_exc",
    "_fiber_main_saved_call_frame",
    // serialize()/unserialize() back-reference counters and policy fields.
    "_ser_value_counter",
    "_ser_obj_count",
    "_unser_count",
    "_unser_depth",
    "_unser_active",
    "_unser_context",
    "_unser_allowed_mode",
    // `json_last_error()` plus the encoder/decoder state its message is rendered from. Every
    // json call site rewrites the active fields before use, so zero is a safe boundary value.
    "_json_last_error",
    "_json_active_flags",
    "_json_active_depth",
    "_json_indent_depth",
    "_json_depth_limit",
    "_json_validate_idx",
    "_json_validate_ptr",
    "_json_validate_len",
    "_json_decode_assoc",
    "_json_error_source_ptr",
    "_json_error_location_active",
    "_json_error_line",
    "_json_error_column",
    // The `@` suppression depth, which a fatal inside a suppressed call leaves raised.
    "_rt_diag_suppression",
    // The cycle collector's re-entry guard, safepoint counter and release suppression.
    "_gc_collecting",
    "_gc_safepoint_count",
    "_gc_release_suppressed",
];

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

/// Releases the previous value of one refcounted static class property AND zeroes the slot.
///
/// Leaving the pointer behind looks safe -- the handler body's initializer overwrites it next
/// request -- and it is not. That store releases whatever it finds in the slot first, and the
/// arena is deterministic: the next request replays the same allocation sequence and the
/// initializer gets back the SAME address this request used. The store then releases the block
/// it is about to write, taking a brand-new value's refcount from 1 to 0, and the property is
/// left pointing at freed storage for the rest of the request.
///
/// `Request::$trustedProxies = []` in the Symfony `--web` build is exactly that: allocated at
/// arena offset 117104, read a moment later by `isFromTrustedProxy()` with `refcount=0, kind=0`
/// -- the footprint `__rt_heap_free` leaves. Zero releases as a no-op, which is the same reason
/// `emit_static_property_sentinel` clears the value word for properties without a default.
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
    abi::emit_store_zero_to_symbol(emitter, symbol, 0);
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
            emitter.instruction(                                                // compare the property marker with the uninitialized sentinel
                &format!("cmp {}, {}", abi::int_result_reg(emitter), scratch)
            );
            emitter.instruction(&format!("b.eq {}", label));                    // skip the release when the property was never written
        }
        Arch::X86_64 => {
            emitter.instruction(                                                // compare the property marker with the uninitialized sentinel
                &format!("cmp {}, {}", abi::int_result_reg(emitter), scratch)
            );
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

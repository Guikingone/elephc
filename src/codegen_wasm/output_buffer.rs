//! Purpose:
//! PHP's output buffering (`ob_start` and friends) for the wasm32-wasi backend.
//!
//! Called from:
//! - `plan::lower_module` — `emit_output_buffer_runtime`, emitted only for a module that calls
//!   one of these builtins, so every other module keeps the exact bytes it had.
//! - `runtime` — the five stdout writes funnel through `__rt_stdout_write`, which is where a
//!   buffer intercepts them.
//!
//! Key details:
//! - ONE interception point. Every write php can capture goes to fd 1 through
//!   `__rt_wasi_write_or_fail`, and there are five such sites; routing them through
//!   `__rt_stdout_write` is what makes buffering possible without touching each of them.
//! - The buffer stack lives in the scratch region, `LEVEL_BYTES` per level and `MAX_LEVELS`
//!   deep. A deeper `ob_start` answers false, which is what php does when a buffer cannot be
//!   created — rather than growing without bound in a runtime that cannot report failure.
//! - The handler's `$phase` is MEASURED, not assumed: php passes `START` (1) on a buffer's
//!   first flush, `FINAL` (8) when the buffer is being ended, and `9` when the first flush is
//!   also the last. A per-level "has flushed" bit is what distinguishes them.
//! - `ob_get_clean` does NOT run the handler — it answers the raw captured bytes — while
//!   `ob_end_flush` does. That asymmetry is php's.

use super::wat::{Global, ValType, WatModule};

/// Offset of the buffer-level table inside the float-scratch region.
pub(super) const OB_LEVELS: u32 = 0x5100;

/// Bytes per level: buffer pointer, used, capacity, chunk size, handler descriptor, flags.
const LEVEL_BYTES: u32 = 32;

/// How deep `ob_start` will nest. php has no fixed limit; this table does, and answering false
/// past it is the same observable as a buffer php failed to create.
const MAX_LEVELS: u32 = 8;

/// Initial capacity of a level's buffer. php reports `buffer_size` 16384 for a default buffer,
/// which is what `ob_get_status` answers regardless of what is actually allocated.
const INITIAL_CAPACITY: u32 = 256;

/// The `buffer_size` php reports for a default buffer.
const REPORTED_BUFFER_SIZE: i64 = 16384;

/// Adds the output-buffering runtime.
///
/// `has_callable_ladder` says whether `__rt_closure_call` exists in this module. When it does
/// not, no callable can be constructed either, so `ob_start` cannot have received a handler and
/// the flush path is emitted without the call that would otherwise be an undefined symbol.
pub(super) fn emit_output_buffer_runtime(wm: &mut WatModule, has_callable_ladder: bool) {
    wm.add_global(Global {
        name: "__ob_depth".to_string(),
        ty: ValType::I32,
        mutable: true,
        init: 0,
    });
    wm.add_raw_func(&rt_ob_level_ptr());
    wm.add_raw_func(&rt_ob_append());
    wm.add_raw_func(&rt_ob_emit());
    wm.add_raw_func(&rt_ob_run_handler(has_callable_ladder));
    wm.add_raw_func(&rt_ob_start());
    wm.add_raw_func(RT_OB_END_FLUSH);
    wm.add_raw_func(RT_OB_END_CLEAN);
    wm.add_raw_func(RT_OB_GET_CLEAN);
}

/// `__rt_ob_level_ptr`: the record address of one buffer level.
fn rt_ob_level_ptr() -> String {
    format!(
        r#"(func $__rt_ob_level_ptr (param $index i32) (result i32)
  (i32.add (i32.add (global.get $__float_scratch) (i32.const {base}))
           (i32.mul (local.get $index) (i32.const {stride}))))
"#,
        base = OB_LEVELS,
        stride = LEVEL_BYTES
    )
}

/// `__rt_ob_append`: appends bytes to the innermost buffer, growing it when needed.
///
/// The growth doubles rather than fitting exactly, so a `echo` loop inside a buffer costs
/// amortised copying rather than one reallocation per write.
fn rt_ob_append() -> String {
    r#"(func $__rt_ob_append (param $ptr i32) (param $len i64)
  (local $lvl i32) (local $buf i32) (local $used i32) (local $cap i32) (local $need i32)
  (local $grown i32) (local $i i32) (local $n i32)
  (local.set $n (i32.wrap_i64 (local.get $len)))
  (if (i32.eqz (local.get $n))
    (then (return)))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))
  (local.set $buf (i32.load offset=0 (local.get $lvl)))
  (local.set $used (i32.load offset=4 (local.get $lvl)))
  (local.set $cap (i32.load offset=8 (local.get $lvl)))
  (local.set $need (i32.add (local.get $used) (local.get $n)))
  (if (i32.gt_u (local.get $need) (local.get $cap))
    (then
      (local.set $cap (i32.mul (local.get $cap) (i32.const 2)))
      (if (i32.gt_u (local.get $need) (local.get $cap))
        (then (local.set $cap (local.get $need))))
      (local.set $grown (call $__rt_heap_alloc (local.get $cap)))
      (local.set $i (i32.const 0))
      (block $copied (loop $byte
        (br_if $copied (i32.ge_u (local.get $i) (local.get $used)))
        (i32.store8 (i32.add (local.get $grown) (local.get $i))
                    (i32.load8_u (i32.add (local.get $buf) (local.get $i))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $byte)))
      (call $__rt_heap_free (local.get $buf))
      (local.set $buf (local.get $grown))
      (i32.store offset=0 (local.get $lvl) (local.get $buf))
      (i32.store offset=8 (local.get $lvl) (local.get $cap))))
  (local.set $i (i32.const 0))
  (block $written (loop $byte
    (br_if $written (i32.ge_u (local.get $i) (local.get $n)))
    (i32.store8 (i32.add (local.get $buf) (i32.add (local.get $used) (local.get $i)))
                (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $byte)))
  (i32.store offset=4 (local.get $lvl) (local.get $need)))
"#
    .to_string()
}

/// `__rt_stdout_write`: the single interception point for everything php can capture.
///
/// With no buffer active this is exactly the write it replaces; with one it appends, and a
/// CHUNKED buffer auto-flushes once its content reaches the chunk size — which is the observable
/// php produces for `ob_start($handler, 8)`.
pub(super) const RT_STDOUT_WRITE: &str = r#"(func $__rt_stdout_write (param $ptr i32) (param $len i32)
  (local $lvl i32)
  (if (i32.eqz (global.get $__ob_depth))
    (then
      (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (local.get $len))
      (return)))
  (call $__rt_ob_append (local.get $ptr) (i64.extend_i32_u (local.get $len)))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))
  ;; Both operands must be BOOLEANS: `i32.and` is bitwise, so testing the chunk size itself
  ;; against a 0/1 comparison answers `8 & 1 = 0` and the buffer never auto-flushes.
  (if (i32.and (i32.ne (i32.load offset=12 (local.get $lvl)) (i32.const 0))
               (i32.ge_u (i32.load offset=4 (local.get $lvl)) (i32.load offset=12 (local.get $lvl))))
    (then (call $__rt_ob_emit (i32.const 0)))))
"#;

/// The `__rt_stdout_write` a module that never buffers gets: the write it replaces, and nothing
/// else. Emitted instead of the buffering one so a program with no `ob_start` keeps the exact
/// bytes it had rather than carrying a buffer stack it can never enter.
pub(super) const RT_STDOUT_WRITE_PASSTHROUGH: &str =
    r#"(func $__rt_stdout_write (param $ptr i32) (param $len i32)
  (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (local.get $len)))
"#;

/// Whether this module calls any output-buffering builtin.
pub(super) fn module_uses_output_buffering(module: &crate::ir::Module) -> bool {
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
                let target = match target {
                    RuntimeCallTarget::Function(id) => *id,
                    RuntimeCallTarget::ProfiledFunction { target, .. } => *target,
                    _ => return false,
                };
                matches!(
                    target,
                    RuntimeFnId::ObStart
                        | RuntimeFnId::ObGetClean
                        | RuntimeFnId::ObEndFlush
                        | RuntimeFnId::ObEndClean
                        | RuntimeFnId::ObGetStatus
                )
            })
        })
}

/// The seven keys `ob_get_status` answers, in php's own order.
pub(super) const STATUS_KEYS: [&str; 7] = [
    "name",
    "type",
    "flags",
    "level",
    "chunk_size",
    "buffer_size",
    "buffer_used",
];

/// The handler name php reports for a buffer started without one.
pub(super) const DEFAULT_HANDLER_NAME: &str = "default output handler";

/// `__rt_ob_emit`: flushes the innermost buffer's contents outward, keeping the level.
///
/// `$final` says whether the buffer is being ENDED, which the handler sees in its phase. The
/// level's bytes are consumed either way: an auto-flush leaves the level live and empty.
fn rt_ob_emit() -> String {
    r#"(func $__rt_ob_emit (param $final i32)
  (local $lvl i32) (local $ptr i32) (local $len i64) (local $depth i32)
  (local.set $depth (global.get $__ob_depth))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (local.get $depth) (i32.const 1))))
  (call $__rt_ob_run_handler (local.get $lvl) (local.get $final))
  (local.set $len)
  (local.set $ptr)
  ;; The level is emptied BEFORE the bytes travel outward: a handler that echoes would
  ;; otherwise append into the buffer it is flushing.
  (i32.store offset=4 (local.get $lvl) (i32.const 0))
  (i32.store offset=24 (local.get $lvl) (i32.const 1))            ;; this level has now flushed
  (global.set $__ob_depth (i32.sub (local.get $depth) (i32.const 1)))
  (if (i64.ne (local.get $len) (i64.const 0))
    (then
      (if (i32.eqz (global.get $__ob_depth))
        (then (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (i32.wrap_i64 (local.get $len))))
        (else (call $__rt_ob_append (local.get $ptr) (local.get $len))))))
  (global.set $__ob_depth (local.get $depth)))
"#
    .to_string()
}

/// `__rt_ob_run_handler`: the bytes a level contributes when it flushes.
///
/// With no handler that is the captured bytes themselves. With one, php calls it as
/// `handler(string $buffer, int $phase)` and emits what it returns; the phase is
/// `START` (1) on the level's first flush, `FINAL` (8) when the buffer is being ended, and both
/// when the first flush is also the last — measured on php-src 8.5.6.
fn rt_ob_run_handler(has_callable_ladder: bool) -> String {
    if !has_callable_ladder {
        // No callable can exist in this module, so `ob_start` cannot have been given a handler.
        return r#"(func $__rt_ob_run_handler (param $lvl i32) (param $final i32) (result i32 i64)
  (i32.load offset=0 (local.get $lvl))
  (i64.extend_i32_u (i32.load offset=4 (local.get $lvl))))
"#
        .to_string();
    }
    r#"(func $__rt_ob_run_handler (param $lvl i32) (param $final i32) (result i32 i64)
  (local $desc i64) (local $args i32) (local $cell i32) (local $res i32)
  (local $phase i64) (local $ptr i32) (local $len i64) (local $tag i64) (local $ptr2 i64)
  (local.set $desc (i64.load offset=16 (local.get $lvl)))
  (if (i64.eqz (local.get $desc))
    (then (return
      (i32.load offset=0 (local.get $lvl))
      (i64.extend_i32_u (i32.load offset=4 (local.get $lvl))))))
  (local.set $phase (i64.or
    (select (i64.const 0) (i64.const 1) (i32.load offset=24 (local.get $lvl)))
    (select (i64.const 8) (i64.const 0) (local.get $final))))
  (local.set $args (call $__rt_array_new (i64.const 2) (i64.const 16)))
  (local.set $cell (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (i32.load offset=0 (local.get $lvl)))
    (i64.extend_i32_u (i32.load offset=4 (local.get $lvl)))))
  ;; `__rt_array_push_mixed` records the pointer WITHOUT taking a reference — every lowered
  ;; call site increfs before pushing — so the array is given this cell's only one. Releasing
  ;; here freed what the array had just recorded, and the wrapper then read zeros.
  (local.set $args (call $__rt_array_push_mixed (local.get $args) (local.get $cell)))
  (local.set $cell (call $__rt_mixed_from_value (i64.const 0) (local.get $phase) (i64.const 0)))
  (local.set $args (call $__rt_array_push_mixed (local.get $args) (local.get $cell)))
  (local.set $res (call $__rt_closure_call (i32.wrap_i64 (local.get $desc)) (local.get $args)))
  (call $__rt_decref_any (local.get $args))
  ;; The handler answers a string; anything else contributes nothing, which is php's behaviour
  ;; for a handler that returns false.
  ;; Read through `__rt_mixed_unbox` rather than by hand: it is the sanctioned reader, and it
  ;; unwraps a NESTED cell — which a raw load of the tag word would mistake for a non-string.
  (call $__rt_mixed_unbox (local.get $res))
  (local.set $len)
  (local.set $ptr2)
  (local.set $tag)
  (if (i64.ne (local.get $tag) (i64.const 1))
    (then
      (call $__rt_decref_any (local.get $res))
      (return (i32.const 0) (i64.const 0))))
  (local.set $ptr (i32.wrap_i64 (local.get $ptr2)))
  ;; The cell owns those bytes, so they are persisted before it is released.
  (call $__rt_str_persist (local.get $ptr) (local.get $len))
  (local.set $len)
  (local.set $ptr)
  (call $__rt_decref_any (local.get $res))
  (local.get $ptr)
  (local.get $len))
"#
    .to_string()
}

/// `__rt_ob_start`: pushes a buffer level, answering php's bool.
fn rt_ob_start() -> String {
    format!(
        r#"(func $__rt_ob_start (param $handler i64) (param $chunk i64) (result i64)
  (local $lvl i32)
  (if (i32.ge_u (global.get $__ob_depth) (i32.const {max}))
    (then (return (i64.const 0))))
  (local.set $lvl (call $__rt_ob_level_ptr (global.get $__ob_depth)))
  (i32.store offset=0 (local.get $lvl) (call $__rt_heap_alloc (i32.const {cap})))
  (i32.store offset=4 (local.get $lvl) (i32.const 0))
  (i32.store offset=8 (local.get $lvl) (i32.const {cap}))
  (i32.store offset=12 (local.get $lvl) (i32.wrap_i64 (local.get $chunk)))
  (i64.store offset=16 (local.get $lvl) (local.get $handler))
  ;; The level OWNS its handler. The EIR releases the descriptor as soon as `ob_start` returns
  ;; — the closure is an owning temporary there — so a borrowed one would be freed long before
  ;; the flush that calls it.
  (if (i64.ne (local.get $handler) (i64.const 0))
    (then (call $__rt_incref (i32.wrap_i64 (local.get $handler)))))
  (i32.store offset=24 (local.get $lvl) (i32.const 0))
  (global.set $__ob_depth (i32.add (global.get $__ob_depth) (i32.const 1)))
  (i64.const 1))
"#,
        max = MAX_LEVELS,
        cap = INITIAL_CAPACITY
    )
}

/// `__rt_ob_end_flush`: flushes the innermost buffer outward and discards the level.
const RT_OB_END_FLUSH: &str = r#"(func $__rt_ob_end_flush (result i64)
  (local $lvl i32)
  (if (i32.eqz (global.get $__ob_depth))
    (then (return (i64.const 0))))
  (call $__rt_ob_emit (i32.const 1))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))
  (if (i64.ne (i64.load offset=16 (local.get $lvl)) (i64.const 0))
    (then (call $__rt_decref_any (i32.wrap_i64 (i64.load offset=16 (local.get $lvl))))))
  (call $__rt_heap_free (i32.load offset=0 (local.get $lvl)))
  (global.set $__ob_depth (i32.sub (global.get $__ob_depth) (i32.const 1)))
  (i64.const 1))
"#;

/// `__rt_ob_end_clean`: discards the innermost buffer without emitting anything.
const RT_OB_END_CLEAN: &str = r#"(func $__rt_ob_end_clean (result i64)
  (local $lvl i32)
  (if (i32.eqz (global.get $__ob_depth))
    (then (return (i64.const 0))))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))
  (if (i64.ne (i64.load offset=16 (local.get $lvl)) (i64.const 0))
    (then (call $__rt_decref_any (i32.wrap_i64 (i64.load offset=16 (local.get $lvl))))))
  (call $__rt_heap_free (i32.load offset=0 (local.get $lvl)))
  (global.set $__ob_depth (i32.sub (global.get $__ob_depth) (i32.const 1)))
  (i64.const 1))
"#;

/// `__rt_ob_get_clean`: answers the captured bytes and discards the level.
///
/// The handler is NOT run: php's `ob_get_clean` returns what was captured, unlike
/// `ob_end_flush`, which emits what the handler produced.
const RT_OB_GET_CLEAN: &str = r#"(func $__rt_ob_get_clean (result i32)
  (local $lvl i32) (local $ptr i32) (local $len i64) (local $cell i32)
  (if (i32.eqz (global.get $__ob_depth))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))
  (local.set $cell (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (i32.load offset=0 (local.get $lvl)))
    (i64.extend_i32_u (i32.load offset=4 (local.get $lvl)))))
  (if (i64.ne (i64.load offset=16 (local.get $lvl)) (i64.const 0))
    (then (call $__rt_decref_any (i32.wrap_i64 (i64.load offset=16 (local.get $lvl))))))
  (call $__rt_heap_free (i32.load offset=0 (local.get $lvl)))
  (global.set $__ob_depth (i32.sub (global.get $__ob_depth) (i32.const 1)))
  (local.get $cell))
"#;

/// Emits `__rt_ob_get_status`, which needs the seven key names laid out as data.
///
/// php answers an ordered map, and `foreach` over it must see php's key ORDER, so the keys are
/// inserted in the order php reports them: name, type, flags, level, chunk_size, buffer_size,
/// buffer_used — measured on php-src 8.5.6.
pub(super) fn emit_ob_get_status(wm: &mut WatModule, keys: &[(u32, u32)], handler: (u32, u32)) {
    debug_assert_eq!(keys.len(), 7);
    let entry = |index: usize, value: String| {
        let (ptr, len) = keys[index];
        format!(
            "  (local.set $h (call $__rt_hash_set (local.get $h) \
             (i64.const {ptr}) (i64.const {len}) {value}))\n"
        )
    };
    let (name_ptr, name_len) = handler;
    let mut wat = String::from(
        "(func $__rt_ob_get_status (result i32)\n  (local $h i32) (local $lvl i32) (local $used i64)\n",
    );
    wat.push_str("  (local.set $used (i64.const 0))\n");
    wat.push_str("  (if (global.get $__ob_depth)\n    (then\n");
    wat.push_str(
        "      (local.set $lvl (call $__rt_ob_level_ptr (i32.sub (global.get $__ob_depth) (i32.const 1))))\n",
    );
    wat.push_str(
        "      (local.set $used (i64.extend_i32_u (i32.load offset=4 (local.get $lvl))))))\n",
    );
    wat.push_str("  (local.set $h (call $__rt_hash_new (i64.const 16) (i64.const 0)))\n");
    // Tag 1 is a string value, tag 0 an int; the hash stores each inline with its own tag.
    wat.push_str(&entry(
        0,
        format!("(i64.const {name_ptr}) (i64.const {name_len}) (i64.const 1)"),
    ));
    wat.push_str(&entry(1, "(i64.const 0) (i64.const 0) (i64.const 0)".to_string()));
    wat.push_str(&entry(2, "(i64.const 112) (i64.const 0) (i64.const 0)".to_string()));
    wat.push_str(&entry(3, "(i64.const 0) (i64.const 0) (i64.const 0)".to_string()));
    wat.push_str(&entry(4, "(i64.const 0) (i64.const 0) (i64.const 0)".to_string()));
    wat.push_str(&entry(
        5,
        format!("(i64.const {REPORTED_BUFFER_SIZE}) (i64.const 0) (i64.const 0)"),
    ));
    wat.push_str(&entry(6, "(local.get $used) (i64.const 0) (i64.const 0)".to_string()));
    wat.push_str("  (call $__rt_mixed_from_value (i64.const 5)\n");
    wat.push_str("    (i64.extend_i32_u (local.get $h)) (i64.const 0)))\n");
    wm.add_raw_func(&wat);
}

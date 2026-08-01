//! Purpose:
//! Lowers PHP builtin functions that compile to a handful of WebAssembly instructions with no
//! runtime helper and no allocation, plus the audit contract each of them is admitted under.
//!
//! Called from:
//! - `crate::codegen_wasm::inst::lower_runtime_call` for emission.
//! - `crate::codegen_wasm::capability::runtime_function_shape_issue` for the static audit.
//!
//! Key details:
//! - Everything here is an EXACT identity: the WebAssembly instruction and the PHP builtin agree
//!   on every input including NaN, both infinities and negative zero, so there is no diagnostic
//!   to emit and no profile to branch on. A builtin that needs a table, an allocation or a
//!   warning does not belong in this module.
//! - The audit and the emitter read the same operand contract, so a shape the emitter cannot
//!   lower is refused before planning rather than producing an invalid module.

use super::context::{FnCtx, Result};
use super::wat::WatModule;
use super::inst::{operand, store_result};
use super::WasmError;
use crate::ir::{Function, Instruction, IrHeapKind, IrType, RuntimeFnId, UnaryStringRuntime};
use crate::types::PhpType;

/// Registers the WAT helpers the builtins in this module call.
///
/// Emitted for every module: none of these touch WASI, so a reactor carries them too.
pub(super) fn emit_builtin_runtime(wm: &mut WatModule) {
    wm.add_raw_func(RT_STR_REGION_EQ);
    wm.add_raw_func(RT_STR_CONTAINS);
    wm.add_raw_func(RT_STR_MAP_CASE);
    wm.add_raw_func(RT_STR_REVERSE);
    wm.add_raw_func(RT_STR_ALLOC);
    wm.add_raw_func(RT_STR_BIN2HEX);
    wm.add_raw_func(RT_STR_ADDSLASHES);
    wm.add_raw_func(RT_STR_STRIPSLASHES);
    wm.add_raw_func(RT_STR_NL2BR);
}

/// `__rt_str_map_case`: owns a copy of a string with its ASCII letters case-mapped.
///
/// `$upper` selects the direction. Since PHP 8.2 `strtoupper` and `strtolower` are
/// LOCALE-INDEPENDENT and touch `A-Z` / `a-z` only — byte `0xE9` comes back unchanged, which is
/// what makes a pure byte map correct here rather than an approximation.
const RT_STR_MAP_CASE: &str = r#"(func $__rt_str_map_case (param $ptr i32) (param $len i64) (param $upper i32) (result i32) (result i64)
  (local $out i32)
  (local $olen i64)
  (local $i i64)
  (local $byte i32)
  (call $__rt_str_persist (local.get $ptr) (local.get $len))      ;; own a copy to transform in place
  (local.set $olen)                                               ;; persisted length
  (local.set $out)                                                ;; persisted pointer
  (local.set $i (i64.const 0))                                    ;; i = 0
  (block $end (loop $map
    (br_if $end (i64.ge_s (local.get $i) (local.get $olen)))      ;; every byte visited
    (local.set $byte (i32.load8_u (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))))
    (if (local.get $upper)
      (then
        (if (i32.and (i32.ge_u (local.get $byte) (i32.const 97)) (i32.le_u (local.get $byte) (i32.const 122)))
          (then (i32.store8 (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))
                            (i32.sub (local.get $byte) (i32.const 32))))))  ;; a-z -> A-Z
      (else
        (if (i32.and (i32.ge_u (local.get $byte) (i32.const 65)) (i32.le_u (local.get $byte) (i32.const 90)))
          (then (i32.store8 (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))
                            (i32.add (local.get $byte) (i32.const 32)))))))  ;; A-Z -> a-z
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $map)))
  (local.get $out) (local.get $olen))                             ;; owned result
"#;

/// `__rt_str_reverse`: owns a byte-reversed copy of a string.
///
/// `strrev` operates on BYTES, not characters, so a multi-byte sequence comes back with its
/// bytes in reverse order — which is what PHP does.
const RT_STR_REVERSE: &str = r#"(func $__rt_str_reverse (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)
  (local $olen i64)
  (local $i i64)
  (call $__rt_str_persist (local.get $ptr) (local.get $len))      ;; own a copy sized like the source
  (local.set $olen)                                               ;; persisted length
  (local.set $out)                                                ;; persisted pointer
  (local.set $i (i64.const 0))                                    ;; i = 0
  (block $end (loop $rev
    (br_if $end (i64.ge_s (local.get $i) (local.get $olen)))      ;; every byte placed
    (i32.store8
      (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))
      (i32.load8_u (i32.add (local.get $ptr)
                            (i32.wrap_i64 (i64.sub (i64.sub (local.get $olen) (i64.const 1)) (local.get $i))))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $rev)))
  (local.get $out) (local.get $olen))                             ;; owned result
"#;

/// `__rt_str_alloc`: reserves an owned kind-1 string block of `bytes` capacity.
///
/// A re-encoding transform cannot size its result before it runs, so it reserves the worst case
/// here and returns the length it actually produced. The block stays at its reserved size, which
/// costs slack rather than correctness: a PHP string is the `(ptr, len)` pair, and the header the
/// release path reads is the reservation. `__rt_checked_layout` is what rejects a negative or
/// wasm32-overflowing size before the allocation rather than after.
const RT_STR_ALLOC: &str = r#"(func $__rt_str_alloc (param $bytes i64) (result i32)
  (local $new i32)                                                ;; reserved block
  (local.set $new
    (call $__rt_heap_alloc
      (call $__rt_checked_layout
        (local.get $bytes)
        (i64.const 1)
        (i64.const 0))))                                          ;; checked byte count -> block
  (i64.store (i32.sub (local.get $new) (i32.const 8)) (i64.const 1)) ;; stamp header kind = 1 (string)
  (local.get $new))                                               ;; reserved string block
"#;

/// `__rt_str_bin2hex`: owns the lowercase hex expansion of a string's bytes.
///
/// `bin2hex` is total and exactly doubles the length: every byte becomes its high then low
/// nibble as `0-9a-f`. The digit map is arithmetic rather than a table because `'a' - 10` is 87.
const RT_STR_BIN2HEX: &str = r#"(func $__rt_str_bin2hex (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $nib i32)                                                ;; nibble being written
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (i64.mul (local.get $len) (i64.const 2))))
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $hex
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte expanded
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (local.set $nib (i32.shr_u (local.get $byte) (i32.const 4)))  ;; high nibble first
    (i32.store8 (i32.add (local.get $out) (local.get $w))
      (i32.add (local.get $nib)
        (select (i32.const 48) (i32.const 87)
                (i32.lt_u (local.get $nib) (i32.const 10)))))     ;; 0-9 else a-f
    (local.set $w (i32.add (local.get $w) (i32.const 1)))         ;; w++
    (local.set $nib (i32.and (local.get $byte) (i32.const 15)))   ;; low nibble
    (i32.store8 (i32.add (local.get $out) (local.get $w))
      (i32.add (local.get $nib)
        (select (i32.const 48) (i32.const 87)
                (i32.lt_u (local.get $nib) (i32.const 10)))))     ;; 0-9 else a-f
    (local.set $w (i32.add (local.get $w) (i32.const 1)))         ;; w++
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $hex)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_addslashes`: owns a copy with PHP's four escaped bytes backslash-prefixed.
///
/// `addslashes` escapes exactly `'`, `"`, `\` and NUL, and NUL becomes the two characters
/// `\0` rather than a backslash plus a zero byte — measured against php-src, where
/// `"\x00"` comes back as the bytes `5c 30`. Every other byte, including UTF-8 continuation
/// bytes, passes through untouched. Worst case is two output bytes per input byte.
const RT_STR_ADDSLASHES: &str = r#"(func $__rt_str_addslashes (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (i64.mul (local.get $len) (i64.const 2))))
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $esc
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (if (i32.or
          (i32.eqz (local.get $byte))
          (i32.or
            (i32.eq (local.get $byte) (i32.const 39))
            (i32.or
              (i32.eq (local.get $byte) (i32.const 34))
              (i32.eq (local.get $byte) (i32.const 92)))))        ;; NUL, ' , " or backslash
      (then
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (i32.const 92))  ;; leading backslash
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (i32.store8 (i32.add (local.get $out) (local.get $w))
          (select (i32.const 48) (local.get $byte)
                  (i32.eqz (local.get $byte))))                   ;; NUL escapes to the digit zero
        (local.set $w (i32.add (local.get $w) (i32.const 1))))    ;; w++
      (else
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $byte))
        (local.set $w (i32.add (local.get $w) (i32.const 1)))))   ;; w++
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $esc)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_stripslashes`: owns a copy with one level of backslash escaping removed.
///
/// It is NOT the inverse of a C unescape: `\n` yields the letter `n`, not a newline. Only `\0`
/// is special, producing a NUL byte. A backslash consumes the byte after it whatever that is
/// (so `\\` yields one backslash), and a trailing lone backslash is dropped. Measured against
/// php-src, including `\\0` yielding a backslash followed by the digit zero.
const RT_STR_STRIPSLASHES: &str = r#"(func $__rt_str_stripslashes (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (local.get $len)))        ;; never grows
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $strip
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (if (i32.eq (local.get $byte) (i32.const 92))                 ;; a backslash escapes what follows
      (then
        (local.set $i (i64.add (local.get $i) (i64.const 1)))     ;; consume the backslash
        (if (i64.lt_s (local.get $i) (local.get $len))            ;; a trailing lone backslash is dropped
          (then
            (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
            (i32.store8 (i32.add (local.get $out) (local.get $w))
              (select (i32.const 0) (local.get $byte)
                      (i32.eq (local.get $byte) (i32.const 48)))) ;; \0 alone becomes a NUL byte
            (local.set $w (i32.add (local.get $w) (i32.const 1))) ;; w++
            (local.set $i (i64.add (local.get $i) (i64.const 1)))))) ;; consume the escaped byte
      (else
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $byte))
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (local.set $i (i64.add (local.get $i) (i64.const 1)))))   ;; i++
    (br $strip)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_nl2br`: owns a copy with `<br />` inserted BEFORE each line break.
///
/// The break itself is kept, which is what `nl2br` does — it inserts rather than replaces. A
/// `\r\n` or `\n\r` pair counts as ONE break and both bytes survive after the single tag;
/// `\n\n` is two breaks. Measured against php-src, whose default XHTML form is `<br />`.
/// Worst case is seven output bytes per input byte, when every byte is a lone break.
const RT_STR_NL2BR: &str = r#"(func $__rt_str_nl2br (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $next i32)                                               ;; byte after a line break
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (i64.mul (local.get $len) (i64.const 7))))
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $scan
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (if (i32.or (i32.eq (local.get $byte) (i32.const 10))
                (i32.eq (local.get $byte) (i32.const 13)))        ;; a line feed or carriage return
      (then
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (i32.const 60))   ;; <
        (i32.store8 offset=1 (i32.add (local.get $out) (local.get $w)) (i32.const 98))  ;; b
        (i32.store8 offset=2 (i32.add (local.get $out) (local.get $w)) (i32.const 114)) ;; r
        (i32.store8 offset=3 (i32.add (local.get $out) (local.get $w)) (i32.const 32))  ;; space
        (i32.store8 offset=4 (i32.add (local.get $out) (local.get $w)) (i32.const 47))  ;; /
        (i32.store8 offset=5 (i32.add (local.get $out) (local.get $w)) (i32.const 62))  ;; >
        (local.set $w (i32.add (local.get $w) (i32.const 6)))     ;; the six tag bytes
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $byte)) ;; keep the break
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (local.set $i (i64.add (local.get $i) (i64.const 1)))     ;; i++
        (if (i64.lt_s (local.get $i) (local.get $len))
          (then
            (local.set $next (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
            (if (i32.and
                  (i32.or (i32.eq (local.get $next) (i32.const 10))
                          (i32.eq (local.get $next) (i32.const 13)))
                  (i32.ne (local.get $next) (local.get $byte)))   ;; the OTHER break byte pairs with it
              (then
                (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $next))
                (local.set $w (i32.add (local.get $w) (i32.const 1)))  ;; w++
                (local.set $i (i64.add (local.get $i) (i64.const 1)))))))) ;; the pair is one break
      (else
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $byte))
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (local.set $i (i64.add (local.get $i) (i64.const 1)))))   ;; i++
    (br $scan)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// Returns whether a unary string transform is lowered by this module.
///
/// Admitted so far: the exact same-length BYTE transforms, plus the re-encoders whose rules are
/// pure byte arithmetic (hex expansion, backslash escaping, line-break tagging). Still out are
/// base64 and url coding, and the html entity decoders, which need a table.
pub(super) fn unary_string_is_supported(target: UnaryStringRuntime) -> bool {
    matches!(
        target,
        UnaryStringRuntime::StrToUpper
            | UnaryStringRuntime::StrToLower
            | UnaryStringRuntime::StrReverse
            | UnaryStringRuntime::BinToHex
            | UnaryStringRuntime::AddSlashes
            | UnaryStringRuntime::StripSlashes
            | UnaryStringRuntime::NlToBr
    )
}

/// Validates one unary string transform: a string in, a string out.
pub(super) fn unary_string_shape_issue(
    function: &Function,
    call: &Instruction,
    target: UnaryStringRuntime,
) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one string operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("string operand is missing from the value table".to_string());
    };
    if value.ir_type != IrType::Str || value.php_type.codegen_repr() != PhpType::Str {
        return Some(format!(
            "expected a string operand, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::Str
        || call.result_php_type.codegen_repr() != PhpType::Str
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected Str/Str",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers one unary string transform to its byte-mapping helper.
pub(super) fn lower_unary_string(
    ctx: &mut FnCtx,
    inst: &Instruction,
    target: UnaryStringRuntime,
) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    match target {
        UnaryStringRuntime::StrToUpper => {
            ctx.fb.ins("i32.const 1", "map towards upper case");
            ctx.fb
                .ins("call $__rt_str_map_case", "ASCII-only case mapping");
        }
        UnaryStringRuntime::StrToLower => {
            ctx.fb.ins("i32.const 0", "map towards lower case");
            ctx.fb
                .ins("call $__rt_str_map_case", "ASCII-only case mapping");
        }
        UnaryStringRuntime::StrReverse => {
            ctx.fb
                .ins("call $__rt_str_reverse", "reverse the bytes");
        }
        UnaryStringRuntime::BinToHex => {
            ctx.fb
                .ins("call $__rt_str_bin2hex", "expand each byte to two hex digits");
        }
        UnaryStringRuntime::AddSlashes => {
            ctx.fb
                .ins("call $__rt_str_addslashes", "escape quotes, backslash and NUL");
        }
        UnaryStringRuntime::StripSlashes => {
            ctx.fb
                .ins("call $__rt_str_stripslashes", "remove one level of backslash escaping");
        }
        UnaryStringRuntime::NlToBr => {
            ctx.fb
                .ins("call $__rt_str_nl2br", "insert a break tag before each line break");
        }
        other => {
            return Err(WasmError::Unsupported(format!(
                "unary string transform {:?}",
                other
            )))
        }
    }
    store_result(ctx, inst)
}

/// `__rt_str_region_eq`: compares `nlen` bytes of a needle against a haystack at `offset`.
///
/// The caller guarantees the region is in bounds, which every user below checks by comparing
/// lengths first. An empty needle matches anywhere, which is what makes `str_contains($h, "")`
/// true in PHP.
const RT_STR_REGION_EQ: &str = r#"(func $__rt_str_region_eq (param $hptr i32) (param $nptr i32) (param $nlen i64) (param $offset i64) (result i64)
  (local $i i64)
  (local.set $i (i64.const 0))                                    ;; i = 0
  (block $end (loop $cmp
    (br_if $end (i64.ge_s (local.get $i) (local.get $nlen)))      ;; every needle byte matched
    (if (i32.ne
          (i32.load8_u (i32.add (local.get $hptr)
                                (i32.wrap_i64 (i64.add (local.get $offset) (local.get $i)))))
          (i32.load8_u (i32.add (local.get $nptr) (i32.wrap_i64 (local.get $i)))))
      (then (return (i64.const 0))))                              ;; first mismatch decides
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $cmp)))
  (i64.const 1))                                                  ;; the whole needle matched
"#;

/// `__rt_str_contains`: whether the needle occurs anywhere in the haystack.
///
/// Scans every start offset that leaves room for the needle. A needle longer than the haystack
/// leaves none, so the answer is false without reading a byte; an empty needle matches at offset
/// zero, which PHP reports as true even for an empty haystack.
const RT_STR_CONTAINS: &str = r#"(func $__rt_str_contains (param $hptr i32) (param $hlen i64) (param $nptr i32) (param $nlen i64) (result i64)
  (local $offset i64)
  (local $last i64)
  (local.set $last (i64.sub (local.get $hlen) (local.get $nlen)))  ;; last start offset with room
  (if (i64.lt_s (local.get $last) (i64.const 0))
    (then (return (i64.const 0))))                                 ;; the needle cannot fit
  (local.set $offset (i64.const 0))                                ;; start at the beginning
  (block $end (loop $scan
    (br_if $end (i64.gt_s (local.get $offset) (local.get $last)))  ;; no room left
    (if (i64.eq (call $__rt_str_region_eq (local.get $hptr) (local.get $nptr) (local.get $nlen) (local.get $offset)) (i64.const 1))
      (then (return (i64.const 1))))                               ;; occurrence found
    (local.set $offset (i64.add (local.get $offset) (i64.const 1)))  ;; try the next offset
    (br $scan)))
  (i64.const 0))                                                   ;; no occurrence
"#;

/// The storage one direct builtin accepts and produces.
///
/// Both the audit and the emitter derive from this single description, which is what keeps a
/// newly admitted builtin from being auditable but unlowerable, or the reverse.
struct DirectSignature {
    /// EIR type every operand must carry.
    operand_ir: IrType,
    /// PHP type every operand must carry, after `codegen_repr`.
    operand_php: PhpType,
    /// EIR type the result must carry.
    result_ir: IrType,
    /// PHP type the result must carry, after `codegen_repr`.
    result_php: PhpType,
}

/// Returns the signature and WebAssembly instruction for a builtin lowered inline, or `None`
/// when the builtin needs a runtime helper.
///
/// `count` is absent from the instruction column because it is a memory load rather than an
/// arithmetic operation; it is handled separately by [`lower_count`].
fn direct_builtin(target: RuntimeFnId, operand_php: &PhpType) -> Option<(DirectSignature, &'static str)> {
    let float = |instruction| {
        Some((
            DirectSignature {
                operand_ir: IrType::F64,
                operand_php: PhpType::Float,
                result_ir: IrType::F64,
                result_php: PhpType::Float,
            },
            instruction,
        ))
    };
    match target {
        // `abs` is the one entry whose storage depends on its argument: PHP keeps an integer
        // argument integral and a float one floating.
        RuntimeFnId::Abs => match operand_php {
            PhpType::Int => Some((
                DirectSignature {
                    operand_ir: IrType::I64,
                    operand_php: PhpType::Int,
                    result_ir: IrType::I64,
                    result_php: PhpType::Int,
                },
                // WebAssembly has no i64 absolute value; the branchless form is
                // `(x ^ (x >> 63)) - (x >> 63)`, emitted by `lower_int_abs`.
                "",
            )),
            PhpType::Float => float("f64.abs"),
            _ => None,
        },
        RuntimeFnId::Floor => float("f64.floor"),
        RuntimeFnId::Ceil => float("f64.ceil"),
        RuntimeFnId::Sqrt => float("f64.sqrt"),
        _ => None,
    }
}

/// Returns whether `target` is lowered inline by this module.
pub(super) fn is_direct_builtin(target: RuntimeFnId) -> bool {
    matches!(
        target,
        RuntimeFnId::Abs
            | RuntimeFnId::Floor
            | RuntimeFnId::Ceil
            | RuntimeFnId::Sqrt
            | RuntimeFnId::Count
            | RuntimeFnId::ArrayIsList
            | RuntimeFnId::ArrayKeys
            | RuntimeFnId::ArrayValues
            | RuntimeFnId::InArray
            | RuntimeFnId::ArrayReverse
            | RuntimeFnId::ArraySum
            | RuntimeFnId::ArrayProduct
            | RuntimeFnId::Max
            | RuntimeFnId::Min
            | RuntimeFnId::Intdiv
            | RuntimeFnId::ArrayFill
            | RuntimeFnId::StrContains
            | RuntimeFnId::StrStartsWith
            | RuntimeFnId::StrEndsWith
    )
}

/// Returns whether `value` is an indexed array whose slots this module can read directly.
///
/// Everything in the array family below reads raw i64 slots, which is what an `array<int>`
/// stores. A string or mixed element array uses a different slot width and carries refcounted
/// payloads, so it is not served here rather than being read at the wrong stride.
///
/// `array<never>` is admitted alongside `array<int>`: it is the type of the empty array literal,
/// and an array that provably holds nothing is read at no stride at all — every operation here
/// answers from its length, which is zero.
fn indexed_int_array(value: &crate::ir::Value) -> bool {
    value.ir_type == IrType::Heap(IrHeapKind::Array)
        && matches!(
            value.php_type.codegen_repr(),
            PhpType::Array(element) if matches!(*element, PhpType::Int | PhpType::Never)
        )
}

/// Validates one direct builtin's operand and result storage before planning.
pub(super) fn direct_builtin_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    if target == RuntimeFnId::Count {
        return count_shape_issue(function, call);
    }
    if target == RuntimeFnId::ArrayIsList {
        return array_is_list_shape_issue(function, call);
    }
    if matches!(
        target,
        RuntimeFnId::ArrayKeys | RuntimeFnId::ArrayValues | RuntimeFnId::ArrayReverse
    ) {
        return indexed_array_result_shape_issue(function, call, target);
    }
    if matches!(target, RuntimeFnId::ArraySum | RuntimeFnId::ArrayProduct) {
        return array_fold_shape_issue(function, call, target);
    }
    if matches!(target, RuntimeFnId::Max | RuntimeFnId::Min | RuntimeFnId::Intdiv) {
        return int_pair_shape_issue(function, call, target);
    }
    if target == RuntimeFnId::ArrayFill {
        return array_fill_shape_issue(function, call);
    }
    if matches!(
        target,
        RuntimeFnId::StrContains | RuntimeFnId::StrStartsWith | RuntimeFnId::StrEndsWith
    ) {
        return string_predicate_shape_issue(function, call, target);
    }
    if target == RuntimeFnId::InArray {
        return in_array_shape_issue(function, call);
    }
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("operand is missing from the value table".to_string());
    };
    let operand_php = value.php_type.codegen_repr();
    let Some((signature, _)) = direct_builtin(target, &operand_php) else {
        return Some(format!(
            "no inline lowering for a {operand_php:?} argument"
        ));
    };
    if value.ir_type != signature.operand_ir || operand_php != signature.operand_php {
        return Some(format!(
            "operand {:?}/{operand_php:?} is not the expected {:?}/{:?}",
            value.ir_type, signature.operand_ir, signature.operand_php
        ));
    }
    if call.result.is_none()
        || call.result_type != signature.result_ir
        || call.result_php_type.codegen_repr() != signature.result_php
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected {:?}/{:?}",
            call.result_type,
            call.result_php_type.codegen_repr(),
            signature.result_ir,
            signature.result_php
        ));
    }
    None
}

/// Validates `count($array)` against the one shape its load can serve.
///
/// The length is read straight from the container header, so the operand has to be a container
/// this backend allocated. PHP's `count()` of a non-countable value is a `TypeError`, which a
/// header load cannot raise, so any other operand type is refused rather than answering nonsense.
fn count_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one container operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("container operand is missing from the value table".to_string());
    };
    if !matches!(
        value.ir_type,
        IrType::Heap(IrHeapKind::Array | IrHeapKind::Hash)
    ) || !matches!(
        value.php_type.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return Some(format!(
            "expected a statically typed array or hash, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected I64/Int",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers one direct builtin.
pub(super) fn lower_direct_builtin(
    ctx: &mut FnCtx,
    inst: &Instruction,
    target: RuntimeFnId,
) -> Result<()> {
    if target == RuntimeFnId::Count {
        return lower_count(ctx, inst);
    }
    if target == RuntimeFnId::ArrayIsList {
        return lower_array_is_list(ctx, inst);
    }
    if target == RuntimeFnId::ArrayKeys {
        return lower_array_keys(ctx, inst);
    }
    if target == RuntimeFnId::ArrayValues {
        return lower_array_values(ctx, inst);
    }
    if target == RuntimeFnId::ArrayReverse {
        return lower_array_reverse(ctx, inst);
    }
    if matches!(target, RuntimeFnId::ArraySum | RuntimeFnId::ArrayProduct) {
        return lower_array_fold(ctx, inst, target);
    }
    if matches!(target, RuntimeFnId::Max | RuntimeFnId::Min) {
        return lower_int_extremum(ctx, inst, target);
    }
    if target == RuntimeFnId::Intdiv {
        return super::inst::lower_signed_int_div(ctx, inst);
    }
    if target == RuntimeFnId::ArrayFill {
        return lower_array_fill(ctx, inst);
    }
    if matches!(
        target,
        RuntimeFnId::StrContains | RuntimeFnId::StrStartsWith | RuntimeFnId::StrEndsWith
    ) {
        return lower_string_predicate(ctx, inst, target);
    }
    if target == RuntimeFnId::InArray {
        return lower_in_array(ctx, inst);
    }
    let argument = operand(inst, 0)?;
    let operand_php = ctx.value_php_type(argument)?.codegen_repr();
    let Some((_, instruction)) = direct_builtin(target, &operand_php) else {
        return Err(WasmError::Unsupported(format!(
            "builtin {:?} over a {operand_php:?} argument",
            target
        )));
    };
    if target == RuntimeFnId::Abs && operand_php == PhpType::Int {
        return lower_int_abs(ctx, inst, argument);
    }
    ctx.emit_load_value(argument)?;
    ctx.fb.ins(instruction, "PHP builtin lowered inline");
    store_result(ctx, inst)
}

/// Lowers `abs($int)` branchlessly as `(x ^ (x >> 63)) - (x >> 63)`.
///
/// KNOWN DIVERGENCE, shared with the native backend and rooted in EIR rather than in either
/// emitter: PHP promotes `abs(PHP_INT_MIN)` to the float `9.2233720368548E+18`, because its
/// magnitude has no integer representation. EIR types this call `I64`/`int`, so there is no slot
/// a float could be returned in, and both backends therefore answer `PHP_INT_MIN` unchanged.
/// Every other input is exact.
fn lower_int_abs(ctx: &mut FnCtx, inst: &Instruction, argument: crate::ir::ValueId) -> Result<()> {
    let mask = ctx.fresh_temp(super::wat::ValType::I64);
    ctx.emit_load_value(argument)?;
    ctx.fb.ins("i64.const 63", "sign-bit shift distance");
    ctx.fb
        .ins("i64.shr_s", "all ones for a negative argument, zero otherwise");
    ctx.fb.ins(&format!("local.tee {}", mask), "keep the sign mask");
    ctx.emit_load_value(argument)?;
    ctx.fb.ins("i64.xor", "conditionally invert the argument");
    ctx.fb.ins(&format!("local.get {}", mask), "the sign mask again");
    ctx.fb.ins("i64.sub", "add one back for a negative argument");
    store_result(ctx, inst)
}

/// Validates `array_is_list($array)` against the one operand whose answer is known statically.
///
/// This backend's `Heap(Array)` IS the contiguous representation: its keys are `0..n-1` in order
/// by construction, which is exactly PHP's definition of a list, and an empty array qualifies.
/// A `Heap(Hash)` carries arbitrary keys and would need a real scan, so it is refused rather
/// than answered from the representation.
fn array_is_list_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one array operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("array operand is missing from the value table".to_string());
    };
    if value.ir_type != IrType::Heap(IrHeapKind::Array)
        || !matches!(value.php_type.codegen_repr(), PhpType::Array(_))
    {
        return Some(format!(
            "expected a statically typed indexed array, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Bool
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected I64/Bool",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers `array_is_list($indexed)` to the constant true its representation guarantees.
///
/// The operand is still evaluated and dropped: it may be a call whose side effects PHP performs
/// before answering, and discarding the expression rather than the value would skip them.
fn lower_array_is_list(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb
        .ins("drop", "the answer follows from the representation, not the contents");
    ctx.fb
        .ins("i64.const 1", "an indexed array is a list by construction");
    store_result(ctx, inst)
}

/// Validates `array_keys($list)` and `array_values($list)`, which both answer a fresh
/// `array<int>` built from an `array<int>`.
///
/// `array_keys` of a list is `[0, 1, ..., n-1]` because its keys ARE its positions, and
/// `array_values` of a list is a copy because re-indexing a list changes nothing. Both facts
/// hold only for the indexed representation, so a hash operand is refused.
fn indexed_array_result_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one array operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("array operand is missing from the value table".to_string());
    };
    if !indexed_int_array(value) {
        return Some(format!(
            "expected a statically typed array<int>, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::Heap(IrHeapKind::Array)
        || !matches!(
            call.result_php_type.codegen_repr(),
            PhpType::Array(element) if matches!(*element, PhpType::Int | PhpType::Never)
        )
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected Heap(Array)/array<int>",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates `in_array($needle, $haystack, true)` against the one comparison the scan performs.
///
/// The third argument must be the literal `true`: PHP's LOOSE form applies type juggling that an
/// identity comparison over raw slots does not, so admitting a runtime-valued or absent `$strict`
/// would answer the wrong question whenever it turned out to be false.
fn in_array_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [needle, haystack, strict] = call.operands.as_slice() else {
        return Some(format!(
            "expected needle, haystack and an explicit strict flag, got {} operands",
            call.operands.len()
        ));
    };
    let Some(needle) = function.value(*needle) else {
        return Some("needle is missing from the value table".to_string());
    };
    if needle.ir_type != IrType::I64 || needle.php_type.codegen_repr() != PhpType::Int {
        return Some(format!(
            "expected an int needle, got {:?}/{:?}",
            needle.ir_type,
            needle.php_type.codegen_repr()
        ));
    }
    let Some(haystack) = function.value(*haystack) else {
        return Some("haystack is missing from the value table".to_string());
    };
    if !indexed_int_array(haystack) {
        return Some(format!(
            "expected a statically typed array<int> haystack, got {:?}/{:?}",
            haystack.ir_type,
            haystack.php_type.codegen_repr()
        ));
    }
    if !literal_true(function, *strict) {
        return Some("only the strict form is lowered; $strict must be the literal true".to_string());
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Bool
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected I64/Bool",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Returns whether `value` is the constant `true`, following ownership forwarding.
fn literal_true(function: &Function, value: crate::ir::ValueId) -> bool {
    let Some(defined) = function.value(value) else {
        return false;
    };
    let crate::ir::ValueDef::Instruction { inst, .. } = defined.def else {
        return false;
    };
    let Some(defining) = function.instruction(inst) else {
        return false;
    };
    defining.op == crate::ir::Op::ConstBool
        && matches!(defining.immediate, Some(crate::ir::Immediate::Bool(true)))
}

/// Lowers `array_keys($list)` to the positional key array its representation implies.
fn lower_array_keys(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb
        .ins("call $__rt_array_index_keys", "keys of a list are its positions");
    store_result(ctx, inst)
}

/// Lowers `array_values($list)` to a shallow clone.
///
/// Re-indexing a list changes nothing, so the values are the source's in order; the clone is
/// what makes the result an independent owned array rather than an alias of the source.
fn lower_array_values(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb
        .ins("call $__rt_array_clone_shallow", "values of a list are the list itself");
    store_result(ctx, inst)
}

/// Validates `array_sum($list)` / `array_product($list)`, which fold an `array<int>` to an int.
fn array_fold_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one array operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("array operand is missing from the value table".to_string());
    };
    if !indexed_int_array(value) {
        return Some(format!(
            "expected a statically typed array<int>, got {:?}/{:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected I64/Int",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers `array_reverse($list)` to a reversed copy.
fn lower_array_reverse(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb.ins(
        "call $__rt_array_reverse_int",
        "reversing a list re-indexes it from zero",
    );
    store_result(ctx, inst)
}

/// Lowers `array_sum($list)` / `array_product($list)` to their accumulating scan.
///
/// KNOWN DIVERGENCE, shared with the native backend and rooted in the checker rather than in
/// either emitter: PHP promotes an overflowing sum or product to a float, so
/// `array_sum([PHP_INT_MAX, 1])` is `9.2233720368548E+18`. The checker types this call `int`,
/// leaving no slot a float could be returned in, and both backends therefore wrap. Closing it
/// means widening the declared result to `int|float`, which is an EIR-level change.
fn lower_array_fold(ctx: &mut FnCtx, inst: &Instruction, target: RuntimeFnId) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    let (helper, comment) = if target == RuntimeFnId::ArraySum {
        ("call $__rt_array_sum_int", "PHP sums an empty array to 0")
    } else {
        ("call $__rt_array_product_int", "PHP's empty product is 1")
    };
    ctx.fb.ins(helper, comment);
    store_result(ctx, inst)
}

/// Validates the builtins taking two ints and answering one: `max`, `min` and `intdiv`.
///
/// PHP's `max`/`min` are variadic and compare across types; only the two-integer form is served
/// here, where the comparison is a plain signed ordering with no juggling.
fn int_pair_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [left, right] = call.operands.as_slice() else {
        return Some(format!(
            "expected two int operands, got {}",
            call.operands.len()
        ));
    };
    for operand in [left, right] {
        let Some(value) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        if value.ir_type != IrType::I64 || value.php_type.codegen_repr() != PhpType::Int {
            return Some(format!(
                "expected an int operand, got {:?}/{:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Int
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected I64/Int",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates `array_fill(0, $count, $value)` against the one shape a list can represent.
///
/// A non-zero start index produces the keys `start..start+count-1`, which is not a list, so the
/// start must be the literal `0`. The value must be an int because the result's slots are raw
/// i64s.
fn array_fill_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [start, count, value] = call.operands.as_slice() else {
        return Some(format!(
            "expected start, count and value, got {} operands",
            call.operands.len()
        ));
    };
    if !literal_zero(function, *start) {
        return Some(
            "only a literal 0 start index yields a list; other starts key from the start index"
                .to_string(),
        );
    }
    for operand in [count, value] {
        let Some(operand) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        if operand.ir_type != IrType::I64 || operand.php_type.codegen_repr() != PhpType::Int {
            return Some(format!(
                "expected an int operand, got {:?}/{:?}",
                operand.ir_type,
                operand.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::Heap(IrHeapKind::Array)
        || !matches!(
            call.result_php_type.codegen_repr(),
            PhpType::Array(element) if matches!(*element, PhpType::Int | PhpType::Never)
        )
    {
        return Some(format!(
            "result {:?}/{:?} is not the expected Heap(Array)/array<int>",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Returns whether `value` is the constant integer zero.
fn literal_zero(function: &Function, value: crate::ir::ValueId) -> bool {
    let Some(defined) = function.value(value) else {
        return false;
    };
    let crate::ir::ValueDef::Instruction { inst, .. } = defined.def else {
        return false;
    };
    let Some(defining) = function.instruction(inst) else {
        return false;
    };
    defining.op == crate::ir::Op::ConstI64
        && matches!(defining.immediate, Some(crate::ir::Immediate::I64(0)))
}

/// Lowers the two-integer `max` / `min` to a signed comparison and a select.
fn lower_int_extremum(ctx: &mut FnCtx, inst: &Instruction, target: RuntimeFnId) -> Result<()> {
    let left = operand(inst, 0)?;
    let right = operand(inst, 1)?;
    ctx.emit_load_value(left)?;
    ctx.emit_load_value(right)?;
    ctx.emit_load_value(left)?;
    ctx.emit_load_value(right)?;
    if target == RuntimeFnId::Max {
        ctx.fb.ins("i64.gt_s", "is the left operand the larger?");
    } else {
        ctx.fb.ins("i64.lt_s", "is the left operand the smaller?");
    }
    ctx.fb
        .ins("select", "keep the operand the comparison chose");
    store_result(ctx, inst)
}

/// Lowers `array_fill(0, $count, $value)` to a filled list.
fn lower_array_fill(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 1)?)?;
    ctx.emit_load_value(operand(inst, 2)?)?;
    ctx.fb
        .ins("call $__rt_array_fill_int", "one repeated value per slot");
    store_result(ctx, inst)
}

/// Validates `str_contains`, `str_starts_with` and `str_ends_with`: two strings in, a bool out.
fn string_predicate_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [haystack, needle] = call.operands.as_slice() else {
        return Some(format!(
            "expected a haystack and a needle, got {} operands",
            call.operands.len()
        ));
    };
    for operand in [haystack, needle] {
        let Some(value) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        if value.ir_type != IrType::Str || value.php_type.codegen_repr() != PhpType::Str {
            return Some(format!(
                "expected a string operand, got {:?}/{:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::I64
        || call.result_php_type.codegen_repr() != PhpType::Bool
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected I64/Bool",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Lowers the three PHP 8 substring predicates to a byte comparison.
///
/// `str_starts_with` and `str_ends_with` compare ONE region, so they check the needle fits and
/// then compare at offset zero or at `hlen - nlen`. `str_contains` scans every offset that
/// leaves room. An empty needle matches in all three, which is PHP's answer.
fn lower_string_predicate(
    ctx: &mut FnCtx,
    inst: &Instruction,
    target: RuntimeFnId,
) -> Result<()> {
    let haystack = operand(inst, 0)?;
    let needle = operand(inst, 1)?;
    let (hptr, hlen) = match ctx.value_repr(haystack)?.clone() {
        super::values::WasmRepr::Str { ptr, len } => (ptr, len),
        other => {
            return Err(WasmError::Unsupported(format!(
                "string predicate haystack is {:?}",
                other
            )))
        }
    };
    let (nptr, nlen) = match ctx.value_repr(needle)?.clone() {
        super::values::WasmRepr::Str { ptr, len } => (ptr, len),
        other => {
            return Err(WasmError::Unsupported(format!(
                "string predicate needle is {:?}",
                other
            )))
        }
    };
    if target == RuntimeFnId::StrContains {
        ctx.fb.ins(&format!("local.get {}", hptr), "haystack pointer");
        ctx.fb.ins(&format!("local.get {}", hlen), "haystack length");
        ctx.fb.ins(&format!("local.get {}", nptr), "needle pointer");
        ctx.fb.ins(&format!("local.get {}", nlen), "needle length");
        ctx.fb
            .ins("call $__rt_str_contains", "scan every start offset");
        return store_result(ctx, inst);
    }
    // A needle longer than the haystack cannot match at any single offset, and the comparison
    // would read past the end, so the length check has to come first.
    ctx.fb.ins(&format!("local.get {}", nlen), "needle length");
    ctx.fb.ins(&format!("local.get {}", hlen), "haystack length");
    ctx.fb.ins("i64.gt_s", "does the needle overrun the haystack?");
    ctx.fb.ins("if (result i64)", "needle too long");
    ctx.fb.ins("i64.const 0", "an overrunning needle never matches");
    ctx.fb.ins("else", "the needle fits");
    ctx.fb.ins(&format!("local.get {}", hptr), "haystack pointer");
    ctx.fb.ins(&format!("local.get {}", nptr), "needle pointer");
    ctx.fb.ins(&format!("local.get {}", nlen), "needle length");
    if target == RuntimeFnId::StrStartsWith {
        ctx.fb.ins("i64.const 0", "compare at the start");
    } else {
        ctx.fb.ins(&format!("local.get {}", hlen), "haystack length");
        ctx.fb.ins(&format!("local.get {}", nlen), "needle length");
        ctx.fb.ins("i64.sub", "compare at the trailing region");
    }
    ctx.fb
        .ins("call $__rt_str_region_eq", "compare the one candidate region");
    ctx.fb.ins("end", "end needle-length guard");
    store_result(ctx, inst)
}

/// Lowers strict `in_array($needle, $haystack, true)` to an identity scan.
fn lower_in_array(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 1)?)?;
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb
        .ins("call $__rt_array_contains_int", "strict in_array over raw i64 slots");
    store_result(ctx, inst)
}

/// Lowers `count($array)` to the container header's element count at `[ptr + 0]`.
fn lower_count(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb.ins("i64.load", "container element count @ +0");
    store_result(ctx, inst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Builder, Immediate, Op, Ownership, RuntimeCallTarget};

    /// Builds a one-instruction function calling `target` with one operand of the given storage.
    fn call_with(
        target: RuntimeFnId,
        operand_ir: IrType,
        operand_php: PhpType,
        result_ir: IrType,
        result_php: PhpType,
    ) -> Function {
        let mut function = Function::new("probe".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let slot = builder.add_local(
                Some("v".to_string()),
                operand_ir,
                operand_php.clone(),
                crate::ir::LocalKind::PhpLocal,
            );
            let argument = builder.emit_load_local(slot, operand_ir, operand_php);
            builder.emit(
                Op::RuntimeCall,
                vec![argument],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(target))),
                result_ir,
                result_php,
                Ownership::NonHeap,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Returns the audit verdict for the last instruction of `function`.
    fn verdict(function: &Function, target: RuntimeFnId) -> Option<String> {
        let call = function
            .instructions
            .last()
            .expect("the probe emitted a call");
        direct_builtin_shape_issue(function, call, target)
    }

    /// Builds `in_array($needle, $haystack, $strict)` with the given strict operand.
    fn in_array_call(strict_is_literal_true: bool) -> Function {
        let mut function = Function::new("probe".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let needle_slot = builder.add_local(
                Some("n".to_string()),
                IrType::I64,
                PhpType::Int,
                crate::ir::LocalKind::PhpLocal,
            );
            let haystack_slot = builder.add_local(
                Some("a".to_string()),
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                crate::ir::LocalKind::PhpLocal,
            );
            let needle = builder.emit_load_local(needle_slot, IrType::I64, PhpType::Int);
            let haystack = builder.emit_load_local(
                haystack_slot,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
            );
            let strict = if strict_is_literal_true {
                builder.emit_const_bool(true)
            } else {
                builder.emit_const_bool(false)
            };
            builder.emit(
                Op::RuntimeCall,
                vec![needle, haystack, strict],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(
                    RuntimeFnId::InArray,
                ))),
                IrType::I64,
                PhpType::Bool,
                Ownership::NonHeap,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Builds a unary string transform call with the given operand storage.
    fn unary_string_call(
        target: UnaryStringRuntime,
        operand_ir: IrType,
        operand_php: PhpType,
    ) -> Function {
        let mut function = Function::new("probe".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let slot = builder.add_local(
                Some("s".to_string()),
                operand_ir,
                operand_php.clone(),
                crate::ir::LocalKind::PhpLocal,
            );
            let argument = builder.emit_load_local(slot, operand_ir, operand_php);
            builder.emit(
                Op::RuntimeCall,
                vec![argument],
                Some(Immediate::RuntimeCall(RuntimeCallTarget::UnaryString(
                    target,
                ))),
                IrType::Str,
                PhpType::Str,
                Ownership::MaybeOwned,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Verifies the admitted unary string transforms take a string and nothing else.
    #[test]
    fn unary_string_transforms_admit_only_strings() {
        for target in [
            UnaryStringRuntime::StrToUpper,
            UnaryStringRuntime::StrToLower,
            UnaryStringRuntime::StrReverse,
            UnaryStringRuntime::BinToHex,
            UnaryStringRuntime::AddSlashes,
            UnaryStringRuntime::StripSlashes,
            UnaryStringRuntime::NlToBr,
        ] {
            assert!(unary_string_is_supported(target), "{target:?} is lowered");
            let ok = unary_string_call(target, IrType::Str, PhpType::Str);
            let call = ok.instructions.last().expect("the probe emitted a call");
            assert_eq!(unary_string_shape_issue(&ok, call, target), None);

            let scalar = unary_string_call(target, IrType::I64, PhpType::Int);
            let call = scalar.instructions.last().expect("the probe emitted a call");
            assert!(
                unary_string_shape_issue(&scalar, call, target).is_some(),
                "{target:?} maps string bytes"
            );
        }

        // Base64 and url coding still need their own alphabets, and the html decoders a table.
        assert!(!unary_string_is_supported(UnaryStringRuntime::Base64Encode));
        assert!(!unary_string_is_supported(UnaryStringRuntime::UrlEncode));
        assert!(!unary_string_is_supported(
            UnaryStringRuntime::HtmlEntityDecode
        ));
    }

    /// Verifies every lowered unary string transform reserves enough room for its own output.
    ///
    /// A re-encoder allocates a worst case up front and reports the length it actually wrote, so
    /// a reservation smaller than the expansion would corrupt the heap block after it rather
    /// than fail visibly. Each factor is read back out of the emitted helper so a helper whose
    /// escaping grows without its reservation growing cannot pass.
    #[test]
    fn re_encoding_helpers_reserve_their_worst_case_expansion() {
        for (helper, factor, transform) in [
            (RT_STR_BIN2HEX, 2, "bin2hex writes two hex digits per byte"),
            (
                RT_STR_ADDSLASHES,
                2,
                "addslashes writes a backslash before an escaped byte",
            ),
            (
                RT_STR_NL2BR,
                7,
                "nl2br writes a six-byte tag before a kept break",
            ),
        ] {
            assert!(
                helper.contains(&format!(
                    "(call $__rt_str_alloc (i64.mul (local.get $len) (i64.const {factor})))"
                )),
                "{transform}, so it must reserve {factor} bytes per input byte:\n{helper}"
            );
        }
        // stripslashes only ever removes bytes, so the source length is already its worst case.
        assert!(RT_STR_STRIPSLASHES.contains("(call $__rt_str_alloc (local.get $len))"));
        for helper in [
            RT_STR_BIN2HEX,
            RT_STR_ADDSLASHES,
            RT_STR_STRIPSLASHES,
            RT_STR_NL2BR,
        ] {
            assert!(
                helper.contains("(i64.extend_i32_u (local.get $w))"),
                "a re-encoder returns what it WROTE, not what it reserved:\n{helper}"
            );
        }
    }

    /// Verifies `in_array` is lowered only in its STRICT form.
    ///
    /// PHP's loose comparison applies type juggling that an identity scan over raw slots does
    /// not perform, so a `$strict` that is not the literal `true` has to be refused rather than
    /// answered with the wrong comparison.
    #[test]
    fn in_array_is_admitted_only_when_strict_is_literally_true() {
        let strict = in_array_call(true);
        assert_eq!(verdict(&strict, RuntimeFnId::InArray), None);

        let loose = in_array_call(false);
        assert!(
            verdict(&loose, RuntimeFnId::InArray).is_some(),
            "the loose form needs PHP's juggling rules, which this scan does not implement"
        );
    }

    /// Verifies each inline builtin admits exactly the storage its lowering can emit.
    ///
    /// `RuntimeFnId::Floor`, `RuntimeFnId::Ceil` and `RuntimeFnId::Sqrt` are float-only;
    /// `RuntimeFnId::Abs` accepts both widths and must keep an integral argument integral; and
    /// `RuntimeFnId::Count` reads a container header, so a scalar operand has to be refused
    /// rather than loading whatever lies at that address.
    #[test]
    fn direct_builtins_admit_only_the_storage_they_lower() {
        for target in [RuntimeFnId::Floor, RuntimeFnId::Ceil, RuntimeFnId::Sqrt] {
            let ok = call_with(target, IrType::F64, PhpType::Float, IrType::F64, PhpType::Float);
            assert_eq!(verdict(&ok, target), None, "{target:?} over a float");

            let bad = call_with(target, IrType::I64, PhpType::Int, IrType::I64, PhpType::Int);
            assert!(
                verdict(&bad, target).is_some(),
                "{target:?} has no integral lowering"
            );
        }

        let int_abs = call_with(
            RuntimeFnId::Abs,
            IrType::I64,
            PhpType::Int,
            IrType::I64,
            PhpType::Int,
        );
        assert_eq!(verdict(&int_abs, RuntimeFnId::Abs), None);
        let float_abs = call_with(
            RuntimeFnId::Abs,
            IrType::F64,
            PhpType::Float,
            IrType::F64,
            PhpType::Float,
        );
        assert_eq!(verdict(&float_abs, RuntimeFnId::Abs), None);
        let widened_abs = call_with(
            RuntimeFnId::Abs,
            IrType::I64,
            PhpType::Int,
            IrType::F64,
            PhpType::Float,
        );
        assert!(
            verdict(&widened_abs, RuntimeFnId::Abs).is_some(),
            "an integral argument must not claim a float result"
        );

        let counted = call_with(
            RuntimeFnId::Count,
            IrType::Heap(IrHeapKind::Array),
            PhpType::Array(Box::new(PhpType::Int)),
            IrType::I64,
            PhpType::Int,
        );
        assert_eq!(verdict(&counted, RuntimeFnId::Count), None);
        let listed = call_with(
            RuntimeFnId::ArrayIsList,
            IrType::Heap(IrHeapKind::Array),
            PhpType::Array(Box::new(PhpType::Int)),
            IrType::I64,
            PhpType::Bool,
        );
        assert_eq!(verdict(&listed, RuntimeFnId::ArrayIsList), None);
        let hashed = call_with(
            RuntimeFnId::ArrayIsList,
            IrType::Heap(IrHeapKind::Hash),
            PhpType::AssocArray {
                key: Box::new(PhpType::Str),
                value: Box::new(PhpType::Int),
            },
            IrType::I64,
            PhpType::Bool,
        );
        assert!(
            verdict(&hashed, RuntimeFnId::ArrayIsList).is_some(),
            "a hash needs a real scan, not an answer from the representation"
        );

        // The array family reads raw i64 slots, so it accepts `array<int>` and the empty
        // `array<never>`, and nothing else.
        for target in [
            RuntimeFnId::ArrayKeys,
            RuntimeFnId::ArrayValues,
            RuntimeFnId::ArrayReverse,
        ] {
            let ok = call_with(
                target,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
            );
            assert_eq!(verdict(&ok, target), None, "{target:?} over array<int>");

            let stringly = call_with(
                target,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Str)),
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Str)),
            );
            assert!(
                verdict(&stringly, target).is_some(),
                "{target:?} must not read string slots at the integer stride"
            );
        }

        // `RuntimeFnId::ArraySum` and `RuntimeFnId::ArrayProduct` fold to an int, so they take
        // an array<int> and answer I64/Int.
        for target in [RuntimeFnId::ArraySum, RuntimeFnId::ArrayProduct] {
            let ok = call_with(
                target,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Int)),
                IrType::I64,
                PhpType::Int,
            );
            assert_eq!(verdict(&ok, target), None, "{target:?} over array<int>");

            let floaty = call_with(
                target,
                IrType::Heap(IrHeapKind::Array),
                PhpType::Array(Box::new(PhpType::Float)),
                IrType::F64,
                PhpType::Float,
            );
            assert!(
                verdict(&floaty, target).is_some(),
                "{target:?} folds integer slots only"
            );
        }

        // `RuntimeFnId::Max`, `RuntimeFnId::Min` and `RuntimeFnId::Intdiv` take two ints; the
        // variadic and cross-type forms of max/min are not served.
        for target in [RuntimeFnId::Max, RuntimeFnId::Min, RuntimeFnId::Intdiv] {
            let single = call_with(
                target,
                IrType::I64,
                PhpType::Int,
                IrType::I64,
                PhpType::Int,
            );
            assert!(
                verdict(&single, target).is_some(),
                "{target:?} needs exactly two operands"
            );
        }

        // `RuntimeFnId::ArrayFill` needs its start index, count and value; one operand is not it.
        let short_fill = call_with(
            RuntimeFnId::ArrayFill,
            IrType::I64,
            PhpType::Int,
            IrType::Heap(IrHeapKind::Array),
            PhpType::Array(Box::new(PhpType::Int)),
        );
        assert!(
            verdict(&short_fill, RuntimeFnId::ArrayFill).is_some(),
            "array_fill takes three operands"
        );

        // `RuntimeFnId::StrContains`, `RuntimeFnId::StrStartsWith` and `RuntimeFnId::StrEndsWith`
        // take two strings; a scalar operand has no bytes to compare.
        for target in [
            RuntimeFnId::StrContains,
            RuntimeFnId::StrStartsWith,
            RuntimeFnId::StrEndsWith,
        ] {
            let scalar = call_with(target, IrType::I64, PhpType::Int, IrType::I64, PhpType::Bool);
            assert!(
                verdict(&scalar, target).is_some(),
                "{target:?} compares string bytes"
            );
        }

        let scalar_count = call_with(
            RuntimeFnId::Count,
            IrType::I64,
            PhpType::Int,
            IrType::I64,
            PhpType::Int,
        );
        assert!(
            verdict(&scalar_count, RuntimeFnId::Count).is_some(),
            "count() of a scalar is a PHP TypeError a header load cannot raise"
        );
    }
}

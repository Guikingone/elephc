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
/// Emitted for every module: none of these touch WASI directly. `has_main` selects whether
/// `chr`/`ord` can reach the PHP 8.5 deprecation helpers, which do — a reactor gets the same
/// answer with no diagnostic rather than a reference it cannot resolve.
pub(super) fn emit_builtin_runtime(wm: &mut WatModule, has_main: bool) {
    let diagnoses = has_main
        && matches!(
            crate::codegen_support::compile_php_version(),
            crate::web_prelude::PhpVersion::Php85
        );
    let (chr, ord) = str_chr_ord(diagnoses);
    wm.add_raw_func(&chr);
    wm.add_raw_func(&ord);
    wm.add_raw_func(RT_STR_REGION_EQ);
    wm.add_raw_func(RT_STR_CONTAINS);
    wm.add_raw_func(RT_STR_MAP_CASE);
    wm.add_raw_func(RT_STR_REVERSE);
    wm.add_raw_func(RT_STR_ALLOC);
    wm.add_raw_func(RT_STR_BIN2HEX);
    wm.add_raw_func(RT_STR_ADDSLASHES);
    wm.add_raw_func(RT_STR_STRIPSLASHES);
    wm.add_raw_func(RT_STR_NL2BR);
    wm.add_raw_func(RT_HEX_DIGIT_VALUE);
    wm.add_raw_func(RT_STR_URL_ENCODE);
    wm.add_raw_func(RT_STR_URL_DECODE);
    wm.add_raw_func(RT_B64_CHAR);
    wm.add_raw_func(RT_B64_VALUE);
    wm.add_raw_func(RT_STR_BASE64_ENCODE);
    wm.add_raw_func(RT_STR_BASE64_DECODE);
    wm.add_raw_func(RT_STR_CASE_EDGE);
    wm.add_raw_func(RT_STR_UCWORDS);
    wm.add_raw_func(RT_STR_CMP);
    wm.add_raw_func(RT_TRIM_MATCHES);
    wm.add_raw_func(RT_STR_TRIM);
    wm.add_raw_func(RT_STR_SUBSTR);
    wm.add_raw_func(RT_STR_REPEAT);
    wm.add_raw_func(RT_STR_FIND);
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

/// `__rt_hex_digit_value`: the value of one ASCII hex digit, or -1 for any other byte.
///
/// Both cases are accepted, which is what makes `urldecode("%aB")` and `urldecode("%Ab")` agree
/// with php-src. The -1 sentinel is what lets a caller distinguish "not hex" from the digit zero.
const RT_HEX_DIGIT_VALUE: &str = r#"(func $__rt_hex_digit_value (param $c i32) (result i32)
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 48))
               (i32.le_u (local.get $c) (i32.const 57)))
    (then (return (i32.sub (local.get $c) (i32.const 48)))))     ;; 0-9
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 65))
               (i32.le_u (local.get $c) (i32.const 70)))
    (then (return (i32.sub (local.get $c) (i32.const 55)))))     ;; A-F
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 97))
               (i32.le_u (local.get $c) (i32.const 102)))
    (then (return (i32.sub (local.get $c) (i32.const 87)))))     ;; a-f
  (i32.const -1))                                                ;; not a hex digit
"#;

/// `__rt_str_url_encode`: owns the percent-encoded form of a string.
///
/// `$raw` selects `rawurlencode` over `urlencode`. Measured over all 256 bytes against php-src:
/// both leave `A-Z a-z 0-9 - . _` alone, `rawurlencode` additionally leaves `~`, and `urlencode`
/// alone maps a space to `+`. Everything else becomes `%` and two UPPERCASE hex digits. Worst
/// case is three output bytes per input byte.
const RT_STR_URL_ENCODE: &str = r#"(func $__rt_str_url_encode (param $ptr i32) (param $len i64) (param $raw i32) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $nib i32)                                                ;; nibble being written
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (i64.mul (local.get $len) (i64.const 3))))
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $enc
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (if (i32.or
          (i32.or
            (i32.and (i32.ge_u (local.get $byte) (i32.const 48))
                     (i32.le_u (local.get $byte) (i32.const 57)))  ;; 0-9
            (i32.or
              (i32.and (i32.ge_u (local.get $byte) (i32.const 65))
                       (i32.le_u (local.get $byte) (i32.const 90)))  ;; A-Z
              (i32.and (i32.ge_u (local.get $byte) (i32.const 97))
                       (i32.le_u (local.get $byte) (i32.const 122))))) ;; a-z
          (i32.or
            (i32.or (i32.eq (local.get $byte) (i32.const 45))
                    (i32.eq (local.get $byte) (i32.const 46)))    ;; - and .
            (i32.or (i32.eq (local.get $byte) (i32.const 95))
                    (i32.and (local.get $raw)
                             (i32.eq (local.get $byte) (i32.const 126)))))) ;; _ and raw-only ~
      (then
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (local.get $byte))
        (local.set $w (i32.add (local.get $w) (i32.const 1))))    ;; unreserved byte passes through
      (else
        (if (i32.and (i32.eqz (local.get $raw))
                     (i32.eq (local.get $byte) (i32.const 32)))   ;; urlencode alone folds space
          (then
            (i32.store8 (i32.add (local.get $out) (local.get $w)) (i32.const 43))  ;; +
            (local.set $w (i32.add (local.get $w) (i32.const 1))))
          (else
            (i32.store8 (i32.add (local.get $out) (local.get $w)) (i32.const 37))  ;; %
            (local.set $w (i32.add (local.get $w) (i32.const 1)))
            (local.set $nib (i32.shr_u (local.get $byte) (i32.const 4)))
            (i32.store8 (i32.add (local.get $out) (local.get $w))
              (i32.add (local.get $nib)
                (select (i32.const 48) (i32.const 55)
                        (i32.lt_u (local.get $nib) (i32.const 10)))))  ;; 0-9 else A-F
            (local.set $w (i32.add (local.get $w) (i32.const 1)))
            (local.set $nib (i32.and (local.get $byte) (i32.const 15)))
            (i32.store8 (i32.add (local.get $out) (local.get $w))
              (i32.add (local.get $nib)
                (select (i32.const 48) (i32.const 55)
                        (i32.lt_u (local.get $nib) (i32.const 10)))))  ;; 0-9 else A-F
            (local.set $w (i32.add (local.get $w) (i32.const 1)))))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $enc)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_url_decode`: owns the percent-decoded form of a string.
///
/// `$plus` selects `urldecode` over `rawurldecode`, which differ only in whether `+` becomes a
/// space. Decoding is TOLERANT and never fails: a `%` without two hex digits after it stays a
/// literal `%`, which is what php-src does for `"a%2"` and `"a%zz"`. Never grows.
const RT_STR_URL_DECODE: &str = r#"(func $__rt_str_url_decode (param $ptr i32) (param $len i64) (param $plus i32) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $byte i32)                                               ;; current source byte
  (local $hi i32)                                                 ;; high hex digit value
  (local $lo i32)                                                 ;; low hex digit value
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (local.get $len)))        ;; never grows
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $dec
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (local.set $hi (i32.const -1))                                ;; assume no escape here
    (if (i32.and (i32.eq (local.get $byte) (i32.const 37))        ;; a percent
                 (i64.le_s (i64.add (local.get $i) (i64.const 3)) (local.get $len)))
      (then
        (local.set $hi (call $__rt_hex_digit_value
          (i32.load8_u (i32.add (local.get $ptr)
                                (i32.wrap_i64 (i64.add (local.get $i) (i64.const 1)))))))
        (local.set $lo (call $__rt_hex_digit_value
          (i32.load8_u (i32.add (local.get $ptr)
                                (i32.wrap_i64 (i64.add (local.get $i) (i64.const 2)))))))
        (if (i32.or (i32.lt_s (local.get $hi) (i32.const 0))
                    (i32.lt_s (local.get $lo) (i32.const 0)))
          (then (local.set $hi (i32.const -1))))))                ;; not both hex: stay literal
    (if (i32.ge_s (local.get $hi) (i32.const 0))
      (then
        (i32.store8 (i32.add (local.get $out) (local.get $w))
          (i32.or (i32.shl (local.get $hi) (i32.const 4)) (local.get $lo)))
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (local.set $i (i64.add (local.get $i) (i64.const 3))))    ;; consume %HH
      (else
        (i32.store8 (i32.add (local.get $out) (local.get $w))
          (select (i32.const 32) (local.get $byte)
                  (i32.and (local.get $plus)
                           (i32.eq (local.get $byte) (i32.const 43))))) ;; urldecode folds + to space
        (local.set $w (i32.add (local.get $w) (i32.const 1)))     ;; w++
        (local.set $i (i64.add (local.get $i) (i64.const 1)))))   ;; i++
    (br $dec)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_b64_char`: the base64 alphabet character for a 6-bit value.
///
/// Arithmetic rather than a data segment: the four runs are contiguous, so each is one offset.
const RT_B64_CHAR: &str = r#"(func $__rt_b64_char (param $v i32) (result i32)
  (if (i32.lt_u (local.get $v) (i32.const 26))
    (then (return (i32.add (local.get $v) (i32.const 65)))))      ;; 0-25  -> A-Z
  (if (i32.lt_u (local.get $v) (i32.const 52))
    (then (return (i32.add (local.get $v) (i32.const 71)))))      ;; 26-51 -> a-z
  (if (i32.lt_u (local.get $v) (i32.const 62))
    (then (return (i32.sub (local.get $v) (i32.const 4)))))       ;; 52-61 -> 0-9
  (select (i32.const 47) (i32.const 43)
          (i32.eq (local.get $v) (i32.const 63))))                ;; 62 -> + , 63 -> /
"#;

/// `__rt_b64_value`: the 6-bit value of one base64 character, or -1 for any other byte.
///
/// The -1 sentinel is what makes non-strict decoding possible: php-src's one-argument
/// `base64_decode` SKIPS every byte outside the alphabet, padding and whitespace included.
const RT_B64_VALUE: &str = r#"(func $__rt_b64_value (param $c i32) (result i32)
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 65))
               (i32.le_u (local.get $c) (i32.const 90)))
    (then (return (i32.sub (local.get $c) (i32.const 65)))))      ;; A-Z
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 97))
               (i32.le_u (local.get $c) (i32.const 122)))
    (then (return (i32.sub (local.get $c) (i32.const 71)))))      ;; a-z
  (if (i32.and (i32.ge_u (local.get $c) (i32.const 48))
               (i32.le_u (local.get $c) (i32.const 57)))
    (then (return (i32.add (local.get $c) (i32.const 4)))))       ;; 0-9
  (if (i32.eq (local.get $c) (i32.const 43)) (then (return (i32.const 62))))  ;; +
  (if (i32.eq (local.get $c) (i32.const 47)) (then (return (i32.const 63))))  ;; /
  (i32.const -1))                                                 ;; outside the alphabet
"#;

/// `__rt_str_base64_encode`: owns the base64 form of a string, padded to a multiple of four.
///
/// The final group is padded with `=` to a full quartet, which is what php-src emits. Output is
/// exactly four characters per three input bytes, rounded up.
const RT_STR_BASE64_ENCODE: &str = r#"(func $__rt_str_base64_encode (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $left i64)                                               ;; bytes left in this group
  (local $acc i32)                                                ;; the 24-bit group
  (local $w i32)                                                  ;; destination cursor
  (local.set $out
    (call $__rt_str_alloc
      (i64.mul (i64.div_s (i64.add (local.get $len) (i64.const 2)) (i64.const 3))
               (i64.const 4))))                                   ;; four chars per three bytes
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $enc
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every group emitted
    (local.set $left (i64.sub (local.get $len) (local.get $i)))   ;; bytes remaining
    (local.set $acc
      (i32.shl (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i))))
               (i32.const 16)))                                   ;; first byte of the group
    (if (i64.ge_s (local.get $left) (i64.const 2))
      (then (local.set $acc (i32.or (local.get $acc)
        (i32.shl (i32.load8_u (i32.add (local.get $ptr)
                                       (i32.wrap_i64 (i64.add (local.get $i) (i64.const 1)))))
                 (i32.const 8))))))                               ;; second byte when present
    (if (i64.ge_s (local.get $left) (i64.const 3))
      (then (local.set $acc (i32.or (local.get $acc)
        (i32.load8_u (i32.add (local.get $ptr)
                              (i32.wrap_i64 (i64.add (local.get $i) (i64.const 2)))))))))
    (i32.store8 (i32.add (local.get $out) (local.get $w))
      (call $__rt_b64_char (i32.and (i32.shr_u (local.get $acc) (i32.const 18)) (i32.const 63))))
    (i32.store8 offset=1 (i32.add (local.get $out) (local.get $w))
      (call $__rt_b64_char (i32.and (i32.shr_u (local.get $acc) (i32.const 12)) (i32.const 63))))
    (i32.store8 offset=2 (i32.add (local.get $out) (local.get $w))
      (select
        (call $__rt_b64_char (i32.and (i32.shr_u (local.get $acc) (i32.const 6)) (i32.const 63)))
        (i32.const 61)
        (i64.ge_s (local.get $left) (i64.const 2))))              ;; = pads a one-byte tail
    (i32.store8 offset=3 (i32.add (local.get $out) (local.get $w))
      (select
        (call $__rt_b64_char (i32.and (local.get $acc) (i32.const 63)))
        (i32.const 61)
        (i64.ge_s (local.get $left) (i64.const 3))))              ;; = pads a two-byte tail
    (local.set $w (i32.add (local.get $w) (i32.const 4)))         ;; one quartet written
    (local.set $i (i64.add (local.get $i) (i64.const 3)))         ;; next group
    (br $enc)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_base64_decode`: owns the base64-decoded bytes of a string, php-src's tolerant way.
///
/// One-argument `base64_decode` is non-strict: every byte outside the alphabet is SKIPPED rather
/// than rejected, padding, whitespace and punctuation alike, and a trailing group of fewer than
/// eight accumulated bits is discarded. So `"YWJj="`, `"YW Jj"` and `"YWJj\n"` all decode to
/// `abc`, `"YWJ"` decodes to `ab`, and `"!!!!"` and `"a"` both decode to the empty string —
/// measured against php-src. Six bits per input byte means the source length is a safe bound.
const RT_STR_BASE64_DECODE: &str = r#"(func $__rt_str_base64_decode (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor
  (local $v i32)                                                  ;; value of the current char
  (local $acc i32)                                                ;; accumulated bits
  (local $bits i32)                                               ;; how many bits are accumulated
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (local.get $len)))        ;; 6 bits in, 8 bits out: never grows
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $w (i32.const 0))                                    ;; w = 0
  (local.set $acc (i32.const 0))                                  ;; empty bit accumulator
  (local.set $bits (i32.const 0))                                 ;; no bits yet
  (block $end (loop $dec
    (br_if $end (i64.ge_s (local.get $i) (local.get $len)))       ;; every byte examined
    (local.set $v (call $__rt_b64_value
      (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i))))))
    (if (i32.ge_s (local.get $v) (i32.const 0))                   ;; anything else is skipped
      (then
        (local.set $acc (i32.or (i32.shl (local.get $acc) (i32.const 6)) (local.get $v)))
        (local.set $bits (i32.add (local.get $bits) (i32.const 6)))
        (if (i32.ge_u (local.get $bits) (i32.const 8))
          (then
            (local.set $bits (i32.sub (local.get $bits) (i32.const 8)))
            (i32.store8 (i32.add (local.get $out) (local.get $w))
              (i32.and (i32.shr_u (local.get $acc) (local.get $bits)) (i32.const 255)))
            (local.set $w (i32.add (local.get $w) (i32.const 1)))))))  ;; a whole byte is ready
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $dec)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; leftover bits are discarded
"#;

/// `__rt_str_chr`: owns the one-byte string PHP's `chr` returns for any integer.
///
/// PHP does not reject an out-of-range codepoint: it constrains it with `% 256`, and a NEGATIVE
/// remainder is brought back up by adding 256, so `chr(-1)` is `\xff` and `chr(1000000)` is
/// `\x40` — measured. `$deprecate` carries whether this profile diagnoses the out-of-range
/// argument; the RESULT is the same either way, which is why the flag only gates the message.
const RT_STR_CHR_TEMPLATE: &str = r#"(func $__rt_str_chr (param $n i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $byte i64)                                               ;; the constrained byte value
{deprecation}  (local.set $byte (i64.rem_s (local.get $n) (i64.const 256)))    ;; % 256 keeps the sign in C
  (if (i64.lt_s (local.get $byte) (i64.const 0))
    (then (local.set $byte (i64.add (local.get $byte) (i64.const 256))))) ;; back into [0, 255]
  (local.set $out (call $__rt_str_alloc (i64.const 1)))           ;; exactly one byte
  (i32.store8 (local.get $out) (i32.wrap_i64 (local.get $byte)))  ;; the byte itself
  (local.get $out) (i64.const 1))                                 ;; owned one-byte string
"#;

/// `__rt_str_ord`: the first byte of a string as PHP's `ord` reports it.
///
/// An EMPTY string answers 0 rather than failing, and a longer string answers its FIRST byte —
/// both measured. `$deprecate` carries whether this profile diagnoses a length other than one;
/// as with `chr`, the answer does not depend on it.
const RT_STR_ORD_TEMPLATE: &str = r#"(func $__rt_str_ord (param $ptr i32) (param $len i64) (result i64)
{deprecation}  (if (i64.le_s (local.get $len) (i64.const 0))
    (then (return (i64.const 0))))                                ;; the empty string is zero
  (i64.extend_i32_u (i32.load8_u (local.get $ptr))))              ;; the first byte, unsigned
"#;

/// Renders `__rt_str_chr` and `__rt_str_ord` for this module's diagnostic capability.
///
/// The deprecation helpers they would call are command-only (they write to stderr through WASI)
/// and PHP 8.5-only, so a reactor or an earlier profile gets the same helper WITHOUT the call
/// rather than a dangling reference. The answer is identical either way — only the message is
/// conditional — which is exactly why eliding it is sound rather than a silent divergence.
fn str_chr_ord(diagnoses: bool) -> (String, String) {
    let (chr_deprecation, ord_deprecation) = if diagnoses {
        (
            concat!(
                "  (if (i32.or (i64.lt_s (local.get $n) (i64.const 0))\n",
                "              (i64.gt_s (local.get $n) (i64.const 255)))\n",
                "    (then (call $__rt_deprecated_chr_range)))                    ;; 8.5 diagnoses, then answers\n",
            ),
            concat!(
                "  (if (i64.ne (local.get $len) (i64.const 1))\n",
                "    (then (call $__rt_deprecated_ord_length)))                   ;; 8.5 diagnoses, then answers\n",
            ),
        )
    } else {
        ("", "")
    };
    (
        RT_STR_CHR_TEMPLATE.replace("{deprecation}", chr_deprecation),
        RT_STR_ORD_TEMPLATE.replace("{deprecation}", ord_deprecation),
    )
}

/// `__rt_str_case_edge`: owns a copy with only the FIRST byte case-mapped.
///
/// `$upper` selects `ucfirst` over `lcfirst`. Only an ASCII letter moves — `héllo` keeps its
/// `0xc3 0xa9` — and the empty string comes back empty rather than failing.
const RT_STR_CASE_EDGE: &str = r#"(func $__rt_str_case_edge (param $ptr i32) (param $len i64) (param $upper i32) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $olen i64)                                               ;; persisted length
  (local $byte i32)                                               ;; the first byte
  (call $__rt_str_persist (local.get $ptr) (local.get $len))      ;; own a copy to edit in place
  (local.set $olen)                                               ;; persisted length
  (local.set $out)                                                ;; persisted pointer
  (if (i64.gt_s (local.get $olen) (i64.const 0))                  ;; the empty string is unchanged
    (then
      (local.set $byte (i32.load8_u (local.get $out)))
      (if (local.get $upper)
        (then
          (if (i32.and (i32.ge_u (local.get $byte) (i32.const 97))
                       (i32.le_u (local.get $byte) (i32.const 122)))
            (then (i32.store8 (local.get $out) (i32.sub (local.get $byte) (i32.const 32))))))
        (else
          (if (i32.and (i32.ge_u (local.get $byte) (i32.const 65))
                       (i32.le_u (local.get $byte) (i32.const 90)))
            (then (i32.store8 (local.get $out) (i32.add (local.get $byte) (i32.const 32)))))))))
  (local.get $out) (local.get $olen))                             ;; owned result
"#;

/// `__rt_str_ucwords`: owns a copy with the first ASCII letter after each delimiter upper-cased.
///
/// PHP's default delimiter set is exactly space, tab, newline, carriage return, form feed and
/// VERTICAL TAB — measured, which is why `\x0b` starts a word here but `-`, `_` and `.` do not.
/// Two delimiters in a row do not skip a word: the byte after each one is a word start.
const RT_STR_UCWORDS: &str = r#"(func $__rt_str_ucwords (param $ptr i32) (param $len i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $olen i64)                                               ;; persisted length
  (local $i i64)                                                  ;; cursor
  (local $byte i32)                                               ;; current byte
  (local $start i32)                                              ;; is this byte a word start?
  (call $__rt_str_persist (local.get $ptr) (local.get $len))      ;; own a copy to edit in place
  (local.set $olen)                                               ;; persisted length
  (local.set $out)                                                ;; persisted pointer
  (local.set $i (i64.const 0))                                    ;; i = 0
  (local.set $start (i32.const 1))                                ;; the first byte starts a word
  (block $end (loop $scan
    (br_if $end (i64.ge_s (local.get $i) (local.get $olen)))      ;; every byte visited
    (local.set $byte (i32.load8_u (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))))
    (if (local.get $start)
      (then
        (if (i32.and (i32.ge_u (local.get $byte) (i32.const 97))
                     (i32.le_u (local.get $byte) (i32.const 122)))
          (then (i32.store8 (i32.add (local.get $out) (i32.wrap_i64 (local.get $i)))
                            (i32.sub (local.get $byte) (i32.const 32)))))))  ;; a-z -> A-Z
    (local.set $start
      (i32.or
        (i32.or (i32.eq (local.get $byte) (i32.const 32))         ;; space
                (i32.eq (local.get $byte) (i32.const 9)))         ;; tab
        (i32.or
          (i32.or (i32.eq (local.get $byte) (i32.const 10))       ;; line feed
                  (i32.eq (local.get $byte) (i32.const 13)))      ;; carriage return
          (i32.or (i32.eq (local.get $byte) (i32.const 12))       ;; form feed
                  (i32.eq (local.get $byte) (i32.const 11))))))   ;; vertical tab
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $scan)))
  (local.get $out) (local.get $olen))                             ;; owned result
"#;

/// `__rt_str_cmp`: PHP's `strcmp` / `strcasecmp` result for two byte strings.
///
/// The two halves of the answer follow DIFFERENT rules, which is the whole subtlety here and was
/// measured rather than assumed: a byte mismatch yields the raw UNSIGNED difference
/// (`strcmp("ABC", "abc")` is -32 and `strcmp("\xff", "\x01")` is 254), while a pure length
/// difference is normalized to -1 or 1 (`strcmp("abcd", "a")` is 1, not 3). `$fold` lowercases
/// ASCII letters before comparing, so `strcasecmp("Z", "a")` is 25 — the distance between the
/// FOLDED bytes, not the original ones.
const RT_STR_CMP: &str = r#"(func $__rt_str_cmp (param $aptr i32) (param $alen i64) (param $bptr i32) (param $blen i64) (param $fold i32) (result i64)
  (local $i i64)                                                  ;; cursor
  (local $shortest i64)                                           ;; bytes both strings have
  (local $x i32)                                                  ;; byte from the left
  (local $y i32)                                                  ;; byte from the right
  (local.set $shortest
    (select (local.get $alen) (local.get $blen)
            (i64.lt_s (local.get $alen) (local.get $blen))))      ;; min(alen, blen)
  (local.set $i (i64.const 0))                                    ;; i = 0
  (block $end (loop $cmp
    (br_if $end (i64.ge_s (local.get $i) (local.get $shortest)))  ;; common prefix exhausted
    (local.set $x (i32.load8_u (i32.add (local.get $aptr) (i32.wrap_i64 (local.get $i)))))
    (local.set $y (i32.load8_u (i32.add (local.get $bptr) (i32.wrap_i64 (local.get $i)))))
    (if (local.get $fold)
      (then
        (if (i32.and (i32.ge_u (local.get $x) (i32.const 65))
                     (i32.le_u (local.get $x) (i32.const 90)))
          (then (local.set $x (i32.add (local.get $x) (i32.const 32)))))  ;; A-Z -> a-z
        (if (i32.and (i32.ge_u (local.get $y) (i32.const 65))
                     (i32.le_u (local.get $y) (i32.const 90)))
          (then (local.set $y (i32.add (local.get $y) (i32.const 32)))))))
    (if (i32.ne (local.get $x) (local.get $y))
      (then (return (i64.extend_i32_s (i32.sub (local.get $x) (local.get $y))))))  ;; raw byte distance
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $cmp)))
  (if (i64.lt_s (local.get $alen) (local.get $blen))
    (then (return (i64.const -1))))                               ;; a prefix sorts first
  (if (i64.gt_s (local.get $alen) (local.get $blen))
    (then (return (i64.const 1))))                                ;; ...and its extension last
  (i64.const 0))                                                  ;; identical
"#;

/// `__rt_str_trim`: owns a copy with bytes stripped from one or both ends.
///
/// `$mode` bit 0 strips the left end and bit 1 the right, so one helper covers `trim`, `ltrim`
/// and `rtrim`. A `$cl_len` of -1 selects PHP's DEFAULT set — space, tab, newline, carriage
/// return, NUL and vertical tab — which is passed as a sentinel rather than a data segment so a
/// module that never calls the one-argument form carries no extra bytes. An explicitly EMPTY
/// charlist strips nothing, which is what php-src does.
const RT_STR_TRIM: &str = r#"(func $__rt_str_trim (param $ptr i32) (param $len i64) (param $cl_ptr i32) (param $cl_len i64) (param $mode i32) (result i32) (result i64)
  (local $start i64)                                              ;; first kept byte
  (local $stop i64)                                               ;; one past the last kept byte
  (local $out i32)                                                ;; owned result block
  (local $w i32)                                                  ;; copy cursor
  (local.set $start (i64.const 0))                                ;; nothing stripped yet
  (local.set $stop (local.get $len))                              ;; ...from either end
  (if (i32.and (local.get $mode) (i32.const 1))                   ;; strip the left end
    (then
      (block $ldone (loop $lscan
        (br_if $ldone (i64.ge_s (local.get $start) (local.get $stop)))
        (br_if $ldone (i32.eqz (call $__rt_trim_matches
          (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $start))))
          (local.get $cl_ptr) (local.get $cl_len))))
        (local.set $start (i64.add (local.get $start) (i64.const 1)))
        (br $lscan)))))
  (if (i32.and (local.get $mode) (i32.const 2))                   ;; strip the right end
    (then
      (block $rdone (loop $rscan
        (br_if $rdone (i64.le_s (local.get $stop) (local.get $start)))
        (br_if $rdone (i32.eqz (call $__rt_trim_matches
          (i32.load8_u (i32.add (local.get $ptr)
                                (i32.wrap_i64 (i64.sub (local.get $stop) (i64.const 1)))))
          (local.get $cl_ptr) (local.get $cl_len))))
        (local.set $stop (i64.sub (local.get $stop) (i64.const 1)))
        (br $rscan)))))
  (local.set $out (call $__rt_str_alloc (i64.sub (local.get $stop) (local.get $start))))
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $copy
    (br_if $end (i64.ge_s (local.get $start) (local.get $stop)))  ;; every kept byte copied
    (i32.store8 (i32.add (local.get $out) (local.get $w))
      (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $start)))))
    (local.set $w (i32.add (local.get $w) (i32.const 1)))         ;; w++
    (local.set $start (i64.add (local.get $start) (i64.const 1))) ;; next kept byte
    (br $copy)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_trim_matches`: whether one byte belongs to a trim character set.
///
/// A `$cl_len` of -1 means PHP's default set rather than a caller-provided list, so the one- and
/// two-argument forms of `trim` share the same scan.
const RT_TRIM_MATCHES: &str = r#"(func $__rt_trim_matches (param $byte i32) (param $cl_ptr i32) (param $cl_len i64) (result i32)
  (local $i i64)                                                  ;; charlist cursor
  (if (i64.lt_s (local.get $cl_len) (i64.const 0))                ;; the default set
    (then (return (i32.or
      (i32.or (i32.eq (local.get $byte) (i32.const 32))           ;; space
              (i32.eq (local.get $byte) (i32.const 9)))           ;; tab
      (i32.or
        (i32.or (i32.eq (local.get $byte) (i32.const 10))         ;; line feed
                (i32.eq (local.get $byte) (i32.const 13)))        ;; carriage return
        (i32.or (i32.eqz (local.get $byte))                       ;; NUL
                (i32.eq (local.get $byte) (i32.const 11))))))))   ;; vertical tab
  (local.set $i (i64.const 0))                                    ;; i = 0
  (block $end (loop $scan
    (br_if $end (i64.ge_s (local.get $i) (local.get $cl_len)))    ;; charlist exhausted
    (if (i32.eq (local.get $byte)
                (i32.load8_u (i32.add (local.get $cl_ptr) (i32.wrap_i64 (local.get $i)))))
      (then (return (i32.const 1))))                              ;; listed
    (local.set $i (i64.add (local.get $i) (i64.const 1)))         ;; i++
    (br $scan)))
  (i32.const 0))                                                  ;; not listed
"#;

/// `__rt_str_substr`: owns PHP's `substr` slice of a string.
///
/// Every out-of-range case answers the EMPTY string rather than false, which is PHP 8's
/// behaviour. A negative `$offset` counts from the end and saturates at 0, so `substr("hello",
/// -9)` is the whole string. `$has_len` distinguishes the two-argument form from an explicit
/// length; a negative length names an end offset from the right, and an end at or before the
/// start yields the empty string. All measured against php-src.
const RT_STR_SUBSTR: &str = r#"(func $__rt_str_substr (param $ptr i32) (param $len i64) (param $offset i64) (param $count i64) (param $has_len i32) (result i32) (result i64)
  (local $start i64)                                              ;; first byte taken
  (local $stop i64)                                               ;; one past the last byte taken
  (local $out i32)                                                ;; owned result block
  (local $w i32)                                                  ;; copy cursor
  (local.set $start (local.get $offset))                          ;; assume a forward offset
  (if (i64.lt_s (local.get $start) (i64.const 0))
    (then
      (local.set $start (i64.add (local.get $len) (local.get $start)))  ;; count from the end
      (if (i64.lt_s (local.get $start) (i64.const 0))
        (then (local.set $start (i64.const 0))))))                ;; ...saturating at the start
  (if (i64.gt_s (local.get $start) (local.get $len))
    (then (local.set $start (local.get $len))))                   ;; past the end takes nothing
  (local.set $stop (local.get $len))                              ;; the two-argument form runs to the end
  (if (local.get $has_len)
    (then
      (if (i64.lt_s (local.get $count) (i64.const 0))
        (then (local.set $stop (i64.add (local.get $len) (local.get $count))))  ;; end from the right
        (else (local.set $stop (i64.add (local.get $start) (local.get $count)))))
      (if (i64.gt_s (local.get $stop) (local.get $len))
        (then (local.set $stop (local.get $len))))                ;; clamp to the end
      (if (i64.lt_s (local.get $stop) (local.get $start))
        (then (local.set $stop (local.get $start))))))            ;; an inverted range is empty
  (local.set $out (call $__rt_str_alloc (i64.sub (local.get $stop) (local.get $start))))
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $end (loop $copy
    (br_if $end (i64.ge_s (local.get $start) (local.get $stop)))  ;; every selected byte copied
    (i32.store8 (i32.add (local.get $out) (local.get $w))
      (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $start)))))
    (local.set $w (i32.add (local.get $w) (i32.const 1)))         ;; w++
    (local.set $start (i64.add (local.get $start) (i64.const 1))) ;; next byte
    (br $copy)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_repeat`: owns a string repeated a non-negative number of times.
///
/// The caller has already refused a negative count, so this only has to handle 0 — which yields
/// the empty string, not a failure — and the ordinary case. `__rt_checked_layout` inside
/// `__rt_str_alloc` is what rejects a product that would overflow wasm32 rather than wrapping.
const RT_STR_REPEAT: &str = r#"(func $__rt_str_repeat (param $ptr i32) (param $len i64) (param $times i64) (result i32) (result i64)
  (local $out i32)                                                ;; owned result block
  (local $i i64)                                                  ;; source cursor within one copy
  (local $left i64)                                               ;; copies still to write
  (local $w i32)                                                  ;; destination cursor
  (local.set $out (call $__rt_str_alloc (i64.mul (local.get $len) (local.get $times))))
  (local.set $left (local.get $times))                            ;; every copy still pending
  (local.set $w (i32.const 0))                                    ;; w = 0
  (block $done (loop $copies
    (br_if $done (i64.le_s (local.get $left) (i64.const 0)))      ;; zero copies is the empty string
    (local.set $i (i64.const 0))                                  ;; restart at the source
    (block $end (loop $bytes
      (br_if $end (i64.ge_s (local.get $i) (local.get $len)))     ;; one whole copy written
      (i32.store8 (i32.add (local.get $out) (local.get $w))
        (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
      (local.set $w (i32.add (local.get $w) (i32.const 1)))       ;; w++
      (local.set $i (i64.add (local.get $i) (i64.const 1)))       ;; i++
      (br $bytes)))
    (local.set $left (i64.sub (local.get $left) (i64.const 1)))   ;; one fewer copy to go
    (br $copies)))
  (local.get $out) (i64.extend_i32_u (local.get $w)))             ;; owned result
"#;

/// `__rt_str_find`: the index of a needle in a haystack, or -1 when it is absent.
///
/// `$fold` lowercases ASCII letters on both sides, which is the only difference between `strpos`
/// and `stripos`. An EMPTY needle matches at 0 — `strpos("abc", "")` is 0, not false — and a
/// needle longer than the haystack cannot match. The -1 sentinel is what the caller turns into
/// PHP's `false`, and it is unambiguous because a real index is never negative.
const RT_STR_FIND: &str = r#"(func $__rt_str_find (param $hptr i32) (param $hlen i64) (param $nptr i32) (param $nlen i64) (param $fold i32) (result i64)
  (local $at i64)                                                 ;; candidate start offset
  (local $i i64)                                                  ;; cursor within the needle
  (local $x i32)                                                  ;; haystack byte
  (local $y i32)                                                  ;; needle byte
  (local.set $at (i64.const 0))                                   ;; start at the beginning
  (block $none (loop $scan
    (br_if $none (i64.gt_s (i64.add (local.get $at) (local.get $nlen)) (local.get $hlen)))
    (local.set $i (i64.const 0))                                  ;; compare the needle here
    (block $mismatch
      (block $matched (loop $bytes
        (br_if $matched (i64.ge_s (local.get $i) (local.get $nlen)))  ;; an empty needle matches at once
        (local.set $x (i32.load8_u (i32.add (local.get $hptr)
          (i32.wrap_i64 (i64.add (local.get $at) (local.get $i))))))
        (local.set $y (i32.load8_u (i32.add (local.get $nptr) (i32.wrap_i64 (local.get $i)))))
        (if (local.get $fold)
          (then
            (if (i32.and (i32.ge_u (local.get $x) (i32.const 65))
                         (i32.le_u (local.get $x) (i32.const 90)))
              (then (local.set $x (i32.add (local.get $x) (i32.const 32)))))  ;; A-Z -> a-z
            (if (i32.and (i32.ge_u (local.get $y) (i32.const 65))
                         (i32.le_u (local.get $y) (i32.const 90)))
              (then (local.set $y (i32.add (local.get $y) (i32.const 32)))))))
        (br_if $mismatch (i32.ne (local.get $x) (local.get $y)))  ;; this offset is out
        (local.set $i (i64.add (local.get $i) (i64.const 1)))     ;; i++
        (br $bytes)))
      (return (local.get $at)))                                   ;; every needle byte matched
    (local.set $at (i64.add (local.get $at) (i64.const 1)))       ;; try the next offset
    (br $scan)))
  (i64.const -1))                                                 ;; absent
"#;

/// Lowers `strstr` in both its arities.
///
/// PHP's result is `string|false`, so the two outcomes are boxed under different Mixed tags the
/// way `strpos` boxes its own. The returned slice is a REGION of the haystack — from the match to
/// the end, or from the start up to the match when `$before_needle` is true — and boxing under
/// the string tag persists a copy, so pointing into the source is safe rather than aliasing it.
/// An empty needle matches at 0, which is why `strstr("abcdef", "")` is the whole string and its
/// `before` form is empty.
fn lower_strstr(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    let found = ctx.fb.local("__strstr_at", super::wat::ValType::I64);
    let hptr = ctx.fb.local("__strstr_hptr", super::wat::ValType::I32);
    let hlen = ctx.fb.local("__strstr_hlen", super::wat::ValType::I64);
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb.ins(&format!("local.set {hlen}"), "spill haystack length");
    ctx.fb.ins(&format!("local.set {hptr}"), "spill haystack pointer");
    ctx.fb.ins(&format!("local.get {hptr}"), "haystack pointer");
    ctx.fb.ins(&format!("local.get {hlen}"), "haystack length");
    ctx.emit_load_value(operand(inst, 1)?)?;
    ctx.fb.ins("i32.const 0", "strstr is case-sensitive");
    ctx.fb
        .ins("call $__rt_str_find", "first offset, or -1 when absent");
    ctx.fb
        .ins(&format!("local.set {found}"), "spill the scan result");

    ctx.fb.ins(&format!("local.get {found}"), "scan result");
    ctx.fb.ins("i64.const 0", "the absent sentinel is negative");
    ctx.fb.ins("i64.lt_s", "was the needle absent?");
    ctx.fb
        .ins("if (result i32)", "string|false travels as a Mixed cell");
    ctx.fb.ins("i64.const 3", "mixed tag (bool)");
    ctx.fb.ins("i64.const 0", "the value false");
    ctx.fb.ins("i64.const 0", "hi unused");
    ctx.fb.ins("call $__rt_mixed_from_value", "box PHP's false");
    ctx.fb.ins("else", "the needle was found");
    ctx.fb.ins("i64.const 1", "mixed tag (string)");
    if inst.operands.len() == 3 {
        // `$before_needle` selects which side of the match survives, and it is a runtime value
        // rather than a literal, so both regions are computed and one is selected.
        ctx.fb.ins(&format!("local.get {hptr}"), "the haystack start");
        ctx.fb.ins("i64.extend_i32_u", "start pointer -> lo");
        ctx.fb.ins(&format!("local.get {hptr}"), "the haystack start");
        ctx.fb.ins("i64.extend_i32_u", "widen before adding the offset");
        ctx.fb.ins(&format!("local.get {found}"), "the match offset");
        ctx.fb.ins("i64.add", "pointer to the match");
        ctx.emit_load_value(operand(inst, 2)?)?;
        ctx.fb.ins("i64.const 0", "compare the flag against false");
        ctx.fb
            .ins("i64.ne", "a truthy flag selects the leading region");
        ctx.fb.ins("select", "which region's pointer");
        ctx.fb.ins(&format!("local.get {found}"), "bytes before the match");
        ctx.fb.ins(&format!("local.get {hlen}"), "haystack length");
        ctx.fb.ins(&format!("local.get {found}"), "the match offset");
        ctx.fb.ins("i64.sub", "bytes from the match to the end");
        ctx.emit_load_value(operand(inst, 2)?)?;
        ctx.fb.ins("i64.const 0", "compare the flag against false");
        ctx.fb
            .ins("i64.ne", "a truthy flag selects the leading region");
        ctx.fb.ins("select", "which region's length");
    } else {
        ctx.fb.ins(&format!("local.get {hptr}"), "the haystack start");
        ctx.fb.ins("i64.extend_i32_u", "widen before adding the offset");
        ctx.fb.ins(&format!("local.get {found}"), "the match offset");
        ctx.fb.ins("i64.add", "lo: pointer to the match");
        ctx.fb.ins(&format!("local.get {hlen}"), "haystack length");
        ctx.fb.ins(&format!("local.get {found}"), "the match offset");
        ctx.fb.ins("i64.sub", "hi: bytes from the match to the end");
    }
    ctx.fb
        .ins("call $__rt_mixed_from_value", "box the region (persists a copy)");
    ctx.fb.ins("end", "end string|false selection");
    store_result(ctx, inst)
}

/// Validates `strstr`: two strings, an optional bool, and PHP's `string|false` Mixed out.
fn strstr_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    if !matches!(call.operands.len(), 2 | 3) {
        return Some(format!(
            "expected a haystack, a needle and an optional flag, got {} operands",
            call.operands.len()
        ));
    }
    for (index, operand) in call.operands.iter().enumerate() {
        let Some(value) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        let (want_ir, want_php) = if index < 2 {
            (IrType::Str, PhpType::Str)
        } else {
            (IrType::I64, PhpType::Bool)
        };
        if value.ir_type != want_ir || value.php_type.codegen_repr() != want_php {
            return Some(format!(
                "strstr operand {index} is {:?}/{:?}, expected {want_ir:?}/{want_php:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::Heap(IrHeapKind::Mixed)
        || call.result_php_type.codegen_repr() != PhpType::Mixed
    {
        return Some(format!(
            "strstr result {:?}/{:?} is not the Mixed cell PHP's string|false needs",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Returns whether a unary string transform is lowered by this module.
///
/// Admitted: the exact same-length BYTE transforms, the re-encoders whose rules are pure byte
/// arithmetic (hex expansion, backslash escaping, line-break tagging), and base64 and url coding,
/// whose alphabets are contiguous enough to compute rather than tabulate. Still out are the html
/// entity decoders, which need a real named-entity table, and `hex2bin`, whose PHP result is
/// `string|false` rather than the `string` this family's signature promises.
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
            | UnaryStringRuntime::UrlEncode
            | UnaryStringRuntime::RawUrlEncode
            | UnaryStringRuntime::UrlDecode
            | UnaryStringRuntime::RawUrlDecode
            | UnaryStringRuntime::Base64Encode
            | UnaryStringRuntime::Base64Decode
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
        UnaryStringRuntime::UrlEncode => {
            ctx.fb.ins("i32.const 0", "urlencode folds a space to a plus");
            ctx.fb
                .ins("call $__rt_str_url_encode", "percent-encode the reserved bytes");
        }
        UnaryStringRuntime::RawUrlEncode => {
            ctx.fb.ins("i32.const 1", "rawurlencode keeps a tilde and encodes a space");
            ctx.fb
                .ins("call $__rt_str_url_encode", "percent-encode the reserved bytes");
        }
        UnaryStringRuntime::UrlDecode => {
            ctx.fb.ins("i32.const 1", "urldecode reads a plus as a space");
            ctx.fb
                .ins("call $__rt_str_url_decode", "percent-decode tolerantly");
        }
        UnaryStringRuntime::RawUrlDecode => {
            ctx.fb.ins("i32.const 0", "rawurldecode keeps a plus literal");
            ctx.fb
                .ins("call $__rt_str_url_decode", "percent-decode tolerantly");
        }
        UnaryStringRuntime::Base64Encode => {
            ctx.fb
                .ins("call $__rt_str_base64_encode", "encode with padding to a quartet");
        }
        UnaryStringRuntime::Base64Decode => {
            ctx.fb
                .ins("call $__rt_str_base64_decode", "decode skipping non-alphabet bytes");
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
            | RuntimeFnId::Chr
            | RuntimeFnId::Ord
            | RuntimeFnId::Ucfirst
            | RuntimeFnId::Lcfirst
            | RuntimeFnId::Ucwords
            | RuntimeFnId::Strcmp
            | RuntimeFnId::Strcasecmp
            | RuntimeFnId::Trim
            | RuntimeFnId::Ltrim
            | RuntimeFnId::Rtrim
            | RuntimeFnId::Substr
            | RuntimeFnId::StrRepeat
            | RuntimeFnId::Strpos
            | RuntimeFnId::Strstr
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
    if matches!(target, RuntimeFnId::Chr | RuntimeFnId::Ord) {
        return byte_conversion_shape_issue(function, call, target);
    }
    if matches!(target, RuntimeFnId::Strcmp | RuntimeFnId::Strcasecmp) {
        return string_compare_shape_issue(function, call, target);
    }
    if matches!(
        target,
        RuntimeFnId::Trim | RuntimeFnId::Ltrim | RuntimeFnId::Rtrim
    ) {
        return trim_shape_issue(function, call, target);
    }
    if target == RuntimeFnId::Substr {
        return substr_shape_issue(function, call);
    }
    if target == RuntimeFnId::StrRepeat {
        return str_repeat_shape_issue(function, call);
    }
    if target == RuntimeFnId::Strpos {
        return string_search_shape_issue(function, call, target);
    }
    if target == RuntimeFnId::Strstr {
        return strstr_shape_issue(function, call);
    }
    if matches!(
        target,
        RuntimeFnId::Ucfirst | RuntimeFnId::Lcfirst | RuntimeFnId::Ucwords
    ) {
        return trim_shape_issue(function, call, target)
            .or_else(|| (call.operands.len() != 1).then(|| {
                format!("{target:?} takes exactly one string, got {}", call.operands.len())
            }));
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
    if target == RuntimeFnId::Chr {
        ctx.emit_load_value(operand(inst, 0)?)?;
        ctx.fb
            .ins("call $__rt_str_chr", "the byte PHP's chr returns for this integer");
        return store_result(ctx, inst);
    }
    if target == RuntimeFnId::Ord {
        ctx.emit_load_value(operand(inst, 0)?)?;
        ctx.fb
            .ins("call $__rt_str_ord", "the first byte, or zero for the empty string");
        return store_result(ctx, inst);
    }
    if matches!(target, RuntimeFnId::Ucfirst | RuntimeFnId::Lcfirst) {
        ctx.emit_load_value(operand(inst, 0)?)?;
        ctx.fb.ins(
            if target == RuntimeFnId::Ucfirst {
                "i32.const 1"
            } else {
                "i32.const 0"
            },
            "map the first byte towards upper or lower case",
        );
        ctx.fb
            .ins("call $__rt_str_case_edge", "case-map the first byte only");
        return store_result(ctx, inst);
    }
    if target == RuntimeFnId::Ucwords {
        ctx.emit_load_value(operand(inst, 0)?)?;
        ctx.fb
            .ins("call $__rt_str_ucwords", "upper-case each word's first letter");
        return store_result(ctx, inst);
    }
    if matches!(target, RuntimeFnId::Strcmp | RuntimeFnId::Strcasecmp) {
        ctx.emit_load_value(operand(inst, 0)?)?;
        ctx.emit_load_value(operand(inst, 1)?)?;
        ctx.fb.ins(
            if target == RuntimeFnId::Strcasecmp {
                "i32.const 1"
            } else {
                "i32.const 0"
            },
            "fold ASCII case before comparing",
        );
        ctx.fb
            .ins("call $__rt_str_cmp", "byte distance, or +/-1 on length alone");
        return store_result(ctx, inst);
    }
    if matches!(
        target,
        RuntimeFnId::Trim | RuntimeFnId::Ltrim | RuntimeFnId::Rtrim
    ) {
        return lower_trim(ctx, inst, target);
    }
    if target == RuntimeFnId::Substr {
        return lower_substr(ctx, inst);
    }
    if target == RuntimeFnId::StrRepeat {
        return lower_str_repeat(ctx, inst);
    }
    if target == RuntimeFnId::Strpos {
        return lower_string_search(ctx, inst);
    }
    if target == RuntimeFnId::Strstr {
        return lower_strstr(ctx, inst);
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
/// Lowers `trim`, `ltrim` and `rtrim` through the shared end-stripping helper.
///
/// The one-argument form passes a `-1` charlist length, which the helper reads as PHP's default
/// set. That keeps the default out of the data segments of a module that never asks for it, and
/// keeps an explicitly EMPTY charlist — which strips nothing — distinguishable from it.
fn lower_trim(ctx: &mut FnCtx, inst: &Instruction, target: RuntimeFnId) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    if inst.operands.len() == 2 {
        ctx.emit_load_value(operand(inst, 1)?)?;
    } else {
        ctx.fb.ins("i32.const 0", "no charlist pointer");
        ctx.fb
            .ins("i64.const -1", "sentinel: PHP's default character set");
    }
    let mode = match target {
        RuntimeFnId::Ltrim => 1,
        RuntimeFnId::Rtrim => 2,
        _ => 3,
    };
    ctx.fb
        .ins(&format!("i32.const {mode}"), "which ends to strip");
    ctx.fb.ins("call $__rt_str_trim", "strip the selected ends");
    store_result(ctx, inst)
}

/// Lowers `substr`, passing whether an explicit length was written.
///
/// The two- and three-argument forms differ in more than a default: without a length the slice
/// runs to the end, while a NEGATIVE length names an end offset from the right. A single flag
/// tells the helper which rule to apply rather than inventing a length that would have to encode
/// both.
fn lower_substr(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.emit_load_value(operand(inst, 1)?)?;
    if inst.operands.len() == 3 {
        ctx.emit_load_value(operand(inst, 2)?)?;
        ctx.fb.ins("i32.const 1", "an explicit length was written");
    } else {
        ctx.fb.ins("i64.const 0", "unused length");
        ctx.fb.ins("i32.const 0", "no length: run to the end");
    }
    ctx.fb.ins("call $__rt_str_substr", "own the selected bytes");
    store_result(ctx, inst)
}

/// Lowers `strpos` in its two-argument form.
///
/// The scan helper takes a case-folding flag because `stripos`, `strrpos` and `strstr` all want
/// the same search with one knob changed; only `strpos` is registered as a distinct runtime
/// identity today, so this passes the case-sensitive setting.
///
/// PHP's result is `int|false`, which EIR carries as a `Mixed` cell, so the two outcomes are
/// boxed under different tags: an index under the int tag and a miss under the BOOL tag holding
/// zero, which is exactly `false`. Returning 0 under the int tag instead would make
/// `strpos($h, $n) === false` answer wrong for a match at the start — the classic PHP trap this
/// distinction exists to serve.
fn lower_string_search(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    let found = ctx.fb.local("__find_at", super::wat::ValType::I64);
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.emit_load_value(operand(inst, 1)?)?;
    ctx.fb.ins(
        "i32.const 0",
        "strpos is case-sensitive",
    );
    ctx.fb
        .ins("call $__rt_str_find", "first offset, or -1 when absent");
    ctx.fb
        .ins(&format!("local.set {found}"), "spill the scan result");
    ctx.fb.ins(&format!("local.get {found}"), "scan result");
    ctx.fb.ins("i64.const 0", "the absent sentinel is negative");
    ctx.fb.ins("i64.lt_s", "was the needle absent?");
    ctx.fb.ins("if (result i32)", "int|false travels as a Mixed cell");
    ctx.fb.ins("i64.const 3", "mixed tag (bool)");
    ctx.fb.ins("i64.const 0", "the value false");
    ctx.fb.ins("i64.const 0", "hi unused");
    ctx.fb
        .ins("call $__rt_mixed_from_value", "box PHP's false");
    ctx.fb.ins("else", "the needle was found");
    ctx.fb.ins("i64.const 0", "mixed tag (int)");
    ctx.fb.ins(&format!("local.get {found}"), "the byte offset");
    ctx.fb.ins("i64.const 0", "hi unused");
    ctx.fb.ins("call $__rt_mixed_from_value", "box the offset");
    ctx.fb.ins("end", "end int|false selection");
    store_result(ctx, inst)
}

/// Validates `strpos` and `stripos`: two strings in, PHP's `int|false` Mixed out.
///
/// Only the two-argument form is admitted. The three-argument form has to validate `$offset`
/// against the haystack and raise a `ValueError` naming the called function when it does not fit,
/// which is a different contract rather than a default, so it is refused rather than guessed.
fn string_search_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [haystack, needle] = call.operands.as_slice() else {
        return Some(format!(
            "expected a haystack and a needle with no offset, got {} operands",
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
        || call.result_type != IrType::Heap(IrHeapKind::Mixed)
        || call.result_php_type.codegen_repr() != PhpType::Mixed
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the Mixed cell PHP's int|false needs",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// The `__rt_fail` code PHP's negative-`str_repeat` `ValueError` reports under.
const STR_REPEAT_NEGATIVE_FAILURE_CODE: i32 = 11;

/// Lowers `str_repeat`, refusing a negative count the way PHP does.
///
/// PHP does not clamp a negative `$times` to zero: it raises a `ValueError`, which an ordinary
/// `catch` receives. The guard therefore goes through the shared runtime-failure path so it is
/// RAISED where the module can catch it and reported as a fatal where it cannot — the same
/// treatment division by zero gets. The count is spilled to a local because the guard reads it
/// before the helper consumes it.
fn lower_str_repeat(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    let times = ctx.fb.local("__repeat_times", super::wat::ValType::I64);
    ctx.emit_load_value(operand(inst, 1)?)?;
    ctx.fb
        .ins(&format!("local.set {times}"), "spill the repeat count");
    ctx.fb
        .ins(&format!("local.get {times}"), "repeat count");
    ctx.fb.ins("i64.const 0", "PHP's lower bound");
    ctx.fb.ins("i64.lt_s", "negative count?");
    ctx.fb.ins("if", "str_repeat() rejects a negative count");
    super::inst::emit_runtime_failure(
        ctx,
        STR_REPEAT_NEGATIVE_FAILURE_CODE,
        "str_repeat() negative count",
    );
    ctx.fb.ins("end", "end negative-count guard");
    ctx.emit_load_value(operand(inst, 0)?)?;
    ctx.fb
        .ins(&format!("local.get {times}"), "the validated repeat count");
    ctx.fb
        .ins("call $__rt_str_repeat", "own the repeated bytes");
    store_result(ctx, inst)
}

/// Validates `str_repeat`: a string subject and an integer count.
fn str_repeat_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    let [subject, times] = call.operands.as_slice() else {
        return Some(format!(
            "expected a subject and a count, got {} operands",
            call.operands.len()
        ));
    };
    for (operand, want_ir, want_php) in [
        (subject, IrType::Str, PhpType::Str),
        (times, IrType::I64, PhpType::Int),
    ] {
        let Some(value) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        if value.ir_type != want_ir || value.php_type.codegen_repr() != want_php {
            return Some(format!(
                "str_repeat operand is {:?}/{:?}, expected {want_ir:?}/{want_php:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::Str
        || call.result_php_type.codegen_repr() != PhpType::Str
    {
        return Some(format!(
            "str_repeat result {:?}/{:?} is not the expected Str/Str",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates `trim`, `ltrim` and `rtrim`: a string, and optionally a charlist string.
fn trim_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    if !matches!(call.operands.len(), 1 | 2) {
        return Some(format!(
            "expected a subject and an optional charlist, got {} operands",
            call.operands.len()
        ));
    }
    for operand in &call.operands {
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

/// Validates `substr`: a string, an integer offset, and optionally an integer length.
fn substr_shape_issue(function: &Function, call: &Instruction) -> Option<String> {
    if !matches!(call.operands.len(), 2 | 3) {
        return Some(format!(
            "expected a subject, an offset and an optional length, got {} operands",
            call.operands.len()
        ));
    }
    for (index, operand) in call.operands.iter().enumerate() {
        let Some(value) = function.value(*operand) else {
            return Some("operand is missing from the value table".to_string());
        };
        let (want_ir, want_php) = if index == 0 {
            (IrType::Str, PhpType::Str)
        } else {
            (IrType::I64, PhpType::Int)
        };
        if value.ir_type != want_ir || value.php_type.codegen_repr() != want_php {
            return Some(format!(
                "substr operand {index} is {:?}/{:?}, expected {want_ir:?}/{want_php:?}",
                value.ir_type,
                value.php_type.codegen_repr()
            ));
        }
    }
    if call.result.is_none()
        || call.result_type != IrType::Str
        || call.result_php_type.codegen_repr() != PhpType::Str
    {
        return Some(format!(
            "substr result {:?}/{:?} is not the expected Str/Str",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

/// Validates `strcmp` and `strcasecmp`: two strings in, an integer out.
fn string_compare_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let [left, right] = call.operands.as_slice() else {
        return Some(format!(
            "expected two string operands, got {}",
            call.operands.len()
        ));
    };
    for operand in [left, right] {
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

/// Validates `chr` and `ord`: one concrete scalar in, the opposite one out.
///
/// A `mixed` argument is refused rather than coerced. PHP would juggle it, and juggling carries
/// its own per-tag diagnostics that this backend does not reproduce yet, so admitting one here
/// would answer confidently where PHP would have complained first.
fn byte_conversion_shape_issue(
    function: &Function,
    call: &Instruction,
    target: RuntimeFnId,
) -> Option<String> {
    let (argument_ir, argument_php, result_ir, result_php) = if target == RuntimeFnId::Chr {
        (IrType::I64, PhpType::Int, IrType::Str, PhpType::Str)
    } else {
        (IrType::Str, PhpType::Str, IrType::I64, PhpType::Int)
    };
    let [operand] = call.operands.as_slice() else {
        return Some(format!(
            "expected one operand, got {}",
            call.operands.len()
        ));
    };
    let Some(value) = function.value(*operand) else {
        return Some("operand is missing from the value table".to_string());
    };
    if value.ir_type != argument_ir || value.php_type.codegen_repr() != argument_php {
        return Some(format!(
            "{target:?} operand {:?}/{:?} is not the expected {argument_ir:?}/{argument_php:?}",
            value.ir_type,
            value.php_type.codegen_repr()
        ));
    }
    if call.result.is_none()
        || call.result_type != result_ir
        || call.result_php_type.codegen_repr() != result_php
    {
        return Some(format!(
            "{target:?} result {:?}/{:?} is not the expected {result_ir:?}/{result_php:?}",
            call.result_type,
            call.result_php_type.codegen_repr()
        ));
    }
    None
}

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
            UnaryStringRuntime::UrlEncode,
            UnaryStringRuntime::RawUrlEncode,
            UnaryStringRuntime::UrlDecode,
            UnaryStringRuntime::RawUrlDecode,
            UnaryStringRuntime::Base64Encode,
            UnaryStringRuntime::Base64Decode,
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

        // The html decoders need a real named-entity table, and `hex2bin` returns `string|false`
        // in PHP rather than the `string` this family's signature promises.
        assert!(!unary_string_is_supported(
            UnaryStringRuntime::HtmlEntityDecode
        ));
        assert!(!unary_string_is_supported(UnaryStringRuntime::HexToBin));
    }

    /// Builds a one-operand `RuntimeCall` probe with a chosen operand and result typing.
    fn scalar_conversion_call(
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
                Ownership::MaybeOwned,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Verifies `chr` and `ord` take one concrete scalar and refuse anything juggled.
    ///
    /// `RuntimeFnId::Chr` maps an int to a one-byte string and `RuntimeFnId::Ord` maps a string
    /// back to an int, so each refuses the other's typing as well as a `mixed` operand. PHP would
    /// juggle a `mixed` with its own per-tag diagnostics, which this backend does not reproduce,
    /// so admitting one would answer confidently where PHP would have complained first.
    #[test]
    fn chr_and_ord_admit_only_their_concrete_scalar() {
        let chr = scalar_conversion_call(
            RuntimeFnId::Chr,
            IrType::I64,
            PhpType::Int,
            IrType::Str,
            PhpType::Str,
        );
        let call = chr.instructions.last().expect("the probe emitted a call");
        assert_eq!(direct_builtin_shape_issue(&chr, call, RuntimeFnId::Chr), None);

        let ord = scalar_conversion_call(
            RuntimeFnId::Ord,
            IrType::Str,
            PhpType::Str,
            IrType::I64,
            PhpType::Int,
        );
        let call = ord.instructions.last().expect("the probe emitted a call");
        assert_eq!(direct_builtin_shape_issue(&ord, call, RuntimeFnId::Ord), None);

        // Each refuses the other's operand typing, and both refuse a juggled one.
        let swapped = scalar_conversion_call(
            RuntimeFnId::Chr,
            IrType::Str,
            PhpType::Str,
            IrType::Str,
            PhpType::Str,
        );
        let call = swapped.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&swapped, call, RuntimeFnId::Chr).is_some());

        let juggled = scalar_conversion_call(
            RuntimeFnId::Ord,
            IrType::Heap(IrHeapKind::Mixed),
            PhpType::Mixed,
            IrType::I64,
            PhpType::Int,
        );
        let call = juggled.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&juggled, call, RuntimeFnId::Ord).is_some());

        // A wrong RESULT typing is refused too: `chr` cannot answer an int.
        let bad_result = scalar_conversion_call(
            RuntimeFnId::Chr,
            IrType::I64,
            PhpType::Int,
            IrType::I64,
            PhpType::Int,
        );
        let call = bad_result.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&bad_result, call, RuntimeFnId::Chr).is_some());
    }

    /// Verifies the PHP 8.5 deprecations are the ONLY thing the profile changes about chr/ord.
    ///
    /// PHP 8.5 diagnoses an out-of-range `chr` argument and an `ord` argument that is not exactly
    /// one byte, but both still ANSWER, and with the same value an earlier profile gives. So the
    /// two renderings must differ only by the diagnostic call: if the arithmetic differed, an
    /// earlier profile would silently compute something else.
    #[test]
    fn chr_and_ord_differ_between_profiles_only_by_the_deprecation() {
        let (diagnosing_chr, diagnosing_ord) = str_chr_ord(true);
        let (silent_chr, silent_ord) = str_chr_ord(false);

        assert!(diagnosing_chr.contains("call $__rt_deprecated_chr_range"));
        assert!(!silent_chr.contains("__rt_deprecated"));
        assert!(diagnosing_ord.contains("call $__rt_deprecated_ord_length"));
        assert!(!silent_ord.contains("__rt_deprecated"));

        // Strip the guard the diagnosing form adds; what remains must be identical.
        for (diagnosing, silent, marker) in [
            (&diagnosing_chr, &silent_chr, "__rt_deprecated_chr_range"),
            (&diagnosing_ord, &silent_ord, "__rt_deprecated_ord_length"),
        ] {
            let stripped: String = diagnosing
                .lines()
                .skip_while(|line| !line.contains("(if ("))
                .skip_while(|line| !line.contains(marker))
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");
            let expected: String = silent
                .lines()
                .skip_while(|line| !line.contains("(local.set $byte") && !line.contains("(if (i64.le_s"))
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(
                stripped, expected,
                "the profiles must agree on the ANSWER, not just on the diagnostic"
            );
        }
    }

    /// Builds a `RuntimeCall` probe with an arbitrary operand list.
    fn shaped_call(
        target: RuntimeFnId,
        operands: &[(IrType, PhpType)],
        result_ir: IrType,
        result_php: PhpType,
    ) -> Function {
        let mut function = Function::new("probe".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let arguments: Vec<_> = operands
                .iter()
                .enumerate()
                .map(|(index, (ir, php))| {
                    let slot = builder.add_local(
                        Some(format!("a{index}")),
                        *ir,
                        php.clone(),
                        crate::ir::LocalKind::PhpLocal,
                    );
                    builder.emit_load_local(slot, *ir, php.clone())
                })
                .collect();
            builder.emit(
                Op::RuntimeCall,
                arguments,
                Some(Immediate::RuntimeCall(RuntimeCallTarget::Function(target))),
                result_ir,
                result_php,
                Ownership::MaybeOwned,
            );
            builder.terminate(crate::ir::Terminator::Return { value: None });
        }
        function
    }

    /// Verifies the string-shaping builtins accept exactly the arities they lower.
    ///
    /// `trim` and `substr` each have an optional trailing argument that changes the RULE, not
    /// just a default — an absent charlist means PHP's built-in set rather than the empty one,
    /// and an absent length runs to the end rather than meaning zero — so both arities have to be
    /// admitted and a third refused. The rest take exactly what they take.
    #[test]
    fn string_shaping_builtins_admit_only_their_arities() {
        let str_arg = (IrType::Str, PhpType::Str);
        let int_arg = (IrType::I64, PhpType::Int);

        for target in [
            RuntimeFnId::Ucfirst,
            RuntimeFnId::Lcfirst,
            RuntimeFnId::Ucwords,
        ] {
            let ok = shaped_call(target, &[str_arg.clone()], IrType::Str, PhpType::Str);
            let call = ok.instructions.last().expect("the probe emitted a call");
            assert_eq!(direct_builtin_shape_issue(&ok, call, target), None);

            let two = shaped_call(
                target,
                &[str_arg.clone(), str_arg.clone()],
                IrType::Str,
                PhpType::Str,
            );
            let call = two.instructions.last().expect("the probe emitted a call");
            assert!(
                direct_builtin_shape_issue(&two, call, target).is_some(),
                "{target:?} takes one string"
            );
        }

        for target in [RuntimeFnId::Trim, RuntimeFnId::Ltrim, RuntimeFnId::Rtrim] {
            for arity in 1..=2 {
                let operands = vec![str_arg.clone(); arity];
                let ok = shaped_call(target, &operands, IrType::Str, PhpType::Str);
                let call = ok.instructions.last().expect("the probe emitted a call");
                assert_eq!(
                    direct_builtin_shape_issue(&ok, call, target),
                    None,
                    "{target:?} accepts {arity} operand(s)"
                );
            }
            let three = shaped_call(target, &vec![str_arg.clone(); 3], IrType::Str, PhpType::Str);
            let call = three.instructions.last().expect("the probe emitted a call");
            assert!(direct_builtin_shape_issue(&three, call, target).is_some());
        }

        for arity in 2..=3 {
            let mut operands = vec![str_arg.clone()];
            operands.extend(vec![int_arg.clone(); arity - 1]);
            let ok = shaped_call(RuntimeFnId::Substr, &operands, IrType::Str, PhpType::Str);
            let call = ok.instructions.last().expect("the probe emitted a call");
            assert_eq!(
                direct_builtin_shape_issue(&ok, call, RuntimeFnId::Substr),
                None,
                "substr accepts {arity} operands"
            );
        }
        // A string where the offset belongs is refused rather than coerced.
        let bad_offset = shaped_call(
            RuntimeFnId::Substr,
            &[str_arg.clone(), str_arg.clone()],
            IrType::Str,
            PhpType::Str,
        );
        let call = bad_offset.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&bad_offset, call, RuntimeFnId::Substr).is_some());

        // `str_repeat` takes a subject and a count, and refuses a string where the count goes.
        let ok = shaped_call(
            RuntimeFnId::StrRepeat,
            &[str_arg.clone(), int_arg.clone()],
            IrType::Str,
            PhpType::Str,
        );
        let call = ok.instructions.last().expect("the probe emitted a call");
        assert_eq!(
            direct_builtin_shape_issue(&ok, call, RuntimeFnId::StrRepeat),
            None
        );
        let bad_count = shaped_call(
            RuntimeFnId::StrRepeat,
            &[str_arg.clone(), str_arg.clone()],
            IrType::Str,
            PhpType::Str,
        );
        let call = bad_count.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&bad_count, call, RuntimeFnId::StrRepeat).is_some());

        // `strpos` answers PHP's `int|false`, which only a runtime-tagged Mixed cell can carry.
        let searched = shaped_call(
            RuntimeFnId::Strpos,
            &[str_arg.clone(), str_arg.clone()],
            IrType::Heap(IrHeapKind::Mixed),
            PhpType::Mixed,
        );
        let call = searched.instructions.last().expect("the probe emitted a call");
        assert_eq!(
            direct_builtin_shape_issue(&searched, call, RuntimeFnId::Strpos),
            None
        );
        // An Int result would lose the difference between a match at offset 0 and a miss.
        let as_int = shaped_call(
            RuntimeFnId::Strpos,
            &[str_arg.clone(), str_arg.clone()],
            IrType::I64,
            PhpType::Int,
        );
        let call = as_int.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&as_int, call, RuntimeFnId::Strpos).is_some());
        // `strstr` answers `string|false` through the same tagged cell, in BOTH arities.
        for arity in 2..=3 {
            let mut operands = vec![str_arg.clone(), str_arg.clone()];
            if arity == 3 {
                operands.push((IrType::I64, PhpType::Bool));
            }
            let ok = shaped_call(
                RuntimeFnId::Strstr,
                &operands,
                IrType::Heap(IrHeapKind::Mixed),
                PhpType::Mixed,
            );
            let call = ok.instructions.last().expect("the probe emitted a call");
            assert_eq!(
                direct_builtin_shape_issue(&ok, call, RuntimeFnId::Strstr),
                None,
                "strstr accepts {arity} operands"
            );
        }
        // The flag is a bool, not an int offset, and the result is never a bare string.
        let bad_flag = shaped_call(
            RuntimeFnId::Strstr,
            &[str_arg.clone(), str_arg.clone(), int_arg.clone()],
            IrType::Heap(IrHeapKind::Mixed),
            PhpType::Mixed,
        );
        let call = bad_flag.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&bad_flag, call, RuntimeFnId::Strstr).is_some());

        // Only the two-argument form is lowered; the offset form has its own ValueError contract.
        let with_offset = shaped_call(
            RuntimeFnId::Strpos,
            &[str_arg.clone(), str_arg.clone(), int_arg.clone()],
            IrType::Heap(IrHeapKind::Mixed),
            PhpType::Mixed,
        );
        let call = with_offset.instructions.last().expect("the probe emitted a call");
        assert!(direct_builtin_shape_issue(&with_offset, call, RuntimeFnId::Strpos).is_some());

        for target in [RuntimeFnId::Strcmp, RuntimeFnId::Strcasecmp] {
            let ok = shaped_call(
                target,
                &[str_arg.clone(), str_arg.clone()],
                IrType::I64,
                PhpType::Int,
            );
            let call = ok.instructions.last().expect("the probe emitted a call");
            assert_eq!(direct_builtin_shape_issue(&ok, call, target), None);

            // A comparison answers an int, never a bool: PHP's result is a byte distance.
            let as_bool = shaped_call(
                target,
                &[str_arg.clone(), str_arg.clone()],
                IrType::I64,
                PhpType::Bool,
            );
            let call = as_bool.instructions.last().expect("the probe emitted a call");
            assert!(direct_builtin_shape_issue(&as_bool, call, target).is_some());
        }
    }

    /// Verifies `strcmp` reports a byte DISTANCE but normalizes a pure length difference.
    ///
    /// These are two different rules and php-src applies both: `strcmp("ABC", "abc")` is -32
    /// because `A` and `a` are 32 apart, while `strcmp("abcd", "a")` is 1 rather than 3 because
    /// nothing mismatched — only the lengths differ. A helper that returned the length delta, or
    /// that clamped the byte distance to a sign, would pass a naive test and fail php-src.
    #[test]
    fn strcmp_returns_a_byte_distance_but_a_normalized_length_difference() {
        assert!(
            RT_STR_CMP.contains("(return (i64.extend_i32_s (i32.sub (local.get $x) (local.get $y))))"),
            "a mismatched byte yields its raw distance, not a sign"
        );
        for normalized in ["(then (return (i64.const -1)))", "(then (return (i64.const 1)))"] {
            assert!(
                RT_STR_CMP.contains(normalized),
                "a pure length difference is normalized to +/-1"
            );
        }
        // The bytes are read UNSIGNED, which is what makes strcmp("\xff", "\x01") 254 and not -2.
        assert!(!RT_STR_CMP.contains("i32.load8_s"));
    }

    /// Verifies the two url codecs differ exactly where php-src says they do.
    ///
    /// `urlencode` and `rawurlencode` share one helper and are told apart by a flag, so the
    /// difference is one branch rather than two implementations. Those two branches are the whole
    /// contract between them — a space folding to `+`, and `~` counting as unreserved — and this
    /// reads both back out of the emitted helper rather than trusting the call sites.
    #[test]
    fn url_codecs_differ_only_in_space_and_tilde() {
        assert!(
            RT_STR_URL_ENCODE.contains("(i32.eq (local.get $byte) (i32.const 32))"),
            "urlencode folds a space to a plus"
        );
        assert!(
            RT_STR_URL_ENCODE.contains("(i32.eq (local.get $byte) (i32.const 126))"),
            "rawurlencode alone leaves a tilde unreserved"
        );
        assert!(
            RT_STR_URL_DECODE.contains("(i32.eq (local.get $byte) (i32.const 43))"),
            "urldecode alone reads a plus as a space"
        );
        // Percent-encoding is uppercase: 55 is 'A' - 10, where a lowercase table would use 87.
        assert!(RT_STR_URL_ENCODE.contains("(i32.const 55)"));
        assert!(!RT_STR_URL_ENCODE.contains("(i32.const 87)"));
        // ...and hex DECODING accepts both cases, so it needs the lowercase offset as well.
        assert!(RT_HEX_DIGIT_VALUE.contains("(i32.const 55)"));
        assert!(RT_HEX_DIGIT_VALUE.contains("(i32.const 87)"));
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

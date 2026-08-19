//! Purpose:
//! IPv4 address conversion and the scalar half of `var_dump` for the wasm32-wasi backend.
//!
//! Called from:
//! - `builtins::emit_builtin_runtime` — `emit_inet_runtime` adds the four conversions.
//! - `runtime::emit_failure_runtime` — `emit_var_dump_runtime`, which needs the fixed text
//!   fragments that region owns.
//! - `builtins::lower_direct_builtin` / `direct_builtin_shape_issue` — the call sites.
//!
//! Key details:
//! - All four conversions are pure computation over bytes already in linear memory: no WASI
//!   call, no allocation beyond the answer itself.
//! - php's IPv4 parser is STRICTER than a tolerant reader: `ip2long` and `inet_pton` accept
//!   exactly four decimal groups of 1..3 digits, each 0..255, with no leading `+`/`-`, no
//!   spaces and no trailing text. Anything else is `false` — measured against php-src 8.5.6 on
//!   `examples/ip-conversion`, where `"not an address"` must answer `bool(false)` rather than a
//!   partial parse.
//! - `ip2long` answers a value php prints as an UNSIGNED 32-bit number (`long2ip(4294967295)`
//!   round-trips), so the result is zero-extended rather than sign-extended.
//! - `var_dump` here is deliberately PARTIAL: int, float, bool, string and null render, and
//!   every other tag keeps refusing in the capability audit. An array or object printed by a
//!   half-implementation would be worse than a refusal, because the output would look right.

use super::wat::WatModule;

/// Adds the IPv4 conversions. They reference only `__rt_str_persist`, `__rt_heap_alloc`,
/// `__rt_heap_free`, `__rt_itoa` and `__rt_mixed_from_value`, all of which the builtin runtime
/// already emits alongside this.
pub(super) fn emit_inet_runtime(wm: &mut WatModule) {
    wm.add_raw_func(RT_IPV4_PARSE);
    wm.add_raw_func(RT_LONG2IP);
    wm.add_raw_func(RT_IP2LONG);
    wm.add_raw_func(RT_INET_NTOP);
    wm.add_raw_func(RT_INET_PTON);
}

/// `__rt_ipv4_parse`: php's dotted-quad reader, answering `(value, ok)`.
///
/// Shared by `ip2long` and `inet_pton` because php parses the same grammar for both. The
/// strictness is the point: four groups, each one to three digits with a value of at most 255,
/// separated by single dots, and nothing before or after. A group of four digits, an empty
/// group, a fifth group or any non-digit byte answers `ok = 0`.
const RT_IPV4_PARSE: &str = r#"(func $__rt_ipv4_parse (param $ptr i32) (param $len i64) (result i64 i32)
  (local $i i64) (local $group i32) (local $digits i32) (local $acc i32) (local $value i64) (local $byte i32)
  (if (i64.eqz (local.get $len))
    (then (return (i64.const 0) (i32.const 0))))
  (block $done (loop $scan
    (br_if $done (i64.ge_u (local.get $i) (local.get $len)))
    (local.set $byte (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (if (i32.eq (local.get $byte) (i32.const 46))                  ;; '.'
      (then
        (if (i32.eqz (local.get $digits))                          ;; ".." or a leading dot
          (then (return (i64.const 0) (i32.const 0))))
        (if (i32.ge_u (local.get $group) (i32.const 3))            ;; a fifth group
          (then (return (i64.const 0) (i32.const 0))))
        (local.set $value (i64.or (i64.shl (local.get $value) (i64.const 8))
                                  (i64.extend_i32_u (local.get $acc))))
        (local.set $group (i32.add (local.get $group) (i32.const 1)))
        (local.set $acc (i32.const 0))
        (local.set $digits (i32.const 0))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $scan)))
    (if (i32.or (i32.lt_u (local.get $byte) (i32.const 48))
                (i32.gt_u (local.get $byte) (i32.const 57)))       ;; not a digit
      (then (return (i64.const 0) (i32.const 0))))
    (if (i32.ge_u (local.get $digits) (i32.const 3))               ;; four digits in one group
      (then (return (i64.const 0) (i32.const 0))))
    (local.set $acc (i32.add (i32.mul (local.get $acc) (i32.const 10))
                             (i32.sub (local.get $byte) (i32.const 48))))
    (if (i32.gt_u (local.get $acc) (i32.const 255))
      (then (return (i64.const 0) (i32.const 0))))
    (local.set $digits (i32.add (local.get $digits) (i32.const 1)))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $scan)))
  (if (i32.or (i32.ne (local.get $group) (i32.const 3))            ;; too few groups
              (i32.eqz (local.get $digits)))                       ;; or a trailing dot
    (then (return (i64.const 0) (i32.const 0))))
  (local.set $value (i64.or (i64.shl (local.get $value) (i64.const 8))
                            (i64.extend_i32_u (local.get $acc))))
  (local.get $value)
  (i32.const 1))
"#;

/// `__rt_long2ip`: an unsigned 32-bit address as its dotted-quad text.
///
/// The value is masked to 32 bits first: php's `long2ip` takes the low word of whatever integer
/// arrives, so `long2ip(4294967295)` is `255.255.255.255` rather than an overflowed read.
const RT_LONG2IP: &str = r#"(func $__rt_long2ip (param $value i64) (result i32 i64)
  (local $v i64) (local $out i32) (local $w i32) (local $i i32) (local $part i64)
  (local $tptr i32) (local $tlen i32) (local $j i32)
  (local $pptr i32) (local $plen i64)
  (local.set $v (i64.and (local.get $value) (i64.const 4294967295)))
  (local.set $out (call $__rt_heap_alloc (i32.const 16)))          ;; "255.255.255.255" fits
  (block $built (loop $group
    (br_if $built (i32.ge_u (local.get $i) (i32.const 4)))
    (if (local.get $i)
      (then
        (i32.store8 (i32.add (local.get $out) (local.get $w)) (i32.const 46))  ;; '.'
        (local.set $w (i32.add (local.get $w) (i32.const 1)))))
    (local.set $part (i64.and
      (i64.shr_u (local.get $v) (i64.extend_i32_u (i32.mul (i32.sub (i32.const 3) (local.get $i)) (i32.const 8))))
      (i64.const 255)))
    (call $__rt_itoa (local.get $part) (global.get $__float_scratch))
    (local.set $tlen)
    (local.set $tptr)
    (local.set $j (i32.const 0))
    (block $copied (loop $byte
      (br_if $copied (i32.ge_u (local.get $j) (local.get $tlen)))
      (i32.store8 (i32.add (local.get $out) (local.get $w))
                  (i32.load8_u (i32.add (local.get $tptr) (local.get $j))))
      (local.set $w (i32.add (local.get $w) (i32.const 1)))
      (local.set $j (i32.add (local.get $j) (i32.const 1)))
      (br $byte)))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $group)))
  ;; `__rt_str_persist` answers (ptr i32, len i64) — a different width from `__rt_itoa`'s
  ;; (ptr i32, len i32), so the persisted pair needs its own locals rather than the digit ones.
  (call $__rt_str_persist (local.get $out) (i64.extend_i32_u (local.get $w)))
  (local.set $plen)
  (local.set $pptr)
  (call $__rt_heap_free (local.get $out))
  (local.get $pptr)
  (local.get $plen))
"#;

/// `__rt_ip2long`: dotted quad to its unsigned 32-bit value, boxed as php's `int|false`.
const RT_IP2LONG: &str = r#"(func $__rt_ip2long (param $ptr i32) (param $len i64) (result i32)
  (local $value i64) (local $ok i32)
  (call $__rt_ipv4_parse (local.get $ptr) (local.get $len))
  (local.set $ok)
  (local.set $value)
  (if (i32.eqz (local.get $ok))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (call $__rt_mixed_from_value (i64.const 0) (local.get $value) (i64.const 0)))
"#;

/// `__rt_inet_ntop`: four packed bytes to dotted-quad text, boxed as php's `string|false`.
///
/// php answers `false` for any length other than 4 or 16; only the IPv4 width is served here,
/// so a 16-byte IPv6 address answers `false` rather than a wrong string. That is a DIVERGENCE
/// from php for IPv6 input, and the capability audit cannot see the length, so it is the
/// runtime that answers.
const RT_INET_NTOP: &str = r#"(func $__rt_inet_ntop (param $ptr i32) (param $len i64) (result i32)
  (local $value i64) (local $sptr i32) (local $slen i64)
  (if (i64.ne (local.get $len) (i64.const 4))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $value (i64.or (i64.or (i64.or
    (i64.shl (i64.extend_i32_u (i32.load8_u offset=0 (local.get $ptr))) (i64.const 24))
    (i64.shl (i64.extend_i32_u (i32.load8_u offset=1 (local.get $ptr))) (i64.const 16)))
    (i64.shl (i64.extend_i32_u (i32.load8_u offset=2 (local.get $ptr))) (i64.const 8)))
    (i64.extend_i32_u (i32.load8_u offset=3 (local.get $ptr)))))
  (call $__rt_long2ip (local.get $value))
  (local.set $slen)
  (local.set $sptr)
  (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (local.get $sptr)) (local.get $slen)))
"#;

/// `__rt_inet_pton`: dotted-quad text to four packed bytes, boxed as php's `string|false`.
const RT_INET_PTON: &str = r#"(func $__rt_inet_pton (param $ptr i32) (param $len i64) (result i32)
  (local $value i64) (local $ok i32) (local $buf i32) (local $cell i32)
  (local $sptr i32) (local $slen i64)
  (call $__rt_ipv4_parse (local.get $ptr) (local.get $len))
  (local.set $ok)
  (local.set $value)
  (if (i32.eqz (local.get $ok))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $buf (call $__rt_heap_alloc (i32.const 4)))
  (i32.store8 offset=0 (local.get $buf)
    (i32.wrap_i64 (i64.and (i64.shr_u (local.get $value) (i64.const 24)) (i64.const 255))))
  (i32.store8 offset=1 (local.get $buf)
    (i32.wrap_i64 (i64.and (i64.shr_u (local.get $value) (i64.const 16)) (i64.const 255))))
  (i32.store8 offset=2 (local.get $buf)
    (i32.wrap_i64 (i64.and (i64.shr_u (local.get $value) (i64.const 8)) (i64.const 255))))
  (i32.store8 offset=3 (local.get $buf)
    (i32.wrap_i64 (i64.and (local.get $value) (i64.const 255))))
  (call $__rt_str_persist (local.get $buf) (i64.const 4))
  (local.set $slen)
  (local.set $sptr)
  (call $__rt_heap_free (local.get $buf))
  (local.set $cell (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (local.get $sptr)) (local.get $slen)))
  (local.get $cell))
"#;

/// Adds `__rt_var_dump`, which needs the nine fixed fragments the command data region owns.
///
/// Emitted from `runtime::emit_failure_runtime` for that reason rather than beside the other
/// builtins: the fragment offsets are only known there.
pub(super) fn emit_var_dump_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 9);
    let (int_ptr, int_len) = offsets[0];
    // Reserved: php renders a `var_dump` float with serialize_precision (17), not the
    // precision (14) `echo` uses, so the existing formatter would print a DIFFERENT number
    // for some values. The fragment stays laid out so the region offsets do not move.
    let (_float_ptr, _float_len) = offsets[1];
    let (str_ptr, str_len) = offsets[2];
    let (mid_ptr, mid_len) = offsets[3];
    let (suffix_ptr, suffix_len) = offsets[4];
    let (close_ptr, close_len) = offsets[5];
    let (true_ptr, true_len) = offsets[6];
    let (false_ptr, false_len) = offsets[7];
    let (null_ptr, null_len) = offsets[8];
    wm.add_raw_func(&format!(
        r#"(func $__rt_var_dump (param $cell i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $tptr i32) (local $tlen i32)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eqz (local.get $tag))                                   ;; tag 0 = int
    (then
      (call $__rt_echo_str (i32.const {int_ptr}) (i64.const {int_len}))
      (call $__rt_itoa (local.get $lo) (global.get $__float_scratch))
      (local.set $tlen)
      (local.set $tptr)
      (call $__rt_echo_str (local.get $tptr) (i64.extend_i32_u (local.get $tlen)))
      (call $__rt_echo_str (i32.const {close_ptr}) (i64.const {close_len}))
      (return (i64.const 9223372036854775806))))
  (if (i64.eq (local.get $tag) (i64.const 1))                      ;; tag 1 = string
    (then
      (call $__rt_echo_str (i32.const {str_ptr}) (i64.const {str_len}))
      (call $__rt_itoa (local.get $hi) (global.get $__float_scratch))
      (local.set $tlen)
      (local.set $tptr)
      (call $__rt_echo_str (local.get $tptr) (i64.extend_i32_u (local.get $tlen)))
      (call $__rt_echo_str (i32.const {mid_ptr}) (i64.const {mid_len}))
      (call $__rt_echo_str (i32.wrap_i64 (local.get $lo)) (local.get $hi))
      (call $__rt_echo_str (i32.const {suffix_ptr}) (i64.const {suffix_len}))
      (return (i64.const 9223372036854775806))))
  (if (i64.eq (local.get $tag) (i64.const 3))                      ;; tag 3 = bool
    (then
      (if (i64.eqz (local.get $lo))
        (then (call $__rt_echo_str (i32.const {false_ptr}) (i64.const {false_len})))
        (else (call $__rt_echo_str (i32.const {true_ptr}) (i64.const {true_len}))))
      (return (i64.const 9223372036854775806))))
  ;; Every other tag is refused by the capability audit, so reaching here would be a gate bug.
  (call $__rt_echo_str (i32.const {null_ptr}) (i64.const {null_len}))
  (i64.const 9223372036854775806))
"#
    ));
}

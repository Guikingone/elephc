//! Purpose:
//! Regression tests for extern return normalization against C callees that leave unspecified register bits dirty.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - The C fixture returns full 64-bit words whose low bits encode the declared C return type, mirroring
//!   what the SysV x86_64 and AAPCS64 ABIs let a callee leave above a narrow return value.
//! - Narrow-argument observers take a full 32-bit register so they report exactly what the caller
//!   left there, which is how the de-facto extension contract for 8 and 16-bit arguments is checked.
//! - The fixture is assembled into the test program itself, so no shared library or search path is needed.

use super::*;

/// C fixture whose functions are declared in PHP with narrower return types than the word they return.
const DIRTY_RETURN_FIXTURE: &str = r#"
#include <stdint.h>

// Declared as `bool` in PHP: bits 0..7 say false, bit 8 is garbage the ABI allows.
uint64_t elephc_test_dirty_false(void) { return 0x100; }

// Declared as `bool` in PHP: the low byte says true, every other bit is garbage.
uint64_t elephc_test_dirty_true(void) { return 0xffffffffffffff01ull; }

// Declared as `i32` or `u32`: the low half holds the value, the high half is garbage or clean,
// so only the declared width and signedness decide what PHP reads.
uint64_t elephc_test_dirty_32_positive(void) { return 0xffffffff00000005ull; }
uint64_t elephc_test_dirty_32_negative(void) { return 0x00000000fffffffbull; }

// Declared as `i16` or `u16`.
uint64_t elephc_test_dirty_16_positive(void) { return 0xffffffffffff0005ull; }
uint64_t elephc_test_dirty_16_negative(void) { return 0x000000000000fffbull; }

// Declared as `i8` or `u8`.
uint64_t elephc_test_dirty_8_positive(void) { return 0xffffffffffffff05ull; }
uint64_t elephc_test_dirty_8_negative(void) { return 0x00000000000000fbull; }

// Declared as `int`, `i64` and `u64`: every bit is the value.
int64_t elephc_test_wide_i64(void) { return -5000000000ll; }
uint64_t elephc_test_wide_u64(void) { return 0xffffffffffffffffull; }

// Narrow-argument observers: declared in PHP with a narrow parameter but reading the whole
// 32-bit argument register, so they report exactly what the caller left there.
uint32_t elephc_test_observe_u8(uint32_t raw) { return raw; }
int32_t elephc_test_observe_i8(int32_t raw) { return raw; }
uint32_t elephc_test_observe_u16(uint32_t raw) { return raw; }
int32_t elephc_test_observe_i16(int32_t raw) { return raw; }
uint32_t elephc_test_observe_u32(uint32_t raw) { return raw; }
"#;

/// Verifies a C `bool` return keeps only its low byte: an ABI-valid false with dirty upper bits reads as false
/// through `var_dump`, truthiness, identity comparison, and a stored local.
#[test]
fn test_ffi_extern_bool_return_uses_only_the_low_byte() {
    let out = compile_and_run_with_c_fixture(
        r#"<?php
extern function elephc_test_dirty_false(): bool;
extern function elephc_test_dirty_true(): bool;
var_dump(elephc_test_dirty_false());
var_dump(elephc_test_dirty_true());
echo elephc_test_dirty_false() ? "truthy" : "falsy", "\n";
$ok = elephc_test_dirty_false();
var_dump($ok === false);
var_dump(elephc_test_dirty_true() === true);
"#,
        DIRTY_RETURN_FIXTURE,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(true)\nfalsy\nbool(true)\nbool(true)\n"
    );
}

/// Verifies integer returns widen from the declared C width: narrow signed returns sign-extend, narrow
/// unsigned returns zero-extend, and 64-bit returns pass through untouched (a `u64` above
/// `PHP_INT_MAX` reads as the same bits reinterpreted as a signed `int`).
#[test]
fn test_ffi_extern_int_returns_widen_from_the_declared_c_width() {
    let out = compile_and_run_with_c_fixture(
        r#"<?php
extern function elephc_test_dirty_32_positive(): i32;
extern function elephc_test_dirty_32_negative(): i32;
extern function elephc_test_dirty_16_positive(): i16;
extern function elephc_test_dirty_16_negative(): i16;
extern function elephc_test_dirty_8_positive(): i8;
extern function elephc_test_dirty_8_negative(): i8;
extern function elephc_test_wide_i64(): int;
extern function elephc_test_wide_u64(): u64;
echo elephc_test_dirty_32_positive(), " ", elephc_test_dirty_32_negative(), "\n";
echo elephc_test_dirty_16_positive(), " ", elephc_test_dirty_16_negative(), "\n";
echo elephc_test_dirty_8_positive(), " ", elephc_test_dirty_8_negative(), "\n";
echo elephc_test_wide_i64(), " ", elephc_test_wide_u64(), "\n";
"#,
        DIRTY_RETURN_FIXTURE,
    );
    assert_eq!(out, "5 -5\n5 -5\n5 -5\n-5000000000 -1\n");
}

/// Verifies unsigned narrow returns zero-extend the same words the signed test sign-extends, and that
/// `i64` is an alias of `int`.
#[test]
fn test_ffi_extern_unsigned_int_returns_zero_extend() {
    let out = compile_and_run_with_c_fixture(
        r#"<?php
extern function elephc_test_dirty_32_negative(): u32;
extern function elephc_test_dirty_16_negative(): u16;
extern function elephc_test_dirty_8_negative(): u8;
extern function elephc_test_wide_i64(): i64;
echo elephc_test_dirty_32_negative(), " ", elephc_test_dirty_16_negative(), " ", elephc_test_dirty_8_negative(), "\n";
echo elephc_test_wide_i64(), "\n";
"#,
        DIRTY_RETURN_FIXTURE,
    );
    assert_eq!(out, "4294967291 65531 251\n-5000000000\n");
}

/// Verifies 8 and 16-bit arguments are reduced modulo their width and extended to 32 bits by the
/// caller, while 32-bit arguments pass through and the callee reads only its declared bits.
#[test]
fn test_ffi_extern_narrow_int_args_are_truncated_and_extended_by_the_caller() {
    let out = compile_and_run_with_c_fixture(
        r#"<?php
extern function elephc_test_observe_u8(u8 $value): u32;
extern function elephc_test_observe_i8(i8 $value): i32;
extern function elephc_test_observe_u16(u16 $value): u32;
extern function elephc_test_observe_i16(i16 $value): i32;
extern function elephc_test_observe_u32(u32 $value): u32;
$big = 4294967297;
echo elephc_test_observe_u8(300), " ", elephc_test_observe_u8(-1), "\n";
echo elephc_test_observe_i8(200), " ", elephc_test_observe_i8(-1), "\n";
echo elephc_test_observe_u16(70000), " ", elephc_test_observe_i16(40000), "\n";
echo elephc_test_observe_u32($big), "\n";
"#,
        DIRTY_RETURN_FIXTURE,
    );
    assert_eq!(out, "44 255\n-56 -1\n4464 -25536\n1\n");
}

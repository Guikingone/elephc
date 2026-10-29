//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of a scalar stored in a
//! property declared with another scalar type, which PHP's weak mode converts (or refuses with a
//! catchable `TypeError`) for instance and static slots alike.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout.
//! - Expected output is verbatim `LC_ALL=C php -n` 8.5.

use super::*;

/// The assignment's value is the converted one the slot holds, and a string that does not
/// convert raises a `TypeError` the enclosing `try` catches, static slots included.
#[test]
fn test_typed_property_weak_scalar_conversion() {
    let out = compile_and_run(
        r#"<?php
class P { public int $count = 0; public float $ratio = 0.0; public string $s = ''; public bool $b = false; public static int $n = 0; }
$p = new P;
$x = $p->count = "12";
var_dump($x, $p->count);
var_dump($p->ratio = 3, $p->s = 5, $p->b = "x", P::$n = "3");
try { $p->count = "abc"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { P::$n = "abc"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
var_dump($p->count, P::$n);
"#,
    );
    assert_eq!(
        out,
        "int(12)\nint(12)\nfloat(3)\nstring(1) \"5\"\nbool(true)\nint(3)\n\
Cannot assign string to property P::$count of type int\n\
Cannot assign string to property P::$n of type int\nint(12)\nint(3)\n"
    );
}

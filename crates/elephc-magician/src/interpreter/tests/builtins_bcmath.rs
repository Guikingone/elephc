//! Purpose:
//! Interpreter tests for the PHP BCMath procedural builtin surface.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - Fixtures cover shared scale, exact decimal outputs, array results, discovery, and Throwables.

use std::sync::{Mutex, MutexGuard};

use super::super::*;
use super::support::*;

static BCMATH_SCALE_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Serializes Magician BCMath tests and restores their shared process scale on drop.
struct BcmathScaleTestGuard {
    previous: i32,
    _lock: MutexGuard<'static, ()>,
}

impl BcmathScaleTestGuard {
    /// Acquires the BCMath test lock, resets scale to zero, and saves the prior value.
    fn acquire() -> Self {
        let lock = BCMATH_SCALE_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = elephc_bcmath::set_scale(0).expect("reset bcmath scale");
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for BcmathScaleTestGuard {
    /// Restores the prior scale before releasing the Magician BCMath test lock.
    fn drop(&mut self) {
        elephc_bcmath::set_scale(i64::from(self.previous)).expect("restore bcmath scale");
    }
}

/// Verifies all BCMath result shapes and process-scale reads through direct eval calls.
#[test]
fn execute_program_dispatches_bcmath_arithmetic() {
    let _guard = BcmathScaleTestGuard::acquire();
    let program = parse_fragment(
        br#"bcscale(4);
echo bcadd("1.234", "5"), ":";
echo bcsub("5", "1.25", 2), ":";
echo bcmul("2.5", "4", 3), ":";
echo bcdiv("105", "6.55957", 3), ":";
echo bcmod("5", "3", 0), ":";
echo bcpow("2", "-3", 4), ":";
echo bcpowmod("4", "13", "497", 0), ":";
echo bcsqrt("2", 3), ":";
echo bccomp("1.00", "1.001", 2), ":";
echo bcceil("-1.2"), ":", bcfloor("-1.2"), ":";
echo bcround("9.5", 0, 1), ":", bcround("9.5", 0, 2), ":";
$parts = bcdivmod("-5", "3");
echo $parts[0], ":", $parts[1], ":";
return bcscale();"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        "6.2340:3.75:10.000:16.007:2:0.1250:445:1.414:0:-1:-2:10:9:-1:-2.0000:"
    );
    assert_eq!(values.get(result), FakeValue::Int(4));
}

/// Verifies eval maps directional rounding mode integers exactly like PHP 8.4.
#[test]
fn execute_program_dispatches_bcround_directional_modes() {
    let _guard = BcmathScaleTestGuard::acquire();
    let program = parse_fragment(
        br#"foreach ([5, 6, 7, 8] as $mode) {
    echo bcround("9.5", 0, $mode), ":", bcround("-9.5", 0, $mode), "|";
}
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(values.output, "10:-9|9:-10|9:-9|10:-10|");
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies eval follows PHP's digitless-zero and verbatim-whitespace numeric grammar.
#[test]
fn execute_program_dispatches_bcmath_numeric_grammar() {
    let _guard = BcmathScaleTestGuard::acquire();
    let program = parse_fragment(
        br#"foreach (["", "+", "-", ".", "+.", "-."] as $zero) {
    echo bcadd($zero, "2", 2), "|";
}
foreach ([" 0", "0 ", "\t0"] as $bad) {
    try {
        bcadd($bad, "2", 2);
    } catch (ValueError $e) {
        echo get_class($e), "|";
    }
}
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        "2.00|2.00|2.00|2.00|2.00|2.00|ValueError|ValueError|ValueError|"
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies case-insensitive names, named/callable binding, and BCMath discovery under eval.
#[test]
fn execute_program_dispatches_bcmath_metadata_and_callables() {
    let _guard = BcmathScaleTestGuard::acquire();
    let program = parse_fragment(
        br#"echo BCADD(num1: "1", num2: "2", scale: 0), ":";
echo call_user_func("bcsqrt", "9", 0), ":";
echo call_user_func_array("bcpowmod", ["num" => "4", "exponent" => "13", "modulus" => "497", "scale" => 0]), ":";
echo function_exists("bcadd") ? "F" : "bad";
echo extension_loaded("bcmath") ? "E" : "bad";
return is_callable("bcround");"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(values.output, "3:3:445:FE");
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies malformed input and zero division become catchable PHP error classes with messages.
#[test]
fn execute_program_dispatches_bcmath_throwables() {
    let _guard = BcmathScaleTestGuard::acquire();
    let program = parse_fragment(
        br#"try {
    bcadd("1e2", "1");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "|";
}
try {
    bcdiv("1", "0");
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage();
}
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        "ValueError:bcadd(): Argument #1 ($num1) is not well-formed|DivisionByZeroError:Division by zero"
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

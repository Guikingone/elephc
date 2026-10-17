//! Purpose:
//! Checks ownership of immutable Number operators, replacements, and exception operands.
//!
//! Called from:
//! - The codegen runtime GC integration group under heap-debug instrumentation.
//!
//! Key details:
//! - Every owned Number/string/cell must be released on normal and caught-error paths.

use crate::support::*;

/// Repeatedly constructs and replaces native and boxed Numbers without retaining temporaries.
#[test]
fn test_bcmath_number_heap_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function calculate(mixed $n): mixed { return ($n + '0.25') * 2; }
for ($i = 0; $i < 16; $i++) {
    $n = new BcMath\Number('1.20');
    $alias = $n;
    $old = $n++;
    $n += new BcMath\Number('.30');
    $result = calculate($n);
    $pair = $result->divmod(2);
    $text = (string)($result / 4);
    unset($n, $alias, $old, $result, $pair, $text);
}
echo 'done';
"#);
    assert_eq!(out.stdout, "done", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A right operand can replace the left variable without changing the captured operator value.
#[test]
fn test_bcmath_number_operand_order_and_capture() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function replace(BcMath\Number &$n): BcMath\Number {
    echo 'R';
    $n = new BcMath\Number(10);
    return new BcMath\Number(2);
}
$n = new BcMath\Number('1.20');
echo ($n + replace($n))->value, '|', $n->value;
unset($n);
"#);
    assert_eq!(out.stdout, "R3.20|10", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Failed operations retire fresh temporary operands before entering a same-frame catch.
#[test]
fn test_bcmath_number_exception_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function add_unknown_number(mixed $a, mixed $b): mixed { return $a + $b; }
for ($i = 0; $i < 8; $i++) {
    try { new BcMath\Number('1.20') / new BcMath\Number(0); }
    catch (DivisionByZeroError $e) { unset($e); }
    try { new BcMath\Number('1.20') + 'invalid'; }
    catch (ValueError $e) { unset($e); }
    try { add_unknown_number(new BcMath\Number('1.20'), []); }
    catch (TypeError $e) { unset($e); }
}
echo 'caught';
"#);
    assert_eq!(out.stdout, "caught", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Dynamic eval releases native operator results and hidden argument collectors after errors.
#[test]
fn test_bcmath_number_eval_heap_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$n = new BcMath\Number('1.20');
$source = $argc > 1 ? 'echo "unexpected";' : 'for ($i = 0; $i < 8; $i++) { $result = ($n + "0.30") * 2; $result++; try { $result / new \\BcMath\\Number(0); } catch (DivisionByZeroError $e) { unset($e); } unset($result); } echo "eval";';
eval($source);
unset($n, $source);
"#);
    assert_eq!(out.stdout, "eval", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Silent empty probes share Number truthiness for properties, array items, and magic getters.
#[test]
fn test_bcmath_number_empty_places() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class NumberBox {
    public BcMath\Number $number;
    public static BcMath\Number $shared;
}
class MagicNumberBox {
    public function __isset(string $name): bool { return true; }
    public function __get(string $name): BcMath\Number { return new BcMath\Number('0.00'); }
}
$box = new NumberBox();
var_dump(empty($box->number), empty(NumberBox::$shared));
$box->number = new BcMath\Number('0.00');
NumberBox::$shared = new BcMath\Number('0.00');
$items = [new BcMath\Number('0.00')];
var_dump(empty($box->number), empty(NumberBox::$shared), empty($items[0]), empty((new MagicNumberBox())->missing), empty($undefined));
unset($box, $items);
"#);
    assert_eq!(out.stdout, "bool(true)\n".repeat(7), "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

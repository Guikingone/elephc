//! Purpose:
//! Regression coverage for array-valued Reflection constants and profile-dependent defaults.
//!
//! Called from:
//! - The `codegen_tests` integration test binary.
//!
//! Key details:
//! - Exercises native/eval inventories, constant APIs, key normalization and value independence.

use super::*;

/// Native and opaque eval inventories retain the parser's negative-key constant arrays.
#[test]
fn test_mixed_literal_constant_inventory_and_eval_keep_negative_keys() {
    let out = compile_and_run(r#"<?php
const X = [-5 => 'a', 'b'];
const NESTED = ['value' => [-5 => 'a', 'b']];
echo json_encode(get_defined_constants(true)['user']['X']), '|';
echo json_encode(get_defined_constants()['NESTED']), '|';
eval(substr("xecho json_encode(X), '|', json_encode(NESTED);", $argc));
"#);
    assert_eq!(out, "{\"-5\":\"a\",\"-4\":\"b\"}|{\"value\":{\"-5\":\"a\",\"-4\":\"b\"}}|{\"-5\":\"a\",\"-4\":\"b\"}|{\"value\":{\"-5\":\"a\",\"-4\":\"b\"}}");
}

/// Normalizing implicit keys keeps source-order side effects under both PHP profiles.
#[test]
fn test_mixed_literal_profile_normalization_preserves_evaluation_order() {
    let source = r#"<?php
function mark(string $value): string { echo $value; return $value; }
$value = [-5 => mark('a'), mark('b'), -5 => mark('c')];
echo '|', json_encode($value);
"#;
    for (profile, expected) in [
        (elephc::php_version::PhpVersion::Php82, "abc|{\"-5\":\"c\",\"0\":\"b\"}"),
        (elephc::php_version::PhpVersion::Php83, "abc|{\"-5\":\"c\",\"-4\":\"b\"}"),
    ] {
        assert_eq!(compile_and_run_with_php_version(source, profile), expected);
    }
}

/// Float-key metadata uses PHP's modulo conversion rather than Rust's saturating cast.
#[test]
fn test_reflection_constant_arrays_wrap_out_of_range_float_keys() {
    let out = compile_and_run(r#"<?php
class FloatKeyConstants {
    const VALUE = [1e20 => 'p', -1e20 => 'n', 9223372036854775808.0 => 'min', 1e100 => 'zero'];
}
function withFloatKeys($value = FloatKeyConstants::VALUE) {}
$class = new ReflectionClass(FloatKeyConstants::class);
$parameter = new ReflectionParameter('withFloatKeys', 0);
echo json_encode(array_keys($class->getConstant('VALUE'))), '|';
echo json_encode(array_keys($parameter->getDefaultValue()));
"#);
    let keys = "[7766279631452241920,-7766279631452241920,-9223372036854775808,0]";
    assert_eq!(out, format!("{keys}|{keys}"));
}

/// Mutating one fetched nested array cannot overwrite stored Reflection constant metadata.
#[test]
fn test_reflection_constant_arrays_keep_fetched_values_independent() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ArrayConstants { const VALUE = [['x' => 1], ['x' => 2]]; }
$class = new ReflectionClass(ArrayConstants::class);
$first = $class->getConstant('VALUE');
$first[0]['x'] = 9;
echo json_encode($first), '|', json_encode($class->getConstant('VALUE')), '|';
$constant = $class->getReflectionConstant('VALUE');
$second = $constant->getValue();
$second[1]['x'] = 8;
echo json_encode($second), '|', json_encode($constant->getValue());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "[{\"x\":9},{\"x\":2}]|[{\"x\":1},{\"x\":2}]|[{\"x\":1},{\"x\":8}]|[{\"x\":1},{\"x\":2}]");
}

/// Verifies that reflection constant metadata folds array constants with PHP's key rules:
/// duplicate keys collapse (first position, last value), spread integer keys renumber from
/// the highest key (negative or not), NAN/INF keys cast to 0, and lists normalize to the
/// packed shape (regression for #1230).
#[test]
fn test_reflection_constant_folder_php_key_rules() {
    let out = compile_and_run_capture(
        r#"<?php
const A = [1, 2];
const B = [3];
const DUP = [1 => "a", 1 => "b"];
const S = [0, ...DUP];
const NSTR = ["2" => "a", "b"];
const NEG = [-5 => "a", "b"];
const NEGSPREAD = [-5 => 1, ...["a" => 2, 0 => 3]];
const EXPL = [0 => "a", "b"];
const NANINF = [INF => 1, NAN => 2];
const SPREADONLY = [1, ...[5, 6]];
function gA($x = A) {}
function gB($x = B) {}
function gS($x = S) {}
function gNSTR($x = NSTR) {}
function gNEG($x = NEG) {}
function gNEGSPREAD($x = NEGSPREAD) {}
function gEXPL($x = EXPL) {}
function gNANINF($x = NANINF) {}
function gSPREADONLY($x = SPREADONLY) {}
// One literal constructor call per name: the AOT checker requires string literals for
// reflection owner constructors (dynamic-name lookup is a separate, later gap).
$p = new ReflectionParameter("gA", 0);
echo "A:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gB", 0);
echo "B:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gS", 0);
echo "S:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gNSTR", 0);
echo "NSTR:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gNEG", 0);
echo "NEG:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gNEGSPREAD", 0);
echo "NEGSPREAD:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gEXPL", 0);
echo "EXPL:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gNANINF", 0);
echo "NANINF:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
$p = new ReflectionParameter("gSPREADONLY", 0);
echo "SPREADONLY:", json_encode($p->getDefaultValue()), " list=", array_is_list($p->getDefaultValue()) ? 1 : 0, "\n";
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(
        out.stdout,
        "A:[1,2] list=1\nB:[3] list=1\nS:[0,\"b\"] list=1\nNSTR:{\"2\":\"a\",\"3\":\"b\"} list=0\nNEG:{\"-5\":\"a\",\"-4\":\"b\"} list=0\nNEGSPREAD:{\"-5\":1,\"a\":2,\"-4\":3} list=0\nEXPL:[\"a\",\"b\"] list=1\nNANINF:[2] list=1\nSPREADONLY:[1,5,6] list=1\n"
    );
}

/// Verifies that parameters defaulted to array-valued constants, global or class,
/// reflect the default the way PHP does (regression for #1230).
#[test]
fn test_reflection_parameter_array_valued_constant_defaults() {
    let out = compile_and_run_capture(
        r#"<?php
const ARR = [1, 2, 3];
const ASSOC = ["a" => 1, "1" => 2];
const SCALAR = 7;
const NESTED = [1, ["deep"], "s", null, ["m" => true]];

class ConstHolder {
    public const LIST = [10, 20];
    public const MAP = ["k" => 5];
    public const SCALAR = 9;
}

function withGlobalArray($x = ARR) { return 0; }
function withGlobalAssoc($x = ASSOC) { return 0; }
function withGlobalScalar($x = SCALAR) { return 0; }
function withNested($x = NESTED) { return 0; }
function withClassList($x = ConstHolder::LIST) { return 0; }
function withClassMap($x = ConstHolder::MAP) { return 0; }
function withClassScalar($x = ConstHolder::SCALAR) { return 0; }

$p = new ReflectionParameter("withGlobalArray", 0);
echo "ga: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withGlobalAssoc", 0);
echo "go: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withGlobalScalar", 0);
echo "gs: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withNested", 0);
echo "ne: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withClassList", 0);
echo "cl: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withClassMap", 0);
echo "cm: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
$p = new ReflectionParameter("withClassScalar", 0);
echo "cs: ", $p->isDefaultValueAvailable() ? "y" : "n", " ", $p->isDefaultValueConstant() ? $p->getDefaultValueConstantName() : "-", " ", json_encode($p->getDefaultValue()), "\n";
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(
        out.stdout,
        "ga: y ARR [1,2,3]\ngo: y ASSOC {\"a\":1,\"1\":2}\ngs: y SCALAR 7\nne: y NESTED [1,[\"deep\"],\"s\",null,{\"m\":true}]\ncl: y ConstHolder::LIST [10,20]\ncm: y ConstHolder::MAP {\"k\":5}\ncs: y ConstHolder::SCALAR 9\n"
    );
}

/// Verifies that `ReflectionClass::getConstant()`, `getConstants()`, and
/// `ReflectionClassConstant::getValue()` reflect array-valued constants,
/// renumbering mixed-spread integer keys the way PHP does (regression for #1230).
#[test]
fn test_reflection_constant_apis_return_array_values() {
    let out = compile_and_run_capture(
        r#"<?php
const OTHER = ["b" => 7, 1 => 8];
class ReflectArrayConstHolder {
    public const LIST = [1, 2, 3];
    public const MAP = ["a" => 1, "1" => 2];
    public const NESTED = [1, ["deep"], "s", null, ["m" => true]];
    public const MIXED = ["first" => 1, ...OTHER, 9];
    public const SCALAR = 7;
}
$ref = new ReflectionClass(ReflectArrayConstHolder::class);
echo json_encode($ref->getConstant("LIST")), "\n";
echo json_encode($ref->getConstant("MAP")), "\n";
echo json_encode($ref->getConstant("NESTED")), "\n";
echo json_encode($ref->getConstant("MIXED")), "\n";
$all = $ref->getConstants();
echo count($all), ":", json_encode($all["MIXED"]), "\n";
$c = $ref->getReflectionConstant("MIXED");
echo $c ? json_encode($c->getValue()) : "false", "\n";
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(
        out.stdout,
        "[1,2,3]\n{\"a\":1,\"1\":2}\n[1,[\"deep\"],\"s\",null,{\"m\":true}]\n{\"first\":1,\"b\":7,\"0\":8,\"1\":9}\n5:{\"first\":1,\"b\":7,\"0\":8,\"1\":9}\n{\"first\":1,\"b\":7,\"0\":8,\"1\":9}\n"
    );
}

/// Reflection folds a constant array's implicit integer keys with the selected PHP profile's rule:
/// PHP 8.2 restarts at 0 after a negative key, PHP 8.3+ continues at max + 1.
#[test]
fn test_reflection_constant_array_implicit_keys_follow_php_profile() {
    let source = r#"<?php
class NegativeKeyConstHolder {
    public const VALUE = [-5 => "a", "b"];
}
$class = new ReflectionClass(NegativeKeyConstHolder::class);
$constant = $class->getConstant("VALUE");
$constants = $class->getConstants();
echo PHP_VERSION, " ", json_encode($constant), " ", json_encode($constants["VALUE"]), "\n";
"#;
    let php82 = compile_and_run_with_php_version(source, elephc::php_version::PhpVersion::Php82);
    assert_eq!(
        php82,
        "8.2.0 {\"-5\":\"a\",\"0\":\"b\"} {\"-5\":\"a\",\"0\":\"b\"}\n"
    );

    let php83 = compile_and_run_with_php_version(source, elephc::php_version::PhpVersion::Php83);
    assert_eq!(
        php83,
        "8.3.0 {\"-5\":\"a\",\"-4\":\"b\"} {\"-5\":\"a\",\"-4\":\"b\"}\n"
    );
}

/// Pins the known limitation that a parameter defaulted to an enum case reflects as having
/// no default (PHP reports the default; the enum-case default form is a later gap). A case
/// nested in an array constant is a separate, later gap and is not exercised here: the
/// base already fatals registering such a constant with the eval bridge.
#[test]
fn test_reflection_parameter_enum_case_defaults_report_unavailable() {
    let out = compile_and_run_capture(
        r#"<?php
enum PinEnum { case Hearts; }
function withEnumCase($x = PinEnum::Hearts) {}
echo (new ReflectionParameter("withEnumCase", 0))->isDefaultValueAvailable() ? "y" : "n", "\n";
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "n\n");
}

/// A default spelled `[-5 => "a", "b"]` (which the parser keeps as a mixed literal, because the
/// bare entry's key depends on the PHP profile) materializes and reflects exactly like the hash
/// PHP 8.5 makes of it: property and static property defaults, method and function parameter
/// defaults called directly, through `call_user_func()` and through a first-class callable, and
/// `ReflectionParameter`/`ReflectionProperty::getDefaultValue()`. Regression for #1340.
#[test]
fn test_mixed_literal_defaults_materialize_and_reflect() {
    let out = compile_and_run(
        r#"<?php
class C {
    public array $p = [-5 => "a", "b"];
    public static array $s = [-5 => "a", "b"];
    public function m(array $x = [-5 => "a", "b"]): array { return $x; }
}
function f(array $x = [-5 => "a", "b"]): array { return $x; }
var_dump((new C())->p, C::$s, (new C())->m(), f());
var_dump((new ReflectionFunction('f'))->getParameters()[0]->getDefaultValue());
var_dump((new ReflectionMethod('C', 'm'))->getParameters()[0]->getDefaultValue());
var_dump((new ReflectionProperty('C', 'p'))->getDefaultValue());
var_dump(call_user_func('f'));
$fn = f(...);
var_dump($fn());
"#,
    );
    let pair = "array(2) {\n  [-5]=>\n  string(1) \"a\"\n  [-4]=>\n  string(1) \"b\"\n}\n";
    assert_eq!(out, pair.repeat(9));
}

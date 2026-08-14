//! Purpose:
//! Interpreter tests for the array-taking builtin `TypeError` family.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - Every expected message is quoted from `php -n` 8.5.6, including the variadic
//!   spellings that carry no `($name)` segment.

use super::super::*;
use super::support::*;

/// Verifies eval `count()` throws PHP's `Countable|array` TypeError for non-countables.
#[test]
fn execute_program_count_rejects_non_countable_with_php_type_error() {
    let program = parse_fragment(
        br#"class EvalCountableSeven implements Countable { public function count(): int { return 7; } }
$false = false;
try { count($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { count(null); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { count("s"); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { count(new stdClass()); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|" . count([1, 2]) . "|" . count(new EvalCountableSeven());
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "count(): Argument #1 ($value) must be of type Countable|array, false given",
            "|count(): Argument #1 ($value) must be of type Countable|array, null given",
            "|count(): Argument #1 ($value) must be of type Countable|array, string given",
            "|count(): Argument #1 ($value) must be of type Countable|array, stdClass given",
            "|2|7",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies eval `in_array()` names the offending VALUE's PHP type, not just its tag.
#[test]
fn execute_program_in_array_rejects_non_array_haystack_with_php_type_error() {
    let program = parse_fragment(
        br#"$false = false;
$true = true;
try { in_array("x", $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { in_array("x", $true); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { in_array("x", null); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { in_array("x", 3); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { in_array("x", 3.5); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { in_array("x", "s"); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
return in_array(2, [1, 2, 3]);"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "in_array(): Argument #2 ($haystack) must be of type array, false given",
            "|in_array(): Argument #2 ($haystack) must be of type array, true given",
            "|in_array(): Argument #2 ($haystack) must be of type array, null given",
            "|in_array(): Argument #2 ($haystack) must be of type array, int given",
            "|in_array(): Argument #2 ($haystack) must be of type array, float given",
            "|in_array(): Argument #2 ($haystack) must be of type array, string given",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies a fully variadic builtin drops the `($name)` segment on EVERY argument.
///
/// `array_merge()` is the family's odd member: `array_diff()` names its first argument
/// `$array` and leaves only the tail unnamed, while `array_merge()` names none of them.
#[test]
fn execute_program_variadic_array_builtins_omit_the_parameter_name() {
    let program = parse_fragment(
        br#"$false = false;
try { array_merge($false, []); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_merge([], $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_diff($false, []); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_diff([], $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_intersect_key([], $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_map("strlen", $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "array_merge(): Argument #1 must be of type array, false given",
            "|array_merge(): Argument #2 must be of type array, false given",
            "|array_diff(): Argument #1 ($array) must be of type array, false given",
            "|array_diff(): Argument #2 must be of type array, false given",
            "|array_intersect_key(): Argument #2 must be of type array, false given",
            "|array_map(): Argument #2 ($array) must be of type array, false given",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies the BY-REFERENCE ordering receivers throw rather than fatally refusing.
///
/// `sort($d)` on a `scandir()` failure is the motivating case: the receiver is written
/// back only after the type check, so a non-array leaves through a catchable TypeError.
#[test]
fn execute_program_by_reference_array_mutators_throw_php_type_error() {
    let program = parse_fragment(
        br#"$false = false;
$a = $false;
try { sort($a); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
$b = $false;
try { rsort($b); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
$c = $false;
try { array_push($c, 1); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
$d = $false;
try { array_pop($d); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
$e2 = $false;
try { reset($e2); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
$sorted = [3, 1, 2];
sort($sorted);
echo implode(",", $sorted);
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "sort(): Argument #1 ($array) must be of type array, false given",
            "|rsort(): Argument #1 ($array) must be of type array, false given",
            "|array_push(): Argument #1 ($array) must be of type array, false given",
            "|array_pop(): Argument #1 ($array) must be of type array, false given",
            "|reset(): Argument #1 ($array) must be of type array, false given",
            "|1,2,3",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies an eval-DECLARED class names itself, not the runtime cell's backing class.
#[test]
fn execute_program_array_builtin_type_error_names_the_eval_declared_class() {
    let program = parse_fragment(
        br#"class EvalHaystackImposter {}
$imposter = new EvalHaystackImposter();
try { in_array("x", $imposter); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_values($imposter); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "in_array(): Argument #2 ($haystack) must be of type array, EvalHaystackImposter given",
            "|array_values(): Argument #1 ($array) must be of type array, EvalHaystackImposter given",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies the single-array projections all reach the shared check.
#[test]
fn execute_program_single_array_builtins_throw_php_type_error() {
    let program = parse_fragment(
        br#"$false = false;
try { array_values($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_unique($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_flip($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_sum($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_product($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_reverse($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_slice($false, 0); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_filter($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_count_values($false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_pad($false, 1, 0); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_column($false, "x"); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
echo "|";
try { array_search("x", $false); echo "no-throw"; } catch (TypeError $e) { echo $e->getMessage(); }
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "array_values(): Argument #1 ($array) must be of type array, false given",
            "|array_unique(): Argument #1 ($array) must be of type array, false given",
            "|array_flip(): Argument #1 ($array) must be of type array, false given",
            "|array_sum(): Argument #1 ($array) must be of type array, false given",
            "|array_product(): Argument #1 ($array) must be of type array, false given",
            "|array_reverse(): Argument #1 ($array) must be of type array, false given",
            "|array_slice(): Argument #1 ($array) must be of type array, false given",
            "|array_filter(): Argument #1 ($array) must be of type array, false given",
            "|array_count_values(): Argument #1 ($array) must be of type array, false given",
            "|array_pad(): Argument #1 ($array) must be of type array, false given",
            "|array_column(): Argument #1 ($array) must be of type array, false given",
            "|array_search(): Argument #2 ($haystack) must be of type array, false given",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

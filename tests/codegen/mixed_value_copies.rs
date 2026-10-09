//! Purpose:
//! Tests PHP by-value copies of boxed values across bindings and user-call boundaries.
//!
//! Called from:
//! - `cargo test --test codegen_tests mixed_value_copies`.
//!
//! Key details:
//! - Array copies must detach zval cells while reference parameters remain aliased.
//! - Objects and resources preserve identity; every normal or exceptional owner is released.

use crate::support::*;

#[test]
fn mixed_value_copies_argument_sources_precede_fixed_and_variadic_binding() {
    assert_clean_copy(
        r#"
class Text { public function __toString(): string { echo 'bind|'; return 'text'; } }
function later(): mixed { echo 'eval|'; return boxed('tail'); }
function fixed(string $first, mixed $second): int { return 0; }
function variadic(string ...$texts): int { return 0; }
fixed(boxed(new Text()), later());
echo ';';
variadic(boxed(new Text()), later());
"#,
        "eval|bind|;eval|bind|",
    );
}

#[test]
fn mixed_value_copies_typed_array_argument_is_pinned_before_later_mutation() {
    assert_clean_copy(
        r#"
function mutate(array &$value): int { $value[0] = 2; return 0; }
function observe(array $first, int $ignored): string { return json_encode($first); }
$source = [1];
echo observe($source, mutate($source)), '|', json_encode($source);
unset($source);
"#,
        "[1]|[2]",
    );
}

#[test]
fn mixed_value_copies_immediate_closure_heap_results_outlive_descriptors() {
    assert_clean_copy(
        r#"
$source = boxed(['capture' => 3]);
$captured = (static fn (): mixed => $source)();
$argument = (static function (mixed $value): mixed { return $value; })(boxed(['argument' => 4]));
unset($source);
echo json_encode($captured), '|', json_encode($argument), '|';
(static function (): void { echo 'void'; })();
unset($captured); unset($argument);
"#,
        r#"{"capture":3}|{"argument":4}|void"#,
    );
}

#[test]
fn mixed_value_copies_unused_immediate_closures_preserve_string_bindings() {
    assert_clean_copy(
        r#"
class BrokenText {
    public function __toString(): string { throw new RuntimeException('binding'); }
}
$value = boxed(new BrokenText());
try { (static function (string $text): int { return 0; })($value); }
catch (RuntimeException $error) { echo 'fixed|'; unset($error); }
try { (static function (string ...$texts): int { return 0; })($value); }
catch (RuntimeException $error) { echo 'variadic'; unset($error); }
unset($value);
"#,
        "fixed|variadic",
    );
}

#[test]
fn mixed_value_copies_static_closure_body_throw_releases_argument_owner() {
    assert_clean_copy(
        r#"
$callback = static function (mixed $value): int { throw new RuntimeException('body'); };
try { $callback(boxed(['payload' => 1])); }
catch (RuntimeException $error) { echo 'caught'; unset($error); }
unset($callback);
"#,
        "caught",
    );
}

#[test]
fn mixed_value_copies_named_variadic_partial_conversion_unwinds_collector() {
    assert_clean_copy(
        r#"
class ValidText { public function __toString(): string { return 'owned'; } }
class BrokenText {
    public function __toString(): string { throw new RuntimeException('binding'); }
}
function consume(string ...$texts): int { return 0; }
try { consume(first: boxed(new ValidText()), second: boxed(new BrokenText())); }
catch (RuntimeException $error) { echo 'caught'; unset($error); }
"#,
        "caught",
    );
}

#[test]
fn mixed_value_copies_unused_variadic_call_preserves_string_binding_exception() {
    assert_clean_copy(
        r#"
class BrokenText {
    public function __toString(): string { throw new RuntimeException('binding'); }
}
function consume(string ...$texts): int { return 0; }
$value = boxed(new BrokenText());
try { consume($value); }
catch (RuntimeException $error) { echo 'caught'; unset($error); }
unset($value);
"#,
        "caught",
    );
}

#[test]
fn mixed_value_copies_unused_call_preserves_string_binding_exception() {
    assert_clean_copy(
        r#"
class ThrowingText {
    public function __toString(): string { throw new RuntimeException('coercion'); }
}
function bind_text(string $first, string $second): int { return 0; }
$source = boxed('payload');
$throwing = boxed(new ThrowingText());
try { bind_text(first: $source, second: $throwing); }
catch (RuntimeException $error) { echo 'caught|'; unset($error); }
echo $source;
unset($source); unset($throwing);
"#,
        "caught|payload",
    );
}

#[test]
fn mixed_value_copies_named_coercion_failure_does_not_release_consumed_snapshot() {
    assert_clean_copy(
        r#"
class ThrowingText {
    public function __toString(): string { throw new RuntimeException('coercion'); }
}
function bind_text(string $first, string $second): string { return $first . $second; }
$source = boxed('payload');
$throwing = boxed(new ThrowingText());
try { echo bind_text(first: $source, second: $throwing); }
catch (RuntimeException $error) { echo 'caught|'; unset($error); }
echo $source;
unset($source); unset($throwing);
"#,
        "caught|payload",
    );
}

#[test]
fn mixed_value_copies_preparation_unwind_skips_unevaluated_snapshots() {
    assert_clean_copy(
        r#"
function fail_first(): mixed { throw new RuntimeException('first'); }
function take_both(mixed $first, mixed $second): int { return 0; }
try { take_both(fail_first(), boxed(['unused' => 1])); }
catch (RuntimeException $error) { echo 'caught'; unset($error); }
"#,
        "caught",
    );
}

#[test]
fn mixed_value_copies_nested_argument_evaluation_unwinds_both_ledgers() {
    assert_clean_copy(
        r#"
function outer(mixed $value, int $ignored): int { return 0; }
function inner(mixed $value, int $ignored): int { return 0; }
function fail_later(): int { throw new RuntimeException('inner'); }
$source = boxed(['x' => 1]);
try { outer($source, inner(boxed(['y' => 3]), fail_later())); }
catch (RuntimeException $error) { echo 'caught|'; unset($error); }
echo json_encode($source);
unset($source);
"#,
        r#"caught|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_named_preparation_exception_releases_snapshot() {
    assert_clean_copy(
        r#"
function take_first(mixed $value, int $ignored): mixed { return $value; }
function fail_later(): int { throw new RuntimeException('argument'); }
$source = boxed(['x' => 1]);
try { take_first(value: $source, ignored: fail_later()); }
catch (RuntimeException $error) { echo 'caught|'; unset($error); }
echo json_encode($source);
unset($source);
"#,
        r#"caught|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_argument_snapshot_is_released_when_later_argument_throws() {
    assert_clean_copy(
        r#"
function take_first(mixed $value, int $ignored): mixed { return $value; }
function fail_later(): int { throw new RuntimeException('argument'); }
$source = boxed(['x' => 1]);
try { take_first($source, fail_later()); }
catch (RuntimeException $error) { echo 'caught|'; unset($error); }
echo json_encode($source);
unset($source);
"#,
        r#"caught|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_named_argument_snapshot_precedes_later_mutation() {
    assert_clean_copy(
        r#"
function take_first(mixed $value, int $ignored): mixed { return $value; }
function change_later(mixed &$value): int { $value['x'] = 2; return 0; }
$source = boxed(['x' => 1]);
$copy = take_first(value: $source, ignored: change_later($source));
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":2}|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_static_assignment_is_not_a_shared_value_cell() {
    assert_clean_copy(
        r#"
function keep_snapshot(mixed $value): mixed {
    static $saved = null;
    $saved = $value;
    $value['x'] = 2;
    return $saved;
}
$source = boxed(['x' => 1]);
$copy = keep_snapshot($source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_argument_snapshot_precedes_later_argument_mutation() {
    assert_clean_copy(
        r#"
function take_first(mixed $value, int $ignored): mixed { return $value; }
function change_later(mixed &$value): int { $value['x'] = 2; return 0; }
$source = boxed(['x' => 1]);
$copy = take_first($source, change_later($source));
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":2}|{"x":1}"#,
    );
}

#[test]
fn mixed_value_copies_from_reference_are_values_not_aliases() {
    assert_clean_copy(
        r#"
$source = boxed(['x' => 1]);
$reference =& $source;
$copy = $reference;
$copy['x'] = 2;
echo json_encode($source), '|', json_encode($reference), '|', json_encode($copy);
unset($copy); unset($reference); unset($source);
"#,
        r#"{"x":1}|{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_assignment_into_reference_copies_the_value() {
    assert_clean_copy(
        r#"
$source = boxed(['x' => 1]);
$target = boxed([]);
$reference =& $target;
$reference = $source;
$source['x'] = 2;
echo json_encode($target), '|', json_encode($reference), '|', json_encode($source);
unset($reference); unset($target); unset($source);
"#,
        r#"{"x":1}|{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_global_assignment_is_not_a_shared_value_cell() {
    assert_clean_copy(
        r#"
function publish(mixed $value): mixed {
    global $published;
    $published = $value;
    $value['x'] = 2;
    return $value;
}
$published = boxed([]);
$source = boxed(['x' => 1]);
$changed = publish($source);
echo json_encode($source), '|', json_encode($published), '|', json_encode($changed);
unset($source); unset($published); unset($changed);
"#,
        r#"{"x":1}|{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_closure_capture_is_a_value_snapshot() {
    assert_clean_copy(
        r#"
$source = boxed(['x' => 1]);
$callback = function () use ($source): mixed { return $source; };
$source['x'] = 2;
$copy = $callback();
echo json_encode($source), '|', json_encode($copy);
unset($copy); unset($callback); unset($source);
"#,
        r#"{"x":2}|{"x":1}"#,
    );
}

fn assert_clean_copy(body: &str, expected: &str) {
    let source = format!(
        "<?php\nfunction boxed(mixed $value): mixed {{ return $value; }}\n{body}\n"
    );
    let out = compile_and_run_with_heap_debug(&source);
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn mixed_value_copies_assignment_detaches_array_cell() {
    assert_clean_copy(
        r#"
$source = boxed(['x' => 1]);
$copy = $source;
$copy['x'] = 2;
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_byvalue_parameter_detaches_array_cell() {
    assert_clean_copy(
        r#"
function mutate(mixed $value): mixed { $value['x'] = 2; return $value; }
$source = boxed(['x' => 1]);
$copy = mutate($source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_identity_return_remains_an_independent_value() {
    assert_clean_copy(
        r#"
function identity(mixed $value): mixed { return $value; }
$source = boxed(['x' => 1]);
$copy = identity($source);
$copy['x'] = 2;
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_forwarded_parameter_does_not_alias_source() {
    assert_clean_copy(
        r#"
function mutate(mixed $value): mixed { $value['x'] = 2; return $value; }
function forward(mixed $value): mixed { return mutate($value); }
$source = boxed(['x' => 1]);
$copy = forward($source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_method_parameter_does_not_alias_source() {
    assert_clean_copy(
        r#"
final class Mutator {
    public function apply(mixed $value): mixed { $value['x'] = 2; return $value; }
}
$source = boxed(['x' => 1]);
$mutator = new Mutator();
$copy = $mutator->apply($source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy); unset($mutator);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_closure_parameter_does_not_alias_source() {
    assert_clean_copy(
        r#"
$source = boxed(['x' => 1]);
$callback = static function (mixed $value): mixed { $value['x'] = 2; return $value; };
$copy = $callback($source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy); unset($callback);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_reference_parameter_keeps_caller_alias() {
    assert_clean_copy(
        r#"
function mutate_ref(mixed &$value): void { $value['x'] = 2; }
$source = boxed(['x' => 1]);
mutate_ref($source);
echo json_encode($source);
unset($source);
"#,
        r#"{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_runtime_string_descriptor_does_not_alias_source() {
    assert_clean_copy(
        r#"
function mutate(mixed $value): mixed { $value['x'] = 2; return $value; }
function invoke(string $name, mixed $value): mixed { return call_user_func_array($name, [$value]); }
$source = boxed(['x' => 1]);
$copy = invoke('mutate', $source);
echo json_encode($source), '|', json_encode($copy);
unset($source); unset($copy);
"#,
        r#"{"x":1}|{"x":2}"#,
    );
}

#[test]
fn mixed_value_copies_preserve_object_identity_and_shared_properties() {
    assert_clean_copy(
        r#"
function identity(mixed $value): mixed { return $value; }
final class Payload { public int $x = 1; }
function update_payload(Payload $value): void { $value->x = 2; }
function read_payload(Payload $value): int { return $value->x; }
$source = boxed(new Payload());
$copy = identity($source);
echo $source === $copy ? 'same' : 'different';
if ($copy instanceof Payload) { update_payload($copy); }
else { echo '|not-payload'; }
if ($source instanceof Payload) { echo '|', read_payload($source); }
else { echo '|not-source'; }
unset($source); unset($copy);
"#,
        "same|2",
    );
}

#[test]
fn mixed_value_copies_preserve_resource_identity_position_and_lifetime() {
    assert_clean_copy(
        r#"
function identity(mixed $value): mixed { return $value; }
$source = identity(tmpfile());
fwrite($source, 'abcdef');
rewind($source);
$copy = identity($source);
echo $source === $copy ? 'same' : 'different';
echo '|', fread($copy, 3), '|', fread($source, 3);
unset($source);
echo '|', fwrite($copy, 'g') === 1 ? 'live' : 'closed-too-soon';
fclose($copy);
unset($copy);
"#,
        "same|abc|def|live",
    );
}

#[test]
fn mixed_value_copies_exception_releases_private_cell_without_mutating_source() {
    assert_clean_copy(
        r#"
function mutate_throw(mixed $value): void {
    $value['x'] = str_repeat('payload', 8);
    throw new RuntimeException('caught');
}
$source = boxed(['x' => 1]);
try { mutate_throw($source); }
catch (RuntimeException $error) { echo $error->getMessage(), '|'; unset($error); }
echo json_encode($source);
unset($source);
"#,
        r#"caught|{"x":1}"#,
    );
}

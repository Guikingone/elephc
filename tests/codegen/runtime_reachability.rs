//! Purpose:
//! Regression coverage for feature-gated runtime and synthetic builtin reachability.
//!
//! Called from:
//! - `cargo test` through the codegen integration-test harness.
//!
//! Key details:
//! - Plain native programs must not carry the optional eval Reflection surface.

use crate::support::{
    compile_and_run, compile_source_to_asm_with_options, fs, make_cli_test_dir,
};

/// Verifies a program without eval or Reflection omits their synthetic methods and metadata.
#[test]
fn test_plain_program_omits_unreferenced_reflection_surface() {
    let dir = make_cli_test_dir("elephc_plain_runtime_reachability");
    let (user_asm, _runtime_asm, required_libraries) =
        compile_source_to_asm_with_options("<?php echo 1;", &dir, 8_388_608, false, false);

    assert!(
        !user_asm.contains("@fn name=Reflection"),
        "plain program unexpectedly lowered synthetic Reflection methods"
    );
    assert!(
        !user_asm.contains("_eval_reflection_"),
        "plain program unexpectedly emitted eval Reflection metadata"
    );
    assert!(
        !required_libraries
            .iter()
            .any(|library| library == "elephc_magician"),
        "plain program unexpectedly requested the Magician bridge"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies repeated boxed Mixed echo sites share one dispatch ladder while a lone site stays inline.
#[test]
fn test_mixed_string_ladder_is_shared_only_for_repeated_sites() {
    const DECLARATIONS: &str = r#"<?php
class Stamp { public function __toString(): string { return "S"; } }
function pick(int $i): mixed { return $i === 0 ? new Stamp() : $i; }
"#;

    let dir = make_cli_test_dir("elephc_shared_mixed_string_many");
    let many = format!(
        "{DECLARATIONS}$a = pick(0); $b = pick(1); $c = pick(2); \
         $d = (string) pick(0); $e = (string) pick(1); echo $a; echo $b; echo $c, $d, $e;"
    );
    let (many_asm, _runtime_asm, _libraries) =
        compile_source_to_asm_with_options(&many, &dir, 8_388_608, false, false);
    assert_eq!(
        many_asm.matches("_eir_shared_mixed_echo:").count(),
        1,
        "the repeated echo ladder must have exactly one helper definition"
    );
    assert!(
        many_asm.matches("_eir_shared_mixed_echo").count() >= 4,
        "all three sites must call the shared helper"
    );
    assert_eq!(
        many_asm.matches("_eir_shared_mixed_to_string:").count(),
        1,
        "the repeated result ladder must have exactly one helper definition"
    );
    assert!(
        many_asm.matches("_eir_shared_mixed_to_string").count() >= 3,
        "both result sites must call the shared helper"
    );
    let _ = fs::remove_dir_all(&dir);

    let dir = make_cli_test_dir("elephc_shared_mixed_string_one");
    let one = format!("{DECLARATIONS}$a = pick(0); echo $a;");
    let (one_asm, _runtime_asm, _libraries) =
        compile_source_to_asm_with_options(&one, &dir, 8_388_608, false, false);
    assert!(
        !one_asm.contains("_eir_shared_mixed_echo"),
        "one site must remain inline instead of paying for a helper body"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an exception raised inside the shared `__toString` ladder reaches the caller's catch.
#[test]
fn test_shared_mixed_string_ladder_preserves_exception_unwinding() {
    let out = compile_and_run(
        r#"<?php
class Boom { public function __toString(): string { throw new RuntimeException("boom"); } }
class Fine { public function __toString(): string { return "fine"; } }
function first(mixed $value): void { echo $value; }
function second(mixed $value): void { echo $value; }
function third(mixed $value): void { echo $value; }
first(new Fine());
echo "|";
try {
    second(new Boom());
    echo "not-reached";
} catch (RuntimeException $error) {
    echo "caught:", $error->getMessage();
}
echo "|";
third(new Fine());
"#,
    );
    assert_eq!(out, "fine|caught:boom|fine");
}

/// Verifies repeated open Mixed callback sites share one complete runtime dispatch ladder.
#[test]
fn test_open_mixed_callable_dispatch_is_shared_across_sites() {
    const DECLARATIONS: &str = r#"<?php
function advance(mixed $value): mixed { return (int)$value + 1; }
class Runner {
    public mixed $callback;
    public function __construct(mixed $callback) { $this->callback = $callback; }
    public function first(int $value): int {
        return (int)call_user_func($this->callback, $value);
    }
    public function second(int $value): int {
        return (int)call_user_func($this->callback, $value);
    }
}
"#;
    let dir = make_cli_test_dir("elephc_shared_mixed_callable_many");
    let many = format!(
        r#"{DECLARATIONS}$runner = new Runner("advance");
echo $runner->first(1), ":", $runner->second(4);
"#
    );
    let (many_asm, _runtime_asm, _libraries) =
        compile_source_to_asm_with_options(&many, &dir, 8_388_608, false, false);
    assert_eq!(
        many_asm
            .matches("_eir_shared_mixed_callable_invoke:")
            .count(),
        1,
        "repeated open Mixed callback sites must emit one shared helper"
    );
    assert!(
        many_asm
            .matches("_eir_shared_mixed_callable_invoke")
            .count()
            >= 3,
        "both call sites must invoke the shared helper"
    );
    let _ = fs::remove_dir_all(&dir);

    let dir = make_cli_test_dir("elephc_shared_mixed_callable_one");
    let one = r#"<?php
function advance_one(mixed $value): mixed { return (int)$value + 1; }
class OneRunner {
    public mixed $callback;
    public function __construct(mixed $callback) { $this->callback = $callback; }
    public function run(int $value): int {
        return (int)call_user_func($this->callback, $value);
    }
}
$runner = new OneRunner("advance_one");
echo $runner->run(1);
"#;
    let (one_asm, _runtime_asm, _libraries) =
        compile_source_to_asm_with_options(one, &dir, 8_388_608, false, false);
    assert!(
        !one_asm.contains("_eir_shared_mixed_callable_invoke"),
        "one reachable open Mixed callback site must stay inline"
    );
    assert_eq!(
        one_asm
            .lines()
            .filter(|line| {
                line.contains("runtime_descriptor_release_done")
                    && line.trim_end().ends_with(':')
            })
            .count(),
        1,
        "the inline resolver must join one ownership-aware descriptor release tail"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the shared Mixed callable frame returns values and unwinds throws to its caller.
#[test]
fn test_shared_mixed_callable_dispatch_preserves_results_and_exceptions() {
    let out = compile_and_run(
        r#"<?php
function fail_now(mixed $value): mixed { throw new RuntimeException("boom:" . (string)$value); }
function keep_value(mixed $value): mixed { return $value; }
function invoke_nested(mixed $value): mixed {
    $inner = "keep_value";
    return call_user_func($inner, $value);
}
class CallableTarget {
    public function add(int $value): int { return $value + 10; }
    public static function triple(int $value): int { return $value * 3; }
    public function __invoke(int $value): int { return $value + 20; }
}
class Runner {
    public mixed $callback;
    public function __construct(mixed $callback) { $this->callback = $callback; }
    public function first(int $value): int {
        return (int)call_user_func($this->callback, $value);
    }
    public function second(int $value): int {
        return (int)call_user_func($this->callback, $value);
    }
}
$runner = new Runner("fail_now");
try {
    $runner->first(3);
    echo "not-reached";
} catch (RuntimeException $error) {
    echo "caught:", $error->getMessage();
}
$runner->callback = "keep_value";
echo "|", $runner->second(7);
$target = new CallableTarget();
$runner->callback = [$target, "add"];
echo "|", $runner->first(2);
$runner->callback = ["CallableTarget", "triple"];
echo "|", $runner->second(3);
$runner->callback = $target;
echo "|", $runner->first(4);
$runner->callback = keep_value(...);
echo "|", $runner->second(8);
$runner->callback = keep_value(...);
echo "|", $runner->first(9), ":", $runner->second(10);
$runner->callback = "invoke_nested";
echo "|", $runner->first(11);
"#,
    );
    assert_eq!(out, "caught:boom:3|7|12|9|24|8|9:10|11");
}

/// Verifies an inline string ladder saves the reserved nested-call register before using it.
#[test]
fn test_mixed_string_context_saves_the_nested_call_register() {
    let dir = make_cli_test_dir("elephc_string_context_nested_reg");
    let (user_asm, _runtime_asm, _libraries) = compile_source_to_asm_with_options(
        r#"<?php
class Stamp { public function __toString(): string { return "S"; } }
function show(mixed $value): void { echo $value; }
show(new Stamp());
"#,
        &dir,
        8_388_608,
        false,
        false,
    );

    let body = user_asm
        .split("@fn name=show")
        .nth(1)
        .expect("the show function must be emitted")
        .split("@endfn")
        .next()
        .expect("the show function must be terminated");
    let register = if cfg!(target_arch = "x86_64") {
        "r12"
    } else {
        "x19"
    };
    let first_write = body
        .find(&format!("mov {register},"))
        .or_else(|| body.find(&format!("mov {register} ,")))
        .expect("the inline ladder must move its receiver into the nested-call register");
    let saved_before = body[..first_write].lines().any(|line| {
        let line = line.trim();
        line.starts_with(&format!("stur {register},"))
            || line.starts_with(&format!("str {register},"))
            || line.starts_with(&format!("push {register}"))
            || (line.starts_with("mov QWORD PTR [rbp -")
                && line.ends_with(&format!(", {register}")))
    });
    assert!(
        saved_before,
        "the nested-call register must be saved before the ladder overwrites it:\n{body}"
    );

    let _ = fs::remove_dir_all(&dir);
}

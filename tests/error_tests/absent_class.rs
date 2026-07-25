//! Purpose:
//! Diagnostic tests for the checker's absent-class tolerance: a class name that is unresolved
//! everywhere in the closed world degrades to `Mixed` with a warning (not a hard error) in every
//! reference position, while genuinely-malformed type syntax and reserved-word misuse keep erroring.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `expect_warning` asserts the source type-checks and carries the absent-class warning; the
//!   softened typo detection is the accepted cost of compiling frameworks with optional dependencies.
//! - Reserved-word/syntax errors must still hard-error, so the tolerance is not over-broad.
//! - Unrelated same-named methods must not refine calls made through an absent dependency.

use super::*;

/// The message substring shared by every absent-class degradation warning.
const ABSENT_MESSAGE: &str = "treated as an absent optional dependency";

/// Verifies an unresolved class used as a parameter type hint is a warning, not an error: the
/// program type-checks and the absent-class warning is emitted (`array|\Process`-style hints on
/// uninstalled optional dependencies must compile).
#[test]
fn test_absent_class_type_hint_warns_not_errors() {
    expect_warning("<?php function f(\\No\\Such $x) { return $x; }", ABSENT_MESSAGE);
}

/// Verifies a union type hint mixing a known type with an absent class warns and compiles,
/// mirroring symfony's `array|\Process` optional-dependency signatures.
#[test]
fn test_absent_class_union_type_hint_warns_not_errors() {
    expect_warning(
        "<?php function f(array|\\No\\Such $cmd) { return $cmd; }",
        ABSENT_MESSAGE,
    );
}

/// Verifies `new AbsentClass()` is tolerated as `Mixed` with a warning instead of an
/// "Undefined class" error.
#[test]
fn test_absent_class_new_warns_not_errors() {
    expect_warning(
        "<?php function f() { return new \\No\\Such(); }",
        ABSENT_MESSAGE,
    );
}

/// Verifies a static call on an absent class (`Absent::from(...)`) is tolerated as `Mixed` with a
/// warning instead of an "Undefined class" error.
#[test]
fn test_absent_class_static_call_warns_not_errors() {
    expect_warning(
        "<?php function f() { return \\No\\Such::from(1); }",
        ABSENT_MESSAGE,
    );
}

/// Verifies an absent dependency receiver is not spuriously refined from unrelated methods with
/// the same name. Distinct `prepare()` return types make the receiver result genuinely gradual;
/// the following opaque driver call must therefore remain `Mixed` instead of becoming `A|B`.
#[test]
fn test_absent_class_method_result_ignores_unrelated_candidate_return_union() {
    expect_warning(
        r#"<?php
class HtmlResponse {}
class FileResponse {}
class HtmlController {
    public function prepare(string $request): HtmlResponse { return new HtmlResponse(); }
}
class FileController {
    public function prepare(string $request): FileResponse { return new FileResponse(); }
}
class OptionalAdapter {
    private \Missing\Connection $connection;
    public function execute(): void {
        $statement = $this->connection->prepare("SELECT 1");
        $statement->bindValue(1, 1);
    }
}
"#,
        ABSENT_MESSAGE,
    );
}

/// Verifies an absent-dependency receiver whose method name is declared by exactly ONE unrelated
/// class is NOT bound to that lone class's concrete nominal return type. Modeled on Symfony's
/// `private readonly \Symfony\Component\Stopwatch\Stopwatch $stopwatch` (uninstalled) calling
/// `$this->stopwatch->start(...)->stop()`: a single `TracerAdapter::start(): TracerEvent` must not
/// make `start()` resolve to `TracerEvent`, which would mis-report `TracerEvent::stop` /
/// `::getDuration` as "Undefined method". The chained call must stay gradual `Mixed` and compile.
#[test]
fn test_absent_class_method_result_ignores_lone_candidate_object_return() {
    expect_warning(
        r#"<?php
class TracerEvent {}
class TracerAdapter {
    public function start(string $name): TracerEvent { return new TracerEvent(); }
}
class TraceableRunner {
    public function __construct(private readonly \Symfony\Component\Stopwatch\Stopwatch $stopwatch) {}
    public function run(): void {
        $event = $this->stopwatch->start('run');
        $event->stop();
        $event->getDuration();
    }
}
"#,
        ABSENT_MESSAGE,
    );
}

/// Negative control for the absent-receiver gradual routing: a receiver whose class DOES exist in
/// the closed world but lacks the called method still hard-errors "Undefined method" — the
/// gradual tolerance is scoped to absent-class receivers, never a real class with a real typo.
#[test]
fn test_existing_class_missing_method_still_errors() {
    expect_error(
        "<?php class Real { public function foo(): int { return 1; } } $r = new Real(); $r->missingMethod();",
        "Undefined method: Real::missingMethod",
    );
}

/// Verifies an `instanceof`-narrowed optional extension receiver still produces a gradual method
/// result, so guarded iteration does not inherit the legacy integer fallback.
#[test]
fn test_absent_class_instanceof_method_result_is_mixed() {
    expect_warning(
        r#"<?php
function visitOptionalHosts(mixed $client): void {
    if ($client instanceof \Missing\ClusterClient) {
        foreach ($client->hosts() as $host) {
            echo $host;
        }
    }
}
"#,
        ABSENT_MESSAGE,
    );
}

/// Verifies `catch (AbsentException $e)` is tolerated with a warning instead of an
/// "Undefined class" error (an uninstalled optional-dependency exception simply never matches).
#[test]
fn test_absent_class_catch_warns_not_errors() {
    expect_warning(
        "<?php function f() { try { echo 1; } catch (\\No\\Such\\Exc $e) { echo 2; } }",
        ABSENT_MESSAGE,
    );
}

/// Guards against over-broad tolerance: `self` as a type outside a class context is a reserved-word
/// misuse that must still hard-error rather than degrade to `Mixed`.
#[test]
fn test_reserved_self_type_outside_class_still_errors() {
    expect_error(
        "<?php function f(self $x) { return $x; }",
        "Cannot use 'self' as a type outside of a class",
    );
}

/// Guards against over-broad tolerance: instantiating a class-like symbol that *is* known but is
/// not instantiable (an interface) still hard-errors. The tolerance only fires for a class name
/// absent everywhere in the closed world, so the interface guard runs before it.
#[test]
fn test_new_on_known_interface_still_errors() {
    expect_error(
        "<?php interface I {} function f() { return new I(); }",
        "Cannot instantiate interface",
    );
}

/// A class whose parent is an absent optional dependency (a supertype that exists nowhere in the
/// closed world, e.g. from an uninstalled component) degrades to a warning instead of failing the
/// whole compile: PHP never autoloads such a class on any reached path. The declaration is still
/// removed from the closed world, so its own uses fall through to the absent-optional fallback with
/// no misleading `Undefined class` cascade and no spurious checking of its unavailable method body.
#[test]
fn test_absent_parent_class_degrades_without_cascade() {
    let result = check_source_full(
        r#"<?php
class OptionalWrapper extends MissingBase {
    public function prepare($items): void {
        array_unshift($items, "default");
    }
}
function makeWrapper() {
    return new OptionalWrapper();
}
"#,
    )
    .expect("an absent optional parent must degrade to a warning, not fail the compile");
    let warnings: Vec<String> = result
        .warnings
        .iter()
        .map(|warning| warning.message.clone())
        .collect();
    assert!(
        warnings.iter().any(|message| message.contains("MissingBase")),
        "expected the absent-optional-parent warning, got: {warnings:?}"
    );
    assert!(
        !warnings
            .iter()
            .any(|message| message.contains("Undefined class: OptionalWrapper")),
        "degraded declarations must not create secondary undefined-class diagnostics: {warnings:?}"
    );
    assert!(
        !warnings
            .iter()
            .any(|message| message.contains("array_unshift() first argument")),
        "method bodies of unavailable declarations must not be checked: {warnings:?}"
    );
}

/// A class implementing an interface that is an absent optional dependency degrades to a warning
/// (the absent contract simply can never be satisfied and only matters at a real use site), rather
/// than hard-erroring the whole compile with `Unknown interface`. Mirrors Symfony's
/// `ExpressionLanguageProvider implements ExpressionFunctionProviderInterface` when
/// symfony/expression-language is not installed.
#[test]
fn test_absent_interface_implementation_degrades_to_warning() {
    let result = check_source_full(
        r#"<?php
class Provider implements \Missing\Optional\ProviderInterface {
    public function getFunctions(): array { return []; }
}
"#,
    )
    .expect("an absent optional interface must degrade to a warning, not fail the compile");
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("Missing\\Optional\\ProviderInterface")),
        "expected the absent-optional-interface warning, got: {:?}",
        result
            .warnings
            .iter()
            .map(|warning| warning.message.clone())
            .collect::<Vec<_>>(),
    );
}

/// Negative control for the flattening tolerance: extending a class-like symbol that DOES exist in
/// the closed world but is not a valid parent (an interface) still hard-errors. The tolerance is
/// scoped to supertypes absent everywhere, never a present-but-misused one.
#[test]
fn test_extends_known_interface_still_errors() {
    expect_error(
        "<?php interface Contract {} class Impl extends Contract {}",
        "cannot extend interface",
    );
}

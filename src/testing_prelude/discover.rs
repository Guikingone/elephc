//! Purpose:
//! Compile-time test discovery for `elephc test`: finds every concrete class whose ancestry
//! reaches `Elephc\Testing\TestCase`, lists its test methods, and appends the synthesized
//! driver (`Runner::run` per test, `exit(finish())`) to the program's top level.
//!
//! Called from:
//! - `crate::pipeline::compile_inner` for a test-suite entry, right after `autoload::run`.
//!
//! Key details:
//! - Runs AFTER name resolution and autoloading: class names are canonical, `extends` is
//!   resolved, and every base class the tests reach, including one autoloaded from `src/`
//!   or `vendor/`, is in the AST. Running any earlier would miss those parents.
//! - The driver is PHP TEXT parsed and name-resolved like a prelude, not hand-built AST: it
//!   then has exactly the node shapes the rest of the pipeline expects for `new`, a closure
//!   and `exit()`, and the one place that knows the driver's shape is the string below.
//! - Every test is called through a closure (`fn() => $case->testX()`): declaration
//!   reachability compiles only what the program reaches, and that call is what keeps the
//!   method, and its class, in the binary. The `new` expressions root the classes.
//! - A test method is public, non-static, has a body, and either starts with `test` or
//!   carries an attribute whose unqualified name is `Test` (PHPUnit's attribute, matched by
//!   name so suites need no import of `PHPUnit\Framework\Attributes\Test`). Inherited
//!   test methods run on the concrete class, own methods first then each ancestor's, an
//!   overridden name once: the order PHPUnit's reflection walk produces.
//! - `--filter` is a case-sensitive substring over `Class::method`, applied here, so a
//!   filtered-out test is never compiled.

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{ClassMethod, Program, Stmt, StmtKind, Visibility};
use crate::span::Span;

/// The canonical root every test class descends from (the PHPUnit alias is its subclass).
const TEST_CASE_ROOT: &str = "Elephc\\Testing\\TestCase";

/// One discovered test: the class as declared and the method name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredTest {
    pub class: String,
    pub method: String,
}

/// A class declaration as discovery sees it.
struct ClassInfo {
    name: String,
    parent: Option<String>,
    is_abstract: bool,
    tests: Vec<String>,
}

/// Appends the synthesized driver for every discovered test to `program`.
///
/// Fails with a diagnostic when no test class is declared or when `filter` matches nothing,
/// so an empty run is a compile error rather than a binary that reports "No tests executed".
pub fn append_driver(
    mut program: Program,
    filter: Option<&str>,
) -> Result<Program, CompileError> {
    let tests = discover(&program, filter)?;
    let source = driver_source(&tests);
    let tokens = crate::lexer::tokenize(&source)
        .map_err(|error| error.with_file("<elephc test driver>".to_string()))?;
    let driver = crate::parser::parse_internal(&tokens)
        .map_err(|error| error.with_file("<elephc test driver>".to_string()))?;
    let driver = crate::name_resolver::resolve(driver)?;
    program.extend(driver);
    Ok(program)
}

/// Lists every test in declaration order, filtered by `filter` when given.
pub fn discover(
    program: &Program,
    filter: Option<&str>,
) -> Result<Vec<DiscoveredTest>, CompileError> {
    let mut classes = Vec::new();
    collect_classes(program, &mut classes);
    let by_key: HashMap<String, usize> = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (php_symbol_key(&class.name), index))
        .collect();
    let root = php_symbol_key(TEST_CASE_ROOT);

    let mut tests = Vec::new();
    let mut declared_classes = 0usize;
    for class in &classes {
        if class.is_abstract || !descends_from_root(class, &classes, &by_key, &root) {
            continue;
        }
        declared_classes += 1;
        for method in inherited_tests(class, &classes, &by_key) {
            tests.push(DiscoveredTest {
                class: class.name.clone(),
                method,
            });
        }
    }
    if declared_classes == 0 {
        return Err(CompileError::new(
            Span::dummy(),
            "no test classes found: a test class is a concrete class extending \
             Elephc\\Testing\\TestCase or PHPUnit\\Framework\\TestCase",
        ));
    }
    if tests.is_empty() {
        return Err(CompileError::new(
            Span::dummy(),
            "no tests found: a test is a public method whose name starts with `test` or \
             that carries the #[Test] attribute",
        ));
    }
    if let Some(pattern) = filter {
        tests.retain(|test| format!("{}::{}", test.class, test.method).contains(pattern));
        if tests.is_empty() {
            return Err(CompileError::new(
                Span::dummy(),
                &format!("no tests match --filter '{pattern}'"),
            ));
        }
    }
    Ok(tests)
}

/// The test methods `class` runs: its own in declaration order, then each ancestor's, with a
/// name that appears again further up skipped (the override already ran).
fn inherited_tests(
    class: &ClassInfo,
    classes: &[ClassInfo],
    by_key: &HashMap<String, usize>,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tests = Vec::new();
    let mut current = Some(class);
    let mut visited = HashSet::new();
    while let Some(info) = current {
        if !visited.insert(php_symbol_key(&info.name)) {
            break;
        }
        for method in &info.tests {
            if seen.insert(php_symbol_key(method)) {
                tests.push(method.clone());
            }
        }
        current = info
            .parent
            .as_deref()
            .and_then(|parent| by_key.get(&php_symbol_key(parent)))
            .map(|index| &classes[*index]);
    }
    tests
}

/// Walks the parent chain of `class` through the declared classes, answering whether it
/// reaches the runtime root. A chain that leaves the declared set (an unknown parent) or
/// cycles answers `false`.
fn descends_from_root(
    class: &ClassInfo,
    classes: &[ClassInfo],
    by_key: &HashMap<String, usize>,
    root: &str,
) -> bool {
    let mut seen = HashSet::new();
    let mut current = class.parent.as_deref().map(php_symbol_key);
    while let Some(parent) = current {
        if parent == root {
            return true;
        }
        if !seen.insert(parent.clone()) {
            return false;
        }
        current = by_key
            .get(&parent)
            .and_then(|index| classes[*index].parent.as_deref())
            .map(php_symbol_key);
    }
    false
}

/// Collects class declarations in declaration order, descending into the wrappers an
/// included or conditionally declared file leaves around them.
fn collect_classes(statements: &[Stmt], classes: &mut Vec<ClassInfo>) {
    for statement in statements {
        match &statement.kind {
            StmtKind::ClassDecl {
                name,
                extends,
                is_abstract,
                methods,
                ..
            } => classes.push(ClassInfo {
                name: name.clone(),
                parent: extends.as_ref().map(|parent| parent.as_canonical()),
                is_abstract: *is_abstract,
                tests: methods
                    .iter()
                    .filter(|method| is_test_method(method))
                    .map(|method| method.name.clone())
                    .collect(),
            }),
            StmtKind::Synthetic(body)
            | StmtKind::IncludeOnceGuard { body, .. }
            | StmtKind::NamespaceBlock { body, .. } => collect_classes(body, classes),
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_classes(then_body, classes);
                for (_, body) in elseif_clauses {
                    collect_classes(body, classes);
                }
                if let Some(body) = else_body {
                    collect_classes(body, classes);
                }
            }
            _ => {}
        }
    }
}

/// A test method is public, non-static, concrete, and named `test*` or attributed `#[Test]`.
fn is_test_method(method: &ClassMethod) -> bool {
    if method.visibility != Visibility::Public
        || method.is_static
        || method.is_abstract
        || !method.has_body
    {
        return false;
    }
    method.name.starts_with("test")
        || method.attributes.iter().any(|group| {
            group.attributes.iter().any(|attribute| {
                attribute
                    .name
                    .last_segment()
                    .is_some_and(|segment| segment.eq_ignore_ascii_case("Test"))
            })
        })
}

/// Renders the driver: one runner, one instance per class, one `run` call per test, and the
/// runner's verdict as the exit code.
pub fn driver_source(tests: &[DiscoveredTest]) -> String {
    let mut source = String::from("<?php\n$__elephc_runner = new \\Elephc\\Testing\\Runner();\n");
    let mut instances: HashMap<String, usize> = HashMap::new();
    for test in tests {
        let next = instances.len();
        let index = *instances.entry(test.class.clone()).or_insert_with(|| {
            source.push_str(&format!(
                "$__elephc_case_{next} = new \\{}();\n",
                test.class
            ));
            next
        });
        source.push_str(&format!(
            "$__elephc_runner->run($__elephc_case_{index}, '{method}', fn() => $__elephc_case_{index}->{method}());\n",
            method = test.method
        ));
    }
    source.push_str("exit($__elephc_runner->finish());\n");
    source
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for test discovery: the ancestry walk, the method rules, the filter, the
    //! diagnostics for empty runs, and the rendered driver.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Fixtures are resolved the way the pipeline resolves them: the prelude is injected
    //!   and names are canonicalized before `discover` runs, so `extends` is a resolved name.

    use super::*;

    /// Parses, injects the prelude and name-resolves `source`, as the pipeline would have
    /// by the time discovery runs.
    fn resolved(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("fixture must tokenize");
        let program = crate::parser::parse(&tokens).expect("fixture must parse");
        let mut inventory = crate::optimize::reachability::PreludeInventory::new();
        let program = crate::testing_prelude::inject_if_used(program, true, &mut inventory);
        crate::name_resolver::resolve(program).expect("fixture must resolve")
    }

    /// Lists discovered tests as `Class::method` strings.
    fn names(source: &str, filter: Option<&str>) -> Vec<String> {
        discover(&resolved(source), filter)
            .expect("discovery must succeed")
            .into_iter()
            .map(|test| format!("{}::{}", test.class, test.method))
            .collect()
    }

    /// Direct and aliased subclasses are found, with their `test*` and `#[Test]` methods
    /// in declaration order; private, static and helper methods are skipped.
    #[test]
    fn finds_test_methods_on_both_roots() {
        let found = names(
            r#"<?php
namespace App\Tests;
use Elephc\Testing\TestCase;
final class AlphaTest extends TestCase {
    public function testOne(): void {}
    private function testHidden(): void {}
    public static function testStatic(): void {}
    public function helper(): void {}
    #[Test]
    public function runsWithAttribute(): void {}
    #[\PHPUnit\Framework\Attributes\Test]
    public function runsWithQualifiedAttribute(): void {}
}
final class BetaTest extends \PHPUnit\Framework\TestCase {
    public function testTwo(): void {}
}
"#,
            None,
        );
        assert_eq!(
            found,
            vec![
                "App\\Tests\\AlphaTest::testOne",
                "App\\Tests\\AlphaTest::runsWithAttribute",
                "App\\Tests\\AlphaTest::runsWithQualifiedAttribute",
                "App\\Tests\\BetaTest::testTwo",
            ]
        );
    }

    /// An intermediate abstract base is walked through but never run itself, its test
    /// methods run on the concrete class after that class's own (an override once), and a
    /// class unrelated to the runtime is ignored even when its methods are named `test*`.
    #[test]
    fn walks_intermediate_bases_and_ignores_unrelated_classes() {
        let found = names(
            r#"<?php
namespace App\Tests;
abstract class IntegrationTestCase extends \PHPUnit\Framework\TestCase {
    public function testShared(): void {}
    public function testOverridden(): void {}
}
final class DbTest extends IntegrationTestCase {
    public function testQuery(): void {}
    public function testOverridden(): void {}
}
final class NotATest {
    public function testNothing(): void {}
}
"#,
            None,
        );
        assert_eq!(
            found,
            vec![
                "App\\Tests\\DbTest::testQuery",
                "App\\Tests\\DbTest::testOverridden",
                "App\\Tests\\DbTest::testShared",
            ]
        );
    }

    /// `--filter` keeps the `Class::method` pairs containing the pattern.
    #[test]
    fn filter_is_a_substring_over_class_and_method() {
        let source = r#"<?php
use Elephc\Testing\TestCase;
final class AlphaTest extends TestCase { public function testOne(): void {} public function testTwo(): void {} }
final class BetaTest extends TestCase { public function testOne(): void {} }
"#;
        assert_eq!(
            names(source, Some("testOne")),
            vec!["AlphaTest::testOne", "BetaTest::testOne"]
        );
        assert_eq!(names(source, Some("Beta")), vec!["BetaTest::testOne"]);
        assert_eq!(names(source, Some("AlphaTest::testTwo")), vec!["AlphaTest::testTwo"]);
    }

    /// Empty runs are diagnosed with a message that names the rule that was not met.
    #[test]
    fn empty_runs_are_diagnosed() {
        let no_classes = discover(&resolved("<?php class Plain {}"), None).unwrap_err();
        assert!(no_classes.message.contains("no test classes found"), "{}", no_classes.message);
        let no_methods = discover(
            &resolved("<?php final class T extends \\Elephc\\Testing\\TestCase { public function helper(): void {} }"),
            None,
        )
        .unwrap_err();
        assert!(no_methods.message.contains("no tests found"), "{}", no_methods.message);
        let no_match = discover(
            &resolved("<?php final class T extends \\Elephc\\Testing\\TestCase { public function testIt(): void {} }"),
            Some("nope"),
        )
        .unwrap_err();
        assert!(no_match.message.contains("no tests match --filter 'nope'"), "{}", no_match.message);
    }

    /// The driver instantiates each class once, calls every test through a closure and
    /// exits with the runner's verdict.
    #[test]
    fn driver_source_has_the_documented_shape() {
        let tests = vec![
            DiscoveredTest { class: "App\\Tests\\AlphaTest".into(), method: "testOne".into() },
            DiscoveredTest { class: "App\\Tests\\AlphaTest".into(), method: "testTwo".into() },
            DiscoveredTest { class: "BetaTest".into(), method: "testThree".into() },
        ];
        assert_eq!(
            driver_source(&tests),
            "<?php\n\
             $__elephc_runner = new \\Elephc\\Testing\\Runner();\n\
             $__elephc_case_0 = new \\App\\Tests\\AlphaTest();\n\
             $__elephc_runner->run($__elephc_case_0, 'testOne', fn() => $__elephc_case_0->testOne());\n\
             $__elephc_runner->run($__elephc_case_0, 'testTwo', fn() => $__elephc_case_0->testTwo());\n\
             $__elephc_case_1 = new \\BetaTest();\n\
             $__elephc_runner->run($__elephc_case_1, 'testThree', fn() => $__elephc_case_1->testThree());\n\
             exit($__elephc_runner->finish());\n"
        );
    }

    /// The appended driver parses, resolves, and lands after the user's declarations.
    #[test]
    fn append_driver_extends_the_program() {
        let program = resolved(
            "<?php use Elephc\\Testing\\TestCase; final class T extends TestCase { public function testIt(): void {} }",
        );
        let before = program.len();
        let appended = append_driver(program, None).expect("driver must append");
        // One runner assignment, one instance, one run call, one exit.
        assert_eq!(appended.len(), before + 4);
    }
}

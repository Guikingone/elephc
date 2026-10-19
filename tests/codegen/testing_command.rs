//! Purpose:
//! End-to-end coverage for the `elephc test` subcommand through the real binary: test-file
//! discovery under a project, Composer `autoload` and `autoload-dev` resolution from the
//! project root, compile-time test discovery (`test*`, `#[Test]`, inherited methods, the
//! PHPUnit alias, mixed `.php` and `.lfc` files), `--filter`, the exit code, and the
//! diagnostics for empty runs and refused flags.
//!
//! Called from:
//! - `cargo test --test codegen_tests testing_command` through Rust's test harness.
//!
//! Key details:
//! - Every test builds its own project directory and runs the CLI from inside it, with the
//!   cache redirected under that directory by `elephc_cli_command`, so builds never collide.
//! - The report lines are asserted verbatim: they are the surface a CI log shows.

use crate::support::*;
use std::path::Path;

/// Writes the fixture project: a `src/` class, a plain-PHP PHPUnit-style test, a tagless
/// `.lfc` test, an abstract base with an inherited test, and a nested test directory.
fn write_project(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests/Unit")).unwrap();
    fs::write(
        dir.join("composer.json"),
        r#"{
    "autoload": { "psr-4": { "App\\": "src/" } },
    "autoload-dev": { "psr-4": { "App\\Tests\\": "tests/" } }
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("src/Counter.php"),
        r#"<?php
namespace App;
final class Counter {
    private int $value = 0;
    public function increment(int $by = 1): void { $this->value += $by; }
    public function value(): int { return $this->value; }
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("tests/IntegrationTestCase.php"),
        r#"<?php
namespace App\Tests;
use PHPUnit\Framework\TestCase;
abstract class IntegrationTestCase extends TestCase {
    public function testInheritedCheck(): void { $this->assertTrue(true); }
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("tests/CounterTest.php"),
        r#"<?php
namespace App\Tests;
use App\Counter;
final class CounterTest extends IntegrationTestCase {
    private Counter $counter;
    protected function setUp(): void { $this->counter = new Counter(); }
    public function testStartsAtZero(): void { $this->assertSame(0, $this->counter->value()); }
    public function testIncrements(): void {
        $this->counter->increment(2);
        $this->assertSame(2, $this->counter->value());
        $this->assertGreaterThan(1, $this->counter->value());
    }
    public function helper(): void { $this->fail('never run'); }
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("tests/Unit/StringsTest.lfc"),
        r#"namespace App\Tests\Unit;
use Elephc\Testing\TestCase;
final class StringsTest extends TestCase {
    #[Test]
    public function typedAssertions(): void {
        $this->assertSameInt(6, strlen('elephc'));
        $this->assertSameString('a', 'a');
    }
    public function testFails(): void { $this->assertSame(1, 2, 'deliberate'); }
}
"#,
    )
    .unwrap();
    // Not a test file: a helper the tests could autoload, never required by the entry.
    fs::write(
        dir.join("tests/Unit/Support.php"),
        "<?php\nnamespace App\\Tests\\Unit;\nfinal class Support { public function testNotATest(): void {} }\n",
    )
    .unwrap();
}

/// Runs `elephc test` with `args` inside `dir`, returning stdout, stderr and the exit code.
fn run_test_command(dir: &Path, args: &[&str]) -> (String, String, i32) {
    let output = elephc_cli_command(dir)
        .arg("test")
        .args(args)
        .output()
        .expect("failed to run elephc test");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

/// The whole project: files found in sorted order, `test*`, `#[Test]` and inherited methods
/// discovered, autoload from `src/` and `autoload-dev` resolved, the deliberate failure
/// reported, and the exit code non-zero because of it.
#[test]
fn test_cli_test_runs_the_discovered_suite() {
    let dir = make_cli_test_dir("elephc_cli_test_suite");
    write_project(&dir);
    let (stdout, stderr, code) = run_test_command(&dir, &[]);
    assert_eq!(
        stdout,
        "ok   App\\Tests\\CounterTest::testStartsAtZero (1 assertion)\n\
         ok   App\\Tests\\CounterTest::testIncrements (2 assertions)\n\
         ok   App\\Tests\\CounterTest::testInheritedCheck (1 assertion)\n\
         ok   App\\Tests\\Unit\\StringsTest::typedAssertions (2 assertions)\n\
         FAIL App\\Tests\\Unit\\StringsTest::testFails\n\
         \x20    deliberate\n\
         \x20    Failed asserting that 2 is identical to 1.\n\
         \n\
         Tests: 5, Assertions: 7, Failures: 1, Errors: 0.\n",
        "stderr: {stderr}"
    );
    assert_eq!(code, 1, "a failing suite exits 1\nstderr: {stderr}");
    assert!(
        !stderr.contains("Compiled '"),
        "the test build must not print the compile success line: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// `--filter` keeps only the matching `Class::method` pairs, so a green subset exits 0,
/// and the filter is applied before compilation (the failing test never runs).
#[test]
fn test_cli_test_filter_selects_tests_and_exits_zero_when_green() {
    let dir = make_cli_test_dir("elephc_cli_test_filter");
    write_project(&dir);
    let (stdout, stderr, code) = run_test_command(&dir, &["--filter", "CounterTest"]);
    assert_eq!(
        stdout,
        "ok   App\\Tests\\CounterTest::testStartsAtZero (1 assertion)\n\
         ok   App\\Tests\\CounterTest::testIncrements (2 assertions)\n\
         ok   App\\Tests\\CounterTest::testInheritedCheck (1 assertion)\n\
         \n\
         OK (3 tests, 4 assertions)\n",
        "stderr: {stderr}"
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let (stdout, _, code) = run_test_command(&dir, &["--filter=typedAssertions"]);
    assert!(stdout.ends_with("OK (1 test, 2 assertions)\n"), "{stdout}");
    assert_eq!(code, 0);
    let _ = fs::remove_dir_all(&dir);
}

/// A path argument narrows the files: a subdirectory, or a single file, while the project
/// root (and so autoload) is still found from the nearest `composer.json` above it.
#[test]
fn test_cli_test_accepts_a_directory_or_a_file_path() {
    let dir = make_cli_test_dir("elephc_cli_test_path");
    write_project(&dir);
    let (stdout, _, code) = run_test_command(&dir, &["tests/Unit", "--filter", "typed"]);
    assert!(stdout.starts_with("ok   App\\Tests\\Unit\\StringsTest::typedAssertions"), "{stdout}");
    assert_eq!(code, 0);
    let (stdout, stderr, code) = run_test_command(&dir, &["tests/CounterTest.php"]);
    assert!(stdout.ends_with("OK (3 tests, 4 assertions)\n"), "{stdout}\n{stderr}");
    assert_eq!(code, 0);
    let _ = fs::remove_dir_all(&dir);
}

/// Running from a subdirectory of the project still resolves the root through the test
/// path's ancestors, not the working directory.
#[test]
fn test_cli_test_resolves_the_root_from_the_test_path() {
    let dir = make_cli_test_dir("elephc_cli_test_root");
    write_project(&dir);
    let output = elephc_cli_command(&dir.join("tests/Unit"))
        .args(["test", "../../tests", "--filter", "testStartsAtZero"])
        .output()
        .expect("failed to run elephc test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.ends_with("OK (1 test, 1 assertion)\n"), "{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
    let _ = fs::remove_dir_all(&dir);
}

/// Empty runs are errors with a message naming the rule: no test files, no test classes,
/// no matching filter, a missing path.
#[test]
fn test_cli_test_diagnoses_empty_runs() {
    let dir = make_cli_test_dir("elephc_cli_test_empty");
    fs::create_dir_all(dir.join("tests")).unwrap();
    let (_, stderr, code) = run_test_command(&dir, &[]);
    assert!(stderr.contains("no test files found under 'tests'"), "{stderr}");
    assert_eq!(code, 1);
    let (_, stderr, code) = run_test_command(&dir, &["missing"]);
    assert!(stderr.contains("no such path 'missing'"), "{stderr}");
    assert_eq!(code, 1);

    fs::write(dir.join("tests/PlainTest.php"), "<?php\nclass PlainTest { public function testNothing(): void {} }\n").unwrap();
    let (_, stderr, code) = run_test_command(&dir, &[]);
    assert!(stderr.contains("no test classes found"), "{stderr}");
    assert_eq!(code, 1);

    write_project(&dir);
    let (_, stderr, code) = run_test_command(&dir, &["--filter", "nothing-matches"]);
    assert!(stderr.contains("no tests match --filter 'nothing-matches'"), "{stderr}");
    assert_eq!(code, 1);
    let _ = fs::remove_dir_all(&dir);
}

/// Output-mode and target flags are refused with the usage text; `--help` prints it and
/// exits 0; ordinary compile options such as `--strict-php` are forwarded.
#[test]
fn test_cli_test_refuses_output_modes_and_forwards_compile_options() {
    let dir = make_cli_test_dir("elephc_cli_test_flags");
    write_project(&dir);
    let (_, stderr, code) = run_test_command(&dir, &["--emit-ir"]);
    assert!(stderr.contains("--emit-ir is not accepted by `elephc test`"), "{stderr}");
    assert!(stderr.contains("Usage: elephc test"), "{stderr}");
    assert_eq!(code, 1);

    let (stdout, _, code) = run_test_command(&dir, &["--help"]);
    assert!(stdout.starts_with("Usage: elephc test [PATH]"), "{stdout}");
    assert_eq!(code, 0);

    // `--strict-php` reaches the compiler: the `.lfc` file keeps its extensions, the
    // tagged `.php` files are audited, and the suite still runs.
    let (stdout, stderr, code) = run_test_command(&dir, &["--strict-php", "--filter", "CounterTest"]);
    assert!(stdout.ends_with("OK (3 tests, 4 assertions)\n"), "{stdout}\n{stderr}");
    assert_eq!(code, 0);
    let _ = fs::remove_dir_all(&dir);
}

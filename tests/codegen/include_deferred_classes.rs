//! Purpose:
//! End-to-end value tests for NAMING, in compiled code, a class that only a runtime include or a
//! registered autoloader can provide. Symfony's container writes `new $class` and
//! `new ReflectionClass($id)` against classes the Composer autoloader supplies, and the checker
//! used to REFUSE those programs at type-check time where `php -n` exits 0.
//!
//! Called from:
//! - `cargo test --test codegen_tests include_deferred_classes` through Rust's test harness.
//!
//! Key details:
//! - The include path is DELIBERATELY not a string literal; see `include_reflection` for why a
//!   literal path makes such a fixture prove nothing.
//! - The refusal was in the CHECKER, not in lowering. `ir_lower::stmt::includes::lower_include`
//!   has always called `ctx.apply_eval_barrier()`, so `object_construction` already lowers a
//!   `new` of an unknown class to `Op::EvalObjectNew`, which resolves the name at run time through
//!   the bridge. What was missing is that `Checker::allows_absent_runtime_class` only answered
//!   `true` inside a function body or after a literal `eval()`; at top level, after a runtime
//!   include, it still reported `Undefined class`.
//! - The NEGATIVE control is the point of the third test. Deferring must not turn a genuinely
//!   missing class into a silent success: PHP raises a catchable `Error` reading
//!   `Class "X" not found`, and that is what the binary must print too.
//! - Every expected string is `php -n` 8.5.6's output on the same files.

use crate::support::*;

/// Verifies the caller can reflect on a class a runtime include declared.
///
/// This is the twin of `include_reflection::test_caller_reflects_on_a_class_a_runtime_include_declared`
/// with the reflected name reaching a variable, and it fails the same way: the reflection
/// constructor validator reported `ReflectionClass::__construct(): undefined class 'S1Plain'`
/// where `php -n` 8.5.6 prints `loaded;S1Plain;done`.
#[test]
fn test_reflection_constructor_defers_a_class_only_the_include_declares() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\n$name = \"piece\" . \".php\";\ninclude $name;\n$r = new ReflectionClass(\"S1Plain\");\necho $r->getName();\necho \";done\";\n",
            ),
            (
                "piece.php",
                "<?php\nclass S1Plain { public int $id = 4; }\necho \"loaded;\";\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "loaded;S1Plain;done");
}

/// Verifies compiled code can construct a class only a runtime include declares.
///
/// `php -n` 8.5.6 prints `loaded;7;done`. Before this change the checker refused with
/// `Undefined class: S1Plain` at the `new`.
#[test]
fn test_new_defers_a_class_only_the_include_declares() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\n$name = \"piece\" . \".php\";\ninclude $name;\n$o = new S1Plain();\necho $o->v;\necho \";done\";\n",
            ),
            (
                "piece.php",
                "<?php\nclass S1Plain { public int $v = 7; }\necho \"loaded;\";\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "loaded;7;done");
}

/// Verifies a class that exists NOWHERE still produces PHP's runtime Error, not a silent success.
///
/// This is the control that keeps the deferral honest. `php -n` 8.5.6 prints
/// `loaded;Error: Class "NoSuchClassAnywhere" not found;done` and exits 0, because the Error is
/// catchable. A deferral that answered "constructed" here would be worse than the refusal it
/// replaces.
#[test]
fn test_a_class_that_exists_nowhere_still_raises_phps_runtime_error() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\n$name = \"piece\" . \".php\";\ninclude $name;\ntry {\n    $x = new NoSuchClassAnywhere();\n    echo \"constructed\";\n} catch (\\Throwable $e) {\n    echo get_class($e), \": \", $e->getMessage();\n}\necho \";done\";\n",
            ),
            (
                "piece.php",
                "<?php\nclass S1Plain { public int $v = 7; }\necho \"loaded;\";\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(
        out,
        "loaded;Error: Class \"NoSuchClassAnywhere\" not found;done"
    );
}

// THE AUTOLOADER ROUTE HAS NO FIXTURE HERE, and the reason is worth writing down rather than
// leaving as an absence.
//
// The checker half works: `spl_autoload_register(...)` followed by `new MadeByLoader()` used to
// be refused with `Undefined class: MadeByLoader` at type-check, and now compiles. Measured on
// the real pipeline with the release compiler.
//
// The route still does not produce PHP's value, for a reason that belongs to the autoload
// registry rather than to this deferral. `autoload::Registry::build` CONSUMES a collectable
// `spl_autoload_register` call and strips it from the program, turning it into a compile-time
// rule. A loader whose body is `eval("class …")` cannot be reduced to such a rule, and it is no
// longer registered at run time either, so the interpreter's callback chain is empty —
// `ELEPHC_EVAL_TRACE=1` reports `phase=spl_autoload_local class="MadeByLoader" callbacks=0` — and
// the binary prints `Fatal error: Uncaught Error: Class "MadeByLoader" not found` where
// `php -n` 8.5.6 prints `loaded;done`.
//
// It is not pinned here because the helper that runs a program expected to FAIL builds no
// autoload registry, so it cannot model the pipeline for this shape at all; a fixture written
// against it would prove something about the harness instead of about the compiler. Closing the
// divergence means either keeping consumed registrations live at run time for whatever the
// compiled rules cannot resolve, or only consuming registrations provably reducible to
// classmap/PSR-4 — and the fixture belongs with that change, next to a harness that models it.

/// Verifies `include_once` of an AUTOLOADED file answers "already included" instead of re-running it.
///
/// The autoload pass performs, at compile time, the inclusion PHP's autoloader performs at run
/// time: it opens the class file and splices its declarations into the program. The runtime
/// inclusion guard for that file stayed clear all the same, so an `include_once` reaching it
/// through a COMPUTED path re-ran the file and died redeclaring a class the binary already
/// carries. Symfony's compiled container does exactly this
/// (`include_once dirname(__DIR__, 4).'/vendor/.../ParameterBagInterface.php'`), and the whole
/// request died on it.
///
/// `php -n` 8.5.6 on the Composer-shaped equivalent (`scratchpad/preinc/ref.php`, this session,
/// a `spl_autoload_register` that `require`s the same file) prints
/// `first=thing` / `again=true` / `second=thing`.
#[test]
fn test_include_once_of_an_autoloaded_file_reports_it_as_already_included() {
    let out = compile_and_run_files(
        &[
            (
                "project.json",
                r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
            ),
            (
                "src/Thing.php",
                "<?php\nnamespace App;\ninterface ThingContract { public function tag(): string; }\n\
                 class Thing implements ThingContract { public function tag(): string { return 'thing'; } }\n",
            ),
            (
                "main.php",
                "<?php\n$thing = new \\App\\Thing();\necho 'first=', $thing->tag(), \"\\n\";\n\
                 $path = __DIR__ . '/src/' . 'Thing' . '.php';\n$again = include_once $path;\n\
                 echo 'again=', var_export($again, true), \"\\n\";\n\
                 echo 'second=', (new \\App\\Thing())->tag(), \"\\n\";\n",
            ),
        ],
        "main.php",
    );

    assert_eq!(out, "first=thing\nagain=true\nsecond=thing\n");
}

/// Verifies `class_exists($name, false)` reports a probe-only autoloaded class as NOT LOADED.
///
/// A closed-world build declares everything it compiled from the first instruction, so a class
/// the autoload pass pulled in solely to answer `class_exists(X::class)` answered "loaded" where
/// php answers "not loaded". `symfony/runtime` reads exactly that difference to decide whether
/// `symfony/dotenv` is installed:
///
/// ```php
/// class_exists(MissingDotenv::class, false) || class_exists(Dotenv::class) || class_exists(MissingDotenv::class);
/// // ...
/// if (... && !class_exists(MissingDotenv::class, false)) { $dotenv->bootEnv(...); }
/// ```
///
/// php never loads `MissingDotenv` when dotenv IS installed, so `.env` is read. elephc answered
/// `true`, skipped `bootEnv()`, and every `%env(...)%` parameter then failed to resolve --
/// `EnvNotFoundException: Environment variable not found: "DEFAULT_URI"` on the first request.
///
/// A class the program actually USES stays loaded: php would have loaded it too.
///
/// `php -n` 8.5.6 on the Composer-shaped equivalent (`scratchpad/probeonly/ref.php`, this
/// session) prints `before=false` / `probe=true` / `after=true` / `used=optional/true`.
#[test]
fn test_class_exists_without_autoload_reports_a_probe_only_class_as_unloaded() {
    let out = compile_and_run_files(
        &[
            ("project.json", r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#),
            ("src/Fallback.php", "<?php\nnamespace App;\nclass Fallback {}\n"),
            (
                "src/Optional.php",
                "<?php\nnamespace App;\nclass Optional { public function tag(): string { return 'optional'; } }\n",
            ),
            (
                "main.php",
                "<?php\n\
                 echo 'before=', (class_exists(\\App\\Fallback::class, false) ? 'true' : 'false'), \"\\n\";\n\
                 echo 'probe=', (class_exists(\\App\\Fallback::class) ? 'true' : 'false'), \"\\n\";\n\
                 echo 'after=', (class_exists(\\App\\Fallback::class, false) ? 'true' : 'false'), \"\\n\";\n\
                 $optional = new \\App\\Optional();\n\
                 echo 'used=', $optional->tag(), '/', (class_exists(\\App\\Optional::class, false) ? 'true' : 'false'), \"\\n\";\n",
            ),
        ],
        "main.php",
    );

    assert_eq!(
        out,
        "before=false\nprobe=true\nafter=true\nused=optional/true\n"
    );
}

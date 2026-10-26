//! Purpose:
//! End-to-end tests for compiling a refused construct as php's RUN-TIME fault instead of
//! handing its whole class file to the runtime autoloader (`autoload::runtime_faults`).
//!
//! Called from:
//! - `cargo test --test codegen_tests autoload_runtime_faults` through Rust's test harness.
//!
//! Key details:
//! - php binds a member or a class name when the expression executes: a method nothing declares,
//!   called on a branch the program never takes, is a loadable program. The checker refuses the
//!   body, and the file used to go to the interpreter for it (doctrine/orm's `WhenClause.php`
//!   calls `$walker->walkWhenClauseExpression()`, which no `SqlWalker` declares).
//! - Every fixture registers an autoloader, so its class files are deferrable, and asserts on the
//!   compiler's stderr: the `note:` that names the demotion must be there and the deferral
//!   warning must NOT.
//! - Every expected string is `php -n` 8.5's output on the same files, except where the test
//!   says why the compiled output differs (the location a helper frame reports).

use crate::support::*;

/// A Composer-like loader: it checks the file exists, so a class nothing declares is php's
/// `Class "X" not found` rather than a failed `require`.
const AUTOLOADER: &str = "<?php\nspl_autoload_register(function (string $class): void {\n    $file = __DIR__ . \"/src/\" . $class . \".php\";\n    if (is_file($file)) { require $file; }\n});\n";

const WALKER: &str = "<?php\nclass Walker {\n    public int $v = 3;\n    public function walkOk(Clause $c): string { return \"walked\"; }\n}\n";

/// Undefined instance methods (plain and nullsafe), a static method, a static property and a
/// class constant, each on a class the compiled world holds in full, raise php's `Error` only
/// when their line runs, and the file stays compiled.
#[test]
fn test_members_nothing_declares_raise_phps_error_when_their_line_runs() {
    let main = format!(
        "{AUTOLOADER}$walker = new Walker();\n$clause = new Clause();\necho $clause->ok($walker), \"\\n\";\necho $clause->constantGuarded(), \"\\n\";\nforeach ([\"dispatch\", \"dispatchNullsafe\", \"staticMissing\", \"staticProp\", \"constantMissing\"] as $m) {{\n    try {{\n        if ($argc > 5) {{ echo \"never\\n\"; }}\n        var_dump($clause->$m($argc > 5 ? null : $walker));\n    }} catch (\\Error $e) {{\n        echo get_class($e), \": \", $e->getMessage(), \"\\n\";\n    }}\n}}\nvar_dump($clause->dispatchNullsafe(null));\necho \"done\\n\";\n"
    );
    let clause = "<?php\nclass Clause {\n    public function ok(Walker $w): string { return $w->walkOk($this); }\n    public function dispatch(?Walker $w): string { return $w->walkWhenClauseExpression($this); }\n    public function dispatchNullsafe(?Walker $w): ?string { return $w?->walkWhenClauseExpression($this); }\n    public function staticMissing(?Walker $w): string { $x = Walker::nothing(1, 2); return \"x\" . $x; }\n    public function staticProp(?Walker $w): int { return Walker::$count + 1; }\n    public function constantMissing(?Walker $w): string { return strtoupper(Walker::MISSING); }\n    public function constantGuarded(): string { return defined(\"Walker::MISSING\") ? Walker::MISSING : \"guarded\"; }\n}\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[("main.php", &main), ("src/Walker.php", WALKER), ("src/Clause.php", clause)],
        "main.php",
    );
    assert_eq!(
        out,
        "walked\nguarded\nError: Call to undefined method Walker::walkWhenClauseExpression()\nError: Call to undefined method Walker::walkWhenClauseExpression()\nError: Call to undefined method Walker::nothing()\nError: Access to undeclared static property Walker::$count\nError: Undefined constant Walker::MISSING\nNULL\ndone\n"
    );
    assert!(
        !stderr.contains("is not compiled"),
        "Clause.php was handed to the autoloader:\n{stderr}"
    );
    for expected in [
        "the call to `->walkWhenClauseExpression()` is resolved by name when it runs",
        "the call to `?->walkWhenClauseExpression()` is resolved by name when it runs",
        "php's Error `Call to undefined method Walker::nothing()` is thrown when it runs",
        "php's Error `Access to undeclared static property Walker::$count` is thrown when it runs",
        "php's Error `Undefined constant Walker::MISSING` is thrown when it runs",
    ] {
        assert!(
            stderr.contains(expected),
            "missing note `{expected}` in:\n{stderr}"
        );
    }
    assert_eq!(
        stderr.matches("note: autoload: ").count(),
        6,
        "one note per demoted construct (the guarded constant read counts too):\n{stderr}"
    );
}

/// A refused construct inside an `if` condition stops the checker before it installs what the
/// condition's `||` assigns, so the read after the `if` is "undefined" too. That second error is
/// a cascade of the first: once the call is demoted the file checks clean, and it must not be
/// deferred for an error the rewrite already removed (symfony/framework-bundle's
/// `DebugAutowiringCommand::getFileLink()` is the shape).
#[test]
fn test_an_error_cascading_from_a_demoted_condition_does_not_defer_the_file() {
    let main = format!(
        "{AUTOLOADER}$linker = new Linker();\necho \"[\", $linker->link(null), \"]\\n\";\ntry {{ echo $linker->link(new Walker()), \"\\n\"; }} catch (\\Error $e) {{ echo get_class($e), \": \", $e->getMessage(), \"\\n\"; }}\necho \"done\\n\";\n"
    );
    let walker = "<?php\nclass Walker {\n    public int $v = 3;\n}\n";
    let linker = "<?php\nclass Linker {\n    public function link(?Walker $w): string {\n        if (null === $w || (null === $r = $w->findReflection(\"x\"))) {\n            return \"\";\n        }\n        return $r . \"!\";\n    }\n}\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[("main.php", &main), ("src/Walker.php", walker), ("src/Linker.php", linker)],
        "main.php",
    );
    assert_eq!(
        out,
        "[]\nError: Call to undefined method Walker::findReflection()\ndone\n"
    );
    assert!(
        !stderr.contains("is not compiled"),
        "Linker.php was handed to the autoloader:\n{stderr}"
    );
    assert!(
        stderr.contains("the call to `->findReflection()` is resolved by name when it runs"),
        "{stderr}"
    );
}

/// A property the receiver's class does not declare reads null with php's Warning.
///
/// php prints `Warning: Undefined property: Walker::$missing in .../Clause.php on line 3` on
/// stdout; the compiled program writes its warnings to stderr (every compiled warning does), and
/// names the helper's frame rather than `Clause.php:3`, so the Warning is checked on stderr by
/// message only. The values are php's.
#[test]
fn test_a_property_nothing_declares_warns_and_reads_null() {
    let main = format!(
        "{AUTOLOADER}$clause = new Clause();\nvar_dump($clause->readMissing(new Walker()));\nvar_dump($clause->readMissingNullsafe(null));\necho \"done\\n\";\n"
    );
    let clause = "<?php\nclass Clause {\n    public function readMissing(Walker $w): mixed { return $w->missing; }\n    public function readMissingNullsafe(?Walker $w): mixed { return $w?->missing; }\n}\n";
    let (stderr, out, run_stderr) = compile_cli_files_and_run_with_compile_and_run_stderr(
        &[("main.php", &main), ("src/Walker.php", WALKER), ("src/Clause.php", clause)],
        "main.php",
    );
    assert_eq!(out, "NULL\nNULL\ndone\n");
    assert!(
        run_stderr.contains("Warning: Undefined property: Walker::$missing in "),
        "php's Warning is missing:\n{run_stderr}"
    );
    assert!(!stderr.contains("is not compiled"), "{stderr}");
    assert!(
        stderr.contains("the read of `->missing` is resolved by name when it runs")
            && stderr.contains("the read of `?->missing` is resolved by name when it runs"),
        "{stderr}"
    );
}

/// The member is resolved against the RUN-TIME object: a subclass the compiler never saw (here
/// declared by `eval()`, so only the interpreter has it) that declares the method or property
/// is dispatched to, exactly as php does, while the base class still gets php's Error. This is
/// what rules out compiling the call as an unconditional `throw`.
#[test]
fn test_a_runtime_subclass_declaring_the_member_is_dispatched_to() {
    let main = format!(
        "{AUTOLOADER}eval(\"class LateWalker extends Walker {{ public \\$extra = \\\"late\\\"; public function walkWhenClauseExpression(Clause \\$c): string {{ return \\\"late-walked\\\"; }} }}\");\n$clause = new Clause();\necho $clause->dispatch(new LateWalker()), \"\\n\";\nvar_dump($clause->readMissing(new LateWalker()));\ntry {{ echo $clause->dispatch(new Walker()), \"\\n\"; }} catch (\\Error $e) {{ echo get_class($e), \": \", $e->getMessage(), \"\\n\"; }}\necho \"done\\n\";\n"
    );
    let walker = "<?php\nclass Walker {\n    public int $v = 3;\n}\n";
    let clause = "<?php\nclass Clause {\n    public function dispatch(Walker $w): string { return $w->walkWhenClauseExpression($this); }\n    public function readMissing(Walker $w): mixed { return $w->extra; }\n}\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[("main.php", &main), ("src/Walker.php", walker), ("src/Clause.php", clause)],
        "main.php",
    );
    assert_eq!(
        out,
        "late-walked\nstring(4) \"late\"\nError: Call to undefined method Walker::walkWhenClauseExpression()\ndone\n"
    );
    assert!(!stderr.contains("is not compiled"), "{stderr}");
    assert_eq!(stderr.matches("note: autoload: ").count(), 2, "{stderr}");
}

/// A property read through a union of classes where one member lacks it (Doctrine's
/// `SchemaTool::gatherColumnOptions()`: `isset($mapping->enumType)` over
/// `JoinColumnMapping|FieldMapping|...`) is resolved by name when it runs: the declaring member
/// answers, the other raises php's Warning and reads null. The file stays compiled.
/// Value-checked against `php -n` (stdout; the Warning goes to stderr in a compiled program).
#[test]
fn test_a_property_some_union_members_lack_is_read_by_name() {
    let main = format!(
        "{AUTOLOADER}$s = new Schema();\necho json_encode($s->options(new FieldMapping())), json_encode($s->options(new JoinColumnMapping())), \"\\n\";\nvar_dump($s->raw(new JoinColumnMapping()));\n"
    );
    let field = "<?php\nfinal class FieldMapping { public ?string $enumType = \"Suit\"; public array $options = [\"a\" => 1]; }\n";
    let join = "<?php\nfinal class JoinColumnMapping { public array $options = [\"b\" => 2]; }\n";
    let schema = "<?php\nfinal class Schema {\n    public function options(JoinColumnMapping|FieldMapping $mapping): array {\n        $o = $mapping->options ?? [];\n        if (isset($mapping->enumType)) { $o[\"enumType\"] = $mapping->enumType; }\n        return $o;\n    }\n    public function raw(JoinColumnMapping|FieldMapping $mapping): mixed { return $mapping->enumType; }\n}\n";
    let (stderr, out, run_stderr) = compile_cli_files_and_run_with_compile_and_run_stderr(
        &[
            ("main.php", &main),
            ("src/FieldMapping.php", field),
            ("src/JoinColumnMapping.php", join),
            ("src/Schema.php", schema),
        ],
        "main.php",
    );
    assert_eq!(out, "{\"a\":1,\"enumType\":\"Suit\"}{\"b\":2}\nNULL\n");
    assert!(run_stderr.contains("Undefined property: JoinColumnMapping::$enumType"), "{run_stderr}");
    assert!(!stderr.contains("is not compiled"), "{stderr}");
}

/// A constant of a PDO driver the build leaves out (`PDO::SQLSRV_ENCODING_BINARY` in Doctrine
/// DBAL's `PDO\SQLSrv\Statement`) is php's "Undefined constant" Error when the line runs, as on a
/// php without `pdo_sqlsrv`; the file stays compiled. Value-checked against `php`.
#[test]
fn test_a_disabled_pdo_driver_constant_raises_phps_error_when_read() {
    let main = format!(
        "{AUTOLOADER}$s = new SqlsrvStatement();\necho $s->name(), \"\\n\";\ntry {{ echo $s->encoding(), \"\\n\"; }} catch (\\Error $e) {{ echo get_class($e), \": \", $e->getMessage(), \"\\n\"; }}\n"
    );
    let statement = "<?php\nclass SqlsrvStatement {\n    public function name(): string { return 'sqlsrv'; }\n    public function encoding(): int { return \\PDO::SQLSRV_ENCODING_BINARY; }\n}\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[("main.php", &main), ("src/SqlsrvStatement.php", statement)],
        "main.php",
    );
    assert_eq!(out, "sqlsrv\nError: Undefined constant PDO::SQLSRV_ENCODING_BINARY\n");
    assert!(!stderr.contains("is not compiled"), "{stderr}");
}

/// A `static::` constant is bound to the called class when the line runs, so no verdict against
/// the lexical class is php's: the read becomes the run-time lookup
/// `constant(static::class . '::TOKEN')`, which answers the subclass's constant and raises php's
/// own Error for a class without one. The file stays compiled (Symfony's `FirewallAwareTrait`
/// reads `static::FIREWALL_OPTION` behind `defined(static::class.'::FIREWALL_OPTION')`).
/// Value-checked against `php -n`.
#[test]
fn test_a_late_bound_static_constant_is_looked_up_on_the_called_class() {
    let main = format!(
        "{AUTOLOADER}echo (new Child())->read(), \"\\n\";\ntry {{ echo (new Base())->read(), \"\\n\"; }} catch (\\Error $e) {{ echo get_class($e), \": \", $e->getMessage(), \"\\n\"; }}\necho (new Base())->guarded(), (new Child())->guarded(), \"\\n\";\n"
    );
    let base = "<?php\nclass Base {\n    public function read(): string { return static::TOKEN; }\n    public function guarded(): string { return defined(static::class . '::TOKEN') ? static::TOKEN : 'none'; }\n}\n";
    let child = "<?php\nclass Child extends Base {\n    public const TOKEN = \"child\";\n}\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[("main.php", &main), ("src/Base.php", base), ("src/Child.php", child)],
        "main.php",
    );
    assert_eq!(out, "child\nError: Undefined constant Base::TOKEN\nnonechild\n");
    assert!(!stderr.contains("is not compiled"), "{stderr}");
    assert!(
        stderr.contains("`static::TOKEN` is looked up on the called class when it runs"),
        "{stderr}"
    );
}

/// `isset(self::$emptyDefaults)` in a trait used by a class that declares no such static
/// (Symfony's `HttpClientTrait` in `CachingHttpClient`) is php's `false`, and the guarded read
/// is php's Error only when it runs. Both files stay compiled. Value-checked against `php -n`.
#[test]
fn test_isset_of_a_static_the_using_class_lacks_is_false() {
    let main = format!(
        "{AUTOLOADER}echo json_encode((new WithDefaults())->opts()), json_encode((new Without())->opts()), \"\\n\";\n"
    );
    let defaults = "<?php\ntrait Defaults {\n    public function opts(): array {\n        $options = ['a' => 1];\n        if (isset(self::$emptyDefaults)) {\n            foreach (self::$emptyDefaults as $k => $v) { $options[$k] ??= $v; }\n        }\n        return $options;\n    }\n}\n";
    let with = "<?php\nclass WithDefaults { use Defaults; private static array $emptyDefaults = ['b' => 2]; }\n";
    let without = "<?php\nclass Without { use Defaults; }\n";
    let (stderr, out) = compile_cli_files_and_run_with_compile_stderr(
        &[
            ("main.php", &main),
            ("src/Defaults.php", defaults),
            ("src/WithDefaults.php", with),
            ("src/Without.php", without),
        ],
        "main.php",
    );
    assert_eq!(out, "{\"a\":1,\"b\":2}{\"a\":1}\n");
    assert!(!stderr.contains("is not compiled"), "{stderr}");
}

//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of include paths, including include with dunder dir concat, include with const ref, and include with define ref.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Multi-file fixtures exercise include/require resolution, temporary project layout, and native binary output.

use crate::support::*;

/// Verifies a variable include path executes the selected file in the caller's scope.
#[test]
fn test_runtime_dynamic_include_variable_path_shares_scope() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { $prefix = 'runtime'; include $path; } load('piece.php');",
            ),
            ("piece.php", "<?php echo $prefix . '-include';"),
        ],
        "main.php",
    );
    assert_eq!(out, "runtime-include");
}

/// Verifies a by-reference nested `foreach` write reaches the outer array when the whole
/// fragment executes through the eval bridge (a runtime-only-known include path forces the
/// interpreter to run the included file, rather than the AOT compiler lowering it directly).
///
/// This is the shape `examples/symfony-app/vendor/symfony/event-dispatcher/EventDispatcher.php`'s
/// `optimizeListeners()` uses: `foreach ($this->listeners as &$byPriority) { foreach
/// ($byPriority as $priority => &$listeners) { ... } }`. `php -n` 8.5.6 prints `X,Y/Z`.
#[test]
fn test_runtime_dynamic_include_nested_by_reference_foreach_writes_through_outer_reference() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include $path; } load('piece.php');",
            ),
            (
                "piece.php",
                "<?php\n$d = [[\"x\", \"y\"], [\"z\"]];\nforeach ($d as &$row) {\n    foreach ($row as &$cell) {\n        $cell = strtoupper($cell);\n    }\n    unset($cell);\n}\nunset($row);\necho implode(\",\", $d[0]), \"/\", implode(\",\", $d[1]);\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "X,Y/Z");
}

/// Verifies a `use (&$x)` closure capture binds the CELL `$x` names at closure-creation time,
/// not the NAME, when the whole fragment executes through the eval bridge (a runtime-only-known
/// include path forces the interpreter to run the included file, rather than the AOT compiler
/// lowering it directly) -- a faithful replica of
/// `Symfony\Component\EventDispatcher\EventDispatcher::optimizeListeners()`: a private method
/// nests two by-reference `foreach` loops, binds a reference to an appended slot of an object
/// property array, and stores a `static` closure that captures that reference (plus the loop's
/// element alias) and replaces itself in that slot on first call.
///
/// `php -n` 8.5.6 prints `high:x|W(x)|` then `high:y|W(y)|W(y)` then `p4done`. Before the fix,
/// the closure's by-reference captures resolved their caller-side write-back target by
/// (raw-scope-pointer, NAME) instead of holding the cell captured at closure-creation time; here
/// the named scope is `optimizeListeners()`'s own activation, which has already returned and
/// been dropped by the time any stored closure is first invoked, so write-back dereferenced
/// freed Rust stack memory. elephc SIGSEGV'd (exit 139) with no output at all.
#[test]
fn test_runtime_dynamic_include_self_replacing_closure_survives_dropped_defining_method_scope() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include $path; } load('piece.php');",
            ),
            (
                "piece.php",
                r#"<?php
class Wrapped
{
    public function __invoke(string $tag): string
    {
        return "W(" . $tag . ")";
    }
}

class Dispatcher
{
    private array $listeners = [];
    private array $optimized = [];

    public function add(string $event, int $priority, callable $listener): void
    {
        $this->listeners[$event][$priority][] = $listener;
        unset($this->optimized[$event]);
    }

    private function optimizeListeners(string $eventName): array
    {
        krsort($this->listeners[$eventName]);
        $this->optimized[$eventName] = [];

        foreach ($this->listeners[$eventName] as &$listeners) {
            foreach ($listeners as &$listener) {
                $closure = &$this->optimized[$eventName][];
                if (\is_array($listener) && isset($listener[0]) && $listener[0] instanceof \Closure && 2 >= \count($listener)) {
                    $closure = static function (...$args) use (&$listener, &$closure) {
                        if ($listener[0] instanceof \Closure) {
                            $listener[0] = $listener[0]();
                            $listener[1] ??= '__invoke';
                        }
                        ($closure = $listener(...))(...$args);
                    };
                } else {
                    $closure = $listener instanceof Wrapped ? $listener : $listener(...);
                }
            }
        }

        return $this->optimized[$eventName];
    }

    public function call(string $eventName, string $tag): string
    {
        $out = [];
        foreach ($this->optimized[$eventName] ?? $this->optimizeListeners($eventName) as $listener) {
            $out[] = $listener($tag);
        }
        return implode("|", $out);
    }
}

$d = new Dispatcher();
$d->add("evt", 10, static fn (string $t): string => "high:" . $t);
$d->add("evt", 5, new Wrapped());
$d->add("evt", 1, [static fn (): object => new Wrapped(), "__invoke"]);

echo $d->call("evt", "x"), "\n";
echo $d->call("evt", "y"), "\n";
echo "p4done\n";
"#,
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "high:x|W(x)|\nhigh:y|W(y)|W(y)\np4done\n");
}

/// Verifies `EXPR(...)` first-class-callable syntax produces a working `Closure` for every
/// runtime value shape when the whole fragment executes through the eval bridge (a
/// runtime-only-known include path forces the interpreter to run the included file, rather than
/// the AOT compiler lowering it directly) -- the exact surface
/// `Symfony\Component\EventDispatcher\EventDispatcher::optimizeListeners()` exercises through
/// `$listener(...)` where `$listener` already holds a `Closure`.
///
/// `php -n` 8.5.6 prints `closurevar:6|arrayvar:m:7|stringvar:eight`. Before the fix, the
/// interpreter unconditionally wrapped every `EXPR(...)` value in an `InvokableObject { object }`
/// closure target regardless of what `EXPR` evaluated to: a Closure-valued variable produced a
/// target whose "object" was itself a closure vessel, so calling it asked the runtime for that
/// vessel's native class -- the placeholder `stdClass`, never `Closure` -- and raised `Error:
/// Object of type stdClass is not callable`; an array or string callable variable had no matching
/// dispatch case at all and failed with `unsupported DynamicCall expression`.
#[test]
fn test_runtime_dynamic_include_first_class_callable_syntax_on_variable_values() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include $path; } load('piece.php');",
            ),
            (
                "piece.php",
                "<?php\nclass FccValHolder {\n    public function m($t) { return \"m:\" . $t; }\n}\n$h = new FccValHolder();\n\n$arrow = static function ($t) { return $t; };\n$closureVar = $arrow;\n$e = $closureVar(...);\necho \"closurevar:\" . $e(\"6\") . \"|\";\n\n$pair = [$h, \"m\"];\n$arrayCallableVar = $pair;\n$f = $arrayCallableVar(...);\necho \"arrayvar:\" . $f(\"7\") . \"|\";\n\n$name = \"strtolower\";\n$stringNameVar = $name;\n$i = $stringNameVar(...);\necho \"stringvar:\" . $i(\"EIGHT\");\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "closurevar:6|arrayvar:m:7|stringvar:eight");
}

/// Verifies a dynamic require expression returns the included file's explicit value.
#[test]
fn test_runtime_dynamic_require_expression_returns_value() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { return require $path; } echo load('value.php');",
            ),
            ("value.php", "<?php return 'dynamic-value';"),
        ],
        "main.php",
    );
    assert_eq!(out, "dynamic-value");
}

/// Verifies dynamic include_once canonicalizes and executes one selected path only once.
#[test]
fn test_runtime_dynamic_include_once_tracks_loaded_path() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include_once $path; } load('once.php'); load('./once.php');",
            ),
            ("once.php", "<?php echo 'once';"),
        ],
        "main.php",
    );
    assert_eq!(out, "once");
}

// `require __DIR__ . '/...';` is the most common idiomatic include pattern
// in PHP. After magic-constant substitution, __DIR__ becomes a string literal
// and the resolver's path folder concatenates it with the trailing literal.

/// Verifies `require __DIR__ . '/lib/inner.php'` works after magic-constant
/// substitution. `__DIR__` is lowered to a string literal by the magic-constants
/// pass; the resolver then concatenates it with the trailing literal to produce
/// the resolved include path.
#[test]
fn test_include_with_dunder_dir_concat() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire __DIR__ . '/lib/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require LIB . '/inner.php'` where `LIB` is a top-level `const`
/// defined before the require. The resolver resolves the const reference to
/// the string `'lib'` before path resolution.
#[test]
fn test_include_with_const_ref() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nconst LIB = 'lib';\nrequire LIB . '/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require LIB . '/inner.php'` where `LIB` is a top-level `define()`
/// evaluated before the require. The resolver tracks defines incrementally as
/// files are inlined, allowing the require to reference a define set earlier.
#[test]
fn test_include_with_define_ref() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\ndefine('LIB', 'lib');\nrequire LIB . '/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require LIB . '/inner.php'` where `LIB` is set via the fully
/// qualified `\define()` call. The backslash prefix is canonical and the
/// define still feeds the include path resolution.
#[test]
fn test_include_with_fully_qualified_define_ref() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\n\\define('LIB', 'lib');\nrequire LIB . '/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require __DIR__ . '/' . 'lib' . '/' . 'inner.php'` with multiple
/// chained concatenations. The resolver must correctly evaluate all BinaryOp
/// nodes in the path expression before resolving the include path.
#[test]
fn test_include_with_nested_concat() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire __DIR__ . '/' . 'lib' . '/' . 'inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies a constant defined in an included bootstrap file can be used in a
/// subsequent require within the main file. The resolver tracks constants
/// incrementally as files are inlined in order, so this cross-file forward
/// reference works.
#[test]
fn test_include_with_const_defined_in_included_file() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'bootstrap.php';\nrequire SUBDIR . '/inner.php';\necho 'after';\n",
            ),
            (
                "bootstrap.php",
                "<?php\nconst SUBDIR = 'lib';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `const BASE = __DIR__ . '/lib'; require BASE . '/inner.php'`
/// works. `__DIR__` is lowered to a string literal by the magic-constants pass
/// before the resolver runs, so the path folder sees a plain
/// `BinaryOp(StringLiteral, Concat, StringLiteral)`.
#[test]
fn test_include_with_dunder_file_dir_const() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nconst BASE = __DIR__ . '/lib';\nrequire BASE . '/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require LIB . '/inner.php'` where `LIB` is declared inside a
/// namespace (`namespace App; const LIB = 'lib';`). The name resolver applies
/// the namespace scope, so the const reference resolves correctly.
#[test]
fn test_include_with_namespaced_const_ref() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nnamespace App;\nconst LIB = 'lib';\nrequire LIB . '/inner.php';\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies `require LIB . '/inner.php'` where `LIB` is imported via
/// `use const Config\LIB` from an included config file. The `use const`
/// directive makes the const available in the importing file's scope for
/// path resolution.
#[test]
fn test_include_with_const_import_ref() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nnamespace App;\nuse const Config\\LIB;\nrequire 'config.php';\nrequire LIB . '/inner.php';\necho 'after';\n",
            ),
            (
                "config.php",
                "<?php\nnamespace Config;\nconst LIB = 'lib';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies that a const declared in one namespace is not accessible from
/// another namespace when used in an include path. The require must fail
/// because `LIB` in namespace `B` refers to a non-existent const in that scope.
#[test]
fn test_include_does_not_use_const_from_other_namespace() {
    assert!(compile_files_fails(
        &[
            (
                "main.php",
                "<?php\nnamespace A;\nconst LIB = 'lib';\nnamespace B;\nrequire LIB . '/inner.php';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    ));
}

/// Verifies that a `define()` call inside a function does not feed a
/// top-level require. Constants defined inside a function have local scope
/// and cannot be referenced by statements outside that function.
#[test]
fn test_define_inside_function_does_not_feed_top_level_include() {
    assert!(compile_files_fails(
        &[
            (
                "main.php",
                "<?php\nfunction boot() {\n    define('LIB', 'lib');\n}\nrequire LIB . '/inner.php';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    ));
}

/// Verifies that a namespaced `Config\define()` call does not feed a require
/// that references `LIB` at the top level. The callable `Config\define` is not
/// the global `define`, so the const is not set and the require fails.
#[test]
fn test_qualified_define_call_does_not_feed_include() {
    assert!(compile_files_fails(
        &[
            (
                "main.php",
                "<?php\nnamespace App;\nConfig\\define('LIB', 'lib');\nrequire LIB . '/inner.php';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    ));
}

/// Verifies that a `define()` inside a function can feed a `require` that is
/// also inside the same function. Function-local defines are in scope for
/// statements within that function body.
#[test]
fn test_define_inside_function_can_feed_include_in_same_function() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nfunction boot() {\n    define('LIB', 'lib');\n    require LIB . '/inner.php';\n}\nboot();\necho 'after';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho 'inner';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "innerafter");
}

/// Verifies that when an included file defines a function with an untyped
/// parameter, calling it with a specialized array from `load_items()` produces
/// a specialized variant. The `append_item` function is called with array
/// type info, so its `count($items) > 0` guard is not folded away.
#[test]
fn test_include_function_variant_specializes_untyped_array_param_from_call() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$items = load_items();\n$items = append_item($items, \"c\");\necho count($items) . ':' . $items[2];\n",
            ),
            (
                "lib.php",
                "<?php\nfunction load_items() {\n    return [\"a\", \"b\"];\n}\n\nfunction append_item($items, $value) {\n    if (count($items) > 0) {\n        $items[] = $value;\n    }\n    return $items;\n}\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "3:c");
}

/// Verifies that when an included file defines a function with an untyped
/// parameter, calling it with a specialized string from `trim()` produces a
/// specialized variant. The `describe_text` function is called with string type
/// info, enabling inlining and DCE of the strlen branch.
#[test]
fn test_include_function_variant_specializes_untyped_string_param_from_call() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho describe_text(trim(\" hello \"));\n",
            ),
            (
                "lib.php",
                "<?php\nfunction describe_text($input) {\n    if (strlen($input) === 0) {\n        return \"empty\";\n    }\n    return \"len=\" . strlen($input);\n}\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "len=5");
}

/// Verifies that when an included file defines a function with an untyped
/// parameter, calling it with an integer argument (which cannot be passed to
/// strlen) produces a compile error. The specialized variant is not available
/// for that type, and no valid fallback exists.
#[test]
fn test_include_function_variant_keeps_error_when_call_does_not_respecialize() {
    assert!(compile_files_fails(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho describe_text(123);\n",
            ),
            (
                "lib.php",
                "<?php\nfunction describe_text($input) {\n    return strlen($input);\n}\n",
            ),
        ],
        "main.php",
    ));
}

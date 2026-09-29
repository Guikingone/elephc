//! Purpose:
//! PHP sources of the `codegen::locals_retype` end-to-end fixtures that claim the boxed
//! mixed-storage path, shared with the checker-side marking meta-test (issue #787).
//!
//! Called from:
//! - `tests/codegen/locals_retype.rs`, which compiles and runs each source.
//! - `tests/error_tests/type_system.rs`
//!   (`test_every_lowering_fixture_takes_the_mixed_storage_path`), which asserts the checker
//!   really marked each one.
//!
//! Key details:
//! - Included through `#[path]` by both test binaries, so it holds plain `&str` data only.
//! - The meta-test iterates [`MIXED_STORAGE_FIXTURES`]: a new mixed-storage fixture joins that
//!   list, and an edit to a source here reaches the end-to-end run and the marking check at once.
//!   The two sides used to carry hand-copied text that nothing kept in step.

/// An `if`/`else` storing an int on one arm and a string on the other.
pub const BRANCH_DIVERGENT_LOCAL: &str = "<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;";

/// A one-armed `if` that retypes the local; its condition is always true from the CLI.
pub const SINGLE_BRANCH_RETYPE_TAKEN: &str = "<?php $a = 41; if ($argc > 0) { $a = \"ciao\"; } echo $a;";

/// A one-armed `if` that retypes the local only when the program gets five arguments.
pub const SINGLE_BRANCH_RETYPE_NOT_TAKEN: &str = "<?php $a = 41; if ($argc > 5) { $a = \"ciao\"; } echo $a;";

/// A loop that re-stores a fresh concatenated string every iteration.
pub const LOOP_CARRIED_HETEROGENEOUS_LOCAL: &str = "<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a = \"s\" . $i; } echo $a;";

/// The divergent local read by `strlen`, whose checked lowering demands one operand shape.
pub const BRANCH_DIVERGENT_LOCAL_INTO_A_CHECKED_BUILTIN: &str = "<?php if ($argc > 1) { $a = 42; } else { $a = \"hello\"; } echo strlen($a);";

/// The divergent local read by several builtin and inspection surfaces.
pub const BRANCH_DIVERGENT_LOCAL_THROUGH_SEVERAL_BUILTINS: &str = r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
echo strlen($a), "|", strtoupper($a), "|", gettype($a), "|";
var_dump(is_string($a));"#;

/// An int entry binding a loop body may overwrite with a string.
pub const ZERO_TRIP_LOOP_KEEPS_THE_ENTRY_BINDING: &str = "<?php $a = 123456789; for ($i = 1; $i < $argc; $i++) { $a = \"s\"; } var_dump($a);";

/// A divergent top-level local another function writes through `global $a`.
pub const WRITTEN_THROUGH_A_FUNCTION_GLOBAL_ALIAS: &str = r#"<?php
function q() { global $a; $a = 42; }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
q();
echo $a, "|";
var_dump($a);"#;

/// The same cross-body write from a method.
pub const WRITTEN_THROUGH_A_METHOD_GLOBAL_ALIAS: &str = r#"<?php
class W { public function w() { global $a; $a = 42; } }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
(new W())->w();
echo $a, "|";
var_dump($a);"#;

/// The loop-carried local with extra iterations, for the heap-debug fixture.
pub const LOOP_CARRIED_HEAP_STRINGS: &str = "<?php $a = 0; for ($i = 0; $i < $argc + 3; $i++) { $a = \"s\" . $i; } echo $a;";

/// A divergent local holding a heap string on one arm and a raw int on the other.
pub const BRANCH_DIVERGENT_HEAP_STRING: &str = "<?php if ($argc > 1) { $a = 42; } else { $a = \"hello\" . $argc; } echo $a;";

/// A divergent local captured by value by a closure that overwrites its own copy.
pub const CAPTURED_BY_VALUE_AND_OVERWRITTEN_IN_A_CLOSURE: &str = r#"<?php
if ($argc > 1) { $m = 1; } else { $m = "z"; }
$f = function (int $n) use ($m) {
    if ($n > 1) { $m = 0; } else { $m = "s"; }
    return $m;
};
var_dump($f($argc));
$g = function () use ($m) { return $m; };
var_dump($g());"#;

/// A divergent top-level local a function reads back through `global $a`.
pub const READ_BACK_THROUGH_A_GLOBAL_ALIAS: &str = r#"<?php
function q() { global $a; var_dump($a); }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
q();
$a = 42;
q();"#;

/// A divergent top-level local a closure writes through `global $a`.
pub const CLOSURE_DECLARED_GLOBAL: &str = r#"<?php
$w = function () { global $a; $a = 42; };
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
$w();
echo $a, "|";"#;

/// A divergent local re-assigned to a literal, then read by `strlen`.
pub const REASSIGNED_TO_A_LITERAL_INTO_A_CHECKED_BUILTIN: &str = r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
$a = 99;
echo strlen($a);"#;

/// The same re-assigned local read by the builtins whose lowering failed in other ways.
pub const REASSIGNED_TO_A_LITERAL_THROUGH_SEVERAL_BUILTINS: &str = r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
$a = 99;
echo str_repeat($a, 2), "|", strlen($a), "|", strtoupper($a), "|", gettype($a), "|";
var_dump($a);"#;

/// The concat-widened form of the re-assigned shape.
pub const CONCAT_MARKED_REASSIGNED_TO_A_LITERAL: &str = r#"<?php
$a = 0;
if ($argc > 1) { $a = "s" . $argc; }
$a = 5;
echo strlen($a), "|", strtoupper($a), "|";
var_dump($a);"#;

/// The re-assigned shape inside a function body and through non-builtin consumers.
pub const REASSIGNED_TO_A_LITERAL_INSIDE_A_FUNCTION_BODY: &str = r#"<?php
function f(int $n) {
    if ($n > 1) { $a = 42; } else { $a = "hello"; }
    $a = 7;
    return strlen($a) . "|" . strtoupper($a) . "|" . gettype($a);
}
echo f($argc), "\n";
if ($argc > 1) { $c = 1; } else { $c = "x"; }
$c = 3;
switch ($c) { case 3: echo "three|"; break; default: echo "other|"; }
echo ($c == 3 ? "eq" : "ne"), "|", $c + 1, "|";
var_dump($c);"#;

/// A marked store in the `default` arm of a fallthrough single-case `switch`.
pub const STORED_IN_A_FALLTHROUGH_SWITCH_DEFAULT: &str = "<?php $a = 0; switch ($argc) { case 1: echo \"one|\"; default: $a = \"ciao\" . $argc; } echo $a, \"|\"; var_dump($a);";

/// Every fixture above, in the order the marking meta-test checks them.
pub const MIXED_STORAGE_FIXTURES: &[&str] = &[
    BRANCH_DIVERGENT_LOCAL,
    SINGLE_BRANCH_RETYPE_TAKEN,
    SINGLE_BRANCH_RETYPE_NOT_TAKEN,
    LOOP_CARRIED_HETEROGENEOUS_LOCAL,
    BRANCH_DIVERGENT_LOCAL_INTO_A_CHECKED_BUILTIN,
    BRANCH_DIVERGENT_LOCAL_THROUGH_SEVERAL_BUILTINS,
    ZERO_TRIP_LOOP_KEEPS_THE_ENTRY_BINDING,
    WRITTEN_THROUGH_A_FUNCTION_GLOBAL_ALIAS,
    WRITTEN_THROUGH_A_METHOD_GLOBAL_ALIAS,
    LOOP_CARRIED_HEAP_STRINGS,
    BRANCH_DIVERGENT_HEAP_STRING,
    CAPTURED_BY_VALUE_AND_OVERWRITTEN_IN_A_CLOSURE,
    READ_BACK_THROUGH_A_GLOBAL_ALIAS,
    CLOSURE_DECLARED_GLOBAL,
    REASSIGNED_TO_A_LITERAL_INTO_A_CHECKED_BUILTIN,
    REASSIGNED_TO_A_LITERAL_THROUGH_SEVERAL_BUILTINS,
    CONCAT_MARKED_REASSIGNED_TO_A_LITERAL,
    REASSIGNED_TO_A_LITERAL_INSIDE_A_FUNCTION_BODY,
    STORED_IN_A_FALLTHROUGH_SWITCH_DEFAULT,
];

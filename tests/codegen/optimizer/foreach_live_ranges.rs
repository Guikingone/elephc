//! Purpose:
//! Verifies that a trailing `foreach` which starts a new live range of a reused local keeps PHP's
//! results, destructor order and heap ownership once it binds a fresh name.
//!
//! Called from:
//! - The optimizer codegen integration suite.
//!
//! Key details:
//! - Covers function and top-level bodies, an object range retired at the first iteration of the
//!   later loop, an empty later loop that must keep the earlier value alive, a body whose later
//!   mention of the name forbids the split, PHP's value-then-key release order, function-exit
//!   destruction at the original variable's position, and a superglobal binding.

use super::*;

/// Split `foreach` ranges keep PHP output and destructor timing in both optimizer modes.
#[test]
fn test_split_foreach_live_ranges_keep_php_semantics() {
    let source = r#"<?php
class D { public function __construct(public string $n) {} public function __destruct() { echo "destruct {$this->n}\n"; } }
function f1(): void {
    foreach ([new D("a"), new D("b")] as $w) { echo "first {$w->n}\n"; }
    echo "between\n";
    foreach (["x" => 1, "y" => 2] as $w => $c) { echo "second $w $c\n"; }
    echo "end f1\n";
}
function f2(): void {
    foreach ([new D("c")] as $w) { echo "first {$w->n}\n"; }
    foreach ([] as $w => $c) { echo "never\n"; }
    echo "end f2\n";
}
function f3(array $words): array {
    $counts = [];
    foreach ($words as $w) { $counts[$w] = ($counts[$w] ?? 0) + 1; }
    $out = [];
    foreach ($counts as $w => $c) { $out[] = "$w:$c"; }
    return $out;
}
function f4(): void {
    $w = "keep";
    foreach ([1, 2] as $w) { echo $w, "\n"; }
    echo "after: $w\n";
}
function f5(): void {
    $k = new D("oldkey"); $v = new D("oldval"); echo "start\n";
    foreach (["x" => 1] as $k => $v) { echo "iter $k\n"; }
    echo "end f5\n";
}
function f6(): void {
    $w = new D("w"); $z = new D("z");
    foreach (["only" => 1] as $w => $c) { echo "iter $w\n"; }
    echo "end f6\n";
}
function f7(): void {
    $_GET = ["init" => 1];
    foreach (["a" => 1] as $_GET => $c) { echo json_encode($_GET), "\n"; }
}
f5(); f6(); f7();
f1(); f2();
echo implode(",", f3(["10", "a", "10", "b", "a"])), "\n";
f4();
$words = ["p", "q", "p"];
$n = [];
foreach ($words as $w) { $n[$w] = ($n[$w] ?? 0) + 1; }
foreach ($n as $w => $c) { echo "$w=$c\n"; }
foreach ([new D("top")] as $v) { echo "top first\n"; }
foreach ([1 => "one"] as $v) { echo "top second $v\n"; }
echo "script end\n";
"#;
    let expected = r#"start
destruct oldval
destruct oldkey
iter x
end f5
destruct w
iter only
end f6
destruct z
"a"
first a
first b
destruct a
between
destruct b
second x 1
second y 2
end f1
first c
end f2
destruct c
10:2,a:2,b:1
1
2
after: 2
p=2
q=1
top first
destruct top
top second one
script end
"#;
    for off in [false, true] {
        let run = || {
            let out = compile_and_run_with_heap_debug(source);
            assert!(out.success, "{}", out.stderr);
            assert_eq!(out.stdout, expected, "{}", out.stderr);
            assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
        };
        if off { without_ir_opt(run); } else { run(); }
    }
}

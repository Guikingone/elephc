//! Purpose:
//! Regressions for surplus positional arguments to user functions, methods and constructors.
//!
//! Called from:
//! - `cargo test --test codegen_tests surplus_arguments`.
//!
//! Key details:
//! - php ignores arguments a user callable does not declare (they stay visible to
//!   `func_get_args()`) and still evaluates them; only php's built-in functions refuse them.
//! - `crate::func_args` gives exactly the over-supplied callables the hidden argument collector.

use crate::support::*;

/// Surplus arguments are evaluated, ignored by the parameters, and seen by `func_get_args()`.
#[test]
fn test_surplus_arguments_to_user_callables_are_ignored() {
    let run = compile_and_run_with_heap_debug(r#"<?php
function f($a) { return $a; }
function g($a, $b = 1) { return $a + $b; }
function tick($v) { echo "eval $v\n"; return $v; }
echo f(1, 2), "\n";
echo g(1, 2, 3), "\n";
echo f(tick(1), tick(2)), "\n";
class C {
    public function get() { return implode(",", func_get_args()); }
    public static function s($x) { return "s:$x"; }
}
echo (new C)->get('a', 'b'), "\n";
echo C::s(1, 2), "\n";
class Owner {
    private function __construct(int $n) { echo "n=$n\n"; }
    public static function makeChild(): Owner { return new Child(1, 2, 3); }
}
class Child extends Owner {}
var_dump(Owner::makeChild() instanceof Child);
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "1\n3\neval 1\neval 2\n1\na,b\ns:1\nn=1\nbool(true)\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}

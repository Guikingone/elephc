//! Purpose:
//! Regressions for untyped properties without a default that the program never writes.
//!
//! Called from:
//! - `cargo test --test codegen_tests untyped_null_properties`.
//!
//! Key details:
//! - Such a slot starts `void` in the checker and only a write refines it; a never-written one is
//!   finalized as `mixed`, the representation of php's `null`.
//! - `var $x;` is php's legacy spelling of `public $x;`.

use crate::support::*;

/// A never-written untyped property reads as `null` through every consumer and leaks nothing.
#[test]
fn test_never_written_untyped_property_reads_null() {
    let run = compile_and_run_with_heap_debug(r#"<?php
class P { public $a = 1; var $b; protected $c; private $d; }
class Child extends P { public $e; }
$p = new P;
var_dump($p->b);
var_dump($p);
print_r(new Child); echo "\n";
echo json_encode(new P), "\n";
var_dump(isset($p->b), $p->b ?? "dflt", clone $p == $p);
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "NULL\nobject(P)#1 (4) {\n  [\"a\"]=>\n  int(1)\n  [\"b\"]=>\n  NULL\n  [\"c\":protected]=>\n  NULL\n  [\"d\":\"P\":private]=>\n  NULL\n}\n\
         Child Object\n(\n    [a] => 1\n    [b] => \n    [c:protected] => \n    [d:P:private] => \n    [e] => \n)\n\n\
         {\"a\":1,\"b\":null}\nbool(false)\nstring(4) \"dflt\"\nbool(true)\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}

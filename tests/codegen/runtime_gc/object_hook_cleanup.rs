//! Purpose:
//! Checks object releases that skip the protected cleanup boundary when no PHP code can run.
//!
//! Called from:
//! - Focused `codegen_tests_runtime` runtime ownership regression runs.
//!
//! Key details:
//! - Shared-owner decrements, destructor-free objects and unobserved WeakMap keys take the
//!   direct cleanup stub; a nested throwing destructor and an observed key keep protection.
//! - Output must match PHP and the heap must finish balanced.

use crate::support::*;

/// Mixes direct and protected object-hook releases in one program.
#[test]
fn test_object_hook_releases_keep_destructor_and_weak_map_semantics() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Plain { public $child; public $n; function __construct($n, $c = null) { $this->n = $n; $this->child = $c; } }
class Boom { function __destruct() { throw new RuntimeException("boom"); } }
class Loud { public $n; function __construct($n) { $this->n = $n; } function __destruct() { echo "bye ", $this->n, "\n"; } }
$shapes = [new Plain(1), new Plain(2)];
$sum = 0;
for ($i = 0; $i < 1000; $i++) { $s = $shapes[$i % 2]; $sum += $s->n; }
echo $sum, "\n";
try { $p = new Plain(3, new Boom()); $p = null; echo "not reached\n"; } catch (RuntimeException $e) { echo "caught ", $e->getMessage(), "\n"; }
$wm = new WeakMap(); $k = new Plain(4); $wm[$k] = "v"; echo count($wm), "\n"; $k = null; echo count($wm), "\n";
$a = new Loud("a"); $b = $a; $a = null; echo "shared\n"; $b = null;
$list = [new Plain(5, new Loud("nested")), new Plain(6)]; unset($list); echo "end\n";
"#,
    );
    assert_eq!(out.stdout, "1500\ncaught boom\n1\n0\nshared\nbye a\nbye nested\nend\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

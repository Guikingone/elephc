//! Purpose:
//! End-to-end regressions for issue #982: an eval-declared function that returns a
//! value it only borrowed — `$this`, or a by-value parameter — must not destroy the
//! caller's object when the result is discarded.
//!
//! Called from:
//! - `cargo test --test codegen_tests eval_borrowed_return` through Rust's test harness.
//!
//! Key details:
//! - `$this` and by-value parameters are bound into the callee scope as
//!   `ScopeCellOwnership::Borrowed`; `return $this;` handed the caller a reference it
//!   never owned, and discarding the result released the receiver's own reference.
//! - The reported symptom was the fluent-interface idiom `$o->add("a");` making every
//!   later use of `$o` fail. `$o instanceof Bag` answering `bool(false)` is the sharper
//!   pin: the call itself always succeeded, so the receiver is destroyed, not refused.
//! - Fixtures cover `self`, `static`, an undeclared return type, a plain class name,
//!   a function returning its parameter, and the values that were already correct
//!   (a callee-created object, a property read) as controls.

use crate::support::compile_and_run;

/// Verifies discarding a `$this` return leaves the receiver usable, for every spelling
/// of the return type including none at all.
#[test]
fn test_eval_discarded_this_return_keeps_the_receiver_alive() {
    let out = compile_and_run(
        r#"<?php
eval('
class EvalBorrowBag {
    private array $items = [];
    public function add(string $v): self { $this->items[] = $v; return $this; }
    public function addStatic(string $v): static { $this->items[] = $v; return $this; }
    public function addBare(string $v) { $this->items[] = $v; return $this; }
    public function count(): int { return count($this->items); }
}

$o = new EvalBorrowBag();
$o->add("a");
echo ($o instanceof EvalBorrowBag) ? "yes" : "no", "|";
$o->addStatic("b");
$o->addBare("c");
echo $o->count(), "|";
$o->add("d")->add("e");
echo $o->count(), "|";
');
"#,
    );

    assert_eq!(out, "yes|3|5|");
}

/// Verifies a function returning its own by-value parameter does not destroy the
/// caller's object when the result is discarded, and that the two shapes that were
/// already correct stay correct.
#[test]
fn test_eval_discarded_parameter_passthrough_keeps_the_caller_object_alive() {
    let out = compile_and_run(
        r#"<?php
eval('
class EvalBorrowNode { public int $n = 7; }

function evalBorrowPassthru(EvalBorrowNode $t): EvalBorrowNode { return $t; }
function evalBorrowMake(): EvalBorrowNode { $t = new EvalBorrowNode(); return $t; }

class EvalBorrowHolder {
    public EvalBorrowNode $inner;
    public function __construct() { $this->inner = new EvalBorrowNode(); }
    public function get(): EvalBorrowNode { return $this->inner; }
}

$o = new EvalBorrowNode();
evalBorrowPassthru($o);
echo ($o instanceof EvalBorrowNode) ? "yes" : "no", "|";
echo $o->n, "|";

$kept = evalBorrowMake();
evalBorrowMake();
echo ($kept instanceof EvalBorrowNode) ? "yes" : "no", "|";

$h = new EvalBorrowHolder();
$h->get();
echo ($h->inner instanceof EvalBorrowNode) ? "yes" : "no", "|";
echo $h->inner->n, "|";
');
"#,
    );

    assert_eq!(out, "yes|7|yes|yes|7|");
}

/// Verifies discarded borrowed returns of non-object values leave the caller's
/// string, array, and int intact, and that repeated discards on one receiver
/// accumulate rather than tear it down.
#[test]
fn test_eval_discarded_scalar_and_array_passthrough_leaves_the_caller_value_intact() {
    let out = compile_and_run(
        r#"<?php
eval('
function evalBorrowEcho(string $s): string { return $s; }
function evalBorrowArr(array $a): array { return $a; }
function evalBorrowInt(int $n): int { return $n; }

$s = "hello";
evalBorrowEcho($s);
echo $s, "|", strlen($s), "|";

$a = ["x", "y", "z"];
evalBorrowArr($a);
echo count($a), "|", $a[1], "|";

$n = 41;
evalBorrowInt($n);
echo $n + 1, "|";

class EvalBorrowChain {
    public array $log = [];
    public function tap(string $v): self { $this->log[] = $v; return $this; }
}
$c = new EvalBorrowChain();
$c->tap("a");
$c->tap("b");
$c->tap("c");
echo count($c->log), "|", implode(",", $c->log), "|";
');
"#,
    );

    assert_eq!(out, "hello|5|3|y|42|3|a,b,c|");
}

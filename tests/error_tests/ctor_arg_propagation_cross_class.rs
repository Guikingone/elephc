//! Purpose:
//! Regression tests for cross-class constructor-argument type propagation. They pin the fix
//! for the param-index corruption bug in `Checker::propagate_constructor_arg_type`, where
//! instantiating a subclass without its own constructor corrupted a reordering sibling's
//! stored constructor signature.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `expect_ok` runs the full frontend and asserts no diagnostic is reported.
//! - The reordering-child fixture mirrors the confirmed Symfony console failure shape
//!   (`ConsoleEvent` / `ConsoleCommandEvent` / `ConsoleErrorEvent`) reduced to a minimal repro.
//! - The preserve-refinement fixtures guard that genuinely-untyped promoted params still refine.

use super::*;

/// Regression: instantiating a no-own-constructor sibling must NOT corrupt a reordering
/// child's stored constructor signature.
///
/// `CommandEvent extends BaseEvent` has no own `__construct`, so propagation falls back to the
/// parent's promoted-property order (command, input, output). `ErrorEvent extends BaseEvent`
/// reorders its own constructor and does not promote command/input/output. Before the fix, the
/// propagation loop reused the instantiated class's param index and overwrote `ErrorEvent`'s
/// params[0..3] with the wrong types, yielding a spurious
/// "parameter $input expects Command, got InputInterface". The fix resolves the param index
/// per-target from each class's own `constructor_param_to_prop`, so the reordering child is
/// skipped and its signature stays intact. This must now type-check cleanly.
#[test]
fn test_reordering_child_ctor_not_corrupted_by_sibling_instantiation() {
    expect_ok(
        "<?php
class Command {}
interface InputInterface {}
interface OutputInterface {}
class BaseEvent {
    public function __construct(
        protected ?Command $command,
        protected InputInterface $input,
        protected OutputInterface $output,
    ) {}
}
class CommandEvent extends BaseEvent {}
class ErrorEvent extends BaseEvent {
    public function __construct(
        InputInterface $input,
        OutputInterface $output,
        private \\Throwable $error,
        ?Command $command = null,
    ) { parent::__construct($command, $input, $output); }
}
function trigger(?Command $c, InputInterface $i, OutputInterface $o, \\Throwable $e): void {
    $s = new CommandEvent($c, $i, $o);
    $x = new ErrorEvent($i, $o, $e);
}
",
    );
}

/// Regression (order-independent): the corruption is global once it happens, so instantiating
/// the reordering child BEFORE the no-own-ctor sibling must also stay clean. Guards against a
/// fix that only worked because of a particular instantiation order.
#[test]
fn test_reordering_child_ctor_clean_regardless_of_instantiation_order() {
    expect_ok(
        "<?php
class Command {}
interface InputInterface {}
interface OutputInterface {}
class BaseEvent {
    public function __construct(
        protected ?Command $command,
        protected InputInterface $input,
        protected OutputInterface $output,
    ) {}
}
class CommandEvent extends BaseEvent {}
class ErrorEvent extends BaseEvent {
    public function __construct(
        InputInterface $input,
        OutputInterface $output,
        private \\Throwable $error,
        ?Command $command = null,
    ) { parent::__construct($command, $input, $output); }
}
function trigger(?Command $c, InputInterface $i, OutputInterface $o, \\Throwable $e): void {
    $x = new ErrorEvent($i, $o, $e);
    $s = new CommandEvent($c, $i, $o);
}
",
    );
}

/// Preserve-refinement: a genuinely-untyped promoted constructor param must still be refined
/// from the argument type, both on the declaring class and on a subclass that has no own
/// constructor. The fix must not regress the intended gradual-typing sharpening. Passing an
/// `int` into the untyped promoted `$x` on both `C` and its no-own-ctor subclass `D` must
/// remain accepted (no spurious diagnostic).
#[test]
fn test_untyped_promoted_param_still_refines_across_no_ctor_subclass() {
    expect_ok(
        "<?php
class C {
    public function __construct(public $x) {}
}
class D extends C {}
function build(): void {
    $c = new C(5);
    $d = new D(7);
}
",
    );
}

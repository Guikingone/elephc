//! Purpose:
//! Unit tests for the Stage-2 reachability fixpoint (`compute_reachable`): the worklist, the edge
//! table, virtual dispatch, and the soundness fallbacks (untyped receiver, dynamic `new`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`cargo test tree_shake::reach`).
//!
//! Key details:
//! - Each test parses a small PHP source through the real lexer + parser (no name resolver), so
//!   declaration names are already canonical (sources are un-namespaced). It then harvests the
//!   skeleton and asserts over the reachable set. `has_method`/`has_fn` are case-insensitive on
//!   the method key (PHP method lookup is case-insensitive; the reachable set stores lowercase).

use super::{compute_reachable, harvest_skeleton, Reachable};

/// Parses PHP source, harvests its skeleton, and computes the reachable set from its top level.
fn reach(src: &str) -> Reachable {
    let tokens = crate::lexer::tokenize(src).expect("tokenize should succeed");
    let program = crate::parser::parse(&tokens).expect("parse should succeed");
    let skeleton = harvest_skeleton(&program);
    compute_reachable(&program, &skeleton)
}

/// Returns whether `(class, method)` is reachable, matching the class FQN and lowercase method.
fn has_method(r: &Reachable, class: &str, method: &str) -> bool {
    r.methods.contains(&(class.to_string(), method.to_ascii_lowercase()))
}

/// Returns whether the free function `fqn` is reachable.
fn has_fn(r: &Reachable, fqn: &str) -> bool {
    r.functions.contains(fqn)
}

/// Verifies transitive free-function reachability from the entry point: `a()` calls `b()`, so both
/// are reachable, but the never-called `c()` is pruned.
#[test]
fn transitive_free_functions_only() {
    let r = reach("<?php function a(){ b(); } function b(){} function c(){} a();");
    assert!(has_fn(&r, "a"), "entry function a reachable");
    assert!(has_fn(&r, "b"), "a() calls b()");
    assert!(!has_fn(&r, "c"), "c() is never called");
}

/// Verifies a constructor's body is followed: `new C()` reaches `__construct`, which calls
/// `$this->m()`, so `m` is reachable, while an unused method is pruned.
#[test]
fn constructor_body_and_this_dispatch() {
    let r = reach(
        "<?php
        class C {
            function __construct(){ $this->m(); }
            function m(){}
            function unused(){}
        }
        new C();",
    );
    assert!(r.instantiated.contains("C"), "C is constructed");
    assert!(has_method(&r, "C", "__construct"), "constructor reachable");
    assert!(has_method(&r, "C", "m"), "$this->m() reachable from ctor");
    assert!(!has_method(&r, "C", "unused"), "unused method pruned");
}

/// Verifies interface-typed virtual dispatch fans out to every implementer that is instantiated:
/// `f(I $x){ $x->go(); }` keeps `Impl::go` (Impl is `new`-ed) but NOT `Other::go` (never
/// constructed), and only `Impl::__construct` is reachable — RTA keeps dispatch tight.
#[test]
fn interface_dispatch_over_instantiated_implementers() {
    let r = reach(
        "<?php
        interface I { function go(); }
        class Impl implements I { function go(){} }
        class Other implements I { function go(){} }
        function f(I $x){ $x->go(); }
        f(new Impl());",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go reachable (new-ed implementer)");
    assert!(r.instantiated.contains("Impl"), "Impl constructed");
    assert!(!r.instantiated.contains("Other"), "Other never constructed");
    assert!(!has_method(&r, "Other", "go"), "Other::go pruned (never instantiated)");
}

/// Verifies interface dispatch stays sound when a second implementer IS also constructed:
/// constructing both `Impl` and `Other` and calling `$x->go()` on an `I` keeps both `go`s.
#[test]
fn interface_dispatch_keeps_all_constructed_implementers() {
    let r = reach(
        "<?php
        interface I { function go(); }
        class Impl implements I { function go(){} }
        class Other implements I { function go(){} }
        function f(I $x){ $x->go(); }
        f(new Impl());
        f(new Other());",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go reachable");
    assert!(has_method(&r, "Other", "go"), "Other::go reachable (also constructed)");
}

/// Verifies an abstract-base `$this->execute()` reaches both the base method and a subclass
/// override once the subclass is constructed (mirrors `Command` → `HelloCommand::execute`).
#[test]
fn abstract_base_this_dispatch_reaches_override() {
    let r = reach(
        "<?php
        abstract class Base {
            function run(){ $this->execute(); }
            function execute(){}
        }
        class Sub extends Base {
            function execute(){}
        }
        function drive(Base $b){ $b->run(); }
        drive(new Sub());",
    );
    assert!(has_method(&r, "Base", "run"), "Base::run reachable");
    assert!(has_method(&r, "Sub", "execute"), "override Sub::execute reachable");
    // The base `execute` is only kept if Base itself is instantiated; here only Sub is `new`-ed,
    // so `$this->execute()` resolves to the constructed Sub override.
    assert!(!r.instantiated.contains("Base"), "abstract Base never constructed");
}

/// Verifies the untyped-receiver FALLBACK is sound and over-approximating: an unknown-typed
/// variable receiver `$x->run()` keeps `run` on EVERY constructed class that has it — including an
/// unrelated `Far` — and records a fallback note. This documents the fallback's cost.
#[test]
fn untyped_receiver_fallback_keeps_all_constructed() {
    let r = reach(
        "<?php
        class Far { function run(){} }
        class Near { function run(){} }
        function make(){ return null; }
        new Far();
        new Near();
        $x = make();
        $x->run();",
    );
    assert!(has_method(&r, "Far", "run"), "unrelated Far::run kept by fallback (sound)");
    assert!(has_method(&r, "Near", "run"), "Near::run kept by fallback");
    assert!(
        r.fallback_sites.iter().any(|note| note.contains("->run()")),
        "a fallback note records the untyped ->run() site"
    );
}

/// Verifies `new $dynamic` triggers the sound construction fallback: every instantiable class is
/// marked constructed and a fallback note is recorded, so a subsequent dispatch cannot miss a
/// method on a class the runtime class-string could name.
#[test]
fn dynamic_new_instantiates_all_instantiable() {
    let r = reach(
        "<?php
        class A { function __construct(){} }
        class B { function __construct(){} }
        abstract class Abs {}
        function spawn($name){ $obj = new $name(); return $obj; }
        spawn('A');",
    );
    assert!(r.instantiated.contains("A"), "A marked constructed by dynamic-new fallback");
    assert!(r.instantiated.contains("B"), "B marked constructed by dynamic-new fallback");
    assert!(!r.instantiated.contains("Abs"), "abstract class is not instantiable");
    assert!(
        r.fallback_sites.iter().any(|note| note.contains("new $dynamic")),
        "a fallback note records the dynamic new site"
    );
    assert!(has_method(&r, "A", "__construct"), "A::__construct kept once constructed");
    assert!(has_method(&r, "B", "__construct"), "B::__construct kept once constructed");
}

/// Verifies a static method call is followed and resolves through inheritance: `C::make()` on a
/// subclass whose `make` is inherited from a parent reaches the declaring parent's method.
#[test]
fn static_call_resolves_through_inheritance() {
    let r = reach(
        "<?php
        class Base { static function make(){ helper(); } }
        class Sub extends Base {}
        function helper(){}
        Sub::make();",
    );
    assert!(has_method(&r, "Base", "make"), "inherited static make resolves to Base");
    assert!(has_fn(&r, "helper"), "static method body followed");
}

/// Verifies a literal `'Class::method'` string callable argument is resolved and its target kept.
#[test]
fn literal_string_callable_is_resolved() {
    let r = reach(
        "<?php
        class Handler { static function handle(){} }
        function apply($cb){}
        apply('Handler::handle');",
    );
    assert!(has_method(&r, "Handler", "handle"), "literal 'Class::method' callable resolved");
}

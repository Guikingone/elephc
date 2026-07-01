//! Purpose:
//! Unit tests for the Stage-2 reachability fixpoint (`compute_reachable`): the worklist, the edge
//! table, virtual dispatch, and the soundness fallbacks (untyped receiver, dynamic `new`). Also
//! covers the Stage-2.5 intra-body local-variable typing (`locals.rs`): local `new`/typed-call
//! receivers resolve precisely, while reassignment and by-reference closure capture widen to `Any`.
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

/// Stage 2.5: a local assigned from an inline `new C` types the receiver to exactly `C`, so
/// `$x->go()` reaches only `Impl::go` even though an unrelated `Other` is also constructed and
/// declares `go`. This is the precision win over Stage 2's untyped-local fallback, which kept both.
#[test]
fn local_new_receiver_types_precisely() {
    let r = reach(
        "<?php
        class Impl { function go(){} }
        class Other { function go(){} }
        new Other();
        $x = new Impl();
        $x->go();",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go reachable via local-new typing");
    assert!(r.instantiated.contains("Other"), "Other is constructed");
    assert!(
        !has_method(&r, "Other", "go"),
        "Other::go pruned: $x typed to Impl beats the untyped fallback"
    );
}

/// Stage 2.5: a local assigned from a call whose callee declares a class return type
/// (`function make(): Impl`) inherits that type, so `$y->go()` reaches only `Impl::go`. `Other` is
/// never constructed here, confirming the return-type path drives both instantiation and dispatch.
#[test]
fn local_typed_call_return_types_precisely() {
    let r = reach(
        "<?php
        class Impl { function go(){} }
        class Other { function go(){} }
        function make(): Impl { return new Impl(); }
        $y = make();
        $y->go();",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go reachable via return-type typing");
    assert!(!r.instantiated.contains("Other"), "Other never constructed");
    assert!(!has_method(&r, "Other", "go"), "Other::go pruned (never constructed, no fallback)");
}

/// Stage 2.5 soundness: a local reassigned with an untyped value joins to `Any` for the whole body
/// (join-per-body, not flow-sensitive), so `$z->go()` falls back and keeps `go` on EVERY
/// constructed class. Proves multi-assignment never under-approximates.
#[test]
fn local_reassignment_widens_to_any() {
    let r = reach(
        "<?php
        class Impl { function go(){} }
        class Other { function go(){} }
        function untyped(){ return null; }
        new Impl();
        new Other();
        $z = new Impl();
        if (untyped()) { $z = untyped(); }
        $z->go();",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go kept via fallback after reassign-to-Any");
    assert!(
        has_method(&r, "Other", "go"),
        "Other::go kept: $z widened to Any keeps go on all constructed classes"
    );
}

/// Stage 2.5 soundness: capturing a local by reference into a closure (`use (&$w)`) lets the closure
/// rebind it, so `$w` widens to `Any` for the whole body and `$w->go()` falls back to every
/// constructed `go`. Proves the by-reference-capture poison rule.
#[test]
fn local_byref_closure_capture_widens_to_any() {
    let r = reach(
        "<?php
        class Impl { function go(){} }
        class Other { function go(){} }
        new Impl();
        new Other();
        $w = new Impl();
        $f = function() use (&$w) {};
        $w->go();",
    );
    assert!(has_method(&r, "Impl", "go"), "Impl::go kept via fallback after by-ref capture");
    assert!(
        has_method(&r, "Other", "go"),
        "Other::go kept: by-reference capture widened $w to Any"
    );
}

/// Stage 2.6: a method used ONLY as a `[$obj, 'literalMethod']` callable value — created outside
/// call-argument position (assigned to a variable) — is collected into the callable universe and
/// kept, while an unrelated method on the same class stays pruned. Proves the universe collection at
/// a value position resolves the object part precisely (typed `$h` → only `Handler::handle`).
#[test]
fn literal_object_callable_value_is_kept() {
    let r = reach(
        "<?php
        class Handler { function handle(){} function unused(){} }
        $h = new Handler();
        $cb = [$h, 'handle'];",
    );
    assert!(r.instantiated.contains("Handler"), "Handler constructed");
    assert!(has_method(&r, "Handler", "handle"), "[$h,'handle'] callable value keeps handle");
    assert!(!has_method(&r, "Handler", "unused"), "unrelated unused method pruned");
}

/// Stage 2.6 (the key precision win): a method that is NEVER used as a callable and not otherwise
/// dispatched is PRUNED even though a reachable opaque `call_user_func($closureVar)` exists — because
/// the closure value is provably bounded (its body is scanned inline), so it does NOT trigger the
/// broad all-methods fallback. Under the old blanket rule, `neverCallable` would have been kept.
#[test]
fn opaque_closure_invocation_does_not_keep_unrelated_methods() {
    let r = reach(
        "<?php
        class Thing { function used(){} function neverCallable(){} }
        $t = new Thing();
        $t->used();
        $fn = function(){};
        call_user_func($fn);",
    );
    assert!(has_method(&r, "Thing", "used"), "Thing::used reachable via typed dispatch");
    assert!(
        !has_method(&r, "Thing", "nevercallable"),
        "neverCallable pruned: opaque call of a bounded closure does not broaden"
    );
    assert!(
        r.fallback_sites.iter().all(|note| !note.contains("computed-string")),
        "a bounded closure invocation records no computed-string (broad) fallback"
    );
}

/// Stage 2.6 soundness: a dynamic-method-name callable `[$o, $m]` (the method name is a variable, not
/// a string literal) created in a callable position is genuinely unbounded, so it fires the broad
/// fallback — keeping BOTH sibling methods of the receiver's class and, soundly, methods of every
/// other constructed class. Proves the form-4 dynamic-method detection never under-approximates.
#[test]
fn dynamic_method_name_callable_triggers_broad_fallback() {
    let r = reach(
        "<?php
        class Pair { function first(){} function second(){} }
        class Unrelated { function first(){} function second(){} }
        $o = new Pair();
        new Unrelated();
        $m = 'first';
        call_user_func([$o, $m]);",
    );
    assert!(has_method(&r, "Pair", "first"), "Pair::first kept by broad fallback");
    assert!(has_method(&r, "Pair", "second"), "sibling Pair::second kept by broad fallback");
    assert!(
        has_method(&r, "Unrelated", "first") && has_method(&r, "Unrelated", "second"),
        "unrelated constructed class methods kept: dynamic method name is sound-broad"
    );
    assert!(
        r.fallback_sites.iter().any(|note| note.contains("form-4 dynamic method-name")),
        "a form-4 dynamic method-name fallback note is recorded"
    );
}

/// Stage 2.6: an invokable object opaquely invoked (`call_user_func($g)` where `$g` is a typed
/// object) keeps that object's `__invoke`, but NOT its unrelated methods — the receiver is provably
/// an object, so it is bounded to `__invoke` rather than broadening to all methods.
#[test]
fn invokable_object_opaque_invocation_keeps_invoke_only() {
    let r = reach(
        "<?php
        class Greeter { function __invoke(){} function other(){} }
        $g = new Greeter();
        call_user_func($g);",
    );
    assert!(r.instantiated.contains("Greeter"), "Greeter constructed");
    assert!(has_method(&r, "Greeter", "__invoke"), "invokable object keeps __invoke");
    assert!(
        !has_method(&r, "Greeter", "other"),
        "unrelated other() pruned: typed object bounded to __invoke, not broadened"
    );
}

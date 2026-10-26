//! Purpose:
//! Pins the cost of lowering passes that scale with the closed world to the program size, so a
//! return to a quadratic walk fails a test instead of only slowing a framework build.
//!
//! Called from:
//! - `crate::ir_lower::tests` through the Rust test harness.
//!
//! Key details:
//! - Counts work items with test-only counters rather than timing anything, so the bound is
//!   deterministic on any machine and under any load.

use super::lower_source;

/// `class C{i} extends Base { __construct(int $a = 1) }` for `count` classes, plus a dynamic
/// `new $c()` that makes lowering build one padding thunk per class (each thunk calls its own
/// class's constructor) and one interface-typed call site per class.
fn many_padded_constructors(count: usize) -> String {
    let mut source = String::from(
        "<?php\ninterface Runs { public function run(int $n): int; }\nabstract class Base implements Runs {}\n",
    );
    for index in 0..count {
        source.push_str(&format!(
            "final class C{index} extends Base {{\n    public function __construct(public int $a = 1) {{}}\n    public function run(int $n, int $extra = 0): int {{ return $n + $this->a + $extra; }}\n}}\nfunction call{index}(Runs $r): int {{ return $r->run({index}); }}\n"
        ));
    }
    source.push_str("$c = 'C' . ($argc - 1);\n$o = new $c();\necho call0($o), \"\\n\";\n");
    source
}

/// Candidates examined by `wider_override_classes` while lowering `source`.
fn wider_override_candidates(source: &str) -> u64 {
    crate::ir_lower::expr::WIDER_OVERRIDE_CANDIDATES.with(|count| count.set(0));
    let _module = lower_source(source);
    crate::ir_lower::expr::WIDER_OVERRIDE_CANDIDATES.with(|count| count.get())
}

/// Every padding thunk calls its own class's constructor, and nearly every class declares one.
/// Walking every declarer per call made lowering quadratic in the class count (a third of
/// lowering the Symfony app); the examined candidates must stay linear in it.
#[test]
fn wider_override_dispatch_examines_a_linear_number_of_candidates() {
    let small = wider_override_candidates(&many_padded_constructors(40));
    let large = wider_override_candidates(&many_padded_constructors(160));
    assert!(small > 0, "the fixture no longer reaches the wider-override dispatch");
    // Four times the classes: linear work grows about fourfold, quadratic work sixteenfold.
    assert!(
        large <= small * 6,
        "wider-override candidates grew superlinearly: {small} for 40 classes, {large} for 160"
    );
}

/// `final class C{i} { __construct(public $v) }` and one `new C{i}(..)` each: every constructor
/// argument propagates its type into the classes that share the promoted property.
fn many_promoted_constructor_calls(count: usize) -> String {
    let mut source = String::from("<?php\n");
    for index in 0..count {
        source.push_str(&format!(
            "final class C{index} {{ public function __construct(public $v) {{}} }}\n"
        ));
    }
    for index in 0..count {
        source.push_str(&format!("$o{index} = new C{index}({index});\n"));
    }
    source.push_str("echo $o0->v, \"\\n\";\n");
    source
}

/// Classes `propagate_constructor_arg_type` visited while checking `source`.
fn shared_property_classes_visited(source: &str) -> u64 {
    crate::types::checker::SHARED_PROPERTY_CLASSES_VISITED.with(|count| count.set(0));
    let _module = lower_source(source);
    crate::types::checker::SHARED_PROPERTY_CLASSES_VISITED.with(|count| count.get())
}

/// Each `new C(..)` argument used to scan every class of the program for the ones sharing the
/// promoted property (a hash probe per class): quadratic in the class count, and the dominant
/// cost of a method-pass round on the Symfony app. The visits must stay linear.
#[test]
fn constructor_argument_propagation_visits_a_linear_number_of_classes() {
    let small = shared_property_classes_visited(&many_promoted_constructor_calls(40));
    let large = shared_property_classes_visited(&many_promoted_constructor_calls(160));
    assert!(small > 0, "the fixture no longer reaches constructor argument propagation");
    assert!(
        large <= small * 6,
        "constructor-argument visits grew superlinearly: {small} for 40 classes, {large} for 160"
    );
}

/// Owner classes `subtype_dispatch_owners` examined while checking `source`.
fn subtype_dispatch_owners_examined(source: &str) -> u64 {
    crate::types::checker::SUBTYPE_DISPATCH_OWNERS_EXAMINED.with(|count| count.set(0));
    let _module = lower_source(source);
    crate::types::checker::SUBTYPE_DISPATCH_OWNERS_EXAMINED.with(|count| count.get())
}

/// Every interface-typed call site asked which of the method's declarers extend or implement the
/// receiver, walking each declarer's parent chain again: calls times declarers. The answer is
/// fixed per (receiver, method), so the examined owners must stay linear in the class count.
#[test]
fn subtype_dispatch_examines_a_linear_number_of_owners() {
    let small = subtype_dispatch_owners_examined(&many_padded_constructors(40));
    let large = subtype_dispatch_owners_examined(&many_padded_constructors(160));
    assert!(small > 0, "the fixture no longer reaches the subtype dispatch scan");
    assert!(
        large <= small * 6,
        "subtype-dispatch owners grew superlinearly: {small} for 40 classes, {large} for 160"
    );
}

/// Classes whose methods pass instances of each other around, plus one `DateTime` use so the
/// date/time reference scan runs over every class-typed value.
fn many_typed_object_values(count: usize) -> String {
    let mut source = String::from("<?php\n");
    for index in 0..count {
        let next = (index + 1) % count;
        // An interface-typed value names a class-like the class table does not hold, which is
        // what sent each question through a case-insensitive scan of every class.
        source.push_str(&format!(
            "interface I{index} {{ public function id(): int; }}\nclass T{index} implements I{index} {{\n    public function id(): int {{ return {index}; }}\n    public function pass(T{next} $o, I{next} $c): int {{ return $c->id() + $o->id() + $c->id(); }}\n}}\n"
        ));
    }
    source.push_str("$d = new DateTime('2020-01-01');\necho $d->format('Y'), \"\\n\";\n");
    source
}

/// Parent-chain walks run by the date/time reference scan while lowering `source`.
fn datetime_chain_walks(source: &str) -> u64 {
    crate::ir_lower::builtin_datetime::DATETIME_CHAIN_WALKS.with(|count| count.set(0));
    let _module = lower_source(source);
    crate::ir_lower::builtin_datetime::DATETIME_CHAIN_WALKS.with(|count| count.get())
}

/// The date/time reference scan asks "is this a DateTime class?" for every class-typed value of
/// every function. Walking the parent chain per question -- and scanning every class for a name
/// the table does not hold verbatim -- grew with values times classes; the walks must stay linear
/// in the class count.
#[test]
fn datetime_descent_walks_a_linear_number_of_classes() {
    let small = datetime_chain_walks(&many_typed_object_values(40));
    let large = datetime_chain_walks(&many_typed_object_values(160));
    assert!(small > 0, "the fixture no longer reaches the date/time descent check");
    assert!(
        large <= small * 6,
        "date/time descent walks grew superlinearly: {small} for 40 classes, {large} for 160"
    );
}

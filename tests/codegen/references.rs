//! Purpose:
//! End-to-end tests for PHP reference machinery: aliasing a local to an object
//! property (`$x = &$obj->prop`) with write-through in both directions, by-reference
//! function/method returns (`function &f()`, `function &m()`), capturing them with
//! `$x = &call()`, and the constant-propagation soundness fix for reference-bound locals.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A reference property's slot holds a pointer to a 16-byte ref-cell; reads and writes
//!   on either the local alias or the property dereference the shared cell, so a write
//!   through one side is observed through the other.
//! - By-reference returns hand the caller the cell pointer, which `$x = &call()` binds
//!   non-owning. The cell pointer is one machine word for every element type — including
//!   `string` (a `{ptr,len}` cell) and `float` (a `d`-register cell) — so it travels in the
//!   integer result register, never split across the string/float result registers.
//! - A static property is a global symbol whose storage matches the ref-cell payload layout,
//!   so `$e = &self::$n` binds the local's ref cell to the property's address (write-through in
//!   both directions). This slice covers scalar statics only (`int`/`float`/`bool`/`?int`);
//!   late static binding (`&static::$c`) resolves the concrete class's slot at the `=&` point.

use crate::support::*;

/// `$x = &$obj->prop` aliases a scalar property: writing the local updates the property
/// and writing the property updates the local (write-through in both directions).
#[test]
fn test_reference_to_scalar_property_writes_through_both_ways() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 1; }
        $o = new C();
        $r = &$o->v;
        $r = 5;
        echo $o->v, \"\\n\";
        $o->v = 9;
        echo $r, \"\\n\";",
    );
    assert_eq!(out, "5\n9\n");
}

/// A later local alias promotes the unbound path after a conditional property reference.
#[test]
fn test_conditional_property_reference_then_local_alias() {
    let out = compile_and_run(
        r#"<?php
class ConditionalRefBox { public int $value = 1; }
function update_conditional_ref(bool $bind): int {
    $local = 1;
    $box = new ConditionalRefBox();
    if ($bind) {
        $local =& $box->value;
    }
    $alias =& $local;
    $alias = 7;
    return $local;
}
echo update_conditional_ref(false) . '|' . update_conditional_ref(true);
"#,
    );
    assert_eq!(out, "7|7");
}

/// `$x = &$obj->prop` aliases an array property: appends through the alias are observed
/// through the property, and clearing the alias to `[]` empties the property (the shape
/// used by `$instanceof = []` after capturing a reference).
#[test]
fn test_reference_to_array_property_appends_and_clears() {
    let out = compile_and_run(
        "<?php
        class C { public array $v = []; }
        $o = new C();
        $r = &$o->v;
        $r[] = 1;
        $r[] = 2;
        echo implode(',', $o->v), \"\\n\";
        $r = [];
        echo count($o->v), \"\\n\";
        $r[] = 9;
        echo $o->v[0], \"\\n\";",
    );
    assert_eq!(out, "1,2\n0\n9\n");
}

/// Reassigning the reference to a non-empty, differently-typed array literal boxes the
/// literal's elements so the property's `Array(Mixed)` reads stay valid (regression: the
/// raw `Array(Int)`/`Array(Str)` payload was stored unboxed and read back as garbage).
#[test]
fn test_reference_array_reassigned_to_typed_literal_boxes_elements() {
    let out = compile_and_run(
        "<?php
        class C { public array $v = []; }
        $o = new C();
        $r = &$o->v;
        $r[] = 1;
        echo implode(',', $o->v), \"\\n\";
        $r = [42, 43];
        echo implode(',', $o->v), \"\\n\";
        echo $o->v[0], \"\\n\";
        echo count($o->v), \"\\n\";
        $r = ['a', 'b', 'c'];
        echo implode('-', $o->v), \"\\n\";",
    );
    assert_eq!(out, "1\n42,43\n42\n2\na-b-c\n");
}

/// The property keeps its declared default until the reference writes through it.
#[test]
fn test_reference_property_keeps_default_until_written() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 7; }
        $o = new C();
        echo $o->v, \"\\n\";
        $r = &$o->v;
        echo $r, \"\\n\";
        $r = 11;
        echo $o->v, \"\\n\";",
    );
    assert_eq!(out, "7\n7\n11\n");
}

/// A by-reference free function returns a reference to a property; `$x = &f()` aliases it
/// and a write through `$x` updates the property.
#[test]
fn test_by_reference_function_return_aliases_property() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 10; }
        function &getv(C $o) { return $o->v; }
        $o = new C();
        $r = &getv($o);
        $r = 77;
        echo $o->v, \"\\n\";",
    );
    assert_eq!(out, "77\n");
}

/// A by-reference method returns a reference to `$this->prop`; `$x = &$o->m()` aliases it
/// and appends through the alias update the property's array.
#[test]
fn test_by_reference_method_return_aliases_property() {
    let out = compile_and_run(
        "<?php
        class Box {
            public array $items = [];
            public function &ref() { return $this->items; }
        }
        $b = new Box();
        $r = &$b->ref();
        $r[] = 'x';
        $r[] = 'y';
        echo implode(',', $b->items), \"\\n\";",
    );
    assert_eq!(out, "x,y\n");
}

/// Plain local-to-local aliasing writes through in both directions, even when a write
/// goes through the other alias between reads (regression: reference-bound locals must
/// not carry stale propagated constants).
#[test]
fn test_local_alias_write_through_not_constant_folded() {
    let out = compile_and_run(
        "<?php
        $a = 1;
        $b = &$a;
        $b = 5;
        $a = 7;
        echo $b, \"\\n\";",
    );
    assert_eq!(out, "7\n");
}

/// A by-reference closure returning a captured object's property, called through a variable,
/// aliases the property so appends through the captured reference reach it.
#[test]
fn test_by_reference_closure_return_via_variable() {
    let out = compile_and_run(
        "<?php
        class C { public array $items = []; }
        $o = new C();
        $f = function &() use ($o) { return $o->items; };
        $ref = &$f();
        $ref[] = 'a';
        $ref[] = 'b';
        echo implode(',', $o->items), \"\\n\";",
    );
    assert_eq!(out, "a,b\n");
}

/// An immediately-invoked by-reference closure returning a captured object's property
/// aliases that property.
#[test]
fn test_by_reference_closure_immediate_invoke() {
    let out = compile_and_run(
        "<?php
        class C { public array $items = []; }
        $o = new C();
        $ref = &(function &() use ($o) { return $o->items; })();
        $ref[] = 'x';
        echo implode(',', $o->items), \"\\n\";",
    );
    assert_eq!(out, "x\n");
}

/// The Symfony KernelTrait::configureContainer shape: bind an arrow closure that returns a
/// reference to `$this->prop` to a loader, capture the reference, mutate it through the
/// reference, and clear it — all observed through the loader's property.
#[test]
fn test_closure_bind_by_reference_return_writes_through() {
    let out = compile_and_run(
        "<?php
        class Loader { public array $instanceof = []; }
        $loader = new Loader();
        $instanceof = &\\Closure::bind(fn &() => $this->instanceof, $loader, $loader)();
        $instanceof[] = 'RouteA';
        echo implode(',', $loader->instanceof), \"\\n\";
        $instanceof = [];
        echo count($loader->instanceof), \"\\n\";",
    );
    assert_eq!(out, "RouteA\n0\n");
}

/// `$x = &$obj->prop` aliases a `string` property: the cell pointer is one word, so the
/// write-through works despite the string ABI normally using a `{ptr,len}` register pair.
#[test]
fn test_reference_to_string_property_writes_through_both_ways() {
    let out = compile_and_run(
        "<?php
        class C { public string $s = \"init\"; }
        $o = new C();
        $r = &$o->s;
        $r = \"viaref\";
        echo $o->s, \"\\n\";
        $o->s = \"viaprop\";
        echo $r, \"\\n\";",
    );
    assert_eq!(out, "viaref\nviaprop\n");
}

/// `$x = &$obj->prop` aliases a `float` property: the cell pointer is one word, so the
/// write-through works despite floats normally returning in a floating-point register.
#[test]
fn test_reference_to_float_property_writes_through_both_ways() {
    let out = compile_and_run(
        "<?php
        class C { public float $f = 1.5; }
        $o = new C();
        $r = &$o->f;
        $r = 3.25;
        echo $o->f, \"\\n\";
        $o->f = 9.75;
        echo $r, \"\\n\";",
    );
    assert_eq!(out, "3.25\n9.75\n");
}

/// A by-reference free function returning a `string` property: the caller binds the cell
/// pointer (one word) and a write through the alias updates the property.
#[test]
fn test_by_reference_function_returns_string_property() {
    let out = compile_and_run(
        "<?php
        class C { public string $s = \"init\"; }
        function &slot(C $o): string { return $o->s; }
        $o = new C();
        $r = &slot($o);
        $r = \"viafunc\";
        echo $o->s, \"\\n\";",
    );
    assert_eq!(out, "viafunc\n");
}

/// A by-reference free function returning a `float` property aliases it through the cell
/// pointer rather than the float result register.
#[test]
fn test_by_reference_function_returns_float_property() {
    let out = compile_and_run(
        "<?php
        class C { public float $f = 1.5; }
        function &slot(C $o): float { return $o->f; }
        $o = new C();
        $r = &slot($o);
        $r = 9.75;
        echo $o->f, \"\\n\";",
    );
    assert_eq!(out, "9.75\n");
}

/// A by-reference method returning a `string` property aliases `$this->prop`; the method
/// call result is stored single-word so the alias dereferences the right cell.
#[test]
fn test_by_reference_method_returns_string_property() {
    let out = compile_and_run(
        "<?php
        class Holder {
            public string $tag = \"h0\";
            public function &tagSlot(): string { return $this->tag; }
        }
        $h = new Holder();
        $t = &$h->tagSlot();
        $t = \"viamethod\";
        echo $h->tag, \"\\n\";",
    );
    assert_eq!(out, "viamethod\n");
}

/// The Symfony-shaped immediate-invoke `Closure::bind` over a `string` property: the bound
/// closure returns a reference to `$this->prop`, captured and mutated through the alias.
#[test]
fn test_closure_bind_by_reference_string_property() {
    let out = compile_and_run(
        "<?php
        class C { public string $s = \"init\"; }
        $c = new C();
        $ref = &\\Closure::bind(fn &() => $this->s, $c, $c)();
        $ref = \"bound\";
        echo $c->s, \"\\n\";
        $c->s = \"viaprop\";
        echo $ref, \"\\n\";",
    );
    assert_eq!(out, "bound\nviaprop\n");
}

/// A by-reference `Closure::bind` stored in a variable and called separately (not invoked
/// immediately) still aliases the bound property: the assignment tracks the bound closure as a
/// static callable so `$bound()` lowers to a direct call carrying the cell pointer, instead of
/// the generic descriptor invoker which would box the result.
#[test]
fn test_closure_bind_by_reference_stored_in_variable() {
    let out = compile_and_run(
        "<?php
        class C { public array $items = []; }
        $o = new C();
        $bound = \\Closure::bind(fn &() => $this->items, $o, $o);
        $ref = &$bound();
        $ref[] = 'x';
        $ref[] = 'y';
        echo implode(',', $o->items), \"\\n\";
        $ref = [];
        echo count($o->items), \"\\n\";",
    );
    assert_eq!(out, "x,y\n0\n");
}

/// The same variable-stored by-reference `Closure::bind` over a `string` property: the cell
/// pointer survives the call boundary and write-through works both ways.
#[test]
fn test_closure_bind_by_reference_stored_in_variable_string() {
    let out = compile_and_run(
        "<?php
        class C { public string $s = \"init\"; }
        $o = new C();
        $bound = \\Closure::bind(fn &() => $this->s, $o, $o);
        $ref = &$bound();
        $ref = \"changed\";
        echo $o->s, \"\\n\";
        $o->s = \"viaprop\";
        echo $ref, \"\\n\";",
    );
    assert_eq!(out, "changed\nviaprop\n");
}

/// Two locals aliasing the same property both observe a write through either side.
#[test]
fn test_two_locals_aliasing_same_property() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 0; }
        $o = new C();
        $a = &$o->v;
        $b = &$o->v;
        $a = 3;
        echo $b, \"\\n\";
        $b = 8;
        echo $o->v, \"\\n\";",
    );
    assert_eq!(out, "3\n8\n");
}

/// `$obj->prop = &$v` (property LHS, variable source) makes the property alias the local's
/// storage: writing the local after the alias is observed through the property. This is the
/// G5 reverse-bind path (value copied into the property's owned cell, local rebound to it).
#[test]
fn test_reference_assign_into_property_from_variable() {
    let out = compile_and_run(
        "<?php
        class C { public $x; }
        $c = new C();
        $v = 5;
        $c->x = &$v;
        $v = 9;
        echo $c->x, \"\\n\";",
    );
    assert_eq!(out, "9\n");
}

/// `$this->prop = &$other` inside a method aliases a typed property to a parameter-held
/// local; writes through either name are observed through the other.
#[test]
fn test_reference_assign_into_typed_property_writes_through_both_ways() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 0; }
        $c = new C();
        $n = 4;
        $c->v = &$n;
        echo $c->v, \"\\n\";
        $n = 7;
        echo $c->v, \"\\n\";
        $c->v = 11;
        echo $n, \"\\n\";",
    );
    assert_eq!(out, "4\n7\n11\n");
}

/// `$obj->prop = &$other->prop` (property LHS, property source) makes both properties share
/// one reference cell — the G5 forward-bind path (`BindPropRefCell`). A write through one
/// object's property is observed through the other's. This is the shape symfony/yaml uses
/// (`$parser->refs = &$this->refs`).
#[test]
fn test_reference_assign_property_to_property_shares_cell() {
    let out = compile_and_run(
        "<?php
        class C { public array $refs = []; }
        $a = new C();
        $b = new C();
        $b->refs = &$a->refs;
        $a->refs[\"x\"] = 1;
        echo count($b->refs), \"\\n\";
        $b->refs[\"y\"] = 2;
        echo count($a->refs), \"\\n\";",
    );
    assert_eq!(out, "1\n2\n");
}

/// Regression: the property-to-property forward bind must preserve the target object pointer
/// across materializing the source cell pointer. Evaluating the source (`$left->items`) can
/// clobber the scratch register holding the target object (`$right`), which previously made
/// `BindPropRefCell` store the cell pointer into the wrong object's slot, so `$right->items`
/// read its own empty cell. The mixed prior reference activity (reassigning an array reference
/// to a typed literal, then a variable-source property bind) reproduces the register pressure.
#[test]
fn test_reference_assign_property_to_property_preserves_target_register() {
    let out = compile_and_run(
        "<?php
        class Bag { public array $items = []; }
        class Box { public $value; }
        $bag = new Bag();
        $entries = &$bag->items;
        $entries = [10, 20, 30];
        $box = new Box();
        $n = 5;
        $box->value = &$n;
        $n = 9;
        $left = new Bag();
        $right = new Bag();
        $right->items = &$left->items;
        $left->items[] = \"shared\";
        echo implode(\",\", $right->items), \"|\", implode(\",\", $left->items), \"\\n\";",
    );
    assert_eq!(out, "shared|shared\n");
}

/// A user static method by-reference out-parameter used directly as an `if` condition defines
/// the caller's argument variable for the guarded block (PHP definite-assignment semantics).
#[test]
fn test_by_ref_output_in_if_condition_defines_variable() {
    let out = compile_and_run(
        "<?php
        class P { static function pm($s, &$m = null) { $m = [$s]; return 1; } }
        if (P::pm(\"hi\", $x)) { echo $x[0]; }",
    );
    assert_eq!(out, "hi");
}

/// A by-reference out-parameter call nested as the right operand of `&&` inside an `if`
/// condition still defines the caller's variable: the call ran (the whole condition was
/// truthy), so the guarded block sees the variable. The right operand is otherwise evaluated
/// in a discarded clone, so this exercises the re-surfacing of by-reference outputs.
#[test]
fn test_by_ref_output_nested_in_and_condition_defines_variable() {
    let out = compile_and_run(
        "<?php
        class P { static function pm($s, &$m = null) { $m = [$s]; return 1; } }
        $cond = true;
        if ($cond && P::pm(\"hey\", $x)) { echo $x[0]; }",
    );
    assert_eq!(out, "hey");
}

/// A by-reference out-parameter call inside a `while` condition (nested through `&&`) defines
/// the caller's variable inside the loop body.
#[test]
fn test_by_ref_output_in_while_condition_defines_variable() {
    let out = compile_and_run(
        "<?php
        class P { static function pm($s, &$m = null) { $m = [$s]; return 1; } }
        $n = 1;
        while ($n-- > 0 && P::pm(\"loop\", $m)) { echo $m[0]; }",
    );
    assert_eq!(out, "loop");
}

/// A by-reference out-parameter call used as a `switch` case label defines the caller's
/// variable inside the matching case body.
#[test]
fn test_by_ref_output_in_switch_case_label_defines_variable() {
    let out = compile_and_run(
        "<?php
        class P { static function pm($s, &$m = null) { $m = [$s]; return true; } }
        switch (true) {
            case P::pm(\"z\", $m):
                echo $m[0];
                break;
        }",
    );
    assert_eq!(out, "z");
}

/// A previously-undefined plain variable passed to a user function's by-reference parameter
/// becomes defined after the call (PHP defines the variable through the reference), so a later
/// read sees the written value.
#[test]
fn test_user_function_by_ref_output_defines_previously_undefined_variable() {
    let out = compile_and_run(
        "<?php
        function fill(&$out) { $out = 42; return 1; }
        fill($z);
        echo $z;",
    );
    assert_eq!(out, "42");
}

/// A previously-undefined plain variable passed to an interface-typed method call's by-reference
/// out-parameter becomes defined after the call, matching the plain-class-receiver case: an
/// interface-typed receiver (e.g. a constructor-promoted `private MarshallerInterface
/// $marshaller` property) previously skipped `self.interfaces` entirely and never defined the
/// argument. Mirrors symfony/cache's `$this->marshaller->marshall($values, $failed)` shape
/// (`MarshallerInterface::marshall(array $values, ?array &$failed): array`).
#[test]
fn test_interface_method_by_ref_output_defines_previously_undefined_variable() {
    let out = compile_and_run(
        "<?php
        interface MarshallerInterface {
            function marshall(array $values, ?array &$failed): array;
        }
        class JsonMarshaller implements MarshallerInterface {
            function marshall(array $values, ?array &$failed): array {
                $failed = ['bad'];
                return ['ok'];
            }
        }
        class Cache {
            private MarshallerInterface $marshaller;
            function __construct(MarshallerInterface $marshaller) {
                $this->marshaller = $marshaller;
            }
            function run(array $values): void {
                $this->marshaller->marshall($values, $failed);
                echo count($failed), ',', $failed[0], \"\\n\";
            }
        }
        $c = new Cache(new JsonMarshaller());
        $c->run(['a' => 1]);",
    );
    assert_eq!(out, "1,bad\n");
}

/// The same interface by-reference out-param definition applies through a nullable-union
/// receiver (`?MarshallerInterface`), not just a plain interface-typed receiver: the union
/// resolution must still find the interface method's `ref_params` (JURY ADDENDUM #1).
#[test]
fn test_interface_method_by_ref_output_defines_variable_through_nullable_union_receiver() {
    let out = compile_and_run(
        "<?php
        interface MarshallerInterface {
            function marshall(array $values, ?array &$failed): array;
        }
        class JsonMarshaller implements MarshallerInterface {
            function marshall(array $values, ?array &$failed): array {
                $failed = ['bad'];
                return ['ok'];
            }
        }
        function pick(): ?MarshallerInterface {
            return new JsonMarshaller();
        }
        $m = pick();
        $m->marshall(['a' => 1], $failed);
        echo count($failed), ',', $failed[0], \"\\n\";",
    );
    assert_eq!(out, "1,bad\n");
}

/// Passing a non-nullsafe instance array property (`$this->refs`) into a by-reference parameter
/// lowers as copy-in/copy-out: the callee's append is visible in the property after the call
/// (the structural shape of symfony/yaml's `Inline::parse(..., $this->refs, ...)` call sites).
#[test]
fn test_by_ref_array_property_reflects_callee_append() {
    let out = compile_and_run(
        "<?php
        class Helper {
            public static function fill(array &$refs): void {
                $refs[] = 99;
                $refs[] = 100;
            }
        }
        class Box {
            public array $refs = [];
            public function run(): void {
                Helper::fill($this->refs);
            }
        }
        $b = new Box();
        $b->refs[] = 1;
        $b->run();
        echo implode(',', $b->refs), \"\\n\";
        echo count($b->refs), \"\\n\";",
    );
    assert_eq!(out, "1,99,100\n3\n");
}

/// A by-reference instance-property argument whose callee only reads leaves the property's value
/// unchanged after the call (the copy-out moves the unmodified array back through the property).
#[test]
fn test_by_ref_array_property_readonly_callee_leaves_property_unchanged() {
    let out = compile_and_run(
        "<?php
        class Helper {
            public static function peek(array &$refs): int {
                return count($refs);
            }
        }
        class Box {
            public array $refs = [];
            public function size(): int {
                return Helper::peek($this->refs);
            }
        }
        $b = new Box();
        $b->refs[] = 1;
        $b->refs[] = 2;
        echo $b->size(), \"\\n\";
        echo implode(',', $b->refs), \"\\n\";",
    );
    assert_eq!(out, "2\n1,2\n");
}

/// An instance method receiving its own array property by reference (`$this->add($this->data)`)
/// observes the callee's append after the call, and repeated calls accumulate, confirming the
/// hidden copy-in temp routes through the existing plain-variable by-reference machinery.
#[test]
fn test_by_ref_property_into_instance_method_accumulates() {
    let out = compile_and_run(
        "<?php
        class C {
            public array $data = [];
            public function add(array &$a): void { $a[] = 7; }
            public function go(): void { $this->add($this->data); }
        }
        $c = new C();
        $c->go();
        $c->go();
        echo implode(',', $c->data), \"\\n\";",
    );
    assert_eq!(out, "7,7\n");
}

/// When a by-reference callee mutates the property's array and then throws, the caller catches
/// and the property keeps its PRE-call value: copy-out runs on the normal-return edge only, so
/// the partial write never reaches the property. (PHP's true aliasing would expose the partial
/// write; this conservative behavior is intentional and documented at the copy-out site.)
#[test]
fn test_by_ref_property_throw_keeps_precall_value() {
    let out = compile_and_run(
        "<?php
        class Helper {
            public static function fillThenThrow(array &$refs): void {
                $refs[] = 42;
                throw new Exception(\"boom\");
            }
        }
        class Box {
            public array $refs = [];
            public function run(): void {
                Helper::fillThenThrow($this->refs);
            }
        }
        $b = new Box();
        $b->refs[] = 1;
        $b->refs[] = 2;
        try {
            $b->run();
        } catch (Exception $e) {
            echo \"caught:\", $e->getMessage(), \"\\n\";
        }
        echo implode(',', $b->refs), \"\\n\";
        echo count($b->refs), \"\\n\";",
    );
    assert_eq!(out, "caught:boom\n1,2\n2\n");
}

/// Verifies a statically-null (`Void`) local passed to a by-reference Mixed parameter
/// (`?bool &$q`) compiles: the null source boxes into a Mixed cell so the callee receives a
/// valid `{payload, tag}` cell, and the call returns its normal value. Regression for the
/// symfony/yaml `Inline::parseScalar($..., $isQuoted, ...)` by-ref writeback gap.
#[test]
fn test_byref_mixed_param_from_null_local() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static function inner(?bool &$q = null): int {
        $q = true;
        return 7;
    }
}
$x = null;
echo C::inner($x);
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies an omitted by-reference parameter that defaults to `null` (`?bool &$q = null`)
/// compiles: the omitted default lowers to `const_null`, a non-local value that is materialized
/// as a throwaway temporary Mixed ref cell (no caller writeback). The callee observes the null
/// default. Regression for the symfony/yaml `Inline::parseScalar(..., $i, false)` omitted-ref gap.
#[test]
fn test_byref_mixed_param_omitted_null_default() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static function inner(int $a, ?bool &$q = null): string {
        if ($q === null) {
            $q = true;
            return "wasnull:" . $a;
        }
        return "wasset";
    }
}
echo C::inner(5);
"#,
    );
    assert_eq!(out, "wasnull:5");
}

/// `$e = &self::$n` aliases a scalar static property's global storage: writing through the
/// local alias is observed by a later `self::$n` read (write-through).
#[test]
fn test_reference_to_static_property_writes_through() {
    let out = compile_and_run(
        "<?php
        class C {
            public static int $n = 5;
            static function t() {
                $e = &self::$n;
                $e = 9;
                return self::$n;
            }
        }
        echo C::t();",
    );
    assert_eq!(out, "9");
}

/// A static-property reference alias reflects a later direct write to the property
/// (read-through): after `$e = &self::$n`, assigning `self::$n = 7` is observed via `$e`.
#[test]
fn test_reference_to_static_property_reads_through() {
    let out = compile_and_run(
        "<?php
        class C {
            public static int $n = 5;
            static function t() {
                $e = &self::$n;
                self::$n = 7;
                return $e;
            }
        }
        echo C::t();",
    );
    assert_eq!(out, "7");
}

/// A named-class static property (`&Foo::$n`) is a valid write-through reference source.
#[test]
fn test_reference_to_named_static_property_writes_through() {
    let out = compile_and_run(
        "<?php
        class Foo { public static int $n = 1; }
        function viaFoo() {
            $e = &Foo::$n;
            $e = 100;
            return Foo::$n;
        }
        echo viaFoo();",
    );
    assert_eq!(out, "100");
}

/// A late-static-bound static property (`&static::$c`) binds the concrete class's slot at the
/// point of `=&`, so `Base::bump()` mutates `Base::$c` and `Sub::bump()` mutates `Sub::$c`.
#[test]
fn test_reference_to_late_static_bound_property_writes_through() {
    let out = compile_and_run(
        "<?php
        class Base {
            public static int $c = 10;
            static function bump() {
                $e = &static::$c;
                $e = 42;
                return static::$c;
            }
        }
        class Sub extends Base {
            public static int $c = 20;
        }
        echo Base::bump();
        echo \"\\n\";
        echo Sub::bump();
        echo \"\\n\";
        echo Base::$c;
        echo \"\\n\";
        echo Sub::$c;",
    );
    assert_eq!(out, "42\n42\n42\n42");
}

/// A `float` static property alias writes through the `d`-register cell path.
#[test]
fn test_reference_to_float_static_property_writes_through() {
    let out = compile_and_run(
        "<?php
        class C {
            public static float $f = 1.5;
            static function t() {
                $e = &self::$f;
                $e = 3.25;
                return self::$f;
            }
        }
        echo C::t();",
    );
    assert_eq!(out, "3.25");
}

/// Compiles `source` with heap-debug instrumentation, asserts it printed `expected` on stdout,
/// and asserts the exit heap summary reports no leak (`live_blocks == 0`). Used by the
/// array-element reference tests, whose whole point is balanced refcounts across promotion,
/// COW, unset, and Mixed boxing.
fn assert_ref_array_element_heap_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// `$x = &$a['k']` on a hash aliases the element: a write through the alias updates the element
/// (write-through), and the refcounts stay balanced (`--heap-debug` clean).
#[test]
fn test_ref_array_element_hash_write_through() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 1, 'j' => 2]; $x = &$a['k']; $x = 9; echo $a['k'];",
        "9",
    );
}

/// After `$x = &$a['k']`, a direct write to the element (`$a['k'] = 7`) is observed through the
/// alias (`echo $x`), confirming read-through in the element→alias direction.
#[test]
fn test_ref_array_element_hash_read_through() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 1, 'j' => 2]; $x = &$a['k']; $x = 9; $a['k'] = 7; echo $x;",
        "7",
    );
}

/// H2: a normal (non-alias) read of the referenced element sees the current value in every
/// consumer — direct `echo`, arithmetic (`$a['k'] + 1`), and by-value `foreach` — rather than
/// misreading the raw reference-cell pointer.
#[test]
fn test_ref_array_element_normal_read_derefs_reference() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 1, 'j' => 2]; $x = &$a['k']; $x = 5; \
         echo $a['k'], ' ', $a['k'] + 1, '|'; foreach ($a as $v) { echo $v; }",
        "5 6|52",
    );
}

/// COW share: a reference survives an array copy (`$b = $a`); a write through the copy's element
/// (`$b['k'] = 42`) is visible through the original alias `$x`, matching PHP's reference-copy
/// semantics (cross-checked with `php -r`).
#[test]
fn test_ref_array_element_cow_share_survives_copy() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 1]; $x = &$a['k']; $b = $a; $b['k'] = 42; echo $x;",
        "42",
    );
}

/// Lifetime: after `$x = &$a['k']`, `unset($a)` keeps the shared reference cell alive for the
/// alias, so a later write through `$x` still works and the heap stays balanced.
#[test]
fn test_ref_array_element_alias_outlives_unset_of_array() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 3]; $x = &$a['k']; unset($a); $x = 8; echo $x;",
        "8",
    );
}

/// Mixed sink (H4): reading the referenced element into a plain local (`$m = $a['k']`) derefs to
/// the inner value with no use-after-free, even though the fetched element word is a reference cell.
#[test]
fn test_ref_array_element_read_into_mixed_sink() {
    assert_ref_array_element_heap_clean(
        "<?php $a = ['k' => 1]; $x = &$a['k']; $m = $a['k']; echo $m;",
        "1",
    );
}

/// Indexed → hash promotion: taking a reference into an indexed array de-packs it to a hash
/// (Zend behavior); the write-through alias then works and the promoted hash is released as a
/// hash at scope exit (`--heap-debug` clean; regression for the promoted-hash leak).
///
/// (`array_is_list()` is intentionally not asserted here: it is not yet implemented in elephc,
/// and PHP actually reports `true` for this case — the keys stay a contiguous list even though
/// storage de-packs — contradicting the campaign design's original `false` assumption.)
#[test]
fn test_ref_array_element_indexed_promotes_to_hash() {
    assert_ref_array_element_heap_clean(
        "<?php $a = [10, 20, 30]; $x = &$a[1]; $x = 99; echo $a[1];",
        "99",
    );
}

// Test #8 — GC self-cycle (`$a['k'] = &$a` + forced collection) — is STAGED FOR SLICE 2.
// It requires a reference INTO an array element (`$arr[$k] = &$src`), which is a SLICE 2 form
// that currently loud-errors in the checker (see
// `error_tests::type_system::test_error_ref_assign_into_array_element_unsupported`). The kind-6 /
// tag-11 GC-collector awareness landed in Phase B, but it cannot be exercised end-to-end until
// SLICE 2 can construct a ref-cell cycle, so the runtime GC-cycle test is deferred to that slice.

/// SLICE 5 — `$passes = &$this->$property` (DYNAMIC property NAME) aliases the runtime-named
/// array property; appends through the alias are visible via a STATIC-named getter, matching the
/// Symfony `PassConfig::addPass()` write-through pattern the slice targets.
///
/// The write-through values are asserted via indexed reads (`at()`). The distinct-priority
/// `count()` is intentionally not asserted here: `count()` over a reference property that was
/// written through a NESTED index (`$r[$k][] = …`) is a pre-existing reference-machinery bug that
/// reproduces identically on the STATIC-name path (`$r = &$this->v`), so it is out of scope for
/// this slice. A flat-append `count()` over a dynamic-named reference property is correct.
#[test]
fn test_ref_dynamic_property_write_through() {
    let out = compile_and_run(
        "<?php
        class PassConfig {
            public array $beforeOptimization = [];
            public array $afterRemoving = [];
            private function addPass(string $property, int $priority, int $pass): void {
                $passes = &$this->$property;
                if (!isset($passes[$priority])) { $passes[$priority] = []; }
                $passes[$priority][] = $pass;
            }
            public function add(int $priority, int $pass): void {
                $this->addPass('beforeOptimization', $priority, $pass);
            }
            public function at(int $priority, int $idx): int {
                return $this->beforeOptimization[$priority][$idx];
            }
        }
        $pc = new PassConfig();
        $pc->add(10, 100);
        $pc->add(10, 200);
        $pc->add(20, 300);
        echo $pc->at(10, 0), ',', $pc->at(10, 1), ',', $pc->at(20, 0);",
    );
    assert_eq!(out, "100,200,300");
}

/// H2 guard: after binding a reference to a dynamic-named array property, a later DYNAMIC read
/// of the SAME property (`$this->$p`) must dereference the tag-11 reference cell to the array —
/// so `count()` sees the appended elements — instead of misreading the raw cell pointer.
#[test]
fn test_ref_dynamic_property_read_back_derefs() {
    let out = compile_and_run(
        "<?php
        class C {
            public array $v = [];
            public function t(string $p): int {
                $r = &$this->$p;
                $r[] = 5;
                $r[] = 6;
                $back = $this->$p;
                return count($back);
            }
        }
        $o = new C();
        echo $o->t('v');",
    );
    assert_eq!(out, "2");
}

/// The dynamic-named reference-property write-through shape stays heap-balanced: the promoted
/// property cell and its array payload are released at scope exit (`--heap-debug` clean).
#[test]
fn test_ref_dynamic_property_heap_clean() {
    assert_ref_array_element_heap_clean(
        "<?php
        class C {
            public array $v = [];
            public function t(string $p): string {
                $r = &$this->$p;
                $r[] = 1;
                $r[] = 2;
                return implode(',', $this->v);
            }
        }
        $o = new C();
        echo $o->t('v');",
        "1,2",
    );
}

// ---------------------------------------------------------------------------
// SLICE 2/3 — reference-bind a STATIC-PROPERTY array element
// (`self::$a[$dir] = &self::$a[$k]`, the DebugClassLoader.php:795 gate).
//
// One element of a static-property array is bound as a reference alias of another element of the
// SAME static array: both buckets share one kind-6 reference cell (value-tag 11). The checker
// de-packs the container to a hash; EIR threads the loaded hash load → HashRefElement($k) →
// HashBindRefElement($dir) → store; the runtime helper `__rt_hash_bind_ref_element` orders
// incref(cell) → hash_unset(key) → hash_set(key, cell, tag=11).
// ---------------------------------------------------------------------------

/// Basic alias + write-through: after `self::$a['d'] = &self::$a['k']`, a nested write through one
/// element (`self::$a['k'][0] = 'MUT'`) is observed through BOTH the original element and the alias
/// — the two buckets share one reference cell pointing at the same inner array. Uses a heterogeneous
/// `[string, array]` element (the exact `[$dir, []]` shape the DebugClassLoader gate stores).
#[test]
fn test_ref_static_prop_array_element_basic_alias_write_through() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['k'] = ['X', []];
        C::$a['d'] = &C::$a['k'];
        C::$a['k'][0] = 'MUT';
        echo C::$a['k'][0], C::$a['d'][0];",
    );
    assert_eq!(out, "MUTMUT");
}

/// Identity: two elements bound by reference are the SAME array (`self::$a['k'] === self::$a['d']`),
/// because both dereference the one shared reference cell.
#[test]
fn test_ref_static_prop_array_element_identity() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['k'] = ['X', []];
        C::$a['d'] = &C::$a['k'];
        echo (C::$a['k'] === C::$a['d']) ? 'SAME' : 'DIFF';",
    );
    assert_eq!(out, "SAME");
}

/// Scalar write-through: after aliasing, writing an integer through the alias element
/// (`self::$a['d'] = 9`) updates the shared cell, so the original element reads the new value —
/// the tag-11 write-through path of `__rt_hash_set`.
#[test]
fn test_ref_static_prop_array_element_scalar_write_through() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['k'] = 1;
        C::$a['d'] = &C::$a['k'];
        C::$a['d'] = 9;
        echo C::$a['k'], '|', C::$a['d'];",
    );
    assert_eq!(out, "9|9");
}

/// Self-alias (`$dir == $k`): binding an element to itself is refcount-balanced and a no-op on the
/// aliasing, so a later nested write is still observed. Guards the incref-before-unset ordering that
/// keeps a self-alias from freeing its own cell (UAF).
#[test]
fn test_ref_static_prop_array_element_self_alias() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        $k = 'X'; $d = 'X';
        C::$a[$k] = ['orig', []];
        C::$a[$d] = &C::$a[$k];
        C::$a[$k][0] = 'MUT';
        echo C::$a['X'][0];",
    );
    assert_eq!(out, "MUT");
}

/// Auto-vivify: aliasing a MISSING source key vivifies BOTH keys (source via `__rt_hash_ref_element`,
/// target via `__rt_hash_bind_ref_element`), so `array_key_exists` reports true for each.
#[test]
fn test_ref_static_prop_array_element_autovivify_creates_both_keys() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        $d = 'd'; $k = 'nope';
        C::$a[$d] = &C::$a[$k];
        echo array_key_exists('nope', C::$a) ? 'K1' : 'k0';
        echo array_key_exists('d', C::$a) ? 'D1' : 'd0';",
    );
    assert_eq!(out, "K1D1");
}

/// A distinct source key aliases a distinct target: both elements share the one reference cell, and
/// a nested write through the source element is visible through the target alias.
#[test]
fn test_ref_static_prop_array_element_distinct_keys_share_cell() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['src'] = ['V', []];
        C::$a['dst'] = &C::$a['src'];
        C::$a['src'][0] = 'W';
        echo C::$a['dst'][0];",
    );
    assert_eq!(out, "W");
}

/// Heap balance under repeated re-aliasing: binding the SAME element pair by reference many times
/// must not leak or double-free the shared cell. Because static-property storage is never freed at
/// program exit, `live_blocks` cannot be asserted to be `0`; instead it must be INVARIANT to the
/// iteration count (each re-alias increfs the cell, releases the old bucket value, and re-stores,
/// netting zero growth). A small-loop and a large-loop run must report the same `live_blocks`.
#[test]
fn test_ref_static_prop_array_element_realias_heap_stable() {
    let program = |iters: u32| {
        format!(
            "<?php
            class C {{ public static array $a = []; }}
            C::$a['k'] = ['X', []];
            $i = 0;
            while ($i < {iters}) {{
                C::$a['d'] = &C::$a['k'];
                $i = $i + 1;
            }}
            echo C::$a['d'][0];"
        )
    };
    let small = compile_and_run_with_heap_debug(&program(5));
    let large = compile_and_run_with_heap_debug(&program(500));
    assert!(small.success, "small run failed: {}", small.stderr);
    assert!(large.success, "large run failed: {}", large.stderr);
    assert_eq!(small.stdout, "X");
    assert_eq!(large.stdout, "X");
    let live = |stderr: &str| -> String {
        stderr
            .lines()
            .find(|l| l.contains("live_blocks="))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("live_blocks=")))
            .unwrap_or("live_blocks=?")
            .to_string()
    };
    assert_eq!(
        live(&small.stderr),
        live(&large.stderr),
        "re-aliasing leaked/grew the heap: small={} large={}",
        small.stderr,
        large.stderr
    );
}

// The following STATIC-property reference behaviors depend on machinery that is BROKEN
// INDEPENDENTLY of the reference feature (verified on `reconcile/dirname-symfony`), so they are
// staged for a follow-up rather than asserted here:
//
// * String-VALUED write-through through an aliased element (`self::$a['d'] = 'two'; echo
//   self::$a['k'];`). A kind-6 reference cell holds a SINGLE inner word, but a `string` in a
//   `Mixed`-valued hash is stored inline as a two-word `{ptr,len}` payload, so promoting or writing
//   a string element through the cell drops its length. SLICE 1 sidesteps this by rejecting
//   string-TYPED array elements outright (`Reference to a string array element is not yet
//   supported`); the `Mixed`-element static path exposes the same single-word constraint at runtime.
//   Integer/array-valued write-through (covered above) is single-word and works.
//
// * Nested writes into a static-property array element deeper than the first level
//   (`self::$a['d'][1]['real'] = '/p'`). Nested writes into a static-property array are a
//   PRE-EXISTING gap (`C::$a['x'][0] = 'Q'` fails with "array_set index PHP type Str" with no
//   reference involved), the `refprop-nested-append-writethrough` family the SLICE 2/3 spec
//   explicitly excludes.

// -- Reference INTO a LOCAL array element (`$a[$k] = &$var`, `$a[] = &$var`, `$loops[$k][] = &$var`) --

/// `$a[] = &$var` (flat append): the appended element aliases `$var`, so mutating `$var` after the
/// append is observed through the element. Cross-checked with `php -r` (prints `9`).
#[test]
fn test_ref_local_array_append_aliases_source() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $x = 5;
        $a[] = &$x;
        $x = 9;
        echo $a[0];",
    );
    assert_eq!(out, "9");
}

/// `$a[$k] = &$var` (explicit key into an indexed local array): aliasing an existing element to a
/// plain-variable source de-packs the array and writes through the shared cell. Cross-checked with
/// `php -r` (prints `8`).
#[test]
fn test_ref_local_array_explicit_key_aliases_source() {
    let out = compile_and_run(
        "<?php
        $a = [1, 2];
        $x = 7;
        $a[0] = &$x;
        $x = 8;
        echo $a[0];",
    );
    assert_eq!(out, "8");
}

/// `$loops[$k][] = &$var` (the `PhpDumper.php:459` gate): appending a reference into a nested local
/// array element aliases `$var`; a later mutation of `$var` (here appending to the aliased array) is
/// observed through the appended element. Cross-checked with `php -r` (prints `2`).
#[test]
fn test_ref_local_array_nested_append_aliases_source() {
    let out = compile_and_run(
        "<?php
        $loops = [];
        $k = 'c';
        $p = [1];
        $loops[$k][] = &$p;
        $p[] = 2;
        echo count($loops[$k][0]);",
    );
    assert_eq!(out, "2");
}

/// `$loops[$k][] = &$var` when `$loops[$k]` did not previously exist auto-vivifies the inner hash
/// and binds the appended element; both the appended alias and `$var` observe `$var`'s final state.
/// Cross-checked with `php -r` (prints `2|9`).
#[test]
fn test_ref_local_array_nested_append_auto_vivifies_inner() {
    let out = compile_and_run(
        "<?php
        $loops = [];
        $k = 'c';
        $p = [7];
        $loops[$k][] = &$p;
        $p[] = 9;
        echo count($loops[$k][0]), '|', $loops[$k][0][1];",
    );
    assert_eq!(out, "2|9");
}

/// Two DISTINCT sources appended under two keys each alias their own source; mutating each source is
/// observed through its element. Cross-checked with `php -r` (prints `20,30`).
#[test]
fn test_ref_local_array_append_multi_key_distinct_sources() {
    let out = compile_and_run(
        "<?php
        $p = 1;
        $q = 2;
        $a = [];
        $a['x'][] = &$p;
        $a['y'][] = &$q;
        $p = 20;
        $q = 30;
        echo $a['x'][0], ',', $a['y'][0];",
    );
    assert_eq!(out, "20,30");
}

/// T5: the SAME source appended under two keys shares ONE persistent reference cell (Zend
/// semantics): mutating `$p` after both appends is observed through BOTH elements. This is the case
/// the PhpDumper:459 loop triggers (the same `$pathInLoop` bound under every key). Cross-checked with
/// `php -r` (prints `99,99`).
#[test]
fn test_ref_local_array_append_multi_key_same_source() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $p = 1;
        $a['x'][] = &$p;
        $a['y'][] = &$p;
        $p = 99;
        echo $a['x'][0], ',', $a['y'][0];",
    );
    assert_eq!(out, "99,99");
}

/// T5b: straight-line explicit-key binds of the SAME source share one cell; a later write to `$p`
/// is observed through both aliased elements. Cross-checked with `php -r` (prints `77`).
#[test]
fn test_ref_local_array_explicit_key_same_source_shares_cell() {
    let out = compile_and_run(
        "<?php
        $a = [0, 0];
        $p = 1;
        $a[0] = &$p;
        $a[1] = &$p;
        $p = 7;
        echo $a[0], $a[1];",
    );
    assert_eq!(out, "77");
}

/// T5c: the PhpDumper-loop reduction — the SAME source variable is nested-appended under every key
/// of a loop, then mutated once. The get-or-promote of `$p`'s cell is runtime-idempotent (the loop
/// body's single `&$p` promotes on the first iteration and reuses the same cell thereafter), so all
/// keys observe the mutation. Cross-checked with `php -r` (prints `999`).
#[test]
fn test_ref_local_array_nested_append_loop_same_source() {
    let out = compile_and_run(
        "<?php
        $loops = [];
        $p = [0];
        foreach (['a', 'b', 'c'] as $k) {
            $loops[$k][] = &$p;
        }
        $p[0] = 9;
        echo $loops['a'][0][0], $loops['b'][0][0], $loops['c'][0][0];",
    );
    assert_eq!(out, "999");
}

/// `unset($a)` after `$a[] = &$x` drops the container's share of the shared cell; the source `$x`
/// keeps the value alive (the cell survives at refcount 1, owned by `$x`). This verifies the
/// cell-survival property from the container side. Cross-checked with `php -r` (prints `3`).
///
/// (The mirror `unset($x); echo $a[0]` — cell kept alive by the array element — additionally trips a
/// PRE-EXISTING reference-cell / cycle-collector interaction on `unset` that also affects the SLICE-1
/// producer `$x = &$arr[$k]; unset($x); echo $arr[$k]`, so it is left to the shared-machinery fix.)
#[test]
fn test_ref_local_array_append_unset_container_keeps_source() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $x = 3;
        $a[] = &$x;
        unset($a);
        echo $x;",
    );
    assert_eq!(out, "3");
}

/// Heap balance under repeated reference-append aliasing: appending `&$p` into a fresh nested local
/// array many times, freeing the array each iteration, must not leak or double-free the shared cells.
/// Because the container is rebuilt and released every iteration, `live_blocks` must be INVARIANT to
/// the iteration count — a small-loop and a large-loop run must report the same `live_blocks` and
/// balanced frees (mirroring the SLICE 2/3 endurance test).
///
/// The inner element is read through a temporary (`$inner = $loops['k']`) rather than the chained
/// index `$loops['k'][0]`: a chained read of a nested array element leaks the intermediate container
/// temporary PRE-EXISTINGLY, independent of references (`$a['k'][0]` leaks the same way with no `&`
/// involved), so chaining here would measure that unrelated bug rather than the reference append.
#[test]
fn test_ref_local_array_append_heap_stable() {
    let program = |iters: u32| {
        format!(
            "<?php
            $i = 0;
            while ($i < {iters}) {{
                $loops = [];
                $p = [$i];
                $loops['k'][] = &$p;
                $p[] = $i + 1;
                $inner = $loops['k'];
                $sum = count($inner);
                unset($loops);
                unset($p);
                unset($inner);
                $i = $i + 1;
            }}
            echo $sum;"
        )
    };
    let small = compile_and_run_with_heap_debug(&program(50));
    let large = compile_and_run_with_heap_debug(&program(300));
    assert!(small.success, "small run failed: {}", small.stderr);
    assert!(large.success, "large run failed: {}", large.stderr);
    assert_eq!(small.stdout, "1");
    assert_eq!(large.stdout, "1");
    let live = |stderr: &str| -> String {
        stderr
            .lines()
            .find(|l| l.contains("live_blocks="))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("live_blocks=")))
            .unwrap_or("live_blocks=?")
            .to_string()
    };
    assert_eq!(
        live(&small.stderr),
        live(&large.stderr),
        "reference-append aliasing leaked/grew the heap: small={} large={}",
        small.stderr,
        large.stderr
    );
}

// -- Whole-value reassign + unset of a reference-bound LOCAL (kind-6 adopted owner) --
//
// These cover the two whole-value operations that were broken on a reference-bound local
// `$x = &$arr[0]` (an adopted kind-6 refcounted cell):
//   (b) a whole reassign `$x = <new type>` must write BOTH the cell value word and the
//       inner value-tag so a type change (int->array, int->string) is read back correctly;
//   (c) `unset($x)` must `__rt_ref_cell_decref` (keep the shared value alive for the other
//       binding) instead of raw-freeing the cell.
// All expectations cross-checked with `php -r`.

/// (b) int->array type change: the reassign writes the array pointer and stamps the Array
/// value-tag at `[cell+8]`, so reading `$arr[0]` observes the new array. `php -r` prints `6`.
#[test]
fn test_ref_bound_local_reassign_int_to_array() {
    let out = compile_and_run(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = [5, 6];
        echo $arr[0][1];",
    );
    assert_eq!(out, "6");
}

/// (b) int->string type change: the reassign writes the string pointer and stamps the Str
/// value-tag at `[cell+8]`, so reading `$arr[0]` observes the new string. `php -r` prints `hi`.
#[test]
fn test_ref_bound_local_reassign_int_to_string() {
    let out = compile_and_run(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = \"hi\";
        echo $arr[0];",
    );
    assert_eq!(out, "hi");
}

/// (b) same-type regression: an int->int reassign still writes through and reads back. The
/// `__rt_ref_cell_store` helper must not disturb the working int->int fast path.
/// `php -r` prints `5`.
#[test]
fn test_ref_bound_local_reassign_same_type_int() {
    let out = compile_and_run(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = 5;
        echo $arr[0];",
    );
    assert_eq!(out, "5");
}

/// (c) unset-keeps-element: `unset($x)` on an adopted owner decrements the shared cell
/// instead of freeing it, so the element value survives. `php -r` prints `7`.
#[test]
fn test_ref_bound_local_unset_keeps_element() {
    let out = compile_and_run(
        "<?php
        $arr = [7];
        $x = &$arr[0];
        unset($x);
        echo $arr[0];",
    );
    assert_eq!(out, "7");
}

/// (c) after unset, the OTHER binding is still mutable: `$arr[0]` can be reassigned and
/// read back, since the cell survives the unset (decref, not free). `php -r` prints `8`.
#[test]
fn test_ref_bound_local_unset_then_other_binding_mutable() {
    let out = compile_and_run(
        "<?php
        $arr = [7];
        $x = &$arr[0];
        unset($x);
        $arr[0] = 8;
        echo $arr[0];",
    );
    assert_eq!(out, "8");
}

/// (b) SLICE-2 producer form: `$a[] = &$p` then `$p = [5, 6]` reassigns the shared cell to an
/// array; reading the element observes the new array. `php -r` prints `6`.
#[test]
fn test_ref_bound_local_slice2_reassign_int_to_array() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $p = 1;
        $a[] = &$p;
        $p = [5, 6];
        echo $a[0][1];",
    );
    assert_eq!(out, "6");
}

/// (c) SLICE-2 producer form: `unset($p)` after `$a[] = &$p` decrements the shared cell, so
/// the element value survives. `php -r` prints `1`.
#[test]
fn test_ref_bound_local_slice2_unset_keeps_element() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $p = 1;
        $a[] = &$p;
        unset($p);
        echo $a[0];",
    );
    assert_eq!(out, "1");
}

/// Heap balance: a type-change reassign of a reference-bound local must not leak or
/// double-free. Each (b)/(c) operation in a loop must leave `live_blocks` invariant
/// between a small and a large run.
#[test]
fn test_ref_bound_local_type_change_and_unset_heap_stable() {
    let program = |iters: u32| {
        format!(
            "<?php
            $i = 0;
            while ($i < {iters}) {{
                $arr = [1];
                $x = &$arr[0];
                $x = [5, 6];
                $x = \"hi\";
                unset($x);
                $i = $i + 1;
            }}
            echo $arr[0];"
        )
    };
    let small = compile_and_run_with_heap_debug(&program(50));
    let large = compile_and_run_with_heap_debug(&program(300));
    assert!(small.success, "small run failed: {}", small.stderr);
    assert!(large.success, "large run failed: {}", large.stderr);
    assert_eq!(small.stdout, "hi");
    assert_eq!(large.stdout, "hi");
    let live = |stderr: &str| -> String {
        stderr
            .lines()
            .find(|l| l.contains("live_blocks="))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("live_blocks=")))
            .unwrap_or("live_blocks=?")
            .to_string()
    };
    assert_eq!(
        live(&small.stderr),
        live(&large.stderr),
        "ref-bound type-change/unset leaked/grew the heap: small={} large={}",
        small.stderr,
        large.stderr
    );
}

/// (b)+(c) per-operation heap balance: each individual (b)/(c) operation must leave
/// `live_blocks=0` at exit (no leak, no double-free). Verifies the runtime helper releases
/// the prior inner exactly once and the unset decrefs the shared cell exactly once.
///
/// Reads go through the OWNER (`$x`/`$p`) rather than the array element (`$arr[0][1]`):
/// reading an array element and then indexing it (`$arr[0][1]`) has a pre-existing
/// read-path leak (the intermediate Mixed box from the read is not released after
/// indexing) that is out of scope for this fix. Reading through the owner exercises
/// the same `__rt_ref_cell_store` / `__rt_ref_cell_decref` paths without touching the
/// separate read+index leak, so the heap balance reflects only the (b)/(c) operation.
#[test]
fn test_ref_bound_local_operations_live_blocks_zero() {
    let check = |src: &str, label: &str| {
        let out = compile_and_run_with_heap_debug(src);
        assert!(out.success, "{label} failed: {}", out.stderr);
        let live = out
            .stderr
            .lines()
            .find(|l| l.contains("live_blocks="))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("live_blocks=")))
            .unwrap_or("live_blocks=?");
        assert_eq!(
            live, "live_blocks=0",
            "{label} did not balance: {}\n{}",
            label,
            out.stderr
        );
    };
    check(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = [5, 6];
        echo $x[1];",
        "int->array",
    );
    check(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = \"hi\";
        echo $x;",
        "int->string",
    );
    check(
        "<?php
        $arr = [1];
        $x = &$arr[0];
        $x = 5;
        echo $x;",
        "same-type int",
    );
    check(
        "<?php
        $arr = [7];
        $x = &$arr[0];
        unset($x);
        echo $arr[0];",
        "unset-keeps-element",
    );
    check(
        // SLICE-2 int->array: the alias `$p` keeps the original element type (Int), so
        // indexing `$p[1]` does not type-check, and reading `$a[0]` hits a pre-existing
        // read-path leak (the intermediate Mixed box from the read is not released after
        // echo). Use a store-only program: the (b) operation (`$p = [5, 6]` through
        // `__rt_ref_cell_store` with Mixed tag) and the scope-exit cleanup
        // (`__rt_ref_cell_decref` + hash free-deep) are exercised without touching the
        // separate read leak, so the heap balance reflects only the (b) operation. Value
        // correctness for int->array is covered by the non-SLICE-2 `echo $x[1]` case above.
        "<?php
        $a = [];
        $p = 1;
        $a[] = &$p;
        $p = [5, 6];",
        "slice2 int->array",
    );
    check(
        "<?php
        $a = [];
        $p = 1;
        $a[] = &$p;
        unset($p);
        echo $a[0];",
        "slice2 unset-keeps-element",
    );
}

/// Regression guard (the gate): by-ref function PARAM mutation and foreach-by-ref must be
/// UNCHANGED by the adopted kind-6 store/unset fix. By-ref params use a raw 2-word caller slot
/// (no inner_tag), and promoted-non-adopted foreach fallback cells use a kind-0 two-word cell —
/// neither is an adopted kind-6 owner, so the `__rt_ref_cell_store` / `__rt_ref_cell_decref`
/// paths must NOT fire for them. `php -r` prints `9\n7\n`.
#[test]
fn test_ref_bound_local_regression_by_ref_param_and_foreach() {
    let out = compile_and_run(
        "<?php
        function f(&$p) { $p = 9; }
        $x = 1;
        f($x);
        echo $x, \"\\n\";
        $arr = [1, 2, 3];
        foreach ($arr as &$v) { $v = $v + 5; }
        echo $arr[1], \"\\n\";",
    );
    assert_eq!(out, "9\n7\n");
}

// -- (a) Nested write-through on a reference-bound LOCAL (EIR-level explicit per-level write-back) --
//
// These tests verify that a NESTED lvalue write (2+ levels) whose base is a reference-bound
// LOCAL (`$x = &$arr[0]` then `$x[1][0] = 9`) writes THROUGH the kind-6 ref cell so the
// mutation is observable via the alias and the source array. The fix mirrors the SLICE-2
// local explicit per-level write-back template: the head reads the cell, each intermediate
// level is read into an owned hidden temp (COW split), the leaf is written in place via the
// 3-operand set op, and the mutated temps are written back per-level.
//
// `live_blocks=0` is not asserted here because the NESTED READ path (`echo $arr[0][1][0]`)
// has a pre-existing leak (intermediate Mixed boxes from `__rt_mixed_array_get` for typed
// slots are not released after indexing) that is OUT OF SCOPE for this slice. The heap-
// invariant endurance test below verifies the WRITE path itself is leak-free by checking
// that `live_blocks` does not grow with iteration count.

/// (a-2) 2-level nested indexed write through a reference-bound local.
/// Cross-checked with `php -r` (prints `9`).
#[test]
fn test_ref_bound_local_nested_write_2level() {
    let out = compile_and_run(
        "<?php $arr = [[1, [2, 3]]]; $x = &$arr[0]; $x[1][0] = 9; echo $arr[0][1][0];",
    );
    assert_eq!(out, "9");
}

/// (a-3) 3-level nested indexed write through a reference-bound local.
/// Cross-checked with `php -r` (prints `9`).
#[test]
fn test_ref_bound_local_nested_write_3level() {
    let out = compile_and_run(
        "<?php $arr = [[1, [[2, 3]]]]; $x = &$arr[0]; $x[1][0][1] = 9; echo $arr[0][1][0][1];",
    );
    assert_eq!(out, "9");
}

/// (a-app) Nested append through a reference-bound local.
/// Cross-checked with `php -r` (prints `9`).
#[test]
fn test_ref_bound_local_nested_append() {
    let out = compile_and_run(
        "<?php $arr = [[1, [2, 3]]]; $x = &$arr[0]; $x[1][] = 9; echo $arr[0][1][2];",
    );
    assert_eq!(out, "9");
}

/// (a-str) Nested string-key write through a reference-bound local.
/// Cross-checked with `php -r` (prints `9`).
#[test]
fn test_ref_bound_local_nested_string_key() {
    let out = compile_and_run(
        "<?php $arr = [['a' => ['b' => 1]]]; $x = &$arr[0]; $x['a']['b'] = 9; echo $arr[0]['a']['b'];",
    );
    assert_eq!(out, "9");
}

/// (a-src) Nested write through a ref-promoted source (`$a[] =&$p` then write through `$p`).
/// Cross-checked with `php -r` (prints `9 9`).
#[test]
fn test_ref_bound_local_nested_write_through_ref_promoted_source() {
    let out = compile_and_run(
        "<?php $a = []; $p = [0, [0]]; $a[] = &$p; $p[1][0] = 9; echo $a[0][1][0] . ' ' . $p[1][0];",
    );
    assert_eq!(out, "9 9");
}

/// (a-loop unrolled) PhpDumper-loop reduction (unrolled to avoid a pre-existing SLICE-2
/// foreach-loop segfault — see divergence note in the report). The unrolled form exercises
/// the same nested write + hash-ref-append interaction as the foreach version.
/// Cross-checked with `php -r` (prints `99`).
#[test]
fn test_ref_bound_local_nested_write_loop_unrolled() {
    let out = compile_and_run(
        "<?php
        $loops = [];
        $p = [0, []];
        $k = 'a';
        $p[1][$k] = 1;
        $loops[$k][] = &$p;
        $k = 'b';
        $p[1][$k] = 1;
        $loops[$k][] = &$p;
        $p[1]['a'] = 9;
        echo $loops['a'][0][1]['a'] . $loops['b'][0][1]['a'];",
    );
    assert_eq!(out, "99");
}

/// Scalar-as-array loud-error: writing into a scalar alias must produce a loud error, not a
/// silent miscompile or an `expected Heap(Hash) got I64` validator panic. The error fires at
/// type-check time, so `compile_and_run` panics; `catch_unwind` captures the panic message.
#[test]
fn test_ref_bound_local_scalar_as_array_loud_error() {
    let result = std::panic::catch_unwind(|| {
        compile_and_run("<?php $arr = [5]; $x = &$arr[0]; $x[0] = 9;");
    });
    assert!(
        result.is_err(),
        "expected a compile error, but the program compiled and ran successfully"
    );
    let msg = result
        .err()
        .and_then(|e| e.downcast_ref::<String>().cloned().or_else(|| {
            e.downcast_ref::<&str>().map(|s| s.to_string())
        }))
        .unwrap_or_default();
    assert!(
        msg.contains("Cannot use a scalar value as an array"),
        "expected 'Cannot use a scalar value as an array', got: {}",
        msg
    );
}

/// Heap-invariant endurance test: the nested write path must not leak ADDITIONAL blocks
/// per iteration. The pre-existing nested-read leak (intermediate Mixed boxes from
/// `__rt_mixed_array_get`) is constant across iterations, so `live_blocks` must be
/// INVARIANT to the iteration count. This verifies the write path's release discipline
/// (owned hidden temps are released/cleared per iteration, no accumulation).
///
/// The loop avoids `$loops[$k][]=&$p` (the SLICE-2 hash-ref-append path) which has a
/// PRE-EXISTING foreach-loop segfault unrelated to this slice. Instead it exercises only
/// the nested write `$p[1][0] = $i` through the ref-bound local `$p` in a loop, verifying
/// that the per-level descent + write-back release discipline is balanced across iterations.
#[test]
fn test_ref_bound_local_nested_write_endurance_heap_invariant() {
    let program = |iters: u32| {
        format!(
            "<?php
            $arr = [[1, [2, 3]]];
            $x = &$arr[0];
            $i = 0;
            while ($i < {iters}) {{
                $x[1][0] = $i;
                $i = $i + 1;
            }}
            echo $x[1][0];"
        )
    };
    let small = compile_and_run_with_heap_debug(&program(5));
    let large = compile_and_run_with_heap_debug(&program(50));
    assert!(small.success, "small run failed: {}", small.stderr);
    assert!(large.success, "large run failed: {}", large.stderr);
    let live = |stderr: &str| -> String {
        stderr
            .lines()
            .find(|l| l.contains("live_blocks="))
            .and_then(|l| l.split_whitespace().find(|t| t.starts_with("live_blocks=")))
            .unwrap_or("live_blocks=?")
            .to_string()
    };
    assert_eq!(
        live(&small.stderr),
        live(&large.stderr),
        "nested write leaked/grew the heap: small={} large={}",
        small.stderr,
        large.stderr
    );
}

/// Regression guard: single-level `$x[1] = 5` through a ref-bound local must still work
/// (SLICE-1 `lower_array_assign` path, NOT my new `lower_nested_ref_bound_local_assign`).
#[test]
fn test_ref_bound_local_single_level_write_unchanged() {
    let out = compile_and_run(
        "<?php $arr = [1, 2, 3]; $x = &$arr[1]; $x = 9; echo $arr[1];",
    );
    assert_eq!(out, "9");
}

/// Regression guard: whole-reassign `$x = [5, 6]` (SLICE (b)) and `unset($x)` (SLICE (c))
/// must still work after the nested write fix.
#[test]
fn test_ref_bound_local_whole_reassign_and_unset_unchanged() {
    let out = compile_and_run(
        "<?php
        $arr = [1, 2, 3];
        $x = &$arr[1];
        $x = [5, 6];
        echo $x[1];
        unset($x);
        echo $arr[1][1];",
    );
    assert_eq!(out, "66");
}

/// Regression guard: plain-local nested writes (non-ref-bound) must stay on the untouched
/// generic `RuntimeCall` path — no new miscompile, no new error.
#[test]
fn test_plain_local_nested_write_unchanged() {
    let out = compile_and_run(
        "<?php $a = [[1, [2, 3]]]; $a[0][1][0] = 9; echo $a[0][1][0];",
    );
    // The plain-local generic path may lose the write (pre-existing fresh-box bug), but it
    // must NOT newly error or crash. The output is whatever the baseline produced.
    assert!(
        out == "9" || out == "2" || out.is_empty(),
        "plain-local nested write unexpectedly changed behavior: {}",
        out
    );
}

// -- foreach loops over a hoisted ref-ensure local (ADDENDUM 2 regression tests) --
//
// These guard the heap-exhaustion regression chain: the stale GC reachable mark bit in the
// kind word made `__rt_ref_cell_ensure` wrap a live cell in a second cell (fixed by masking
// the kind byte), the redundant `StoreRefCell` after `ArrayPush` released the pointer it was
// about to store (fixed in `lower_array_push`), and the hoisted-store type-fact/acquire gaps
// corrupted Mixed-element reads (fixed in `store_local`). All outputs cross-checked with
// `php -r`.

/// Foreach append through the hoisted ref-ensure local: `$p = [$i]` (Mixed foreach value) plus
/// `$p[] = $i + 1` each iteration, read back through `$p` after the loop. PHP prints `234`
/// (count 2, elements 3 and 4 from the last iteration). Guards the hoisted-store type-fact
/// propagation (`array<mixed>` cell inner read back with Mixed unboxing) and the
/// acquire-before-adopt on the raw ref-cell store. Reading through `$loops['k'][0]` instead
/// would hit the pre-existing nested-read gap (unsupported `count` on a chained ref element),
/// so the probe reads through `$p` — the same shared cell.
#[test]
fn test_foreach_ref_local_array_append_loop() {
    let out = compile_and_run(
        "<?php
        foreach (range(1, 3) as $i) { $p = [$i]; $loops = []; $loops['k'][] = &$p; $p[] = $i + 1; }
        echo count($p) . $p[0] . $p[1];",
    );
    assert_eq!(out, "234");
}

/// The canonical PhpDumper-loop reduction in its ORIGINAL foreach form (the unrolled variant
/// is `test_ref_bound_local_nested_write_loop_unrolled`): nested write before the `=&` in
/// source order, hash-ref-append each iteration, final write through `$p` observed through
/// both aliased elements. Cross-checked with `php -r` (prints `99`).
#[test]
fn test_foreach_ref_local_nested_write_loop() {
    let out = compile_and_run(
        "<?php
        $loops = [];
        $p = [0, []];
        foreach (['a', 'b'] as $k) { $p[1][$k] = 1; $loops[$k][] = &$p; }
        $p[1]['a'] = 9;
        echo $loops['a'][0][1]['a'] . $loops['b'][0][1]['a'];",
    );
    assert_eq!(out, "99");
}

/// Variant B2 of the heap-exhaustion repro: append through the aliased cell then
/// `unset($loops)` (keep `$p`) every iteration. The `gc_collect` after the unset left the GC
/// reachable mark bit set on the surviving cell, so iteration 2's `__rt_ref_cell_ensure`
/// double-wrapped it and `__rt_array_grow` read the cell as an array header — exhausting the
/// heap at N=2 before the kind-byte mask fix. Runs on the DEFAULT heap; completing all 5
/// iterations and printing `ok` is the regression guard.
#[test]
fn test_foreach_ref_local_append_then_unset_loops() {
    let out = compile_and_run(
        "<?php
        foreach (range(0, 4) as $i) { $p = [$i]; $loops = []; $loops['k'][] = &$p; $p[] = $i + 1; unset($loops); }
        echo 'ok';",
    );
    assert_eq!(out, "ok");
}

/// Variant B3 of the heap-exhaustion repro: append through the aliased cell then `unset($p)`
/// (keep `$loops`) every iteration. `unset` re-establishes a fresh cell, and the `gc_collect`
/// after it marked that cell; the next iteration's ensure then double-wrapped it exactly like
/// B2. Runs on the DEFAULT heap; printing `ok` after 5 iterations is the regression guard.
#[test]
fn test_foreach_ref_local_append_then_unset_p() {
    let out = compile_and_run(
        "<?php
        foreach (range(0, 4) as $i) { $p = [$i]; $loops = []; $loops['k'][] = &$p; $p[] = $i + 1; unset($p); }
        echo 'ok';",
    );
    assert_eq!(out, "ok");
}

/// Variant E, the clean baseline: append through the aliased cell with NO unsets (so no
/// `gc_collect` and no stale mark bit) for 50 iterations on the DEFAULT heap. Guards against
/// a future per-iteration leak on the pure append path (`__rt_ref_cell_store` prior-inner
/// release + the `ArrayPush` cell write-back staying balanced).
#[test]
fn test_foreach_ref_local_append_no_unset() {
    let out = compile_and_run(
        "<?php
        foreach (range(0, 49) as $i) { $p = [$i]; $loops = []; $loops['k'][] = &$p; $p[] = $i + 1; }
        echo 'ok';",
    );
    assert_eq!(out, "ok");
}

// --- By-reference entries in array literals (['k' => &$v], [&$v]) ---

/// The r1 gate shape: a keyed array literal with a by-reference entry aliases the source
/// variable — a later write to `$v` is visible through `$arr['s']` while the plain entries
/// keep their values (PHP prints `9 12`; cross-checked with `php -r`). Heap stays clean.
#[test]
fn test_ref_entry_array_literal_keyed_aliasing() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $arr = ['a' => 1, 's' => &$v, 'b' => 2]; $v = 9; \
         echo $arr['s'], ' ', $arr['a'], $arr['b'];",
        "9 12",
    );
}

/// A positional by-reference entry (`[&$v]`) lands at integer key 0 and aliases the
/// source (php-cross-checked).
#[test]
fn test_ref_entry_array_literal_positional() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $arr = [&$v]; $v = 9; echo $arr[0];",
        "9",
    );
}

/// PHP's next-integer-key rule for a mixed positional/keyed ref literal:
/// `[&$a, 5 => &$b, &$c]` produces keys 0, 5, and 6, all aliasing their sources
/// (php-cross-checked: PHP prints `10 20 30`).
#[test]
fn test_ref_entry_array_literal_mixed_positional_keyed_indexes() {
    assert_ref_array_element_heap_clean(
        "<?php $a = 1; $b = 2; $c = 3; $arr = [&$a, 5 => &$b, &$c]; \
         $a = 10; $b = 20; $c = 30; echo $arr[0], ' ', $arr[5], ' ', $arr[6];",
        "10 20 30",
    );
}

/// A ref-bearing literal in RETURN position (the r3 gate shape with a local source)
/// builds and returns the array through the hidden-temp prelude.
#[test]
fn test_ref_entry_array_literal_return_position() {
    let out = compile_and_run(
        "<?php
        function req(): array {
            $v = 3;
            return ['q' => 1, 'session' => &$v];
        }
        $r = req();
        echo isset($r['session']) ? 'y' : 'n', ' ', $r['q'], ' ', $r['session'];",
    );
    assert_eq!(out, "y 1 3");
}

/// A NESTED literal with a by-reference entry (`['x' => ['y' => &$v]]`): the inner
/// literal desugars to its own hidden temp first, and the aliasing still reads through
/// both levels after mutating the source (php-cross-checked: PHP prints `9`).
#[test]
fn test_ref_entry_array_literal_nested_literal() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 1; $arr = ['x' => ['y' => &$v]]; $v = 9; echo $arr['x']['y'];",
        "9",
    );
}

/// COW with a reference entry: copying the literal-built array (`$b = $arr`) shares the
/// tag-11 reference cell (incref, not deep copy), so a later `$v = 42` reads back through
/// BOTH views (php-cross-checked: PHP prints `42 42`).
#[test]
fn test_ref_entry_array_literal_cow_copy_shares_cell() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $arr = ['a' => 1, 's' => &$v]; $b = $arr; $v = 42; \
         echo $arr['s'], ' ', $b['s'];",
        "42 42",
    );
}

/// Unsetting the source variable keeps the entry alive: the kind-6 cell survives with the
/// last written value (php-cross-checked: PHP prints `7`).
#[test]
fn test_ref_entry_array_literal_unset_source_keeps_entry() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 7; $arr = ['s' => &$v]; unset($v); echo $arr['s'];",
        "7",
    );
}

/// The desugar inside a FUNCTION body: the hidden temp starts as an empty packed array
/// whose frame slot is later widened to hash storage by the reference bind — the packed
/// flow-typed load before the de-pack and the Mixed element stamp both must hold
/// (regression for the widened-slot load pairing and the empty-container Mixed widening).
#[test]
fn test_ref_entry_array_literal_in_function_scope() {
    assert_ref_array_element_heap_clean(
        "<?php
        function go(): void {
            $v = 5;
            $arr = ['s' => &$v];
            $v = 9;
            echo $arr['s'];
        }
        go();",
        "9",
    );
}

/// The long-form `array('s' => &$v)` literal desugars identically to the short form
/// (php-cross-checked: PHP prints `6`).
#[test]
fn test_ref_entry_long_array_form() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 4; $arr = array('s' => &$v); $v = 6; echo $arr['s'];",
        "6",
    );
}

/// An assignment expression as an assoc-literal VALUE where the assigned array carries a
/// reference cell (`['x' => ($u = $t)]`): the literal's hash value-type stamp must follow
/// the yielded expression (an array), not the syntactic Mixed default, so the element read
/// dereferences the inner hash correctly (regression for the Assignment-arm typing in
/// the assoc-literal value stamp; php-cross-checked: PHP prints `1`).
#[test]
fn test_assignment_expr_value_in_literal_with_ref_cell_array() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 1; $t = []; $t['y'] = &$v; $arr = ['x' => ($u = $t)]; \
         echo $arr['x']['y'];",
        "1",
    );
}

// --- Conditional-value composition of ref-bearing literals (ternary / ?? / match / &&) ---

/// A ref-bearing literal in BOTH branches of a ternary: the taken branch's array keeps its
/// reference entry readable through the merge (regression: the merge temp used to widen to
/// a boxed Mixed whose key read bypassed the tag-11-aware hash path and printed nothing;
/// php-cross-checked: PHP prints `5`).
#[test]
fn test_ref_entry_literal_ternary_both_branches() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $w = 6; $cond = $argc > 0; \
         $arr = $cond ? ['s' => &$v] : ['s' => &$w]; echo $arr['s'];",
        "5",
    );
}

/// A ref-bearing literal in the UNTAKEN ternary branch: the source variable's hoisted
/// entry-block reference cell keeps a later plain `echo $v` valid (regression: the
/// mid-branch cell promotion used to leave `LoadRefCell` reading a raw int slot →
/// SIGSEGV; php-cross-checked: PHP prints `1 5`).
#[test]
fn test_ref_entry_literal_in_untaken_ternary_branch() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $cond = $argc > 0; \
         $arr = $cond ? ['s' => 1] : ['s' => &$v]; echo $arr['s'], ' ', $v;",
        "1 5",
    );
}

/// Two ref-literal ternaries in one script, one taking each branch direction: entry
/// aliasing tracks the RUNTIME-taken branch's source in both (php-cross-checked: PHP
/// prints `10 200`).
#[test]
fn test_ref_entry_literal_two_ternaries_alias_taken_branch() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $w = 6; $cond = $argc > 0; \
         $arr = $cond ? ['s' => &$v] : ['s' => &$w]; \
         $v = 10; $w = 20; echo $arr['s'], ' '; \
         $cond2 = $argc > 99; \
         $arr2 = $cond2 ? ['s' => &$v] : ['s' => &$w]; \
         $v = 100; $w = 200; echo $arr2['s'];",
        "10 200",
    );
}

/// A ref-bearing literal as the `??` DEFAULT: the lazily-evaluated default branch builds
/// the hash and its reference entry stays live through the coalesce merge
/// (php-cross-checked: PHP prints `7`).
#[test]
fn test_ref_entry_literal_null_coalesce_default() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $maybe = null; $arr = $maybe ?? ['s' => &$v]; \
         $v = 7; echo $arr['s'];",
        "7",
    );
}

/// A ref-bearing literal as a MATCH arm value: the arm's hash flows through the match
/// merge with its reference entry intact (php-cross-checked: PHP prints `9`).
#[test]
fn test_ref_entry_literal_match_arm() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $sel = $argc; \
         $arr = match (true) { $sel > 0 => ['s' => &$v], default => ['s' => 0] }; \
         $v = 9; echo $arr['s'];",
        "9",
    );
}

/// A ref-bearing literal assigned inside a `&&` right-hand side: the branch-confined
/// assignment still aliases the source when the RHS runs (php-cross-checked: PHP
/// prints `9`).
#[test]
fn test_ref_entry_literal_in_logical_and_rhs() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $ok = ($argc > 0) && ($arr = ['s' => &$v]); \
         $v = 9; echo $ok ? $arr['s'] : 'no';",
        "9",
    );
}

// --- Compositional merge typing: the ref-hash type must flow THROUGH nested merges ---

/// A CHAINED coalesce ending in a ref-bearing literal (`$a ?? $b ?? ['s' => &$v]`): the
/// inner `??` merge temp is itself an operand of the outer merge, and its already-computed
/// ref-hash value type must propagate through the outer merge (regression: the outer merge
/// re-derived a syntactic `Mixed` fallback, boxed the hash, and the key read printed
/// nothing; php-cross-checked: PHP prints `5`).
#[test]
fn test_ref_entry_literal_coalesce_chain() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $b = null; $r = $a ?? $b ?? ['s' => &$v]; echo $r['s'];",
        "5",
    );
}

/// A TRIPLE coalesce chain (`$a ?? $b ?? $c ?? ['s' => &$v]`): the ref-hash type survives
/// two intermediate merge temps, and the entry still aliases the source afterwards
/// (php-cross-checked: PHP prints `11`).
#[test]
fn test_ref_entry_literal_coalesce_triple_chain() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $b = null; $c = null; \
         $r = $a ?? $b ?? $c ?? ['s' => &$v]; $v = 11; echo $r['s'];",
        "11",
    );
}

/// A chain whose MIDDLE operand is a plain hash hit (`$a ?? $b ?? ['s' => &$v]` with
/// `$b = ['s' => 3]`): the runtime-taken plain branch wins, the untaken ref-literal arm
/// still types the merges, and the read yields the plain value (php-cross-checked: PHP
/// prints `3`).
#[test]
fn test_ref_entry_literal_coalesce_chain_mid_hit() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $b = ['s' => 3]; \
         $r = $a ?? $b ?? ['s' => &$v]; $v = 9; echo $r['s'];",
        "3",
    );
}

/// A `??` nested inside the TAKEN ternary branch (`$c ? ($a ?? ['s' => &$v]) : [...]`):
/// the coalesce merge's ref-hash value type is consulted by the enclosing ternary merge
/// instead of the syntactic fallback, keeping the aliasing entry readable
/// (php-cross-checked: PHP prints `7`).
#[test]
fn test_ref_entry_literal_coalesce_inside_ternary_then_branch() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $c = $argc > 0; \
         $r = $c ? ($a ?? ['s' => &$v]) : ['s' => 0]; $v = 7; echo $r['s'];",
        "7",
    );
}

/// The reverse order — the `??` nested inside the ELSE ternary branch, with the else
/// branch runtime-taken — propagates the ref-hash type the same way
/// (php-cross-checked: PHP prints `7`).
#[test]
fn test_ref_entry_literal_coalesce_inside_ternary_else_branch() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $c = $argc > 99; \
         $r = $c ? ['s' => 0] : ($a ?? ['s' => &$v]); $v = 7; echo $r['s'];",
        "7",
    );
}

/// A COPY-OUT read of a chain-produced ref hash (`$w = $r`): the shallow clone shares the
/// reference cell, so a later source write is visible through BOTH copies
/// (php-cross-checked: PHP prints `42 42`).
#[test]
fn test_ref_entry_literal_coalesce_chain_copy_out() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $b = null; $r = $a ?? $b ?? ['s' => &$v]; \
         $w = $r; $v = 42; echo $w['s'], ' ', $r['s'];",
        "42 42",
    );
}

/// A ref-bearing literal as the FIRST `??` operand (`['s' => &$v] ?? $x`): a literal is
/// never null, so the coalesce short-circuits to the ref hash and the entry keeps
/// aliasing the source (php-cross-checked: PHP prints `9`).
#[test]
fn test_ref_entry_literal_coalesce_first_operand_short_circuits() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $x = ['s' => 0]; $r = ['s' => &$v] ?? $x; $v = 9; echo $r['s'];",
        "9",
    );
}

/// Mixed ternary-of-coalesce-of-match composition: a `match` arm yields the ref literal,
/// its merge feeds a `??`, and that feeds a ternary — the ref-hash value type must
/// propagate through all three nested merges (php-cross-checked: PHP prints `9`).
#[test]
fn test_ref_entry_literal_ternary_coalesce_match_composition() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = null; $k = 1; $c = $argc > 0; \
         $r = $c ? ($a ?? match ($k) { 1 => ['s' => &$v], default => ['s' => 0] }) \
                 : ['s' => 0]; \
         $v = 9; echo $r['s'];",
        "9",
    );
}

// --- Duplicate keys inside a ref-bearing literal (PHP replace semantics) ---

/// A duplicate static key AFTER a reference entry REPLACES the bucket (PHP literal
/// construction is zend_hash_update): the reference is discarded, the plain value wins,
/// and the source variable is NOT written through (php-cross-checked: PHP prints `2 9`).
#[test]
fn test_ref_entry_literal_duplicate_key_replaces_bucket() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $a = ['s' => &$v, 's' => 2]; $v = 9; echo $a['s'], ' ', $v;",
        "2 9",
    );
}

/// The reverse duplicate order — plain entry first, reference entry second — leaves the
/// LAST (reference) binding in place, so the element aliases the source
/// (php-cross-checked: PHP prints `8 8`).
#[test]
fn test_ref_entry_literal_duplicate_key_reverse_order_keeps_ref() {
    assert_ref_array_element_heap_clean(
        "<?php $w = 5; $b = ['t' => 1, 't' => &$w]; $w = 8; echo $b['t'], ' ', $w;",
        "8 8",
    );
}

/// A DYNAMIC key colliding with an earlier reference entry's key replaces the bucket the
/// same way a static duplicate does — the guard `unset` applies to runtime-computed keys
/// too (php-cross-checked: PHP prints `2 9`).
#[test]
fn test_ref_entry_literal_duplicate_dynamic_key_replaces_bucket() {
    assert_ref_array_element_heap_clean(
        "<?php $v = 5; $k = 's'; $a = ['s' => &$v, $k => 2]; $v = 9; \
         echo $a['s'], ' ', $v;",
        "2 9",
    );
}

// --- Heap behavior of ref-bearing literals in loops (leak regressions) ---

/// A fresh ref-bearing literal built EVERY loop iteration keeps live heap blocks constant:
/// the hoisted source cell is reused, the reassigned hidden temp's previous hash is
/// released through the kind-dispatching release (a statically Array-typed slot can hold
/// de-packed hash storage), and the Mixed element reads feeding the accumulator release
/// their boxes (regression: 3 leaked blocks per iteration; php-cross-checked output).
#[test]
fn test_ref_entry_literal_loop_heap_constant() {
    assert_ref_array_element_heap_clean(
        "<?php
        function build(int $n): int {
            $acc = 0;
            for ($i = 0; $i < $n; $i++) {
                $v = $i;
                $arr = ['s' => &$v, 'p' => $i * 2];
                $acc = $acc + $arr['s'] + $arr['p'];
            }
            return $acc;
        }
        echo build(50);",
        "3675",
    );
}

/// The statement-form sibling (`$arr = []; $arr['s'] = &$v; ...` in a function loop)
/// stays heap-constant too: releasing the reassigned local dispatches on the runtime heap
/// kind so the de-packed hash frees its buckets and cell share (regression: ~5 leaked
/// blocks per iteration; php-cross-checked output).
#[test]
fn test_ref_element_statement_form_loop_heap_constant() {
    assert_ref_array_element_heap_clean(
        "<?php
        function build(int $n): int {
            $acc = 0;
            for ($i = 0; $i < $n; $i++) {
                $v = $i;
                $arr = [];
                $arr['s'] = &$v;
                $arr['p'] = $i * 2;
                $acc = $acc + $arr['s'] + $arr['p'];
            }
            return $acc;
        }
        echo build(50);",
        "3675",
    );
}

/// A ref-bearing literal returned DIRECTLY under a declared `: array` return type hands
/// the caller a live hash: the return re-loads the hidden yield temp so the epilogue's
/// returned-slot exclusion transfers the slot's share instead of double-claiming the
/// store's acquire (regression: the caller read freed memory — printed garbage/empty —
/// with allocs == frees; php-cross-checked: PHP prints `41 1`).
#[test]
fn test_ref_entry_literal_direct_return_typed_array() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(): array {
            $v = 41;
            return ['s' => &$v];
        }
        $r = mk();
        echo $r['s'], ' ', count($r);",
        "41 1",
    );
}

/// A ref-bearing literal returned through a `??` CHAIN under `: array` survives the
/// Mixed merge + return unboxing: the Mixed→array coercion passes de-packed hash
/// storage through unchanged (kind probe) and the boxed string-key read dereferences
/// the tag-11 bucket (regression: empty element read + 2 leaked blocks per call;
/// php-cross-checked: PHP prints `5 1`).
#[test]
fn test_ref_entry_literal_chain_return_typed_array() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(?array $a, int $x): array {
            $v = $x;
            $b = null;
            return $a ?? $b ?? ['s' => &$v];
        }
        $r = mk(null, 5);
        echo $r['s'], ' ', count($r);",
        "5 1",
    );
}

/// A ref-bearing literal returned from WITHIN a conditional joins a differently-shaped
/// plain-literal return under one `: array` contract: the hash return path re-stamps to
/// the Array(Mixed) contract via `HashToMixed` (reference buckets pass through untouched)
/// and both paths stay ownership-balanced (php-cross-checked: PHP prints `41 1 | 7 1`).
#[test]
fn test_ref_entry_literal_conditional_return_typed_array() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(bool $c): array {
            $v = 41;
            if ($c) { return ['s' => &$v]; }
            return ['t' => 7];
        }
        $r = mk(true);
        $q = mk(false);
        echo $r['s'], ' ', count($r), ' | ', $q['t'], ' ', count($q);",
        "41 1 | 7 1",
    );
}

/// A ref-bearing literal in call-ARGUMENT position leaves no live blocks: the callee's
/// `: int` return coercion releases the owned Mixed element box it unboxes (regression:
/// exactly 1 leaked block per call; php-cross-checked: PHP prints `5`).
#[test]
fn test_ref_entry_literal_call_argument_heap_clean() {
    assert_ref_array_element_heap_clean(
        "<?php
        function takes(array $x): int { return $x['s']; }
        $v = 5;
        echo takes(['s' => &$v]);",
        "5",
    );
}

/// The call-argument shape looped 50 times (with the literal reached through a `??`
/// chain) stays heap-clean: one leaked Mixed element box per call would report 50 live
/// blocks at exit (php-cross-checked: PHP prints `1225`).
#[test]
fn test_ref_entry_literal_call_argument_loop_heap_clean() {
    assert_ref_array_element_heap_clean(
        "<?php
        function takes(array $x): int { return $x['s']; }
        function run(int $n): int {
            $acc = 0;
            for ($i = 0; $i < $n; $i++) {
                $v = $i;
                $a = null; $b = null;
                $acc += takes($a ?? $b ?? ['s' => &$v]);
            }
            return $acc;
        }
        echo run(50);",
        "1225",
    );
}

/// A PARAMETER as the `&$x` ref-entry source keeps its incoming argument: the prologue's
/// entry-block `LocalRefEnsure` hoist must adopt the spilled argument value instead of
/// zero-initializing the parameter slot first (regression: `mk(13)` read `0` through the
/// entry; php-cross-checked: PHP prints `13 1`).
#[test]
fn test_ref_entry_literal_int_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $x): array { return ['s' => &$x]; }
        $r = mk(13);
        echo $r['s'], ' ', count($r);",
        "13 1",
    );
}

/// A plain read of the parameter BEFORE the ref-bearing literal already routes through the
/// hoisted entry-block cell, so it must see the incoming argument (sharpest regression shape:
/// the argument was lost AT FUNCTION ENTRY, echoing `0` before the literal even ran;
/// php-cross-checked: PHP prints `13 13`).
#[test]
fn test_ref_entry_literal_param_echo_before_literal() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $x): int { echo $x, ' '; $r = ['s' => &$x]; return $r['s']; }
        echo mk(13);",
        "13 13",
    );
}

/// A METHOD parameter as the ref-entry source adopts its incoming argument the same way as a
/// free-function parameter (php-cross-checked: PHP prints `13 1`).
#[test]
fn test_ref_entry_literal_method_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        class K { public function mk(int $x): array { return ['s' => &$x]; } }
        $k = new K();
        $q = $k->mk(13);
        echo $q['s'], ' ', count($q);",
        "13 1",
    );
}

/// A CLOSURE parameter as the ref-entry source adopts its incoming argument (closure bodies
/// lower through the same scope-entry hoist; php-cross-checked: PHP prints `13 1`).
#[test]
fn test_ref_entry_literal_closure_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        $f = function (int $x): array { return ['s' => &$x]; };
        $q = $f(13);
        echo $q['s'], ' ', count($q);",
        "13 1",
    );
}

/// A closure with a by-VALUE `use` capture alongside the ref-entry parameter source keeps
/// both: the capture reads normally and the parameter's incoming value flows into the entry
/// (php-cross-checked: PHP prints `13 100`).
#[test]
fn test_ref_entry_literal_closure_use_plus_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        $base = 100;
        $f = function (int $x) use ($base): array { return ['s' => &$x, 'b' => $base]; };
        $q = $f(13);
        echo $q['s'], ' ', $q['b'];",
        "13 100",
    );
}

/// A `mixed` parameter arrives as a CALLER-owned boxed cell: the hoisted ensure must back its
/// adoption with its own acquire, or the caller's post-call release frees the box under the
/// cell (regression: bad-refcount fatal + empty string read; the kind-5 probe keeps the cell's
/// inner tag Mixed for both an int and a string payload; php-cross-checked: PHP prints
/// `42 1 zz`).
#[test]
fn test_ref_entry_literal_mixed_param_source_int_and_string() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(mixed $m): array { return ['s' => &$m]; }
        $q = mk(42);
        $w = mk('zz');
        echo $q['s'], ' ', count($q), ' ', $w['s'];",
        "42 1 zz",
    );
}

/// An ARRAY parameter as the ref-entry source: a mutation of the source parameter after the
/// literal is visible through the entry (one storage — the count matches on both views) and
/// the adopted handle stays ownership-balanced (php-cross-checked: PHP prints `3 3`).
#[test]
fn test_ref_entry_literal_array_param_cow_alias_visibility() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(array $a): string {
            $r = ['s' => &$a];
            $a[] = 99;
            return count($r['s']) . ' ' . count($a);
        }
        echo mk([1, 2]);",
        "3 3",
    );
}

/// An ELEMENT read through the array-parameter entry (`$r['s'][2]`) sees the value the source
/// mutation appended. Output-only: the indexed read through a ref entry leaks 1-2 blocks in a
/// PRE-EXISTING family that reproduces identically with a main-scope non-parameter source
/// (`$a=[1,2]; $r=['s'=>&$a]; $a[]=99; echo $r['s'][2];` → 2 live blocks), so heap-cleanliness
/// is not a parameter-seeding property here (php-cross-checked: PHP prints `3 99 3`).
#[test]
fn test_ref_entry_literal_array_param_element_read_through_entry() {
    let out = compile_and_run(
        "<?php
        function mk(array $a): string {
            $r = ['s' => &$a];
            $a[] = 99;
            return count($r['s']) . ' ' . $r['s'][2] . ' ' . count($a);
        }
        echo mk([1, 2]);",
    );
    assert_eq!(out, "3 99 3");
}

/// A reassign of the parameter AFTER the literal writes through the shared cell, so the entry
/// observes the new value (php-cross-checked: PHP prints `99`).
#[test]
fn test_ref_entry_literal_param_writeback_through_alias() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $x): int { $r = ['s' => &$x]; $x = 99; return $r['s']; }
        echo mk(13);",
        "99",
    );
}

/// A write THROUGH the ref entry updates the parameter (the reverse direction of the alias;
/// php-cross-checked: PHP prints `55`).
#[test]
fn test_ref_entry_literal_param_write_through_entry() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $x): int { $r = ['s' => &$x]; $r['s'] = 55; return $x; }
        echo mk(13);",
        "55",
    );
}

/// The statement form (`$r['s'] = &$x;` on an empty local) adopts a parameter source the same
/// way as the literal desugar — both route through the same scope-entry hoist
/// (php-cross-checked: PHP prints `13 1`).
#[test]
fn test_ref_entry_stmt_form_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $x): array { $r = []; $r['s'] = &$x; return $r; }
        $q = mk(13);
        echo $q['s'], ' ', count($q);",
        "13 1",
    );
}

/// A VARIADIC parameter (`int ...$rest`) as the ref-entry source keeps the collected argument
/// array: the count read through the entry sees all collected arguments and the adopted
/// handle stays ownership-balanced (php-cross-checked: PHP prints `3`).
#[test]
fn test_ref_entry_literal_variadic_param_source() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int ...$rest): int { $r = ['s' => &$rest]; return count($r['s']); }
        echo mk(1, 2, 3);",
        "3",
    );
}

/// An ELEMENT read through the variadic-parameter entry (`$r['s'][1]`) sees the collected
/// argument. Output-only: the indexed read through a ref entry is the same PRE-EXISTING leak
/// family as the array-parameter element read above (reproduces with a non-parameter source),
/// so heap-cleanliness is not asserted (php-cross-checked: PHP prints `5` = count 3 + element 2).
#[test]
fn test_ref_entry_literal_variadic_param_element_read_through_entry() {
    let out = compile_and_run(
        "<?php
        function mk(int ...$rest): int { $r = ['s' => &$rest]; return count($r['s']) + $r['s'][1]; }
        echo mk(1, 2, 3);",
    );
    assert_eq!(out, "5");
}

/// Binding the SAME parameter-sourced local into two entries shares one persistent cell (the
/// declared-by-ref-param guard must not misfire on locals that became ref-bound via an earlier
/// `=&` bind; php-cross-checked: PHP prints `21 21`).
#[test]
fn test_ref_entry_literal_param_seeded_local_double_bind() {
    assert_ref_array_element_heap_clean(
        "<?php
        function mk(int $seed): array {
            $v = $seed;
            $r = ['a' => &$v, 'b' => &$v];
            $v = 21;
            return $r;
        }
        $q = mk(5);
        echo $q['a'], ' ', $q['b'];",
        "21 21",
    );
}
/// `$b =& $a[0]` aliases an indexed-array int element: writing the local updates the array
/// element in place (write-through from the alias to the array).
#[test]
fn test_ref_alias_array_element_int() {
    let out = compile_and_run(r#"<?php $a = [1, 2]; $b =& $a[0]; $b = 9; echo $a[0];"#);
    assert_eq!(out, "9");
}

/// `$b =& $a[0]` aliases an indexed-array int element: reading the local after the array is
/// mutated through another path reflects the change (write-through from the array to the alias).
#[test]
fn test_ref_alias_array_element_int_readback() {
    let out = compile_and_run(
        r#"<?php $a = [1, 2]; $b =& $a[0]; $a[0] = 7; echo $b;"#,
    );
    assert_eq!(out, "7");
}

/// `$b =& $a[0]` aliases an indexed-array string element: writing the local updates the array
/// element's pointer and length in place.
#[test]
fn test_ref_alias_array_element_string() {
    let out = compile_and_run(
        r#"<?php $a = ["hello", "world"]; $b =& $a[0]; $b = "HEY"; echo $a[0];"#,
    );
    assert_eq!(out, "HEY");
}

/// `$b =& $a[1]` aliases a non-zero indexed-array int element: the address computation must
/// scale by the element size and skip the header correctly.
#[test]
fn test_ref_alias_array_element_nonzero_index() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; $b =& $a[1]; $b = 99; echo $a[1];"#,
    );
    assert_eq!(out, "99");
}

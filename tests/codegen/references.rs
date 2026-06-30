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

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

/// Two declared properties bound by reference share one cell and observe writes both ways.
#[test]
fn test_property_to_property_reference_assignment_writes_through() {
    let out = compile_and_run(
        "<?php
        class RefPropertyBox {
            public array $refs = [];
            public function bind(RefPropertyBox $source): void {
                $this->refs = &$source->refs;
            }
        }
        $source = new RefPropertyBox();
        $target = new RefPropertyBox();
        $target->bind($source);
        $source->refs[] = 'source';
        echo $target->refs[0], \"\\n\";
        $target->refs[] = 'target';
        echo $source->refs[1], \"\\n\";",
    );
    assert_eq!(out, "source\ntarget\n");
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

/// `isset()` probes the owner-slot marker and dereferenced value without triggering a typed
/// uninitialized-property read, including properties promoted to reference-cell storage.
#[test]
fn test_isset_reference_properties_respects_initialization_and_null() {
    let out = compile_and_run(
        r#"<?php
class ReferenceIssetBox {
    public int $typed;
    public mixed $nullable = null;
    public int $zero = 0;

    public function &typedRef(): int { return $this->typed; }
    public function &nullableRef(): mixed { return $this->nullable; }
    public function &zeroRef(): int { return $this->zero; }
}
$box = new ReferenceIssetBox();
var_dump(isset($box->typed), isset($box->nullable), isset($box->zero));
$box->typed = 7;
$alias =& $box->nullable;
$alias = "set";
var_dump(isset($box->typed), isset($box->nullable));
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\n"
    );
}

/// Unsetting an element through an array by-reference parameter publishes a relocated sparse hash
/// through the shared cell, so string and integer-key removals are both visible to the caller.
#[test]
fn test_unset_array_element_through_by_ref_parameter() {
    let out = compile_and_run(
        r#"<?php
function remove_ref_key(array &$values, mixed $key): void {
    unset($values[$key]);
}
$assoc = ["keep" => 1, "drop" => 2];
$copy = $assoc;
remove_ref_key($assoc, "drop");
echo count($assoc), ":", isset($assoc["drop"]) ? "bad" : "assoc", ":", count($copy), "|";
$list = [10, 20, 30];
remove_ref_key($list, 1);
echo count($list), ":", array_is_list($list) ? "list" : "assoc", ":", $list[2];
"#,
    );
    assert_eq!(out, "1:assoc:2|2:assoc:30");
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

/// Binds an arrow closure that returns a reference to `$this->prop`, captures the reference,
/// mutates it through the alias, and clears it through the same shared property cell.
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

/// An immediately invoked `Closure::bind` over a string property returns a reference to
/// `$this->prop`, which remains shared when captured and mutated through the alias.
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

/// A reference assignment expression vivifies a static hash entry and returns its aliased value.
#[test]
fn test_ref_assignment_expression_static_hash_element() {
    let out = compile_and_run(
        "<?php
        class C {
            private static array $cache = [];
            public static function read(string $key) {
                if (null !== $value = &self::$cache[$key]) {
                    echo $value;
                    return;
                }
                $value = 7;
            }
        }
        C::read('key');
        C::read('key');",
    );
    assert_eq!(out, "7");
}

/// A static-property post-increment expression returns the old value and stores the increment.
#[test]
fn test_static_property_post_increment_expression() {
    let out = compile_and_run(
        "<?php
        class C {
            private static int $level = 0;
            public static function enter() {
                if (!self::$level++) {
                    echo 'first';
                }
            }
        }
        C::enter();
        C::enter();",
    );
    assert_eq!(out, "first");
}

/// A runtime-named property reference aliases the suffix-constrained declared array slot.
#[test]
fn test_dynamic_property_reference_with_known_suffix() {
    let out = compile_and_run(
        "<?php
        class C {
            private array $beforePasses = [[1]];
            private array $afterPasses = [[3]];
            public function append(string $type, int $value) {
                $property = $type.'Passes';
                $passes = &$this->$property;
                $passes[0][] = $value;
            }
            public function printBefore() {
                echo $this->beforePasses[0][0], $this->beforePasses[0][1];
            }
        }
        $passes = new C();
        $passes->append('before', 2);
        $passes->printBefore();",
    );
    assert_eq!(out, "12");
}

/// A nested array append by reference retains the promoted local cell after its frame returns.
#[test]
fn test_nested_array_reference_append_outlives_source_frame() {
    let out = compile_and_run(
        "<?php
        function append_ref(array &$loops): void {
            $path = [1];
            $loops[0][] = &$path;
            $path[] = 2;
        }
        function print_ref(mixed $loops): void {
            echo $loops[0][0][0], $loops[0][0][1];
        }
        $loops = [[]];
        append_ref($loops);
        print_ref($loops);",
    );
    assert_eq!(out, "12");
}

/// A local array element bound to a local variable shares one promoted cell in both directions;
/// unsetting the local alias leaves the array-owned marker alive and writable.
#[test]
fn test_local_array_element_reference_assignment_writes_through() {
    let out = compile_and_run(
        "<?php
        $values = [1];
        $stub = 5;
        $values[0] = &$stub;
        $stub = 7;
        echo $values[0];
        $values[0] = 8;
        echo $stub;
        unset($stub);
        $values[0] = 9;
        echo $values[0];",
    );
    assert_eq!(out, "789");
}

/// A nested associative element retains a local reference cell in
/// both directions, including dynamically concatenated keys and an autovivified parent hash.
#[test]
fn test_nested_assoc_array_element_reference_assignment_writes_through() {
    let out = compile_and_run(
        "<?php
        $data = [];
        $prefix = 'p:';
        $key = 'item';
        $value = 5;
        $data[$prefix.'use']['$'.$key] = &$value;
        $value = 7;
        echo $data['p:use']['$item'];
        $data['p:use']['$item'] = 9;
        echo $value;",
    );
    assert_eq!(out, "79");
}

/// A static-property hash element bound by reference observes a later scalar write through its alias.
#[test]
fn test_ref_static_property_array_element_scalar_write_through() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['source'] = 1;
        C::$a['alias'] = &C::$a['source'];
        C::$a['alias'] = 9;
        echo C::$a['source'];",
    );
    assert_eq!(out, "9");
}

/// The DebugClassLoader-shaped heterogeneous array entry remains shared after reference binding.
#[test]
fn test_ref_static_property_array_element_nested_array_alias() {
    let out = compile_and_run(
        "<?php
        class C { public static array $a = []; }
        C::$a['source'] = ['X', []];
        C::$a['alias'] = &C::$a['source'];
        C::$a['source'][0] = 'MUT';
        echo C::$a['source'][0], C::$a['alias'][0];",
    );
    assert_eq!(out, "MUTMUT");
}

/// An untyped by-reference parameter materializes an undefined caller variable as writable
/// mixed storage before the call and exposes the assigned value afterwards.
#[test]
fn test_undefined_variable_can_be_initialized_through_by_ref_parameter() {
    let out = compile_and_run(
        "<?php
        function initialize(&$destination): void {
            $destination = ['ready'];
        }
        initialize($result);
        echo $result[0];",
    );
    assert_eq!(out, "ready");
}

/// By-reference output discovery follows callable signatures for named arguments as well as
/// positional calls, without relying on a particular function name.
#[test]
fn test_named_by_ref_argument_can_initialize_undefined_variable() {
    let out = compile_and_run(
        "<?php
        function initialize_named($prefix, &$destination): void {
            $destination = $prefix.'done';
        }
        initialize_named(destination: $result, prefix: 'all-');
        echo $result;",
    );
    assert_eq!(out, "all-done");
}

/// A nullable declared by-reference parameter widens an existing compatible array local to the
/// full writable parameter storage contract before a static method call.
#[test]
fn test_nullable_array_by_ref_parameter_accepts_existing_array_variable() {
    let out = compile_and_run(
        "<?php
        final class OutputWriter {
            public static function populate(?array &$destination = null): void {
                $destination = ['ready'];
            }
        }
        $result = [];
        OutputWriter::populate($result);
        echo $result[0];",
    );
    assert_eq!(out, "ready");
}

/// A static call through `self` creates writable caller storage for an undefined variable passed
/// to a declared nullable by-reference parameter.
#[test]
fn test_self_static_by_ref_call_initializes_undefined_variable() {
    let out = compile_and_run(
        "<?php
        final class OutputWriter {
            public static function populate(?array &$destination = null): void {
                $destination = ['ready'];
            }
            public static function render(): string {
                self::populate($result);
                return $result[0];
            }
        }
        echo OutputWriter::render();",
    );
    assert_eq!(out, "ready");
}

/// A by-reference output produced by the right side of `&&` is available in the true branch.
#[test]
fn test_short_circuit_by_ref_output_is_visible_in_true_branch() {
    let out = compile_and_run(
        "<?php
        final class OutputWriter {
            public static function populate(?array &$destination = null): int {
                $destination = ['ready'];
                return 1;
            }
            public static function render(): string {
                if (true && self::populate($result)) {
                    return $result[0];
                }
                return 'missing';
            }
        }
        echo OutputWriter::render();",
    );
    assert_eq!(out, "ready");
}

/// A successful null probe publishes a branch-local fact for an output that is created on only
/// one side of an earlier short-circuit expression.
#[test]
fn test_isset_guards_conditionally_initialized_by_ref_output() {
    let out = compile_and_run(
        "<?php
        final class OutputWriter {
            public static function populate(?array &$destination = null): int {
                $destination = ['value' => 'ready'];
                return 1;
            }
            public static function render(bool $lead, bool $skip): void {
                if ($lead && ($skip || self::populate($result))) {
                    if (isset($result['value']) && $result['value'] === 'ready') {
                        echo $result['value'];
                    }
                }
            }
        }
        OutputWriter::render(true, false);",
    );
    assert_eq!(out, "ready");
}

/// An unresolved direct call throws before evaluating its arguments, so an undefined variable
/// argument does not cause a stale follow-on diagnostic in code after the non-returning call.
#[test]
fn test_late_bound_call_undefined_argument_does_not_cascade() {
    let out = compile_and_run(
        "<?php
        try {
            unavailable_runtime_function('key', $result);
            echo $result;
        } catch (Error $error) {
            echo 'caught';
        }",
    );
    assert_eq!(out, "caught");
}

/// Capturing an undefined local by reference creates a writable null cell in the outer scope.
#[test]
fn test_by_ref_closure_capture_initializes_undefined_variable() {
    let out = compile_and_run(
        "<?php
        $writer = static function () use (&$result): void {
            $result = 'ready';
        };
        $writer();
        echo $result;",
    );
    assert_eq!(out, "ready");
}

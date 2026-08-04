//! Purpose:
//! Regression coverage for nested writes through homogeneous matrices,
//! an `Array(Mixed)` element (`$a[$i][$j] = ...`), and an `ArrayAccess`
//! object parent (`$objects[$i][$key] = ...`) (issue #529).
//!
//! Called from:
//! - `cargo test --test codegen_tests arrays::nested_mixed_write`.
//!
//! Key details:
//! - The nested-assign statement used to lower the FULL target as a read and
//!   then replace the resulting Mixed cell in place. `__rt_mixed_array_get`
//!   returns a detached fresh box whenever the slot storage is not already a
//!   boxed Mixed cell (string/int/float slots of a concrete inner array), so
//!   the write mutated a temporary and was silently lost, leaking the
//!   replacement payload. When the slot WAS a boxed cell the write landed, but
//!   the retained cell returned by the read was never released (leak).
//! - The fix writes through the parent cell instead (three-operand
//!   `Op::RuntimeCall` → `__rt_mixed_array_set` for Mixed parents,
//!   `offsetSet` for `ArrayAccess` object parents), which mutates the aliased
//!   container for every slot representation.

use crate::support::{compile_and_run, compile_and_run_with_heap_debug};

/// Homogeneous `array<array<int>>` matrices preserve concrete slot types while leaf writes
/// COW-split and write the mutated child back through every indexed parent.
#[test]
fn test_nested_write_homogeneous_matrix_from_array_fill() {
    let baseline = compile_and_run_with_heap_debug(
        r#"<?php
function renderMatrixBaseline(): void {
    $matrix = array_fill(0, 3, array_fill(0, 3, 0));
    foreach ($matrix as $row) {
        echo $row[0] . ":" . $row[2] . "\n";
    }
}
renderMatrixBaseline();
"#,
    );
    assert!(
        baseline
            .stderr
            .contains("HEAP DEBUG: leak summary: clean"),
        "expected clean baseline heap, got: {}",
        baseline.stderr
    );
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function renderMatrix(): void {
    $matrix = array_fill(0, 3, array_fill(0, 3, 0));
    for ($i = 1; $i < 3; ++$i) {
        $matrix[$i][0] = $i + 10;
    }
    for ($j = 1; $j < 3; ++$j) {
        $matrix[0][$j] = $j + 20;
    }
    for ($i = 1; $i < 3; ++$i) {
        for ($j = 1; $j < 3; ++$j) {
            $matrix[$i][$j] = $matrix[$i - 1][$j - 1] + $i + $j;
        }
    }
    foreach ($matrix as $row) {
        echo $row[0] . ":" . $row[1] . ":" . $row[2] . "\n";
    }
}
renderMatrix();
"#,
    );
    assert_eq!(out.stdout, "0:21:22\n11:2:24\n12:14:6\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Associative roots whose values are concrete indexed arrays write a COW-split child back into
/// the map instead of mutating a detached `HashGet` result.
#[test]
fn test_nested_write_assoc_root_concrete_array_value() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$scopes = [];
$name = 'property';
$scopes[$name] = ['class', 'name', null, 1];
$scopes[$name][2] = 'write-scope';
echo $scopes[$name][0] . ':' . $scopes[$name][2] . "\n";
"#,
    );
    assert_eq!(out.stdout, "class:write-scope\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// A declared static cache whose entries were aliased by reference must publish a replaced child
/// through the static property's shared storage.
#[test]
fn test_nested_write_static_assoc_cache_entry() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Cache {
    private static array $entries = [];

    public static function replaceFiles(): void {
        self::$entries[0] = ['/', []];
        self::$entries[1] = &self::$entries[0];
        $key = 1;
        $files = ['file' => 'resolved'];
        self::$entries[$key][1] = $files;
        echo self::$entries[$key][0] . ':' . self::$entries[$key][1]['file'] . "\n";
    }
}
Cache::replaceFiles();
"#,
    );
    assert_eq!(
        out.stdout, "/:resolved\n",
        "unexpected stderr: {}",
        out.stderr
    );
    assert!(out.success, "program failed: {}", out.stderr);
    // A control with only the static tuple + alias retains 7 blocks at process exit. The nested
    // write intentionally makes the `$files` hash and its persisted string reachable from that
    // static graph, adding exactly 2 blocks; any extra block is a write-back ownership leak.
    assert!(
        out.stderr
            .contains("HEAP DEBUG: leak summary: live_blocks=9"),
        "expected the 7-block static baseline plus 2 reachable file-map blocks, got: {}",
        out.stderr
    );
}

/// Issue #529 repro: the inner arrays are homogeneous `array<string>` (16-byte
/// string slots), the outer heterogeneous literal is `array<mixed>`. The write
/// must replace the stored element, not a detached copy, and stay heap-clean.
#[test]
fn test_nested_write_string_slot_inner_array() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [['x', 'y0'], ['x', 'y1'], ['x', 'y2'], 7];
$a[2][1] = 'patched';
echo $a[2][1] . "\n";
echo $a[1][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "patched\ny1\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Same shape with homogeneous `array<int>` inner arrays (8-byte int slots).
#[test]
fn test_nested_write_int_slot_inner_array() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2], [3, 4], 'z'];
$a[1][0] = 99;
echo $a[1][0] . "\n";
echo $a[0][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "99\n2\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Heterogeneous inner arrays store boxed Mixed cells: the write already
/// propagated on this shape, but the retained cell returned by the target
/// read was never released, leaking one block per write.
#[test]
fn test_nested_write_boxed_cell_inner_array() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$b = [[1, 'x'], [2, 'y'], 7];
$b[1][0] = 99;
echo $b[1][0] . "\n";
echo $b[1][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "99\ny\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Associative inner container: overwriting an existing string key and adding
/// a brand-new key must both land in the stored hash.
#[test]
fn test_nested_write_assoc_inner_array() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [['k' => 'v'], 7];
$a[0]['k'] = 'patched';
$a[0]['new'] = 'added';
echo $a[0]['k'] . "\n";
echo $a[0]['new'] . "\n";
"#,
    );
    assert_eq!(out.stdout, "patched\nadded\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Compound nested assignment reads the stale element and writes the combined
/// result back through the same nested-write path.
#[test]
fn test_nested_compound_assign() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [['x', 'y'], 7];
$a[0][1] .= '!';
echo $a[0][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "y!\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Three-level chain through boxed-cell intermediates: the middle read
/// returns the STORED cell (retained), so the leaf array stays uniquely
/// owned and `__rt_mixed_array_set` mutates it in place.
///
/// Concrete homogeneous intermediate arrays are normalized to stored Mixed
/// cells by the fetch-for-write path, so the deepest write remains connected
/// to the outer container instead of landing in a detached split copy.
#[test]
fn test_nested_write_three_levels() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[['x', 1], 5], 7];
$a[0][0][0] = 'patched';
echo $a[0][0][0] . "\n";
echo $a[0][0][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "patched\n1\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// The nested write must survive a function-return boundary on the outer array.
#[test]
fn test_nested_write_survives_function_return() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function patch(): array {
    $a = [['x', 'y0'], ['x', 'y1'], 7];
    $a[1][1] = 'patched';
    return $a;
}
$r = patch();
echo $r[1][1] . "\n";
"#,
    );
    assert_eq!(out.stdout, "patched\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Object `ArrayAccess` parent: `$boxes[0]` is a concrete object, so the
/// three-operand nested write must dispatch to `offsetSet` (not Mixed cell
/// replacement) and persist into the stored instance.
#[test]
fn test_nested_write_array_access_object_parent() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Box implements ArrayAccess {
    private string $x = 'old';
    private int $y = 1;
    public function offsetExists(mixed $offset): bool {
        return $offset === 'x' || $offset === 'y';
    }
    public function offsetGet(mixed $offset): mixed {
        if ($offset === 'x') {
            return $this->x;
        }
        if ($offset === 'y') {
            return $this->y;
        }
        return null;
    }
    public function offsetSet(mixed $offset, mixed $value): void {
        if ($offset === 'x') {
            $this->x = (string)$value;
        }
        if ($offset === 'y') {
            $this->y = (int)$value;
        }
    }
    public function offsetUnset(mixed $offset): void {}
}
$boxes = [new Box()];
$boxes[0]['x'] = 'patched';
echo $boxes[0]['x'] . "\n";
echo $boxes[0]['y'] . "\n";
"#,
    );
    assert_eq!(out.stdout, "patched\n1\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// A loop-carried empty root is read before its nested autovivifying write in source order.
///
/// The next iteration can observe the container produced by the previous one, so both checker
/// and lowering must pre-widen the root to boxed elements before the first body read.
#[test]
fn test_loop_backedge_prewidens_empty_nested_write_root() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$unmerged = [];
foreach ([0, 1] as $rowKey) {
    foreach ([0, 1] as $lineKey) {
        if (!array_key_exists($rowKey, $unmerged) || !array_key_exists($lineKey, $unmerged[$rowKey])) {
            $unmerged[$rowKey][$lineKey] = ['seed'];
        }
        $unmerged[$rowKey][$lineKey][1] = $rowKey * 10 + $lineKey;
    }
}
echo $unmerged[0][0][1], ':', $unmerged[0][1][1], ':', $unmerged[1][0][1], ':', $unmerged[1][1][1];
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "0:1:10:11");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// An INT-keyed read of an array whose element type is still `Never` — an empty `[]` literal whose
/// only writer is a nested `$u[$k][$j] = …` — must widen to `Mixed`, not hand back `Never`.
///
/// `Never` there means "nothing written yet", not "uninhabited": PHP grows the array at runtime and
/// the read legitimately lands on what a nested write put there. The STRING-keyed arm of the same
/// match already answered `Mixed` for the byte-identical container, so `$u[$k]` was accepted when
/// `$k` came from `foreach (… as $k)` (a string value) and rejected when it came from
/// `foreach (… as $k => $v)` (an int key) — the only difference between the two. Regression for
/// `Console\Helper\Table::renderRow` (`array_key_exists($lineKey, $unmergedRows[$rowKey])`) and
/// `DependencyInjection\Definition::setMethodCalls`.
#[test]
fn test_int_keyed_read_of_never_element_widens_to_mixed() {
    let out = compile_and_run(
        r#"<?php
$u = [];
foreach ([['a', 'b']] as $rowKey => $row) {
    foreach ($row as $lineKey => $cell) {
        if (!\array_key_exists($rowKey, $u) || !\array_key_exists($lineKey, $u[$rowKey])) {
            $u[$rowKey][$lineKey] = $cell;
        }
    }
}
echo count($u), ":", count($u[0]), ":", $u[0][1];
"#,
    );
    assert_eq!(out, "1:2:b");
}

/// The same read reaching `count()` rather than `array_key_exists()`, so the widening is pinned at
/// the element type and not at one builtin's argument check.
#[test]
fn test_int_keyed_never_element_reaches_count() {
    let out = compile_and_run(
        r#"<?php
$u = [];
foreach ([['x']] as $k => $row) {
    $u[$k][0] = $row[0];
    echo count($u[$k]);
}
"#,
    );
    assert_eq!(out, "1");
}

/// `foreach` over an array whose element type is still `Never` must bind the loop value as `Mixed`.
///
/// `Never` is what an empty `[]` literal — or an `array`-hinted parameter no call site specialized —
/// carries, and it means "nothing written yet", not "uninhabited". Binding the value to `Never` made
/// it unusable in the body: `DependencyInjection\Definition::setMethodCalls` does
/// `foreach ($calls as $call) { $this->addMethodCall($call[0], …); }` and was reported as
/// "Cannot index non-array" on a program `php -n` runs. `Mixed` is what the same element already
/// reads as through `$calls[$i]`.
#[test]
fn test_foreach_over_never_element_binds_value_as_mixed() {
    let out = compile_and_run(
        r#"<?php
class D {
    private array $calls = [];
    public function addMethodCall(string $m, array $a = [], bool $r = false): static {
        $this->calls[] = [$m, $a, $r];
        return $this;
    }
    public function setMethodCalls(array $calls = []): static {
        $this->calls = [];
        foreach ($calls as $call) {
            $this->addMethodCall($call[0], $call[1], $call[2] ?? false);
        }
        return $this;
    }
    public function total(): int { return count($this->calls); }
}
$d = new D();
$d->setMethodCalls([['m1', ['x'], false], ['m2', [], true]]);
echo $d->total();
"#,
    );
    assert_eq!(out, "2");
}

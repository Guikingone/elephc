//! Purpose:
//! Validates public SPL heap loading through inherited __unserialize and wire round trips.
//!
//! Called from:
//! - Runtime GC codegen integration tests with unmodified PHP source.
//!
//! Key details:
//! - Pinned PHP fixes complete outputs, validation order, partial writes and exception chains.
//! - Both optimizer modes execute native binaries and require clean heaps.

use crate::support::*;

/// Exercises the actual public method and parser hook, without trusted AST rewriting.
fn assert_heap_unserialize(source: &str, expected: &str) {
    let source = format!("<?php {source}");
    let run = |mode: &str| {
        if let Some(output) = std::env::var_os("ELEPHC_HEAP_UNSERIALIZE_ASM_DIR") {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("heap_unserialize").rsplit("::").next().unwrap();
            let dir = std::path::PathBuf::from(output).join(name).join(mode);
            std::fs::create_dir_all(&dir).unwrap();
            let (user, runtime, libraries) = compile_source_to_asm_with_options(
                &source, &dir, 8_388_608, false, true,
            );
            std::fs::write(dir.join("user.s"), user).unwrap();
            std::fs::write(dir.join("runtime.s"), runtime).unwrap();
            std::fs::write(dir.join("source.php"), &source).unwrap();
            std::fs::write(dir.join("expected.stdout"), expected).unwrap();
            std::fs::write(dir.join("libraries.txt"), format!("{libraries:?}")).unwrap();
            return;
        }
        let output = compile_and_run_with_heap_debug(&source);
        assert!(output.success, "{}\n{}", output.stdout, output.stderr);
        assert_eq!(output.stdout, expected, "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    };
    run("on");
    without_ir_opt(|| run("off"));
}

/// Preserves PHP native formats.
#[test]
fn test_spl_heap_unserialize_formats() {
    assert_heap_unserialize("$h = new SplMaxHeap(); $h->__unserialize([[], ['flags' => 0, 'heap_elements' => [3, 1, 2]]]); while (!$h->isEmpty()) { echo $h->extract(), ','; } echo '|'; $q = new SplPriorityQueue(); $q->__unserialize([[], ['flags' => 3, 'heap_elements' => [['data' => null, 'priority' => 3], ['data' => 'low', 'priority' => 1]]]]); while (!$q->isEmpty()) { echo json_encode($q->extract()), ';'; }", "3,2,1,|{\"data\":null,\"priority\":3};{\"data\":\"low\",\"priority\":1};");
}

/// Preserves PHP native append override.
#[test]
fn test_spl_heap_unserialize_append_override() {
    assert_heap_unserialize("class H extends SplMaxHeap { function insert(mixed $value): true { echo 'override|'; return true; } function seed(): void { parent::insert(4); } } $h = new H(); $h->seed(); $h->__unserialize([[], ['flags' => 0, 'heap_elements' => [3, 1, 2]]]); while (!$h->isEmpty()) { echo $h->extract(), ','; }", "4,3,2,1,");
}

/// Preserves PHP native properties before state.
#[test]
fn test_spl_heap_unserialize_properties_before_state() {
    assert_heap_unserialize("class H extends SplMaxHeap { public int $value = 1; } $h = new H(); try { $h->__unserialize([['value' => 'bad'], ['flags' => 1, 'heap_elements' => []]]); } catch (Exception $e) { echo $e->getMessage(), '|'; } echo gettype($h->value), ':', $h->value;", "Invalid serialization data for H object|string:bad");
}

/// Preserves PHP native queue flag publication.
#[test]
fn test_spl_heap_unserialize_queue_flag_publication() {
    assert_heap_unserialize("$q = new SplPriorityQueue(); foreach ([6, 4] as $flags) { try { $q->__unserialize([[], ['flags' => $flags, 'heap_elements' => 'bad']]); } catch (Exception $e) { echo $e->getMessage(), '|'; } echo $q->getExtractFlags(), ';'; }", "Invalid serialization data for SplPriorityQueue object|2;Invalid serialization data for SplPriorityQueue object|2;");
}

/// Preserves PHP native comparator error.
#[test]
fn test_spl_heap_unserialize_comparator_error() {
    assert_heap_unserialize("class H extends SplMaxHeap { function compare(mixed $a, mixed $b): int { throw new Exception('compare'); } } $h = new H(); try { $h->__unserialize([[], ['flags' => 0, 'heap_elements' => [1, 2]]]); } catch (Exception $e) { echo $e->getMessage(), '|'; $previous = $e->getPrevious(); if ($previous instanceof Throwable) { echo $previous->getMessage(), '|'; } } echo $h->count(), ':', $h->isCorrupted() ? 'corrupt' : 'valid';", "Invalid serialization data for H object|compare|2:corrupt");
}

/// Preserves PHP native property error continues.
#[test]
fn test_spl_heap_unserialize_property_error_continues() {
    assert_heap_unserialize("class H extends SplMaxHeap { public mixed $value = null; public int $after = 1; } class D { function __destruct() { throw new Exception('drop'); } } $h = new H(); $h->value = new D(); try { $h->__unserialize([['value' => 'new', 'after' => 2], ['flags' => 0, 'heap_elements' => []]]); } catch (Exception $e) { echo $e->getMessage(), '|'; $previous = $e->getPrevious(); if ($previous instanceof Throwable) { echo $previous->getMessage(), '|'; } } echo $h->value, ':', $h->after;", "Invalid serialization data for H object|drop|new:2");
}

/// Preserves PHP native property readonly chain.
#[test]
fn test_spl_heap_unserialize_property_readonly_chain() {
    assert_heap_unserialize("class H extends SplMaxHeap { public mixed $value = null; public readonly int $locked; public int $after = 1; function __construct() { $this->locked = 1; } } class D { function __destruct() { throw new Exception('drop'); } } $h = new H(); $h->value = new D(); try { $h->__unserialize([['value' => 'new', 'locked' => 2, 'after' => 2], ['flags' => 0, 'heap_elements' => []]]); } catch (Throwable $e) { echo get_class($e), ':', $e->getMessage(), '|'; $previous = $e->getPrevious(); if ($previous instanceof Throwable) { echo get_class($previous), ':', $previous->getMessage(), '|'; $previous2 = $previous->getPrevious(); if ($previous2 instanceof Throwable) { echo get_class($previous2), ':', $previous2->getMessage(), '|'; if ($previous2->getPrevious() !== null) { echo 'extra|'; } } } } echo $h->value, ':', $h->after;", "Exception:Invalid serialization data for H object|Error:Cannot modify readonly property H::$locked|Exception:drop|new:1");
}

/// Preserves PHP native wire roundtrip.
#[test]
fn test_spl_heap_unserialize_wire_roundtrip() {
    assert_heap_unserialize("class H extends SplMaxHeap { private int $value = 7; public function value(): mixed { return $this->value; } } $h = new H(); $h->insert(3); $h->insert(1); $h->insert(2); $copy = unserialize(serialize($h)); if ($copy instanceof H) { echo $copy->value(), '|'; while (!$copy->isEmpty()) { echo $copy->extract(), ','; } }", "7|3,2,1,");
}

/// Preserves native corrupt order.
#[test]
fn test_spl_heap_unserialize_corrupt_order() {
    assert_heap_unserialize("class H extends SplMaxHeap { public int $marker = 0; function compare(mixed $a, mixed $b): int { throw new Exception('compare'); } } class Q extends SplPriorityQueue { public int $marker = 0; function compare(mixed $a, mixed $b): int { throw new Exception('compare'); } } $h = new H(); $h->insert(1); try { $h->insert(2); } catch (Exception $e) {} try { $h->__unserialize([['marker' => 2], ['flags' => 0, 'heap_elements' => []]]); } catch (Exception $e) { echo $e->getMessage(), '|'; } echo $h->marker, ';'; $q = new Q(); $q->insert('a', 1); try { $q->insert('b', 2); } catch (Exception $e) {} try { $q->__unserialize([['marker' => 2], ['flags' => 3, 'heap_elements' => []]]); } catch (Exception $e) { echo $e->getMessage(), '|'; } echo $q->marker, ':', $q->getExtractFlags();", "Heap is corrupted, heap properties are no longer ensured.|0;Heap is corrupted, heap properties are no longer ensured.|2:3");
}

/// Preserves native locked order.
#[test]
fn test_spl_heap_unserialize_locked_order() {
    assert_heap_unserialize("class H extends SplMaxHeap { public bool $once = false; function compare(mixed $a, mixed $b): int { if (!$this->once) { $this->once = true; try { $this->__unserialize([]); } catch (Throwable $e) { echo $e->getMessage(), '|'; } } return $a <=> $b; } } class Q extends SplPriorityQueue { public bool $once = false; function compare(mixed $a, mixed $b): int { if (!$this->once) { $this->once = true; try { $this->__unserialize([]); } catch (Throwable $e) { echo $e->getMessage(), '|'; } $this->__unserialize([[], ['flags' => 3, 'heap_elements' => []]]); } return $a <=> $b; } } $h = new H(); $h->insert(1); $h->insert(2); $q = new Q(); $q->insert('a', 1); $q->insert('b', 2); echo $q->getExtractFlags(), ':', $q->count();", "Heap cannot be changed when it is already being modified.|Invalid serialization data for Q object|3:2");
}

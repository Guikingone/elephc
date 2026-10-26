//! Purpose:
//! Executes scoped SPL value snapshots against the complete native runtime.
//!
//! Called from:
//! - The codegen runtime GC test module on executable supported targets.
//!
//! Key details:
//! - C observes cell identity and refcounts; no allocator or ownership helper is stubbed.
//! - Both optimizer modes must preserve capture borrowing and owned reads.

use crate::support::*;
use elephc::codegen_support::platform::Arch;

/// Links C observations into the ordinary PHP/native fixture and checks complete cleanup.
fn assert_snapshot_fixture(source: &str, expected: &str) {
    let run = || {
        let dir = make_cli_test_dir("elephc_borrowed_value");
        let (mut assembly, runtime, libraries) =
            compile_source_to_asm_with_options(source, &dir, 8_388_608, false, true);
        let c_asm = compile_c_fixture_asm(&dir, "snapshot", include_str!("borrowed_value.c"), &[]);
        append_c_fixture_asm(&mut assembly, &c_asm);
        let mut adapters = String::from("\n.text\n");
        for (name, helper) in [
            ("fixture_release_object", "__rt_decref_object"),
            ("fixture_release_cell", "__rt_decref_mixed"),
            ("fixture_cell_kind", "__rt_heap_kind"),
            ("fixture_deref", "__rt_mixed_deref"),
        ] {
            let symbol = target().extern_symbol(name);
            adapters.push_str(&format!(".globl {symbol}\n{symbol}:\n"));
            if target().arch == Arch::X86_64 {
                adapters.push_str("mov rax, rdi # adapt the C argument to the private runtime convention\n");
            }
            let branch = if target().arch == Arch::AArch64 { "b" } else { "jmp" };
            adapters.push_str(&format!("{branch} {helper}\n"));
        }
        assembly.push_str(&adapters);
        let output = assemble_and_run_capture(&assembly, &runtime_obj_for_asm(&runtime),
            &dir, &libraries, &default_link_paths(), &[]);
        assert!(output.success, "{}\n{}", output.stdout, output.stderr);
        assert_eq!(output.stdout, expected, "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
        let _ = std::fs::remove_dir_all(&dir);
    };
    run();
    without_ir_opt(run);
}

/// A borrowed capture must not suppress destruction and an owned read must support resurrection.
#[test]
fn test_borrowed_value_snapshot_resurrection() {
    assert_snapshot_fixture(r#"<?php
extern "c" {
    function fixture_capture_object(): void;
    function fixture_resurrect(): int;
    function fixture_saved_id(): int;
    function fixture_clear_object(): void;
}
class SnapshotObject {
    public int $id = 17;
    public bool $active = true;
    public function __destruct() {
        if ($this->active) { echo 'drop:', fixture_resurrect(), '|'; }
    }
}
function fetch_object(): SnapshotObject { global $object; $copy = [$object]; return array_pop($copy); }
function object_id(SnapshotObject $value): int { return $value->id; }
function fetch_value(): mixed { global $object; $copy = [$object]; return array_pop($copy); }
$object = new SnapshotObject();
fixture_capture_object();
if ($argc > 1000) { var_dump(fetch_object()); object_id($object); var_dump(fetch_value()); }
unset($object);
$other = new SnapshotObject();
$other->id = 29;
$other->active = false;
echo 'saved:', fixture_saved_id(), '|';
fixture_clear_object();
"#, "drop:17|saved:17|");
}

/// Captures preserve all value bits and resource identity without retaining the source at capture.
#[test]
fn test_borrowed_value_snapshot_value_and_resource_contract() {
    assert_snapshot_fixture(r#"<?php
extern "c" { function fixture_roundtrip(): int; }
function fetch_value(): mixed { global $value; $copy = [$value]; return array_pop($copy); }
// Keep both C-accessed native function bodies reachable without running their extra calls.
class SnapshotObject { public int $id = 0; }
function fetch_object(): SnapshotObject { global $object; $copy = [$object]; return array_pop($copy); }
function object_id(SnapshotObject $object): int { return $object->id; }
$object = new SnapshotObject();
if ($argc > 1000) { var_dump(fetch_value()); var_dump(fetch_object()); object_id($object); }
$value = null; echo fixture_roundtrip();
$value = false; echo fixture_roundtrip();
$value = 9223372036854775807; echo fixture_roundtrip();
$value = -0.0; echo fixture_roundtrip();
$value = "a\0b"; echo fixture_roundtrip();
$value = [1, 2]; echo fixture_roundtrip();
$value = ['a' => 3]; echo fixture_roundtrip();
$value = $object; echo fixture_roundtrip();
$value = fopen('php://temp', 'w+'); echo fixture_roundtrip();
fclose($value);
"#, "111111111");
}

/// Raw snapshot handles must have no PHP-callable or reflected method surface.
#[test]
fn test_borrowed_value_snapshot_methods_are_not_php_visible() {
    let source = r#"<?php
echo method_exists('SplHeap', '__elephcHeapSnapshot') ? '1' : '0';
echo method_exists('SplHeap', '__elephcHeapRead') ? '1' : '0';
echo method_exists('SplHeap', '__elephcHeapFreeSnapshot') ? '1' : '0';
echo method_exists('SplMaxHeap', '__elephcHeapSnapshot') ? '1' : '0';
echo method_exists('SplMaxHeap', '__elephcHeapRead') ? '1' : '0';
echo method_exists('SplMaxHeap', '__elephcHeapFreeSnapshot') ? '1' : '0';
echo method_exists('SplPriorityQueue', '__elephcHeapSnapshot') ? '1' : '0';
echo method_exists('SplPriorityQueue', '__elephcHeapRead') ? '1' : '0';
echo method_exists('SplPriorityQueue', '__elephcHeapFreeSnapshot') ? '1' : '0';
"#;
    let run = || {
        let output = compile_and_run_with_heap_debug(source);
        assert!(output.success, "{}", output.stderr);
        assert_eq!(output.stdout, "000000000", "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    };
    run();
    without_ir_opt(run);
}

//! Purpose:
//! Verifies direct instance calls for final classes and preserves virtual base dispatch.
//!
//! Called from:
//! - The codegen integration suite through `oop`.
//!
//! Key details:
//! - Emitter assertions disable inlining and cover every supported target.
//! - Runtime cases retain inheritance, nullable receivers, and literal parameter guards.

use super::*;

/// Removes only the directory allocated for the cross-target emitter fixture.
struct DispatchDirectory(std::path::PathBuf);

impl Drop for DispatchDirectory {
    /// Cleans source and generated assembly on success and assertion failure.
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

/// Selects a function's main body without including adjacent functions or runtime data.
fn function_body<'a>(assembly: &'a str, name: &str) -> &'a str {
    let start = format!("_fn_{name}:\n");
    let end = format!("_fn_{name}_epilogue:");
    assembly.split_once(&start).unwrap_or_else(|| panic!("missing {start}"))
        .1.split_once(&end).unwrap_or_else(|| panic!("missing {end}")).0
}

/// Final-class methods call their emitted implementation directly on all five targets.
#[test]
fn test_final_class_dispatch_supported_target_assembly() {
    let dir = DispatchDirectory(make_cli_test_dir("elephc_final_class_dispatch"));
    let source = dir.0.join("main.php");
    fs::write(&source, r#"<?php
class ParentDispatch {
    public function inherited(int $value): int { return $value + 1; }
    public function value(int $value): int { return $value + 2; }
}
final class SealedDispatch extends ParentDispatch {
    public function value(int $value): int { return $value + 3; }
}
function read_final(SealedDispatch $object, int $value): int { return $object->value($value); }
function read_inherited(SealedDispatch $object, int $value): int { return $object->inherited($value); }
function read_base(ParentDispatch $object, int $value): int { return $object->value($value); }
$object = new SealedDispatch();
echo read_final($object, $argc), read_inherited($object, $argc), read_base($object, $argc);
"#).unwrap();
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let output = elephc_cli_command(&dir.0)
            .args(["--ir-opt=off", "--emit-asm", "--emit", "staticlib", "--target", target])
            .arg(&source).output().unwrap();
        assert!(output.status.success(), "{target}: {}", String::from_utf8_lossy(&output.stderr));
        let assembly = fs::read_to_string(dir.0.join("main.s")).unwrap();
        let call = if target == "linux-x86_64" { "call" } else { "bl" };
        for (function, implementation) in [
            ("read_final", "_method_SealedDispatch_value"),
            ("read_inherited", "_method_ParentDispatch_inherited"),
        ] {
            let body = function_body(&assembly, function);
            assert!(body.contains(&format!("{call} {implementation}")), "{target}: {body}");
            assert!(!body.contains("_class_source_vtable_ptrs"), "{target}: {body}");
            assert!(!body.contains("_class_vtable_ptrs"), "{target}: {body}");
        }
        let base = function_body(&assembly, "read_base");
        assert!(base.contains("_class_source_vtable_ptrs") || base.contains("_class_vtable_ptrs"),
            "{target}: overridable base dispatch became direct: {base}");
    }
}

/// Direct final-class calls preserve inherited bodies, overrides, and nullable payloads.
#[test]
fn test_final_class_dispatch_runtime_and_source_abi() {
    let source = r#"<?php
class DispatchBase {
    public function inherited(int $value): int { return $value + 1; }
    public function value(): string { return 'base'; }
}
final class DispatchLeaf extends DispatchBase {
    public function value(): string { return 'leaf'; }
    public function update(int &$value, string $suffix = '!'): string {
        $value++;
        return $this->value() . $suffix;
    }
}
function from_base(DispatchBase $object): string { return $object->value(); }
function from_nullable(?DispatchLeaf $object): string { return $object?->value() ?? 'null'; }
$leaf = new DispatchLeaf();
$value = $argc;
echo $leaf->inherited($value), ':', from_base($leaf), ':', from_nullable($leaf), ':', from_nullable(null), ':';
echo $leaf->update(value: $value), ':', $value;
"#;
    assert_eq!(compile_and_run(source), "2:leaf:leaf:null:leaf!:2");
    without_ir_opt(|| assert_eq!(compile_and_run(source), "2:leaf:leaf:null:leaf!:2"));
}

/// A final override may widen a parent's false-only parameter without losing its own guard.
#[test]
fn test_final_class_dispatch_literal_parameter_guards() {
    let out = compile_and_run(r#"<?php
class LiteralDispatchBase {
    public function accept(false $value): string { return 'base'; }
}
final class LiteralDispatchLeaf extends LiteralDispatchBase {
    public function accept(bool $value): string { return $value ? 'true' : 'false'; }
    public function onlyFalse(false $value): string { return 'false-only'; }
    public function onlyNull(null $value): string { return 'null-only'; }
}
function widened(LiteralDispatchLeaf $object, bool $value): string { return $object->accept($value); }
function guarded(LiteralDispatchLeaf $object, mixed $value): void {
    try { echo $object->onlyFalse($value); } catch (TypeError $error) { echo 'false-error'; }
    echo ':';
    try { echo $object->onlyNull($value); } catch (TypeError $error) { echo 'null-error'; }
}
$leaf = new LiteralDispatchLeaf();
echo widened($leaf, $argc > 0), ':';
guarded($leaf, false);
echo ':';
guarded($leaf, null);
"#);
    assert_eq!(out, "true:false-only:null-error:false-error:null-only");
}

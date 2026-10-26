//! Purpose:
//! Verifies inherited builtin Throwable constructors have callable, layout-aware EIR bodies.
//!
//! Called from:
//! - The AST-to-EIR regression tests on every supported target.
//!
//! Key details:
//! - Ordinary subclasses add storage while Error and Exception retain their shared method owners.

use super::*;
use crate::ir::{Effects, Immediate, IrHeapKind, IrType, RuntimeCallTarget};

/// Parent calls release their ABI-created nullable argument after return on every supported target.
#[test]
fn lexical_parent_call_retires_default_argument_box_on_all_targets() {
    let source = r#"<?php
class DefaultArgumentOwner {
    public function accept(?Exception $previous = null): void {}
}
class DerivedArgumentOwner extends DefaultArgumentOwner {
    public function exercise(): void { parent::accept(); }
}
$owner = new DerivedArgumentOwner();
$owner->exercise();
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let target = Target::parse(name).unwrap();
        let module = lower_source_at_for_target(source, Path::new("main.php"), Path::new("."), target);
        let assembly = crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        let symbol = crate::names::method_symbol("DefaultArgumentOwner", "accept");
        let call = if target.arch == crate::codegen::platform::Arch::AArch64 { "bl" } else { "call" };
        let (_, tail) = assembly.split_once(&format!("{call} {symbol}")).unwrap();
        let until_return = tail.lines().take_while(|line| line.trim() != "ret").collect::<Vec<_>>().join("\n");
        assert!(until_return.contains("__rt_decref_mixed"), "{name}: {until_return}");
    }
}

/// Every eval bridge owns a nullable previous result before handing it back across the native ABI.
#[test]
fn compact_throwable_eval_previous_getter_boxes_both_return_arms_on_all_targets() {
    let source = r#"<?php
function throwEvalGetterException(): void { throw new RuntimeException("native"); }
$source = 'try { throwEvalGetterException(); } catch (RuntimeException $error) {
    echo $error->getPrevious() === null ? "end" : "previous";
}' . ' // ' . $argc;
eval($source);
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = lower_source_at_for_target(
            source, Path::new("main.php"), Path::new("."), Target::parse(name).unwrap(),
        );
        let assembly = crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        // The bridge body reads the previous slot itself (no `__rt_throwable_previous` lookup any
        // more) and returns an OWNED Mixed on every arm: a raw object pointer is boxed after its
        // heap kind is inspected, an already-boxed constructor cell is retained and handed back,
        // and an empty slot boxes null. x86_64 spells the arm labels with an `_x` suffix.
        let suffix = if name == "linux-x86_64" { "_x" } else { "" };
        let start = assembly.find("__elephc_eval_builtin_throwable_getprevious:").unwrap();
        let body = &assembly[start..];
        let boxed = body.find(&format!("__elephc_eval_builtin_throwable_getprevious_boxed{suffix}:")).unwrap();
        let null = body.find(&format!("__elephc_eval_builtin_throwable_getprevious_null{suffix}:")).unwrap();
        let kind_check = body.find("__rt_heap_kind").unwrap();
        let object_box = body.find("__rt_mixed_from_value").unwrap();
        assert!(kind_check < object_box && object_box < boxed, "{name}: the object arm inspects the heap kind, then boxes");
        assert!(body[boxed..null].contains("__rt_incref"), "{name}: the already-boxed arm hands back a retained cell");
        assert!(body[null..].contains("__rt_mixed_from_value"), "{name}: the empty arm boxes null");
    }
}

/// Both constructor roots remain emitted and use normalized boxed previous operands on all targets.
#[test]
fn inherited_throwable_constructors_have_layout_aware_bodies() {
    let source = r#"<?php
class ExtendedException extends Exception { public int $marker = 42; }
class ExtendedError extends Error { public string $marker = "kept"; }
$error = new ExtendedException("outer", 1, new ExtendedError("inner"));
$error->__construct("again", 2, $error->getPrevious());
echo $error->getMessage();
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = lower_source_at_for_target(source, Path::new("main.php"), Path::new("."), Target::parse(name).unwrap());
        for owner in ["Error", "Exception"] {
            let constructor = module.class_methods.iter()
                .find(|function| function.name == format!("{owner}::__construct"))
                .unwrap_or_else(|| panic!("{name}: missing {owner} constructor"));
            let initialize = constructor.instructions.iter().find(|inst| {
                inst.immediate == Some(Immediate::RuntimeCall(RuntimeCallTarget::ThrowableInitialize))
            }).expect("constructor must initialize the receiver's concrete layout");
            assert_eq!(initialize.operands.len(), 4, "{name}");
            let expected = [IrType::Heap(IrHeapKind::Object), IrType::Str, IrType::I64,
                IrType::Heap(IrHeapKind::Mixed)];
            for (operand, expected) in initialize.operands.iter().zip(expected) {
                assert_eq!(constructor.value(*operand).unwrap().ir_type, expected, "{name}: {owner}");
            }
            let signature = &module.class_infos[owner].methods["__construct"];
            assert_eq!(signature.params[0].1, crate::types::PhpType::Str, "{name}: {owner}");
            assert_eq!(signature.params[1].1, crate::types::PhpType::Int, "{name}: {owner}");
            assert!(signature.declared_params.iter().take(3).all(|declared| *declared), "{name}: {owner}");
            assert!(initialize.effects.contains(Effects::MAY_THROW | Effects::REFCOUNT_OP), "{name}");
        }
    }
}

/// An untyped subclass constructor cannot specialize a typed ancestor's nullable previous parameter.
#[test]
fn subclass_constructor_inference_preserves_ancestor_parameter_contracts() {
    let source = r#"<?php
class InferredPreviousException extends Exception {
    public function __construct($message, $code, $previous) {
        $this->message = $message;
        $this->code = $code;
        $this->previous = $previous;
    }
}
$previous = new Error("inner");
$child = new InferredPreviousException("child", 1, $previous);
$parent = new Exception("parent", 2, null);
$parent->__construct("again", 3, $child->getPrevious());
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = lower_source_at_for_target(source, Path::new("main.php"), Path::new("."), Target::parse(name).unwrap());
        let signature = &module.class_infos["Exception"].methods["__construct"];
        assert_eq!(signature.params[2].1.codegen_repr(), crate::types::PhpType::Mixed, "{name}");
        assert!(signature.declared_params[2], "{name}");
    }
}

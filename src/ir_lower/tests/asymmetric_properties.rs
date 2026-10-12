//! Purpose:
//! Regression coverage for conditional readonly property writes on all supported targets.
//!
//! Called from:
//! - The AST-to-EIR unit suite through `crate::ir_lower::tests`.
//!
//! Key details:
//! - Readonly violations belong to the fallback branch, not the initialized-value branch.
//! - Emission must also support explicit and implicit setter reflection metadata.

use crate::codegen::platform::Target;
use crate::ir::{Op, Terminator};
use std::path::Path;

/// Verifies the readonly fallback graph and target-aware reflection emission.
fn check_conditional_readonly_write(target: &str) {
    let source = r#"<?php
class ConditionalReadonly {
    public(set) readonly int $explicit;
    public readonly int $implicit;
    public readonly ?int $nullable;
}
function fallback(): int { echo "fallback"; return 9; }
function update(ConditionalReadonly $box): void { $box->implicit ??= fallback(); }
function updateNullable(ConditionalReadonly $box): void { $box->nullable ??= fallback(); }
function overwrite(ConditionalReadonly $box): void { $box->implicit = fallback(); }
function initializePublic(ConditionalReadonly $box): void { $box->explicit = fallback(); }
function initializeUnion(ConditionalReadonly|false $box): void { $box->explicit = fallback(); }
function coalesceUnion(ConditionalReadonly|false $box): void { $box->explicit ??= fallback(); }
function coalesceImplicitUnion(ConditionalReadonly|false $box): void { $box->implicit ??= fallback(); }
class UnionHook { public int $value { get => 7; } }
function coalesceHookUnion(UnionHook|false $box): int { return $box->value ?? 9; }
readonly class LegacyReadonly {
    public $id;
    public function __construct(int $id) { $this->id = $id; }
}
class ReadonlyBase { public readonly int $id; }
class ReadonlyChild extends ReadonlyBase {
    public protected(set) readonly int $id;
    public function __construct() { $this->id = 7; }
}
abstract class AbstractGetter { abstract public int $id { get; } }
class ReadonlyGetter extends AbstractGetter {
    public readonly int $id;
    public function __construct() { $this->id = 42; }
}
abstract class AbstractSetter { abstract public int $id { get; set; } }
abstract class AbstractReadonlyStorage extends AbstractSetter { public public(set) readonly int $id; }
class ReadonlyStorageDescendant extends AbstractReadonlyStorage {}
echo (new ReflectionClass(AbstractReadonlyStorage::class))->isAbstract() ? '' : 'bad';
echo (new ReflectionClass(ReadonlyStorageDescendant::class))->isAbstract() ? 'bad' : '';
function acceptAbstractReadonlyStorage(): void {
    eval('abstract class DynamicAbstractReadonlyStorage extends AbstractSetter { public public(set) readonly int $id; }');
}
class NativeMutableParent { public int $id; }
function rejectReadonlyAddition(): void {
    eval('class DynamicReadonlyChild extends NativeMutableParent { public public(set) readonly int $id; }');
}
function receiver(): ConditionalReadonly { echo "receiver"; return new ConditionalReadonly(); }
function updateTemporary(): void { receiver()->implicit ??= fallback(); }
$box = new ConditionalReadonly();
$legacy = new LegacyReadonly(7);
echo $legacy->id;
echo (new ReadonlyGetter())->id;
try { update($box); } catch (Error $e) { echo "error"; }
$properties = (new ReflectionClass(ConditionalReadonly::class))->getProperties(ReflectionProperty::IS_PROTECTED_SET);
echo count($properties);
"#;
    let module = super::lower_source_at_for_target(
        source, Path::new("main.php"), Path::new("."), Target::parse(target).unwrap(),
    );
    for name in ["update", "updateNullable"] {
        let function = module.functions.iter().find(|function| function.name == name).unwrap();
        let insert = function.blocks.iter().find(|block| block.name == "coalesce_assign.default").unwrap();
        let Some(Terminator::CondBr { then_target, else_target, .. }) = &insert.terminator else {
            panic!("{target}: fallback checks initialization before choosing its Error");
        };
        for destination in [then_target, else_target] {
            let arm = function.blocks.iter().find(|block| block.id == *destination).unwrap();
            assert!(matches!(arm.terminator, Some(Terminator::Throw { .. })),
                "{target}: initialized overwrite and inaccessible initialization both throw");
        }
        assert!(insert.instructions.iter().any(|id| function.instruction(*id).unwrap().op == Op::PropInitialized),
            "{target}: a fallback may target an initialized null readonly slot");
        let merge = function.blocks.iter().find(|block| block.name == "coalesce_assign.merge").unwrap();
        assert!(!matches!(merge.terminator, Some(Terminator::Throw { .. })), "{target}: keep does not throw");
        let probe = function.blocks.iter().find(|block| block.name == "coalesce.property.merge").unwrap();
        assert!(probe.instructions.iter().any(|id| function.instruction(*id).unwrap().op == Op::UnsetLocal),
            "{target}: the consumed probe cannot remain in its owner slot");
    }
    let overwrite = module.functions.iter().find(|function| function.name == "overwrite").unwrap();
    assert!(overwrite.blocks.iter().any(|block| matches!(block.terminator,
        Some(Terminator::Throw { .. }))), "{target}: direct readonly writes remain catchable");
    let union = module.functions.iter().find(|function| function.name == "initializeUnion").unwrap();
    assert!(union.instructions.iter().any(|inst| inst.op == Op::TypePredicate
        && inst.immediate == Some(crate::ir::Immediate::TypePredicate(crate::ir::PhpTypePredicate::Object))),
        "{target}: non-object receivers are rejected before inspecting readonly state");
    for name in ["coalesceUnion", "coalesceImplicitUnion"] {
        let function = module.functions.iter().find(|function| function.name == name).unwrap();
        assert!(function.blocks.iter().any(|block| block.name == "coalesce.property.uninitialized"),
            "{target}: a boxed union coalesce must suppress the uninitialized value read");
        assert!(function.instructions.iter().any(|inst| inst.op == Op::PropInitialized),
            "{target}: object and scalar arms use the safe runtime initialization probe");
    }
    let hooked = module.functions.iter().find(|function| function.name == "coalesceHookUnion").unwrap();
    assert!(!hooked.instructions.iter().any(|inst| inst.op == Op::PropInitialized),
        "{target}: virtual union getters never probe their uninitialized backing slot");
    assert!(hooked.instructions.iter().any(|inst| inst.op == Op::MixedUnbox),
        "{target}: the guarded getter borrows no boxed cell as an object pointer");
    for function in [
        module.functions.iter().find(|function| function.name == "initializePublic").unwrap(),
        module.functions.iter().find(|function| function.name == "initializeUnion").unwrap(),
        module.class_methods.iter().find(|function| function.name == "ReadonlyChild::__construct").unwrap(),
        module.class_methods.iter().find(|function| function.name == "ReadonlyGetter::__construct").unwrap(),
    ] {
        assert!(function.instructions.iter().any(|inst| inst.op == Op::PropInitialized), "{target}");
        let initialize = function.blocks.iter().find(|block| block.name == "readonly.write.uninitialized").unwrap();
        assert!(initialize.instructions.iter().any(|id| function.instruction(*id).unwrap().op == Op::PropSet),
            "{target}: authorized first initialization publishes the property");
        let overwrite = function.blocks.iter().find(|block| block.name == "readonly.write.initialized").unwrap();
        assert!(matches!(overwrite.terminator, Some(Terminator::Throw { .. })),
            "{target}: authorized setters still reject an initialized readonly slot");
    }
    let legacy = module.class_methods.iter().find(|function| function.name == "LegacyReadonly::__construct").unwrap();
    assert!(!legacy.instructions.iter().any(|inst| inst.op == Op::PropInitialized),
        "{target}: the legacy untyped constructor initializes an implicitly null slot");
    let temporary = module.functions.iter().find(|function| function.name == "updateTemporary").unwrap();
    assert!(temporary.blocks.iter().flat_map(|block| &block.instructions)
        .any(|id| temporary.instruction(*id).unwrap().op == Op::PushCallOperandOwner),
        "{target}: a throwing fallback must root the temporary receiver");
    crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
}

/// Conditional readonly writes and reflection metadata emit on macOS ARM64.
#[test]
fn conditional_readonly_write_macos() { check_conditional_readonly_write("macos-aarch64"); }

/// Conditional readonly writes and reflection metadata emit on iOS devices.
#[test]
fn conditional_readonly_write_ios_device() { check_conditional_readonly_write("ios-arm64"); }

/// Conditional readonly writes and reflection metadata emit on the iOS Simulator.
#[test]
fn conditional_readonly_write_ios_simulator() { check_conditional_readonly_write("ios-sim-arm64"); }

/// Conditional readonly writes and reflection metadata emit on Linux ARM64.
#[test]
fn conditional_readonly_write_linux_arm64() { check_conditional_readonly_write("linux-aarch64"); }

/// Conditional readonly writes and reflection metadata emit on Linux x86_64.
#[test]
fn conditional_readonly_write_linux_x86_64() { check_conditional_readonly_write("linux-x86_64"); }

//! Purpose:
//! Structural coverage for Closure rebinding operands and bound descriptors in lowered EIR.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Static `Closure::bind()` extracts invalidated closure locals from boxed Mixed storage.
//! - `closure_bind` allocates a fresh descriptor, so its value must be an owned temporary the
//!   call site publishes in the unwind chain and retires on every supported target.

use std::path::Path;

use crate::codegen::platform::Target;
use crate::ir::{Immediate, LocalKind, Op, Ownership};
use crate::types::PhpType;

/// Static `Closure::bind()` passes an extracted descriptor when a global write invalidated the
/// closure local's compile-time identity.
#[test]
fn static_closure_bind_unboxes_invalidated_closure_storage_on_every_target() {
    let source = r#"<?php
function replaceGlobalClosure(): void {
    global $closure;
    $closure = function () { return $this->value; };
}
class Holder {
    public int $value = 7;
}
$closure = function () { return $this->value; };
try {
    replaceGlobalClosure();
} catch (Error $error) {}
$bound = Closure::bind($closure, new Holder());
echo $bound();
"#;
    for name in [
        "macos-aarch64",
        "ios-arm64",
        "ios-sim-arm64",
        "linux-aarch64",
        "linux-x86_64",
    ] {
        let module = super::lower_source_at_for_target(
            source,
            Path::new("main.php"),
            Path::new("."),
            Target::parse(name).unwrap(),
        );
        let main = module
            .functions
            .iter()
            .find(|function| function.flags.is_main)
            .unwrap();
        let extraction = main
            .instructions
            .iter()
            .find(|inst| inst.op == Op::MixedUnbox && inst.result_php_type == PhpType::Callable)
            .unwrap_or_else(|| panic!("{name}: invalidated closure storage was not extracted"));
        let extracted = extraction.result.expect("MixedUnbox produces a descriptor");
        assert_eq!(
            main.value(extracted).unwrap().ownership,
            Ownership::Owned,
            "{name}: the extracted descriptor owns its retained lease",
        );
        let bind = main
            .instructions
            .iter()
            .find(|inst| inst.op == Op::ClosureBind)
            .expect("the extracted descriptor is rebound");
        let mut bind_source = bind.operands[0];
        while let Some(producer) = main.instructions.iter().find(|inst| {
            inst.result == Some(bind_source) && matches!(inst.op, Op::Acquire | Op::Borrow)
        }) {
            bind_source = producer.operands[0];
        }
        assert_eq!(
            bind_source, extracted,
            "{name}: ClosureBind must consume the extracted descriptor, not its Mixed box",
        );
        crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    }
}

/// `Closure::call()` roots its freshly bound descriptor and retires it after the invocation.
#[test]
fn closure_call_roots_and_retires_its_bound_descriptor_on_every_target() {
    let source = r#"<?php
class Holder {
    public int $value = 7;
}
function callBound($extra) {
    $closure = function ($bonus) { return $this->value + $bonus; };
    return $closure->call(new Holder(), $extra);
}
echo callBound(1);
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source, Path::new("main.php"), Path::new("."), Target::parse(name).unwrap(),
        );
        let caller = module.functions.iter()
            .find(|function| function.name == "callBound").unwrap();
        let bind = caller.instructions.iter()
            .find(|inst| inst.op == Op::ClosureBind)
            .expect("the closure is rebound before the call");
        let bound = bind.result.expect("closure_bind produces the bound descriptor");
        assert_eq!(
            caller.value(bound).unwrap().ownership,
            Ownership::Owned,
            "{name}: a fresh bound descriptor is an owned temporary",
        );
        // The root stores either the bind result or its explicit acquire into a frame slot.
        let mut owners = vec![bound];
        owners.extend(caller.instructions.iter().filter_map(|inst| {
            (inst.op == Op::Acquire && inst.operands == [bound]).then_some(inst.result).flatten()
        }));
        let store = caller.instructions.iter()
            .find(|inst| inst.op == Op::StoreLocal
                && inst.operands.iter().all(|operand| owners.contains(operand)))
            .expect("the bound descriptor is stored into an operand-owner slot");
        let Some(Immediate::LocalSlot(slot)) = store.immediate else { unreachable!() };
        let local = &caller.locals[slot.as_raw() as usize];
        assert_eq!(local.php_type.codegen_repr(), PhpType::Callable, "{name}");
        assert_eq!(local.kind, LocalKind::HiddenTemp, "{name}");
        let at = |op: Op| caller.instructions.iter().position(|inst| {
            inst.op == op && inst.immediate == Some(Immediate::LocalSlot(slot))
        });
        let published = at(Op::PushCallOperandOwner)
            .unwrap_or_else(|| panic!("{name}: the bound descriptor needs an unwind owner"));
        let retired = at(Op::ReleaseLocalSlot)
            .unwrap_or_else(|| panic!("{name}: the bound descriptor is never released"));
        assert!(published < retired, "{name}: publication precedes retirement");
        crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    }
}

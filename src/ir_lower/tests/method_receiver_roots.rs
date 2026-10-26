//! Purpose:
//! Keeps immutable this receivers inlineable without dropping eval-widened object leases.
//!
//! Called from:
//! - The AST-to-EIR unit suite on every supported target.
//!
//! Key details:
//! - A later eval barrier changes ownership of even an earlier typed receiver load.

use crate::ir::{Op, ValueDef};

/// Stable this loads stay borrowed, while a later eval keeps their independent owner rooted.
#[test]
fn method_receiver_roots_respect_later_eval_on_all_targets() {
    let source = r#"<?php
class ReceiverRoot {
    public function target(int $value): int { return $value; }
    public function plain(int $value): int { return $this->target($value); }
    public function guarded(int $value, string $source): int {
        $result = $this->target($value);
        eval($source);
        return $result;
    }
}
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source, std::path::Path::new("main.php"), std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        for (name, expected) in [("ReceiverRoot::plain", Op::LoadLocal), ("ReceiverRoot::guarded", Op::Acquire)] {
            let method = module.class_methods.iter().find(|function| function.name == name).unwrap();
            let call = method.instructions.iter().find(|instruction| instruction.op == Op::MethodCall).unwrap();
            let ValueDef::Instruction { inst, .. } = method.value(call.operands[0]).unwrap().def
                else { panic!("{target}: receiver has no defining instruction"); };
            assert_eq!(method.instruction(inst).unwrap().op, expected, "{target}: {name}");
            if expected == Op::Acquire {
                assert!(method.instructions.iter().any(|instruction| instruction.op == Op::PushCallOperandOwner), "{target}");
            }
        }
        crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
    }
}

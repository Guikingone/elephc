//! Purpose:
//! Verifies object-aware nested appends and deep false updates on every target.
//!
//! Called from:
//! - AST-to-EIR unit tests through Rust's test harness.
//!
//! Key details:
//! - Objects use a null-key method call while arrays retain attached boxed mutation.
//! - Native emitters must support the same branch graph on all five targets.

/// Every target emits object-aware append dispatch and accepts deeper false promotion.
#[test]
fn nested_append_review_all_targets() {
    let source = r#"<?php
class Bag implements ArrayAccess {
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetUnset(mixed $key): void {}
    public function offsetSet(mixed $key, mixed $value): void { echo $value; }
}
$items = [new Bag(), [1], null];
for ($i = 0; $i < 3; $i++) { $items[$i][] = 2; }
$deep = [false]; ++$deep[0][0][1];
"#;
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let target = crate::codegen::platform::Target::parse(name).unwrap();
        let module = super::lower_source_at_for_target(source,
            std::path::Path::new("main.php"), std::path::Path::new("."), target);
        let ir = crate::ir::print_module(&module);
        assert!(ir.contains("nested.append.arrayaccess"), "{name}: {ir}");
        assert!(ir.contains("mixed_array_append"), "{name}: {ir}");
        crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
    }
}

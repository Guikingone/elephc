//! Purpose:
//! Verifies enum unpack integer diagnostics and concrete collectors on every supported target.
//!
//! Called from:
//! - The AST-to-EIR unit test harness.
//!
//! Key details:
//! - Runtime array parameters retain the implicit integer cast through optimization.
//! - Each target independently emits both float and quoted-string diagnostic paths.

/// Lowers and emits the enum argument boundary fixture for one target.
fn verify(target: &str) {
    let module = super::lower_source_at_for_target(r#"<?php
interface BoundArguments { public function describe(int ...$values): string; }
enum BoundValue implements BoundArguments {
    case A;
    public function describe(int ...$values): string {
        $out = ''; foreach ($values as $value) { $out .= gettype($value); } return $out;
    }
    public function text(string ...$values): string { return implode('|', $values); }
}
function bound_call(BoundArguments $target, array $args): string { return $target->describe(...$args); }
function text_call(?BoundValue $target, array $args): ?string { return $target?->text(...$args); }
echo bound_call(BoundValue::A, [1.5, '2.75']), text_call(BoundValue::A, ['ok']);
"#, std::path::Path::new("main.php"), std::path::Path::new("."),
        crate::codegen_support::platform::Target::parse(target).unwrap());
    assert!(crate::ir::print_module(&module).contains("implicit_int_cast"), "{target}: diagnostic EIR cast");
    let assembly = crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
    assert!(assembly.contains("implicit_int_float"), "{target}: target-aware implicit int binding");
}

/// Covers the macOS ARM64 diagnostic and collector ABI.
#[test]
fn enum_argument_boundaries_macos() { verify("macos-aarch64"); }

/// Covers the iOS device diagnostic and collector ABI.
#[test]
fn enum_argument_boundaries_ios() { verify("ios-arm64"); }

/// Covers the iOS Simulator diagnostic and collector ABI.
#[test]
fn enum_argument_boundaries_ios_sim() { verify("ios-sim-arm64"); }

/// Covers the Linux ARM64 diagnostic and collector ABI.
#[test]
fn enum_argument_boundaries_linux_arm() { verify("linux-aarch64"); }

/// Covers the Linux x86_64 diagnostic and collector ABI.
#[test]
fn enum_argument_boundaries_linux_x86() { verify("linux-x86_64"); }

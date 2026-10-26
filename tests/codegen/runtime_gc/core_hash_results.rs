//! Purpose:
//! Regressions for the physical hash result of object-member introspection.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - First-class builtin shims must honor the same result contract as checked direct calls.
//! - Full key order, visibility mangling and payload values are asserted with a clean heap.

use crate::support::compile_and_run_with_heap_debug;

/// Checks complete observable member values and final heap ownership.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves visibility-mangled members in a checked case-insensitive builtin call.
#[test]
fn test_core_hash_results_direct() {
    assert_clean(r#"<?php
class C {public int $n=7;private string $label='kept';}
function members(object $o):array {return \GeT_MaNgLeD_ObJeCt_VaRs($o);}
$d=members(new C());echo json_encode(array_keys($d)),':',$d['n'],':',$d["\0C\0label"];
"#, "[\"n\",\"\\u0000C\\u0000label\"]:7:kept");
}

/// Preserves the same physical hash and owners through a first-class builtin shim.
#[test]
fn test_core_hash_results_callable() {
    assert_clean(r#"<?php
class C {public int $n=7;private string $label='kept';}
$get=\GeT_MaNgLeD_ObJeCt_VaRs(...);$d=$get(new C());echo json_encode(array_keys($d)),':',$d['n'],':',$d["\0C\0label"];
"#, "[\"n\",\"\\u0000C\\u0000label\"]:7:kept");
}

/// Injected bodies have no checked call-site entry: their EIR must still describe the raw hash.
#[test]
fn test_core_hash_results_without_checked_call_site_on_every_target() {
    use elephc::codegen_support::platform::Target;
    use elephc::ir::Op;
    use elephc::types::PhpType;

    let tokens = elephc::lexer::tokenize(
        "<?php class C {public int $n=7;} function snapshot(C $o):array {return get_mangled_object_vars($o);} snapshot(new C());",
    ).expect("tokenize");
    let program = elephc::parser::parse(&tokens).expect("parse");
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let target = Target::parse(name).expect("supported target");
        let mut checked = elephc::types::checker::check_types(&program, target.clone())
            .expect("check");
        assert!(!checked.builtin_call_types.is_empty(), "{name}: checked control");
        checked.builtin_call_types.clear();
        let module = elephc::ir_lower::lower_program(&program, &checked, target, false)
            .expect("lower unchecked builtin site");
        let function = module.functions.iter().find(|function| function.name == "snapshot")
            .expect("snapshot function");
        let calls: Vec<_> = function.blocks.iter()
            .flat_map(|block| block.instructions.iter())
            .filter_map(|id| function.instruction(*id))
            .filter(|instruction| matches!(&instruction.op, Op::CoreBuiltin))
            .collect();
        assert_eq!(calls.len(), 1, "{name}: actual Core builtin instruction");
        assert_eq!(calls[0].result_php_type, PhpType::AssocArray {
            key: Box::new(PhpType::Str), value: Box::new(PhpType::Mixed),
        }, "{name}: the backend returns a hash, never an unboxed pointer labeled Mixed");
    }
}

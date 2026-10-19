//! Purpose:
//! Counts actual catalog publications while exercising the generated eval bridge.
//!
//! Called from:
//! - The focused deferred-catalog codegen integration suite.
//!
//! Key details:
//! - Test-only assembly wraps the publication ABI without changing its arguments or result.
//! - Float cells must postpone publication; string cells must request the full catalog.
//! - This detects drift between the Rust proof's tags and the emitted runtime wire tags.

use crate::support::*;
use super::FixtureDir;

/// Instruments catalog publication without changing the real bridge call's ABI or lifetime.
fn run_publication_probe(source: &str) -> String {
    let dir = FixtureDir(make_cli_test_dir("elephc_deferred_catalog_probe"));
    let (user, runtime, requirements) =
        compile_source_to_asm_with_options(source, &dir.0, 8 * 1024 * 1024, false, true);
    let target = target();
    let real = target.extern_symbol("__elephc_eval_context_publish_native_catalog");
    let probe = target.extern_symbol("__elephc_test_catalog_publications");
    let wrapper = "Ldeferred_catalog_publish_probe";
    let counter = "Ldeferred_catalog_publication_count";
    let mnemonic = if target.arch == Arch::AArch64 { "bl" } else { "call" };
    let needle = format!("    {mnemonic} {real}\n");
    assert!(user.contains(&needle), "catalog publication call must remain instrumentable");
    let mut user = user.replace(&needle, &format!("    {mnemonic} {wrapper}\n"));
    let code = if target.arch == Arch::AArch64 {
        format!(".text\n.globl {probe}\n{probe}:\n    adrp x16, {counter}@PAGE\n    add x16, x16, {counter}@PAGEOFF\n    ldr x0, [x16]\n    ret\n{wrapper}:\n    adrp x16, {counter}@PAGE\n    add x16, x16, {counter}@PAGEOFF\n    ldr x17, [x16]\n    add x17, x17, #1\n    str x17, [x16]\n    b {real}\n.comm {counter}, 8\n")
    } else {
        format!(".text\n.globl {probe}\n{probe}:\n    mov rax, QWORD PTR [rip + {counter}]\n    ret\n{wrapper}:\n    inc QWORD PTR [rip + {counter}]\n    jmp {real}\n.comm {counter}, 8\n")
    };
    user.push_str(&target.transform_assembly(&code));
    let runtime_obj = runtime_obj_for_asm(&runtime);
    let output = assemble_and_run(&user, &runtime_obj, &dir.0,
        &requirements, &default_link_paths(), &[]);
    output
}

/// Verifies both runtime tag gates and late publication with a target-native ABI counter.
#[test]
fn test_deferred_native_catalog_float_then_string_publication_count() {
    let source = r#"<?php
extern function __elephc_test_catalog_publications(): int;
function run($initial) {
    global $code;
    $sum = $initial;
    eval($code);
    return $sum;
}
$code = str_repeat('$sum=$sum+3;', $argc);
echo run(1.5), ':', __elephc_test_catalog_publications(), '|';
echo run('1.5'), ':', __elephc_test_catalog_publications();
"#;
    assert_eq!(run_publication_probe(source), "4.5:0|4.5:1");
}

/// A hash-mask collision can retain an unused object until the next scope flush releases it.
#[test]
fn test_deferred_native_catalog_initializes_before_scope_flush_destructor() {
    assert_eq!(elephc_magician::eval_ir::eval_scope_name_mask("sum"),
        elephc_magician::eval_ir::eval_scope_name_mask("held73"));
    let source = r#"<?php
extern function __elephc_test_catalog_publications(): int;
class Victim {
    public function __destruct() { echo 'd', __elephc_test_catalog_publications(); }
}
function run() {
    global $code;
    $held73 = new Victim();
    $sum = 0;
    eval($code);
    unset($held73);
    eval($code);
    echo ':', __elephc_test_catalog_publications();
}
$code = str_repeat('$sum=$sum+3;', $argc);
run();
"#;
    assert_eq!(run_publication_probe(source), "d1:1");
}

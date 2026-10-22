//! Purpose:
//! Verifies that appends follow the final boxed representation of reference-capable locals.
//!
//! Called from:
//! - The AST-to-EIR unit test module.
//!
//! Key details:
//! - Boxed append borrows directly; native layouts transfer the unboxed lease on storeback.

/// Final storage normalization selects boxed or native append coherently on every target.
#[test]
fn ref_local_append_uses_final_storage_on_all_targets() {
    use crate::ir::{Immediate, Op};
    use crate::types::PhpType;
    let source = r#"<?php
function touchUnknown(&$items): void { $items[0] = 7; }
function touchArray(array &$items): void { $items[0] = 8; }
function buildUnknown(int $n): void {
    $items = [];
    for ($i = 0; $i < $n; $i++) { $items[] = $i; }
    touchUnknown($items); echo count($items);
}
function buildArray(int $n): void {
    $items = [];
    for ($i = 0; $i < $n; $i++) { $items[] = $i; }
    touchArray($items); echo count($items);
}
function buildAssoc(int $n): void {
    $items = ["keep" => 1];
    for ($i = 0; $i < $n; $i++) { $items[] = $i; }
    touchUnknown($items); echo count($items);
}
function buildMaybe(int $n, bool $run): void {
    $items = [];
    for ($i = 0; $i < $n; $i++) { $items[] = $i; }
    if ($run) { touchUnknown($items); }
    echo count($items);
}
function buildClosure(int $n): void {
    $run = function () use ($n): void {
        $items = [];
        for ($i = 0; $i < $n; $i++) { $items[] = $i; }
        if ($n < 0) { touchUnknown($items); }
        echo count($items);
    };
    $run();
}
function joinNativeAppend(int $flag): void {
    $items = ["b", "a"];
    if ($flag < 0) { touchUnknown($items); }
    $items[] = "c"; echo count($items);
}
function earlyNativeAppend(int $flag): void {
    $items = ["b", "a"];
    if ($flag < 0) { touchUnknown($items); return; }
    $items[] = "c"; echo count($items);
}

function earlyNativeKey(int $flag): void {
    $items = ["b", "a"];
    if ($flag < 0) { touchUnknown($items); return; }
    $items["k"] = 1;
    echo "key:", count($items), "\n";
}
function earlyNativeAssoc(int $flag): void {
    $items = ["keep" => 1];
    if ($flag < 0) { touchUnknown($items); return; }
    $items["k"] = 2;
    echo "assoc:", $items["keep"], "\n";
}
function earlyNativeForeach(int $flag): void {
    $items = ["b", "a"];
    if ($flag < 0) { touchUnknown($items); return; }
    foreach ($items as &$v) { $v = "z"; }
    echo "each:", $items[0], "\n";
}
function earlyNativeIndex(int $flag): void {
    $items = ["b", "a"];
    if ($flag < 0) { touchUnknown($items); return; }
    $items[0] = "z";
    echo "idx:", $items[0], "\n";
}
function earlyNativeUnset(int $flag): void {
    $items = ["keep" => 1, "drop" => 2];
    if ($flag < 0) { touchUnknown($items); return; }
    unset($items["drop"]);
    echo "unset:", count($items), "\n";
}
function earlyNativeNested(int $flag): void {
    $items = ["keep" => ["a"]];
    if ($flag < 0) { touchUnknown($items); return; }
    $items["keep"][] = "b";
    echo "nested:", count($items["keep"]), "\n";
}
function replaceAppendScalar(&$items, string $pending): string { $items = 7; return $pending; }
function appendReplaced(): void {
    $items = [1];
    try { $items[] = replaceAppendScalar($items, str_repeat("x", 400000)); } catch (Error $e) {}
}
class AppendTarget {
    public function build(int $n): void {
        $items = [];
        for ($i = 0; $i < $n; $i++) { $items[] = $i; }
        $this->touch($items);
        for ($i = $n; $i < $n + 1; $i++) { $items[] = $i; }
        echo count($items);
    }
    public function touch(&$items): void { $items[0] = 9; }
}
buildUnknown($argc); buildArray($argc); buildAssoc($argc); (new AppendTarget())->build($argc);
buildMaybe($argc, $argc < 0);
buildClosure($argc);
appendReplaced();
$items = [];
for ($i = 0; $i < $argc; $i++) { $items[] = $i; }
touchUnknown($items); echo count($items);
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source, std::path::Path::new("main.php"), std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        let mut boxed = 0;
        let mut native = 0;
        for function in module.functions.iter()
            .chain(module.class_methods.iter())
            .chain(module.closures.iter())
        {
            if function.name == "AppendTarget::build" {
                assert!(!function.instructions.iter().any(|inst| inst.op == Op::MixedClone),
                    "{target}: appending after reference promotion must not clone the boxed array");
                let conversion = function.instructions.iter().find(|inst| {
                    inst.op == Op::ArrayToMixed
                }).expect("the reused loop counter widens the array element layout");
                let source = function.value(conversion.operands[0]).unwrap();
                let crate::ir::ValueDef::Instruction { block, index, inst } = source.def else {
                    panic!("{target}: conversion source must load the local");
                };
                let load = function.instruction(inst).unwrap();
                let Some(Immediate::LocalSlot(slot)) = load.immediate else {
                    panic!("{target}: conversion source must identify the local slot");
                };
                let conversion_index = function.value(conversion.result.unwrap()).unwrap().def;
                let crate::ir::ValueDef::Instruction { index: conversion_index, .. } = conversion_index else {
                    panic!("{target}: conversion must define a value");
                };
                assert!(function.block(block).unwrap().instructions
                    [index as usize + 1..conversion_index as usize]
                    .iter().any(|id| {
                        let inst = function.instruction(*id).unwrap();
                        inst.op == Op::ReleaseLocalSlot
                            && inst.immediate == Some(Immediate::LocalSlot(slot))
                    }), "{target}: release the old boxed array before converting its owned unbox");
            }
            if function.name == "appendReplaced" {
                assert!(function.instructions.iter().any(|inst| inst.op == Op::TypePredicate),
                    "{target}: RHS replacement must retain the scalar/string parent guards");
                let error = function.blocks.iter().find(|block| {
                    block.name == "nested.write.scalar.error"
                }).expect("the runtime-shaped parent needs a scalar error block");
                let pop = error.instructions.iter().position(|id| {
                    function.instruction(*id).is_some_and(|inst| inst.op == Op::PopCallOperandOwner)
                }).expect("the rejected string result must detach its unwind pin");
                let release = error.instructions.iter().position(|id| {
                    function.instruction(*id).is_some_and(|inst| inst.op == Op::Release)
                }).expect("the rejected string result must be released");
                assert!(pop < release, "{target}: detach the unwind pin before releasing its string");
            }
            for block in &function.blocks {
                for id in &block.instructions {
                    let append = function.instruction(*id).unwrap();
                    if !matches!(append.op, Op::ArrayPush | Op::HashAppend | Op::MixedArrayAppend) { continue; }
                    let value = function.value(append.operands[0]).unwrap();
                    let crate::ir::ValueDef::Instruction { inst, .. } = value.def else { continue; };
                    let load = function.instruction(inst).unwrap();
                    let Some(Immediate::LocalSlot(slot)) = load.immediate else { continue; };
                    if function.locals[slot.as_raw() as usize].php_type.codegen_repr() != PhpType::Mixed { continue; }
                    if append.op == Op::ArrayPush {
                        assert!(matches!(value.php_type.codegen_repr(), PhpType::Array(_)),
                            "{target}: native consumers retain an indexed-array view");
                        native += 1;
                    } else {
                        assert_eq!(append.op, Op::MixedArrayAppend, "{target}: boxed append uses final storage");
                        assert_eq!(value.ownership, crate::ir::Ownership::Borrowed,
                            "{target}: direct boxed append must borrow its receiver");
                        boxed += 1;
                    }
                }
            }
        }
        assert!(boxed > 0 && native > 0 && boxed + native >= 6,
            "{target}: cover boxed and native views in methods, functions, closures and main: {boxed}/{native}");
        crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
    }
}

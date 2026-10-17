//! Purpose:
//! Checks enum JSON serialization across native objects and runtime-declared eval enums.
//!
//! Called from:
//! - The codegen eval integration test module.
//!
//! Key details:
//! - Runtime-selected source forces Magician instead of literal-eval AOT lowering.
//! - Native enum inputs exercise real property reads and method callback ownership.

use super::*;

/// Both native and eval-declared enum cases follow scalar and JsonSerializable rules in eval.
#[test]
fn test_dynamic_eval_json_enum_serialization() {
    let out = compile_and_run(r#"<?php
enum NativeJsonInt: int { case N = 7; }
enum NativeJsonString: string { case Hi = "hi"; }
enum NativeJsonCustom implements JsonSerializable {
    case Ready;
    public function jsonSerialize(): mixed { return "ready"; }
}
$items = [NativeJsonInt::N, NativeJsonString::Hi, NativeJsonCustom::Ready];
echo json_encode($items) . "\n";
$code = $argc > 1 ? '' : '
echo json_encode($items) . ":" . json_last_error() . "\n";
enum DynamicJsonInt: int { case N = 7; }
enum DynamicJsonString: string { case Hi = "hi"; }
enum DynamicJsonCustom implements JsonSerializable {
    case Ready;
    public function jsonSerialize(): mixed { echo "called:"; return "ready"; }
}
echo json_encode([DynamicJsonInt::N, DynamicJsonString::Hi, DynamicJsonCustom::Ready]) . ":" . json_last_error();
';
eval($code);
"#);
    assert_eq!(out, "[7,\"hi\",\"ready\"]\n[7,\"hi\",\"ready\"]:0\ncalled:[7,\"hi\",\"ready\"]:0");
}

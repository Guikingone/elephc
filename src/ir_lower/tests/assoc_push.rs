//! Purpose:
//! Checks target parity for associative array_push lowering and receiver publication.
//!
//! Called from:
//! - The ir_lower unit test harness.
//!
//! Key details:
//! - Every supported target must emit the shared hash append and count paths.

/// Typed hashes, aliases and Mixed entries reach target-aware assembly on every supported target.
#[test]
fn associative_array_push_emits_on_every_supported_target() {
    let source = r#"<?php
$values = ["a" => 1];
$copy = $values;
echo array_push($values, $argc, 2), array_push(array: $values);
$alias =& $values;
array_push($alias, ...[3, 4]);
$mixed = ["a" => 1, "b" => "s"];
array_push($mixed, false, null, [1]);
$tail = [$argc, 5];
array_push($mixed, ...$tail);
class TargetPushBox { public array $items = ["a" => 1]; }
$box = new TargetPushBox();
array_push($box->items, ...$tail);
$held = $box;
array_push($box->items, (function () use (&$box) { $box = new TargetPushBox(); return 2; })());
$box->items = ["a" => 1, PHP_INT_MAX - 1 => 2];
try { array_push($box->items, 3, 4); } catch (Error $e) { echo $e->getMessage(); }
class TargetReadonlyPushBox {
    public readonly array $items;
    public function __construct() { $this->items = ["a" => 1]; }
}
$readonly = new TargetReadonlyPushBox();
try { array_push($readonly->items, 2); } catch (Error $e) { echo $e->getMessage(); }
$rows = ["inner" => ["a" => 1]];
array_push($rows["inner"], 2);
echo json_encode($values), json_encode($copy), json_encode($mixed);
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source,
            std::path::Path::new("main.php"),
            std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        let asm = crate::codegen::generate_user_asm_from_ir(&module, false, false)
            .unwrap_or_else(|error| panic!("{target}: {error:?}"));
        assert!(asm.contains("__rt_hash_next_index"), "{target}: no associative append");
        assert!(asm.contains("__rt_hash_set"), "{target}: no hash insertion");
    }
}

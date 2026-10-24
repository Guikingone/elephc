//! Purpose:
//! Verifies receiver reads preserve property effects and method argument aliases.
//!
//! Called from:
//! - The optimizer codegen integration suite.
//!
//! Key details:
//! - Both optimizer modes execute ordinary, magic, nullable and boxed receivers.

use super::*;

/// Executes one fixture with an explicit optimizer mode and releases its isolated directory.
fn run_receiver_fixture(source: &str, optimized: bool) -> String {
    let dir = make_cli_test_dir("elephc_receiver_value_alias");
    let path = dir.join("main.php");
    fs::write(&path, source).unwrap();
    let mut command = elephc_cli_command(&dir);
    if !optimized { command.arg("--no-ir-opt"); }
    let compiled = command.arg(&path).output().unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    let output = run_binary(&path.with_extension(""), &dir);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    fs::remove_dir_all(dir).unwrap();
    String::from_utf8(output.stdout).unwrap()
}

/// By-reference arguments can update a scalar and replace the receiver's own local slot.
#[test]
fn test_receiver_value_alias_preserves_method_argument_updates() {
    let source = include_str!("../../../examples/receiver-value-aliasing/main.php");
    for optimized in [false, true] {
        assert_eq!(run_receiver_fixture(source, optimized), "24:3:9:3");
    }
}

/// Magic getters still run for every read, including through nullable and boxed receivers.
#[test]
fn test_receiver_value_alias_preserves_magic_and_nullable_reads() {
    let source = r#"<?php
class ReceiverMagic {
    public int $reads = 0;
    public function __get(string $name): int { $this->reads++; return $this->reads; }
    public function count(): int { return $this->reads; }
}
function visitMagic(ReceiverMagic $receiver, int $count): void {
    $sum = 0;
    for ($index = 0; $index < $count; $index++) { $sum += $receiver->missing; }
    echo $sum, ':', $receiver->count(), '|';
}
function readNullable(?ReceiverMagic $receiver): int {
    if ($receiver === null) { return 0; }
    return $receiver->missing + $receiver->count();
}
function readBoxed(mixed $receiver): int { return $receiver->missing; }
visitMagic(new ReceiverMagic(), $argc + 2);
echo readNullable(null), ':', readNullable(new ReceiverMagic()), ':';
echo readBoxed(new ReceiverMagic());
"#;
    for optimized in [false, true] {
        assert_eq!(run_receiver_fixture(source, optimized), "6:3|0:2:1");
    }
}

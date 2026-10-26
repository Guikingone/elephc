//! Purpose:
//! Checks object parameter receiver lifetimes with receiver-root preparation enabled and disabled.
//!
//! Called from:
//! - The codegen runtime ownership suite on every executable supported target.
//!
//! Key details:
//! - Immutable parameters remain alive across throwing argument evaluation.
//! - Reassigned, aliased and by-reference bindings preserve the original receiver until return.

use crate::support::compile_and_run_with_heap_debug;

/// Receiver borrows and required leases preserve destructor order and retire every owner.
#[test]
fn test_fileinfo_parameter_receiver_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ParameterRootGuard {
    public static int $argumentFailures = 0;
    public int $id;
    public function __construct(int $id) { $this->id = $id; }
    public function read(int $value): int { return $this->id + $value; }
    public function __destruct() {
        if ($this->id === 2) { self::$argumentFailures++; return; }
        echo "D", $this->id, "|";
    }
}
function parameterRootArgument(bool $throw): int {
    if ($throw) { echo "A|"; throw new Error("argument"); }
    return 2;
}
function immutableParameterRoot(ParameterRootGuard $receiver, bool $throw): int {
    return $receiver->read(parameterRootArgument($throw));
}
function reassignedParameterRoot(ParameterRootGuard $receiver): int {
    return $receiver->read(($receiver = new ParameterRootGuard(4))->read(1));
}
function aliasedParameterRoot(ParameterRootGuard $receiver): int {
    $alias = &$receiver;
    return $receiver->read(($alias = new ParameterRootGuard(6))->read(1));
}
function referencedParameterRoot(ParameterRootGuard &$receiver): int {
    return $receiver->read(($receiver = new ParameterRootGuard(8))->read(1));
}
for ($i = 0; $i < 3; $i++) {
    echo immutableParameterRoot(new ParameterRootGuard(1), false), "|";
    try { immutableParameterRoot(new ParameterRootGuard(2), true); }
    catch (Error $error) { echo $error->getMessage(), "|"; }
    echo reassignedParameterRoot(new ParameterRootGuard(3)), "|";
    echo aliasedParameterRoot(new ParameterRootGuard(5)), "|";
    $reference = new ParameterRootGuard(7);
    echo referencedParameterRoot($reference), "|";
    unset($reference, $error);
    echo "E", ParameterRootGuard::$argumentFailures, "|\n";
}
"#);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    let expected = (1..=3).map(|count| format!("D1|3|A|argument|D3|D4|8|D5|D6|12|D7|16|D8|E{count}|\n")).collect::<String>();
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Disabling EIR optimization preserves the same lifetime and exception behavior.
#[test]
fn test_fileinfo_parameter_receiver_owners_without_ir_opt() {
    crate::support::without_ir_opt(test_fileinfo_parameter_receiver_owners);
}

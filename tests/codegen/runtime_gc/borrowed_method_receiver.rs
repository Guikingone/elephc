//! Purpose:
//! Checks borrowed method receiver lifetime while callbacks replace its source binding.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Destructor ordering and heap poisoning expose premature receiver release.

use crate::support::*;

/// A method keeps its receiver alive after a callback clears the last global binding.
#[test]
fn test_borrowed_method_receiver_survives_global_clear() {
    let source = r#"<?php
function clearRoot(): void { global $h; $h = null; echo 'clear|'; }
class H {
    public int $value = 7;
    function run(): void { clearRoot(); echo 'run:', $this->value, '|'; }
    function __destruct() { echo 'drop:', $this->value, '|'; }
}
$h = new H();
$h->run();
echo 'after';
"#;
    assert_receiver(source, "clear|run:7|drop:7|after");
}

/// Checks destructor timing and receiver ownership with optimization enabled and disabled.
fn assert_receiver(source: &str, expected: &str) {
    let run = || {
        let out = compile_and_run_with_heap_debug(source);
        assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
        assert_eq!(out.stdout, expected, "{}", out.stderr);
        assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
    };
    run();
    without_ir_opt(run);
}

/// Preserves the receiver across the argument-clear callback boundary.
#[test]
fn test_borrowed_method_receiver_argument_clear() {
    assert_receiver(r#"<?php
function clearRoot(): int { global $h; $h = null; echo 'clear|'; return 1; }
class H {
 public int $value=7;
 function run(int $arg=0): void { clearRoot(); echo 'run:', $this->value, '|'; }
 function fail(): void { clearRoot(); echo 'fail:', $this->value, '|'; throw new RuntimeException('stop'); }
 function __destruct(){ echo 'drop:', $this->value, '|'; }
}
$h=new H();$h->run(clearRoot());echo 'after';"#, "clear|clear|run:7|drop:7|after");
}

/// Preserves the receiver across the callee-throw callback boundary.
#[test]
fn test_borrowed_method_receiver_callee_throw() {
    assert_receiver(r#"<?php
function clearRoot(): int { global $h; $h = null; echo 'clear|'; return 1; }
class H {
 public int $value=7;
 function run(int $arg=0): void { clearRoot(); echo 'run:', $this->value, '|'; }
 function fail(): void { clearRoot(); echo 'fail:', $this->value, '|'; throw new RuntimeException('stop'); }
 function __destruct(){ echo 'drop:', $this->value, '|'; }
}
$h=new H();try{$h->fail();}catch(RuntimeException $e){echo 'catch|';unset($e);}echo 'after';"#, "clear|fail:7|drop:7|catch|after");
}

/// Preserves the receiver across the nullable callback boundary.
#[test]
fn test_borrowed_method_receiver_nullable() {
    assert_receiver(r#"<?php
function clearRoot(): int { global $h; $h = null; echo 'clear|'; return 1; }
class H {
 public int $value=7;
 function run(int $arg=0): void { clearRoot(); echo 'run:', $this->value, '|'; }
 function fail(): void { clearRoot(); echo 'fail:', $this->value, '|'; throw new RuntimeException('stop'); }
 function __destruct(){ echo 'drop:', $this->value, '|'; }
}
$h=new H();$h?->run();echo 'after';"#, "clear|run:7|drop:7|after");
}

/// Preserves the receiver across the local-reference callback boundary.
#[test]
fn test_borrowed_method_receiver_local_reference() {
    assert_receiver(r#"<?php
class H {public int $value=7;function run(&$root): void {$root=null;echo 'run:',$this->value,'|';}function __destruct(){echo 'drop:',$this->value,'|';}}
function test():void{$h=new H();$h->run($h);echo 'after';}test();"#, "run:7|drop:7|after");
}

/// Preserves the receiver across the property callback boundary.
#[test]
fn test_borrowed_method_receiver_property() {
    assert_receiver(r#"<?php
class H {public int $value=7;function run($owner): void {$owner->item=null;echo 'run:',$this->value,'|';}function __destruct(){echo 'drop:',$this->value,'|';}}
class Owner {public ?H $item=null;}$owner=new Owner();$owner->item=new H();$owner->item->run($owner);echo 'after';"#, "run:7|drop:7|after");
}

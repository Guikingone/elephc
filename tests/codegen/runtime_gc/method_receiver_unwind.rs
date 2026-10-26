//! Purpose:
//! Verifies temporary method receivers survive argument evaluation and unwind exactly once.
//!
//! Called from:
//! - The codegen runtime_gc integration test module.
//!
//! Key details:
//! - Covers direct, indirect, nullable and normal-return paths with native heap checks.

/// Checks exact PHP output and ownership for one receiver lifetime case.
fn assert_receiver(source: &str, expected: &str) {
    let out = crate::support::compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves PHP receiver lifetime for property direct.
#[test]
fn test_method_receiver_unwind_property_direct() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}$h=new H();try{$h->c->fail(9);}catch(Throwable $e){echo $e->getMessage();}unset($h);"##, "method");
}

/// Preserves PHP receiver lifetime for property indirect.
#[test]
fn test_method_receiver_unwind_property_indirect() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}$h=new H();try{$h->c->indirect(9);}catch(Throwable $e){echo $e->getMessage();}unset($h);"##, "indirect");
}

/// Preserves PHP receiver lifetime for new receiver.
#[test]
fn test_method_receiver_unwind_new_receiver() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}try{(new C())->fail(9);}catch(Throwable $e){echo $e->getMessage();}"##, "method");
}

/// Preserves PHP receiver lifetime for returned receiver.
#[test]
fn test_method_receiver_unwind_returned_receiver() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}function make():C{return new C();}try{make()->fail(9);}catch(Throwable $e){echo $e->getMessage();}"##, "method");
}

/// Preserves PHP receiver lifetime for argument throw.
#[test]
fn test_method_receiver_unwind_argument_throw() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}function arg(){throw new Exception("argument");}$h=new H();try{$h->c->ok(arg());}catch(Throwable $e){echo $e->getMessage();}unset($h);"##, "argument");
}

/// Preserves PHP receiver lifetime for nullable property.
#[test]
fn test_method_receiver_unwind_nullable_property() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}class N{public ?C $c;public function __construct(bool $set){$this->c=$set?new C():null;}}foreach([true,false] as $set){$h=new N($set);try{var_dump($h->c?->fail(9));}catch(Throwable $e){echo $e->getMessage(),"|";}unset($h);}"##, "method|NULL\n");
}

/// Preserves PHP receiver lifetime for normal result.
#[test]
fn test_method_receiver_unwind_normal_result() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}$h=new H();echo $h->c->ok(9);unset($h);"##, "7");
}

/// Preserves PHP receiver lifetime for fluent result.
#[test]
fn test_method_receiver_unwind_fluent_result() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}class F{public function self():F{return $this;}public function ok():int{return 5;}}echo (new F())->self()->ok();"##, "5");
}

/// Preserves PHP receiver lifetime for receiver dtor throw.
#[test]
fn test_method_receiver_unwind_receiver_dtor_throw() {
    assert_receiver(r##"<?php class C{public function ok(mixed $x):int{return 7;}public function fail(mixed $value):bool{throw new Exception("method");}private function check():void{throw new Exception("indirect");}public function indirect(mixed $value):bool{$this->check();echo $value;return true;}}class H{public C $c;public function __construct(){$this->c=new C();}}class D{public function value():string{return str_repeat("x",3);}public function __destruct(){throw new Exception("drop");}}try{$value=(new D())->value();}catch(Throwable $e){echo $e->getMessage();}"##, "drop");
}

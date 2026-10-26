//! Purpose:
//! Differential SPL heap root-retirement regressions frozen against PHP 8.5.10.
//!
//! Called from:
//! - Runtime GC codegen tests on executable supported targets.
//!
//! Key details:
//! - Verifies callback order, logical views, corruption, clone state and previous exceptions.
//! - Both optimizer modes must match complete output and finish with a clean native heap.

/// Checks one complete PHP-observable sequence with both EIR optimization settings.
fn assert_retirement(source: &str, expected: &str) {
    use crate::support::*;
    let run = || {
        let dir = make_cli_test_dir("spl_heap_retirement");
        let (user, runtime, libraries) = compile_source_to_asm_with_options(source, &dir, 8_388_608, false, true);
        std::fs::write(dir.join("user.s"), &user).unwrap();
        std::fs::write(dir.join("runtime.s"), &runtime).unwrap();
        std::fs::write(dir.join("source.php"), source).unwrap();
        let output = assemble_and_run_capture(&user, &runtime_obj_for_asm(&runtime), &dir,
            &libraries, &default_link_paths(), &[]);
        assert!(output.success, "{}\n{}\nretained: {}", output.stdout, output.stderr, dir.display());
        assert_eq!(output.stdout, expected, "{} retained: {}", output.stderr, dir.display());
        assert!(output.stderr.contains("leak summary: clean"), "{} retained: {}", output.stderr, dir.display());
        let _ = std::fs::remove_dir_all(dir);
    };
    run();
    without_ir_opt(run);
}

/// Replays the pinned PHP root-retirement behavior for callback order.
#[test]
fn test_spl_heap_retirement_callback_order() {
    assert_retirement(r##"<?php class H extends SplHeap{public bool $trace=false;protected function compare(mixed $a,mixed $b):int{if($this->trace){echo 'compare:',count($this),'|';}return 0;}}class D{public H $h;function __construct(H $h){$this->h=$h;}function __destruct(){echo 'drop:',count($this->h),':',$this->h->current()===$this?'same':'changed','|';}}$h=new H();$h->insert(new D($h));foreach([1,2,3] as $v){$h->insert($v);}$h->trace=true;echo 'remove|';$h->next();echo 'left:',count($h);"##, "remove|drop:4:same|compare:3|compare:3|left:3");
}

/// Replays the pinned PHP root-retirement behavior for queue drop order.
#[test]
fn test_spl_heap_retirement_queue_drop_order() {
    assert_retirement(r##"<?php class Q extends SplPriorityQueue{public function compare(mixed $a,mixed $b):int{return 0;}}class D{public Q $q;public string $name;function __construct(Q $q,string $name){$this->q=$q;$this->name=$name;}function __destruct(){echo $this->name,':',count($this->q),'|';}}$q=new Q();$q->insert(new D($q,'data'),new D($q,'priority'));$q->next();echo count($q);"##, "data:1|priority:1|0");
}

/// Replays the pinned PHP root-retirement behavior for drop throw.
#[test]
fn test_spl_heap_retirement_drop_throw() {
    assert_retirement(r##"<?php class H extends SplHeap{public bool $trace=false;protected function compare(mixed $a,mixed $b):int{if($this->trace){echo 'compare|';}return 0;}}class D{public H $h;function __construct(H $h){$this->h=$h;}function __destruct(){echo 'drop:',count($this->h),'|';throw new Exception('drop-error');}}$h=new H();$h->insert(new D($h));foreach([1,2,3] as $v){$h->insert($v);}$h->trace=true;try{$h->next();}catch(Throwable $e){echo $e->getMessage(),'|';}echo count($h),':',$h->isCorrupted()?'corrupted':'clean';"##, "drop:4|drop-error|3:corrupted");
}

/// Replays the pinned PHP root-retirement behavior for clone named global.
#[test]
fn test_spl_heap_retirement_clone_named_global() {
    assert_retirement(r##"<?php class H extends SplHeap{protected function compare(mixed $a,mixed $b):int{return 0;}}class D{public H $h;function __construct(H $h){$this->h=$h;}function __destruct(){global $copy;echo 'drop:',count($this->h),'|';$copy=clone $this->h;}}$copy=null;$h=new H();$h->insert(new D($h));$h->insert(1);$h->next();$copy=$copy;if($copy instanceof H){echo count($h),':',count($copy),':',is_object($copy->current())?'object':'scalar','|';try{$copy->insert(9);}catch(Throwable $e){echo $e->getMessage();}}else{echo 'bad-copy';}unset($copy);unset($copy);"##, "drop:2|1:2:object|Heap cannot be changed when it is already being modified.");
}

/// Replays the pinned PHP root-retirement behavior for queue two errors.
#[test]
fn test_spl_heap_retirement_queue_two_errors() {
    assert_retirement(r##"<?php class Q extends SplPriorityQueue{public function compare(mixed $a,mixed $b):int{return 0;}} class D{public Q $q;public string $name;function __construct(Q $q,string $name){$this->q=$q;$this->name=$name;}function __destruct(){echo $this->name,':',count($this->q),'|';throw new Exception($this->name);}}$q=new Q();$q->insert(new D($q,'data'),new D($q,'priority'));try{$q->next();}catch(Throwable $e){echo $e->getMessage(),':',$e->getPrevious()?->getMessage()??'none','|';}echo count($q),':',$q->isCorrupted()?'corrupted':'clean';"##, "data:1|priority:1|priority:data|0:corrupted");
}

/// Replays the pinned PHP root-retirement behavior for string view.
#[test]
fn test_spl_heap_retirement_string_view() {
    assert_retirement(r##"<?php class Q extends SplPriorityQueue{public bool $trace=false;public function compare(mixed $a,mixed $b):int{if($this->trace){echo bin2hex($this->current()),'|';}return 0;}}$q=new Q();$data=str_repeat("x\0y",2);$q->insert($data,4);foreach([1,2,3] as $i){$q->insert($i,$i);}$q->trace=true;$q->next();echo count($q);"##, "780079780079|780079780079|3");
}

/// Replays the pinned PHP root-retirement behavior for extract priority drop.
#[test]
fn test_spl_heap_retirement_extract_priority_drop() {
    assert_retirement(r##"<?php class Q extends SplPriorityQueue{public function compare(mixed $a,mixed $b):int{return 0;}}class D{public Q $q;function __construct(Q $q){$this->q=$q;}function __destruct(){echo 'drop:',count($this->q),'|';}}$q=new Q();$q->insert('data',new D($q));$q->setExtractFlags(SplPriorityQueue::EXTR_DATA);echo $q->extract(),'|',count($q);"##, "drop:0|data|0");
}

/// Replays the pinned PHP lifetime behavior for receiver unset.
#[test]
fn test_spl_heap_retirement_receiver_unset() {
    assert_retirement(r##"<?php class H extends SplMaxHeap{function __destruct(){echo 'heap:',count($this),'|';}}class D{function __destruct(){global $h;echo 'data|';$h=null;}}$h=new H();$h->insert(new D());$h->next();echo 'after';"##, "data|heap:0|after");
}

/// Replays the pinned PHP lifetime behavior for resource live view.
#[test]
fn test_spl_heap_retirement_resource_live_view() {
    assert_retirement(r##"<?php class Q extends SplPriorityQueue{public function compare(mixed $a,mixed $b):int{return 0;}}class D{public Q $q;public mixed $r;function __construct(Q $q,mixed $r){$this->q=$q;$this->r=$r;}function __destruct(){echo $this->q->current()===$this->r?'same:':'changed:',count($this->q),'|';}}$r=fopen('php://temp','w+');$q=new Q();$q->insert($r,new D($q,$r));$q->next();echo is_resource($r)?'open':'closed';fclose($r);"##, "same:1|open");
}

/// Replays the pinned PHP lifetime behavior for array live view.
#[test]
fn test_spl_heap_retirement_array_live_view() {
    assert_retirement(r##"<?php class H extends SplHeap{public bool $trace=false;protected function compare(mixed $a,mixed $b):int{if($this->trace){echo is_array($this->current())?'array|':'bad|';}return 0;}}$a=[1,2];$h=new H();$h->insert($a);foreach([1,2,3] as $v){$h->insert($v);}$h->trace=true;$h->next();echo count($h);"##, "array|array|3");
}

/// Replays the pinned PHP lifetime behavior for debug resurrection.
#[test]
fn test_spl_heap_retirement_debug_resurrection() {
    assert_retirement(r##"<?php class H extends SplHeap{public mixed $saved=null;protected function compare(mixed $a,mixed $b):int{return 0;}}class D{public H $h;function __construct(H $h){$this->h=$h;}function __destruct(){echo 'drop:',count($this->h),'|';$view=$this->h->__debugInfo();$this->h->saved=$view["\0SplHeap\0heap"][0];}}$h=new H();$d=new D($h);$w=WeakReference::create($d);$h->insert($d);unset($d);$h->next();echo count($h),':',$w->get()===$h->saved?'alive':'bad','|';$h->saved=null;echo $w->get()===null?'gone':'bad';"##, "drop:1|0:alive|gone");
}

/// Unlinks the older receiver scope while another Fiber keeps its own mutation suspended.
#[test]
fn test_spl_heap_retirement_fiber_scopes() {
    assert_retirement(r##"<?php
class H extends SplMaxHeap {
 public bool $pause = false;
 public string $label;
 function __construct(string $label) { $this->label = $label; }
 function compare($a, $b): int {
  if ($this->pause) { $this->pause = false; echo $this->label, ':', count($this), '|'; Fiber::suspend(); }
  return $a <=> $b;
 }
}
$a = new H('a'); $b = new H('b');
foreach ([4, 3, 2, 1] as $value) { $a->insert($value); $b->insert($value); }
$a->pause = true; $b->pause = true;
$fa = new Fiber(function() use ($a) { $a->next(); echo 'a-done:', $a->top(), '|'; });
$fb = new Fiber(function() use ($b) { $b->next(); echo 'b-done:', $b->top(), '|'; });
$fa->start(); $fb->start(); $fa->resume(); $fb->resume();
echo count($a), ':', count($b);
"##, "a:3|b:3|a-done:3|b-done:3|3:3");
}

/// Materializes the same logical clone through a Mixed property receiver and boxed runtime path.
#[test]
fn test_spl_heap_retirement_clone_boxed_receiver() {
    assert_retirement(r##"<?php class H extends SplHeap{protected function compare(mixed $a,mixed $b):int{return 0;}}class D{public mixed $h;function __construct(H $h){$this->h=$h;}function __destruct(){global $copy;echo 'drop:',count($this->h),'|';$copy=clone $this->h;}}$copy=new H();$h=new H();$h->insert(new D($h));$h->insert(1);$h->next();$copy=$copy;echo count($h),':',count($copy),':',is_object($copy->current())?'object':'scalar','|';try{$copy->insert(9);}catch(Throwable $e){echo $e->getMessage();}unset($copy);unset($copy);"##, "drop:2|1:2:object|Heap cannot be changed when it is already being modified.");
}

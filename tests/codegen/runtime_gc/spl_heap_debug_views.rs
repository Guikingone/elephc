//! Purpose:
//! Differential regressions for SPL physical heap layout, debug views, and corruption.
//!
//! Called from:
//! - The runtime_gc codegen integration test module.
//!
//! Key details:
//! - Expected bytes were produced by pinned PHP 8.5.10; ownership is checked independently.

/// Checks observable output and the native runtime heap for one Zend heap fixture.
fn assert_heap_view(source: &str, expected: &str) {
    let out = crate::support::compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Matches pinned Zend behavior for empty views.
#[test]
fn test_spl_heap_debug_empty_views() {
    assert_heap_view(r##"<?php foreach([new SplMaxHeap(),new SplMinHeap(),new SplPriorityQueue()] as $h){echo json_encode($h->__debugInfo()),"\n";var_dump($h->key(),$h->current(),$h->isCorrupted());}"##, "{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[]}\nint(-1)\nNULL\nbool(false)\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[]}\nint(-1)\nNULL\nbool(false)\n{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[]}\nint(-1)\nNULL\nbool(false)\n");
}

/// Matches pinned Zend behavior for max order.
#[test]
fn test_spl_heap_debug_max_order() {
    assert_heap_view(r##"<?php $h=new SplMaxHeap();foreach([2,5,1,3,5,0] as $v){$h->insert($v);echo json_encode($h->__debugInfo()),"\n";}while(!$h->isEmpty()){echo $h->extract(),':',json_encode($h->__debugInfo()),"\n";}"##, "{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[2]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,2]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,2,1]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,3,1,2]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,5,1,2,3]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,5,1,2,3,0]}\n5:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,3,1,2,0]}\n5:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[3,2,1,0]}\n3:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[2,0,1]}\n2:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[1,0]}\n1:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[0]}\n0:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[]}\n");
}

/// Matches pinned Zend behavior for min order.
#[test]
fn test_spl_heap_debug_min_order() {
    assert_heap_view(r##"<?php $h=new SplMinHeap();foreach([2,5,1,3,5,0] as $v){$h->insert($v);echo json_encode($h->__debugInfo()),"\n";}while($h->valid()){echo $h->current(),':',$h->key(),"|";$h->next();}echo "\n";"##, "{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[2]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[2,5]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[1,5,2]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[1,3,2,5]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[1,3,2,5,5]}\n{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[0,3,1,5,5,2]}\n0:5|1:4|2:3|3:2|5:1|5:0|\n");
}

/// Matches pinned Zend behavior for priority order.
#[test]
fn test_spl_heap_debug_priority_order() {
    assert_heap_view(r##"<?php $h=new SplPriorityQueue();foreach([2,5,1,3] as $p){$h->insert('v'.$p,$p);echo json_encode($h->__debugInfo()),"\n";}echo $h->setExtractFlags(7),':',$h->getExtractFlags(),"\n";while(!$h->isEmpty()){echo json_encode($h->extract()),':',json_encode($h->__debugInfo()),"\n";}"##, "{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v2\",\"priority\":2}]}\n{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v5\",\"priority\":5},{\"data\":\"v2\",\"priority\":2}]}\n{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v5\",\"priority\":5},{\"data\":\"v2\",\"priority\":2},{\"data\":\"v1\",\"priority\":1}]}\n{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v5\",\"priority\":5},{\"data\":\"v3\",\"priority\":3},{\"data\":\"v1\",\"priority\":1},{\"data\":\"v2\",\"priority\":2}]}\n3:3\n{\"data\":\"v5\",\"priority\":5}:{\"\\u0000SplPriorityQueue\\u0000flags\":3,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v3\",\"priority\":3},{\"data\":\"v2\",\"priority\":2},{\"data\":\"v1\",\"priority\":1}]}\n{\"data\":\"v3\",\"priority\":3}:{\"\\u0000SplPriorityQueue\\u0000flags\":3,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v2\",\"priority\":2},{\"data\":\"v1\",\"priority\":1}]}\n{\"data\":\"v2\",\"priority\":2}:{\"\\u0000SplPriorityQueue\\u0000flags\":3,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"v1\",\"priority\":1}]}\n{\"data\":\"v1\",\"priority\":1}:{\"\\u0000SplPriorityQueue\\u0000flags\":3,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[]}\n");
}

/// Matches pinned Zend behavior for user members.
#[test]
fn test_spl_heap_debug_user_members() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{public string $values='user';protected int $flags=7;private string $own='child';}class Q extends SplPriorityQueue{public string $priorities='user';private int $extractFlags=9;}$h=new H();$h->insert(3);$h->insert(7);echo json_encode($h->__debugInfo()),"\n";$q=new Q();$q->insert('a',4);echo json_encode($q->__debugInfo()),"\n";"##, "{\"values\":\"user\",\"\\u0000*\\u0000flags\":7,\"\\u0000H\\u0000own\":\"child\",\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[7,3]}\n{\"priorities\":\"user\",\"\\u0000Q\\u0000extractFlags\":9,\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"a\",\"priority\":4}]}\n");
}

/// Matches pinned Zend behavior for snapshot cow.
#[test]
fn test_spl_heap_debug_snapshot_cow() {
    assert_heap_view(r##"<?php $h=new SplMaxHeap();$h->insert('a');$h->insert('c');$v=$h->__debugInfo();$h->insert('b');echo json_encode($v),"\n";$v["\0SplHeap\0heap"][0]='changed';echo $h->top(),':',json_encode($h->__debugInfo()),"\n";$q=new SplPriorityQueue();$q->insert('a',2);$v=$q->__debugInfo();$q->insert('b',4);echo json_encode($v),"\n";$v["\0SplPriorityQueue\0heap"][0]['data']='changed';echo $q->top(),"\n";"##, "{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[\"c\",\"a\"]}\nc:{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[\"c\",\"a\",\"b\"]}\n{\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":false,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"a\",\"priority\":2}]}\nb\n");
}

/// Matches pinned Zend behavior for compare trace.
#[test]
fn test_spl_heap_debug_compare_trace() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{protected function compare(mixed $a,mixed $b):int{echo $a,':',$b,':',count($this),'|';return $a<=>$b;}}$h=new H();foreach([2,5,1,3] as $v){$h->insert($v);echo 'insert;';}echo 'top:',$h->top(),';';while(!$h->isEmpty()){echo 'out:',$h->extract(),';';}"##, "insert;2:5:1|insert;5:1:2|insert;2:3:3|5:3:3|insert;top:5;out:1:3:3|2:3:3|5;out:1:2:2|1:2:2|3;out:2;out:1;");
}

/// Matches pinned Zend behavior for insert throw.
#[test]
fn test_spl_heap_debug_insert_throw() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{public int $calls=0;protected function compare(mixed $a,mixed $b):int{$this->calls++;if($this->calls===2){throw new Exception('compare');}return $a<=>$b;}}$h=new H();$h->insert(2);$h->insert(5);try{$h->insert(3);}catch(Throwable $e){echo $e->getMessage(),'|';}echo json_encode($h->__debugInfo()),"\n";try{$h->top();}catch(Throwable $e){echo $e->getMessage(),"\n";}var_dump($h->isCorrupted(),$h->current(),$h->key(),$h->recoverFromCorruption(),$h->isCorrupted());while($h->valid()){echo $h->extract(),'|';}"##, "compare|{\"calls\":2,\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":true,\"\\u0000SplHeap\\u0000heap\":[5,2,3]}\nHeap is corrupted, heap properties are no longer ensured.\nbool(true)\nint(5)\nint(2)\nbool(true)\nbool(false)\n5|3|2|");
}

/// Matches pinned Zend behavior for extract throw.
#[test]
fn test_spl_heap_debug_extract_throw() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{public bool $fail=false;protected function compare(mixed $a,mixed $b):int{if($this->fail){throw new Exception('compare');}return $a<=>$b;}}$h=new H();foreach([2,5,1,3] as $v){$h->insert($v);}$h->fail=true;try{$h->extract();}catch(Throwable $e){echo $e->getMessage(),'|';}echo json_encode($h->__debugInfo()),"\n";var_dump(count($h),$h->isCorrupted());$h->fail=false;$h->recoverFromCorruption();while($h->valid()){echo $h->extract(),'|';}"##, "compare|{\"fail\":true,\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":true,\"\\u0000SplHeap\\u0000heap\":[2,3,1]}\nint(3)\nbool(true)\n2|3|1|");
}

/// Matches pinned Zend behavior for reentrant write.
#[test]
fn test_spl_heap_debug_reentrant_write() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{protected function compare(mixed $a,mixed $b):int{echo count($this),':',$this->top(),'|';try{$this->insert(99);}catch(Throwable $e){echo $e->getMessage(),'|';}return $a<=>$b;}}$h=new H();foreach([2,5,1,3] as $v){$h->insert($v);}echo json_encode($h->__debugInfo());"##, "1:2|Heap cannot be changed when it is already being modified.|2:5|Heap cannot be changed when it is already being modified.|3:5|Heap cannot be changed when it is already being modified.|3:5|Heap cannot be changed when it is already being modified.|{\"\\u0000SplHeap\\u0000flags\":0,\"\\u0000SplHeap\\u0000isCorrupted\":false,\"\\u0000SplHeap\\u0000heap\":[5,3,1,2]}");
}

/// Matches pinned Zend behavior for priority throw.
#[test]
fn test_spl_heap_debug_priority_throw() {
    assert_heap_view(r##"<?php class Q extends SplPriorityQueue{public bool $fail=false;public function compare(mixed $a,mixed $b):int{if($this->fail){throw new Exception('priority');}return $a<=>$b;}}$q=new Q();$q->insert('a',2);$q->insert('b',5);$q->fail=true;try{$q->insert('c',3);}catch(Throwable $e){echo $e->getMessage(),'|';}echo json_encode($q->__debugInfo()),"\n";$q->fail=false;$q->recoverFromCorruption();$q->setExtractFlags(3);while($q->valid()){echo json_encode($q->extract()),'|';}"##, "priority|{\"fail\":true,\"\\u0000SplPriorityQueue\\u0000flags\":1,\"\\u0000SplPriorityQueue\\u0000isCorrupted\":true,\"\\u0000SplPriorityQueue\\u0000heap\":[{\"data\":\"b\",\"priority\":5},{\"data\":\"a\",\"priority\":2},{\"data\":\"c\",\"priority\":3}]}\n{\"data\":\"b\",\"priority\":5}|{\"data\":\"c\",\"priority\":3}|{\"data\":\"a\",\"priority\":2}|");
}

/// Matches pinned Zend behavior for flags during compare.
#[test]
fn test_spl_heap_debug_flags_during_compare() {
    assert_heap_view(r##"<?php class Q extends SplPriorityQueue{public bool $flip=false;public function compare(mixed $a,mixed $b):int{if($this->flip){$this->setExtractFlags(3);}return $a<=>$b;}}$q=new Q();foreach([2,5,1,3] as $p){$q->insert('v'.$p,$p);}$q->flip=true;echo json_encode($q->extract()),'|',$q->getExtractFlags();"##, "{\"data\":\"v5\",\"priority\":5}|3");
}

/// Matches pinned Zend behavior for override readers.
#[test]
fn test_spl_heap_debug_override_readers() {
    assert_heap_view(r##"<?php class H extends SplMaxHeap{public function count():int{return 99;}public function isEmpty():bool{return true;}}$h=new H();$h->insert(3);$h->insert(7);echo count($h),':',$h->top(),':',$h->current(),':',$h->key(),'|',$h->extract(),'|',$h->top();"##, "99:7:7:1|7|3");
}

/// Matches pinned Zend behavior for next destructor.
#[test]
fn test_spl_heap_debug_next_destructor() {
    assert_heap_view(r##"<?php class H extends SplHeap{protected function compare(mixed $a,mixed $b):int{return 0;}}class D{public H $h;public function __construct(H $h){$this->h=$h;}public function __destruct(){echo 'drop:',count($this->h),'|';try{$this->h->insert(9);}catch(Throwable $e){echo $e->getMessage(),'|';}}}$h=new H();$h->insert(new D($h));$h->insert(1);$h->next();echo 'left:',count($h);"##, "drop:2|Heap cannot be changed when it is already being modified.|left:1");
}

/// Keeps MultipleIterator's inherited storage schema and snapshot owners native-compatible.
#[test]
fn test_spl_heap_debug_multiple_storage() {
    assert_heap_view(r##"<?php class M extends MultipleIterator{public string $label='child';}$m=new M();$i=new ArrayIterator([1]);$m->attachIterator($i,'x');$v=$m->__debugInfo();echo json_encode(array_keys($v)),'|';$m->detachIterator($i);echo count($v["\0SplObjectStorage\0storage"]),':',get_class($v["\0SplObjectStorage\0storage"][0]['obj']),':',$v["\0SplObjectStorage\0storage"][0]['inf'],'|',count($m->__debugInfo()["\0SplObjectStorage\0storage"]);"##, "[\"label\",\"\\u0000SplObjectStorage\\u0000storage\"]|1:ArrayIterator:x|0");
}

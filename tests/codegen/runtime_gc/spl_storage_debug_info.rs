//! Purpose:
//! Verifies native SPL debug snapshots, member visibility, and returned value ownership.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - Assertions compare PHP 8.5.10 values and complete mangled key order.
//! - The storage method returns independent entry arrays without invoking serialization hooks.

use crate::support::compile_and_run_with_heap_debug;

/// Checks the full PHP snapshot result and balanced owned storage.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Returns exactly the private storage row for an empty native storage.
#[test]
fn test_spl_storage_debug_info_empty() {
    assert_clean(r#"<?php
$s=new SplObjectStorage();$d=$s->__debugInfo();echo json_encode(array_keys($d)),':',count($d["\0SplObjectStorage\0storage"]);
"#, "[\"\\u0000SplObjectStorage\\u0000storage\"]:0");
}

/// Keeps object identities and complete attached info values in storage order.
#[test]
fn test_spl_storage_debug_info_entries() {
    assert_clean(r#"<?php
$s=new SplObjectStorage();$a=new stdClass();$b=new stdClass();$s->offsetSet($a,['name'=>'first']);$s->offsetSet($b,null);$d=$s->__debugInfo();$rows=$d["\0SplObjectStorage\0storage"];echo count($rows),':',$rows[0]['obj']===$a?'same':'bad',':',$rows[1]['obj']===$b?'same':'bad',':',$rows[0]['inf']['name'],':',is_null($rows[1]['inf'])?'null':'bad';
"#, "2:same:same:first:null");
}

/// Preserves initialized user members with their PHP visibility-mangled names.
#[test]
fn test_spl_storage_debug_info_members() {
    assert_clean(r#"<?php
class S extends SplObjectStorage {public string $name='ready';protected int $flag=7;private ?int $own=null;public int $uninitialized;}
$s=new S();$d=$s->__debugInfo();echo json_encode(array_keys($d)),':',$d['name'],':',$d["\0*\0flag"],':',is_null($d["\0S\0own"])?'null':'bad';
"#, "[\"name\",\"\\u0000*\\u0000flag\",\"\\u0000S\\u0000own\",\"\\u0000SplObjectStorage\\u0000storage\"]:ready:7:null");
}

/// Separates returned entry arrays from subsequent live-storage writes.
#[test]
fn test_spl_storage_debug_info_snapshot() {
    assert_clean(r#"<?php
$s=new SplObjectStorage();$o=new stdClass();$s->offsetSet($o,['label'=>'old']);$d=$s->__debugInfo();$s->offsetSet($o,['label'=>'new']);$key="\0SplObjectStorage\0storage";echo $d[$key][0]['inf']['label'],'|';$d[$key][0]['inf']['label']='snapshot';$current=$s->offsetGet($o);echo $current['label'],'|',$d[$key][0]['inf']['label'];
"#, "old|new|snapshot");
}

/// Keeps both object owners until the last snapshot row is released in PHP order.
#[test]
fn test_spl_storage_debug_info_owners() {
    assert_clean(r#"<?php
class Key {public string $name='key';function __destruct(){echo 'drop-key|';}}
class Info {public string $name='info';function __destruct(){echo 'drop-info|';}}
$s=new SplObjectStorage();$key=new Key();$info=new Info();$s->offsetSet($key,$info);$d=$s->__debugInfo();unset($s,$key,$info);$row=$d["\0SplObjectStorage\0storage"][0];echo $row['obj']->name,':',$row['inf']->name,'|';unset($row,$d);echo 'done';
"#, "key:info|drop-key|drop-info|done");
}

/// Builds the snapshot without calling a user serialization override.
#[test]
fn test_spl_storage_debug_info_no_serialization_hook() {
    assert_clean(r#"<?php
class S extends SplObjectStorage {public function __serialize():array {throw new Exception('serialization must not run');}}
$s=new S();$o=new stdClass();$s->offsetSet($o,7);$d=$s->__debugInfo();echo $d["\0SplObjectStorage\0storage"][0]['inf'];
"#, "7");
}

/// Allows a subclass to return its own packed PHP array.
#[test]
fn test_spl_storage_debug_info_override() {
    assert_clean(r#"<?php
class S extends SplObjectStorage {public function __debugInfo():array {return [7];}}
$s=new S();$d=$s->__debugInfo();echo json_encode($d);
"#, "[7]");
}

/// Reads storage without advancing or resetting its iterator cursor.
#[test]
fn test_spl_storage_debug_info_cursor() {
    assert_clean(r#"<?php
$s=new SplObjectStorage();$a=new stdClass();$b=new stdClass();$s->offsetSet($a,1);$s->offsetSet($b,2);$s->rewind();$s->next();$before=$s->key();$d=$s->__debugInfo();echo $before,':',$s->key(),':',count($d["\0SplObjectStorage\0storage"]);
"#, "1:1:2");
}

/// A dump-only use retains the native SPL hook and renders its recursive computed storage view.
#[test]
fn test_spl_storage_debug_info_dump_only_retains_native_hook() {
    let out = crate::support::compile_and_run_with_heap_debug(r#"<?php
$o = new SplObjectStorage();
$o[new stdClass()] = $o;
var_dump($o);
unset($o);
gc_collect_cycles();
"#);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "object(SplObjectStorage)#1 (1) {\n  [\"storage\":\"SplObjectStorage\":private]=>\n  array(1) {\n    [0]=>\n    array(2) {\n      [\"obj\"]=>\n      object(stdClass)#2 (0) {\n      }\n      [\"inf\"]=>\n      *RECURSION*\n    }\n  }\n}\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Matches PHP private storage keys, user members, and snapshot ownership.
#[test]
fn test_spl_array_debug_info_native() {
    assert_clean(r#"<?php
class A extends ArrayObject {public string $label='ready';protected int $flag=7;private int $own=9;public int $pending;public function getArrayCopy():array {throw new Exception('override');}}
class I extends ArrayIterator {public string $label='ready';protected int $flag=7;private int $own=9;public int $pending;public function getArrayCopy():array {throw new Exception('override');}}
$a=new A(['key'=>'old']);$i=new I([2=>3,'x'=>4]);$ad=$a->__debugInfo();$id=$i->__debugInfo();$a['key']='new';$i['x']=5;
echo json_encode($ad),'|',json_encode($id),'|',$a['key'],':',$i['x'];
"#, "{\"label\":\"ready\",\"\\u0000*\\u0000flag\":7,\"\\u0000A\\u0000own\":9,\"\\u0000ArrayObject\\u0000storage\":{\"key\":\"new\"}}|{\"label\":\"ready\",\"\\u0000*\\u0000flag\":7,\"\\u0000I\\u0000own\":9,\"\\u0000ArrayIterator\\u0000storage\":{\"2\":3,\"x\":5}}|new:5");
}

/// Matches PHP private storage keys, user members, and snapshot ownership.
#[test]
fn test_spl_array_debug_info_self() {
    assert_clean(r#"<?php
error_reporting(0);
class A extends ArrayObject {public int $own=7;}
class I extends ArrayIterator {public int $own=9;}
$a=new A();$a->__construct($a);$i=new I();$i->__construct($i);echo json_encode($a->__debugInfo()),'|',json_encode($i->__debugInfo());unset($a,$i);gc_collect_cycles();
"#, "{\"own\":7}|{\"own\":9}");
}

/// Matches PHP private storage keys, user members, and snapshot ownership.
#[test]
fn test_spl_file_info_debug_info_native() {
    assert_clean(r#"<?php
class F extends SplFileInfo {public string $label='ready';protected int $flag=7;private int $own=9;public int $pending;public function getPathname():string {throw new Exception('override');}public function getFilename():string {throw new Exception('override');}}
$f=new F('/tmp/report.txt');echo json_encode($f->__debugInfo());
"#, "{\"label\":\"ready\",\"\\u0000*\\u0000flag\":7,\"\\u0000F\\u0000own\":9,\"\\u0000SplFileInfo\\u0000pathName\":\"\\/tmp\\/report.txt\",\"\\u0000SplFileInfo\\u0000fileName\":\"report.txt\"}");
}

/// Matches PHP private storage keys, user members, and snapshot ownership.
#[test]
fn test_spl_array_debug_info_object_owner() {
    assert_clean(r#"<?php
error_reporting(0);class O {public int $x=7;function __destruct(){echo 'drop|';}}
$o=new O();$a=new ArrayObject($o);$d=$a->__debugInfo();$raw=$d["\0ArrayObject\0storage"];echo $raw===$o?'same|':'bad|';unset($raw,$a,$o);echo 'live|';unset($d);echo 'done';
"#, "same|live|drop|done");
}

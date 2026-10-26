//! Purpose:
//! Checks PHP insertion order during associative-array child destruction.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - Rehashing, tombstones, replacement, reference cells and shared rows retain PHP order.
//! - Throwing destructors must release every child before propagating the final exception.

use crate::support::compile_and_run_with_heap_debug;

/// Checks complete observable output together with final owned-storage cleanup.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves PHP child cleanup for the direct hash case.
#[test]
fn test_hash_destruction_order_direct() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$h=['obj'=>new Drop('key'),'inf'=>new Drop('info')];unset($h);echo 'done';"#, "key|info|done");
}

/// Preserves PHP child cleanup for the rehashed hash case.
#[test]
fn test_hash_destruction_order_rehashed() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$h=['obj'=>new Drop('key'),'old'=>new Drop('removed'),'inf'=>new Drop('info')];unset($h['old']);echo 'erased|';$h['old']=new Drop('reinserted');for($i=0;$i<40;$i++){$h['filler'.$i]=$i;}echo count($h),'|';unset($h);echo 'done';"#, "removed|erased|43|key|info|reinserted|done");
}

/// Preserves PHP child cleanup for the replaced hash case.
#[test]
fn test_hash_destruction_order_replaced() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$h=['obj'=>new Drop('old'),'inf'=>new Drop('info')];$h['obj']=new Drop('new');echo 'replaced|';unset($h);echo 'done';"#, "old|replaced|new|info|done");
}

/// Preserves PHP child cleanup for the reference hash case.
#[test]
fn test_hash_destruction_order_reference() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$h=['obj'=>new Drop('key'),'inf'=>new Drop('info')];$alias=&$h['obj'];unset($alias,$h);echo 'done';"#, "key|info|done");
}

/// Preserves PHP child cleanup for the snapshot hash case.
#[test]
fn test_hash_destruction_order_snapshot() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$key=new Drop('key');$info=new Drop('info');$d=['storage'=>[['obj'=>$key,'inf'=>$info]]];$copy=$d;unset($key,$info,$d);echo 'retained|';$row=$copy['storage'][0];echo $row['obj']->label,':',$row['inf']->label,'|';unset($row,$copy);echo 'done';"#, "retained|key:info|key|info|done");
}

/// Preserves PHP child cleanup for the empty hash case.
#[test]
fn test_hash_destruction_order_empty() {
    assert_clean(r#"<?php $h=['obj'=>1];unset($h['obj']);unset($h);echo 'done';"#, "done");
}

/// Preserves PHP child cleanup for the exception hash case.
#[test]
fn test_hash_destruction_order_exception() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';throw new Exception($this->label);}} $h=['obj'=>new Drop('key'),'inf'=>new Drop('info')];try{unset($h);}catch(Throwable $e){$p=$e->getPrevious();echo $e->getMessage(),':',is_null($p)?'none':$p->getMessage(),'|';}echo 'done';"#, "key|info|info:key|done");
}

/// Preserves PHP child cleanup for the escaped_reference hash case.
#[test]
fn test_hash_destruction_order_escaped_reference() {
    assert_clean(r#"<?php class Drop {public string $label;function __construct(string $label){$this->label=$label;}function __destruct(){echo $this->label,'|';}}
$h=['obj'=>new Drop('key'),'inf'=>new Drop('info')];$alias=&$h['obj'];unset($h);echo 'retained|',$alias->label,'|';unset($alias);echo 'done';"#, "info|retained|key|key|done");
}

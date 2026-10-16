//! Purpose:
//! Structural coverage of invariant closure calls and conservative loop fallbacks.
//!
//! Called from:
//! - `cargo test --lib ir_lower::tests::loop_closures`.
//!
//! Key details:
//! - Tests inspect lowering before EIR optimization can inline or fold calls.
//! - Every supported target receives the same scalar call and identity gates.

use crate::ir::{Module, Op};

/// Counts closure calls in main separately from ordinary named helper calls.
fn calls(module: &Module) -> (usize, usize) {
    let main = module.functions.iter().find(|function| function.flags.is_main).unwrap();
    let direct = main.instructions.iter().filter(|inst| inst.op == Op::Call).count();
    let descriptor = main.instructions.iter()
        .filter(|inst| inst.op == Op::CallableDescriptorInvoke).count();
    (direct, descriptor)
}

/// Builds a dynamic-count loop around one caller-provided body and update.
fn fixture(closure: &str, body: &str, update: &str) -> String {
    format!("<?php $n=$argc; $f={closure}; $sum=0; for($i=0;$i<$n;{update}){{{body}}} echo $sum;")
}

/// Scalar closure identities survive for/while lowering on every target.
#[test]
fn known_loop_closure_scalar_calls_on_all_targets() {
    for source in [
        fixture("function($x){return $x+1;}", "$sum+=$f($i);", "$i++"),
        "<?php $n=$argc; $f=function($x){return $x+1;}; $sum=0;$i=0; while($i<$n){$sum+=$f($i);$i++;} echo $sum;".to_string(),
    ] {
        for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let module = super::lower_source_at_for_target(&source, std::path::Path::new("main.php"),
                std::path::Path::new("."), crate::codegen::platform::Target::parse(target).unwrap());
            assert_eq!(calls(&module), (1, 0), "{target}");
        }
    }
}

/// Calls requiring return validation, references, variadics, or non-scalar captures stay generic.
#[test]
fn known_loop_closure_rejects_fallible_or_non_scalar_targets() {
    for source in [
        fixture("function(int $x):int{return $x+1;}", "$sum+=$f($i);", "$i++"),
        fixture("function($x){throw new RuntimeException('stop');}", "$sum+=$f($i);", "$i++"),
        fixture("function(&$x){return $x;}", "$sum+=$f($i);", "$i++"),
        fixture("function(...$xs){return 1;}", "$sum+=$f($i);", "$i++"),
        "<?php $word='x'; $f=function($x)use($word){return $x+1;};$sum=0;for($i=0;$i<$argc;$i++){$sum+=$f($i);}echo$sum;".to_string(),
        "<?php $k=0; $f=function($x)use(&$k){$k++;return $x;};$sum=0;for($i=0;$i<$argc;$i++){$sum+=$f($i);}echo$sum;".to_string(),
        "<?php function nestedInc($x){return $x+1;} $f=function($x){return nestedInc($x);};$sum=0;for($i=0;$i<$argc;$i++){$sum+=$f($i);}echo$sum;".to_string(),
    ] {
        let module = super::lower_source(&source);
        assert_eq!(calls(&module), (0, 1), "{source}");
    }
}

/// Coercion, expression arguments, named/spread arguments and omitted defaults keep the binder.
#[test]
fn known_loop_closure_rejects_non_exact_arguments() {
    for source in [
        fixture("function(float $x):float{return $x;}", "$sum+=$f($i);", "$i++"),
        fixture("function($x){return $x;}", "$sum+=$f($i+1);", "$i++"),
        fixture("function($x){return $x;}", "$sum+=$f(x:$i);", "$i++"),
        fixture("function($x){return $x;}", "$sum+=$f(...[$i]);", "$i++"),
        fixture("function($x=1){return $x;}", "$sum+=$f();", "$i++"),
    ] {
        assert_eq!(calls(&super::lower_source(&source)), (0, 1), "{source}");
    }
}

/// Reassignment on any backedge invalidates the entry target before the first emitted call.
#[test]
fn known_loop_closure_rejects_every_backedge_writer() {
    for body in [
        "$sum+=$f($i);$f=$g;",
        "$sum+=$f($i);if($i==0){$f=$g;continue;}",
        "$sum+=$f($i);for($j=0;$j<1;$j++){$f=$g;}",
        "$sum+=$f($i);$alias=&$f;",
        "$sum+=$f($i);eval('$f=null;');",
    ] {
        let source = format!("<?php $n=$argc;$f=function($x){{return $x+1;}};$g=function($x){{return $x+10;}};$sum=0;for($i=0;$i<$n;$i++){{{body}}}echo$sum;");
        assert_eq!(calls(&super::lower_source(&source)), (0, 1), "{source}");
    }
    let source = "<?php $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;for($i=0;$i<$argc;$f=$g){$sum+=$f($i);$i++;}echo$sum;";
    assert_eq!(calls(&super::lower_source(source)), (0, 1));
}

/// A do/while reassignment must discard the stale target, while stable legacy calls remain direct.
#[test]
fn known_loop_closure_checks_do_while_identity_before_emission() {
    let source = "<?php $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;$i=0;do{$sum+=$f($i);if($i==0){$f=$g;}$i++;}while($i<$argc);echo$sum;";
    assert_eq!(calls(&super::lower_source(source)), (0, 1));
    let stable = "<?php $f=function(int $x):int{return $x+1;};$sum=0;$i=0;do{$sum+=$f($i);$i++;}while($i<$argc);echo$sum;";
    assert_eq!(calls(&super::lower_source(stable)), (1, 0));
}

/// Typed identity returns and by-value scalar captures use exact scalar ABI operands.
#[test]
fn known_loop_closure_preserves_scalar_capture_snapshots() {
    let source = "<?php $k=3;$f=function($x)use($k){return $x+$k;};$k=100;$sum=0;for($i=0;$i<$argc;$i++){$sum+=$f($i);}echo$sum;";
    assert_eq!(calls(&super::lower_source(source)), (1, 0));
    let typed = fixture("function(int $x):int{return $x;}", "$sum+=$f($i);", "$i++");
    assert_eq!(calls(&super::lower_source(&typed)), (1, 0));
}

/// A condition-created closure must not bypass the invariant-entry gates in the body.
#[test]
fn known_loop_closure_discards_fresh_condition_bindings() {
    let source = "<?php $sum=0;for($i=0;($f=function($x){return $x+1;}) && $i<$argc;$i++){$sum+=$f($i);}echo$sum;";
    assert_eq!(calls(&super::lower_source(source)), (0, 1));
}

/// Existing named-callable condition lowering keeps its original direct path.
#[test]
fn known_loop_closure_preserves_non_closure_header_facts() {
    let source = "<?php function checkIteration(int $i):bool{return $i<2;} $predicate=checkIteration(...);for($i=0;$predicate($i);$i++){echo$i;}";
    assert_eq!(calls(&super::lower_source(source)), (1, 0));
}

/// Mixed arithmetic cannot hide string producers or unknown global input behind deopt effects.
#[test]
fn known_loop_closure_rejects_non_scalar_body_state() {
    for source in [
        fixture("function($x){$word='abc';return $x+$word;}", "$sum+=$f($i);", "$i++"),
        "<?php $input=1; $f=function($x){global $input;return $input+$x;};$sum=0;for($i=0;$i<$argc;$i++){$sum+=$f($i);}echo$sum;".to_string(),
    ] {
        assert_eq!(calls(&super::lower_source(&source)), (0, 1), "{source}");
    }
}

/// Opaque calls, reference aliases, catch bindings and indirect writers discard entry facts.
#[test]
fn known_loop_closure_rejects_indirect_identity_hazards() {
    for (prefix, body) in [
        ("$alias=&$f;", "$sum+=$f($i);"),
        ("function poke(&$value):void{$value=function($x){return $x+10;};}", "$sum+=$f($i);poke($f);"),
        ("function poke():void{global $f;$f=null;}", "$sum+=$f($i);poke();"),
        ("", "$sum+=$f($i);foreach([1] as $v){$f=$g;}"),
        ("", "try{$sum+=$f($i);}catch(Exception $f){}"),
        ("", "$sum+=$f($i);function declaredInside():int{return 1;}"),
    ] {
        let source = format!("<?php $f=function($x){{return $x+1;}};$g=function($x){{return $x+10;}};{prefix}$sum=0;for($i=0;$i<$argc;$i++){{{body}}}echo$sum;");
        assert_eq!(calls(&super::lower_source(&source)).1, 1, "{source}");
    }
}

/// Typed scalar identity arguments and returns use the existing direct ABI on every target.
#[test]
fn known_loop_closure_exact_scalar_signatures_on_all_targets() {
    for (signature, result_type, argument) in [("int", "int", "1"), ("float", "float", "1.5"),
        ("bool", "bool", "true"), ("false", "false", "false"),
        ("int", "float", "1")]
    {
        let source = fixture(&format!("function({signature} $x):{result_type}{{return $x;}}"),
            &format!("$result=$f({argument});"), "$i++");
        for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
            let module = super::lower_source_at_for_target(&source, std::path::Path::new("main.php"),
                std::path::Path::new("."), crate::codegen::platform::Target::parse(target).unwrap());
            assert_eq!(calls(&module), (1, 0), "{target}: {signature}");
        }
    }
}

/// A pure nested static call must stay generic even after its effect summary is refined.
#[test]
fn known_loop_closure_rejects_refined_static_calls_on_all_targets() {
    let source = "<?php class Step{public static function inc($x){return$x+1;}}$f=function($x){return Step::inc($x);};$s=0;for($i=0;$i<$argc;$i++){$s+=$f($i);}echo$s;";
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(source, std::path::Path::new("main.php"),
            std::path::Path::new("."), crate::codegen::platform::Target::parse(target).unwrap());
        assert_eq!(calls(&module), (0, 1), "{target}");
    }
}

/// Branching bodies and explicit casts remain outside the initial primitive-body proof.
#[test]
fn known_loop_closure_rejects_unproved_body_shapes() {
    for source in [
        fixture("function($x){if($x>0){return$x;}return 0;}", "$sum+=$f($i);", "$i++"),
        fixture("function($x):int{return (int)($x+1);}", "$sum+=$f($i);", "$i++"),
    ] {
        assert_eq!(calls(&super::lower_source(&source)), (0, 1), "{source}");
    }
}

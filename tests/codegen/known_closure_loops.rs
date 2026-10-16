//! Purpose:
//! PHP and ownership regressions for known closure identities carried through loops.
//!
//! Called from:
//! - `cargo test --test codegen_tests known_closure_loops`.
//!
//! Key details:
//! - Every fixture runs with IR optimization on/off and heap diagnostics.
//! - Boundary errors and backedge mutations retain descriptor behavior.

use crate::support::*;

/// Runs both optimizer modes and optionally checks the same program with PHP.
fn check_closure_loop(source: &str, expected: &str) {
    check_closure_loop_heap(source, expected, 0);
}

/// Bounds pre-existing descriptor retention while requiring clean heaps for new direct calls.
fn check_closure_loop_heap(source: &str, expected: &str, max_live_bytes: usize) {
    for mode in ["--ir-opt=off", "--ir-opt=on"] {
        let dir = make_cli_test_dir("elephc_known_closure_loop");
        let php_path = dir.join("main.php");
        fs::write(&php_path, source).expect("write closure loop fixture");
        let compile = elephc_cli_command(&dir).args([mode, "--heap-debug"])
            .arg(&php_path).output().expect("compile closure loop fixture");
        if !compile.status.success() {
            fs::remove_dir_all(&dir).expect("remove failed closure fixture");
            panic!("{mode}: {}", String::from_utf8_lossy(&compile.stderr));
        }
        let output = run_binary(&dir.join("main"), &dir);
        let oracle = std::env::var_os("ELEPHC_PHP_CHECK").map(|_| {
            Command::new("php").arg("-n").arg(&php_path).output().expect("run PHP oracle")
        });
        fs::remove_dir_all(dir).expect("remove closure loop fixture");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{mode}: {stderr}");
        assert_eq!(output.stdout, expected.as_bytes(), "{mode}: {stderr}");
        if max_live_bytes == 0 {
            assert!(stderr.contains("HEAP DEBUG: leak summary: clean"), "{mode}: {stderr}");
        } else {
            let live_bytes = stderr.lines().find(|line| line.starts_with("HEAP DEBUG: allocs="))
                .and_then(|line| line.split_whitespace().find_map(|field| {
                    field.strip_prefix("live_bytes=").and_then(|value| value.parse::<usize>().ok())
                })).expect("heap report with live byte count");
            assert!(live_bytes <= max_live_bytes, "{mode}: {stderr}");
        }
        if let Some(php) = oracle {
            assert!(php.status.success(), "{}", String::from_utf8_lossy(&php.stderr));
            assert_eq!(output.stdout, php.stdout, "PHP mismatch with {mode}");
        }
    }
}

/// For/while direct calls retain capture snapshots and balance their scalar owners.
#[test]
fn test_known_closure_loop_scalar_calls_and_capture_snapshot() {
    let source = r#"<?php
function forTotal(int $n): int {
    $k = 3;
    $f = function($x) use ($k) { return $x + $k; };
    $k = 100;
    $sum = 0;
    for ($i = 0; $i < $n; $i++) { $sum += $f($i); }
    return (int) $sum;
}
function whileTotal(int $n): int {
    $f = function($x) { return $x + 1; };
    $sum = 0;
    $i = 0;
    while ($i < $n) { $sum += $f($i); $i++; }
    return (int) $sum;
}
echo forTotal($argc + 3), '|', whileTotal($argc + 3);
"#;
    check_closure_loop(source, "18|10");
}

/// Scalar captures preserve each runtime tag, including null and floating-point values.
#[test]
fn test_known_closure_loop_scalar_capture_types() {
    for (value, expected) in [("3", "integer"), ("1.5", "double"), ("true", "boolean"),
        ("false", "boolean"), ("null", "NULL")]
    {
        let source = format!("<?php $k={value};$f=function($x)use($k){{return $k;}};$result=null;for($i=0;$i<3;$i++){{$result=$f($i);}}echo gettype($result);");
        check_closure_loop(&source, expected);
    }
}

/// Every iteration observes a reassigned descriptor, including do/while and continue edges.
#[test]
fn test_known_closure_loop_identity_backedges() {
    for source in [
        "<?php $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;$i=0;do{$sum+=$f($i);if($i==0){$f=$g;}$i++;}while($i<3);echo$sum;",
        "<?php $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;for($i=0;$i<3;$i++){$sum+=$f($i);$f=$g;continue;}echo$sum;",
        "<?php $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;for($i=0;$i<3;$i++){$sum+=$f($i);for($j=0;$j<1;$j++){$f=$g;}}echo$sum;",
    ] {
        check_closure_loop(source, "24");
    }
}

/// Integer identity preserves both platform limits without owned result cells.
#[test]
fn test_known_closure_loop_integer_identity_limits() {
    for (value, expected) in [("PHP_INT_MAX", "9223372036854775807"),
        ("PHP_INT_MIN", "-9223372036854775808")]
    {
        let source = format!("<?php $f=function(int $x):int{{return $x;}};$value={value};for($i=0;$i<2;$i++){{$result=$f($value);}}echo$result;");
        check_closure_loop(&source, expected);
    }
}

/// Mixed floats keep descriptor coercion rather than entering the exact-scalar call path.
/// The existing mixed-return helper retains one 40-byte cell on the baseline too.
#[test]
fn test_known_closure_loop_mixed_argument_keeps_descriptor_coercion() {
    let source = "<?php error_reporting(0);function opaqueValue($value):mixed{return $value;}$f=function(int $x):int{return $x;};$value=opaqueValue(1.5);for($i=0;$i<2;$i++){$result=$f($value);}echo$result;";
    check_closure_loop_heap(source, "1", 40);
}

/// Overflowed typed returns retain descriptor validation and PHP's TypeError.
/// The unchanged descriptor error path retains one 40-byte cell on the baseline too.
#[test]
fn test_known_closure_loop_fallible_return_keeps_descriptor_validation() {
    let source = r#"<?php
function opaqueValue($value): mixed { return $value; }
$value = opaqueValue(PHP_INT_MAX);
$f = function(int $x): int { return $x + 1; };
try { for ($i = 0; $i < 2; $i++) { $result = $f($value); } }
catch (TypeError $error) { echo 'return-error'; }
"#;
    check_closure_loop_heap(source, "return-error", 40);
}

/// Opaque by-reference mutation keeps the descriptor identity current.
/// The baseline retains two callable blocks on this existing by-reference path.
#[test]
fn test_known_closure_loop_by_reference_replacement_keeps_descriptor_identity() {
    let source = "<?php function replaceClosure(&$f,$g):void{$f=$g;} $f=function($x){return $x+1;};$g=function($x){return $x+10;};$sum=0;for($i=0;$i<3;$i++){$sum+=$f($i);replaceClosure($f,$g);}echo$sum;";
    check_closure_loop_heap(source, "24", 160);
}

/// Exact scalar arguments preserve tags and untyped arithmetic promotes at both integer limits.
#[test]
fn test_known_closure_loop_scalar_arguments_and_arithmetic_overflow() {
    for (value, expected) in [("1.5", "double"), ("true", "boolean"),
        ("false", "boolean"), ("null", "NULL")]
    {
        let source = format!("<?php $f=function($x){{return $x;}};$value={value};$result=null;for($i=0;$i<2;$i++){{$result=$f($value);}}echo gettype($result);");
        check_closure_loop(&source, expected);
    }
    for (value, operation) in [("PHP_INT_MAX", "+1"), ("PHP_INT_MIN", "-1")] {
        let source = format!("<?php $f=function($x){{return $x{operation};}};$value={value};$result=null;for($i=0;$i<2;$i++){{$result=$f($value);}}echo gettype($result);");
        check_closure_loop(&source, "double");
    }
}

/// A refined nonthrowing static call still uses the descriptor path and balances its owners.
#[test]
fn test_known_closure_loop_nested_static_call_stays_generic() {
    let source = "<?php class Step{public static function inc($x){return$x+1;}}$f=function($x){return Step::inc($x);};$sum=0;for($i=0;$i<$argc+2;$i++){$sum+=$f($i);}echo$sum;";
    check_closure_loop(source, "6");
}

/// Native floating-point inputs and result registers preserve PHP values.
#[test]
fn test_known_closure_loop_floating_abis() {
    for (source, expected) in [
        ("<?php $f=function(float $x):float{return$x+1.0;};$value=(float)$argc;$result=0.0;for($i=0;$i<2;$i++){$result=$f($value);}var_dump($result);", "float(2)\n"),
        ("<?php $f=function(int $x):float{return(float)$x;};$value=(int)($argc+13);$result=0.0;for($i=0;$i<2;$i++){$result=$f($value);}var_dump($result);", "float(14)\n"),
    ] {
        check_closure_loop(source, expected);
    }
}

/// Arguments retyped on a backedge keep descriptor decoding on every iteration.
#[test]
fn test_known_closure_loop_retyped_argument_backedges() {
    for replacement in ["1.5", "null"] {
        let expected = if replacement == "1.5" { "2.5" } else { "1" };
        for loop_body in [
            format!("for($i=0;$i<2;$i++){{$out=$f($value);$value={replacement};}}"),
            format!("$i=0;while($i<2){{$out=$f($value);$value={replacement};$i++;}}"),
            format!("$i=0;do{{$out=$f($value);$value={replacement};$i++;}}while($i<2);"),
        ] {
            let source = format!("<?php $f=function($x){{return $x+1;}};$value=10;$out=0;{loop_body}echo$out;");
            check_closure_loop(&source, expected);
        }
    }
}

/// Update clauses, nested writes, and continue paths cannot reuse the first argument type.
#[test]
fn test_known_closure_loop_argument_writes_on_other_backedges() {
    for (loop_body, expected) in [
        ("for($i=0;$i<2;$value=1.5){$out=$f($value);$i++;}", "2.5"),
        ("$i=0;while($i++<2 && ($value=($i==1?10:1.5))>=0){$out=$f($value);}", "2.5"),
        ("for($i=0;$i<2;$i++){$out=$f($value);if($i==0){$value=null;continue;}}", "1"),
        ("for($i=0;$i<2;$i++){$out=$f($value);for($j=0;$j<1;$j++){$value=1.5;}}", "2.5"),
        ("for($i=0;$i<2;$i++){$out=$f($value);($value=1.5);}", "2.5"),
    ] {
        let source = format!("<?php $f=function($x){{return$x+1;}};$value=10;$out=0;{loop_body}echo$out;");
        check_closure_loop(&source, expected);
    }
}

/// Unwritten locals and literal scalars keep correct direct-call argument values.
#[test]
fn test_known_closure_loop_unwritten_argument_controls() {
    for argument in ["$value", "11"] {
        let source = format!("<?php $value=$argc+10;$f=function($x){{return$x+1;}};$out=0;for($i=0;$i<2;$i++){{$out=$f({argument});}}echo$out;");
        check_closure_loop(&source, "12");
    }
}

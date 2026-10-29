//! Purpose:
//! Regressions for hosted diagnostics, value admission, termination and default state costs.
//!
//! Called from:
//! - The opt-in real-PHP `phpext_tests` integration binary.
//!
//! Key details:
//! - Each comparison executes the same source through PHP and the produced extension.
//! - Default heap coverage checks loading without invoking an export.

use super::*;

/// Input and output accept exactly 64 arrays, independent of nested Mixed wrappers.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn array_depth_limit_counts_arrays_only() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("depth_limit", r#"<?php
#[Export] function php_depth_identity(array $value): array { return $value; }
#[Export] function php_depth_result(int $depth): array {
    $value = [];
    for ($i = 1; $i < $depth; $i++) { $value = [$value]; }
    return $value;
}
"#);
    let output = php(&sdk, &module).args(["-r", r#"
function nested($depth, $hash) {
    $value = 1;
    for ($i = 0; $i < $depth; $i++) { $value = $hash ? ['child' => $value] : [$value]; }
    return $value;
}
foreach ([1, 32, 33, 63, 64] as $depth) {
    foreach ([false, true] as $hash) {
        $value = nested($depth, $hash);
        if (php_depth_identity($value) !== $value) { exit(10); }
    }
}
$expected = [];
for ($i = 1; $i < 64; $i++) { $expected = [$expected]; }
if (php_depth_result(64) !== $expected) { exit(11); }
try { php_depth_identity(nested(65, false)); exit(12); }
catch (Error $error) {
    if (!str_contains($error->getMessage(), 'maximum nesting depth (64)')) { throw $error; }
}
try { php_depth_result(65); exit(13); }
catch (Error $error) {
    if (!str_contains($error->getMessage(), 'result exceeds the maximum nesting depth (64)')) { throw $error; }
}
if (php_depth_identity([1]) !== [1]) { exit(14); }
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Escaped PHP bytes stay identical in optional values and Reflection defaults.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn binary_string_defaults_match_php() {
    compare(r#"<?php
#[Export] function php_bytes(string $binary = "\xff\x80\0", string $unicode = "caf\u{e9}",
    string $octal = "\200", mixed $dynamic = "\377", string $last = 'last'): array {
    return [$binary, $unicode, $octal, $dynamic, $last];
}
"#, r#"
$defaults = array_map(fn($parameter) => bin2hex($parameter->getDefaultValue()),
    (new ReflectionFunction('php_bytes'))->getParameters());
echo json_encode([array_map('bin2hex', php_bytes()),
    array_map('bin2hex', php_bytes(last: 'named')), $defaults]);
"#);
}

/// PHP parameter names need string literals rather than the C identifier grammar.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn non_ascii_parameter_names_match_php() {
    // PHP accepts a non-breaking space inside a variable name; C can normalize it.
    let name = format!("gap{}name", '\u{a0}');
    let source = format!(r#"<?php
#[Export] function php_names(float $t° = 1.0, float $x© = 2.0, float $y÷ = 3.0,
    float ${name} = 4.0): array {{ return [$t°, $x©, $y÷, ${name}]; }}
"#);
    let driver = format!(r#"
echo json_encode([php_names(), php_names(y÷: 4.5), php_names({name}: 6.0),
    array_map(fn($parameter) => $parameter->getName(),
        (new ReflectionFunction('php_names'))->getParameters())]);
"#);
    compare(&source, &driver);
}

/// Unsupported dynamic results raise a host Error and leave the next call usable.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn unsupported_dynamic_results_raise_host_errors() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("unsupported", r#"<?php
#[Export] function php_closure(): mixed { return fn() => 1; }
#[Export] function php_callable_array(): array { return ['callback' => strlen(...)]; }
#[Export] function php_object(): mixed { return new RuntimeException('unsupported result'); }
#[Export] function php_recover(int $value): int { return $value + 1; }
"#);
    let output = php(&sdk, &module).args(["-r", r#"
foreach (['php_closure', 'php_callable_array', 'php_object'] as $function) {
    try { $function(); exit(10); }
    catch (Error $error) {
        if (!str_contains($error->getMessage(), 'boundary supports only int, float, bool, string, array and null')) { throw $error; }
    }
    if (php_recover(41) !== 42) { exit(11); }
}
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Explicit nullable declarations retain omitted, explicit and named null defaults.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn explicit_nullable_defaults_match_php() {
    compare(r#"<?php
#[Export] function php_defaults(?int $integer = null, ?float $float = null,
    ?bool $boolean = null, ?string $string = null, ?array $array = null,
    ?false $flag = null, mixed $dynamic = null, null $nothing = null): array {
    return [$integer, $float, $boolean, $string, $array, $flag, $dynamic, $nothing];
}
#[Export] function php_float_default(float $value = 5): float { return $value; }
"#, r#"
echo json_encode([php_defaults(), php_defaults(null, null, null, null, null, null, null, null),
    php_defaults(string: 'ok', flag: false, dynamic: ['key' => 1]), php_float_default()]);
"#);
}

/// Nullable and union float returns widen native integers before Zend validates arginfo.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn float_union_returns_match_php() {
    compare(r#"<?php
#[Export] function php_nullable_float(int $value): ?float { return $value; }
#[Export] function php_float_or_string(int $value): float|string { return $value; }
"#, r#"
var_dump(php_nullable_float(3), php_float_or_string(4));
"#);
}

/// A small shared array graph cannot expand into an unbounded malloc-owned transport tree.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn shared_array_transport_has_a_per_call_budget() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("budget", "<?php #[Export] function php_copy(array $value): array { return $value; }");
    let output = php(&sdk, &module).args(["-r", r#"
$value = [1];
for ($i = 0; $i < 16; $i++) { $value = [$value, $value]; }
try { php_copy($value); exit(10); }
catch (Error $error) {
    if (!str_contains($error->getMessage(), 'transport allocation budget exceeded')) { throw $error; }
}
$input = [[1], [2]];
$result = php_copy($input);
$result[0][0] = 3;
if ($input !== [[1], [2]] || $result !== [[3], [2]]) { exit(11); }
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Shared fatal helpers become recoverable PHP Errors without writing around host diagnostics.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn fatal_helpers_do_not_write_raw_stderr() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("capacity", "<?php #[Export] function php_fill(int $count): array { return array_fill(0, $count, 0); } #[Export] function php_buffer(int $count): int { $buffer = buffer_new<int>($count); return $buffer[0]; } #[Export] function php_buffer_index(int $index): int { $buffer = buffer_new<int>(1); return $buffer[$index]; }");
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
try { php_fill(PHP_INT_MAX); exit(10); } catch (Error $error) {}
try { php_buffer(-1); exit(12); } catch (Error $error) {}
try { php_buffer_index(1); exit(13); } catch (Error $error) {}
if (php_fill(2) !== [0, 0]) { exit(11); }
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Compares independently executed source and native functions, preserving status and diagnostics.
fn compare(source: &str, driver: &str) {
    compare_with_options(source, driver, &[]);
}

/// Compares source and extension execution with extra compiler options.
fn compare_with_options(source: &str, driver: &str, options: &[&str]) {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile_with_options("review", source, options);
    let source_path = project.0.join("review.php");
    let script = format!("if (!extension_loaded('review')) {{ require {:?}; }}\n{driver}",
        source_path.to_str().unwrap());
    let reference = Command::new(sdk.php_binary()).args(["-n", "-d", "display_errors=0", "-r", &script])
        .output().unwrap();
    let native = php(&sdk, &module).args(["-d", "display_errors=0", "-r", &script]).output().unwrap();
    assert_eq!(native.status.code(), reference.status.code(), "{}", String::from_utf8_lossy(&native.stderr));
    assert!(native.stdout == reference.stdout, "native: {}\nPHP: {}",
        String::from_utf8_lossy(&native.stdout), String::from_utf8_lossy(&reference.stdout));
    assert!(native.stderr.is_empty(), "{}", String::from_utf8_lossy(&native.stderr));
    assert!(reference.stderr.is_empty(), "{}", String::from_utf8_lossy(&reference.stderr));
}

/// Default-sized zero storage stays out of the file and unused requests do not copy it.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn default_module_has_sparse_state_and_lazy_requests() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile_with_options("defaults", "<?php #[Export] function php_default_sum(int $a, int $b): int { return $a + $b; }", &[]);
    assert!(fs::metadata(&module).unwrap().len() < 32 * 1024 * 1024,
        "default module stores its zero-initialized heap in the file");
    let script = "echo getrusage()['ru_maxrss'];";
    let baseline = Command::new(sdk.php_binary()).args(["-n", "-r", script]).output().unwrap();
    let loaded = php(&sdk, &module).args(["-r", script]).output().unwrap();
    assert!(baseline.status.success() && loaded.status.success());
    assert!(loaded.stderr.is_empty(), "{}", String::from_utf8_lossy(&loaded.stderr));
    let rss = |bytes: &[u8]| String::from_utf8_lossy(bytes).parse::<u64>().unwrap();
    let unit = if cfg!(target_os = "macos") { 1 } else { 1024 };
    assert!(rss(&loaded.stdout).saturating_sub(rss(&baseline.stdout)) * unit < 64 * 1024 * 1024,
        "loading an unused default module eagerly touches its request heap");
    let invoked = php(&sdk, &module).args(["-r", "echo php_default_sum(20, 22);"]).output().unwrap();
    assert!(invoked.status.success(), "{}", String::from_utf8_lossy(&invoked.stderr));
    assert_eq!(invoked.stdout, b"42");
    assert!(invoked.stderr.is_empty());
}

/// Native deprecations preserve severity, message and the host's reporting mask.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn diagnostics_preserve_php_severity_and_mask() {
    compare("<?php #[Export] function php_float_key(array $value, float $key): array { $value[$key] = 1; return $value; }", r#"
$seen = [];
set_error_handler(function ($level, $message) use (&$seen) { $seen[] = [$level, $message]; return true; });
echo json_encode([php_float_key([], 1.5), $seen]), "\n";
error_reporting(E_ALL & ~E_DEPRECATED);
set_error_handler(function ($level, $message) { if (error_reporting() & $level) { throw new ErrorException($message, 0, $level); } return true; });
try { echo json_encode(php_float_key([], 1.5)); } catch (Throwable $e) { echo get_class($e), ':', $e->getMessage(); }
"#);
}

/// A pending host error-handler exception leaves native finally callbacks usable.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_error_handler_exceptions_leave_native_finally_callbacks_usable() {
    compare(r#"<?php
#[Export] function php_finally_callbacks(array $value): void {
    try { echo $value['first']; }
    finally { echo 'cleanup'; echo $value['second']; echo 'more'; }
}
"#, r#"
ob_start(function ($bytes) {
    if ($bytes === '') { return ''; }
    return "[$bytes]";
}, 1);
$warnings = [];
set_error_handler(function ($level, $message) use (&$warnings) {
    if (str_contains($message, '"first"')) { throw new LogicException('warning'); }
    $warnings[] = [$level, $message];
    return true;
});
try { php_finally_callbacks([]); } catch (LogicException $error) { echo 'caught'; }
echo json_encode($warnings), 'after';
ob_end_flush();
"#);
}

/// Output-handler exceptions run both flattened and kept native finally regions.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn output_handler_exceptions_run_native_finally_regions() {
    compare(r#"<?php
#[Export] function php_output_finally(bool $kept): int {
    static $state = 0;
    try {
        echo $kept ? 'kept' : 'flat';
        if ($kept) { throw new RuntimeException('unreached'); }
        $state += 10;
    } finally {
        $state += 100;
        echo 'cleanup';
    }
    return $state;
}
"#, r#"
foreach ([false, true] as $kept) {
    ob_start(function ($bytes) { throw new LogicException('output'); }, 1);
    try { php_output_finally($kept); exit(10); }
    catch (LogicException $error) { echo 'caught'; }
    ob_end_flush();
    echo ':', php_output_finally(false), '|';
}
"#);
}

/// A native finally return discards a pending host callback exception like PHP.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn native_finally_returns_override_host_callback_exceptions() {
    compare(r#"<?php
#[Export] function php_finally_return(array $value): int {
    try { echo $value['missing']; } finally { return 7; }
}
"#, r#"
set_error_handler(function () { throw new LogicException('host'); });
var_dump(php_finally_return([]));
echo 'after';
"#);
}

/// A native finally exception replaces and chains a pending host callback exception.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn native_finally_exceptions_replace_host_callback_exceptions() {
    compare(r#"<?php
#[Export] function php_finally_throw(array $value): void {
    try { echo $value['missing']; }
    finally { throw new RuntimeException('native'); }
}
"#, r#"
set_error_handler(function () { throw new LogicException('host'); });
try { php_finally_throw([]); }
catch (Throwable $error) {
    $previous = $error->getPrevious();
    echo get_class($error), ':', $error->getMessage(), ':',
        get_class($previous), ':', $previous->getMessage();
}
"#);
}

/// A caught native replacement retires its host predecessor unless that chain is rethrown.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn caught_finally_replacements_settle_host_predecessors() {
    compare(r#"<?php
#[Export] function php_absorb_replacement(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { throw new TypeError('after'); }
}
#[Export] function php_rethrow_replacement(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { throw $error; }
}
#[Export] function php_wrap_replacement(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { throw new TypeError('outer', 0, $error); }
}
"#, r#"
$dump = function (Throwable $error): void {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
    echo "\n";
};
set_error_handler(function () { throw new DomainException('host'); });
foreach (['php_absorb_replacement', 'php_rethrow_replacement', 'php_wrap_replacement'] as $function) {
    try { $function([]); }
    catch (Throwable $error) { $dump($error); }
}
"#);
}

/// Replacement identity is runtime-proven, one-shot, retained, and spliced at its original tail.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_replacement_tracking_uses_exact_runtime_chain_points() {
    compare(r#"<?php
#[Export] function php_catch_inside_finally(array $value): void {
    try { echo $value['missing']; }
    finally {
        try { throw new RuntimeException('inner'); }
        catch (RuntimeException $error) {}
    }
}
#[Export] function php_absorb_then_host(array $value): mixed {
    try { try { echo $value['first']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) {}
    return $value['second'];
}
function php_absorb_for_reuse(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) {}
}
#[Export] function php_throw_after_absorb(array $value): void {
    php_absorb_for_reuse($value);
    throw new RuntimeException('later');
}
#[Export] function php_splice_replacement_tail(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { $saved = $error; }
    try { throw new TypeError('pending'); }
    finally { throw new Exception('outer', 0, $saved); }
}
"#, r#"
$dump = function (Throwable $error): void {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
    echo "\n";
};
set_error_handler(function ($level, $message) { throw new DomainException('host'); });
foreach ([
    'php_catch_inside_finally',
    'php_absorb_then_host',
    'php_throw_after_absorb',
    'php_splice_replacement_tail',
] as $function) {
    try { $function([]); }
    catch (Throwable $error) { $dump($error); }
}
"#);
}

/// Replacement records track live native chains instead of total catch history.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn replacement_records_follow_live_native_chains() {
    compare(r#"<?php
#[Export] function php_many_absorbed_replacements(array $value): int {
    for ($i = 0; $i < 65; $i++) {
        try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
        catch (RuntimeException $error) {}
    }
    return 1;
}
#[Export] function php_catch_wrapped_replacement(array $value): mixed {
    try {
        try { try { echo $value['first']; } finally { throw new RuntimeException('native'); } }
        finally { throw new Exception('wrapper'); }
    } catch (Exception $error) {}
    return $value['second'];
}
#[Export] function php_reuse_replacement_tail(array $value): void {
    $saved = new RuntimeException('native');
    try { try { echo $value['first']; } finally { throw $saved; } }
    catch (RuntimeException $error) {}
    try { echo $value['second']; } finally { throw $saved; }
}
#[Export] function php_replacement_with_intrinsic_previous(array $value): void {
    try { echo $value['missing']; }
    finally { throw new RuntimeException('native', 0, new Exception('intrinsic')); }
}
#[Export] function php_rethrow_replacement_into_host_callback(array $value): void {
    try { try { echo $value['first']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) {
        try { throw $error; } finally { echo $value['second']; }
    }
}
#[Export] function php_nested_sweep_inner(array $value): int {
    for ($i = 0; $i < 2; $i++) {
        try { try { echo $value['inner']; } finally { throw new RuntimeException('inner'); } }
        catch (RuntimeException $error) {}
    }
    return 7;
}
#[Export] function php_nested_sweep_outer(array $value): void {
    try { try { echo $value['outer']; } finally { throw new RuntimeException('outer'); } }
    catch (RuntimeException $error) {
        echo '[', php_nested_sweep_inner($value), ']';
        throw $error;
    }
}
"#, r#"
$dump = function (Throwable $error): void {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
    echo "\n";
};
$host = 0;
set_error_handler(function () use (&$host) { throw new DomainException('host' . ++$host); });
echo php_many_absorbed_replacements([]), "\n";
$host = 0;
try { php_catch_wrapped_replacement([]); }
catch (Throwable $error) { $dump($error); }
$host = 0;
try { php_reuse_replacement_tail([]); }
catch (Throwable $error) { $dump($error); }
$host = 0;
try { php_replacement_with_intrinsic_previous([]); }
catch (Throwable $error) { $dump($error); }
$host = 0;
try { php_rethrow_replacement_into_host_callback([]); }
catch (Throwable $error) { $dump($error); }
$host = 0;
try { php_nested_sweep_outer([]); }
catch (Throwable $error) { $dump($error); }
"#);
}

/// The bounded live-pin failure reports the specific producer limit.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn live_replacement_pin_limit_is_descriptive() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("live_replacement_pin_limit", r#"<?php
#[Export] function php_live_replacement_pin_limit(array $value): int {
    $saved = new RuntimeException('native');
    for ($i = 0; $i < 65; $i++) {
        try { try { echo $value['missing']; } finally { throw $saved; } }
        catch (RuntimeException $error) {}
    }
    return 1;
}
"#);
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
set_error_handler(function () { throw new DomainException('host'); });
try { php_live_replacement_pin_limit([]); echo 'unexpected'; }
catch (Error $error) { echo $error->getMessage(); }
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout,
        b"PHP extension exception chain exceeds the maximum live replacement pin count (64)");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Wide exports execute replacement-owner cleanup beyond AArch64's small frame offsets.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn wide_signature_replacement_cleanup_matches_php() {
    compare(r#"<?php
#[Export] function php_wide_replacement_cleanup(
    int $v0, int $v1, int $v2, int $v3, int $v4,
    int $v5, int $v6, int $v7, int $v8, int $v9,
    int $v10, int $v11, int $v12, int $v13, int $v14,
    int $v15, int $v16, int $v17, int $v18, int $v19,
    int $v20, int $v21, int $v22, int $v23, array $value
): void {
    try { echo $value['missing']; }
    finally { throw new RuntimeException('wide-' . ($v0 + $v23)); }
}
"#, r#"
set_error_handler(function () { throw new DomainException('host'); });
try {
    php_wide_replacement_cleanup(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, []
    );
} catch (Throwable $error) {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
}
"#);
}

/// Fiber graceful exit never becomes a reusable host predecessor splice tail.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn graceful_exit_replacement_splices_do_not_reuse_released_objects() {
    compare(r#"<?php
#[Export] function php_graceful_splice(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { $saved = $error; }
    try { throw new TypeError('pending'); }
    finally { throw new Exception('outer', 0, $saved); }
}
"#, r#"
$fiber = new Fiber(function (): void {
    set_error_handler(function () { Fiber::suspend('parked'); });
    php_graceful_splice([]);
});
echo $fiber->start(), ':';
try { unset($fiber); }
catch (Throwable $error) { echo get_class($error), ':', $error->getMessage(); }
echo ':after';
"#);
}

/// A host callback exception in finally adopts the older native exception as previous.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_finally_exceptions_chain_pending_native_exceptions() {
    compare(r#"<?php
#[Export] function php_pending_native(array $value): void {
    try { throw new RuntimeException('native', 123); }
    finally { echo $value['missing']; }
}
"#, r#"
set_error_handler(function () { throw new LogicException('host', 456); });
try { php_pending_native([]); }
catch (Throwable $error) {
    $previous = $error->getPrevious();
    echo get_class($error), ':', $error->getMessage(), ':', $error->getCode(), ':',
        get_class($previous), ':', $previous->getMessage(), ':', $previous->getCode();
}
"#);
}

/// Native constructor and finally previous chains remain intact across host interleavings.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn native_previous_chains_survive_php_boundary() {
    compare(r#"<?php
#[Export] function php_native_finally_chain(): void {
    try { throw new RuntimeException('first'); }
    finally { throw new TypeError('second'); }
}
#[Export] function php_constructor_chain(): void {
    throw new RuntimeException('outer', 3, new Exception('inner', 4));
}
#[Export] function php_host_spliced_chain(array $value): void {
    try {
        try { throw new RuntimeException('a'); }
        finally { throw new TypeError('b'); }
    } finally { echo $value['missing']; }
}
"#, r#"
$dump = function (Throwable $error): void {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), ':', $error->getCode(), '|';
        $error = $error->getPrevious();
    }
    echo "\n";
};
try { php_native_finally_chain(); } catch (Throwable $error) { $dump($error); }
try { php_constructor_chain(); } catch (Throwable $error) { $dump($error); }
set_error_handler(function () { throw new LogicException('host'); });
try { php_host_spliced_chain([]); } catch (Throwable $error) { $dump($error); }
"#);
}

/// Array helpers propagate a null-object host warning escape after retiring partial owners.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn array_helpers_propagate_host_warning_exceptions() {
    compare(r#"<?php
#[Export] function php_sum_warning(array $value): mixed {
    $result = array_sum($value); echo 'after-sum'; return $result;
}
#[Export] function php_product_warning(array $value): mixed {
    $result = array_product($value); echo 'after-product'; return $result;
}
#[Export] function php_flip_warning(array $value): array {
    $result = array_flip($value); echo 'after-flip'; return $result;
}
"#, r#"
$cases = [
    ['php_sum_warning', [1, [2], 3]],
    ['php_product_warning', [1, [2], 3]],
    ['php_flip_warning', [['nested']]],
];
foreach ($cases as [$function, $value]) {
    set_error_handler(function () use ($function) { throw new LogicException("host:$function"); });
    try { $function($value); echo 'missing'; }
    catch (Throwable $error) { echo get_class($error), ':', $error->getMessage(), '|'; }
    finally { restore_error_handler(); }
}
"#);
}

/// Repeated aggregate warning escapes retire their partial carry on a small native heap.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn aggregate_warning_escapes_reuse_small_heaps() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile_with_options("aggregate_heap", r#"<?php
#[Export] function php_sum_escape(array $value): mixed { return array_sum($value); }
#[Export] function php_product_escape(array $value): mixed { return array_product($value); }
#[Export] function php_after_aggregate(): int { return 42; }
"#, &["--heap-size=65536"]);
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
set_error_handler(function () { throw new LogicException('host'); });
for ($i = 0; $i < 4000; $i++) {
    $function = ($i & 1) ? 'php_sum_escape' : 'php_product_escape';
    try { $function([1, []]); exit(10); }
    catch (LogicException $error) {}
}
restore_error_handler();
if (php_after_aggregate() !== 42) { exit(11); }
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Native infrastructure failures stay authoritative over throwing cleanup output handlers.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn infrastructure_status_survives_finally_output_exceptions() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("infra_status", r#"<?php
#[Export] function php_infra_then_output(): void {
    try { array_fill(0, 100000, 0); } finally { echo 'cleanup'; }
}
#[Export] function php_after_infra(): int { return 42; }
"#);
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
ob_start(function ($bytes) {
    if ($bytes !== '') { throw new LogicException('output'); }
    return '';
}, 1);
try { php_infra_then_output(); $caught = 'missing'; }
catch (Throwable $error) { $caught = get_class($error) . ':' . $error->getMessage(); }
ob_end_clean();
echo json_encode([$caught, php_after_infra()]);
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let json = output.stdout.iter().rposition(|byte| *byte == b'[').unwrap();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout[json..]).unwrap();
    assert_eq!(value[1], 42);
    assert!(value[0].as_str().unwrap().starts_with("Error:"), "{}", String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Native infrastructure failures override finally returns and replacement exceptions.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn infrastructure_status_survives_finally_control_flow() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("infra_control_flow", r#"<?php
function php_infra_counter_impl(bool $increment): int {
    static $state = 0;
    if ($increment) { $state++; }
    return $state;
}
#[Export] function php_infra_counter(): int { return php_infra_counter_impl(false); }
#[Export] function php_infra_finally_return(): int {
    php_infra_counter_impl(true);
    try { array_fill(0, 100000, 0); } finally { return 7; }
}
#[Export] function php_infra_finally_throw(): void {
    php_infra_counter_impl(true);
    try { array_fill(0, 100000, 0); }
    finally { throw new RuntimeException('replacement'); }
}
"#);
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
foreach (['php_infra_finally_return', 'php_infra_finally_throw'] as $function) {
    try { $function(); exit(10); }
    catch (Error $error) {
        if (!str_contains($error->getMessage(), 'heap allocation failed')) { throw $error; }
    }
    if (php_infra_counter() !== 0) { exit(11); }
}
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// A helper finally return retires its host exception before later output and throws.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn nested_finally_returns_discard_host_exceptions() {
    compare(r#"<?php
function php_discard_helper(array $value): int {
    try { echo $value['missing']; } finally { return 1; }
}
function php_discard_replacement(array $value): int {
    try {
        try { echo $value['first']; }
        finally { throw new RuntimeException('native'); }
    } finally { return 1; }
}
#[Export] function php_after_discard(array $value): void {
    php_discard_helper($value);
    echo 'after|';
    throw new RuntimeException('native');
}
#[Export] function php_after_replacement_discard(array $value): mixed {
    php_discard_replacement($value);
    echo 'after-discard|';
    return $value['second'];
}
"#, r#"
$host = 0;
set_error_handler(function () use (&$host) { throw new DomainException('host' . ++$host); });
try { php_after_discard([]); }
catch (Throwable $error) {
    echo get_class($error), ':', $error->getMessage(), ':',
        $error->getPrevious() ? get_class($error->getPrevious()) : 'none';
}
$host = 0;
class HostDiscardException extends Exception {
    public function __destruct() { echo 'D(', $this->getMessage(), ')|'; }
}
set_error_handler(function () use (&$host) { throw new HostDiscardException('host' . ++$host); });
try { php_after_replacement_discard([]); }
catch (Throwable $error) {
    echo '|';
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
}
"#);
}

/// An outer native finally return discards both the host exception and its stored native tail.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn outer_finally_returns_discard_stored_previous_exceptions() {
    compare(r#"<?php
#[Export] function php_previous_then_return(array $value): int {
    try {
        try { throw new RuntimeException('first', 123); }
        finally { echo $value['missing']; }
    } finally { return 7; }
}
"#, r#"
set_error_handler(function () { throw new LogicException('host', 456); });
var_dump(php_previous_then_return([]));
echo 'after';
"#);
}

/// An outer native finally throw preserves native, host and older-native chain order.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn outer_finally_exceptions_keep_complete_previous_chain() {
    compare(r#"<?php
#[Export] function php_previous_then_throw(array $value): void {
    try {
        try { throw new RuntimeException('first', 123); }
        finally { echo $value['missing']; }
    } finally { throw new TypeError('second', 789); }
}
"#, r#"
set_error_handler(function () { throw new LogicException('host', 456); });
try { php_previous_then_throw([]); }
catch (Throwable $error) {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), ':', $error->getCode(), '|';
        $error = $error->getPrevious();
    }
}
"#);
}

/// Multiple transported native exceptions are spliced at their original host-chain anchors.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn transported_native_exceptions_keep_host_chain_order() {
    compare(r#"<?php
#[Export] function php_anchored_chain(array $value): void {
    try {
        try {
            try { echo $value['first']; }
            finally { throw new RuntimeException('native1'); }
        } finally { echo $value['second']; }
    } finally {
        try { throw new TypeError('native2'); }
        finally { echo $value['third']; }
    }
}
"#, r#"
$host = 0;
set_error_handler(function () use (&$host) { throw new LogicException('host' . ++$host); });
try { php_anchored_chain([]); }
catch (Throwable $error) {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
}
"#);
}

/// One host escape anchors every native replacement created by its nested finalizers.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn one_host_escape_anchors_multiple_native_exceptions() {
    compare(r#"<?php
#[Export] function php_shared_anchor(array $value): void {
    try { echo $value['first']; }
    finally {
        try { throw new RuntimeException('native1'); }
        finally {
            try { throw new TypeError('native2'); }
            finally { echo $value['second']; }
        }
    }
}
"#, r#"
$host = 0;
set_error_handler(function () use (&$host) { throw new LogicException('host' . ++$host); });
try { php_shared_anchor([]); }
catch (Throwable $error) {
    while ($error) {
        echo get_class($error), ':', $error->getMessage(), '|';
        $error = $error->getPrevious();
    }
}
"#);
}

/// Fiber destruction keeps graceful-exit cleanup subject to ordinary finally rules.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn fiber_destruction_uses_graceful_finally_semantics() {
    compare(r#"<?php
#[Export] function php_graceful_output(array $value): void {
    try { echo $value['missing']; } finally { echo 'cleanup'; }
}
#[Export] function php_graceful_throw(array $value): void {
    try { echo $value['missing']; } finally { throw new TypeError('second'); }
}
#[Export] function php_graceful_return(array $value): int {
    try { echo $value['missing']; } finally { return 7; }
}
"#, r#"
$run = function (string $function): void {
    $fiber = new Fiber(function () use ($function) {
        set_error_handler(function () { Fiber::suspend('parked'); return true; });
        try {
            try { $result = $function([]); echo 'result:', $result; }
            catch (Throwable $error) {
                echo 'inside:', get_class($error), ':', $error->getMessage(), ':',
                    $error->getPrevious() ? get_class($error->getPrevious()) : 'none';
            }
            echo ':continued';
        } finally { restore_error_handler(); }
    });
    echo $fiber->start(), ':';
    try { unset($fiber); }
    catch (Throwable $error) { echo 'outside:', get_class($error), ':', $error->getMessage(); }
    echo ':after|';
};
$run('php_graceful_output');
$run('php_graceful_throw');
$run('php_graceful_return');
"#);
}

/// A result-pack failure overrides a pending host callback exception and rolls state back.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn result_depth_failures_override_pending_host_exceptions() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("depth_after_host", r#"<?php
#[Export] function php_deep_after_host(array $value): array {
    try { echo $value['missing']; }
    finally {
        $result = [];
        for ($i = 0; $i < 65; $i++) { $result = [$result]; }
        return $result;
    }
}
#[Export] function php_small_after_depth(): array { return [42]; }
"#);
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
set_error_handler(function () { throw new LogicException('host'); });
try { php_deep_after_host([]); exit(10); }
catch (Error $error) {
    if (!str_contains($error->getMessage(), 'result exceeds the maximum nesting depth (64)')) { throw $error; }
}
restore_error_handler();
if (php_small_after_depth() !== [42]) { exit(11); }
echo 'ok';
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"ok");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Host exit remains authoritative over a return from native finally cleanup.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_exit_overrides_native_finally_returns() {
    compare(r#"<?php
#[Export] function php_exit_finally_return(array $value): int {
    try { echo $value['missing']; } finally { return 7; }
}
"#, r#"
set_error_handler(function () { exit(23); });
var_dump(php_exit_finally_return([]));
echo 'after';
"#);
}

/// Host exit remains authoritative over an exception from native finally cleanup.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_exit_overrides_native_finally_exceptions() {
    compare(r#"<?php
#[Export] function php_exit_finally_throw(array $value): void {
    try { echo $value['missing']; }
    finally { throw new RuntimeException('native'); }
}
"#, r#"
set_error_handler(function () { exit(23); });
try { php_exit_finally_throw([]); } catch (Throwable $error) { echo 'caught'; }
echo 'after';
"#);
}

/// Single-owner Zend reference wrappers left by foreach retain by-value array semantics.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn arrays_accept_unshared_reference_wrappers() {
    compare("<?php #[Export] function php_values(array $value): array { return $value; }", r#"
$value = [1, 2, 'key' => ['nested']];
foreach ($value as &$item) {} unset($item);
echo json_encode(php_values($value));
"#);
}

/// Native-created empty lists keep Zend's negative-key append cursor, including nested lists.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn native_empty_lists_preserve_negative_append_cursor() {
    compare(r#"<?php
#[Export] function php_empty_list(): array { return []; }
#[Export] function php_nested_empty_lists(): array { return ['a' => [], 'b' => [[]]]; }
"#, r#"
$plain = php_empty_list();
$plain[-5] = 1;
$plain[] = 2;
$nested = php_nested_empty_lists();
$nested['a'][-5] = 3;
$nested['a'][] = 4;
$nested['b'][0][-5] = 5;
$nested['b'][0][] = 6;
echo json_encode([$plain, $nested]);
"#);
}

/// Zend internal handlers reject extra arguments even though userland PHP functions ignore them.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn exported_functions_use_internal_exact_arity() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("arity", "<?php #[Export] function php_add(int $left, int $right): int { return $left + $right; }");
    let output = php(&sdk, &module).args(["-d", "display_errors=0", "-r", r#"
try { php_add(1, 2, 3); }
catch (ArgumentCountError $error) { echo get_class($error), ':', $error->getMessage(); }
"#]).output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout,
        b"ArgumentCountError:php_add() expects exactly 2 arguments, 3 given");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

/// Same-Fiber reentry cannot overwrite pending cast views in integer or float spill rings.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn nested_reentry_preserves_concat_spill_views() {
    compare_with_options(
        r#"<?php
#[Export] function php_cast_int_spill(array $value, int $prefix, string $fill): string {
    return (string)(strlen($fill . $fill) + $prefix) . $value['missing'];
}
#[Export] function php_cast_float_spill(array $value, float $prefix, string $fill): string {
    return (string)(strlen($fill . $fill) + $prefix) . $value['missing'];
}
#[Export] function php_wrap_int_spills(): string {
    $result = '';
    for ($i = 0; $i < 8; $i++) { $result = (string)$i; }
    return $result;
}
#[Export] function php_wrap_float_spills(): string {
    $result = '';
    for ($i = 0; $i < 4; $i++) { $result = (string)($i + 0.25); }
    return $result;
}
"#,
        r#"
$nested = 'php_wrap_int_spills';
set_error_handler(function () use (&$nested) {
    $nested();
    return true;
});
$fill = str_repeat('x', 262140);
var_dump(php_cast_int_spill([], 12345, $fill));
$nested = 'php_wrap_float_spills';
var_dump(php_cast_float_spill([], 12345.5, $fill));
"#,
        &["--heap-size=524288"],
    );
}

/// Comparison operands survive a warning handler that wraps the integer scratch ring.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn nested_reentry_preserves_comparison_spill_views() {
    compare_with_options(
        r#"<?php
#[Export] function php_compare_int_spill(array $value, int $prefix, string $fill): bool {
    return (string)(strlen($fill . $fill) + $prefix) ===
        (string)$value['missing'] . '536625';
}
#[Export] function php_wrap_comparison_spills(): string {
    $result = '';
    for ($i = 0; $i < 8; $i++) { $result = (string)$i; }
    return $result;
}
"#,
        r#"
set_error_handler(function () {
    php_wrap_comparison_spills();
    return true;
});
$fill = str_repeat('x', 262140);
var_dump(php_compare_int_spill([], 12345, $fill));
"#,
        &["--heap-size=524288"],
    );
}

/// Parameter errors report the caller's actual type and nullable spelling, including false/null.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn parameter_type_errors_match_php() {
    compare("<?php #[Export] function php_values(array $value): array { return $value; } #[Export] function php_nullable(?array $value): ?array { return $value; } #[Export] function php_false(false $value): false { return $value; } #[Export] function php_nullable_false(?false $value): ?false { return $value; } #[Export] function php_null(null $value): null { return $value; }", r#"
foreach (['php_values', 'php_nullable', 'php_false', 'php_nullable_false', 'php_null'] as $function) {
    foreach (['text', 7, true, new stdClass, null] as $value) {
        try { $function($value); echo 'accepted'; } catch (TypeError $e) {
            echo preg_replace('/, called in .*$/s', '', $e->getMessage()), "\n";
        }
    }
}
"#);
}

/// A host exit prevents native finally output from calling PHP again during termination.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn host_exit_suppresses_native_cleanup_output() {
    compare("<?php #[Export] function php_after_exit(array $value): void { try { echo $value['missing']; } finally { echo 'cleanup'; } }", r#"
set_error_handler(function () { exit(23); });
php_after_exit([]);
"#);
}

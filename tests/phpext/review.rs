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
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("review", source);
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

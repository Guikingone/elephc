//! Purpose:
//! Exercises native string scratch lifetimes across synchronous host callback reentry.
//!
//! Called from:
//! - The real PHP SDK integration test binary in `tests/phpext_tests.rs`.
//!
//! Key details:
//! - A live concat result crosses the former 64 KiB boundary before PHP reenters.

use super::*;

/// Nested exports must not overwrite an outer concat temporary while its output callback runs.
#[test]
#[ignore = "requires a real PHP 8.5 development SDK"]
fn compiled_php_reentry_preserves_large_concat_temporaries() {
    let project = Project::new();
    let sdk = project.sdk();
    let module = project.compile("large_reentry", r#"<?php
#[Export] function php_large_outer(string $value): string {
    return ($value . 'TAIL') . (print 'chunk');
}
#[Export] function php_large_inner(string $value): string { return $value . 'nested'; }
"#);
    let driver = r#"
$input = str_repeat('a', 70000);
ob_start(function ($text) {
    if ($text !== '') {
        $nested = str_repeat('b', 8000);
        if (php_large_inner($nested) !== $nested . 'nested') { exit(10); }
    }
    return '';
}, 1);
$result = php_large_outer($input);
ob_end_clean();
if ($result !== $input . 'TAIL1') { fwrite(STDERR, 'outer concat corrupted'); exit(11); }
echo 'ok';
"#;
    let result = php(&sdk, &module).args(["-r", driver]).output().unwrap();
    assert!(result.status.success(), "{:?}: {}", result.status, String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stdout, b"ok");
    assert!(result.stderr.is_empty());
}

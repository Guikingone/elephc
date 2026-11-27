//! Purpose:
//! Checks the guarded fast path of two-level reads `$local[$i][$k]` over a local `array<mixed>`.
//!
//! Called from:
//! - Focused `runtime_gc` codegen tests on the executable supported targets.
//!
//! Key details:
//! - Present elements are served without cloning the intermediate cell; every guard failure
//!   (missing outer or inner key, negative key, null or non-array element) must fall back to the
//!   ordinary chain with PHP's warnings, and copies must stay independent of later writes.

use crate::support::*;

/// Removes only this fixture's source, runtime cache and executable on every exit.
struct FixtureDir(std::path::PathBuf);

impl Drop for FixtureDir {
    /// Removes this test-owned directory after normal completion or assertion failure.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Fast and fallback nested reads match PHP output, warnings and heap ownership.
#[test]
fn test_nested_mixed_reads_match_php_on_fast_and_fallback_paths() {
    let source = r#"<?php
$b = [[1.5, 2.5], [3, "x"], [true, null], [[1, 2]], []];
$b[0][1] = 4.25;
$k = 1;
for ($i = 0; $i < 3; $i++) { var_dump($b[$i][0]); var_dump($b[$i][$k]); }
var_dump($b[3][0]);
var_dump($b[4][0]);
var_dump($b[99][0]);
var_dump($b[0][-1]);
$k = 5;
var_dump($b[1][$k]);
$copy = $b[3][0];
$b[3][0][0] = 100;
var_dump($copy[0], $b[3][0][0]);
$x = $b[0][0];
$b[0][0] = 0.5;
var_dump($x, $b[0][0]);
"#;
    let expected = "float(1.5)\nfloat(4.25)\nint(3)\nstring(1) \"x\"\nbool(true)\nNULL\narray(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\nNULL\nNULL\nNULL\nNULL\nint(1)\nint(100)\nfloat(1.5)\nfloat(0.5)\n";
    for mode in ["--ir-opt=off", "--ir-opt=on"] {
        let dir = FixtureDir(make_cli_test_dir("elephc_nested_mixed_fast_read"));
        let path = dir.0.join("main.php");
        fs::write(&path, source).expect("write nested read fixture");
        let compile = elephc_cli_command(&dir.0)
            .args([mode, "--heap-debug"])
            .arg(&path).output().expect("compile nested read fixture");
        assert!(compile.status.success(), "{mode}: {}", String::from_utf8_lossy(&compile.stderr));
        let output = run_binary(&dir.0.join("main"), &dir.0);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{mode}: {stderr}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{mode}: {stderr}");
        assert!(stderr.contains("HEAP DEBUG: leak summary: clean"), "{mode}: {stderr}");
        for warning in ["Undefined array key 0", "Undefined array key 99",
            "Trying to access array offset on null", "Undefined array key -1", "Undefined array key 5"]
        {
            assert!(stderr.contains(warning), "{mode}: missing {warning}: {stderr}");
        }
        assert_eq!(stderr.matches("Warning:").count(), 5, "{mode}: {stderr}");
    }
}

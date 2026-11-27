//! Purpose:
//! Checks that compound updates of a local array's nested elements do not copy the outer array.
//!
//! Called from:
//! - Focused `runtime_gc` codegen tests on the executable supported targets.
//!
//! Key details:
//! - `$b[$i][$k] op= v` pins the receiver for the diagnosed read; the local fetch-for-write
//!   must not see that pin as a second owner, or every update clones the outer array.
//! - Real sharing (an assigned copy, a by-value parameter) must still separate.

use crate::support::*;

/// Removes only this fixture's source, runtime cache and executable on every exit.
struct FixtureDir(std::path::PathBuf);

impl Drop for FixtureDir {
    /// Removes this test-owned directory after normal completion or assertion failure.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Nested compound updates keep PHP copy-on-write results without per-update outer clones.
#[test]
fn test_nested_local_compound_updates_do_not_clone_the_outer_array() {
    let source = r#"<?php
$b = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
for ($s = 0; $s < 500; $s++) {
    for ($i = 0; $i < 2; $i++) {
        $b[$i][1] += 0.5;
        $b[$i][2] -= 1;
    }
}
$copy = $b;
$copy[1][0] *= 10;
$b[0][0]++;
printf("%.1f %.1f %.1f | %.1f %.1f %.1f\n", $b[0][0], $b[0][1], $b[0][2], $b[1][0], $b[1][1], $b[1][2]);
printf("%.1f %.1f | %.1f\n", $copy[0][0], $copy[1][0], $b[1][0]);
function f(array $a): array { $a[1][2] .= ""; $a[0][0] += 1; return $a; }
$r = f($b);
printf("%.1f %.1f\n", $r[0][0], $b[0][0]);
"#;
    let expected = "2.0 252.0 -497.0 | 4.0 255.0 -494.0\n1.0 40.0 | 4.0\n3.0 2.0\n";
    for mode in ["--ir-opt=off", "--ir-opt=on"] {
        let dir = FixtureDir(make_cli_test_dir("elephc_nested_local_update_cow"));
        let path = dir.0.join("main.php");
        fs::write(&path, source).expect("write nested update fixture");
        let compile = elephc_cli_command(&dir.0)
            .args([mode, "--heap-debug", "--gc-stats"])
            .arg(&path).output().expect("compile nested update fixture");
        assert!(compile.status.success(), "{mode}: {}", String::from_utf8_lossy(&compile.stderr));
        let output = run_binary(&dir.0.join("main"), &dir.0);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{mode}: {stderr}");
        assert_eq!(output.stdout, expected.as_bytes(), "{mode}: {stderr}");
        assert!(stderr.contains("HEAP DEBUG: leak summary: clean"), "{mode}: {stderr}");
        let stats = stderr.lines().find_map(|line| line.strip_prefix("GC: allocs="))
            .expect("GC allocation and free counters");
        let (allocs, frees) = stats.split_once(" frees=").expect("GC counter format");
        let allocs: usize = allocs.parse().unwrap();
        let frees: usize = frees.parse().unwrap();
        assert_eq!(allocs, frees, "{mode}: {stderr}");
        if mode == "--ir-opt=on" {
            // 2000 updates: one outer clone each used to add 2000 array allocations (13053).
            assert!(allocs <= 12000, "outer array cloned per nested update: {allocs} allocations");
        }
    }
}

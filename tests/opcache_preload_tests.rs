//! Purpose:
//! End-to-end tests for `opcache.preload` / `opcache_get_status()['preload_statistics']`, the
//! compile-time preload verdict baked by `src/opcache_prelude.rs` and enforced by
//! `src/pipeline.rs`. Covers all four reference rows: the default (no `preload_statistics` key
//! at all), a resolvable preload file (the statistics block, with and without the
//! outside-the-manifest warning), an unresolvable path with the cache enabled (a COMPILE ERROR,
//! the AOT equivalent of reference PHP's startup fatal), and a set directive with the cache
//! disabled (nothing happens — not even path validation).
//!
//! Called from:
//! - `cargo test --test opcache_preload_tests` through Rust's test harness.
//!
//! Key details:
//! - Every expectation here is PINNED FROM REFERENCE PHP 8.5.6 (Homebrew, `Zend OPcache`
//!   loaded), reproduced with
//!   `php -d opcache.enable=1 -d opcache.enable_cli=1 -d opcache.preload=<file> -r
//!   'var_export(opcache_get_status());'`. The verified reference shape is: `preload_statistics`
//!   sits BETWEEN `opcache_statistics` and `scripts` in the top-level array, and its keys are
//!   `memory_consumption` (int), `functions` (list<string>), `classes` (list<string>),
//!   `scripts` (list<string>) IN THAT ORDER — with `functions`/`classes` OMITTED ENTIRELY when
//!   empty rather than reported as empty arrays. Nothing else is added to the top level.
//! - The reference startup fatal for a missing preload file was verified too:
//!   `PHP Fatal error:  Failed opening required '<path>' … in Unknown on line 0`, exit 1, before
//!   a single line of the script runs. Because elephc fixes its INI at build time, that becomes
//!   a compile error — and, like reference's fatal, it fires whether or not the program ever
//!   calls an OPcache function.
//! - The cache-disabled row was verified too: `-d opcache.enable_cli=0 -d opcache.preload=<missing>`
//!   runs cleanly and exits 0, and `opcache_get_status()` returns `false`. So elephc must not
//!   validate the path in that state either.
//! - Tests invoke the elephc CLI (CARGO_BIN_EXE_elephc) as a subprocess in an isolated temp dir,
//!   the same harness style as `opcache_restrict_api_tests` / `opcache_ini_tests`. Host-target
//!   only (macOS aarch64 local).
//! - The probe uses `count()` and `isset()` rather than `array_keys()` / `array_key_exists()`:
//!   only the former two narrow through elephc's `is_array()` guard on the `array|false` return
//!   today (the latter two are rejected with "argument must be array"). That is a pre-existing
//!   checker limitation unrelated to preloading; the probe works around it rather than pinning it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

/// The probe program. It declares one function, one class and one interface so the
/// `functions`/`classes` lists have something real to report, then dumps the discriminating
/// facts: the top-level key COUNT (9 without preloading, 10 with — the single added key), and,
/// when present, the block's own key count and every field.
const PROBE: &str = r#"<?php
function probe_helper() { return 1; }
class ProbeWidget {}
interface ProbeIface {}
$s = opcache_get_status();
if (is_array($s)) {
    echo 'count=', count($s), "\n";
    if (isset($s['preload_statistics'])) {
        $p = $s['preload_statistics'];
        echo 'pcount=', count($p), "\n";
        echo 'mem=', ($p['memory_consumption'] > 0 ? 'POSITIVE' : 'NONPOSITIVE'), "\n";
        echo 'fns=', implode(',', $p['functions']), "\n";
        echo 'cls=', implode(',', $p['classes']), "\n";
        echo 'scr=', implode(',', $p['scripts']), "\n";
    } else {
        echo "preload=absent\n";
    }
} else {
    echo "status=false\n";
}
"#;

/// Creates an isolated temp dir unique across parallel test threads/processes, returned
/// CANONICALIZED so a preload path built from it matches the spelling elephc resolves to (on
/// macOS `std::env::temp_dir()` lives under `/var/folders/...`, which resolves to
/// `/private/var/folders/...`).
fn make_test_dir(prefix: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("{}_{}_{:?}_{}", prefix, pid, tid, id));
    fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

/// Resolves the elephc CLI binary path (cargo env var, fallback next to the test binary).
fn elephc_bin() -> String {
    std::env::var("CARGO_BIN_EXE_elephc").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().expect("failed to resolve current test binary");
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.join("elephc").to_string_lossy().into_owned()
    })
}

/// Writes `PROBE` into `dir` and runs the compiler over it with the supplied `--ini`
/// assignments, returning `(success, stdout, stderr, executable path)` WITHOUT asserting — the
/// missing-preload row needs the failure, so the assertion belongs to each test.
fn try_compile(dir: &Path, stem: &str, ini: &[String]) -> (bool, String, String, PathBuf) {
    let php = dir.join(format!("{}.php", stem));
    fs::write(&php, PROBE).unwrap();
    let mut cmd = Command::new(elephc_bin());
    cmd.env("XDG_CACHE_HOME", dir.join("cache-root"));
    cmd.current_dir(dir);
    cmd.arg(&php);
    for assignment in ini {
        cmd.arg("--ini").arg(assignment);
    }
    let output = cmd.output().expect("failed to spawn elephc");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        dir.join(stem),
    )
}

/// Compiles `PROBE`, asserting success, and returns `(compiler stderr, executable path)`. The
/// compiler stderr is returned because the outside-the-manifest WARNING is emitted there.
fn compile(dir: &Path, stem: &str, ini: &[String]) -> (String, PathBuf) {
    let (ok, out, err, bin) = try_compile(dir, stem, ini);
    assert!(ok, "elephc compile failed for {ini:?}:\n{out}\n{err}");
    (elephc_diagnostics(&err), bin)
}

/// Keeps only elephc's own diagnostics from a compile's stderr.
///
/// Linking also surfaces the HOST linker's warnings, which are environmental rather than
/// anything elephc emitted: GNU `ld` reports the static-`getaddrinfo`/`gethostbyname` glibc
/// notes and the `.note.GNU-stack` deprecation, while Apple's linker stays silent. Those lines
/// start with `/usr/bin/ld:` or a `(.text.…)` section reference, so anchoring on elephc's own
/// line starts isolates its diagnostics — and still surfaces an UNEXPECTED elephc warning, which
/// an allow-list of known messages would have hidden.
///
/// elephc emits two prefixes: `Warning: …` for the INI-override diagnostics (`src/main.rs`) and
/// `warning: …` / `warning[line:col]: …` for compile warnings (`src/errors/report.rs`), the
/// latter being how the outside-the-manifest preload warning arrives.
fn elephc_diagnostics(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| {
            line.starts_with("Warning: ")
                || line.starts_with("warning:")
                || line.starts_with("warning[")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Runs a compiled executable and returns its stdout, asserting a clean exit.
fn run_binary(bin: &Path) -> String {
    let output = Command::new(bin).output().expect("failed to run compiled binary");
    assert!(
        output.status.success(),
        "compiled binary exited non-zero ({:?}):\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// THE BASELINE: with the cache enabled but `opcache.preload` at its default (empty), the status
/// array carries NO `preload_statistics` key and its top-level key count is the unchanged 9 —
/// the same figure `opcache_restrict_api_tests` pins as `ARRAY9`. Reference PHP agrees: an
/// unset (or explicitly empty) `opcache.preload` produces no such key.
#[test]
fn default_has_no_preload_statistics() {
    let dir = make_test_dir("opcache_preload_default");
    let (err, bin) = compile(&dir, "app", &["opcache.enable_cli=1".to_string()]);
    assert_eq!(err, "", "the default path must emit no diagnostics: {err:?}");
    assert_eq!(run_binary(&bin), "count=9\npreload=absent\n");
}

/// A preload file that IS the entry script (and therefore a member of the compile-time script
/// manifest) emits the statistics block SILENTLY: exactly one key is added to the top level, and
/// the block carries the four reference keys with the binary's real symbols and manifest paths.
#[test]
fn preloading_the_entry_file_emits_statistics_silently() {
    let dir = make_test_dir("opcache_preload_entry");
    let entry = dir.join("app.php");
    let (err, bin) = compile(
        &dir,
        "app",
        &[
            "opcache.enable_cli=1".to_string(),
            format!("opcache.preload={}", entry.display()),
        ],
    );
    assert_eq!(
        err, "",
        "a manifest-member preload file must warn about nothing: {err:?}"
    );

    let out = run_binary(&bin);
    // Exactly ONE key added to the top level (9 → 10): reference adds `preload_statistics` and
    // nothing else.
    assert!(out.contains("count=10\n"), "{out}");
    // The four reference keys: memory_consumption, functions, classes, scripts.
    assert!(out.contains("pcount=4\n"), "{out}");
    assert!(out.contains("mem=POSITIVE\n"), "{out}");
    // REAL user symbols, not a fabricated or empty interim. The interface lands under `classes`,
    // as reference PHP does.
    assert!(out.contains("fns=probe_helper\n"), "{out}");
    assert!(out.contains("cls=ProbeWidget,ProbeIface\n"), "{out}");
    // `scripts` is the compile-time manifest: the canonicalized entry file.
    assert!(
        out.contains(&format!("scr={}\n", entry.display())),
        "scripts must report the canonical manifest path:\n{out}"
    );
    // No compiler prelude leaked into the SYMBOL lists (checked on those two lines only: the
    // `scr=` line legitimately carries the temp dir's name, which contains "opcache_").
    for line in out.lines().filter(|l| l.starts_with("fns=") || l.starts_with("cls=")) {
        assert!(!line.contains("opcache_"), "prelude leaked into symbols: {line}");
        assert!(!line.contains("var_export"), "prelude leaked into symbols: {line}");
        assert!(!line.contains("__elephc"), "prelude leaked into symbols: {line}");
    }
}

/// A preload file that RESOLVES but is OUTSIDE the compile-time script manifest is a WARNING,
/// never an error: preloading a file this program never includes, requires or autoloads is a
/// legitimate configuration that must not break a build. The statistics are still emitted, from
/// the manifest. The membership test is made against the COMPLETE manifest (entry file +
/// statically-resolved includes + autoloaded files), which `src/pipeline.rs` only knows after
/// `autoload::run` — so this warning is emitted from the post-autoload site, not the injection
/// site (see `opcache_prelude::bake_manifest`).
#[test]
fn preload_outside_manifest_warns_but_still_compiles() {
    let dir = make_test_dir("opcache_preload_outside");
    let other = dir.join("other.php");
    fs::write(&other, "<?php\n").unwrap();
    let (err, bin) = compile(
        &dir,
        "app",
        &[
            "opcache.enable_cli=1".to_string(),
            format!("opcache.preload={}", other.display()),
        ],
    );

    assert!(
        err.contains("warning: opcache.preload:"),
        "an out-of-manifest preload file must warn: {err:?}"
    );
    assert!(
        err.contains(&other.display().to_string()),
        "the warning must name the resolved path: {err:?}"
    );
    assert!(
        err.contains("script manifest"),
        "the warning must say why: {err:?}"
    );
    // A warning only — the build produced a working binary with the statistics block.
    let out = run_binary(&bin);
    assert!(out.contains("count=10\n"), "{out}");
    assert!(out.contains("pcount=4\n"), "{out}");
}

/// CACHE ENABLED + UNRESOLVABLE PATH: a hard COMPILE ERROR naming the directive and the path.
/// This is the AOT equivalent of reference PHP's startup fatal `Failed opening required '<path>'`,
/// and like that fatal it does not depend on the program calling any OPcache function.
#[test]
fn missing_preload_file_fails_compilation() {
    let dir = make_test_dir("opcache_preload_missing");
    let missing = dir.join("nope.php");
    let (ok, _out, err, _bin) = try_compile(
        &dir,
        "app",
        &[
            "opcache.enable_cli=1".to_string(),
            format!("opcache.preload={}", missing.display()),
        ],
    );

    assert!(!ok, "a missing preload file must fail the build: {err:?}");
    assert!(err.contains("opcache.preload:"), "{err:?}");
    assert!(
        err.contains(&missing.display().to_string()),
        "the error must name the unresolvable path: {err:?}"
    );
    assert!(
        err.contains("failed opening required"),
        "the error must echo reference's fatal wording: {err:?}"
    );
    // The binary must not exist: nothing is shipped for an unresolvable preload.
    assert!(!dir.join("app").exists(), "no binary may be produced");
}

/// CACHE DISABLED: a set `opcache.preload` is ignored ENTIRELY. The default CLI binary has
/// `opcache.enable_cli=0`, so even a MISSING preload path compiles cleanly, runs, and reports
/// `opcache_get_status() === false` — exactly what reference PHP does with
/// `-d opcache.enable_cli=0 -d opcache.preload=<missing>` (exit 0, nothing preloaded).
#[test]
fn disabled_cache_ignores_preload_entirely() {
    let dir = make_test_dir("opcache_preload_disabled");
    let missing = dir.join("nope.php");
    let (err, bin) = compile(
        &dir,
        "app",
        &[format!("opcache.preload={}", missing.display())],
    );
    assert_eq!(
        err, "",
        "a disabled cache must neither validate the path nor warn: {err:?}"
    );
    assert_eq!(run_binary(&bin), "status=false\n");
}

/// An explicitly EMPTY `--ini opcache.preload=` is the same as the default: no key, no
/// diagnostics. Pinned to reference PHP, where `-d opcache.preload=` reports no
/// `preload_statistics`.
#[test]
fn explicitly_empty_preload_matches_the_default() {
    let dir = make_test_dir("opcache_preload_empty");
    let (err, bin) = compile(
        &dir,
        "app",
        &[
            "opcache.enable_cli=1".to_string(),
            "opcache.preload=".to_string(),
        ],
    );
    assert_eq!(err, "", "{err:?}");
    assert_eq!(run_binary(&bin), "count=9\npreload=absent\n");
}

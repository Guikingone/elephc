//! Purpose:
//! Checks owning raw-memory string copies through the public and internal builtin aliases.
//!
//! Called from:
//! - The runtime GC integration suite and the dedicated fileinfo CI gates.
//!
//! Key details:
//! - Fileinfo returns internal pointer-read copies through fallible PHP methods.
//! - Returned bytes must survive source-buffer release and retire without a second copy leak.

use crate::support::*;

/// Pointer reads transfer independent copies while pointer-write counts retire borrowed inputs.
#[test]
fn test_fileinfo_pointer_read_string_alias_owners() {
    let source = r#"<?php
extern function malloc(int $size): ptr;
extern function free(ptr $p): void;
function fileinfoPublicCopy(ptr $p): string|false { return ptr_read_string($p, 3); }
class FileinfoPointerCopy {
    public function direct(ptr $p): string|false { return __elephc_ptr_read_string($p, 3); }
    public function local(ptr $p): string { $s = __elephc_ptr_read_string($p, 3); return $s; }
}
$p = malloc(3); ptr_write_string($p, "A\0B");
$reader = new FileinfoPointerCopy();
for ($i = 0; $i < 40; $i++) {
    $written = ptr_write_string($p, "A\0B");
    $internalWritten = __elephc_ptr_write_string($p, "A\0B");
    $a = fileinfoPublicCopy($p);
    $b = $reader->direct($p);
    $c = $reader->local($p);
    __elephc_ptr_read_string($p, 3);
}
free($p);
echo strlen($a), ":", strlen($b), ":", strlen($c), ":", ord($b[1]), ":", ord($c[2]), ":", $written + $internalWritten, "\n";
unset($a, $b, $c, $reader);
"#;
    let (out, asm) = compile_and_run_with_heap_debug_and_asm(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3:3:3:0:66:6\n", "{}", out.stderr);
    if !out.stderr.contains("HEAP DEBUG: leak summary: clean") {
        let mut selected = false;
        for line in asm.lines().filter(|line| {
            if line.contains("@fn ") && (line.contains("symbol=main ") || line.ends_with("symbol=main") || line.contains("symbol=_main ") || line.ends_with("symbol=_main")) { selected = true; }
            let include = selected;
            if line.contains("@endfn ") { selected = false; }
            include
        }).take(1600) { eprintln!("pointer original: {line}"); }
        let declarations = source.split("\n$p = malloc(").next().unwrap();
        for (label, tail) in [
            ("writes", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); __elephc_ptr_write_string($p, "A\0B"); free($p);"#),
            ("public", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $a = fileinfoPublicCopy($p); free($p); unset($a);"#),
            ("method", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $reader = new FileinfoPointerCopy(); $a = $reader->direct($p); free($p); unset($a, $reader);"#),
            ("local", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $reader = new FileinfoPointerCopy(); $a = $reader->local($p); free($p); unset($a, $reader);"#),
            ("discard", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); __elephc_ptr_read_string($p, 3); free($p);"#),
            ("loop-local", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $reader = new FileinfoPointerCopy(); for ($i = 0; $i < 40; $i++) { $c = $reader->local($p); } free($p); echo strlen($c); unset($c, $reader);"#),
            ("loop-direct", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); for ($i = 0; $i < 40; $i++) { $c = __elephc_ptr_read_string($p, 3); } free($p); echo strlen($c); unset($c);"#),
            ("loop-boxed", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $reader = new FileinfoPointerCopy(); for ($i = 0; $i < 40; $i++) { $b = $reader->direct($p); } free($p); echo strlen($b); unset($b, $reader);"#),
            ("raw-index", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $reader = new FileinfoPointerCopy(); $c = $reader->local($p); free($p); echo ord($c[2]); unset($c, $reader);"#),
            ("boxed-index", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $a = fileinfoPublicCopy($p); free($p); echo ord($a[1]); unset($a);"#),
            ("boxed-length", r#"$p = malloc(3); ptr_write_string($p, "A\0B"); $a = fileinfoPublicCopy($p); free($p); echo strlen($a); unset($a);"#),
        ] {
            let (control, asm) = compile_and_run_with_heap_debug_and_asm(&format!("{declarations}\n{tail}"));
            eprintln!("pointer ownership control {label}: {}\n{}", control.stdout, control.stderr);
            if !control.stderr.contains("HEAP DEBUG: leak summary: clean") {
                let mut selected = false;
                for line in asm.lines().filter(|line| {
                    if line.contains("@fn ") && (line.contains("symbol=main ") || line.ends_with("symbol=main") || line.contains("symbol=_main ") || line.ends_with("symbol=_main")) { selected = true; }
                    let include = selected;
                    if line.contains("@endfn ") { selected = false; }
                    include
                }).take(900) {
                    eprintln!("{line}");
                }
            }
        }
    }
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

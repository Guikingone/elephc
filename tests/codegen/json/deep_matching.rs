//! Purpose:
//! Verifies that deeply nested JSON decodes identically when element scans skip matched containers.
//!
//! Called from:
//! - The JSON codegen integration suite.
//!
//! Key details:
//! - Documents are long and deep enough for the x86_64 bracket-match pre-pass to be built.
//! - Brackets and escaped quotes inside strings, stray or missing closes, trailing commas and the
//!   depth limit must keep PHP's results, and the heap must stay clean in both optimizer modes.

use super::*;

/// Deep arrays and objects, bracket-bearing strings and malformed nesting keep PHP's results.
#[test]
fn test_json_deep_nesting_with_bracket_skips_matches_php() {
    let source = r#"<?php
function deep(int $n, string $leaf): string { return str_repeat("[", $n) . $leaf . str_repeat("]", $n); }
$pad = str_repeat(" ", 256 + $argc);
$docs = [deep(40, "1"), deep(24, '"a]]}{[\\"x"'),
    deep(20, '{"k":' . deep(20, '"]"') . ',"z":[1,2,{"q":"}"}]}'),
    '[' . str_repeat('{"a":[', 30) . '7' . str_repeat(']}', 30) . ', "tail \\\\", [[[]]], {}]',
    str_repeat('[{"x":', 20) . '1' . str_repeat('}]', 20),
    deep(40, "1") . "]", "[" . deep(40, "1"), deep(30, "1,"), deep(600, "1")];
foreach ($docs as $i => $doc) {
    $value = json_decode($doc . $pad, true);
    echo $i, ':', json_last_error(), ':', json_encode($value), "\n";
    $object = json_decode($pad . $doc . $pad);
    echo $i, 'o:', json_last_error(), ':', json_encode($object), "\n";
}
"#;
    let expected = r#"0:0:[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
0o:0:[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
1:0:[[[[[[[[[[[[[[[[[[[[[[[["a]]}{[\"x"]]]]]]]]]]]]]]]]]]]]]]]]
1o:0:[[[[[[[[[[[[[[[[[[[[[[[["a]]}{[\"x"]]]]]]]]]]]]]]]]]]]]]]]]
2:0:[[[[[[[[[[[[[[[[[[[[{"k":[[[[[[[[[[[[[[[[[[[["]"]]]]]]]]]]]]]]]]]]]],"z":[1,2,{"q":"}"}]}]]]]]]]]]]]]]]]]]]]]
2o:0:[[[[[[[[[[[[[[[[[[[[{"k":[[[[[[[[[[[[[[[[[[[["]"]]]]]]]]]]]]]]]]]]]],"z":[1,2,{"q":"}"}]}]]]]]]]]]]]]]]]]]]]]
3:0:[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[7]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]},"tail \\",[[[]]],[]]
3o:0:[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[{"a":[7]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]},"tail \\",[[[]]],{}]
4:0:[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":1}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]
4o:0:[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":[{"x":1}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]}]
5:4:null
5o:4:null
6:4:null
6o:4:null
7:4:null
7o:4:null
8:1:null
8o:1:null
"#;
    for off in [false, true] {
        let run = || {
            assert_eq!(compile_and_run(source), expected);
            let out = compile_and_run_with_heap_debug(source);
            assert!(out.success, "{}", out.stderr);
            assert_eq!(out.stdout, expected, "{}", out.stderr);
            assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
        };
        if off { without_ir_opt(run); } else { run(); }
    }
}

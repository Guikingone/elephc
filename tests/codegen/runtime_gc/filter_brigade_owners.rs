//! Purpose:
//! Checks native bucket-brigade ownership across successful and throwing PHP callbacks.
//!
//! Called from:
//! - The focused fileinfo ownership gates and the runtime GC integration suite.
//!
//! Key details:
//! - Raw and boxed callback parameters use their declared ABI on every executable target.
//! - Expanded output exceeds the former shared 64 KiB buffer and remains independently owned.

use crate::support::*;

/// Native brigade graphs retire on return and throw, including retained property getters.
#[test]
fn test_fileinfo_user_filter_brigade_owners() {
    let (out, _) = compile_and_run_with_heap_debug_and_asm(r#"<?php
class FileinfoRawBrigade extends php_user_filter {
    public function filter($in, $out, &$consumed, bool $closing): int {
        while ($bucket = stream_bucket_make_writeable($in)) {
            if (!is_bool($closing)) { echo "wrong closing flag\n"; }
            $consumed += $bucket->datalen;
            $bucket->data = str_repeat("x", 70000);
            $bucket->datalen = strlen($bucket->data);
            stream_bucket_append($out, $bucket);
        }
        return PSFS_PASS_ON;
    }
}
class FileinfoBoxedBrigade extends php_user_filter {
    public function filter(mixed $in, mixed $out, mixed &$consumed, mixed $closing): int {
        while ($bucket = stream_bucket_make_writeable($in)) {
            if (!is_bool($closing)) { echo "wrong boxed closing flag\n"; }
            $consumed += $bucket->datalen;
            $bucket->data = str_repeat("y", 70001);
            $bucket->datalen = strlen($bucket->data);
            stream_bucket_append($out, $bucket);
        }
        return PSFS_PASS_ON;
    }
}
class FileinfoThrowingBrigade extends php_user_filter {
    public function filter($in, $out, &$consumed, bool $closing): int {
        $hadData = false;
        while ($bucket = stream_bucket_make_writeable($in)) {
            $hadData = true;
            $consumed += $bucket->datalen;
            stream_bucket_append($out, $bucket);
        }
        if ($hadData) { throw new Error("filter error"); }
        return PSFS_PASS_ON;
    }
}
stream_filter_register("fileinfo.brigade.raw", "FileinfoRawBrigade");
stream_filter_register("fileinfo.brigade.boxed", "FileinfoBoxedBrigade");
stream_filter_register("fileinfo.brigade.throw", "FileinfoThrowingBrigade");
for ($i = 0; $i < 4; $i++) {
    foreach (["raw", "boxed"] as $name) {
        $stream = fopen("php://temp", "w+b"); fwrite($stream, "seed"); rewind($stream);
        $filter = stream_filter_append($stream, "fileinfo.brigade." . $name, STREAM_FILTER_READ);
        $data = $name === "raw" ? stream_get_contents($stream) : stream_get_contents($stream, 100000);
        echo strlen($data), ":", substr($data, 0, 1), "\n";
        fclose($stream); unset($data, $filter);
    }
    $stream = fopen("php://temp", "w+b"); fwrite($stream, "seed"); rewind($stream);
    $filter = stream_filter_append($stream, "fileinfo.brigade.throw", STREAM_FILTER_READ);
    try { stream_get_contents($stream, 100000); } catch (Error $e) { echo $e->getMessage(), "\n"; }
    fclose($stream); unset($filter);
}
unset($stream, $e, $name);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "70000:x\n70001:y\nfilter error\n".repeat(4), "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

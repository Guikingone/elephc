//! Purpose:
//! PHP's `gz*` stream surface — `gzopen`, `gzread`, `gzgets`, `gzwrite`, `gzseek`, `gzfile`,
//! `readgzfile` and their siblings — implemented in elephc-PHP on top of the `compress.zlib://`
//! wrapper the stream functions already serve. All fourteen were absent: a program calling
//! `gzopen($f, "r")` failed with "Undefined function".
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, after include
//!   resolution and before name resolution.
//!
//! Key details:
//! - WHY THIS IS AN EQUIVALENCE AND NOT AN APPROXIMATION. php-src implements `gzopen` as a stream
//!   open on the zlib wrapper, so the whole family IS the plain stream API over that URL. That was
//!   MEASURED rather than read: all fifteen pairs below — `gzread`/`fread`, `gzgets`/`fgets`,
//!   `gzgetc`/`fgetc`, `gzeof`/`feof`, `gzseek`/`fseek` in both `SEEK_SET` and `SEEK_CUR` forms,
//!   `gzrewind`/`rewind`, `gztell`/`ftell`, `gzpassthru`/`fpassthru`, `gzfile`/`file`,
//!   `readgzfile`/`readfile`, `gzwrite`/`fwrite`, and a failed open — answer identically under
//!   `php -n` 8.5.6. elephc's own answer to the wrapper spelling was then measured against php on
//!   the same fifteen: 15 of 15 match, so the prelude inherits nothing wrong.
//! - THE MODE PASSES THROUGH UNTOUCHED. `gzopen($f, "wb9")` carries a compression LEVEL and
//!   STRATEGY that plain `fopen` never sees; `fopen("compress.zlib://…", "wb9")` and `"wb1f"` were
//!   measured to answer exactly what `gzopen` does with the same mode, so no parsing is needed
//!   here.
//! - `$stream` IS DECLARED `mixed`, and has to be. An UNTYPED parameter infers as `int` here, and
//!   the stream builtins refuse that with "expects resource, got int" — raised in the bodies of
//!   the functions the program never calls, which are checked all the same. `mixed` is what
//!   `ensure_stream_resource` accepts for a value it cannot narrow, and is what the directory
//!   prelude already passes to `readdir()` for the same reason.
//! - `?int $length = null` IS BRANCHED ON, not forwarded. php's `gzgets($h, null)` reads a whole
//!   line, and forwarding the null would make it a length. The branch is what keeps the two
//!   spellings apart at the one place that knows they differ.
//! - `$use_include_path` IS FORWARDED, not accepted and ignored. php honours it on all three of
//!   `gzopen`, `gzfile` and `readgzfile`, and dropping it would also leave the parameter unread —
//!   which elephc reports as `Unused variable: $use_include_path` and refuses the compile over,
//!   so the prelude cannot carry a decorative parameter even if that were acceptable.
//! - PAY-FOR-USE. Injected only when `detect::program_uses_gz` finds a reference, so a program
//!   that never touches a gzip stream carries none of this.
//! - A program that declares its OWN function of one of these names suppresses injection. Not for
//!   php-fidelity — php FATALS with "Cannot redeclare gzopen()" there — but because elephc emits
//!   BOTH declarations and the ASSEMBLER stops the build: MEASURED, a user `function foo()` plus
//!   an `if (!function_exists("foo"))` redeclaration fails with
//!   `error: symbol '_fn_foo' is already defined`, where php answers the first one. Suppressing
//!   keeps such a program compiling instead of ending in an assembler diagnostic.
//!   The `function_exists`-guarded polyfill shape still does not compile, for a SEPARATE and
//!   pre-existing reason — a conditionally declared function is not registered, so the call
//!   reports "Undefined function" — and that is unchanged by this prelude either way.
//! - MEASURED DIVERGENCE, and the price of implementing a builtin in PHP: a failed open warns in
//!   the words of the call this is BUILT from, at the line that call sits on. `gzopen("nope.gz",
//!   "r")` on line 1 answers `Warning: fopen(nope.gz): Failed to open stream: No such file or
//!   directory ... on line 4` where php says `Warning: gzopen(nope.gz): ... on line 1`. The VALUE
//!   — `false` — matches, and warning with a wrong name and line was preferred to the alternative
//!   of suppressing the inner warning and going SILENT, which is the failure mode with no
//!   diagnostic at all.
//! - A SECOND divergence lives one level down and is not this prelude's: elephc's own
//!   `fopen("compress.zlib://nope.gz", "r")` warns `fopen(nope.gz): ... No such file or directory`
//!   where php warns `fopen(compress.zlib://nope.gz): ... operation failed`, naming the URL rather
//!   than the underlying path. Fixing it there fixes it here.

mod detect;

/// The elephc-PHP gzip-stream prelude.
///
/// Every body is one existing builtin call on a `compress.zlib://` URL, so the whole surface
/// compiles through the ordinary function pipeline with NO new assembly and both architectures get
/// it at once.
pub(crate) const GZ_PRELUDE_SRC: &str = r#"<?php

function gzopen(string $filename, string $mode, int $use_include_path = 0) {
    return fopen('compress.zlib://' . $filename, $mode, $use_include_path !== 0);
}

function gzclose(mixed $stream): bool {
    return fclose($stream);
}

function gzeof(mixed $stream): bool {
    return feof($stream);
}

function gzgetc(mixed $stream): string|false {
    return fgetc($stream);
}

function gzgets(mixed $stream, ?int $length = null): string|false {
    if ($length === null) {
        return fgets($stream);
    }
    return fgets($stream, $length);
}

function gzread(mixed $stream, int $length): string|false {
    return fread($stream, $length);
}

function gzwrite(mixed $stream, string $data, ?int $length = null): int|false {
    if ($length === null) {
        return fwrite($stream, $data);
    }
    return fwrite($stream, $data, $length);
}

function gzputs(mixed $stream, string $data, ?int $length = null): int|false {
    return gzwrite($stream, $data, $length);
}

function gzpassthru(mixed $stream): int {
    return fpassthru($stream);
}

function gzrewind(mixed $stream): bool {
    return rewind($stream);
}

function gzseek(mixed $stream, int $offset, int $whence = SEEK_SET): int {
    return fseek($stream, $offset, $whence);
}

function gztell(mixed $stream): int|false {
    return ftell($stream);
}

function gzfile(string $filename, int $use_include_path = 0): array|false {
    return file('compress.zlib://' . $filename, $use_include_path !== 0 ? FILE_USE_INCLUDE_PATH : 0);
}

function readgzfile(string $filename, int $use_include_path = 0): int|false {
    return readfile('compress.zlib://' . $filename, $use_include_path !== 0);
}
"#;

/// Injects the gzip-stream prelude when the program references one of its functions, leaving every
/// other program untouched.
///
/// The prelude carries only declarations, so prepending it is order-independent — PHP hoists them.
pub fn inject_if_used(program: crate::parser::ast::Program) -> crate::parser::ast::Program {
    if !detect::program_uses_gz(&program) || detect::program_declares_gz(&program) {
        return program;
    }
    let tokens = crate::lexer::tokenize(GZ_PRELUDE_SRC).expect("gz prelude must tokenize");
    let mut combined = crate::parser::parse_internal(&tokens).expect("gz prelude must parse");
    combined.extend(program);
    combined
}

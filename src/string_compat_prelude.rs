//! Purpose:
//! Injects PHP string builtins that elephc recognizes but has no (or no PHP-correct) EIR lowering
//! for, written in elephc-PHP on top of builtins that DO lower. Each is injected only when used.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the same
//!   pipeline stage as `crate::mb_convert_encoding_prelude` (after `autoload::run` and the
//!   conditional-function hoist, before the type checker collects functions), so PSR-4 autoloaded
//!   usage is detected and the declarations are present before checking.
//!
//! Key details:
//! - Most of these names were catalog-visible with NO lowering at all: the checker accepted the
//!   call and codegen then answered `unsupported EIR backend feature: builtin call <name>`.
//! - Written in PHP rather than as per-target runtime assembly, because each reduces exactly to
//!   builtins elephc already lowers. Every reduction below was verified byte-identical to
//!   `php -n` (PHP 8.5) BEFORE being written, including the raw-byte-difference convention
//!   (`strncmp("hello","help",4)` is `-4`, not `-1`).
//! - `catalog::is_prelude_overridable_builtin` keeps each real BUILTIN name in the catalog (so
//!   `function_exists()` still reports a real PHP function) while allowing these declarations to
//!   supply the bodies. Reserved `__elephc_*` aliases are NOT builtin names and are excluded.
//! - A program that declares its own global function of the same name wins: that entry is skipped.

use crate::parser::ast::{Program, Stmt, StmtKind};

/// Reserved function name the three-argument `strpos()` form is name-resolved to.
pub(crate) const STRPOS_OFFSET_FUNCTION_NAME: &str = "__elephc_strpos_offset";

/// Reserved function name the three-argument `strrpos()` form is name-resolved to.
pub(crate) const STRRPOS_OFFSET_FUNCTION_NAME: &str = "__elephc_strrpos_offset";

/// One prelude-supplied function: the global name it defines and its elephc-PHP source.
struct StringCompatEntry {
    /// The global PHP function name this entry declares.
    name: &'static str,
    /// Whether `name` is a real PHP builtin (so the redeclare-builtin guard must allow it) rather
    /// than a reserved `__elephc_*` alias.
    overridable_builtin: bool,
    /// Standalone elephc-PHP source declaring exactly that function.
    source: &'static str,
}

/// Every function supplied as elephc-PHP, injected individually on demand.
///
/// Order matters: `inject_if_used` walks this list in REVERSE and prepends, so an entry declared
/// EARLIER here is considered LATER and can therefore see references made by the bodies of entries
/// declared after it. `__elephc_strpos_offset` is first because `stripos` calls it.
const ENTRIES: &[StringCompatEntry] = &[
    StringCompatEntry {
        name: STRPOS_OFFSET_FUNCTION_NAME,
        overridable_builtin: false,
        // The three-argument `strpos()` had two PHP-compliance defects, both silent. The lowering
        // applied the offset as `ptr += offset; len -= offset`, so a NEGATIVE offset walked the
        // haystack pointer BEFORE the string (an out-of-bounds read) and answered as though the
        // offset were relative to the start: `strpos("abcabc", "a", -3)` returned 0 where PHP
        // returns 3, and `strpos("hello", "l", -1)` returned 2 where PHP returns false. An
        // out-of-range offset returned `false` where PHP 8 raises a ValueError.
        //
        // Normalizing here and delegating the search to the TWO-argument native `strpos` keeps the
        // fast path untouched: only a call that actually passes an offset goes through this.
        source: r#"<?php
function __elephc_strpos_offset(string $haystack, string $needle, int $offset): int|false {
    $length = strlen($haystack);
    if ($offset < 0) {
        $offset = $length + $offset;
    }
    if ($offset < 0 || $offset > $length) {
        throw new \ValueError('strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)');
    }
    $found = strpos(substr($haystack, $offset), $needle);
    if ($found === false) {
        return false;
    }
    return $found + $offset;
}
"#,
    },
    StringCompatEntry {
        name: STRRPOS_OFFSET_FUNCTION_NAME,
        overridable_builtin: false,
        // `strrpos()`'s offset means something DIFFERENT from `strpos()`'s, and the native lowering
        // (which shares `strpos`'s haystack-adjusting code) got the negative case wrong the same
        // silent way: `strrpos("hello", "l", -3)` returned 3 where PHP returns 2, and
        // `strrpos("hello", "l", -5)` returned 3 where PHP returns false.
        //
        // PHP: a POSITIVE offset requires the match to START at or after it. A NEGATIVE offset
        // requires the match to START at or before `strlen + offset` — so the window ends
        // `strlen($needle)` bytes further along, which is what the `substr` below expresses.
        source: r#"<?php
function __elephc_strrpos_offset(string $haystack, string $needle, int $offset): int|false {
    $length = strlen($haystack);
    if ($offset > $length || $offset < -$length) {
        throw new \ValueError('strrpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)');
    }
    if ($offset < 0) {
        return strrpos(substr($haystack, 0, $length + $offset + strlen($needle)), $needle);
    }
    $found = strrpos(substr($haystack, $offset), $needle);
    if ($found === false) {
        return false;
    }
    return $found + $offset;
}
"#,
    },
    StringCompatEntry {
        name: "strripos",
        overridable_builtin: true,
        // Case-insensitive `strrpos`: ASCII folding preserves byte length, so positions map 1:1.
        source: r#"<?php
function strripos(string $haystack, string $needle, int $offset = 0): int|false {
    return __elephc_strrpos_offset(strtolower($haystack), strtolower($needle), $offset);
}
"#,
    },
    StringCompatEntry {
        name: "strncmp",
        overridable_builtin: true,
        // Comparing at most `$length` leading bytes is exactly comparing the two strings truncated
        // to that length.
        source: r#"<?php
function strncmp(string $string1, string $string2, int $length): int {
    if ($length < 0) {
        throw new \ValueError('strncmp(): Argument #3 ($length) must be greater than or equal to 0');
    }
    if ($length === 0) {
        return 0;
    }
    return strcmp(substr($string1, 0, $length), substr($string2, 0, $length));
}
"#,
    },
    StringCompatEntry {
        name: "strncasecmp",
        overridable_builtin: true,
        // PHP 8's case-insensitive comparisons fold ASCII only, and `strtolower` is likewise
        // ASCII-only, so folding both truncations gives the same signed byte difference.
        source: r#"<?php
function strncasecmp(string $string1, string $string2, int $length): int {
    if ($length < 0) {
        throw new \ValueError('strncasecmp(): Argument #3 ($length) must be greater than or equal to 0');
    }
    if ($length === 0) {
        return 0;
    }
    return strcmp(
        strtolower(substr($string1, 0, $length)),
        strtolower(substr($string2, 0, $length))
    );
}
"#,
    },
    StringCompatEntry {
        name: "stripos",
        overridable_builtin: true,
        // ASCII case folding preserves byte length, so positions in the folded haystack map 1:1
        // onto the original. The offset-normalizing helper is called directly rather than
        // `strpos(…, …, $offset)`: prelude bodies are injected AFTER name resolution, so a call
        // written here would keep the native three-argument lowering and its defects.
        source: r#"<?php
function stripos(string $haystack, string $needle, int $offset = 0): int|false {
    return __elephc_strpos_offset(strtolower($haystack), strtolower($needle), $offset);
}
"#,
    },
];

/// Prepends every prelude entry the program references and does not declare itself.
pub fn inject_if_used(program: Program) -> Program {
    let mut program = program;
    for entry in ENTRIES.iter().rev() {
        program = inject_entry(program, entry);
    }
    program
}

/// Prepends one entry's declaration when the program references its name and does not declare a
/// global function of that name itself.
fn inject_entry(program: Program, entry: &StringCompatEntry) -> Program {
    if !crate::ast_usage::collect(&program).references(entry.name)
        || program_declares(&program, entry.name)
    {
        return program;
    }
    let tokens = crate::lexer::tokenize(entry.source)
        .unwrap_or_else(|error| panic!("{} prelude must tokenize: {error}", entry.name));
    let mut combined = crate::parser::parse(&tokens)
        .unwrap_or_else(|error| panic!("{} prelude must parse: {error}", entry.name));
    combined.extend(program);
    combined
}

/// Returns whether the program already declares its own global function called `name`.
fn program_declares(program: &[Stmt], name: &str) -> bool {
    program.iter().any(|stmt| stmt_declares(stmt, name))
}

/// Returns whether one statement declares a function called `name`, recursing only into the block
/// forms that can host a hoisted function declaration.
fn stmt_declares(stmt: &Stmt, name: &str) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { name: declared, .. } => declared.eq_ignore_ascii_case(name),
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body.iter().any(|stmt| stmt_declares(stmt, name)),
        _ => false,
    }
}

/// Returns whether `canonical` is a real builtin supplied by this prelude, so the
/// redeclare-builtin guard treats it as overridable. Mirrors `ENTRIES` so the two cannot drift.
pub(crate) fn supplies(canonical: &str) -> bool {
    ENTRIES
        .iter()
        .any(|entry| entry.overridable_builtin && entry.name.eq_ignore_ascii_case(canonical))
}

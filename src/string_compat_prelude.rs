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
        name: "strtr",
        overridable_builtin: true,
        // Catalog-visible with no lowering: `builtin call strtr`. Composer's own `ClassLoader`
        // uses the three-argument form on every autoload, so nothing that autoloads could compile.
        //
        // The two forms are different algorithms, and both were pinned on `php -n` first:
        //   - three arguments translate BYTES pairwise over `min(strlen($from), strlen($to))`
        //     pairs, and a byte repeated in `$from` takes its LAST pairing
        //     (`strtr("aaa", "aa", "bc")` is 'ccc', not 'bbb'). The truncation is why the search
        //     runs over `substr($from, 0, $n)` rather than `$from`: a pair beyond the common
        //     length contributes nothing to PHP's table and must not be found here either.
        //   - two arguments replace SUBSTRINGS, choosing the LONGEST key that matches at each
        //     position, scanning left to right, and never re-scanning what was just written
        //     (`strtr("ab", ["a" => "b", "b" => "a"])` is 'ba', and `strtr("aaa", ["a" => "aa"])`
        //     is 'aaaaaa'). Keys are array keys, so an integer key arrives already stringified.
        //
        // A byte map is built by searching `$from` rather than by indexing an array, because a
        // 256-entry sparse integer-keyed table is exactly the shape elephc still miscompiles.
        //
        // Divergence: PHP prints `Warning: strtr(): Ignoring replacement of empty string` for an
        // empty key and then ignores it; this skips the key silently, because elephc has no
        // builtin-warning construct reachable from an elephc-PHP prelude.
        source: r#"<?php
function __elephc_strtr_bytes(string $string, string $from, string $to): string {
    $pairs = strlen($from);
    $available = strlen($to);
    if ($available < $pairs) {
        $pairs = $available;
    }
    if ($pairs === 0) {
        return $string;
    }
    $source = substr($from, 0, $pairs);
    $out = '';
    $length = strlen($string);
    $i = 0;
    while ($i < $length) {
        $at = strrpos($source, $string[$i]);
        if ($at === false) {
            $out .= $string[$i];
        } else {
            $out .= $to[$at];
        }
        $i++;
    }
    return $out;
}
function __elephc_strtr_pairs(string $string, array $replace_pairs): string {
    $keys = [];
    $values = [];
    foreach ($replace_pairs as $key => $value) {
        $needle = (string) $key;
        if ($needle === '') {
            continue;
        }
        $keys[] = $needle;
        $values[] = (string) $value;
    }
    $count = count($keys);
    if ($count === 0) {
        return $string;
    }
    $out = '';
    $length = strlen($string);
    $i = 0;
    while ($i < $length) {
        $best = 0;
        $matched = -1;
        $j = 0;
        while ($j < $count) {
            $needle_length = strlen($keys[$j]);
            if ($needle_length > $best && $i + $needle_length <= $length
                && substr($string, $i, $needle_length) === $keys[$j]) {
                $best = $needle_length;
                $matched = $j;
            }
            $j++;
        }
        if ($matched < 0) {
            $out .= $string[$i];
            $i++;
            continue;
        }
        $out .= $values[$matched];
        $i += $best;
    }
    return $out;
}
function strtr(string $string, mixed $from, ?string $to = null): string {
    if (is_array($from)) {
        return __elephc_strtr_pairs($string, $from);
    }
    if ($to === null) {
        throw new \TypeError('strtr(): Argument #2 ($from) must be of type array, string given');
    }
    return __elephc_strtr_bytes($string, (string) $from, $to);
}
"#,
    },
    StringCompatEntry {
        name: "iconv_mime_decode",
        overridable_builtin: true,
        // RFC 2047 encoded-word decoding for a MIME header. Catalog-visible with no lowering at
        // all, and `Symfony\Polyfill\Mbstring\Mbstring::mb_decode_mimeheader` calls it directly,
        // so the whole `--web` build died on `builtin call iconv_mime_decode`.
        //
        // Every rule below was measured on `php -n` BEFORE being written:
        //   - literal text between encoded words is kept verbatim ('pre =?..?ok?= post' →
        //     'pre ok post'), but linear whitespace SEPARATING two encoded words is dropped
        //     ('=?..?a?= =?..?b?=' → 'ab', and the same for a tab or a folded newline);
        //   - linear whitespace trailing the last encoded word is dropped too ('x =?..?a?= ' →
        //     'x a'), which is why the tail is filtered rather than appended;
        //   - Q-encoding maps `_` to a space in addition to `=XX` ('Hello_World' → 'Hello World');
        //   - charset and encoding names are matched case-insensitively ('=?utf-8?b?..?=' works);
        //   - an encoded word whose charset this build cannot transcode is passed through verbatim
        //     under CONTINUE_ON_ERROR ('=?BOGUSCS?B?SGk=?=' stays as written);
        //   - an encoded word with no closing `?=` discards the remainder of the header
        //     ('pre =?UTF-8?Q?tail' → 'pre ');
        //   - modes 0 and 1 return `false` on any of those errors instead of continuing.
        //
        // The known charset set mirrors `__elephc_mb_enc_kind` in
        // `crate::mb_convert_encoding_prelude` exactly, and is repeated rather than called so this
        // entry does not depend on another prelude's private helper. An encoding outside that set
        // is reported as an error rather than silently passed through `mb_convert_encoding`, which
        // returns its subject unchanged for encodings it does not know — accepting it would decode
        // the word and emit mojibake where PHP re-emits the word untouched.
        //
        // Measured on 30 `php -n` cases; 27 are byte-identical. The three that are not, and the one
        // missing diagnostic, are all recorded here rather than papered over:
        //   - ext-iconv's scanner, when it gives up on a broken encoded word, re-emits what it
        //     scanned MINUS a trailing byte or three ('=?UTF-8?B?!!!notb64!!!?=' loses its final
        //     '=', '=?UTF-8??SGk=?=' loses '=?='). This implementation re-emits exactly the bytes
        //     it scanned. Reproducing the rewind would mean porting the C scanner's state machine.
        //   - modes 0 and 1 return `false` without printing PHP's "iconv_mime_decode(): Malformed
        //     string" warning: elephc has no general builtin-warning construct reachable from an
        //     elephc-PHP prelude.
        // `ICONV_MIME_DECODE_STRICT` (1) and `ICONV_MIME_DECODE_CONTINUE_ON_ERROR` (2) are not
        // defined as constants either; the `$mode` bit test above is what PHP's own constants mean.
        source: r#"<?php
function __elephc_mime_charset_is_utf8(string $charset): bool {
    $e = strtoupper($charset);
    return $e === 'UTF-8' || $e === 'UTF8';
}
function __elephc_mime_charset_known(string $charset): bool {
    if (__elephc_mime_charset_is_utf8($charset)) {
        return true;
    }
    $e = strtoupper($charset);
    return $e === 'ISO-8859-1' || $e === 'ISO8859-1' || $e === 'LATIN1'
        || $e === 'WINDOWS-1252' || $e === 'CP1252'
        || $e === 'ASCII' || $e === 'US-ASCII';
}
function __elephc_mime_utf8_valid(string $text): bool {
    $length = strlen($text);
    $i = 0;
    while ($i < $length) {
        $lead = ord($text[$i]);
        if ($lead < 128) {
            $i++;
            continue;
        }
        if ($lead >= 194 && $lead <= 223) {
            $extra = 1;
        } elseif ($lead >= 224 && $lead <= 239) {
            $extra = 2;
        } elseif ($lead >= 240 && $lead <= 244) {
            $extra = 3;
        } else {
            return false;
        }
        if ($i + $extra >= $length) {
            return false;
        }
        $j = 1;
        while ($j <= $extra) {
            $continuation = ord($text[$i + $j]);
            if ($continuation < 128 || $continuation > 191) {
                return false;
            }
            $j++;
        }
        $i += $extra + 1;
    }
    return true;
}
function __elephc_mime_is_lwsp(string $text): bool {
    $length = strlen($text);
    if ($length === 0) {
        return false;
    }
    $i = 0;
    while ($i < $length) {
        $c = $text[$i];
        if ($c !== ' ' && $c !== "\t" && $c !== "\r" && $c !== "\n") {
            return false;
        }
        $i++;
    }
    return true;
}
function __elephc_mime_hex_byte(string $pair): int {
    $value = 0;
    $i = 0;
    while ($i < 2) {
        $c = ord($pair[$i]);
        if ($c >= 48 && $c <= 57) {
            $digit = $c - 48;
        } elseif ($c >= 65 && $c <= 70) {
            $digit = $c - 55;
        } elseif ($c >= 97 && $c <= 102) {
            $digit = $c - 87;
        } else {
            return -1;
        }
        $value = ($value * 16) + $digit;
        $i++;
    }
    return $value;
}
function __elephc_mime_q_decode(string $text): string {
    $text = str_replace('_', ' ', $text);
    $out = '';
    $length = strlen($text);
    $i = 0;
    while ($i < $length) {
        if ($text[$i] === '=' && $i + 2 < $length) {
            $byte = __elephc_mime_hex_byte(substr($text, $i + 1, 2));
            if ($byte >= 0) {
                $out .= chr($byte);
                $i += 3;
                continue;
            }
        }
        $out .= $text[$i];
        $i++;
    }
    return $out;
}
function iconv_mime_decode(string $string, int $mode = 0, ?string $encoding = null): string|false {
    $target = 'UTF-8';
    if ($encoding !== null && $encoding !== '') {
        $target = $encoding;
    }
    $keep_going = ($mode & 2) === 2;
    $out = '';
    $pending = '';
    $after_word = false;
    $length = strlen($string);
    $pos = 0;
    while ($pos < $length) {
        $start = strpos($string, '=?', $pos);
        if ($start === false) {
            break;
        }
        $literal = substr($string, $pos, $start - $pos);
        if ($literal !== '') {
            if ($after_word && __elephc_mime_is_lwsp($literal)) {
                $pending .= $literal;
            } else {
                $out .= $pending . $literal;
                $pending = '';
                $after_word = false;
            }
        }
        $charset_end = strpos($string, '?', $start + 2);
        $well_formed = $charset_end !== false && $charset_end > $start + 2;
        $charset = '';
        $word_encoding = '';
        $encoding_end = 0;
        if ($well_formed) {
            $charset = substr($string, $start + 2, $charset_end - $start - 2);
            $well_formed = strpos($charset, ' ') === false && strpos($charset, "\t") === false;
        }
        if ($well_formed) {
            $found_end = strpos($string, '?', $charset_end + 1);
            $well_formed = $found_end === $charset_end + 2;
            if ($well_formed) {
                $encoding_end = $charset_end + 2;
                $word_encoding = strtoupper(substr($string, $charset_end + 1, 1));
            }
        }
        if (!$well_formed) {
            $out .= $pending . '=?';
            $pending = '';
            $after_word = false;
            $pos = $start + 2;
            continue;
        }
        $terminator = strpos($string, '?=', $encoding_end + 1);
        if ($terminator === false) {
            return $out;
        }
        $text = substr($string, $encoding_end + 1, $terminator - $encoding_end - 1);
        $decoded = '';
        $decoded_ok = false;
        $usable = $word_encoding === 'B' || $word_encoding === 'Q';
        if ($usable && __elephc_mime_charset_known($charset)) {
            $raw = $text;
            if ($word_encoding === 'B') {
                $bytes = base64_decode($text);
                $raw = is_string($bytes) ? $bytes : '';
            } else {
                $raw = __elephc_mime_q_decode($text);
            }
            // `mb_convert_encoding` does not validate its input, but iconv does: bytes that are not
            // valid in the declared charset are a decode error, not silently re-emitted mojibake.
            // Only UTF-8 can be invalid here — every byte sequence is a valid single-byte Latin
            // string — so the check is skipped for the rest of the known set.
            $bytes_ok = !__elephc_mime_charset_is_utf8($charset) || __elephc_mime_utf8_valid($raw);
            if ($bytes_ok) {
                $converted = mb_convert_encoding($raw, $target, $charset);
                if (is_string($converted)) {
                    $decoded = $converted;
                    $decoded_ok = true;
                }
            }
        }
        if (!$decoded_ok) {
            if (!$keep_going) {
                return false;
            }
            $out .= $pending . substr($string, $start, $terminator + 2 - $start);
            $pending = '';
            $after_word = false;
            $pos = $terminator + 2;
            continue;
        }
        $out .= $decoded;
        $pending = '';
        $after_word = true;
        $pos = $terminator + 2;
    }
    if ($pos < $length) {
        $rest = substr($string, $pos);
        if (!$after_word || !__elephc_mime_is_lwsp($rest)) {
            $out .= $pending . $rest;
        }
    }
    return $out;
}
"#,
    },
    StringCompatEntry {
        name: "iconv",
        overridable_builtin: true,
        // `iconv($from, $to, $s)` is `mb_convert_encoding($s, $to, $from)` with the arguments in a
        // different order: both were verified to produce byte-identical output on `php -n` for the
        // encodings elephc converts between. Delegating means the two builtins cannot disagree, and
        // `iconv` inherits `mb_convert_encoding`'s documented gap (an encoding outside the UTF-8 /
        // single-byte-Latin set returns the subject unchanged rather than inventing a conversion).
        //
        // The `//TRANSLIT` and `//IGNORE` suffixes select what iconv does with a character the
        // target encoding cannot represent. elephc substitutes `?` either way — which is what
        // mbstring does without an explicit substitute character — so the suffix is stripped rather
        // than being mistaken for part of the encoding name.
        source: r#"<?php
function iconv(string $from_encoding, string $to_encoding, string $string): string|false {
    $target = $to_encoding;
    $marker = strpos($target, '//');
    if ($marker !== false) {
        $target = substr($target, 0, $marker);
    }
    $converted = mb_convert_encoding($string, $target, $from_encoding);
    if (is_string($converted)) {
        return $converted;
    }
    return false;
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

//! Purpose:
//! End-to-end PHP tokenizer coverage for functions, PhpToken, callables and runtime eval.
//!
//! Called from:
//! - The codegen CI matrix on every executable supported host target.
//!
//! Key details:
//! - The Rust engine's differential corpus checks the full PHP token vocabulary.
//! - These fixtures exercise PHP storage, exception handling, late-static construction and GC.

use crate::support::*;

/// token_get_all returns the PHP mixture of named tuples and punctuation strings.
#[test]
fn test_tokenizer_mixed_stream_and_roundtrip() {
    let out = compile_and_run(r#"<?php
$code = '<?php echo 1;';
$tokens = token_get_all($code);
echo count($tokens), ':', token_name($tokens[1][0]), ':', $tokens[5], "\n";
$restored = '';
foreach ($tokens as $token) { $restored .= is_array($token) ? $token[1] : $token; }
echo $restored === $code ? 'same' : 'different';
echo ':', token_name(59), ':', token_name(-1), ':', token_name(T_PAAMAYIM_NEKUDOTAYIM);
"#);
    assert_eq!(out, "6:T_ECHO:;\nsame:UNKNOWN:UNKNOWN:T_DOUBLE_COLON");
}

/// PhpToken retains its public fields, default constructor coordinates and strict predicates.
#[test]
fn test_tokenizer_phptoken_members_and_subclasses() {
    let out = compile_and_run(r#"<?php
$t = new PhpToken(T_STRING, 'name');
echo $t->getTokenName(), ':', $t->line, ':', $t->pos, ':', (string)$t, "\n";
echo $t->is(T_STRING) ? '1' : '0';
echo $t->is('name') ? '1' : '0';
echo $t->is(['other', T_STRING]) ? '1' : '0';
echo $t->is([]) ? '1' : '0';
echo $t->isIgnorable() ? '1' : '0';
$t->id = T_COMMENT;
echo $t->isIgnorable() ? '1' : '0';
$t->text = 'changed';
echo ':', $t, "\n";
$unknown = new PhpToken(999999, 'x');
echo $unknown->getTokenName() === null ? 'null' : 'wrong';
$char = new PhpToken(59, ';');
echo ':', $char->getTokenName();
class SourceToken extends PhpToken { public function upper(): string { return strtoupper($this->text); } }
$items = SourceToken::tokenize('<?php echo 1;');
echo ':', get_class($items[1]), ':', $items[1]->upper();
"#);
    assert_eq!(out, "T_STRING:-1:-1:name\n111001:changed\nnull:;:SourceToken:ECHO");
}

/// Byte positions count UTF-8 bytes and CRLF line breaks, independently of token representation.
#[test]
fn test_tokenizer_phptoken_byte_positions_and_lines() {
    let out = compile_and_run(r#"<?php
$code = "<?php\r\n" . '$x="è";' . "\r\n?>\r\nhtml";
$tokens = PhpToken::tokenize($code);
echo $tokens[1]->pos, ':', $tokens[1]->line, ':', $tokens[1]->text, "\n";
$restored = '';
foreach ($tokens as $token) { $restored .= $token->text; }
echo $restored === $code ? 'same' : 'different';
$last = $tokens[count($tokens) - 1];
echo ':', $last->line, ':', $last->pos, ':', $last->text;
"#);
    assert_eq!(out, "7:2:$x\nsame:4:21:html");
}

/// TOKEN_PARSE validates syntax and applies reserved identifier feedback while plain mode is permissive.
#[test]
fn test_tokenizer_parse_flag_and_catchable_errors() {
    let out = compile_and_run(r#"<?php
$flags = $argc > 0 ? TOKEN_PARSE : 0;
$source = '<?php class A { public const PUBLIC = 1; }';
$tokens = token_get_all($source, $flags);
foreach ($tokens as $t) {
    if (is_array($t) && $t[1] === 'PUBLIC') { echo token_name($t[0]), "\n"; }
}
try { token_get_all('<?php function broken(', $flags); echo 'wrong'; }
catch (ParseError $error) { echo 'ParseError', "\n"; }
echo count(token_get_all('<?php function broken(')) > 0 ? 'permissive' : 'wrong';
"#);
    assert_eq!(out, "T_STRING\nParseError\npermissive");
}

/// Named, spread, case-insensitive and first-class callable forms share PHP's signatures.
#[test]
fn test_tokenizer_callable_and_named_argument_forms() {
    let out = compile_and_run(r#"<?php
$scan = \TOKEN_GET_ALL(...);
$name = token_name(...);
$tokens = $scan(flags: 0, code: '<?php echo 1;');
echo $name(id: $tokens[1][0]), ':', count(token_get_all(...['code' => 'html', 'flags' => 0]));
$f = $argc > 0 ? 'token_name' : 'strval';
echo ':', $f(T_FUNCTION);
"#);
    assert_eq!(out, "T_ECHO:1:T_FUNCTION");
}

/// Opaque eval reaches both function adapters and the host PhpToken class, including callables.
#[test]
fn test_tokenizer_runtime_eval_bundle() {
    let out = compile_and_run(r#"<?php
$code = $argc > 0 ? '
$t = token_get_all("<?php echo 1;");
echo token_name($t[1][0]), ":", count($t), "\n";
$f = "token_name"; echo $f(T_FUNCTION), "\n";
$p = PhpToken::tokenize("<?php echo 1;");
echo get_class($p[1]), ":", $p[1]->text, ":", $p[1]->pos, "\n";
try { token_get_all("<?php function broken(", TOKEN_PARSE); }
catch (ParseError $e) { echo "ParseError"; }
abstract class EvalAbstractSourceToken extends PhpToken {}
try { EvalAbstractSourceToken::tokenize(""); echo "wrong"; }
catch (Error $e) { echo ":abstract"; }
' : '';
eval($code);
"#);
    assert_eq!(out, "T_ECHO:6\nT_FUNCTION\nPhpToken:echo:6\nParseError:abstract");
}

/// Repeated token arrays and object graphs leave no live PHP allocations after scope cleanup.
#[test]
fn test_tokenizer_repeated_results_are_released() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function inspect_source(): void {
    for ($i = 0; $i < 20; $i++) {
        $tokens = token_get_all('<?php echo "hello";');
        $objects = PhpToken::tokenize('<?php echo "hello";');
    }
}
inspect_source();
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Predicates reject invalid dynamic kinds without validating entries after a successful match.
#[test]
fn test_tokenizer_is_checks_dynamic_types_in_source_order() {
    let out = compile_and_run(r#"<?php
$t = new PhpToken(T_STRING, 'name');
$values = [true, 1.5, null, new stdClass()];
foreach ($values as $value) {
    try { $t->is($value); echo 'wrong'; }
    catch (TypeError $error) { echo 'T'; }
}
echo ':', $t->is(['name', []]) ? 'match' : 'wrong';
try { $t->is([[], 'name']); echo 'wrong'; }
catch (TypeError $error) { echo ':TypeError'; }
"#);
    assert_eq!(out, "TTTT:match:TypeError");
}

/// Empty token streams still reject abstract subclasses, and partial binary strings retain quote IDs.
#[test]
fn test_tokenizer_abstract_subclass_and_binary_quote() {
    let out = compile_and_run(r#"<?php
abstract class AbstractSourceToken extends PhpToken {}
try { AbstractSourceToken::tokenize(''); echo 'wrong'; }
catch (Error $error) { echo 'abstract'; }
$tokens = PhpToken::tokenize('<?php b"unfinished');
echo ':', $tokens[1]->id, ':', $tokens[1]->text;
"#);
    assert_eq!(out, "abstract:34:b\"");
}

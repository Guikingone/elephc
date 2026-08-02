//! Purpose:
//! Type-check-only integration tests for the built-in DOM extension shell classes
//! (`DOMNode`, `DOMDocument`, `DOMElement`, `DOMText`, `DOMNodeList`) registered for
//! vendor code such as symfony/console's `XmlDescriptor.php`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - These classes are declared as ordinary PHP by `elephc::dom_prelude` and injected by the
//!   pipeline, so they reach lowering and DO execute. `compile_and_run` therefore works and is
//!   used for the behavioural assertions; `type_checks_cleanly` is kept for the signature-shape
//!   assertions, and injects the prelude itself because it reimplements the frontend rather than
//!   calling the pipeline.
//! - Still incomplete on purpose: there is no XML parser, so `loadXML()` and
//!   `schemaValidateSource()` report failure. A shell may answer "I could not", never "I did".

use crate::codegen::oop::compile_and_run;

/// Runs the frontend pipeline through type-checking only (tokenize → parse → autoload
/// alias collection → name resolution → constant folding → type check), without
/// lowering to EIR or generating assembly. Mirrors `tests/error_tests.rs`'s
/// `check_source` helper and `oop::attributes`'s `type_checks_cleanly` helper (each
/// builtin-shell test module keeps its own local copy rather than sharing one, matching
/// this codebase's existing convention of self-contained builtin-type test modules).
fn type_checks_cleanly(source: &str) -> Result<(), String> {
    let tokens = elephc::lexer::tokenize(source).map_err(|e| e.message.clone())?;
    let ast = elephc::parser::parse(&tokens).map_err(|e| e.message.clone())?;
    let ast = elephc::dom_prelude::inject(ast);
    let ast = elephc::autoload::collect_aliases(ast);
    let ast = elephc::name_resolver::resolve(ast).map_err(|e| e.message.clone())?;
    let ast = elephc::optimize::fold_constants(ast);
    elephc::types::check(&ast).map_err(|e| e.message.clone())?;
    Ok(())
}

/// Verifies that `\DOMNode`- and `\DOMDocument`-typed parameters/return types
/// type-check, and that `DOMDocument::saveXML(): string` is callable with no
/// arguments (its `?DOMNode $node = null` parameter is optional).
#[test]
fn test_dom_node_and_document_typed_signatures_type_check() {
    let result = type_checks_cleanly(
        r#"<?php
function f(\DOMNode $n): void {}
function g(\DOMDocument $d): string { return $d->saveXML(); }
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {:?}", result);
}

/// Verifies the `XmlDescriptor.php`-style construction sequence type-checks:
/// constructing a `DOMDocument`, creating an element via `createElement()`, writing an
/// attribute via `setAttribute()`, appending it via `appendChild()`, and serializing via
/// `saveXML()`. This is a type-check-only assertion (see the module docblock for why a
/// full `compile_and_run` is not used here — DOM runtime/codegen is a later cluster).
#[test]
fn test_dom_document_create_element_append_save_xml_type_checks() {
    let result = type_checks_cleanly(
        r#"<?php
$d = new \DOMDocument();
$e = $d->createElement('x');
$e->setAttribute('a', 'b');
$d->appendChild($e);
echo $d->saveXML();
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {:?}", result);
}

/// Verifies `DOMElement`/`DOMText` are assignable where `\DOMNode` is expected, proving
/// the `extends: "DOMNode"` wiring on both subclasses (and that a `DOMText` created via
/// `createTextNode()` can be appended through a `\DOMNode`-typed parameter).
#[test]
fn test_dom_element_and_text_assignable_to_dom_node_param() {
    let result = type_checks_cleanly(
        r#"<?php
function accept(\DOMNode $n): void {}
$d = new \DOMDocument();
$el = $d->createElement('x');
accept($el);
$text = $d->createTextNode('hi');
accept($text);
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {:?}", result);
}

/// Verifies the `symfony/config` `XmlUtils::parse()` sequence type-checks: setting
/// `$validateOnParse`, calling `loadXML()` in a boolean condition, `normalizeDocument()`,
/// and `schemaValidateSource()`. These four members are what made that file the sole
/// source of every remaining DOM diagnostic in the Symfony app.
#[test]
fn test_dom_document_parse_surface_type_checks() {
    let result = type_checks_cleanly(
        r#"<?php
function parseIt(string $content, string $schema): bool {
    $dom = new \DOMDocument();
    $dom->validateOnParse = true;
    if (!$dom->loadXML($content, 0)) {
        return false;
    }
    $dom->normalizeDocument();
    return $dom->schemaValidateSource($schema);
}
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {:?}", result);
}

/// Verifies the `XmlUtils::convertDomElementToArray()` sequence type-checks: reading
/// `$prefix` off a `\DOMElement`, iterating `$attributes` (a `DOMNamedNodeMap`, whose only
/// job in this shell is to make that `foreach` resolve), and reading `$nodeValue` off a
/// node narrowed to `\DOMText`. All three properties live on `DOMNode` in PHP, so the
/// subclasses inherit them here exactly as they do there.
#[test]
fn test_dom_node_value_prefix_and_attributes_type_check() {
    let result = type_checks_cleanly(
        r#"<?php
function convert(\DOMElement $element): array {
    $prefix = $element->prefix;
    $config = [];
    foreach ($element->attributes as $name => $node) {
        $config[$name] = $prefix;
    }
    foreach ($element->childNodes as $node) {
        if ($node instanceof \DOMText) {
            $config['text'] = trim($node->nodeValue);
        }
    }
    return $config;
}
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {:?}", result);
}

/// The point of declaring the DOM surface as PHP rather than as checker-only shells: it now has
/// EIR method bodies, so a DOM program COMPILES AND RUNS. Before this, the identical source
/// type-checked cleanly and then died at the backend with "constructor call to
/// DOMDocument::__construct without an emitted EIR method body".
#[test]
fn test_dom_document_constructs_and_runs() {
    let out = compile_and_run(
        r#"<?php
$d = new \DOMDocument('1.0', 'UTF-8');
$d->formatOutput = true;
$e = $d->createElement('item', 'v');
$e->setAttribute('id', 7);
$d->appendChild($e);
echo 'ok:', $d->saveXML(), '|', $e->prefix, '|', $e->localName, "\n";
"#,
    );
    assert_eq!(out, "ok:||\n");
}

/// The parse surface runs and reports FAILURE, which is the honest answer with no XML parser
/// behind it: `XmlUtils::parse()` then takes its own `XmlParsingException` path instead of
/// building a container out of an empty document it believed was parsed.
#[test]
fn test_dom_parse_predicates_run_and_report_failure() {
    let out = compile_and_run(
        r#"<?php
$d = new \DOMDocument();
$d->validateOnParse = true;
$loaded = $d->loadXML('<root/>', 0);
$d->normalizeDocument();
$valid = $d->schemaValidateSource('<xsd/>');
echo $d->validateOnParse ? 'set' : 'unset', '|', $loaded ? 'y' : 'n', '|', $valid ? 'y' : 'n', "\n";
"#,
    );
    assert_eq!(out, "set|n|n\n");
}

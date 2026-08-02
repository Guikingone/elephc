//! Purpose:
//! Type-check-only integration tests for the built-in DOM extension shell classes
//! (`DOMNode`, `DOMDocument`, `DOMElement`, `DOMText`, `DOMNodeList`) registered for
//! vendor code such as symfony/console's `XmlDescriptor.php`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - These classes are TYPE-CHECK-ONLY shells: there is no DOM runtime yet, so every
//!   assertion here goes through `type_checks_cleanly` (tokenize → parse → autoload
//!   alias collection → name resolution → constant folding → type check) rather than
//!   `compile_and_run`. A `compile_and_run` test that actually executes a DOM method
//!   would fail at EIR/codegen (expected — DOM codegen/runtime is a later, separate
//!   cluster) so none is written here.

/// Runs the frontend pipeline through type-checking only (tokenize → parse → autoload
/// alias collection → name resolution → constant folding → type check), without
/// lowering to EIR or generating assembly. Mirrors `tests/error_tests.rs`'s
/// `check_source` helper and `oop::attributes`'s `type_checks_cleanly` helper (each
/// builtin-shell test module keeps its own local copy rather than sharing one, matching
/// this codebase's existing convention of self-contained builtin-type test modules).
fn type_checks_cleanly(source: &str) -> Result<(), String> {
    let tokens = elephc::lexer::tokenize(source).map_err(|e| e.message.clone())?;
    let ast = elephc::parser::parse(&tokens).map_err(|e| e.message.clone())?;
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

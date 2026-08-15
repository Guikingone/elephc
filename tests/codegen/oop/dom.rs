//! Purpose:
//! Integration coverage for the standard DOM class surface injected by the compiler.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Type checks pin inheritance, iteration, properties, signatures, and named parameters.
//! - Runtime checks prove the declarations reach EIR lowering and method dispatch.
//! - Parsing predicates report failure until an XML parser backs the declared surface.

use crate::codegen::oop::compile_and_run;

/// Runs the frontend through type checking with the standard DOM declarations injected.
fn type_checks_cleanly(source: &str) -> Result<(), String> {
    let tokens = elephc::lexer::tokenize(source).map_err(|error| error.message.clone())?;
    let ast = elephc::parser::parse(&tokens).map_err(|error| error.message.clone())?;
    let ast = elephc::autoload::collect_aliases(ast);
    let ast = elephc::dom_prelude::inject(ast);
    let ast = elephc::name_resolver::resolve(ast).map_err(|error| error.message.clone())?;
    let ast = elephc::optimize::fold_constants(ast);
    elephc::types::check(&ast).map_err(|error| error.message.clone())?;
    Ok(())
}

/// Verifies the node hierarchy and optional serialization parameter type-check.
#[test]
fn test_dom_node_and_document_typed_signatures_type_check() {
    let result = type_checks_cleanly(
        r#"<?php
function accept_node(\DOMNode $node): void {}
function serialize_document(\DOMDocument $document): string { return $document->saveXML(); }
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {result:?}");
}

/// Verifies the standard construction, mutation, append, and serialization call surface.
#[test]
fn test_dom_document_mutation_surface_type_checks() {
    let result = type_checks_cleanly(
        r#"<?php
$document = new \DOMDocument();
$element = $document->createElement(name: 'item', value: 'content');
$element->setAttribute(name: 'id', value: '7');
$document->appendChild($element);
echo $document->saveXML(node: $element);
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {result:?}");
}

/// Verifies node collections and named-node maps are iterable with inherited node properties.
#[test]
fn test_dom_collection_and_property_surface_type_checks() {
    let result = type_checks_cleanly(
        r#"<?php
function inspect_element(\DOMElement $element): array {
    $result = [];
    foreach ($element->attributes as $name => $attribute) {
        $result[$name] = $attribute->nodeValue;
    }
    foreach ($element->childNodes as $node) {
        if ($node instanceof \DOMText) {
            $result['text'] = $node->nodeValue;
        }
    }
    return $result;
}
"#,
    );
    assert!(result.is_ok(), "expected type-check success, got: {result:?}");
}

/// Verifies DOM declarations and constants reach EIR with executable method bodies.
#[test]
fn test_dom_document_constructs_and_runs() {
    let out = compile_and_run(
        r#"<?php
$document = new \DOMDocument('1.0', 'UTF-8');
$document->formatOutput = true;
$element = $document->createElement('item', 'value');
$text = $document->createTextNode('content');
echo $element->ownerDocument === $document ? 'owner' : 'missing';
echo '|', $text->nodeValue, '|', $document->saveXML(), '|', XML_DOCUMENT_TYPE_NODE;
"#,
    );
    assert_eq!(out, "owner|content||10");
}

/// Verifies parsing predicates fail honestly while their backend implementation is unavailable.
#[test]
fn test_dom_parse_predicates_report_failure() {
    let out = compile_and_run(
        r#"<?php
$document = new \DOMDocument();
$document->validateOnParse = true;
$loaded = $document->loadXML('<root/>');
$document->normalizeDocument();
$valid = $document->schemaValidateSource('<schema/>');
echo $document->validateOnParse ? 'set' : 'unset';
echo '|', $loaded ? 'yes' : 'no', '|', $valid ? 'yes' : 'no';
"#,
    );
    assert_eq!(out, "set|no|no");
}

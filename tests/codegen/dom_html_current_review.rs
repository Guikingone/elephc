//! Purpose:
//! Regressions for legacy HTML implied bodies, reopened roots and non-void tags.
//!
//! Called from:
//! - The DOM HTML codegen integration test module.
//!
//! Key details:
//! - Expected trees follow PHP with libxml 2.9.14, not modern HTML5 parsing.

use crate::support::*;

/// An implied body excludes head and roots following the closed outer html.
#[test]
fn dom_html_current_review_implied_body() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
$doc->loadHTML('<html><head></head><div>B</div></html><span>C</span>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
$html = $doc->getElementsByTagName('html')->item(0);
$body = $doc->getElementsByTagName('body')->item(0);
echo $doc->saveXML($html), '|', $body->childNodes->item(0)->nodeName, '|';
echo $body->nodeValue, ':', $html->childNodes->item(0)->nodeName;
"#);
    assert_eq!(out, "<html><head/><body><div>B</div></body><html><span>C</span></html></html>|div|B:head");
}

/// A later explicit html root is adopted rather than nested in another wrapper.
#[test]
fn dom_html_current_review_adopt_reopened_html() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
$doc->loadHTML('<html><head></head><div>B</div></html><html><span>C</span></html>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
echo $doc->saveXML($doc->getElementsByTagName('html')->item(0));
"#);
    assert_eq!(out, "<html><head/><body><div>B</div></body><html><span>C</span></html></html>");
}

/// A head-only document does not acquire an empty body containing its head.
#[test]
fn dom_html_current_review_head_only() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
$doc->loadHTML('<html><head></head></html>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
echo $doc->saveXML($doc->getElementsByTagName('html')->item(0)), ':', $doc->getElementsByTagName('body')->length;
"#);
    assert_eq!(out, "<html><head/></html>:0");
}

/// Legacy libxml retains text children in embed, source, track and wbr elements.
#[test]
fn dom_html_current_review_legacy_non_void_elements() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
foreach (['embed', 'source', 'track', 'wbr'] as $tag) {
    $doc->loadHTML('<div>a<' . $tag . '>b</div>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
    $node = $doc->getElementsByTagName($tag)->item(0);
    echo $doc->saveXML($doc->getElementsByTagName('div')->item(0)), ':', $node->nodeValue, '|';
}
"#);
    assert_eq!(out, "<div>a<embed>b</embed></div>:b|<div>a<source>b</source></div>:b|<div>a<track>b</track></div>:b|<div>a<wbr>b</wbr></div>:b|");
}

/// Implied-body construction preserves the same tree without EIR optimization.
#[test]
fn dom_html_current_review_implied_body_no_ir_opt() {
    without_ir_opt(dom_html_current_review_implied_body);
}

/// Explicit reopened wrappers remain single nodes without EIR optimization.
#[test]
fn dom_html_current_review_adopt_reopened_html_no_ir_opt() {
    without_ir_opt(dom_html_current_review_adopt_reopened_html);
}

/// Legacy tag classification is independent of EIR optimization.
#[test]
fn dom_html_current_review_legacy_non_void_elements_no_ir_opt() {
    without_ir_opt(dom_html_current_review_legacy_non_void_elements);
}

/// Literal wrapper partitioning retires its intermediate arrays without DOM cycles.
#[test]
fn dom_html_current_review_wrapper_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function enableDom(DOMDocument $doc): void {}
for ($i = 0; $i < 16; $i++) {
    $head = __elephc_dom_new_node('element', 'head', '', [], []);
    $text = __elephc_dom_new_node('text', '#text', 'B', [], []);
    $div = __elephc_dom_new_node('element', 'div', '', [], [$text]);
    $html = __elephc_dom_new_node('element', 'html', '', [], [$head, $div]);
    $later = __elephc_dom_new_node('element', 'html', '', [], [$div]);
    $roots = [$html, $later];
    $tree = __elephc_dom_wrap_html($roots);
    echo __elephc_dom_text_of($tree), '|';
    unset($tree, $roots);
}
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "BB|".repeat(16));
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Wrapper temporary owners remain balanced without EIR optimization.
#[test]
fn dom_html_current_review_wrapper_owners_no_ir_opt() {
    without_ir_opt(dom_html_current_review_wrapper_owners);
}

/// Head-only wrapper selection is independent of EIR optimization.
#[test]
fn dom_html_current_review_head_only_no_ir_opt() {
    without_ir_opt(dom_html_current_review_head_only);
}

/// A reopened document creates its first body when the original html contained only a head.
#[test]
fn dom_html_latest_review_head_only_reopened_body() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
foreach (['<html><head></head></html><div>B</div>', '<html><head></head></html><html lang="en"><div>B</div></html>'] as $source) {
    $doc->loadHTML($source, LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
    $body = $doc->getElementsByTagName('body')->item(0);
    echo $doc->saveXML($doc->getElementsByTagName('html')->item(0)), ':', $body->nodeValue, '|';
}
"#);
    assert_eq!(out, "<html><head/><html><body><div>B</div></body></html></html>:B|<html><head/><html lang=\"en\"><body><div>B</div></body></html></html>:B|");
}

/// An html start tag inside an open html is ignored, including its attributes.
#[test]
fn dom_html_latest_review_redundant_html() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
foreach (['<html id=a><html id=b class=c><div>B</div></html></html>', '<html><body>A</body><html lang=en><span>B</span></html></html>'] as $source) {
    $doc->loadHTML($source, LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
    echo $doc->getElementsByTagName('html')->length, ':', $doc->saveXML($doc->getElementsByTagName('html')->item(0)), '|';
}
"#);
    assert_eq!(out, "1:<html id=\"a\"><body><div>B</div></body></html>|1:<html><body>A</body><span>B</span></html>|");
}

/// Head-compatible metadata after head stays outside the subsequently implied body.
#[test]
fn dom_html_latest_review_post_head_metadata() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
$doc->loadHTML('<html><head></head><script>X</script><style>Y</style><base href=x><link href=y><meta name=z><title>T</title><div>B</div></html>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
echo $doc->saveXML($doc->getElementsByTagName('html')->item(0)), ':', $doc->getElementsByTagName('body')->item(0)->nodeValue;
"#);
    assert_eq!(out, "<html><head/><script><![CDATA[X]]></script><style><![CDATA[Y]]></style><base href=\"x\"/><link href=\"y\"/><meta name=\"z\"/><title>T</title><body><div>B</div></body></html>:B");
}

/// NOBLANKS removes wrapper padding only after the implied body's boundaries are known.
#[test]
fn dom_html_latest_review_implied_body_blanks() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
foreach ([0, LIBXML_NOBLANKS] as $extra) {
    $doc->loadHTML('<html><head></head> <div>A</div> <b>B</b> </html>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR | $extra);
    echo $doc->saveXML($doc->getElementsByTagName('html')->item(0)), ':', $doc->getElementsByTagName('body')->item(0)->childNodes->length, '|';
}
"#);
    assert_eq!(out, "<html><head/> <body><div>A</div> <b>B</b> </body></html>:4|<html><head/><body><div>A</div> <b>B</b> </body></html>:4|");
}

/// Legacy frame and isindex tags are void and leave following text beside them.
#[test]
fn dom_html_latest_review_legacy_void_elements() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
$doc->loadHTML('<div>A<frame>B<isindex>C</div>', LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
$div = $doc->getElementsByTagName('div')->item(0);
echo $doc->saveXML($div), ':', $div->childNodes->length, ':', $div->nodeValue;
"#);
    assert_eq!(out, "<div>A<frame/>B<isindex/>C</div>:5:ABC");
}

/// A stray slash consumes its malformed attribute run but does not self-close a tag.
#[test]
fn dom_html_latest_review_stray_attribute_slash() {
    let out = compile_and_run(r#"<?php
$doc = new DOMDocument();
foreach (['<div /class=x /id=y>A</div><b>Z</b>', '<div a/b=c d=e>A</div><b>Z</b>', '<div / / >A</div><b>Z</b>', '<div a=x/ b=y>A</div><b>Z</b>'] as $source) {
    $doc->loadHTML($source, LIBXML_HTML_NODEFDTD | LIBXML_NOERROR);
    echo $doc->saveXML($doc->getElementsByTagName('body')->item(0)), '|';
}
"#);
    assert_eq!(out, "<body><div>A</div><b>Z</b></body>|<body><div a=\"\" d=\"e\">A</div><b>Z</b></body>|<body><div>A</div><b>Z</b></body>|<body><div a=\"x/\" b=\"y\">A</div><b>Z</b></body>|");
}

/// Reopened-body construction is independent of EIR optimization.
#[test]
fn dom_html_latest_review_head_only_reopened_body_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_head_only_reopened_body);
}

/// Ignoring redundant html tags is independent of EIR optimization.
#[test]
fn dom_html_latest_review_redundant_html_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_redundant_html);
}

/// Post-head metadata placement is independent of EIR optimization.
#[test]
fn dom_html_latest_review_post_head_metadata_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_post_head_metadata);
}

/// Implied-body whitespace is preserved without EIR optimization.
#[test]
fn dom_html_latest_review_implied_body_blanks_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_implied_body_blanks);
}

/// Legacy void-element boundaries are independent of EIR optimization.
#[test]
fn dom_html_latest_review_legacy_void_elements_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_legacy_void_elements);
}

/// Attribute slash recovery is independent of EIR optimization.
#[test]
fn dom_html_latest_review_stray_attribute_slash_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_stray_attribute_slash);
}

/// Isolates one wrapper ownership path from the tokenizer and cyclic DOM object graph.
fn verify_literal_tree_owners(reopened: bool, flags: u32) {
    let setup = r#"<?php
function enableDom(DOMDocument $doc): void {}
for ($i = 0; $i < 8; $i++) {
    $head = __elephc_dom_new_node('element', 'head', '', [], []);
    $div = __elephc_dom_new_node('element', 'div', '', [], [__elephc_dom_new_node('text', '#text', 'A', [], [])]);
"#;
    let roots = if reopened {
        r#"$html = __elephc_dom_new_node('element', 'html', '', [], [$head]);
$later = __elephc_dom_new_node('element', 'html', '', [], [$div]);
$roots = [$html, $later];
"#
    } else {
        r#"$blank = __elephc_dom_new_node('text', '#text', ' ', [], []);
$title = __elephc_dom_new_node('element', 'title', '', [], [__elephc_dom_new_node('text', '#text', 'T', [], [])]);
$html = __elephc_dom_new_node('element', 'html', '', [], [$head, $blank, $title, $div, $blank]);
$roots = [$html];
"#
    };
    let finish = r#"$tree = __elephc_dom_document_from_roots($roots, LIBXML_NOBLANKS);
echo __elephc_dom_text_of($tree), '|';
unset($tree, $roots);
}
"#;
    let finish = finish.replace("LIBXML_NOBLANKS", &flags.to_string());
    let out = compile_and_run_with_heap_debug(&[setup, roots, &finish].concat());
    assert!(out.success, "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
    assert_eq!(out.stdout, if reopened { "A|" } else { "TA |" }.repeat(8));
}

/// Literal metadata and whitespace trees retire the new body and filter owners.
#[test]
fn dom_html_latest_review_tree_owners() {
    verify_literal_tree_owners(false, 256);
}

/// Body and whitespace temporary owners stay balanced without EIR optimization.
#[test]
fn dom_html_latest_review_tree_owners_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_tree_owners);
}

/// A head-only document's reopened body retires all temporary wrapper owners.
#[test]
fn dom_html_latest_review_reopened_tree_owners() {
    verify_literal_tree_owners(true, 256);
}

/// Reopened-body temporary owners stay balanced without EIR optimization.
#[test]
fn dom_html_latest_review_reopened_tree_owners_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_reopened_tree_owners);
}

/// The reopened-body ownership control also covers parsing without blank filtering.
#[test]
fn dom_html_latest_review_reopened_tree_owners_no_flags() {
    verify_literal_tree_owners(true, 0);
}

/// Reopened-body ownership without flags is independent of EIR optimization.
#[test]
fn dom_html_latest_review_reopened_tree_owners_no_flags_no_ir_opt() {
    without_ir_opt(dom_html_latest_review_reopened_tree_owners_no_flags);
}

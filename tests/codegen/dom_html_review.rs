//! Purpose:
//! Covers user-class ownership and tree construction regressions in the DOM HTML subset.
//!
//! Called from:
//! - The DOM HTML codegen integration tests.
//!
//! Key details:
//! - Closing tags match open element names; comments retain their own value only.
//! - User-owned DOM names must not trigger conflicting prelude declarations.
//! - Generic type references inject DOM before monomorphization, without stealing user names.

use crate::support::*;

/// Entity helper temporaries and the static named table release without constructing a DOM tree.
#[test]
fn dom_html_third_review_entity_decoder_heap_clean() {
    for probe in [
        "echo __elephc_dom_named_entity_codepoint('AElig');",
        "echo __elephc_dom_entity_utf8(128512);",
        "echo __elephc_dom_decode_entities('plain');",
        "echo __elephc_dom_decode_entities('&AElig;');",
        "echo __elephc_dom_decode_entities('&#65');",
    ] {
        let source = format!("<?php function enableDomPrelude(DOMDocument $unused): void {{}} {probe}");
        let result = compile_and_run_with_heap_debug(&source);
        assert!(result.success, "{probe}: {}", result.stderr);
        assert!(result.stderr.contains("HEAP DEBUG: leak summary: clean"), "{probe}: {}", result.stderr);
    }
    let out = compile_and_run_with_heap_debug(r###"<?php
function enableDomPrelude(DOMDocument $unused): void {}
for ($i = 0; $i < 3; $i++) {
    echo __elephc_dom_decode_entities('&AElig;&#x1F600;&amp;lt; &unknown; &#65'), '|';
}
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "Æ😀&lt; &unknown; A|Æ😀&lt; &unknown; A|Æ😀&lt; &unknown; A|", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Text, quoted and unquoted attributes decode legacy named and numeric HTML references.
#[test]
fn dom_html_third_review_entities() {
    let out = compile_and_run_capture(r###"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div title="&euro;&#x1F600;&#60;&apos;" data=&nbsp;&#65;>a&nbsp;b&#60;c&apos;d</div>');
$div = $dom->getElementsByTagName('div')->item(0);
echo $div->nodeValue, '|', $div->getAttribute('title'), '|', $div->getAttribute('data'), '|';
$dom->loadHTML('<div>a&nbsp;b&#60;c&apos;d</div>');
echo $dom->saveXML($dom->getElementsByTagName('div')->item(0));
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "a b<c'd|€😀<'| A|<div>a b&lt;c'd</div>", "{}", out.stderr);
}

/// Script and style retain raw text and serialize it as safe CDATA.
#[test]
fn dom_html_third_review_raw_text() {
    let out = compile_and_run_capture(r###"<?php
foreach (['script', 'style'] as $tag) {
    $dom = new DOMDocument();
    $dom->loadHTML('<div><' . $tag . '>a<b&amp;</' . strtoupper($tag) . '>c</div>');
    $div = $dom->getElementsByTagName('div')->item(0);
    echo $div->nodeValue, '|', $div->childNodes->length, '|', $dom->saveXML($div), '|';
}
$dom->loadHTML('<div><script>a]]>b</script>c</div>');
$div = $dom->getElementsByTagName('div')->item(0);
echo $dom->nodeValue === null ? 'N' : 'bad', '|', $div->nodeValue, '|', $dom->saveXML($div);
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "a<b&amp;c|2|<div><script><![CDATA[a<b&amp;]]></script>c</div>|a<b&amp;c|2|<div><style><![CDATA[a<b&amp;]]></style>c</div>|N|a]]>bc|<div><script><![CDATA[a]]]]><![CDATA[>b]]></script>c</div>", "{}", out.stderr);
}

/// Invalid closing-name starts preserve the remaining text in one text node.
#[test]
fn dom_html_third_review_spaced_end_tag() {
    let out = compile_and_run_capture(r###"<?php
$dom = new DOMDocument();
foreach (['</ div>', '</1>', '</>'] as $end) {
    $dom->loadHTML('<div>a' . $end . 'b</div>');
    $div = $dom->getElementsByTagName('div')->item(0);
    echo $div->nodeValue, '|', $div->childNodes->length, '|', $dom->saveXML($div), ';';
}
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "a div>b|1|<div>a div&gt;b</div>;a1>b|1|<div>a1&gt;b</div>;a>b|1|<div>a&gt;b</div>;", "{}", out.stderr);
}

/// XML serialization escapes CR everywhere and LF/TAB inside attributes.
#[test]
fn dom_html_third_review_carriage_returns() {
    let out = compile_and_run_capture(r###"<?php
$dom = new DOMDocument();
$dom->loadHTML("<div title=\"a\r\nb\tc\">a\r\nb\tc</div>");
$div = $dom->getElementsByTagName('div')->item(0);
echo json_encode($div->nodeValue), '|', json_encode($div->getAttribute('title')), '|', $dom->saveXML($div);
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "\"a\\r\\nb\\tc\"|\"a\\r\\nb\\tc\"|<div title=\"a&#13;&#10;b&#9;c\">a&#13;\nb\tc</div>", "{}", out.stderr);
}

/// The document has no owner while its current element and text children retain the document.
#[test]
fn dom_html_third_review_document_owner() {
    let out = compile_and_run_capture(r###"<?php
$dom = new DOMDocument();
echo $dom->ownerDocument === null ? 'N' : 'bad', '|';
$dom->loadHTML('<div>Hi</div>');
$div = $dom->getElementsByTagName('div')->item(0);
$text = $div->childNodes->item(0);
echo $dom->ownerDocument === null ? 'N' : 'bad', '|';
echo $div->ownerDocument === $dom ? 'O' : 'bad', $text->ownerDocument === $dom ? 'O' : 'bad', '|';
$dom->loadHTML('<span>Bye</span>');
$span = $dom->getElementsByTagName('span')->item(0);
echo $dom->ownerDocument === null ? 'N' : 'bad', '|', $span->ownerDocument === $dom ? 'O' : 'bad';
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "N|N|OO|N|O", "{}", out.stderr);
}

/// Entity decoding is nonrecursive and preserves unknown or unterminated legacy names.
#[test]
fn dom_html_third_review_entity_decode_once() {
    let out = compile_and_run_capture(r###"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div>&amp;lt; &notAnEntity; &nbsp &AMP; &#65 &#x41 &#0; &#xD800;</div>');
$div = $dom->getElementsByTagName('div')->item(0);
echo $div->nodeValue, '|', $dom->saveXML($div);
"###);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "&lt; &notAnEntity; &nbsp &AMP; A A  |<div>&amp;lt; &amp;notAnEntity; &amp;nbsp &amp;AMP; A A  </div>", "{}", out.stderr);
}

/// Every legacy HTML4 entity plus apos matches the independent PHP/libxml 2.9 oracle.
#[test]
fn dom_html_third_review_all_named_entities() {
    let entries = r#"
AElig=198 Aacute=193 Acirc=194 Agrave=192 Alpha=913 Aring=197 Atilde=195 Auml=196 Beta=914
Ccedil=199 Chi=935 Dagger=8225 Delta=916 ETH=208 Eacute=201 Ecirc=202 Egrave=200 Epsilon=917
Eta=919 Euml=203 Gamma=915 Iacute=205 Icirc=206 Igrave=204 Iota=921 Iuml=207 Kappa=922
Lambda=923 Mu=924 Ntilde=209 Nu=925 OElig=338 Oacute=211 Ocirc=212 Ograve=210 Omega=937
Omicron=927 Oslash=216 Otilde=213 Ouml=214 Phi=934 Pi=928 Prime=8243 Psi=936 Rho=929 Scaron=352
Sigma=931 THORN=222 Tau=932 Theta=920 Uacute=218 Ucirc=219 Ugrave=217 Upsilon=933 Uuml=220
Xi=926 Yacute=221 Yuml=376 Zeta=918 aacute=225 acirc=226 acute=180 aelig=230 agrave=224
alefsym=8501 alpha=945 amp=38 and=8743 ang=8736 apos=39 aring=229 asymp=8776 atilde=227
auml=228 bdquo=8222 beta=946 brvbar=166 bull=8226 cap=8745 ccedil=231 cedil=184 cent=162
chi=967 circ=710 clubs=9827 cong=8773 copy=169 crarr=8629 cup=8746 curren=164 dArr=8659
dagger=8224 darr=8595 deg=176 delta=948 diams=9830 divide=247 eacute=233 ecirc=234 egrave=232
empty=8709 emsp=8195 ensp=8194 epsilon=949 equiv=8801 eta=951 eth=240 euml=235 euro=8364
exist=8707 fnof=402 forall=8704 frac12=189 frac14=188 frac34=190 frasl=8260 gamma=947 ge=8805
gt=62 hArr=8660 harr=8596 hearts=9829 hellip=8230 iacute=237 icirc=238 iexcl=161 igrave=236
image=8465 infin=8734 int=8747 iota=953 iquest=191 isin=8712 iuml=239 kappa=954 lArr=8656
lambda=955 lang=9001 laquo=171 larr=8592 lceil=8968 ldquo=8220 le=8804 lfloor=8970 lowast=8727
loz=9674 lrm=8206 lsaquo=8249 lsquo=8216 lt=60 macr=175 mdash=8212 micro=181 middot=183
minus=8722 mu=956 nabla=8711 nbsp=160 ndash=8211 ne=8800 ni=8715 not=172 notin=8713 nsub=8836
ntilde=241 nu=957 oacute=243 ocirc=244 oelig=339 ograve=242 oline=8254 omega=969 omicron=959
oplus=8853 or=8744 ordf=170 ordm=186 oslash=248 otilde=245 otimes=8855 ouml=246 para=182
part=8706 permil=8240 perp=8869 phi=966 pi=960 piv=982 plusmn=177 pound=163 prime=8242
prod=8719 prop=8733 psi=968 quot=34 rArr=8658 radic=8730 rang=9002 raquo=187 rarr=8594
rceil=8969 rdquo=8221 real=8476 reg=174 rfloor=8971 rho=961 rlm=8207 rsaquo=8250 rsquo=8217
sbquo=8218 scaron=353 sdot=8901 sect=167 shy=173 sigma=963 sigmaf=962 sim=8764 spades=9824
sub=8834 sube=8838 sum=8721 sup=8835 sup1=185 sup2=178 sup3=179 supe=8839 szlig=223 tau=964
there4=8756 theta=952 thetasym=977 thinsp=8201 thorn=254 tilde=732 times=215 trade=8482
uArr=8657 uacute=250 uarr=8593 ucirc=251 ugrave=249 uml=168 upsih=978 upsilon=965 uuml=252
weierp=8472 xi=958 yacute=253 yen=165 yuml=255 zeta=950 zwj=8205 zwnj=8204
"#;
    let mut html = String::from("<div>");
    let mut expected = String::new();
    for entry in entries.split_whitespace() {
        let (name, codepoint) = entry.split_once('=').unwrap();
        html.push_str(&format!("&{name};"));
        expected.push(char::from_u32(codepoint.parse().unwrap()).unwrap());
    }
    html.push_str("</div>");
    let source = format!("<?php $dom = new DOMDocument(); $dom->loadHTML('{html}'); echo $dom->getElementsByTagName('div')->item(0)->nodeValue;");
    let out = compile_and_run_capture(&source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
}

/// HTML parsing retains whitespace-only siblings even when NOBLANKS is requested.
#[test]
fn dom_html_second_review_noblanks_preserves_whitespace() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<span>a</span> <span>b</span>', LIBXML_NOERROR | LIBXML_NOBLANKS);
$body = $dom->getElementsByTagName('body')->item(0);
echo $body->nodeValue, '|', $body->childNodes->length, '|';
$dom->loadHTML("<div>\n  <span>x</span>\n</div>", LIBXML_NOERROR | LIBXML_NOBLANKS);
$div = $dom->getElementsByTagName('div')->item(0);
echo json_encode($div->nodeValue), '|', $div->childNodes->length;
"#);
    assert_eq!(out, "a b|3|\"\\n  x\\n\"|3");
}

/// Legacy libxml declaration handling preserves unknown bang markup as one text run.
#[test]
fn dom_html_second_review_unknown_declaration_is_text() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<!DOCTYPE html><div>a<!not>b</div>');
$div = $dom->getElementsByTagName('div')->item(0);
echo $div->nodeValue, '|', $div->childNodes->length, '|', $dom->saveXML($div);
"#);
    assert_eq!(out, "a<!not>b|1|<div>a&lt;!not&gt;b</div>");
}

/// XML escaping differs between text and double-quoted attribute values.
#[test]
fn dom_html_second_review_contextual_xml_escaping() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML("<div title=\"it's &quot;ok&quot; &amp; &lt;&gt;\">say \"hi\" &amp; &lt;&gt; 'yes'</div>");
$div = $dom->getElementsByTagName('div')->item(0);
echo $dom->saveXML($div);
"#);
    assert_eq!(out, "<div title=\"it's &quot;ok&quot; &amp; &lt;&gt;\">say \"hi\" &amp; &lt;&gt; 'yes'</div>");
}

/// Loading or reloading a document never changes its null nodeValue.
#[test]
fn dom_html_second_review_document_value_is_null() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
echo $dom->nodeValue === null ? 'N' : 'bad';
$dom->loadHTML('<div>Hi</div>');
echo $dom->nodeValue === null ? 'N' : 'bad';
echo '|', $dom->getElementsByTagName('html')->item(0)->nodeValue;
$dom->loadHTML('<span>Bye</span>');
echo '|', $dom->nodeValue === null ? 'N' : 'bad';
echo '|', $dom->getElementsByTagName('html')->item(0)->nodeValue;
"#);
    assert_eq!(out, "NN|Hi|N|Bye");
}

/// Slashes in unquoted values remain data, while a standalone closing slash ends the tag.
#[test]
fn dom_html_token_review_unquoted_slashes() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<a href=http://example.com title=ok>x</a><div class=foo/bar>z</div><span / title=kept>t</span><b data=x />');
$a = $dom->getElementsByTagName('a')->item(0);
$div = $dom->getElementsByTagName('div')->item(0);
$span = $dom->getElementsByTagName('span')->item(0);
$b = $dom->getElementsByTagName('b')->item(0);
echo $a->getAttribute('href'), '|', $a->getAttribute('title'), '|', $a->nodeValue, '|',
    $div->getAttribute('class'), '|', $div->nodeValue, '|', $span->getAttribute('title'), '|',
    $span->nodeValue, '|', $dom->saveXML($b);
"#);
    assert_eq!(out, "http://example.com|ok|x|foo/bar|z|kept|t|<b data=\"x\"/>");
}

/// A less-than sign followed by a non-letter stays text rather than becoming a tag.
#[test]
fn dom_html_token_review_nonletter_start() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div>1<2</div><span>x<_y</span><b>3<-4</b>');
foreach (['div', 'span', 'b'] as $name) {
    $node = $dom->getElementsByTagName($name)->item(0);
    echo $node->nodeValue, '|', $node->childNodes->length, '|', $dom->saveXML($node), ';';
}
"#);
    assert_eq!(out, "1<2|1|<div>1&lt;2</div>;x<_y|1|<span>x&lt;_y</span>;3<-4|1|<b>3&lt;-4</b>;");
}

/// Libxml's HTML boolean names use their name for an omitted value, unlike other attributes.
#[test]
fn dom_html_token_review_boolean_attributes() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<input CHECKED compact declare defer disabled ismap multiple nohref noresize noshade nowrap readonly selected hidden required title><input disabled="" checked=no>');
$input = $dom->getElementsByTagName('input')->item(0);
foreach (['checked', 'compact', 'declare', 'defer', 'disabled', 'ismap', 'multiple', 'nohref', 'noresize', 'noshade', 'nowrap', 'readonly', 'selected', 'hidden', 'required', 'title'] as $name) {
    echo $name, '=', $input->getAttribute($name), ';';
}
$explicit = $dom->getElementsByTagName('input')->item(1);
echo '|', $explicit->getAttribute('disabled'), '|', $explicit->getAttribute('checked');
"#);
    assert_eq!(out, "checked=checked;compact=compact;declare=declare;defer=defer;disabled=disabled;ismap=ismap;multiple=multiple;nohref=nohref;noresize=noresize;noshade=noshade;nowrap=nowrap;readonly=readonly;selected=selected;hidden=;required=;title=;||no");
}

/// XML serialization uses empty-element syntax for ordinary elements as well as void tags.
#[test]
fn dom_html_token_review_empty_elements() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div class="a"></div><span></span><p> </p><i><!--comment--></i><br>');
$body = $dom->getElementsByTagName('body')->item(0);
foreach ($body->childNodes as $node) { echo $dom->saveXML($node); }
"#);
    assert_eq!(out, "<div class=\"a\"/><span/><p> </p><i><!--comment--></i><br/>");
}

/// A generic function's bound keeps the namespace ownership of a user-defined DOM class.
#[test]
fn dom_html_review_generic_function_bound_uses_namespace() {
    let out = compile_and_run(r#"<?php
namespace App;
class DOMDocument {}
function identity<T: DOMDocument>(T $value): T { return $value; }
$document = identity(new DOMDocument());
echo get_class($document);
"#);
    assert_eq!(out, "App\\DOMDocument");
}

/// A DOM reference carried only by inherited generic arguments still enables dynamic creation.
#[test]
fn dom_html_review_generic_inheritance_injects_prelude() {
    let out = compile_and_run(r#"<?php
class Box<T> {}
class Documents extends Box<DOMDocument> {}
$name = 'DOMDocument';
$document = new $name();
echo $document->nodeName;
"#);
    assert_eq!(out, "#document");
}

/// A user's generic DOM class remains user-owned through explicit instantiation.
#[test]
fn dom_html_review_user_generic_class_is_not_redeclared() {
    let out = compile_and_run(r#"<?php
class DOMDocument<T> {
    public function __construct(public T $value) {}
}
$document = new DOMDocument<int>(42);
echo $document->value;
"#);
    assert_eq!(out, "42");
}

/// The user-facing example exercises sibling stack reuse and comment-free aggregate text.
#[test]
fn dom_html_review_example() {
    let out = compile_and_run(include_str!("../../examples/dom-html/main.php"));
    assert_eq!(out, "div class=text-green-500 text=Hi there!\n");
}

/// A global user-owned DOM class keeps its definition even under case-folded references.
#[test]
fn dom_html_review_user_class_is_not_redeclared() {
    let out = compile_and_run(r#"<?php
class DOMDocument { public function marker(): string { return 'user'; } }
$doc = new \domdocument();
echo $doc->marker();
"#);
    assert_eq!(out, "user");
}

/// A namespaced user class and the real global DOM surface can coexist through imports.
#[test]
fn dom_html_review_namespaced_user_class_and_global_prelude() {
    let out = compile_and_run(r#"<?php
namespace App {
    class DOMDocument { public function marker(): string { return 'app'; } }
}
namespace Client {
    use App\DOMDocument as UserDoc;
    use \DOMDocument as HtmlDoc;
    $user = new UserDoc();
    $html = new HtmlDoc();
    $html->loadHTML('<div>html</div>');
    echo $user->marker(), ':', $html->getElementsByTagName('div')->item(0)->nodeValue;
}
"#);
    assert_eq!(out, "app:html");
}

/// Unmatched closing tags are ignored and a matching ancestor closes its descendants.
#[test]
fn dom_html_review_closing_tags_match_open_elements() {
    let out = compile_and_run_capture(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div>A</missing>B<span>C</div>D<b>E</b></span>');
$body = $dom->getElementsByTagName('body')->item(0);
$div = $dom->getElementsByTagName('div')->item(0);
echo $dom->saveXML($body), '|', $div->nodeValue, '|', $body->childNodes->length;
"#);
    assert!(out.success, "{}", out.stderr.lines().take(4).collect::<Vec<_>>().join("\n"));
    assert_eq!(out.stdout, "<body><div>AB<span>C</span></div>D<b>E</b></body>|ABC|3");
    assert!(out.stderr.is_empty(), "{}", out.stderr.lines().take(4).collect::<Vec<_>>().join("\n"));
}

/// Comments remain visible as nodes and in XML but not in ancestor descendant text.
#[test]
fn dom_html_review_comments_do_not_aggregate_into_element_text() {
    let out = compile_and_run(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div>A<!--hidden--><span>C<!--nested--></span>B</div>');
$div = $dom->getElementsByTagName('div')->item(0);
$span = $dom->getElementsByTagName('span')->item(0);
$comment = $div->childNodes->item(1);
echo $div->nodeValue, '|', $span->nodeValue, '|', $comment->nodeValue, '|', $dom->saveXML($div);
"#);
    assert_eq!(out, "ACB|C|hidden|<div>A<!--hidden--><span>C<!--nested--></span>B</div>");
}

/// A dynamic NodeList loop stops after its two nodes without reading past the end.
#[test]
fn dom_html_review_iterator_stops_at_the_end() {
    let out = compile_and_run_capture(r#"<?php
$dom = new DOMDocument();
$dom->loadHTML('<div><span>A</span><b>B</b></div>');
$div = $dom->getElementsByTagName('div')->item(0);
$seen = 0;
foreach ($div->childNodes as $key => $child) {
    echo $key, ':', $child->nodeName, ':', $child->nodeValue, ';';
    $seen++;
    if ($seen > 3) { break; }
}

echo $seen;
"#);
    assert!(out.success, "{}", out.stderr.lines().take(4).collect::<Vec<_>>().join("\n"));
    assert_eq!(out.stdout, "0:span:A;1:b:B;2");
    assert!(out.stderr.is_empty(), "{}", out.stderr.lines().take(4).collect::<Vec<_>>().join("\n"));
}

/// The tree-builder stack can append back into the integer slot removed by pop.
#[test]
fn dom_html_review_stack_pop_reuses_tail_index() {
    let out = compile_and_run(r#"<?php
function replaceStackTop(array $stack): array {
    $removed = array_pop($stack);
    $stack[] = ['name' => 'b'];
    return $stack;
}
$stack = [['name' => 'div'], ['name' => 'span']];
$stack = replaceStackTop($stack);
echo count($stack), ':';
foreach ($stack as $key => $node) { echo $key, '=', $node['name'], ';'; }
"#);
    assert_eq!(out, "2:0=div;1=b;");
}

//! Purpose:
//! php-src `.phpt` conformance tests for json, migrated to Rust: each test compiles the
//! phpt `--FILE--` section and compares stdout with PHP's output.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`codegen::php_src`).
//!
//! Key details:
//! - Generated from php-src (https://github.com/php/php-src); only tests whose output elephc
//!   reproduces byte for byte (with no warnings) are kept. The expected strings are the PHP
//!   interpreter's own output, which matches the phpt `--EXPECT--` section.

use crate::support::*;

/// php-src `ext/json/tests/002.phpt`: json_encode() tests
#[test]
fn phpt_002() {
    let out = compile_and_run(r#"<?php

var_dump(json_encode(""));
var_dump(json_encode(NULL));
var_dump(json_encode(TRUE));

var_dump(json_encode(array(""=>"")));
var_dump(json_encode(array(array(1))));
var_dump(json_encode(array()));

var_dump(json_encode(array(""=>""), JSON_FORCE_OBJECT));
var_dump(json_encode(array(array(1)), JSON_FORCE_OBJECT));
var_dump(json_encode(array(), JSON_FORCE_OBJECT));

var_dump(json_encode(1));
var_dump(json_encode("руссиш"));

echo "Done\n";
?>
"#);
    assert_eq!(out, "string(2) \"\"\"\"\nstring(4) \"null\"\nstring(4) \"true\"\nstring(7) \"{\"\":\"\"}\"\nstring(5) \"[[1]]\"\nstring(2) \"[]\"\nstring(7) \"{\"\":\"\"}\"\nstring(13) \"{\"0\":{\"0\":1}}\"\nstring(2) \"{}\"\nstring(1) \"1\"\nstring(38) \"\"\\u0440\\u0443\\u0441\\u0441\\u0438\\u0448\"\"\nDone\n");
}

/// php-src `ext/json/tests/006.phpt`: json_encode() & extended encoding
#[test]
fn phpt_006() {
    let out = compile_and_run(r#"<?php
$a = array('<foo>', "'bar'", '"baz"', '&blong&');

echo "Normal: ", json_encode($a), "\n";
echo "Tags: ",   json_encode($a, JSON_HEX_TAG), "\n";
echo "Apos: ",   json_encode($a, JSON_HEX_APOS), "\n";
echo "Quot: ",   json_encode($a, JSON_HEX_QUOT), "\n";
echo "Amp: ",    json_encode($a, JSON_HEX_AMP), "\n";
echo "All: ",    json_encode($a, JSON_HEX_TAG|JSON_HEX_APOS|JSON_HEX_QUOT|JSON_HEX_AMP), "\n";
?>
"#);
    assert_eq!(out, "Normal: [\"<foo>\",\"'bar'\",\"\\\"baz\\\"\",\"&blong&\"]\nTags: [\"\\u003Cfoo\\u003E\",\"'bar'\",\"\\\"baz\\\"\",\"&blong&\"]\nApos: [\"<foo>\",\"\\u0027bar\\u0027\",\"\\\"baz\\\"\",\"&blong&\"]\nQuot: [\"<foo>\",\"'bar'\",\"\\u0022baz\\u0022\",\"&blong&\"]\nAmp: [\"<foo>\",\"'bar'\",\"\\\"baz\\\"\",\"\\u0026blong\\u0026\"]\nAll: [\"\\u003Cfoo\\u003E\",\"\\u0027bar\\u0027\",\"\\u0022baz\\u0022\",\"\\u0026blong\\u0026\"]\n");
}

/// php-src `ext/json/tests/008.phpt`: json_decode() with large integers
#[test]
fn phpt_008() {
    let out = compile_and_run(r#"<?php
$json = '{"largenum":123456789012345678901234567890}';
$x = json_decode($json);
var_dump($x->largenum);
$x = json_decode($json, false, 512, JSON_BIGINT_AS_STRING);
var_dump($x->largenum);
echo "Done\n";
?>
"#);
    assert_eq!(out, "float(1.2345678901234568E+29)\nstring(30) \"123456789012345678901234567890\"\nDone\n");
}

/// php-src `ext/json/tests/009.phpt`: json_encode() with non-packed array that should be encoded as an array rather than object
#[test]
fn phpt_009() {
    let out = compile_and_run(r#"<?php
$a = array(1, 2, 3, 'foo' => 'bar');
unset($a['foo']);

var_dump(json_encode($a));
echo "Done\n";
?>
"#);
    assert_eq!(out, "string(7) \"[1,2,3]\"\nDone\n");
}

/// php-src `ext/json/tests/bug41034.phpt`: Bug #41034 (json_encode() ignores null byte started keys in arrays)
#[test]
fn phpt_bug41034() {
    let out = compile_and_run(r#"<?php
echo json_encode(array(0, "\0ab"=>1, "\0null-prefixed value"));
echo "\nDone\n";
?>
"#);
    assert_eq!(out, "{\"0\":0,\"\\u0000ab\":1,\"1\":\"\\u0000null-prefixed value\"}\nDone\n");
}

/// php-src `ext/json/tests/bug41067.phpt`: Bug #41067 (json_encode() problem with UTF-16 input)
#[test]
fn phpt_bug41067() {
    let out = compile_and_run(r#"<?php
$single_barline = "\360\235\204\200";
$array = array($single_barline);
print bin2hex($single_barline) . "\n";
// print $single_barline . "\n\n";
$json = json_encode($array);
print $json . "\n\n";
$json_decoded = json_decode($json, true);
// print $json_decoded[0] . "\n";
print bin2hex($json_decoded[0]) . "\n";
print "END\n";
?>
"#);
    assert_eq!(out, "f09d8480\n[\"\\ud834\\udd00\"]\n\nf09d8480\nEND\n");
}

/// php-src `ext/json/tests/bug41504.phpt`: Bug #41504 (json_decode() converts empty array keys to "_empty_")
#[test]
fn phpt_bug41504() {
    let out = compile_and_run(r#"<?php

var_dump(json_decode('{"":"value"}', true));
var_dump(json_decode('{"":"value", "key":"value"}', true));
var_dump(json_decode('{"key":"value", "":"value"}', true));

echo "Done\n";
?>
"#);
    assert_eq!(out, "array(1) {\n  [\"\"]=>\n  string(5) \"value\"\n}\narray(2) {\n  [\"\"]=>\n  string(5) \"value\"\n  [\"key\"]=>\n  string(5) \"value\"\n}\narray(2) {\n  [\"key\"]=>\n  string(5) \"value\"\n  [\"\"]=>\n  string(5) \"value\"\n}\nDone\n");
}

/// php-src `ext/json/tests/bug42090.phpt`: Bug #42090 (json_decode causes segmentation fault)
#[test]
fn phpt_bug42090() {
    let out = compile_and_run(r#"<?php
var_dump(
    json_decode('""'),
    json_decode('"..".'),
    json_decode('"'),
    json_decode('""""'),
    json_encode('"'),
    json_decode(json_encode('"')),
    json_decode(json_encode('""'))
);
?>
"#);
    assert_eq!(out, "string(0) \"\"\nNULL\nNULL\nNULL\nstring(4) \"\"\\\"\"\"\nstring(1) \"\"\"\nstring(2) \"\"\"\"\n");
}

/// php-src `ext/json/tests/bug46944.phpt`: Bug #46944 (json_encode() doesn't handle 3 byte utf8 correctly)
#[test]
fn phpt_bug46944() {
    let out = compile_and_run(r#"<?php

for ($i = 1; $i <= 16; $i++) {
    $first = 0xf0 | ($i >> 2);
    $second = 0x8f | ($i & 3) << 4;
    $string = sprintf("aa%c%c\xbf\xbdzz", $first, $second);
    echo json_encode($string) . "\n";
}


echo "Done\n";
?>
"#);
    assert_eq!(out, "\"aa\\ud83f\\udffdzz\"\n\"aa\\ud87f\\udffdzz\"\n\"aa\\ud8bf\\udffdzz\"\n\"aa\\ud8ff\\udffdzz\"\n\"aa\\ud93f\\udffdzz\"\n\"aa\\ud97f\\udffdzz\"\n\"aa\\ud9bf\\udffdzz\"\n\"aa\\ud9ff\\udffdzz\"\n\"aa\\uda3f\\udffdzz\"\n\"aa\\uda7f\\udffdzz\"\n\"aa\\udabf\\udffdzz\"\n\"aa\\udaff\\udffdzz\"\n\"aa\\udb3f\\udffdzz\"\n\"aa\\udb7f\\udffdzz\"\n\"aa\\udbbf\\udffdzz\"\n\"aa\\udbff\\udffdzz\"\nDone\n");
}

/// php-src `ext/json/tests/bug54484.phpt`: Bug #54484 (Empty string in json_decode doesn't reset json_last_error)
#[test]
fn phpt_bug54484() {
    let out = compile_and_run(r#"<?php
json_decode('{"test":"test"}');
var_dump(json_last_error());

json_decode("");
var_dump(json_last_error());

json_decode("invalid json");
var_dump(json_last_error());

json_decode("\"\001 invalid json\"");
var_dump(json_last_error());

json_decode("");
var_dump(json_last_error());
?>
"#);
    assert_eq!(out, "int(0)\nint(4)\nint(4)\nint(3)\nint(4)\n");
}

/// php-src `ext/json/tests/bug62369.phpt`: FR #62369 (Segfault on json_encode(deeply_nested_array)
#[test]
fn phpt_bug62369() {
    let out = compile_and_run(r#"<?php

$array = array();
for ($i=0; $i < 550; $i++) {
    $array = array($array);
}

json_encode($array, 0, 551);
switch (json_last_error()) {
    case JSON_ERROR_NONE:
        echo 'OK' . PHP_EOL;
    break;
    case JSON_ERROR_DEPTH:
        echo 'ERROR' . PHP_EOL;
    break;
}

json_encode($array, 0, 540);
switch (json_last_error()) {
    case JSON_ERROR_NONE:
        echo 'OK' . PHP_EOL;
    break;
    case JSON_ERROR_DEPTH:
        echo 'ERROR' . PHP_EOL;
    break;
}
?>
"#);
    assert_eq!(out, "OK\nERROR\n");
}

/// php-src `ext/json/tests/bug63737.phpt`: Bug #63737 (json_decode does not properly decode with options parameter)
#[test]
fn phpt_bug63737() {
    let out = compile_and_run(r#"<?php
function decode($json) {
    $x = json_decode($json);
    var_dump($x);
    $x = json_decode($json, false, 512, JSON_BIGINT_AS_STRING);
    var_dump($x);
}

decode('123456789012345678901234567890');
decode('-123456789012345678901234567890');

// This shouldn't affect floats, but let's check that.
decode('123456789012345678901234567890.1');
decode('-123456789012345678901234567890.1');

echo "Done\n";
?>
"#);
    assert_eq!(out, "float(1.2345678901234568E+29)\nstring(30) \"123456789012345678901234567890\"\nfloat(-1.2345678901234568E+29)\nstring(31) \"-123456789012345678901234567890\"\nfloat(1.2345678901234568E+29)\nfloat(1.2345678901234568E+29)\nfloat(-1.2345678901234568E+29)\nfloat(-1.2345678901234568E+29)\nDone\n");
}

/// php-src `ext/json/tests/bug64874_part1.phpt`: Whitespace part of bug #64874 ("json_decode handles whitespace and case-sensitivity incorrectly")
#[test]
fn phpt_bug64874_part1() {
    let out = compile_and_run(r#"<?php
function decode($json) {
    var_dump(json_decode($json));
    var_dump(json_last_error() !== 0);
    echo "\n";
}

// Leading whitespace should be ignored
decode(" true");
decode("\ttrue");
decode("\ntrue");
decode("\rtrue");

// So should trailing whitespace
decode("true ");
decode("true\t");
decode("true\n");
decode("true\r");

// And so should the combination of both
decode(" true ");
decode(" true\t");
decode(" true\n");
decode(" true\r");
decode("\ttrue ");
decode("\ttrue\t");
decode("\ttrue\n");
decode("\ttrue\r");
decode("\ntrue ");
decode("\ntrue\t");
decode("\ntrue\n");
decode("\ntrue\r");
decode("\rtrue ");
decode("\rtrue\t");
decode("\rtrue\n");
decode("\rtrue\r");

echo "Done\n";
?>
"#);
    assert_eq!(out, "bool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nbool(true)\nbool(false)\n\nDone\n");
}

/// php-src `ext/json/tests/bug64874_part2.phpt`: Case-sensitivity part of bug #64874 ("json_decode handles whitespace and case-sensitivity incorrectly")
#[test]
fn phpt_bug64874_part2() {
    let out = compile_and_run(r#"<?php
function decode($json) {
    var_dump(json_decode($json));
    echo ((json_last_error() !== 0) ? 'ERROR' : 'SUCCESS') . PHP_EOL;
}

// Only lowercase should work
decode('true');
decode('True');
decode('[true]');
decode('[True]');
echo PHP_EOL;

decode('false');
decode('False');
decode('[false]');
decode('[False]');
echo PHP_EOL;

decode('null');
decode('Null');
decode('[null]');
decode('[Null]');
echo PHP_EOL;

echo "Done\n";
?>
"#);
    assert_eq!(out, "bool(true)\nSUCCESS\nNULL\nERROR\narray(1) {\n  [0]=>\n  bool(true)\n}\nSUCCESS\nNULL\nERROR\n\nbool(false)\nSUCCESS\nNULL\nERROR\narray(1) {\n  [0]=>\n  bool(false)\n}\nSUCCESS\nNULL\nERROR\n\nNULL\nSUCCESS\nNULL\nERROR\narray(1) {\n  [0]=>\n  NULL\n}\nSUCCESS\nNULL\nERROR\n\nDone\n");
}

/// php-src `ext/json/tests/bug66025.phpt`: Bug #66025 (Indent wrong when json_encode() called from jsonSerialize function)
#[test]
fn phpt_bug66025() {
    let out = compile_and_run(r#"<?php

class Foo implements JsonSerializable {
    public function jsonSerialize(): mixed {
        return json_encode([1], JSON_PRETTY_PRINT);
    }
}

echo json_encode([new Foo]), "\n";
?>
"#);
    assert_eq!(out, "[\"[\\n    1\\n]\"]\n");
}

/// php-src `ext/json/tests/bug68567.phpt`: Bug #68567 JSON_PARTIAL_OUTPUT_ON_ERROR can result in JSON with null key
#[test]
fn phpt_bug68567() {
    let out = compile_and_run(r#"<?php

var_dump(json_encode(array("\x80" => 1), JSON_PARTIAL_OUTPUT_ON_ERROR));

?>
"#);
    assert_eq!(out, "string(6) \"{\"\":1}\"\n");
}

/// php-src `ext/json/tests/bug68817.phpt`: Bug #68817 (Null pointer deference)
#[test]
fn phpt_bug68817() {
    let out = compile_and_run(r#"<?php

var_dump(json_decode('[""]'));

?>
"#);
    assert_eq!(out, "array(1) {\n  [0]=>\n  string(0) \"\"\n}\n");
}

/// php-src `ext/json/tests/bug68938.phpt`: Bug #68938 (json_decode() decodes empty string without indicating error)
#[test]
fn phpt_bug68938() {
    let out = compile_and_run(r#"<?php
json_decode("");
var_dump(json_last_error());
?>
"#);
    assert_eq!(out, "int(4)\n");
}

/// php-src `ext/json/tests/bug68992.phpt`: Bug #68992 (json_encode stacks exceptions thrown by JsonSerializable classes)
#[test]
fn phpt_bug68992() {
    let out = compile_and_run(r#"<?php

class MyClass implements JsonSerializable {
    public function jsonSerialize(): mixed {
        throw new Exception('Not implemented!');
    }
}
$classes = [];
for($i = 0; $i < 5; $i++) {
    $classes[] = new MyClass();
}

try {
    json_encode($classes);
} catch(Exception $e) {
    do {
        printf("%s (%d) [%s]\n", $e->getMessage(), $e->getCode(), get_class($e));
    } while ($e = $e->getPrevious());
}
?>
"#);
    assert_eq!(out, "Not implemented! (0) [Exception]\n");
}

/// php-src `ext/json/tests/bug73113.phpt`: Bug #73113 (Segfault with throwing JsonSerializable)
#[test]
fn phpt_bug73113() {
    let out = compile_and_run(r#"<?php

class JsonSerializableObject implements \JsonSerializable
{
    public function jsonSerialize(): mixed
    {
        throw new \Exception('This error is expected');
    }
}

$obj = new JsonSerializableObject();
try {
    echo json_encode($obj);
} catch (\Throwable $e) {
    echo $e::class, ': ', $e->getMessage(), "\n";
}
?>
"#);
    assert_eq!(out, "Exception: This error is expected\n");
}

/// php-src `ext/json/tests/fail001.phpt`: JSON (http://www.crockford.com/JSON/JSON_checker/test/fail*.json)
#[test]
fn phpt_fail001() {
    let out = compile_and_run(r#"<?php

$tests = array(
    '"A JSON payload should be an object or array, not a string."',
    '["Unclosed array"',
    '{unquoted_key: "keys must be quoted}',
    '["extra comma",]',
    '["double extra comma",,]',
    '[   , "<-- missing value"]',
    '["Comma after the close"],',
    '["Extra close"]]',
    '{"Extra comma": true,}',
    '{"Extra value after close": true} "misplaced quoted value"',
    '{"Illegal expression": 1 + 2}',
    '{"Illegal invocation": alert()}',
    '{"Numbers cannot have leading zeroes": 013}',
    '{"Numbers cannot be hex": 0x14}',
    '["Illegal backslash escape: \\x15"]',
    '["Illegal backslash escape: \\\'"]',
    '["Illegal backslash escape: \\017"]',
    '[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[["Too deep"]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]',
    '{"Missing colon" null}',
    '{"Double colon":: null}',
    '{"Comma instead of colon", null}',
    '["Colon instead of comma": false]',
    '["Bad value", truth]',
    "['single quote']"
);

foreach ($tests as $test) {
    echo 'Testing: ' . $test . "\n";
    echo "AS OBJECT\n";
    var_dump(json_decode($test));
    echo "AS ARRAY\n";
    var_dump(json_decode($test, true));
}

?>
"#);
    assert_eq!(out, "Testing: \"A JSON payload should be an object or array, not a string.\"\nAS OBJECT\nstring(58) \"A JSON payload should be an object or array, not a string.\"\nAS ARRAY\nstring(58) \"A JSON payload should be an object or array, not a string.\"\nTesting: [\"Unclosed array\"\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {unquoted_key: \"keys must be quoted}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"extra comma\",]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"double extra comma\",,]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [   , \"<-- missing value\"]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Comma after the close\"],\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Extra close\"]]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Extra comma\": true,}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Extra value after close\": true} \"misplaced quoted value\"\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Illegal expression\": 1 + 2}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Illegal invocation\": alert()}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Numbers cannot have leading zeroes\": 013}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Numbers cannot be hex\": 0x14}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Illegal backslash escape: \\x15\"]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Illegal backslash escape: \\'\"]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Illegal backslash escape: \\017\"]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[\"Too deep\"]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Missing colon\" null}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Double colon\":: null}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: {\"Comma instead of colon\", null}\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Colon instead of comma\": false]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: [\"Bad value\", truth]\nAS OBJECT\nNULL\nAS ARRAY\nNULL\nTesting: ['single quote']\nAS OBJECT\nNULL\nAS ARRAY\nNULL\n");
}

/// php-src `ext/json/tests/inf_nan_error.phpt`: An error is thrown when INF or NaN are encoded
#[test]
fn phpt_inf_nan_error() {
    let out = compile_and_run(r#"<?php

$inf = INF;

var_dump($inf);

var_dump(json_encode($inf));
var_dump(json_last_error(), json_last_error_msg());

var_dump(json_encode($inf, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_last_error(), json_last_error_msg());

echo "\n";

$nan = NAN;

var_dump($nan);

var_dump(json_encode($nan));
var_dump(json_last_error(), json_last_error_msg());

var_dump(json_encode($nan, JSON_PARTIAL_OUTPUT_ON_ERROR));
var_dump(json_last_error(), json_last_error_msg());
?>
"#);
    assert_eq!(out, "float(INF)\nbool(false)\nint(7)\nstring(34) \"Inf and NaN cannot be JSON encoded\"\nstring(1) \"0\"\nint(7)\nstring(34) \"Inf and NaN cannot be JSON encoded\"\n\nfloat(NAN)\nbool(false)\nint(7)\nstring(34) \"Inf and NaN cannot be JSON encoded\"\nstring(1) \"0\"\nint(7)\nstring(34) \"Inf and NaN cannot be JSON encoded\"\n");
}

/// php-src `ext/json/tests/json_encode_basic_utf8.phpt`: Test json_encode() function : basic functionality with UTF8 string input
#[test]
fn phpt_json_encode_basic_utf8() {
    let out = compile_and_run(r#"<?php
echo "*** Testing json_encode() : basic functionality with UTF-8 input***\n";

$utf8_string = base64_decode('5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=');
var_dump(json_encode($utf8_string));

?>
"#);
    assert_eq!(out, "*** Testing json_encode() : basic functionality with UTF-8 input***\nstring(103) \"\"\\u65e5\\u672c\\u8a9e\\u30c6\\u30ad\\u30b9\\u30c8\\u3067\\u3059\\u300201234\\uff15\\uff16\\uff17\\uff18\\uff19\\u3002\"\"\n");
}

/// php-src `ext/json/tests/json_encode_pretty_print.phpt`: json_encode() with JSON_PRETTY_PRINT
#[test]
fn phpt_json_encode_pretty_print() {
    let out = compile_and_run(r#"<?php
function encode_decode($json) {
    $struct = json_decode($json);
    $pretty = json_encode($struct, JSON_PRETTY_PRINT);
    echo "$pretty\n";
    $pretty = json_decode($pretty);
    printf("Match: %d\n", $pretty == $struct);
}

encode_decode('[1,2,3,[1,2,3]]');
encode_decode('{"a":1,"b":[1,2],"c":{"d":42}}');
?>
"#);
    assert_eq!(out, "[\n    1,\n    2,\n    3,\n    [\n        1,\n        2,\n        3\n    ]\n]\nMatch: 1\n{\n    \"a\": 1,\n    \"b\": [\n        1,\n        2\n    ],\n    \"c\": {\n        \"d\": 42\n    }\n}\nMatch: 1\n");
}

/// php-src `ext/json/tests/json_encode_unescaped_slashes.phpt`: json_decode() tests
#[test]
fn phpt_json_encode_unescaped_slashes() {
    let out = compile_and_run(r#"<?php
var_dump(json_encode('a/b'));
var_dump(json_encode('a/b', JSON_UNESCAPED_SLASHES));
?>
"#);
    assert_eq!(out, "string(6) \"\"a\\/b\"\"\nstring(5) \"\"a/b\"\"\n");
}

/// php-src `ext/json/tests/json_validate_001.phpt`: json_validate() - General usage
#[test]
fn phpt_json_validate_001() {
    let out = compile_and_run(r#"<?php

var_dump(
  json_validate(""),
  json_validate("."),
  json_validate("<?>"),
  json_validate(";"),
  json_validate("руссиш"),
  json_validate("blah"),
  json_validate('{ "": "": "" } }'),
  json_validate('{ "": { "": "" }'),
  json_validate('{ "test": {} "foo": "bar" }, "test2": {"foo" : "bar" }, "test2": {"foo" : "bar" } }'),

  json_validate('{ "test": { "foo": "bar" } }'),
  json_validate('{ "test": { "foo": "" } }'),
  json_validate('{ "": { "foo": "" } }'),
  json_validate('{ "": { "": "" } }'),
  json_validate('{ "test": {"foo": "bar"}, "test2": {"foo" : "bar" }, "test2": {"foo" : "bar" } }'),
  json_validate('{ "test": {"foo": "bar"}, "test2": {"foo" : "bar" }, "test3": {"foo" : "bar" } }'),
);

?>
"#);
    assert_eq!(out, "bool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n");
}

//! Purpose:
//! php-src `.phpt` conformance tests for strings, migrated to Rust: each test compiles the
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

/// php-src `ext/standard/tests/strings/add-and-stripcslashes.phpt`: addcslashes() and stripcslashes() function
#[test]
fn phpt_add_and_stripcslashes() {
    let out = compile_and_run(r#"<?php
echo addcslashes("", "")."\n";
echo addcslashes("", "burp")."\n";
echo addcslashes("kaboemkara!", "")."\n";
echo addcslashes("foobarbaz", 'bar')."\n";
echo addcslashes('foo[ ]', 'A..z')."\n";
echo @addcslashes("zoo['.']", 'z..A')."\n";
echo addcslashes('abcdefghijklmnopqrstuvwxyz', "a\145..\160z")."\n";
echo "\n\r" == stripcslashes('\n\r'),"\n";
echo stripcslashes('\065\x64')."\n";
echo stripcslashes('')."\n";
?>
"#);
    assert_eq!(out, "\n\nkaboemkara!\nfoo\\b\\a\\r\\b\\az\n\\f\\o\\o\\[ \\]\n\\zoo['\\.']\n\\abcd\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\pqrstuvwxy\\z\n1\n5d\n\n");
}

/// php-src `ext/standard/tests/strings/add-and-stripslashes.phpt`: addslashes() and stripslashes() functions
#[test]
fn phpt_add_and_stripslashes() {
    let out = compile_and_run(r#"<?php

$input = '';
for($i=0; $i<512; $i++) {
    $input .= chr($i%256);
}

echo "Normal: ";
if($input === stripslashes(addslashes($input))) {
    echo "OK\n";
} else {
    echo "FAILED\n";
}

?>
"#);
    assert_eq!(out, "Normal: OK\n");
}

/// php-src `ext/standard/tests/strings/basename.phpt`: basename() function
#[test]
fn phpt_basename() {
    let out = compile_and_run(r#"<?php
    // simple checks
    var_dump(basename("bar"));
    var_dump(basename("/foo/bar"));
    var_dump(basename("/bar"));

    // simple checks with trailing slashes
    var_dump(basename("bar/"));
    var_dump(basename("/foo/bar/"));
    var_dump(basename("/bar/"));

    // suffix removal checks
    var_dump(basename("bar.gz", ".gz"));
    var_dump(basename("/foo/bar.gz", ".gz"));
    var_dump(basename("/bar.gz", ".gz"));

    // suffix removal checks with trailing slashes
    var_dump(basename("bar.gz/", ".gz"));
    var_dump(basename("/foo/bar.gz/", ".gz"));
    var_dump(basename("/bar.gz/", ".gz"));

    // suffix removal checks
    var_dump(basename("/.gz", ".gz"));
    var_dump(basename("/foo/.gz", ".gz"));
    var_dump(basename("/.gz", ".gz"));

    // binary safe?
    var_dump(basename("foo".chr(0)."bar"));
    var_dump(basename("foo".chr(0)."bar.gz", ".gz"));
?>
"#);
    assert_eq!(out, "string(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \"bar\"\nstring(3) \".gz\"\nstring(3) \".gz\"\nstring(3) \".gz\"\nstring(7) \"foo\u{0}bar\"\nstring(7) \"foo\u{0}bar\"\n");
}

/// php-src `ext/standard/tests/strings/basename_basic.phpt`: Test basename() function : basic functionality
#[test]
fn phpt_basename_basic() {
    let out = compile_and_run(r#"<?php
$file_paths = array (
  /* simple paths */
  array("bar"),
  array("/foo/bar"),
  array("foo/bar"),
  array("/bar"),

  /* simple paths with trailing slashes */
  array("bar/"),
  array("/bar/"),
  array("/foo/bar/"),
  array("foo/bar/"),
  array("/bar/"),

  /* paths with suffix removal */
  array("bar.gz", ".gz"),
  array("bar.gz", "bar.gz"),
  array("/foo/bar.gz", ".gz"),
  array("foo/bar.gz", ".gz"),
  array("/bar.gz", ".gz"),

  /* paths with suffix and trailing slashes with suffix removal*/
  array("bar.gz/", ".gz"),
  array("/bar.gz/", ".gz"),
  array("/foo/bar.gz/", ".gz"),
  array("foo/bar.gz/", ".gz"),
  array("/bar.gz/", ".gz"),

  /* paths with basename only suffix, with suffix removal*/
  array("/.gz", ".gz"),
  array(".gz", ".gz"),
  array("/foo/.gz", ".gz"),

  /* paths with basename only suffix & trailing slashes, with suffix removal*/
  array(".gz/", ".gz"),
  array("/foo/.gz/", ".gz"),
  array("foo/.gz/", ".gz"),

  /* paths with binary value to check if the function is binary safe*/
  array("foo".chr(0)."bar"),
  array("/foo".chr(0)."bar"),
  array("/foo".chr(0)."bar/"),
  array("foo".chr(0)."bar/"),
  array("foo".chr(0)."bar/test"),
  array("/foo".chr(0)."bar/bar.gz", ".gz"),
  array("/foo".chr(0)."bar/bar.gz")
);

function check_basename( $path_arrays ) {
   $loop_counter = 1;
   foreach ($path_arrays as $path) {
     echo "\n--Iteration $loop_counter--\n"; $loop_counter++;
     if( 1 == count($path) ) { // no suffix provided
       var_dump( basename($path[0]) );
     } else { // path as well as suffix provided,
       var_dump( basename($path[0], $path[1]) );
     }
   }
}

echo "*** Testing basic operations ***\n";
check_basename( $file_paths );

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing basic operations ***\n\n--Iteration 1--\nstring(3) \"bar\"\n\n--Iteration 2--\nstring(3) \"bar\"\n\n--Iteration 3--\nstring(3) \"bar\"\n\n--Iteration 4--\nstring(3) \"bar\"\n\n--Iteration 5--\nstring(3) \"bar\"\n\n--Iteration 6--\nstring(3) \"bar\"\n\n--Iteration 7--\nstring(3) \"bar\"\n\n--Iteration 8--\nstring(3) \"bar\"\n\n--Iteration 9--\nstring(3) \"bar\"\n\n--Iteration 10--\nstring(3) \"bar\"\n\n--Iteration 11--\nstring(6) \"bar.gz\"\n\n--Iteration 12--\nstring(3) \"bar\"\n\n--Iteration 13--\nstring(3) \"bar\"\n\n--Iteration 14--\nstring(3) \"bar\"\n\n--Iteration 15--\nstring(3) \"bar\"\n\n--Iteration 16--\nstring(3) \"bar\"\n\n--Iteration 17--\nstring(3) \"bar\"\n\n--Iteration 18--\nstring(3) \"bar\"\n\n--Iteration 19--\nstring(3) \"bar\"\n\n--Iteration 20--\nstring(3) \".gz\"\n\n--Iteration 21--\nstring(3) \".gz\"\n\n--Iteration 22--\nstring(3) \".gz\"\n\n--Iteration 23--\nstring(3) \".gz\"\n\n--Iteration 24--\nstring(3) \".gz\"\n\n--Iteration 25--\nstring(3) \".gz\"\n\n--Iteration 26--\nstring(7) \"foo\u{0}bar\"\n\n--Iteration 27--\nstring(7) \"foo\u{0}bar\"\n\n--Iteration 28--\nstring(7) \"foo\u{0}bar\"\n\n--Iteration 29--\nstring(7) \"foo\u{0}bar\"\n\n--Iteration 30--\nstring(4) \"test\"\n\n--Iteration 31--\nstring(3) \"bar\"\n\n--Iteration 32--\nstring(6) \"bar.gz\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/basename_variation.phpt`: Test basename() function : usage variations
#[test]
fn phpt_basename_variation() {
    let out = compile_and_run(r#"<?php
$file_path_variations = array (
  /* paths with shortcut home dir char, with suffix variation */
  array("~/home/user/bar"),
  array("~/home/user/bar", ""),
  array("~/home/user/bar", ' '),
  array("~/home/user/bar.tar", ".tar"),
  array("~/home/user/bar.tar", "~"),
  array("~/home/user/bar.tar/", "~"),
  array("~/home/user/bar.tar/", ""),
  array("~/home/user/bar.tar", ''),
  array("~/home/user/bar.tar", " "),

  /* paths with hostname:dir notation, with suffix variation */
  array("hostname:/home/usr/bar.tar"),
  array("hostname:/home/user/bar.tar", "home"),
  array("hostname:/home/user/tbar.gz", "bar.gz"),
  array("hostname:/home/user/tbar.gz", "/bar.gz"),
  array("hostname:/home/user/tbar.gz", "/bar.gz/"),
  array("hostname:/home/user/tbar.gz/", "/bar.gz/"),
  array("hostname:/home/user/tbar.gz/", "/bar.gz/"),
  array("hostname:/home/user/My Pics.gz/", "/bar.gz/"),
  array("hostname:/home/user/My Pics.gz/", "Pics.gz/"),
  array("hostname:/home/user/My Pics.gz/", "Pics.gz"),
  array("hostname:/home/user/My Pics.gz/", ".gz"),
  array("hostname:/home/user/My Pics.gz/"),
  array("hostname:/home/user/My Pics.gz/", ' '),
  array("hostname:/home/user/My Pics.gz/", ''),
  array("hostname:/home/user/My Pics.gz/", "My Pics.gz"),

  /* paths with numeirc strings */
  array("10.5"),
  array("10.5", ".5"),
  array("10.5", "10.5"),
  array("10"),
  array("105", "5"),
  array("/10.5"),
  array("10.5/"),
  array("10/10.gz"),
  array("0"),
  array('0'),

  /* paths and suffix given as same */
  array("bar.gz", "bar.gz"),
  array("/bar.gz", "/bar.gz"),
  array("/bar.gz/", "/bar.gz/"),
  array(" ", " "),
  array(' ', ' '),

  /* path with spaces */
  array(" "),
  array(' '),

  /* empty paths */
  array(""),
  array(''),
);

function check_basename( $path_arrays ) {
   $loop_counter = 1;
   foreach ($path_arrays as $path) {
     echo "\n--Iteration $loop_counter--\n"; $loop_counter++;
     if( 1 == count($path) ) { // no suffix provided
       var_dump( basename($path[0]) );
     } else { // path as well as suffix provided,
       var_dump( basename($path[0], $path[1]) );
     }
   }
}

echo "*** Testing possible variations in path and suffix ***\n";
check_basename( $file_path_variations );

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing possible variations in path and suffix ***\n\n--Iteration 1--\nstring(3) \"bar\"\n\n--Iteration 2--\nstring(3) \"bar\"\n\n--Iteration 3--\nstring(3) \"bar\"\n\n--Iteration 4--\nstring(3) \"bar\"\n\n--Iteration 5--\nstring(7) \"bar.tar\"\n\n--Iteration 6--\nstring(7) \"bar.tar\"\n\n--Iteration 7--\nstring(7) \"bar.tar\"\n\n--Iteration 8--\nstring(7) \"bar.tar\"\n\n--Iteration 9--\nstring(7) \"bar.tar\"\n\n--Iteration 10--\nstring(7) \"bar.tar\"\n\n--Iteration 11--\nstring(7) \"bar.tar\"\n\n--Iteration 12--\nstring(1) \"t\"\n\n--Iteration 13--\nstring(7) \"tbar.gz\"\n\n--Iteration 14--\nstring(7) \"tbar.gz\"\n\n--Iteration 15--\nstring(7) \"tbar.gz\"\n\n--Iteration 16--\nstring(7) \"tbar.gz\"\n\n--Iteration 17--\nstring(10) \"My Pics.gz\"\n\n--Iteration 18--\nstring(10) \"My Pics.gz\"\n\n--Iteration 19--\nstring(3) \"My \"\n\n--Iteration 20--\nstring(7) \"My Pics\"\n\n--Iteration 21--\nstring(10) \"My Pics.gz\"\n\n--Iteration 22--\nstring(10) \"My Pics.gz\"\n\n--Iteration 23--\nstring(10) \"My Pics.gz\"\n\n--Iteration 24--\nstring(10) \"My Pics.gz\"\n\n--Iteration 25--\nstring(4) \"10.5\"\n\n--Iteration 26--\nstring(2) \"10\"\n\n--Iteration 27--\nstring(4) \"10.5\"\n\n--Iteration 28--\nstring(2) \"10\"\n\n--Iteration 29--\nstring(2) \"10\"\n\n--Iteration 30--\nstring(4) \"10.5\"\n\n--Iteration 31--\nstring(4) \"10.5\"\n\n--Iteration 32--\nstring(5) \"10.gz\"\n\n--Iteration 33--\nstring(1) \"0\"\n\n--Iteration 34--\nstring(1) \"0\"\n\n--Iteration 35--\nstring(6) \"bar.gz\"\n\n--Iteration 36--\nstring(6) \"bar.gz\"\n\n--Iteration 37--\nstring(6) \"bar.gz\"\n\n--Iteration 38--\nstring(1) \" \"\n\n--Iteration 39--\nstring(1) \" \"\n\n--Iteration 40--\nstring(1) \" \"\n\n--Iteration 41--\nstring(1) \" \"\n\n--Iteration 42--\nstring(0) \"\"\n\n--Iteration 43--\nstring(0) \"\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/bin2hex.phpt`: bin2hex() function
#[test]
fn phpt_bin2hex() {
    let out = compile_and_run(r#"<?php
$s = '';
for($i=0; $i<256; $i++) {
    $s .= chr($i);
}
echo bin2hex($s)."\n";
echo bin2hex("abc")."\n";
?>
"#);
    assert_eq!(out, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff\n616263\n");
}

/// php-src `ext/standard/tests/strings/bin2hex_001.phpt`: bin2hex(); function test
#[test]
fn phpt_bin2hex_001() {
    let out = compile_and_run(r#"<?php
echo bin2hex("123456");
?>
"#);
    assert_eq!(out, "313233343536");
}

/// php-src `ext/standard/tests/strings/bin2hex_basic.phpt`: Test bin2hex() function : basic functionality
#[test]
fn phpt_bin2hex_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing bin2hex() : basic functionality ***\n";

// array with different values for $string
$strings =  array (

          //double quoted strings
/*1*/	  "Here is a simple string",
          "\t This String contains \t\t some control characters\r\n",
          "\x90\x91\x00\x93\x94\x90\x91\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f",

           //single quoted strings
/*4*/	  'Here is a simple string',
          '\t This String contains \t\t some control characters\r\n',
          '\x90\x91\x00\x93\x94\x90\x91\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f',
);

// loop through with each element of the $strings array to test bin2hex() function
$count = 1;
foreach($strings as $string) {
  echo "-- Iteration $count --\n";
  var_dump(bin2hex($string));
  $count ++;
}
?>
"#);
    assert_eq!(out, "*** Testing bin2hex() : basic functionality ***\n-- Iteration 1 --\nstring(46) \"4865726520697320612073696d706c6520737472696e67\"\n-- Iteration 2 --\nstring(102) \"09205468697320537472696e6720636f6e7461696e7320090920736f6d6520636f6e74726f6c20636861726163746572730d0a\"\n-- Iteration 3 --\nstring(36) \"9091009394909195969798999a9b9c9d9e9f\"\n-- Iteration 4 --\nstring(46) \"4865726520697320612073696d706c6520737472696e67\"\n-- Iteration 5 --\nstring(112) \"5c74205468697320537472696e6720636f6e7461696e73205c745c7420736f6d6520636f6e74726f6c20636861726163746572735c725c6e\"\n-- Iteration 6 --\nstring(144) \"5c7839305c7839315c7830305c7839335c7839345c7839305c7839315c7839355c7839365c7839375c7839385c7839395c7839615c7839625c7839635c7839645c7839655c783966\"\n");
}

/// php-src `ext/standard/tests/strings/bug20169.phpt`: Bug #20169 (implode() clobbers first argument)
#[test]
fn phpt_bug20169() {
    let out = compile_and_run(r#"<?php
    @set_time_limit(5);
    $delimiter = "|";

    echo "delimiter: $delimiter\n";
    implode($delimiter, array("foo", "bar"));
    echo "delimiter: $delimiter\n";
?>
"#);
    assert_eq!(out, "delimiter: |\ndelimiter: |\n");
}

/// php-src `ext/standard/tests/strings/bug20927.phpt`: Bug #20927 (Segfault on wordwrap statement)
#[test]
fn phpt_bug20927() {
    let out = compile_and_run(r#"<?php
$string		= str_repeat("1234567890 X ", 10);
$break		= str_repeat("a-very-long-break-string-to-clobber-the-heap", 8);
$linelength	= 10;

echo "Length of original string:  ".strlen($string)."\n";
echo "Length of break string:     ".strlen($break)."\n";

var_dump(wordwrap($string, $linelength, $break, 1));
?>
"#);
    assert_eq!(out, "Length of original string:  130\nLength of break string:     352\nstring(6799) \"1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapXa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heap1234567890a-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapa-very-long-break-string-to-clobber-the-heapX \"\n");
}

/// php-src `ext/standard/tests/strings/bug20934.phpt`: Bug #20934 (htmlspecialchars returns latin1 from UTF-8)
#[test]
fn phpt_bug20934() {
    let out = compile_and_run(r#"<?php
$str = "\xc3\xa0\xc3\xa1";
echo bin2hex($str), "\n";
echo bin2hex(htmlspecialchars($str, ENT_COMPAT, "UTF-8")), "\n";
?>
"#);
    assert_eq!(out, "c3a0c3a1\nc3a0c3a1\n");
}

/// php-src `ext/standard/tests/strings/bug21338.phpt`: Bug #20934 (html_entity_decode() crash when "" is passed)
#[test]
fn phpt_bug21338() {
    let out = compile_and_run(r#"<?php
var_dump(html_entity_decode(""));
?>
"#);
    assert_eq!(out, "string(0) \"\"\n");
}

/// php-src `ext/standard/tests/strings/bug22187.phpt`: Bug #22187 (possible crash in number_format() function)
#[test]
fn phpt_bug22187() {
    let out = compile_and_run(r#"<?php
    var_dump(number_format(0.0001, 1));
    var_dump(number_format(0.0001, 0));
?>
"#);
    assert_eq!(out, "string(3) \"0.0\"\nstring(1) \"0\"\n");
}

/// php-src `ext/standard/tests/strings/bug22227.phpt`: Bug #22227 (printf() field limiters broke between 4.2.3 and 4.3.0)
#[test]
fn phpt_bug22227() {
    let out = compile_and_run(r#"<?php
printf("%-3.3s", "abcdef");
print "\n";
?>
"#);
    assert_eq!(out, "abc\n");
}

/// php-src `ext/standard/tests/strings/bug24098.phpt`: Bug #24098 (pathinfo() crash)
#[test]
fn phpt_bug24098() {
    let out = compile_and_run(r#"<?php
    var_dump(pathinfo("/dsds.asa"));
?>
"#);
    assert_eq!(out, "array(4) {\n  [\"dirname\"]=>\n  string(1) \"/\"\n  [\"basename\"]=>\n  string(8) \"dsds.asa\"\n  [\"extension\"]=>\n  string(3) \"asa\"\n  [\"filename\"]=>\n  string(4) \"dsds\"\n}\n");
}

/// php-src `ext/standard/tests/strings/bug27278.phpt`: Bug #27278 (*printf() functions treat arguments as if passed by reference)
#[test]
fn phpt_bug27278() {
    let out = compile_and_run(r#"<?php

function foo ($a)
{
    $a=sprintf("%02d",$a);
    var_dump($a);
}

$x="02";
var_dump($x);
foo($x);
var_dump($x);

?>
"#);
    assert_eq!(out, "string(2) \"02\"\nstring(2) \"02\"\nstring(2) \"02\"\n");
}

/// php-src `ext/standard/tests/strings/bug27675.phpt`: Bug #27675 (str_ireplace segfaults when shrinking string)
#[test]
fn phpt_bug27675() {
    let out = compile_and_run(r#"<?php
echo str_ireplace('/*<B>', '<B>', '/*<b> I am a comment</b>*/');
?>
"#);
    assert_eq!(out, "<B> I am a comment</b>*/");
}

/// php-src `ext/standard/tests/strings/bug28386.phpt`: Bug #28386 (wordwrap() wraps text 1 character too soon)
#[test]
fn phpt_bug28386() {
    let out = compile_and_run(r#"<?php
$text = "Some text";
$string = "$text $text $text $text";
echo wordwrap($string, 9);
?>
"#);
    assert_eq!(out, "Some text\nSome text\nSome text\nSome text");
}

/// php-src `ext/standard/tests/strings/bug28633.phpt`: Bug #28633 (sprintf incorrectly adding padding to floats)
#[test]
fn phpt_bug28633() {
    let out = compile_and_run(r#"<?php
    echo sprintf("%05.2f", 0.02) . "\n";
    echo sprintf("%05.2f", 2.02) . "\n";
?>
"#);
    assert_eq!(out, "00.02\n02.02\n");
}

/// php-src `ext/standard/tests/strings/bug29075.phpt`: Bug #29075 (strnatcmp() incorrectly handles whitespace)
#[test]
fn phpt_bug29075() {
    let out = compile_and_run(r#"<?php
    var_dump(
        strnatcmp('foo ', 'foo '),
        strnatcmp('foo', 'foo'),
        strnatcmp(' foo', ' foo')
    );
?>
"#);
    assert_eq!(out, "int(0)\nint(0)\nint(0)\n");
}

/// php-src `ext/standard/tests/strings/bug33076.phpt`: Bug #33076 (str_ireplace() incorrectly counts result string length and may cause segfault)
#[test]
fn phpt_bug33076() {
    let out = compile_and_run(r#"<?php

$value = str_ireplace("t", "bz", "Text");

var_dump($value);

echo "Done\n";
?>
"#);
    assert_eq!(out, "string(6) \"bzexbz\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/bug36306.phpt`: Bug #36306 (crc32() 64bit)
#[test]
fn phpt_bug36306() {
    let out = compile_and_run(r#"<?php

/* as an example how to write crc32 tests
   PHP does not have uint values, you cannot
   display crc32 like a signed integer.
   Have to find some small strings to truly reproduce
   the problem, this example being not a problem
*/
echo dechex(crc32("platform independent")) . "\n";
?>
"#);
    assert_eq!(out, "cbd056ba\n");
}

/// php-src `ext/standard/tests/strings/bug39350.phpt`: Bug #39350 (implode/join() crash on empty input strings)
#[test]
fn phpt_bug39350() {
    let out = compile_and_run(r#"<?php

implode('', array(null));
implode('', array(false));
implode('', array(""));

echo "Done\n";
?>
"#);
    assert_eq!(out, "Done\n");
}

/// php-src `ext/standard/tests/strings/bug39621.phpt`: Bug #39621 (str_replace() is not binary safe on strings with equal length)
#[test]
fn phpt_bug39621() {
    let out = compile_and_run(r#"<?php

$search =  "qxxx\0qqqqqqqq";
$subject = "qxxx\0xxxxxxxx";
$replace = "any text";

$result = str_replace ( $search, $replace, $subject );

var_dump($result);

$search =  "QXXX\0qqqqqqqq";
$subject = "qxxx\0xxxxxxxx";
$replace = "any text";

$result = str_ireplace ( $search, $replace, $subject );

var_dump($result);

$search =  "qxxx\0xxxxxxxx";
$subject = "qxxx\0xxxxxxxx";
$replace = "any text";

$result = str_replace ( $search, $replace, $subject );

var_dump($result);

$search =  "qXxx\0xXxXxXxx";
$subject = "qxXx\0xxxxxxxx";
$replace = "any text";

$result = str_ireplace ( $search, $replace, $subject );

var_dump($result);

echo "Done\n";
?>
"#);
    assert_eq!(out, "string(13) \"qxxx\u{0}xxxxxxxx\"\nstring(13) \"qxxx\u{0}xxxxxxxx\"\nstring(8) \"any text\"\nstring(8) \"any text\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/bug45166.phpt`: Bug #45166 (substr() )
#[test]
fn phpt_bug45166() {
    let out = compile_and_run(r#"<?php
    echo substr('cd', -3) . "\n";
?>
"#);
    assert_eq!(out, "cd\n");
}

/// php-src `ext/standard/tests/strings/bug47168.phpt`: Bug #47168 (printf of floating point variable prints maximum of 40 decimal places)
#[test]
fn phpt_bug47168() {
    let out = compile_and_run(r#"<?php

$dyadic = 0.00000000000045474735088646411895751953125;
var_dump(printf ("%1.41f\n",unserialize(serialize($dyadic))));

?>
"#);
    assert_eq!(out, "0.00000000000045474735088646411895751953125\nint(44)\n");
}

/// php-src `ext/standard/tests/strings/bug60801.phpt`: Bug #60801 (strpbrk() mishandles NUL byte)
#[test]
fn phpt_bug60801() {
    let out = compile_and_run(r#"<?php
$haystack = "foob\x00ar";
$needle = "a\x00b";

var_dump(strpbrk($haystack, 'ar'));
var_dump(strpbrk($haystack, "\x00"));
var_dump(strpbrk($haystack, $needle));
var_dump(strpbrk('foobar', $needle));
var_dump(strpbrk("\x00", $needle));
var_dump(strpbrk('xyz', $needle));
var_dump(strpbrk($haystack, 'xyz'));
?>
"#);
    assert_eq!(out, "string(2) \"ar\"\nstring(3) \"\u{0}ar\"\nstring(4) \"b\u{0}ar\"\nstring(3) \"bar\"\nstring(1) \"\u{0}\"\nbool(false)\nbool(false)\n");
}

/// php-src `ext/standard/tests/strings/bug71188.phpt`: Bug #71188 (str_replace converts integers in original $search array to strings)
#[test]
fn phpt_bug71188() {
    let out = compile_and_run(r#"<?php
$a = [0, 1, 2];
$b = ["Nula", "Jedna", "Dva"];

var_dump($a);
str_replace($a, $b, "1");
var_dump($a);
?>
"#);
    assert_eq!(out, "array(3) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n}\narray(3) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n}\n");
}

/// php-src `ext/standard/tests/strings/bug71190.phpt`: Bug #71190 (substr_replace converts integers in original $search array to strings)
#[test]
fn phpt_bug71190() {
    let out = compile_and_run(r#"<?php
$b = [0, 1, 2];

var_dump($b);
substr_replace("test", $b, "1");
var_dump($b);
?>
"#);
    assert_eq!(out, "array(3) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n}\narray(3) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n}\n");
}

/// php-src `ext/standard/tests/strings/chop_variation3.phpt`: Test chop() function : usage variations - with heredoc string
#[test]
fn phpt_chop_variation3() {
    let out = compile_and_run(r#"<?php
/*
 * Testing chop() : with heredoc strings
*/

echo "*** Testing chop() : with heredoc strings ***\n";

// defining different heredoc strings
$empty_heredoc = <<<EOT
EOT;

$heredoc_with_newline = <<<EOT
\n

EOT;

$heredoc_with_characters = <<<EOT
first line of heredoc string
second line of heredoc string
third line of heredocstring
EOT;

$heredoc_with_newline_and_tabs = <<<EOT
hello\tworld\nhello\nworld\n
EOT;

$heredoc_with_alphanumerics = <<<EOT
hello123world456
1234hello\t1234
EOT;

$heredoc_with_embedded_nulls = <<<EOT
hello\0world\0hello
\0hello\0
EOT;

$heredoc_strings = array(
                   $empty_heredoc,
                   $heredoc_with_newline,
                   $heredoc_with_characters,
           $heredoc_with_newline_and_tabs,
           $heredoc_with_alphanumerics,
           $heredoc_with_embedded_nulls
           );
$count = 1;
foreach($heredoc_strings as $string)  {
  echo "\n--- Iteration $count ---\n";
  var_dump( chop($string) );
  var_dump( chop($string, "12345o\0\n\t") );
  $count++;
}

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing chop() : with heredoc strings ***\n\n--- Iteration 1 ---\nstring(0) \"\"\nstring(0) \"\"\n\n--- Iteration 2 ---\nstring(0) \"\"\nstring(0) \"\"\n\n--- Iteration 3 ---\nstring(86) \"first line of heredoc string\nsecond line of heredoc string\nthird line of heredocstring\"\nstring(86) \"first line of heredoc string\nsecond line of heredoc string\nthird line of heredocstring\"\n\n--- Iteration 4 ---\nstring(23) \"hello\tworld\nhello\nworld\"\nstring(23) \"hello\tworld\nhello\nworld\"\n\n--- Iteration 5 ---\nstring(31) \"hello123world456\n1234hello\t1234\"\nstring(25) \"hello123world456\n1234hell\"\n\n--- Iteration 6 ---\nstring(24) \"hello\u{0}world\u{0}hello\n\u{0}hello\"\nstring(23) \"hello\u{0}world\u{0}hello\n\u{0}hell\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/chop_variation4.phpt`: Test chop() function : usage variations - strings with embedded nulls
#[test]
fn phpt_chop_variation4() {
    let out = compile_and_run(r#"<?php
/*
 * Testing chop() : with nulls embedded in input string
*/

echo "*** Testing chop() : string with embedded nulls ***\n";

// defining varous strings with embedded nulls
$strings_with_nulls = array(
               "hello\0world",
               "\0hello",
               "hello\0",
               "\0\0hello\tworld\0\0",
               "\\0hello\\0",
               'hello\0\0',
               chr(0),
               chr(0).chr(0),
                   chr(0).'hello'.chr(0),
               'hello'.chr(0).'world'
               );

$count = 1;
foreach($strings_with_nulls as $string)  {
  echo "\n--- Iteration $count ---\n";
  var_dump( chop($string) );
  var_dump( chop($string, "\0") );
  var_dump( chop($string, '\0') );
  $count++;
}

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing chop() : string with embedded nulls ***\n\n--- Iteration 1 ---\nstring(11) \"hello\u{0}world\"\nstring(11) \"hello\u{0}world\"\nstring(11) \"hello\u{0}world\"\n\n--- Iteration 2 ---\nstring(6) \"\u{0}hello\"\nstring(6) \"\u{0}hello\"\nstring(6) \"\u{0}hello\"\n\n--- Iteration 3 ---\nstring(5) \"hello\"\nstring(5) \"hello\"\nstring(6) \"hello\u{0}\"\n\n--- Iteration 4 ---\nstring(13) \"\u{0}\u{0}hello\tworld\"\nstring(13) \"\u{0}\u{0}hello\tworld\"\nstring(15) \"\u{0}\u{0}hello\tworld\u{0}\u{0}\"\n\n--- Iteration 5 ---\nstring(9) \"\\0hello\\0\"\nstring(9) \"\\0hello\\0\"\nstring(7) \"\\0hello\"\n\n--- Iteration 6 ---\nstring(9) \"hello\\0\\0\"\nstring(9) \"hello\\0\\0\"\nstring(5) \"hello\"\n\n--- Iteration 7 ---\nstring(0) \"\"\nstring(0) \"\"\nstring(1) \"\u{0}\"\n\n--- Iteration 8 ---\nstring(0) \"\"\nstring(0) \"\"\nstring(2) \"\u{0}\u{0}\"\n\n--- Iteration 9 ---\nstring(6) \"\u{0}hello\"\nstring(6) \"\u{0}hello\"\nstring(7) \"\u{0}hello\u{0}\"\n\n--- Iteration 10 ---\nstring(11) \"hello\u{0}world\"\nstring(11) \"hello\u{0}world\"\nstring(11) \"hello\u{0}world\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/chr_basic.phpt`: Test chr() function : basic functionality
#[test]
fn phpt_chr_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing chr() : basic functionality ***\n";

echo chr(72). chr(101) . chr(108) . chr(108). chr(111); // Hello
echo chr(10); // "\n"
echo "World";
echo "\n";
?>
"#);
    assert_eq!(out, "*** Testing chr() : basic functionality ***\nHello\nWorld\n");
}

/// php-src `ext/standard/tests/strings/crc32.phpt`: crc32() function
#[test]
fn phpt_crc32() {
    let out = compile_and_run(r#"<?php
$input = array("foo", "bar", "baz", "grldsajkopallkjasd");
foreach($input AS $i) {
    printf("%u\n", crc32($i));
}
printf("%u\n", crc32("1234567890123456"));
printf("%u\n", crc32("1234567890123456abc"));
printf("%u\n", crc32("12345678901234561234567890123456"));
printf("%u\n", crc32("12345678901234561234567890123456abc"));
printf("%u\n", crc32("123456789012345612345678901234561234567890123456"));
printf("%u\n", crc32("123456789012345612345678901234561234567890123456abc"));
printf("%u\n", crc32("1234567890123456123456789012345612345678901234561234567890123456"));
printf("%u\n", crc32("1234567890123456123456789012345612345678901234561234567890123456abc"));
printf("%u\n", crc32("12345678901234561234567890123456123456789012345612345678901234561234567890123456"));
printf("%u\n", crc32("12345678901234561234567890123456123456789012345612345678901234561234567890123456abc"));
printf("%u\n", crc32("12345678901234561234567890123456123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456"));
printf("%u\n", crc32("12345678901234561234567890123456123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456abc"));
printf("%u\n", crc32("123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456"));
printf("%u\n", crc32("123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456abc"));
?>
"#);
    assert_eq!(out, "2356372769\n1996459178\n2015626392\n824412087\n509595063\n1890929711\n156217630\n941689929\n1939457775\n2213121284\n522627406\n3806880586\n105010221\n1135881371\n640556525\n3081003972\n3751918510\n1076024698\n");
}

/// php-src `ext/standard/tests/strings/crc32_basic.phpt`: Test crc32() function : basic functionality
#[test]
fn phpt_crc32_basic() {
    let out = compile_and_run(r#"<?php
/*
 * Testing crc32() : basic functionality
*/

echo "*** Testing crc32() : basic functionality ***\n";


// Initialise all required variables
$str = 'string_val1234';

// Calling crc32() with all possible arguments

// checking for the return type of the function
var_dump( is_int(crc32($str)) );

// Printing the returned value from the function
printf("%u\n", crc32($str) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing crc32() : basic functionality ***\nbool(true)\n256895812\nDone");
}

/// php-src `ext/standard/tests/strings/explode_variation4.phpt`: Test explode() function : usage variations - match longer string
#[test]
fn phpt_explode_variation4() {
    let out = compile_and_run(r#"<?php

echo "*** Testing explode() function: match longer string ***\n";

$pizza  = "piece1 piece2 piece3 piece4 piece5 piece6 p";
$pieces = explode(" p", $pizza);
var_dump($pieces);
?>
"#);
    assert_eq!(out, "*** Testing explode() function: match longer string ***\narray(7) {\n  [0]=>\n  string(6) \"piece1\"\n  [1]=>\n  string(5) \"iece2\"\n  [2]=>\n  string(5) \"iece3\"\n  [3]=>\n  string(5) \"iece4\"\n  [4]=>\n  string(5) \"iece5\"\n  [5]=>\n  string(5) \"iece6\"\n  [6]=>\n  string(0) \"\"\n}\n");
}

/// php-src `ext/standard/tests/strings/hex2bin_basic.phpt`: hex2bin(); function test
#[test]
fn phpt_hex2bin_basic() {
    let out = compile_and_run(r#"<?php

var_dump(bin2hex(hex2bin('012345')) == '012345');
var_dump(bin2hex(hex2bin('abc123')) == 'abc123');
var_dump(bin2hex(hex2bin('123abc')) == '123abc');
var_dump(bin2hex(hex2bin('FFFFFF')) == 'ffffff');

?>
"#);
    assert_eq!(out, "bool(true)\nbool(true)\nbool(true)\nbool(true)\n");
}

/// php-src `ext/standard/tests/strings/md5.phpt`: md5() with ASCII output
#[test]
fn phpt_md5() {
    let out = compile_and_run(r#"<?php
echo md5("")."\n";
echo md5("a")."\n";
echo md5("abc")."\n";
echo md5("message digest")."\n";
echo md5("abcdefghijklmnopqrstuvwxyz")."\n";
echo md5("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")."\n";
echo md5("12345678901234567890123456789012345678901234567890123456789012345678901234567890")."\n";
?>
"#);
    assert_eq!(out, "d41d8cd98f00b204e9800998ecf8427e\n0cc175b9c0f1b6a831c399e269772661\n900150983cd24fb0d6963f7d28e17f72\nf96b697d7cb7938d525a2f31aaf161d0\nc3fcd3d76192e4007dfb496cca67e13b\nd174ab98d277d9f5a5611c2c9f419d9f\n57edf4a22be3c955ac49da2e2107b67a\n");
}

/// php-src `ext/standard/tests/strings/md5_basic1.phpt`: Test md5() function : basic functionality
#[test]
fn phpt_md5_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing md5() : basic functionality ***\n";
var_dump(md5("apple"));
?>
"#);
    assert_eq!(out, "*** Testing md5() : basic functionality ***\nstring(32) \"1f3870be274f6c49b3e31a0c6728957f\"\n");
}

/// php-src `ext/standard/tests/strings/md5_basic2.phpt`: Test md5() function : basic functionality - with raw output
#[test]
fn phpt_md5_basic2() {
    let out = compile_and_run(r#"<?php
echo "*** Testing md5() : basic functionality - with raw output***\n";
$str = "Hello World";
$md5_raw = md5($str, true);
var_dump(bin2hex($md5_raw));

$md5 = md5($str, false);

if (strcmp(bin2hex($md5_raw), $md5) == 0 ) {
    echo "TEST PASSED\n";
} else {
    echo "TEST FAILED\n";
    var_dump($md5_raw, $md5);
}

?>
"#);
    assert_eq!(out, "*** Testing md5() : basic functionality - with raw output***\nstring(32) \"b10a8db164e0754105b7a99be72e3fe5\"\nTEST PASSED\n");
}

/// php-src `ext/standard/tests/strings/md5raw.phpt`: md5() with RAW output
#[test]
fn phpt_md5raw() {
    let out = compile_and_run(r#"<?php
echo bin2hex(md5("", TRUE))."\n";
echo bin2hex(md5("a", TRUE))."\n";
echo bin2hex(md5("abc", TRUE))."\n";
echo bin2hex(md5("message digest", TRUE))."\n";
echo bin2hex(md5("abcdefghijklmnopqrstuvwxyz", TRUE))."\n";
echo bin2hex(md5("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", TRUE))."\n";
echo bin2hex(md5("12345678901234567890123456789012345678901234567890123456789012345678901234567890", TRUE))."\n";
?>
"#);
    assert_eq!(out, "d41d8cd98f00b204e9800998ecf8427e\n0cc175b9c0f1b6a831c399e269772661\n900150983cd24fb0d6963f7d28e17f72\nf96b697d7cb7938d525a2f31aaf161d0\nc3fcd3d76192e4007dfb496cca67e13b\nd174ab98d277d9f5a5611c2c9f419d9f\n57edf4a22be3c955ac49da2e2107b67a\n");
}

/// php-src `ext/standard/tests/strings/nl2br_variation2.phpt`: Test nl2br() function : usage variations - single quoted strings for 'str' argument
#[test]
fn phpt_nl2br_variation2() {
    let out = compile_and_run(r#"<?php
/* Test nl2br() function by passing single quoted strings containing various
 *   combinations of new line chars to 'str' argument
*/

echo "*** Testing nl2br() : usage variations ***\n";
$strings = array(
  '\n',
  '\r',
  '\r\n',
  'Hello\nWorld',
  'Hello\rWorld',
  'Hello\r\nWorld',

  //one blank line
  '
',

  //two blank lines
  '

',

  //inserted new line
  'Hello
World'
);

//loop through $strings array to test nl2br() function with each element
$count = 1;
foreach( $strings as $str ){
  echo "-- Iteration $count --\n";
  var_dump(nl2br($str) );
  $count ++ ;
}
echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing nl2br() : usage variations ***\n-- Iteration 1 --\nstring(2) \"\\n\"\n-- Iteration 2 --\nstring(2) \"\\r\"\n-- Iteration 3 --\nstring(4) \"\\r\\n\"\n-- Iteration 4 --\nstring(12) \"Hello\\nWorld\"\n-- Iteration 5 --\nstring(12) \"Hello\\rWorld\"\n-- Iteration 6 --\nstring(14) \"Hello\\r\\nWorld\"\n-- Iteration 7 --\nstring(7) \"<br />\n\"\n-- Iteration 8 --\nstring(14) \"<br />\n<br />\n\"\n-- Iteration 9 --\nstring(17) \"Hello<br />\nWorld\"\nDone");
}

/// php-src `ext/standard/tests/strings/print_basic.phpt`: Test print() function : basic functionality
#[test]
fn phpt_print_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing print() : basic functionality ***\n";

echo "\n-- Iteration 1 --\n";
print("Hello World");

echo "\n-- Iteration 2 --\n";
print "print() also works without parentheses.";

echo "\n-- Iteration 3 --\n";
print "This spans
multiple lines. The newlines will be
output as well";

echo "\n-- Iteration 4 --\n";
print "This also spans\nmultiple lines. The newlines will be\noutput as well.";

echo "\n-- Iteration 5 --\n";
print "escaping characters is done \"Like this\".";

// You can use variables inside of a print statement
$foo = "foobar";
$bar = "barbaz";

echo "\n-- Iteration 6 --\n";
print "foo is $foo"; // foo is foobar

// You can also use arrays
$bar = array("value" => "foo");

echo "\n-- Iteration 7 --\n";
print "this is {$bar['value']} !"; // this is foo !

// Using single quotes will print the variable name, not the value
echo "\n-- Iteration 8 --\n";
print 'foo is $foo'; // foo is $foo

// If you are not using any other characters, you can just print variables
echo "\n-- Iteration 9 --\n";
print $foo;          // foobar

echo "\n-- Iteration 10 --\n";
$variable = "VARIABLE";
print <<<END
This uses the "here document" syntax to output
multiple lines with $variable interpolation. Note
that the here document terminator must appear on a
line with just a semicolon no extra whitespace!\n
END;
?>
"#);
    assert_eq!(out, "*** Testing print() : basic functionality ***\n\n-- Iteration 1 --\nHello World\n-- Iteration 2 --\nprint() also works without parentheses.\n-- Iteration 3 --\nThis spans\nmultiple lines. The newlines will be\noutput as well\n-- Iteration 4 --\nThis also spans\nmultiple lines. The newlines will be\noutput as well.\n-- Iteration 5 --\nescaping characters is done \"Like this\".\n-- Iteration 6 --\nfoo is foobar\n-- Iteration 7 --\nthis is foo !\n-- Iteration 8 --\nfoo is $foo\n-- Iteration 9 --\nfoobar\n-- Iteration 10 --\nThis uses the \"here document\" syntax to output\nmultiple lines with VARIABLE interpolation. Note\nthat the here document terminator must appear on a\nline with just a semicolon no extra whitespace!\n");
}

/// php-src `ext/standard/tests/strings/printf_basic1.phpt`: Test printf() function : basic functionality - string format
#[test]
fn phpt_printf_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing printf() : basic functionality - using string format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%s";
$format2 = "%s %s";
$format3 = "%s %s %s";
$arg1 = "arg1 argument";
$arg2 = "arg2 argument";
$arg3 = "arg3 argument";

echo "\n-- Calling printf() with no arguments --\n";
$result = printf($format);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with one arguments --\n";
$result = printf($format1, $arg1);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with two arguments --\n";
$result = printf($format2, $arg1, $arg2);
echo "\n";
var_dump($result);


echo "\n-- Calling printf() with string three arguments --\n";
$result = printf($format3, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing printf() : basic functionality - using string format ***\n\n-- Calling printf() with no arguments --\nformat\nint(6)\n\n-- Calling printf() with one arguments --\narg1 argument\nint(13)\n\n-- Calling printf() with two arguments --\narg1 argument arg2 argument\nint(27)\n\n-- Calling printf() with string three arguments --\narg1 argument arg2 argument arg3 argument\nint(41)\n");
}

/// php-src `ext/standard/tests/strings/printf_basic2.phpt`: Test printf() function : basic functionality - integer format
#[test]
fn phpt_printf_basic2() {
    let out = compile_and_run(r#"<?php
echo "*** Testing printf() : basic functionality - using integer format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%d";
$format2 = "%d %d";
$format3 = "%d %d %d";
$arg1 = 111;
$arg2 = 222;
$arg3 = 333;

echo "\n-- Calling printf() with no arguments --\n";
$result = printf($format);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with one arguments--\n";
$result = printf($format1, $arg1);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with two arguments--\n";
$result = printf($format2, $arg1, $arg2);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with three arguments--\n";
$result = printf($format3, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing printf() : basic functionality - using integer format ***\n\n-- Calling printf() with no arguments --\nformat\nint(6)\n\n-- Calling printf() with one arguments--\n111\nint(3)\n\n-- Calling printf() with two arguments--\n111 222\nint(7)\n\n-- Calling printf() with three arguments--\n111 222 333\nint(11)\n");
}

/// php-src `ext/standard/tests/strings/printf_basic4.phpt`: Test printf() function : basic functionality - bool format
#[test]
fn phpt_printf_basic4() {
    let out = compile_and_run(r#"<?php
echo "*** Testing printf() : basic functionality - using bool format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%b";
$format2 = "%b %b";
$format3 = "%b %b %b";
$arg1 = TRUE;
$arg2 = FALSE;
$arg3 = true;

echo "\n-- Calling printf() with no arguments --\n";
$result = printf($format);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with one arguments--\n";
$result = printf($format1, $arg1);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with two arguments--\n";
$result = printf($format2, $arg1, $arg2);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with three arguments--\n";
$result = printf($format3, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);
?>
"#);
    assert_eq!(out, "*** Testing printf() : basic functionality - using bool format ***\n\n-- Calling printf() with no arguments --\nformat\nint(6)\n\n-- Calling printf() with one arguments--\n1\nint(1)\n\n-- Calling printf() with two arguments--\n1 0\nint(3)\n\n-- Calling printf() with three arguments--\n1 0 1\nint(5)\n");
}

/// php-src `ext/standard/tests/strings/printf_basic5.phpt`: Test printf() function : basic functionality - char format
#[test]
fn phpt_printf_basic5() {
    let out = compile_and_run(r#"<?php
echo "*** Testing printf() : basic functionality - using char format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%c";
$format2 = "%c %c";
$format3 = "%c %c %c";
$arg1 = 65;
$arg2 = 66;
$arg3 = 67;

echo "\n-- Calling printf() with no arguments --\n";
$result = printf($format);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with one arguments --\n";
$result = printf($format1, $arg1);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with two arguments --\n";
$result = printf($format2, $arg1, $arg2);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with three arguments --\n";
$result = printf($format3, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);
?>
"#);
    assert_eq!(out, "*** Testing printf() : basic functionality - using char format ***\n\n-- Calling printf() with no arguments --\nformat\nint(6)\n\n-- Calling printf() with one arguments --\nA\nint(1)\n\n-- Calling printf() with two arguments --\nA B\nint(3)\n\n-- Calling printf() with three arguments --\nA B C\nint(5)\n");
}

/// php-src `ext/standard/tests/strings/printf_basic9.phpt`: Test printf() function : basic functionality - hexadecimal format
#[test]
fn phpt_printf_basic9() {
    let out = compile_and_run(r#"<?php
echo "*** Testing printf() : basic functionality - using hexadecimal format ***\n";

// Initialise all required variables

// Initialising different format strings
$format = "format";
$format1 = "%x";
$format2 = "%x %x";
$format3 = "%x %x %x";

$format11 = "%X";
$format22 = "%X %X";
$format33 = "%X %X %X";

$arg1 = 11;
$arg2 = 132;
$arg3 = 177;

echo "\n-- Calling printf() with no arguments --\n";
$result = printf($format);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with one arguments --\n";
$result = printf($format1, $arg1);
echo "\n";
var_dump($result);
$result = printf($format11, $arg1);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with two arguments --\n";
$result = printf($format2, $arg1, $arg2);
echo "\n";
var_dump($result);
$result = printf($format22, $arg1, $arg2);
echo "\n";
var_dump($result);

echo "\n-- Calling printf() with three arguments --\n";
$result = printf($format3, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);
$result = printf($format33, $arg1, $arg2, $arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing printf() : basic functionality - using hexadecimal format ***\n\n-- Calling printf() with no arguments --\nformat\nint(6)\n\n-- Calling printf() with one arguments --\nb\nint(1)\nB\nint(1)\n\n-- Calling printf() with two arguments --\nb 84\nint(4)\nB 84\nint(4)\n\n-- Calling printf() with three arguments --\nb 84 b1\nint(7)\nB 84 B1\nint(7)\n");
}

/// php-src `ext/standard/tests/strings/sha1_basic.phpt`: sha1() with ASCII output.
#[test]
fn phpt_sha1_basic() {
    let out = compile_and_run(r#"<?php


echo "*** Testing sha1() : basic functionality ***\n";

echo "\n-- Without raw argument --\n";
var_dump(sha1(""));
var_dump(sha1("a"));
var_dump(sha1("abc"));
var_dump(sha1("message digest"));
var_dump(sha1("abcdefghijklmnopqrstuvwxyz"));
var_dump(sha1("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"));
var_dump(sha1("12345678901234567890123456789012345678901234567890123456789012345678901234567890"));

echo "\n-- With raw == false --\n";
var_dump(sha1("", false));
var_dump(sha1("a", false));
var_dump(sha1("abc", false));
var_dump(sha1("message digest", false));
var_dump(sha1("abcdefghijklmnopqrstuvwxyz", false));
var_dump(sha1("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", false));
var_dump(sha1("12345678901234567890123456789012345678901234567890123456789012345678901234567890", false));

echo "\n-- With raw == true --\n";
var_dump(bin2hex(sha1("", true)));
var_dump(bin2hex(sha1("a", true)));
var_dump(bin2hex(sha1("abc", true)));
var_dump(bin2hex(sha1("message digest", true)));
var_dump(bin2hex(sha1("abcdefghijklmnopqrstuvwxyz", true)));
var_dump(bin2hex(sha1("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", true)));
var_dump(bin2hex(sha1("12345678901234567890123456789012345678901234567890123456789012345678901234567890", true)));

?>
"#);
    assert_eq!(out, "*** Testing sha1() : basic functionality ***\n\n-- Without raw argument --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\n\n-- With raw == false --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\n\n-- With raw == true --\nstring(40) \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"\nstring(40) \"86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\"\nstring(40) \"a9993e364706816aba3e25717850c26c9cd0d89d\"\nstring(40) \"c12252ceda8be8994d5fa0290a47231c1d16aae3\"\nstring(40) \"32d10c7b8cf96570ca04ce37f2a19d84240d3a89\"\nstring(40) \"761c457bf73b14d27e9e9265c46f4b4dda11f940\"\nstring(40) \"50abf5706a150990a08b2c5ea40fa0e585554732\"\n");
}

/// php-src `ext/standard/tests/strings/sha1raw.phpt`: sha1() with RAW output
#[test]
fn phpt_sha1raw() {
    let out = compile_and_run(r#"<?php
echo bin2hex(sha1("abc", TRUE))."\n";
echo bin2hex(sha1("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", TRUE))."\n";
echo bin2hex(sha1("a", TRUE))."\n";
echo bin2hex(sha1("0123456701234567012345670123456701234567012345670123456701234567", TRUE))."\n";
?>
"#);
    assert_eq!(out, "a9993e364706816aba3e25717850c26c9cd0d89d\n84983e441c3bd26ebaae4aa1f95129e5e54670f1\n86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\ne0c094e867ef46c350ef54a7f59dd60bed92ae83\n");
}

/// php-src `ext/standard/tests/strings/sprintf_basic1.phpt`: Test sprintf() function : basic functionality - string format
#[test]
fn phpt_sprintf_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing sprintf() : basic functionality - using string format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%s";
$format2 = "%s %s";
$format3 = "%s %s %s";
$arg1 = "arg1 argument";
$arg2 = "arg2 argument";
$arg3 = "arg3 argument";

// Calling sprintf() with default arguments
var_dump( sprintf($format) );

// Calling sprintf() with two arguments
var_dump( sprintf($format1, $arg1) );

// Calling sprintf() with three arguments
var_dump( sprintf($format2, $arg1, $arg2) );

// Calling sprintf() with four arguments
var_dump( sprintf($format3, $arg1, $arg2, $arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing sprintf() : basic functionality - using string format ***\nstring(6) \"format\"\nstring(13) \"arg1 argument\"\nstring(27) \"arg1 argument arg2 argument\"\nstring(41) \"arg1 argument arg2 argument arg3 argument\"\nDone");
}

/// php-src `ext/standard/tests/strings/sprintf_basic2.phpt`: Test sprintf() function : basic functionality - integer format
#[test]
fn phpt_sprintf_basic2() {
    let out = compile_and_run(r#"<?php
echo "*** Testing sprintf() : basic functionality - using integer format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%d";
$format2 = "%d %d";
$format3 = "%d %d %d";
$arg1 = 111;
$arg2 = 222;
$arg3 = 333;

// Calling sprintf() with default arguments
var_dump( sprintf($format) );

// Calling sprintf() with two arguments
var_dump( sprintf($format1, $arg1) );

// Calling sprintf() with three arguments
var_dump( sprintf($format2, $arg1, $arg2) );

// Calling sprintf() with four arguments
var_dump( sprintf($format3, $arg1, $arg2, $arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing sprintf() : basic functionality - using integer format ***\nstring(6) \"format\"\nstring(3) \"111\"\nstring(7) \"111 222\"\nstring(11) \"111 222 333\"\nDone");
}

/// php-src `ext/standard/tests/strings/sprintf_basic4.phpt`: Test sprintf() function : basic functionality - bool format
#[test]
fn phpt_sprintf_basic4() {
    let out = compile_and_run(r#"<?php
echo "*** Testing sprintf() : basic functionality - using bool format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%b";
$format2 = "%b %b";
$format3 = "%b %b %b";
$arg1 = TRUE;
$arg2 = FALSE;
$arg3 = true;

// Calling sprintf() with default arguments
var_dump( sprintf($format) );

// Calling sprintf() with two arguments
var_dump( sprintf($format1, $arg1) );

// Calling sprintf() with three arguments
var_dump( sprintf($format2, $arg1, $arg2) );

// Calling sprintf() with four arguments
var_dump( sprintf($format3, $arg1, $arg2, $arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing sprintf() : basic functionality - using bool format ***\nstring(6) \"format\"\nstring(1) \"1\"\nstring(3) \"1 0\"\nstring(5) \"1 0 1\"\nDone");
}

/// php-src `ext/standard/tests/strings/sprintf_basic5.phpt`: Test sprintf() function : basic functionality - char format
#[test]
fn phpt_sprintf_basic5() {
    let out = compile_and_run(r#"<?php
echo "*** Testing sprintf() : basic functionality - using char format ***\n";


// Initialise all required variables
$format = "format";
$format1 = "%c";
$format2 = "%c %c";
$format3 = "%c %c %c";
$arg1 = 65;
$arg2 = 66;
$arg3 = 67;

// Calling sprintf() with default arguments
var_dump( sprintf($format) );

// Calling sprintf() with two arguments
var_dump( sprintf($format1, $arg1) );

// Calling sprintf() with three arguments
var_dump( sprintf($format2, $arg1, $arg2) );

// Calling sprintf() with four arguments
var_dump( sprintf($format3, $arg1, $arg2, $arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing sprintf() : basic functionality - using char format ***\nstring(6) \"format\"\nstring(1) \"A\"\nstring(3) \"A B\"\nstring(5) \"A B C\"\nDone");
}

/// php-src `ext/standard/tests/strings/sprintf_basic9.phpt`: Test sprintf() function : basic functionality - hexadecimal format
#[test]
fn phpt_sprintf_basic9() {
    let out = compile_and_run(r#"<?php
echo "*** Testing sprintf() : basic functionality - using hexadecimal format ***\n";

// Initialise all required variables

// Initialising different format strings
$format = "format";
$format1 = "%x";
$format2 = "%x %x";
$format3 = "%x %x %x";

$format11 = "%X";
$format22 = "%X %X";
$format33 = "%X %X %X";

$arg1 = 11;
$arg2 = 132;
$arg3 = 177;

// Calling sprintf() with default arguments
var_dump( sprintf($format) );

// Calling sprintf() with two arguments
var_dump( sprintf($format1, $arg1) );
var_dump( sprintf($format11, $arg1) );

// Calling sprintf() with three arguments
var_dump( sprintf($format2, $arg1, $arg2) );
var_dump( sprintf($format22, $arg1, $arg2) );

// Calling sprintf() with four arguments
var_dump( sprintf($format3, $arg1, $arg2, $arg3) );
var_dump( sprintf($format33, $arg1, $arg2, $arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing sprintf() : basic functionality - using hexadecimal format ***\nstring(6) \"format\"\nstring(1) \"b\"\nstring(1) \"B\"\nstring(4) \"b 84\"\nstring(4) \"B 84\"\nstring(7) \"b 84 b1\"\nstring(7) \"B 84 B1\"\nDone");
}

/// php-src `ext/standard/tests/strings/str_contains.phpt`: Test str_contains() function
#[test]
fn phpt_str_contains() {
    let out = compile_and_run(r#"<?php
var_dump(str_contains("test string", "test"));
var_dump(str_contains("test string", "string"));
var_dump(str_contains("test string", "strin"));
var_dump(str_contains("test string", "t s"));
var_dump(str_contains("test string", "g"));
var_dump(str_contains("te".chr(0)."st", chr(0)));
var_dump(str_contains("tEst", "test"));
var_dump(str_contains("teSt", "test"));
var_dump(str_contains("", ""));
var_dump(str_contains("a", ""));
var_dump(str_contains("", "a"));
var_dump(str_contains("\\\\a", "\\a"));
?>
"#);
    assert_eq!(out, "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n");
}

/// php-src `ext/standard/tests/strings/str_ends_with.phpt`: str_ends_with() function - unit tests for str_ends_with()
#[test]
fn phpt_str_ends_with() {
    let out = compile_and_run(r#"<?php
$testStr = "beginningMiddleEnd";
var_dump(str_ends_with($testStr, "End"));
var_dump(str_ends_with($testStr, "end"));
var_dump(str_ends_with($testStr, "en"));
var_dump(str_ends_with($testStr, $testStr));
var_dump(str_ends_with($testStr, $testStr.$testStr));
var_dump(str_ends_with($testStr, ""));
var_dump(str_ends_with("", ""));
var_dump(str_ends_with("", " "));
var_dump(str_ends_with($testStr, "\x00"));
var_dump(str_ends_with("\x00", ""));
var_dump(str_ends_with("\x00", "\x00"));
var_dump(str_ends_with("a\x00", "\x00"));
var_dump(str_ends_with("ab\x00c", "b\x00c"));
var_dump(str_ends_with("a\x00b", "d\x00b"));
var_dump(str_ends_with("a\x00b", "a\x00z"));
var_dump(str_ends_with("a", "\x00a"));
var_dump(str_ends_with("a", "a\x00"));
?>
"#);
    assert_eq!(out, "bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n");
}

/// php-src `ext/standard/tests/strings/str_split_basic.phpt`: Test str_split() function : basic functionality
#[test]
fn phpt_str_split_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing str_split() : basic functionality ***\n";

// Initialise all required variables
$str = 'This is basic testcase';
$split_length = 5;

// Calling str_split() with all possible arguments
echo "-- With all possible arguments --\n";
var_dump( str_split($str,$split_length) );

// Calling str_split() with default arguments
echo "-- With split_length as default argument --\n";
var_dump( str_split($str) );

echo "-- Empty string must always return empty array --\n";
var_dump( str_split('') );
var_dump( str_split('', 1) );
var_dump( str_split('', 100) );

?>
"#);
    assert_eq!(out, "*** Testing str_split() : basic functionality ***\n-- With all possible arguments --\narray(5) {\n  [0]=>\n  string(5) \"This \"\n  [1]=>\n  string(5) \"is ba\"\n  [2]=>\n  string(5) \"sic t\"\n  [3]=>\n  string(5) \"estca\"\n  [4]=>\n  string(2) \"se\"\n}\n-- With split_length as default argument --\narray(22) {\n  [0]=>\n  string(1) \"T\"\n  [1]=>\n  string(1) \"h\"\n  [2]=>\n  string(1) \"i\"\n  [3]=>\n  string(1) \"s\"\n  [4]=>\n  string(1) \" \"\n  [5]=>\n  string(1) \"i\"\n  [6]=>\n  string(1) \"s\"\n  [7]=>\n  string(1) \" \"\n  [8]=>\n  string(1) \"b\"\n  [9]=>\n  string(1) \"a\"\n  [10]=>\n  string(1) \"s\"\n  [11]=>\n  string(1) \"i\"\n  [12]=>\n  string(1) \"c\"\n  [13]=>\n  string(1) \" \"\n  [14]=>\n  string(1) \"t\"\n  [15]=>\n  string(1) \"e\"\n  [16]=>\n  string(1) \"s\"\n  [17]=>\n  string(1) \"t\"\n  [18]=>\n  string(1) \"c\"\n  [19]=>\n  string(1) \"a\"\n  [20]=>\n  string(1) \"s\"\n  [21]=>\n  string(1) \"e\"\n}\n-- Empty string must always return empty array --\narray(0) {\n}\narray(0) {\n}\narray(0) {\n}\n");
}

/// php-src `ext/standard/tests/strings/str_split_variation4.phpt`: Test str_split() function : usage variations - different single quoted strings for 'str' argument
#[test]
fn phpt_str_split_variation4() {
    let out = compile_and_run(r#"<?php
/*
* passing different single quoted strings as 'str' argument to str_split()
* split_length is set to 5
*/

echo "*** Testing str_split() : single quoted strings for 'str' ***\n";

//Initialize variables
$split_length = 5;

// different values for 'str'
$values = array(
  '',  //empty
  ' ',  //space
  '1234', //with only numbers
  'simple string',  //regular string
  'It\'s string with quote',  //string containing single quote
  'string\tcontains\rwhite space\nchars',
  'containing @ # $ % ^ & chars',
  'with 1234 numbers',
  'with \0 and ".chr(0)."null chars',  //for binary safe
  'with    multiple     space char',
  'Testing invalid \k and \m escape char',
  'to check with \\n and \\t' //ignoring \n and \t results
);

//loop through each element of $values for 'str' argument
for($count = 0; $count < count($values); $count++) {
  echo "-- Iteration ".($count+1)." --\n";
  var_dump( str_split($values[$count], $split_length) );
}
echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing str_split() : single quoted strings for 'str' ***\n-- Iteration 1 --\narray(0) {\n}\n-- Iteration 2 --\narray(1) {\n  [0]=>\n  string(1) \" \"\n}\n-- Iteration 3 --\narray(1) {\n  [0]=>\n  string(4) \"1234\"\n}\n-- Iteration 4 --\narray(3) {\n  [0]=>\n  string(5) \"simpl\"\n  [1]=>\n  string(5) \"e str\"\n  [2]=>\n  string(3) \"ing\"\n}\n-- Iteration 5 --\narray(5) {\n  [0]=>\n  string(5) \"It's \"\n  [1]=>\n  string(5) \"strin\"\n  [2]=>\n  string(5) \"g wit\"\n  [3]=>\n  string(5) \"h quo\"\n  [4]=>\n  string(2) \"te\"\n}\n-- Iteration 6 --\narray(8) {\n  [0]=>\n  string(5) \"strin\"\n  [1]=>\n  string(5) \"g\\tco\"\n  [2]=>\n  string(5) \"ntain\"\n  [3]=>\n  string(5) \"s\\rwh\"\n  [4]=>\n  string(5) \"ite s\"\n  [5]=>\n  string(5) \"pace\\\"\n  [6]=>\n  string(5) \"nchar\"\n  [7]=>\n  string(1) \"s\"\n}\n-- Iteration 7 --\narray(6) {\n  [0]=>\n  string(5) \"conta\"\n  [1]=>\n  string(5) \"ining\"\n  [2]=>\n  string(5) \" @ # \"\n  [3]=>\n  string(5) \"$ % ^\"\n  [4]=>\n  string(5) \" & ch\"\n  [5]=>\n  string(3) \"ars\"\n}\n-- Iteration 8 --\narray(4) {\n  [0]=>\n  string(5) \"with \"\n  [1]=>\n  string(5) \"1234 \"\n  [2]=>\n  string(5) \"numbe\"\n  [3]=>\n  string(2) \"rs\"\n}\n-- Iteration 9 --\narray(7) {\n  [0]=>\n  string(5) \"with \"\n  [1]=>\n  string(5) \"\\0 an\"\n  [2]=>\n  string(5) \"d \".c\"\n  [3]=>\n  string(5) \"hr(0)\"\n  [4]=>\n  string(5) \".\"nul\"\n  [5]=>\n  string(5) \"l cha\"\n  [6]=>\n  string(2) \"rs\"\n}\n-- Iteration 10 --\narray(7) {\n  [0]=>\n  string(5) \"with \"\n  [1]=>\n  string(5) \"   mu\"\n  [2]=>\n  string(5) \"ltipl\"\n  [3]=>\n  string(5) \"e    \"\n  [4]=>\n  string(5) \" spac\"\n  [5]=>\n  string(5) \"e cha\"\n  [6]=>\n  string(1) \"r\"\n}\n-- Iteration 11 --\narray(8) {\n  [0]=>\n  string(5) \"Testi\"\n  [1]=>\n  string(5) \"ng in\"\n  [2]=>\n  string(5) \"valid\"\n  [3]=>\n  string(5) \" \\k a\"\n  [4]=>\n  string(5) \"nd \\m\"\n  [5]=>\n  string(5) \" esca\"\n  [6]=>\n  string(5) \"pe ch\"\n  [7]=>\n  string(2) \"ar\"\n}\n-- Iteration 12 --\narray(5) {\n  [0]=>\n  string(5) \"to ch\"\n  [1]=>\n  string(5) \"eck w\"\n  [2]=>\n  string(5) \"ith \\\"\n  [3]=>\n  string(5) \"n and\"\n  [4]=>\n  string(3) \" \\t\"\n}\nDone");
}

/// php-src `ext/standard/tests/strings/str_starts_with.phpt`: str_starts_with() function - unit tests for str_starts_with()
#[test]
fn phpt_str_starts_with() {
    let out = compile_and_run(r#"<?php
$testStr = "beginningMiddleEnd";
var_dump(str_starts_with($testStr, "beginning"));
var_dump(str_starts_with($testStr, "Beginning"));
var_dump(str_starts_with($testStr, "eginning"));
var_dump(str_starts_with($testStr, $testStr));
var_dump(str_starts_with($testStr, $testStr.$testStr));
var_dump(str_starts_with($testStr, ""));
var_dump(str_starts_with("", ""));
var_dump(str_starts_with("", " "));
var_dump(str_starts_with($testStr, "\x00"));
var_dump(str_starts_with("\x00", ""));
var_dump(str_starts_with("\x00", "\x00"));
var_dump(str_starts_with("\x00a", "\x00"));
var_dump(str_starts_with("a\x00bc", "a\x00b"));
var_dump(str_starts_with("a\x00b", "a\x00d"));
var_dump(str_starts_with("a\x00b", "z\x00b"));
var_dump(str_starts_with("a", "a\x00"));
var_dump(str_starts_with("a", "\x00a"));
?>
"#);
    assert_eq!(out, "bool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n");
}

/// php-src `ext/standard/tests/strings/strcspn.phpt`: Test strcspn() behavior
#[test]
fn phpt_strcspn() {
    let out = compile_and_run(r#"<?php
$a = "22222222aaaa bbb1111 cccc";
$b = "1234";
var_dump($a);
var_dump($b);
var_dump(strcspn($a,$b));
var_dump(strcspn($a,$b,9));
var_dump(strcspn($a,$b,9,6));
var_dump(strcspn('a', 'B', 1, 2147483647));
?>
"#);
    assert_eq!(out, "string(25) \"22222222aaaa bbb1111 cccc\"\nstring(4) \"1234\"\nint(0)\nint(7)\nint(6)\nint(0)\n");
}

/// php-src `ext/standard/tests/strings/strcspn_basic.phpt`: Test strcspn() function : basic functionality
#[test]
fn phpt_strcspn_basic() {
    let out = compile_and_run(r#"<?php
/*
* Testing strcspn() : basic functionality
*/

echo "*** Testing strcspn() : basic functionality ***\n";


// Initialise all required variables
$str = "this is the test string";
$mask = "es";
$start = 15;
$len = 3;

// Calling strcspn() with all possible arguments
var_dump( strcspn($str, $mask, $start, $len) );

// Calling strcspn() with three arguments
var_dump( strcspn($str, $mask, $start) );

// Calling strcspn() with default arguments
var_dump( strcspn($str, $mask) );

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strcspn() : basic functionality ***\nint(2)\nint(2)\nint(3)\nDone");
}

/// php-src `ext/standard/tests/strings/stripcslashes_basic.phpt`: Test stripcslashes() function : basic functionality
#[test]
fn phpt_stripcslashes_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing stripcslashes() : basic functionality ***\n";
var_dump(stripcslashes('\H\e\l\l\o \W\or\l\d'));
var_dump(stripcslashes('Hello World\\r\\n'));
var_dump(stripcslashes('\\\Hello World'));
var_dump(stripcslashes('\x48\x65\x6c\x6c\x6f \x57\x6f\x72\x6c\x64'));
var_dump(stripcslashes('\110\145\154\154\157 \127\157\162\154\144'));

var_dump(bin2hex(stripcslashes('\\a')));
var_dump(bin2hex(stripcslashes('\\b')));
var_dump(bin2hex(stripcslashes('\\f')));
var_dump(bin2hex(stripcslashes('\\t')));
var_dump(bin2hex(stripcslashes('\\v')));
?>
"#);
    assert_eq!(out, "*** Testing stripcslashes() : basic functionality ***\nstring(11) \"Hello World\"\nstring(13) \"Hello World\r\n\"\nstring(12) \"\\Hello World\"\nstring(11) \"Hello World\"\nstring(11) \"Hello World\"\nstring(2) \"07\"\nstring(2) \"08\"\nstring(2) \"0c\"\nstring(2) \"09\"\nstring(2) \"0b\"\n");
}

/// php-src `ext/standard/tests/strings/stripslashes_variation3.phpt`: Test stripslashes() function : usage variations - strings with newline and tab characters
#[test]
fn phpt_stripslashes_variation3() {
    let out = compile_and_run(r#"<?php
/*
 * Test stripslashes() with strings containing newline and tab characters.
*/

echo "*** Testing stripslashes() : with strings containing newline and tab characters ***\n";

// initialising  heredoc strings
$heredoc_string_with_newline = <<<EOT
This is line 1 \nof 'heredoc' string
This is line 2 \nof "heredoc" string
EOT;

$heredoc_string_with_tab = <<<EOT
This is line 1 \tof 'heredoc' string
This is line 2 \tof "heredoc" string
EOT;
// initialising the string array

$str_array = array(
                    // string with newline character
                    "\n",
            "\\n",
                    "Hello \nworld",
                    "Hello \\nworld",
                    '\n',
            '\\n',
                    'Hello \nworld',
                    'Hello \\nworld',
                    $heredoc_string_with_newline,

                    // string with tab character
            "\t",
            "\\t",
                    "Hello \tworld",
                    "Hello \\tworld",
            '\t',
            '\\t',
                    'Hello \tworld',
                    'Hello \\tworld',
                    $heredoc_string_with_tab
                  );

$count = 1;
// looping to test for all strings in $str_array
foreach( $str_array as $str )  {
  echo "\n-- Iteration $count --\n";
  var_dump( stripslashes($str) );
  $count ++;
}

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing stripslashes() : with strings containing newline and tab characters ***\n\n-- Iteration 1 --\nstring(1) \"\n\"\n\n-- Iteration 2 --\nstring(1) \"n\"\n\n-- Iteration 3 --\nstring(12) \"Hello \nworld\"\n\n-- Iteration 4 --\nstring(12) \"Hello nworld\"\n\n-- Iteration 5 --\nstring(1) \"n\"\n\n-- Iteration 6 --\nstring(1) \"n\"\n\n-- Iteration 7 --\nstring(12) \"Hello nworld\"\n\n-- Iteration 8 --\nstring(12) \"Hello nworld\"\n\n-- Iteration 9 --\nstring(71) \"This is line 1 \nof 'heredoc' string\nThis is line 2 \nof \"heredoc\" string\"\n\n-- Iteration 10 --\nstring(1) \"\t\"\n\n-- Iteration 11 --\nstring(1) \"t\"\n\n-- Iteration 12 --\nstring(12) \"Hello \tworld\"\n\n-- Iteration 13 --\nstring(12) \"Hello tworld\"\n\n-- Iteration 14 --\nstring(1) \"t\"\n\n-- Iteration 15 --\nstring(1) \"t\"\n\n-- Iteration 16 --\nstring(12) \"Hello tworld\"\n\n-- Iteration 17 --\nstring(12) \"Hello tworld\"\n\n-- Iteration 18 --\nstring(71) \"This is line 1 \tof 'heredoc' string\nThis is line 2 \tof \"heredoc\" string\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/strlen_basic.phpt`: Test strlen() function : basic functionality
#[test]
fn phpt_strlen_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing strlen() : basic functionality ***\n";
var_dump(strlen("abcdef"));
var_dump(strlen(" ab de "));
var_dump(strlen(""));
var_dump(strlen("\x90\x91\x00\x93\x94\x90\x91\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f"));
?>
"#);
    assert_eq!(out, "*** Testing strlen() : basic functionality ***\nint(6)\nint(7)\nint(0)\nint(18)\n");
}

/// php-src `ext/standard/tests/strings/strnatcasecmp_basic.phpt`: Test strnatcasecmp() function : basic functionality
#[test]
fn phpt_strnatcasecmp_basic() {
    let out = compile_and_run(r#"<?php
function str_dump($one, $two) {
    var_dump(strnatcasecmp($one, $two));
}

echo "*** Testing strnatcasecmp() : basic functionality ***\n";

// Calling strnatcasecmp() with all possible arguments
str_dump('A', 'a');
str_dump('a10', 'A20');
str_dump('A1b', 'a');
str_dump('x2-y7', 'x8-y8');
str_dump('1.010', '1.001');
str_dump(' ab', ' aB');
str_dump('acc ', 'acc');
str_dump(11.5, 10.5);
str_dump(10.5, 10.5E1);
str_dump('Rfc822.txt', 'rfc2086.txt');
str_dump('Rfc822.txt', 'rfc822.TXT');
str_dump('pIc 6', 'pic   7');
str_dump(0xFFF, 0Xfff);

?>
"#);
    assert_eq!(out, "*** Testing strnatcasecmp() : basic functionality ***\nint(0)\nint(-1)\nint(1)\nint(-1)\nint(1)\nint(0)\nint(1)\nint(1)\nint(-1)\nint(-1)\nint(0)\nint(-1)\nint(0)\n");
}

/// php-src `ext/standard/tests/strings/strnatcasecmp_variation1.phpt`: Test strnatcasecmp() function : variation
#[test]
fn phpt_strnatcasecmp_variation1() {
    let out = compile_and_run(r#"<?php
/* Preparation */
class a
{
    function __toString()
    {
        return "Hello WORLD";
    }
}

class b
{
    function __toString()
    {
        return "HELLO world";
    }
}

$a = new a();
$b = new b();

function str_dump($a, $b) {
    var_dump(strnatcasecmp($a, $b));
}

echo "*** Testing strnatcasecmp() : variation ***\n";

str_dump('0', false);
str_dump('fooBar', '');
str_dump('', -1);
str_dump("Hello\0world", "Helloworld");
str_dump("\x0", "\0");
str_dump($a, $b);

?>
"#);
    assert_eq!(out, "*** Testing strnatcasecmp() : variation ***\nint(1)\nint(1)\nint(-1)\nint(-1)\nint(0)\nint(0)\n");
}

/// php-src `ext/standard/tests/strings/strnatcmp_basic.phpt`: Test strnatcmp() function : basic functionality
#[test]
fn phpt_strnatcmp_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strnatcmp() : basic functionality ***\n";

$a1 = "abc1";
$b1 = "abc10";
$c1 = "abc15";
$d1 = "abc2";

$a2 = "ABC1";
$b2 = "ABC10";
$c2 = "ABC15";
$d2 = "ABC2";

echo "Less than tests\n";
var_dump(strnatcmp($a1, $b1));
var_dump(strnatcmp($a1, $c1));
var_dump(strnatcmp($a1, $d1));
var_dump(strnatcmp($b1, $c1));
var_dump(strnatcmp($d1, $c1));

var_dump(strnatcmp($a1, $b2));
var_dump(strnatcmp($a1, $c2));
var_dump(strnatcmp($a1, $d2));
var_dump(strnatcmp($b1, $c2));
var_dump(strnatcmp($d1, $c2));


echo "Equal too tests\n";
var_dump(strnatcmp($b1, $b1));
var_dump(strnatcmp($b1, $b2));

echo "Greater than tests\n";
var_dump(strnatcmp($b1, $a1));
var_dump(strnatcmp($c1, $a1));
var_dump(strnatcmp($d1, $a1));
var_dump(strnatcmp($c1, $b1));
var_dump(strnatcmp($c1, $d1));

var_dump(strnatcmp($b1, $a2));
var_dump(strnatcmp($c1, $a2));
var_dump(strnatcmp($d1, $a2));
var_dump(strnatcmp($c1, $b2));
var_dump(strnatcmp($c1, $d2));
?>
"#);
    assert_eq!(out, "*** Testing strnatcmp() : basic functionality ***\nLess than tests\nint(-1)\nint(-1)\nint(-1)\nint(-1)\nint(-1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nEqual too tests\nint(0)\nint(1)\nGreater than tests\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\nint(1)\n");
}

/// php-src `ext/standard/tests/strings/strnatcmp_leftalign.phpt`: Test strnatcmp() function : left align, whitespace, digits
#[test]
fn phpt_strnatcmp_leftalign() {
    let out = compile_and_run(r#"<?php
echo "-- Testing strnatcmp() function whitespace, left-align, digit --\n";
echo "-- Leading whitespace, digits, string 1 longer --\n";
$str1 = " 00";
$str2 = " 0";
var_dump( strnatcmp( $str1, $str2) );

echo "-- Leading whitespace, digits, string 2 longer --\n";
$str1 = " 0";
$str2 = " 00";
var_dump( strnatcmp( $str1, $str2) );
?>
"#);
    assert_eq!(out, "-- Testing strnatcmp() function whitespace, left-align, digit --\n-- Leading whitespace, digits, string 1 longer --\nint(1)\n-- Leading whitespace, digits, string 2 longer --\nint(-1)\n");
}

/// php-src `ext/standard/tests/strings/strpbrk_basic.phpt`: Test strpbrk() function : basic functionality
#[test]
fn phpt_strpbrk_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strpbrk() : basic functionality ***\n";

// Initialise all required variables
$text = 'This is a Simple text.';
var_dump( strpbrk($text, 'mi') );
var_dump( strpbrk($text, 'ZS') );
var_dump( strpbrk($text, 'Z') );
var_dump( strpbrk($text, 'H') );

$text = '';
var_dump( strpbrk($text, 'foo') );

$text = "  aaa aaaSLR";
var_dump( strpbrk($text, '     ') );

var_dump( strpbrk(5, 5) );
var_dump( strpbrk(5, "5") );

?>
"#);
    assert_eq!(out, "*** Testing strpbrk() : basic functionality ***\nstring(20) \"is is a Simple text.\"\nstring(12) \"Simple text.\"\nbool(false)\nbool(false)\nbool(false)\nstring(12) \"  aaa aaaSLR\"\nstring(1) \"5\"\nstring(1) \"5\"\n");
}

/// php-src `ext/standard/tests/strings/strpos_number.phpt`: strpos() matching numbers
#[test]
fn phpt_strpos_number() {
    let out = compile_and_run(r#"<?php
// Integer is handles as an octal representation, so nothing to match
var_dump(strpos("foo 11", 11));
// int(111) is string("o") in octal. Match expected
var_dump(strpos("foo bar", 111));
// string("11") is contained
var_dump(strpos("foo 11", "11"));
?>
"#);
    assert_eq!(out, "int(4)\nbool(false)\nint(4)\n");
}

/// php-src `ext/standard/tests/strings/strpos_variation1.phpt`: Test strpos() function : usage variations - complex strings containing other than 7-bit chars
#[test]
fn phpt_strpos_variation1() {
    let out = compile_and_run(r#"<?php
$string = chr(0).chr(128).chr(129).chr(234).chr(235).chr(254).chr(255);
$stringAsHex = bin2hex($string);
echo "-- Positions of some chars in the string '$stringAsHex' are as follows --\n";
echo bin2hex( chr(128) ) ." => ";
var_dump( strpos($string, chr(128)) );
echo bin2hex( chr(255) ) ." => ";
var_dump( strpos($string, chr(255), 3) );
echo bin2hex( chr(0) ) ." => ";
var_dump( strpos($string, chr(0)) );
?>

DONE
"#);
    assert_eq!(out, "-- Positions of some chars in the string '008081eaebfeff' are as follows --\n80 => int(1)\nff => int(6)\n00 => int(0)\n\nDONE\n");
}

/// php-src `ext/standard/tests/strings/strrchr.phpt`: strrchr() tests
#[test]
fn phpt_strrchr() {
    let out = compile_and_run(r#"<?php

var_dump(strrchr("", ""));
var_dump(strrchr("abc", ""));
var_dump(strrchr("", "abc"));
var_dump(strrchr("abc", "abc"));
var_dump(strrchr("test ".chr(0)." test", " "));
var_dump(strrchr("test".chr(0)."string", "t"));

echo "Done\n";
?>
"#);
    assert_eq!(out, "bool(false)\nbool(false)\nbool(false)\nstring(3) \"abc\"\nstring(5) \" test\"\nstring(5) \"tring\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/strrchr_variation2.phpt`: Test strrchr() function : usage variations - single quoted strings
#[test]
fn phpt_strrchr_variation2() {
    let out = compile_and_run(r#"<?php
/* Test strrchr() function by passing various single quoted strings to 'haystack' & 'needle' */

echo "*** Testing strrchr() function: with various single quoted strings ***";
$haystack = 'Hello,\t\n\0\n  $&!#%\o,()*+-./:;<=>?@hello123456he \x234 \101 ';
$needle = array(
  //regular strings
  'l',
  'L',
  'HELLO',
  'hEllo',

  //escape characters
  '\t',
  '\T',
  '     ',
  '\n',
  '\N',
  '
',  //new line

  //nulls
  '\0',

  //boolean false
  FALSE,
  false,

  //empty string
  '',

  //special chars
  ' ',
  '$',
  ' $',
  '&',
  '!#',
  '%\o',
  '\o,',
  '()',
  '*+',
  '+',
  '-',
  '.',
  '.;',
  ':;',
  ';',
  '<=>',
  '>',
  '=>',
  '?',
  '@',
  '@hEllo',

  '12345', //decimal numeric string
  '\x23',  //hexadecimal numeric string
  '#',  //hexadecimal numeric string
  '\101',  //octal numeric string
  'A',
  '456HEE',  //numerics + chars
  42, //needle as int(ASCII value of '*')
  $haystack  //haystack as needle
);

/* loop through to get the position of the needle in haystack string */
$count = 1;
for($index=0; $index<count($needle); $index++) {
  echo "\n-- Iteration $count --\n";
  var_dump( strrchr($haystack, $needle[$index]) );
  $count ++;
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrchr() function: with various single quoted strings ***\n-- Iteration 1 --\nstring(22) \"lo123456he \\x234 \\101 \"\n\n-- Iteration 2 --\nbool(false)\n\n-- Iteration 3 --\nstring(63) \"Hello,\\t\\n\\0\\n  $&!#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 4 --\nstring(14) \"he \\x234 \\101 \"\n\n-- Iteration 5 --\nstring(5) \"\\101 \"\n\n-- Iteration 6 --\nstring(5) \"\\101 \"\n\n-- Iteration 7 --\nstring(1) \" \"\n\n-- Iteration 8 --\nstring(5) \"\\101 \"\n\n-- Iteration 9 --\nstring(5) \"\\101 \"\n\n-- Iteration 10 --\nbool(false)\n\n-- Iteration 11 --\nstring(5) \"\\101 \"\n\n-- Iteration 12 --\nbool(false)\n\n-- Iteration 13 --\nbool(false)\n\n-- Iteration 14 --\nbool(false)\n\n-- Iteration 15 --\nstring(1) \" \"\n\n-- Iteration 16 --\nstring(47) \"$&!#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 17 --\nstring(1) \" \"\n\n-- Iteration 18 --\nstring(46) \"&!#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 19 --\nstring(45) \"!#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 20 --\nstring(43) \"%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 21 --\nstring(5) \"\\101 \"\n\n-- Iteration 22 --\nstring(39) \"()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 23 --\nstring(37) \"*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 24 --\nstring(36) \"+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 25 --\nstring(35) \"-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 26 --\nstring(34) \"./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 27 --\nstring(34) \"./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 28 --\nstring(32) \":;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 29 --\nstring(31) \";<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 30 --\nstring(30) \"<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 31 --\nstring(28) \">?@hello123456he \\x234 \\101 \"\n\n-- Iteration 32 --\nstring(29) \"=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 33 --\nstring(27) \"?@hello123456he \\x234 \\101 \"\n\n-- Iteration 34 --\nstring(26) \"@hello123456he \\x234 \\101 \"\n\n-- Iteration 35 --\nstring(26) \"@hello123456he \\x234 \\101 \"\n\n-- Iteration 36 --\nstring(2) \"1 \"\n\n-- Iteration 37 --\nstring(5) \"\\101 \"\n\n-- Iteration 38 --\nstring(44) \"#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n\n-- Iteration 39 --\nstring(5) \"\\101 \"\n\n-- Iteration 40 --\nbool(false)\n\n-- Iteration 41 --\nstring(7) \"4 \\101 \"\n\n-- Iteration 42 --\nstring(7) \"4 \\101 \"\n\n-- Iteration 43 --\nstring(63) \"Hello,\\t\\n\\0\\n  $&!#%\\o,()*+-./:;<=>?@hello123456he \\x234 \\101 \"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrchr_variation3.phpt`: Test strrchr() function : usage variations - multi line heredoc string for 'haystack'
#[test]
fn phpt_strrchr_variation3() {
    let out = compile_and_run(r#"<?php
/* Test strrchr() function by passing multi-line heredoc string for haystack and
 *    with various needles
*/

echo "*** Testing strrchr() function: with heredoc strings ***\n";
$multi_line_str = <<<EOD
Example of string
spanning multiple lines
using heredoc syntax.
EOD;

$needles = array(
  "ing",
  "",
  " ",
  $multi_line_str //needle as haystack
);

//loop through to test strrchr() with each needle
foreach($needles as $needle) {
  var_dump( strrchr($multi_line_str, $needle) );
}

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrchr() function: with heredoc strings ***\nstring(19) \"ing heredoc syntax.\"\nbool(false)\nstring(8) \" syntax.\"\nstring(63) \"Example of string\nspanning multiple lines\nusing heredoc syntax.\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrchr_variation4.phpt`: Test strrchr() function : usage variations - heredoc string containing special chars for 'haystack'
#[test]
fn phpt_strrchr_variation4() {
    let out = compile_and_run(r#"<?php
/* Test strrchr() function by passing heredoc string containing special chars for haystack
 * and with various needles
*/

echo "*** Testing strrchr() function: with heredoc strings ***\n";
$special_chars_str = <<<EOD
Example of heredoc string contains
$#%^*&*_("_")!#@@!$#$^^&*(special)
chars.
EOD;

$heredoc_needle = <<<EOD
^^&*(
EOD;

$needles = array(
  "!@@!",
  '_',
  '("_")',
  "$*",
  "(special)",
  $heredoc_needle,  //needle as heredoc string
  $special_chars_str  //needle as haystack
);

//loop through to test strrchr() with each needle
foreach($needles as $needle) {
  var_dump( strrchr($special_chars_str, $needle) );
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrchr() function: with heredoc strings ***\nstring(24) \"!$#$^^&*(special)\nchars.\"\nstring(31) \"_\")!#@@!$#$^^&*(special)\nchars.\"\nstring(16) \"(special)\nchars.\"\nstring(21) \"$^^&*(special)\nchars.\"\nstring(16) \"(special)\nchars.\"\nstring(19) \"^&*(special)\nchars.\"\nstring(76) \"Example of heredoc string contains\n$#%^*&*_(\"_\")!#@@!$#$^^&*(special)\nchars.\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrchr_variation6.phpt`: Test strrchr() function : usage variations - heredoc string containing quote chars for 'haystack'
#[test]
fn phpt_strrchr_variation6() {
    let out = compile_and_run(r#"<?php
/* Test strrchr() function by passing heredoc string containing quote chars for haystack
 *  and with various needles
*/

echo "*** Testing strrchr() function: with heredoc strings ***\n";
$quote_char_str = <<<EOD
"things" "in" "double" "quote"
'things' 'in' 'single' 'quote'
EOD;

$needles = array(
  "things",
  "\"things\"",
  "\'things\'",
  "in",
  "quote",
  $quote_char_str //needle as haystack
);

//loop through to test strrchr() with each needle
foreach($needles as $needle) {
  var_dump( strrchr($quote_char_str, $needle) );
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrchr() function: with heredoc strings ***\nstring(3) \"te'\"\nstring(32) \"\"\n'things' 'in' 'single' 'quote'\"\nbool(false)\nstring(14) \"ingle' 'quote'\"\nstring(6) \"quote'\"\nstring(32) \"\"\n'things' 'in' 'single' 'quote'\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrchr_variation7.phpt`: Test strrchr() function : usage variations - heredoc string containing blank line for 'haystack'
#[test]
fn phpt_strrchr_variation7() {
    let out = compile_and_run(r#"<?php
/* Test strrchr() function by passing heredoc string containing
 *  blank-line for haystack and with various needles
*/

echo "*** Testing strrchr() function: with heredoc strings ***\n";
$blank_line = <<<EOD

EOD;

$needles = array(
  "\n",
  '\n',
  "\r",
  "\r\n",
  "\t",
  "",
  $blank_line //needle as haystack
);

//loop through to test strrchr() with each needle
foreach($needles as $needle) {
  var_dump( strrchr($blank_line, $needle) );
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrchr() function: with heredoc strings ***\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrev.phpt`: strrev() function
#[test]
fn phpt_strrev() {
    let out = compile_and_run(r#"<?php
$i = 0;
$str = '';

while ($i<256) {
    $str .= chr($i++);
}

var_dump(md5(strrev($str)));
var_dump(strrev(""));
?>
"#);
    assert_eq!(out, "string(32) \"ec6df70f2569891eae50321a9179eb82\"\nstring(0) \"\"\n");
}

/// php-src `ext/standard/tests/strings/strrev_basic.phpt`: Test strrev() function : basic functionality
#[test]
fn phpt_strrev_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strrev() : basic functionality ***\n";
$heredoc = <<<EOD
Hello, world
EOD;

//regular string
var_dump( strrev("Hello, World") );
var_dump( strrev('Hello, World') );

//single character
var_dump( strrev("H") );
var_dump( strrev('H') );

//string containing simalr chars
var_dump( strrev("HHHHHH") );
var_dump( strrev("HhhhhH") );

//string containing escape char
var_dump( strrev("Hello, World\n") );
var_dump( strrev('Hello, World\n') );

//heredoc string
var_dump( strrev($heredoc) );
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrev() : basic functionality ***\nstring(12) \"dlroW ,olleH\"\nstring(12) \"dlroW ,olleH\"\nstring(1) \"H\"\nstring(1) \"H\"\nstring(6) \"HHHHHH\"\nstring(6) \"HhhhhH\"\nstring(13) \"\ndlroW ,olleH\"\nstring(14) \"n\\dlroW ,olleH\"\nstring(12) \"dlrow ,olleH\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrev_variation2.phpt`: Test strrev() function : usage variations - single quoted strings
#[test]
fn phpt_strrev_variation2() {
    let out = compile_and_run(r#"<?php
/* Testing strrev() function with various single quoted strings for 'str' */

echo "*** Testing strrev() : with various single quoted strings ***\n";
$str = 'Hiiii';

$strings = array(
  //strings containing escape chars
  'hello\\world',
  'hello\$world',
  '\ttesting\ttesting\tstrrev',
  'testing\rstrrev testing strrev',
  'testing\fstrrev \f testing \nstrrev',
  '\ntesting\nstrrev\n testing \n strrev',
  'using\vvertical\vtab',

  //polyndrome strings
  'HelloolleH',
  'ababababababa',

  //numeric + strings
  'Hello123',
  '123Hello',
  '123.34Hello',
  'Hello1.234',
  '123Hello1.234',

  //concatenated strings
  'Hello'.chr(0).'World',
  'Hello'.'world',
  'Hello'.$str,

  //strings containing white spaces
  '              h',
  'h              ',
  '      h        ',
  'h  e  l  l  o  ',

  //strings containing quotes
  '\hello\'world',
  'hello\"world',

  //special chars in string
  't@@#$% %test ^test &test *test +test -test',
  '!test ~test `test` =test= @test@test.com',
  '/test/r\test\strrev\t\u /uu/',

  //only special chars
  '!@#$%^&*()_+=-`~'
);

$count = 1;
for( $index = 0; $index < count($strings); $index++ ) {
  echo "\n-- Iteration $count --\n";
  var_dump( strrev($strings[$index]) );
  $count ++;
}

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrev() : with various single quoted strings ***\n\n-- Iteration 1 --\nstring(11) \"dlrow\\olleh\"\n\n-- Iteration 2 --\nstring(12) \"dlrow$\\olleh\"\n\n-- Iteration 3 --\nstring(26) \"verrtst\\gnitsett\\gnitsett\\\"\n\n-- Iteration 4 --\nstring(30) \"verrts gnitset verrtsr\\gnitset\"\n\n-- Iteration 5 --\nstring(35) \"verrtsn\\ gnitset f\\ verrtsf\\gnitset\"\n\n-- Iteration 6 --\nstring(37) \"verrts n\\ gnitset n\\verrtsn\\gnitsetn\\\"\n\n-- Iteration 7 --\nstring(20) \"batv\\lacitrevv\\gnisu\"\n\n-- Iteration 8 --\nstring(10) \"HelloolleH\"\n\n-- Iteration 9 --\nstring(13) \"ababababababa\"\n\n-- Iteration 10 --\nstring(8) \"321olleH\"\n\n-- Iteration 11 --\nstring(8) \"olleH321\"\n\n-- Iteration 12 --\nstring(11) \"olleH43.321\"\n\n-- Iteration 13 --\nstring(10) \"432.1olleH\"\n\n-- Iteration 14 --\nstring(13) \"432.1olleH321\"\n\n-- Iteration 15 --\nstring(11) \"dlroW\u{0}olleH\"\n\n-- Iteration 16 --\nstring(10) \"dlrowolleH\"\n\n-- Iteration 17 --\nstring(10) \"iiiiHolleH\"\n\n-- Iteration 18 --\nstring(15) \"h              \"\n\n-- Iteration 19 --\nstring(15) \"              h\"\n\n-- Iteration 20 --\nstring(15) \"        h      \"\n\n-- Iteration 21 --\nstring(15) \"  o  l  l  e  h\"\n\n-- Iteration 22 --\nstring(12) \"dlrow'olleh\\\"\n\n-- Iteration 23 --\nstring(12) \"dlrow\"\\olleh\"\n\n-- Iteration 24 --\nstring(42) \"tset- tset+ tset* tset& tset^ tset% %$#@@t\"\n\n-- Iteration 25 --\nstring(40) \"moc.tset@tset@ =tset= `tset` tset~ tset!\"\n\n-- Iteration 26 --\nstring(28) \"/uu/ u\\t\\verrts\\tset\\r/tset/\"\n\n-- Iteration 27 --\nstring(16) \"~`-=+_)(*&^%$#@!\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrev_variation3.phpt`: Test strrev() function : usage variations - heredoc strings
#[test]
fn phpt_strrev_variation3() {
    let out = compile_and_run(r#"<?php
/* Testing strrev() function with heredoc strings for 'str' */

echo "*** Testing strrev() function: with heredoc strings ***\n";
$multi_line_str = <<<EOD
Example of string
spanning multiple lines
using heredoc syntax.
EOD;

$special_chars_str = <<<EOD
Ex'ple of h'doc st'g, contains
$#%^*&*_("_")!#@@!$#$^^&*(special)
chars.
EOD;

$control_chars_str = <<<EOD
Hello, World\n
Hello\0World
EOD;

$quote_chars_str = <<<EOD
it's bright o'side
"things in double quote"
'things in single quote'
this\line is /with\slashs
EOD;

$blank_line = <<<EOD

EOD;

$empty_str = <<<EOD
EOD;

$strings = array(
  $multi_line_str,
  $special_chars_str,
  $control_chars_str,
  $quote_chars_str,
  $blank_line,
  $empty_str
);

$count = 1;
for( $index = 0; $index < count($strings); $index++ ) {
  echo "\n-- Iteration $count --\n";
  var_dump( strrev($strings[$index]) );
  $count ++;
}

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrev() function: with heredoc strings ***\n\n-- Iteration 1 --\nstring(63) \".xatnys codereh gnisu\nsenil elpitlum gninnaps\ngnirts fo elpmaxE\"\n\n-- Iteration 2 --\nstring(72) \".srahc\n)laiceps(*&^^$#$!@@#!)\"_\"(_*&*^%#$\nsniatnoc ,g'ts cod'h fo elp'xE\"\n\n-- Iteration 3 --\nstring(25) \"dlroW\u{0}olleH\n\ndlroW ,olleH\"\n\n-- Iteration 4 --\nstring(94) \"shsals\\htiw/ si enil\\siht\n'etouq elgnis ni sgniht'\n\"etouq elbuod ni sgniht\"\nedis'o thgirb s'ti\"\n\n-- Iteration 5 --\nstring(0) \"\"\n\n-- Iteration 6 --\nstring(0) \"\"\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_basic1.phpt`: Test strrpos() function : basic functionality - with default arguments
#[test]
fn phpt_strrpos_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strrpos() function: basic functionality ***\n";
$heredoc_str = <<<EOD
Hello, World
EOD;

echo "-- With default arguments --\n";
//regular string for haystack & needle
var_dump( strrpos("Hello, World", "Hello") );
var_dump( strrpos('Hello, World', "hello") );
var_dump( strrpos("Hello, World", 'World') );
var_dump( strrpos('Hello, World', 'WORLD') );

//single char for needle
var_dump( strrpos("Hello, World", "o") );
var_dump( strrpos("Hello, World", ",") );

//heredoc string for haystack & needle
var_dump( strrpos($heredoc_str, "Hello, World") );
var_dump( strrpos($heredoc_str, 'Hello') );
var_dump( strrpos($heredoc_str, $heredoc_str) );

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: basic functionality ***\n-- With default arguments --\nint(0)\nbool(false)\nint(7)\nbool(false)\nint(8)\nint(5)\nint(0)\nint(0)\nint(0)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_basic2.phpt`: Test strrpos() function : basic functionality - with all arguments
#[test]
fn phpt_strrpos_basic2() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strrpos() function: basic functionality ***\n";
$heredoc_str = <<<EOD
Hello, World
EOD;

echo "-- With all arguments --\n";
//regular string for haystack & needle, with various offsets
var_dump( strrpos("Hello, World", "Hello", 0) );
var_dump( strrpos("Hello, World", 'Hello', 1) );
var_dump( strrpos('Hello, World', 'World', 1) );
var_dump( strrpos('Hello, World', "World", 5) );

//heredoc string for haystack & needle, with various offsets
var_dump( strrpos($heredoc_str, "Hello, World", 0) );
var_dump( strrpos($heredoc_str, 'Hello', 0) );
var_dump( strrpos($heredoc_str, 'Hello', 1) );
var_dump( strrpos($heredoc_str, $heredoc_str, 0) );
var_dump( strrpos($heredoc_str, $heredoc_str, 1) );

//various offsets
var_dump( strrpos("Hello, World", "o", 3) );
var_dump( strrpos("Hello, World", "o", 6) );
var_dump( strrpos("Hello, World", "o", 10) );
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: basic functionality ***\n-- With all arguments --\nint(0)\nbool(false)\nint(7)\nint(7)\nint(0)\nint(0)\nbool(false)\nint(0)\nbool(false)\nint(8)\nint(8)\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation12.phpt`: Test strrpos() function : usage variations - checking binary safe with 'haystack' argument
#[test]
fn phpt_strrpos_variation12() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function with null terminated strings for 'haystack' argument
 *  in order to check the binary safe
*/

echo "*** Test strrpos() function: binary safe ***\n";
$haystacks = array(
  "Hello".chr(0)."World",
  chr(0)."Hello World",
  "Hello World".chr(0),
  chr(0).chr(0).chr(0),
  "Hello\0world",
  "\0Hello",
  "Hello\0"
);

for($index = 0; $index < count($haystacks); $index++ ) {
  var_dump( strrpos($haystacks[$index], "\0") );
  var_dump( strrpos($haystacks[$index], "\0", $index) );
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Test strrpos() function: binary safe ***\nint(5)\nint(5)\nint(0)\nbool(false)\nint(11)\nint(11)\nint(2)\nbool(false)\nint(5)\nint(5)\nint(0)\nbool(false)\nint(5)\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation13.phpt`: Test strrpos() function : usage variations - checking bianry safe with 'needle' argument
#[test]
fn phpt_strrpos_variation13() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function with null terminated strings for 'needle' argument
 *  in order to check binary safe
*/

echo "*** Test strrpos() function: binary safe ***\n";
$haystack = "\0Hello\0World\0";

$needles = array(
  "Hello".chr(0)."World",
  chr(0)."Hello\0",
  chr(0),
  "Hello\0world",
  "\0Hello\0world\0",
  "\0Hello",
  "Hello\0"
);

for($index = 0; $index < count($needles); $index++ ) {
  var_dump( strrpos($haystack, $needles[$index]) );
  var_dump( strrpos($haystack, $needles[$index], $index) );
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Test strrpos() function: binary safe ***\nint(1)\nint(1)\nint(0)\nbool(false)\nint(12)\nint(12)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nint(0)\nbool(false)\nint(1)\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation2.phpt`: Test strrpos() function : usage variations - single quoted strings for 'haystack' & 'needle' arguments
#[test]
fn phpt_strrpos_variation2() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function by passing single quoted strings to 'haystack' & 'needle' arguments */

echo "*** Testing strrpos() function: with single quoted strings ***\n";
$haystack = 'Hello,\t\n\0\n  $&!#%()*<=>?@hello123456he \x234 \101 ';
$needle = array(
  //regular strings
  'l',
  'L',
  'HELLO',
  'hEllo',

  //escape characters
  '\t',
  '\T',
  '     ',
  '\n',
  '\N',
  '
',  //new line

  //nulls
  '\0',

  //boolean false
  FALSE,
  false,

  //empty string
  '',

  //special chars
  ' ',
  '$',
  ' $',
  '&',
  '!#',
  '()',
  '<=>',
  '>',
  '=>',
  '?',
  '@',
  '@hEllo',

  '12345', //decimal numeric string
  '\x23',  //hexadecimal numeric string
  '#',  //hexadecimal numeric string
  '\101',  //octal numeric string
  'A',
  '456HEE',  //numerics + chars
  42, //needle as int(ASCII value of '*')
  $haystack  //haystack as needle
);

/* loop through to get the position of the needle in haystack string */
$count = 1;
for($index=0; $index<count($needle); $index++) {
  echo "-- Iteration $count --\n";
  var_dump( strrpos($haystack, $needle[$index]) );
  var_dump( strrpos($haystack, $needle[$index], $index) );
  $count++;
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: with single quoted strings ***\n-- Iteration 1 --\nint(32)\nint(32)\n-- Iteration 2 --\nbool(false)\nbool(false)\n-- Iteration 3 --\nbool(false)\nbool(false)\n-- Iteration 4 --\nbool(false)\nbool(false)\n-- Iteration 5 --\nint(6)\nint(6)\n-- Iteration 6 --\nbool(false)\nbool(false)\n-- Iteration 7 --\nbool(false)\nbool(false)\n-- Iteration 8 --\nint(12)\nint(12)\n-- Iteration 9 --\nbool(false)\nbool(false)\n-- Iteration 10 --\nbool(false)\nbool(false)\n-- Iteration 11 --\nint(10)\nint(10)\n-- Iteration 12 --\nint(54)\nint(54)\n-- Iteration 13 --\nint(54)\nint(54)\n-- Iteration 14 --\nint(54)\nint(54)\n-- Iteration 15 --\nint(53)\nint(53)\n-- Iteration 16 --\nint(16)\nint(16)\n-- Iteration 17 --\nint(15)\nbool(false)\n-- Iteration 18 --\nint(17)\nint(17)\n-- Iteration 19 --\nint(18)\nint(18)\n-- Iteration 20 --\nint(21)\nint(21)\n-- Iteration 21 --\nint(24)\nint(24)\n-- Iteration 22 --\nint(26)\nint(26)\n-- Iteration 23 --\nint(25)\nint(25)\n-- Iteration 24 --\nint(27)\nint(27)\n-- Iteration 25 --\nint(28)\nint(28)\n-- Iteration 26 --\nbool(false)\nbool(false)\n-- Iteration 27 --\nint(34)\nint(34)\n-- Iteration 28 --\nint(43)\nint(43)\n-- Iteration 29 --\nint(19)\nbool(false)\n-- Iteration 30 --\nint(49)\nint(49)\n-- Iteration 31 --\nbool(false)\nbool(false)\n-- Iteration 32 --\nbool(false)\nbool(false)\n-- Iteration 33 --\nbool(false)\nbool(false)\n-- Iteration 34 --\nint(0)\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation3.phpt`: Test strrpos() function : usage variations - multi line heredoc string for 'haystack' argument
#[test]
fn phpt_strrpos_variation3() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function by passing multi-line heredoc string for haystack and
 *  with various needles & offsets
*/

echo "*** Testing strrpos() function: with heredoc strings ***\n";
echo "-- With heredoc string containing multi lines --\n";
$multi_line_str = <<<EOD
Example of string
spanning multiple lines
using heredoc syntax.
EOD;
var_dump( strrpos($multi_line_str, "ing", 0) );
var_dump( strrpos($multi_line_str, "ing", 15) );
var_dump( strrpos($multi_line_str, "ing", 22) );
var_dump( strrpos($multi_line_str, "") );
var_dump( strrpos($multi_line_str, " ") );

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: with heredoc strings ***\n-- With heredoc string containing multi lines --\nint(44)\nint(44)\nint(44)\nint(63)\nint(55)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation4.phpt`: Test strrpos() function : usage variations - heredoc string containing special chars for 'haystack' argument
#[test]
fn phpt_strrpos_variation4() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function by passing heredoc string containing special chars for haystack
 *  and with various needles & offsets
*/

echo "*** Testing strrpos() function: with heredoc strings ***\n";
echo "-- With heredoc string containing special chars --\n";
$special_chars_str = <<<EOD
Ex'ple of h'doc st'g, contains
$#%^*&*_("_")!#@@!$#$^^&$*(special)
chars.
EOD;
var_dump( strrpos($special_chars_str, "Ex'ple", 0) );
var_dump( strrpos($special_chars_str, "!@@!", 23) );
var_dump( strrpos($special_chars_str, '_') );
var_dump( strrpos($special_chars_str, '("_")') );
var_dump( strrpos($special_chars_str, "$*") );
var_dump( strrpos($special_chars_str, "$*", 10) );
var_dump( strrpos($special_chars_str, "(special)") );

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: with heredoc strings ***\n-- With heredoc string containing special chars --\nint(0)\nbool(false)\nint(41)\nint(39)\nint(55)\nint(55)\nint(57)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation5.phpt`: Test strrpos() function : usage variations - heredoc string containing escape chars for 'haystack' argument
#[test]
fn phpt_strrpos_variation5() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function by passing heredoc string containing escape chars for haystack
 *  and with various needles & offsets
*/

echo "*** Testing strrpos() function: with heredoc strings ***\n";
echo "-- With heredoc string containing escape characters --\n";
$control_char_str = <<<EOD
Hello, World\n
Hello\tWorld
EOD;
var_dump( strrpos($control_char_str, "\n") );
var_dump( strrpos($control_char_str, "\t") );
var_dump( strrpos($control_char_str, "\n", 12) );
var_dump( strrpos($control_char_str, "\t", 15) );

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: with heredoc strings ***\n-- With heredoc string containing escape characters --\nint(13)\nint(19)\nint(13)\nint(19)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation6.phpt`: Test strrpos() function : usage variations - heredoc string containing quotes for 'haystack' argument
#[test]
fn phpt_strrpos_variation6() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function by passing heredoc string containing quotes for haystack
 *  and with various needles & offsets
*/

echo "*** Testing strrpos() function: with heredoc strings ***\n";
echo "-- With heredoc string containing quote & slash chars --\n";
$quote_char_str = <<<EOD
it's bright,but i cann't see it.
"things in double quote"
'things in single quote'
this\line is /with\slashs
EOD;
var_dump( strrpos($quote_char_str, "line") );
var_dump( strrpos($quote_char_str, 'things') );
var_dump( strrpos($quote_char_str, 'things', 0) );
var_dump( strrpos($quote_char_str, "things", 20) );
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: with heredoc strings ***\n-- With heredoc string containing quote & slash chars --\nint(88)\nint(59)\nint(59)\nint(59)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strrpos_variation8.phpt`: Test strrpos() function : usage variations - repetitive chars for 'haystack' argument
#[test]
fn phpt_strrpos_variation8() {
    let out = compile_and_run(r#"<?php
/* Test strrpos() function with strings containing multiple occurrences of 'needle' in the 'haystack'
 *  and with various needles & offsets
*/

echo "*** Testing strrpos() function: strings repetitive chars ***\n";
$haystack = "ababababAbaBa";
$needle = "aba";

/* loop through to consider various offsets in getting the position of the needle in haystack string */
$count = 1;
for($offset = -1; $offset <= strlen($haystack); $offset++ ) {
  echo "-- Iteration $count --\n";
  var_dump( strrpos($haystack, $needle, $offset) );
  $count++;
}
echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strrpos() function: strings repetitive chars ***\n-- Iteration 1 --\nint(4)\n-- Iteration 2 --\nint(4)\n-- Iteration 3 --\nint(4)\n-- Iteration 4 --\nint(4)\n-- Iteration 5 --\nint(4)\n-- Iteration 6 --\nint(4)\n-- Iteration 7 --\nbool(false)\n-- Iteration 8 --\nbool(false)\n-- Iteration 9 --\nbool(false)\n-- Iteration 10 --\nbool(false)\n-- Iteration 11 --\nbool(false)\n-- Iteration 12 --\nbool(false)\n-- Iteration 13 --\nbool(false)\n-- Iteration 14 --\nbool(false)\n-- Iteration 15 --\nbool(false)\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strspn.phpt`: Test strspn() behavior
#[test]
fn phpt_strspn() {
    let out = compile_and_run(r#"<?php
$a = "22222222aaaa bbb1111 cccc";
$b = "1234";
var_dump($a);
var_dump($b);
var_dump(strspn($a,$b));
var_dump(strspn($a,$b,2));
var_dump(strspn($a,$b,2,3));
?>
"#);
    assert_eq!(out, "string(25) \"22222222aaaa bbb1111 cccc\"\nstring(4) \"1234\"\nint(8)\nint(6)\nint(3)\n");
}

/// php-src `ext/standard/tests/strings/strspn_basic.phpt`: Test strspn() function : basic functionality
#[test]
fn phpt_strspn_basic() {
    let out = compile_and_run(r#"<?php
/*
* Testing strspn() : basic functionality
*/

echo "*** Testing strspn() : basic functionality ***\n";


// Initialise all required variables
$str = "this is the test string";
$mask = "htes ";
$start = 8;
$len = 12;

// Calling strspn() with all possible arguments
var_dump( strspn($str, $mask, $start, $len) );

// Calling strspn() with three arguments and default len argument
var_dump( strspn($str, $mask, $start) );

// Calling strspn() with default arguments
var_dump( strspn($str, $mask) );

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strspn() : basic functionality ***\nint(11)\nint(11)\nint(2)\nDone");
}

/// php-src `ext/standard/tests/strings/strspn_variation10.phpt`: Test strspn() function : usage variations - with varying mask & default start and len args
#[test]
fn phpt_strspn_variation10() {
    let out = compile_and_run(r#"<?php
/*
* Testing strspn() : with varying mask and default start and len arguments
*/

echo "*** Testing strspn() : with different mask strings and default start and len arguments ***\n";

// initialing required variables
// defining different strings
$strings = array(
                   "",
           '',
           "\n",
           '\n',
           "hello\tworld\nhello\nworld\n",
           'hello\tworld\nhello\nworld\n',
           "1234hello45world\t123",
           '1234hello45world\t123',
           "hello\0world\012",
           'hello\0world\012',
           chr(0).chr(0),
           chr(0)."hello\0world".chr(0),
           chr(0).'hello\0world'.chr(0),
           "hello".chr(0)."world",
           'hello'.chr(0).'world',
           "hello\0\100\xaaaworld",
           'hello\0\100\xaaaworld'
                   );

// define the array of mask strings
$mask_array = array(
            "",
            '',
            "f\n\trelshti \l",
            'f\n\trelsthi \l',
            "\telh",
            "t\ ",
            '\telh',
            "felh\t\ ",
            " \t",
                    "fhel\t\i\100\xa"
                   );


// loop through each element of the array for mask argument

$count = 1;
foreach($strings as $str)  {
  echo "\n-- Iteration $count --\n";
  foreach($mask_array as $mask)  {
      var_dump( strspn($str,$mask) );
  }
  $count++;
}

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strspn() : with different mask strings and default start and len arguments ***\n\n-- Iteration 1 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 2 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 3 --\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\n\n-- Iteration 4 --\nint(0)\nint(0)\nint(1)\nint(2)\nint(0)\nint(1)\nint(1)\nint(1)\nint(0)\nint(1)\n\n-- Iteration 5 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 6 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 7 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 8 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 9 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 10 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 11 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 12 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 13 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 14 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 15 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 16 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\n\n-- Iteration 17 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(4)\nint(0)\nint(4)\nint(4)\nint(0)\nint(4)\nDone");
}

/// php-src `ext/standard/tests/strings/strspn_variation11.phpt`: Test strspn() function : usage variations - with varying start and default len args
#[test]
fn phpt_strspn_variation11() {
    let out = compile_and_run(r#"<?php
/*
* Testing strspn() : with varying start and default len arguments
*/

echo "*** Testing strspn() : with different start and default len values ***\n";

// initialing required variables
// defining different strings
$strings = array(
                   "",
           '',
           "\n",
           '\n',
           "hello\tworld\nhello\nworld\n",
           'hello\tworld\nhello\nworld\n',
           "1234hello45world\t123",
           '1234hello45world\t123',
           "hello\0world\012",
           'hello\0world\012',
           chr(0).chr(0),
           chr(0)."hello\0world".chr(0),
           chr(0).'hello\0world'.chr(0),
           "hello".chr(0)."world",
           'hello'.chr(0).'world',
           "hello\0\100\xaaaworld",
           'hello\0\100\xaaaworld'
                   );

// define the array of mask strings
$mask_array = array(
            "",
            '',
            "f\n\trelshti \l",
            'f\n\trelsthi \l',
            "\telh",
            "t\ ",
            '\telh',
            "felh\t\ ",
            " \t",
                    "fhel\t\i\100\xa"
                   );

// defining the array for start values
$start_array = array(
            0,
            1,
                    2,
            -1,
            -2,
            2147483647,  // max positive integer
            -2147483648,  // min negative integer
                   );


// loop through each element of the arrays for str, mask and start argument
$count = 1;
foreach($strings as $str) {
  echo "\n-- Iteration $count --\n";
  foreach($mask_array as $mask) {
    foreach($start_array as $start) {
      var_dump( strspn($str,$mask,$start) );
    }
  }
  $count++;
}

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strspn() : with different start and default len values ***\n\n-- Iteration 1 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 2 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 3 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\n\n-- Iteration 4 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(1)\nint(2)\nint(1)\nint(0)\nint(1)\nint(2)\nint(0)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(1)\nint(1)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(1)\nint(1)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(1)\n\n-- Iteration 5 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(1)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(1)\nint(0)\nint(0)\nint(4)\n\n-- Iteration 6 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(1)\nint(2)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 7 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 8 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 9 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(1)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(1)\nint(0)\nint(0)\nint(4)\n\n-- Iteration 10 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(0)\nint(0)\nint(4)\n\n-- Iteration 11 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 12 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 13 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 14 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 15 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 16 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 17 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(4)\nint(3)\nint(2)\nint(0)\nint(1)\nint(0)\nint(4)\nDone");
}

/// php-src `ext/standard/tests/strings/strspn_variation12.phpt`: Test strspn() function : usage variations - with varying start and len args
#[test]
fn phpt_strspn_variation12() {
    let out = compile_and_run(r#"<?php
/*
* Testing strspn() : with varying start and len arguments
*/

echo "*** Testing strspn() : with different start and len values ***\n";

// initialing required variables
// defining different strings
$strings = array(
                   "",
           '',
           "\n",
           '\n',
           "hello\tworld\nhello\nworld\n",
           "1234hello45world\t123",
           "hello\0world\012",
           chr(0).chr(0),
           chr(0)."hello\0world".chr(0),
           "hello".chr(0)."world",
           "hello\0\100\xaaaworld",
                   );

// define the array of mask strings
$mask_array = array(
            "",
            '',
            "f\n\trelshti \l",
            'f\n\trelsthi \l',
            "\telh",
            "felh\t\ ",
                    "fhel\t\i\100\xa"
                   );

// defining the array for start values
$start_array = array(
            0,
            1,
            2,
            -1,
            2147483647,  // max positive integer
            -2147483648,  // min negative integer
                   );

// defining an array of len values
$len_array = array(
            0,
            1,
            2,
            -1,
            2147483647,  // max positive integer
            -2147483648,  // min negative integer
                   );


// loop through each element of the array for len argument
$count = 1;
foreach($strings as $str) {
  echo "\n-- Iteration $count --\n";
  foreach($mask_array as $mask) {
    foreach($start_array as $start) {
      foreach($len_array as $len) {
        var_dump( strspn($str,$mask,$start,$len) );
      }
    }
  }
  $count++;
}

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strspn() : with different start and len values ***\n\n-- Iteration 1 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 2 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 3 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\n\n-- Iteration 4 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(1)\nint(2)\nint(1)\nint(2)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(1)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(1)\nint(1)\nint(1)\nint(0)\n\n-- Iteration 5 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\n\n-- Iteration 6 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 7 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(1)\nint(1)\nint(0)\nint(1)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\n\n-- Iteration 8 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 9 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 10 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\n\n-- Iteration 11 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nint(0)\nint(1)\nint(2)\nint(3)\nint(3)\nint(0)\nint(0)\nint(1)\nint(2)\nint(2)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(2)\nint(4)\nint(4)\nint(0)\nDone");
}

/// php-src `ext/standard/tests/strings/strspn_variation6.phpt`: Test strspn() function : usage variations - with heredoc strings, varying mask & default start and len args
#[test]
fn phpt_strspn_variation6() {
    let out = compile_and_run(r#"<?php
/*
* Testing strspn() : with heredoc string, varying mask and default start and len arguments
*/

echo "*** Testing strspn() : with different mask strings ***\n";

// initialing required variables
// defining different heredoc strings
$empty_heredoc = <<<EOT
EOT;

$heredoc_with_newline = <<<EOT
\n

EOT;

$heredoc_with_characters = <<<EOT
first line of heredoc string
second line of heredoc string
third line of heredocstring
EOT;

$heredoc_with_newline_and_tabs = <<<EOT
hello\tworld\nhello\nworld\n
EOT;

$heredoc_with_alphanumerics = <<<EOT
hello123world456
1234hello\t1234
EOT;

$heredoc_with_embedded_nulls = <<<EOT
hello\0world\0hello
\0hello\0
EOT;

$heredoc_with_hexa_octal = <<<EOT
hello\0\100\xaaworld\0hello
\0hello\0
EOT;

// defining array of different heredoc strings
$heredoc_strings = array(
                   $empty_heredoc,
                   $heredoc_with_newline,
                   $heredoc_with_characters,
                   $heredoc_with_newline_and_tabs,
                   $heredoc_with_alphanumerics,
                   $heredoc_with_embedded_nulls,
                   $heredoc_with_hexa_octal
                   );

// defining array of different mask strings
$mask_array = array(
            "",
            '',
            "fh\ne\trlsti \l",
            'fieh\n\trlsti \l',
            "\t",
            "lt\ ",
            'l\t',
            "fl\t\eh ",
            "l \te",
                    "lf\the\i\100\xaa"
                   );


// loop through each element of the array for different heredoc and mask strings

$count = 1;

foreach($heredoc_strings as $str)  {
  echo "\n-- Iteration $count --\n";
  foreach($mask_array as $mask)  {
      var_dump( strspn($str,$mask) ); // with default start and len value
  }
  $count++;
}

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing strspn() : with different mask strings ***\n\n-- Iteration 1 --\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 2 --\nint(0)\nint(0)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\n\n-- Iteration 3 --\nint(0)\nint(0)\nint(8)\nint(11)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(2)\n\n-- Iteration 4 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 5 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 6 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(4)\n\n-- Iteration 7 --\nint(0)\nint(0)\nint(4)\nint(4)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nint(4)\nDone");
}

/// php-src `ext/standard/tests/strings/strstr_variation1.phpt`: Test strstr() function : usage variations - complex strings containing other than 7-bit chars
#[test]
fn phpt_strstr_variation1() {
    let out = compile_and_run(r#"<?php
$string = chr(0).chr(128).chr(129).chr(234).chr(235).chr(254).chr(255);
$stringAsHex = bin2hex($string);
echo "-- Positions of some chars in the string '$stringAsHex' are as follows --\n";
echo bin2hex( chr(128) ) ." => ";
var_dump( bin2hex( strstr($string, chr(128) ) ) );
echo bin2hex( chr(255) ) ." => ";
var_dump( bin2hex( strstr($string, chr(255) ) ) );
echo bin2hex( chr(0) ) ." => ";
var_dump( bin2hex( strstr($string, chr(0) ) ) );
?>

DONE
"#);
    assert_eq!(out, "-- Positions of some chars in the string '008081eaebfeff' are as follows --\n80 => string(12) \"8081eaebfeff\"\nff => string(2) \"ff\"\n00 => string(14) \"008081eaebfeff\"\n\nDONE\n");
}

/// php-src `ext/standard/tests/strings/strtolower.phpt`: Test strtolower() function
#[test]
fn phpt_strtolower() {
    let out = compile_and_run(r#"<?php

echo "*** Testing strtolower() with 256 chars ***\n";
for ($i=0; $i<=255; $i++){
  $char = chr($i);
  print(bin2hex($char))." => ".(bin2hex(strtolower("$char")))."\n";
}

echo "*** Testing strlower() with basic strings ***\n";
$str = "Mary Had A liTTle LAmb and ShE loveD IT So\n";
var_dump(strtolower($str));

echo "\n*** Testing strtolower() with various strings ***";
/* strings to pass strtolower() */
$strings = array (
  "",
  "string",
  "stRINg0234",
  "1.233.344StrinG12333",
  "$$$$$$!!!!@@@@@@@ ABCDEF !!!***",
  "ABCD\0abcdABCD",
  TRUE,
  FALSE,

  /* Check for off-by-one errors in the SSE implementation */
  "AAAAAAAAAAAAAAAAAAAA",
  "ZZZZZZZZZZZZZZZZZZZZ",
  "@@@@@@@@@@@@@@@@@@@@",
  "[[[[[[[[[[[[[[[[[[[[",
  "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
);

$count = 0;
/* loop through to check possible variations */
foreach ($strings as $string) {
  echo "\n-- Iteration $count --\n";
  var_dump( strtolower($string) );
  $count++;
}

echo "\n*** Testing strtolower() with two different case strings ***\n";
if (strtolower("HeLLo woRLd") === strtolower("hEllo WORLD"))
  echo "strings are same, with Case Insensitive\n";
else
  echo "strings are not same\n";

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strtolower() with 256 chars ***\n00 => 00\n01 => 01\n02 => 02\n03 => 03\n04 => 04\n05 => 05\n06 => 06\n07 => 07\n08 => 08\n09 => 09\n0a => 0a\n0b => 0b\n0c => 0c\n0d => 0d\n0e => 0e\n0f => 0f\n10 => 10\n11 => 11\n12 => 12\n13 => 13\n14 => 14\n15 => 15\n16 => 16\n17 => 17\n18 => 18\n19 => 19\n1a => 1a\n1b => 1b\n1c => 1c\n1d => 1d\n1e => 1e\n1f => 1f\n20 => 20\n21 => 21\n22 => 22\n23 => 23\n24 => 24\n25 => 25\n26 => 26\n27 => 27\n28 => 28\n29 => 29\n2a => 2a\n2b => 2b\n2c => 2c\n2d => 2d\n2e => 2e\n2f => 2f\n30 => 30\n31 => 31\n32 => 32\n33 => 33\n34 => 34\n35 => 35\n36 => 36\n37 => 37\n38 => 38\n39 => 39\n3a => 3a\n3b => 3b\n3c => 3c\n3d => 3d\n3e => 3e\n3f => 3f\n40 => 40\n41 => 61\n42 => 62\n43 => 63\n44 => 64\n45 => 65\n46 => 66\n47 => 67\n48 => 68\n49 => 69\n4a => 6a\n4b => 6b\n4c => 6c\n4d => 6d\n4e => 6e\n4f => 6f\n50 => 70\n51 => 71\n52 => 72\n53 => 73\n54 => 74\n55 => 75\n56 => 76\n57 => 77\n58 => 78\n59 => 79\n5a => 7a\n5b => 5b\n5c => 5c\n5d => 5d\n5e => 5e\n5f => 5f\n60 => 60\n61 => 61\n62 => 62\n63 => 63\n64 => 64\n65 => 65\n66 => 66\n67 => 67\n68 => 68\n69 => 69\n6a => 6a\n6b => 6b\n6c => 6c\n6d => 6d\n6e => 6e\n6f => 6f\n70 => 70\n71 => 71\n72 => 72\n73 => 73\n74 => 74\n75 => 75\n76 => 76\n77 => 77\n78 => 78\n79 => 79\n7a => 7a\n7b => 7b\n7c => 7c\n7d => 7d\n7e => 7e\n7f => 7f\n80 => 80\n81 => 81\n82 => 82\n83 => 83\n84 => 84\n85 => 85\n86 => 86\n87 => 87\n88 => 88\n89 => 89\n8a => 8a\n8b => 8b\n8c => 8c\n8d => 8d\n8e => 8e\n8f => 8f\n90 => 90\n91 => 91\n92 => 92\n93 => 93\n94 => 94\n95 => 95\n96 => 96\n97 => 97\n98 => 98\n99 => 99\n9a => 9a\n9b => 9b\n9c => 9c\n9d => 9d\n9e => 9e\n9f => 9f\na0 => a0\na1 => a1\na2 => a2\na3 => a3\na4 => a4\na5 => a5\na6 => a6\na7 => a7\na8 => a8\na9 => a9\naa => aa\nab => ab\nac => ac\nad => ad\nae => ae\naf => af\nb0 => b0\nb1 => b1\nb2 => b2\nb3 => b3\nb4 => b4\nb5 => b5\nb6 => b6\nb7 => b7\nb8 => b8\nb9 => b9\nba => ba\nbb => bb\nbc => bc\nbd => bd\nbe => be\nbf => bf\nc0 => c0\nc1 => c1\nc2 => c2\nc3 => c3\nc4 => c4\nc5 => c5\nc6 => c6\nc7 => c7\nc8 => c8\nc9 => c9\nca => ca\ncb => cb\ncc => cc\ncd => cd\nce => ce\ncf => cf\nd0 => d0\nd1 => d1\nd2 => d2\nd3 => d3\nd4 => d4\nd5 => d5\nd6 => d6\nd7 => d7\nd8 => d8\nd9 => d9\nda => da\ndb => db\ndc => dc\ndd => dd\nde => de\ndf => df\ne0 => e0\ne1 => e1\ne2 => e2\ne3 => e3\ne4 => e4\ne5 => e5\ne6 => e6\ne7 => e7\ne8 => e8\ne9 => e9\nea => ea\neb => eb\nec => ec\ned => ed\nee => ee\nef => ef\nf0 => f0\nf1 => f1\nf2 => f2\nf3 => f3\nf4 => f4\nf5 => f5\nf6 => f6\nf7 => f7\nf8 => f8\nf9 => f9\nfa => fa\nfb => fb\nfc => fc\nfd => fd\nfe => fe\nff => ff\n*** Testing strlower() with basic strings ***\nstring(43) \"mary had a little lamb and she loved it so\n\"\n\n*** Testing strtolower() with various strings ***\n-- Iteration 0 --\nstring(0) \"\"\n\n-- Iteration 1 --\nstring(6) \"string\"\n\n-- Iteration 2 --\nstring(10) \"string0234\"\n\n-- Iteration 3 --\nstring(20) \"1.233.344string12333\"\n\n-- Iteration 4 --\nstring(31) \"$$$$$$!!!!@@@@@@@ abcdef !!!***\"\n\n-- Iteration 5 --\nstring(13) \"abcd\u{0}abcdabcd\"\n\n-- Iteration 6 --\nstring(1) \"1\"\n\n-- Iteration 7 --\nstring(0) \"\"\n\n-- Iteration 8 --\nstring(20) \"aaaaaaaaaaaaaaaaaaaa\"\n\n-- Iteration 9 --\nstring(20) \"zzzzzzzzzzzzzzzzzzzz\"\n\n-- Iteration 10 --\nstring(20) \"@@@@@@@@@@@@@@@@@@@@\"\n\n-- Iteration 11 --\nstring(20) \"[[[[[[[[[[[[[[[[[[[[\"\n\n-- Iteration 12 --\nstring(62) \"abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz\"\n\n*** Testing strtolower() with two different case strings ***\nstrings are same, with Case Insensitive\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strtoupper1.phpt`: Test strtoupper() function
#[test]
fn phpt_strtoupper1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strtoupper() with 256 chars ***\n";
for ($i=0; $i<=255; $i++){
  $char = chr($i);
  print(bin2hex($char))." => ".(bin2hex(strtoupper("$char")))."\n";
}

echo "\n*** Testing strtoupper() with basic strings ***\n";
$str = "Mary Had A liTTle LAmb and ShE loveD IT So\n";
var_dump(strtoupper($str));

echo "\n*** Testing strtoupper() with various strings ***";
/* strings to pass strtoupper() */
$strings = array (
  "",
  "string",
  "stRINg0234",
  "1.233.344StrinG12333",
  "$$$$$$!!!!@@@@@@@ ABCDEF !!!***",
  "ABCD\0abcdABCD",
  TRUE,
  FALSE,
  /* Check for off-by-one errors in the SSE implementation */
  "aaaaaaaaaaaaaaaaaaaa",
  "zzzzzzzzzzzzzzzzzzzz",
  "````````````````````",
  "{{{{{{{{{{{{{{{{{{{{",
  /* And the AVX2 implementation also */
  "{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{",
  "abcdefghijklmnopqrstuvwxyz01234",
  "abcdefghijklmnopqrstuvwxyz012345",
  "abcdefghijklmnopqrstuvwxyz0123456",
  "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
);

$count = 0;
/* loop through to check possible variations */
foreach ($strings as $string) {
  echo "\n-- Iteration $count --\n";
  var_dump( strtoupper($string) );
  $count++;
}

echo "\n*** Testing strtoupper() with two different case strings ***\n";
if (strtoupper("HeLLo woRLd") === strtoupper("hEllo WORLD"))
  echo "strings are same, with Case Insensitive\n";
else
  echo "strings are not same\n";

echo "*** Done ***";
?>
"#);
    assert_eq!(out, "*** Testing strtoupper() with 256 chars ***\n00 => 00\n01 => 01\n02 => 02\n03 => 03\n04 => 04\n05 => 05\n06 => 06\n07 => 07\n08 => 08\n09 => 09\n0a => 0a\n0b => 0b\n0c => 0c\n0d => 0d\n0e => 0e\n0f => 0f\n10 => 10\n11 => 11\n12 => 12\n13 => 13\n14 => 14\n15 => 15\n16 => 16\n17 => 17\n18 => 18\n19 => 19\n1a => 1a\n1b => 1b\n1c => 1c\n1d => 1d\n1e => 1e\n1f => 1f\n20 => 20\n21 => 21\n22 => 22\n23 => 23\n24 => 24\n25 => 25\n26 => 26\n27 => 27\n28 => 28\n29 => 29\n2a => 2a\n2b => 2b\n2c => 2c\n2d => 2d\n2e => 2e\n2f => 2f\n30 => 30\n31 => 31\n32 => 32\n33 => 33\n34 => 34\n35 => 35\n36 => 36\n37 => 37\n38 => 38\n39 => 39\n3a => 3a\n3b => 3b\n3c => 3c\n3d => 3d\n3e => 3e\n3f => 3f\n40 => 40\n41 => 41\n42 => 42\n43 => 43\n44 => 44\n45 => 45\n46 => 46\n47 => 47\n48 => 48\n49 => 49\n4a => 4a\n4b => 4b\n4c => 4c\n4d => 4d\n4e => 4e\n4f => 4f\n50 => 50\n51 => 51\n52 => 52\n53 => 53\n54 => 54\n55 => 55\n56 => 56\n57 => 57\n58 => 58\n59 => 59\n5a => 5a\n5b => 5b\n5c => 5c\n5d => 5d\n5e => 5e\n5f => 5f\n60 => 60\n61 => 41\n62 => 42\n63 => 43\n64 => 44\n65 => 45\n66 => 46\n67 => 47\n68 => 48\n69 => 49\n6a => 4a\n6b => 4b\n6c => 4c\n6d => 4d\n6e => 4e\n6f => 4f\n70 => 50\n71 => 51\n72 => 52\n73 => 53\n74 => 54\n75 => 55\n76 => 56\n77 => 57\n78 => 58\n79 => 59\n7a => 5a\n7b => 7b\n7c => 7c\n7d => 7d\n7e => 7e\n7f => 7f\n80 => 80\n81 => 81\n82 => 82\n83 => 83\n84 => 84\n85 => 85\n86 => 86\n87 => 87\n88 => 88\n89 => 89\n8a => 8a\n8b => 8b\n8c => 8c\n8d => 8d\n8e => 8e\n8f => 8f\n90 => 90\n91 => 91\n92 => 92\n93 => 93\n94 => 94\n95 => 95\n96 => 96\n97 => 97\n98 => 98\n99 => 99\n9a => 9a\n9b => 9b\n9c => 9c\n9d => 9d\n9e => 9e\n9f => 9f\na0 => a0\na1 => a1\na2 => a2\na3 => a3\na4 => a4\na5 => a5\na6 => a6\na7 => a7\na8 => a8\na9 => a9\naa => aa\nab => ab\nac => ac\nad => ad\nae => ae\naf => af\nb0 => b0\nb1 => b1\nb2 => b2\nb3 => b3\nb4 => b4\nb5 => b5\nb6 => b6\nb7 => b7\nb8 => b8\nb9 => b9\nba => ba\nbb => bb\nbc => bc\nbd => bd\nbe => be\nbf => bf\nc0 => c0\nc1 => c1\nc2 => c2\nc3 => c3\nc4 => c4\nc5 => c5\nc6 => c6\nc7 => c7\nc8 => c8\nc9 => c9\nca => ca\ncb => cb\ncc => cc\ncd => cd\nce => ce\ncf => cf\nd0 => d0\nd1 => d1\nd2 => d2\nd3 => d3\nd4 => d4\nd5 => d5\nd6 => d6\nd7 => d7\nd8 => d8\nd9 => d9\nda => da\ndb => db\ndc => dc\ndd => dd\nde => de\ndf => df\ne0 => e0\ne1 => e1\ne2 => e2\ne3 => e3\ne4 => e4\ne5 => e5\ne6 => e6\ne7 => e7\ne8 => e8\ne9 => e9\nea => ea\neb => eb\nec => ec\ned => ed\nee => ee\nef => ef\nf0 => f0\nf1 => f1\nf2 => f2\nf3 => f3\nf4 => f4\nf5 => f5\nf6 => f6\nf7 => f7\nf8 => f8\nf9 => f9\nfa => fa\nfb => fb\nfc => fc\nfd => fd\nfe => fe\nff => ff\n\n*** Testing strtoupper() with basic strings ***\nstring(43) \"MARY HAD A LITTLE LAMB AND SHE LOVED IT SO\n\"\n\n*** Testing strtoupper() with various strings ***\n-- Iteration 0 --\nstring(0) \"\"\n\n-- Iteration 1 --\nstring(6) \"STRING\"\n\n-- Iteration 2 --\nstring(10) \"STRING0234\"\n\n-- Iteration 3 --\nstring(20) \"1.233.344STRING12333\"\n\n-- Iteration 4 --\nstring(31) \"$$$$$$!!!!@@@@@@@ ABCDEF !!!***\"\n\n-- Iteration 5 --\nstring(13) \"ABCD\u{0}ABCDABCD\"\n\n-- Iteration 6 --\nstring(1) \"1\"\n\n-- Iteration 7 --\nstring(0) \"\"\n\n-- Iteration 8 --\nstring(20) \"AAAAAAAAAAAAAAAAAAAA\"\n\n-- Iteration 9 --\nstring(20) \"ZZZZZZZZZZZZZZZZZZZZ\"\n\n-- Iteration 10 --\nstring(20) \"````````````````````\"\n\n-- Iteration 11 --\nstring(20) \"{{{{{{{{{{{{{{{{{{{{\"\n\n-- Iteration 12 --\nstring(40) \"{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{\"\n\n-- Iteration 13 --\nstring(31) \"ABCDEFGHIJKLMNOPQRSTUVWXYZ01234\"\n\n-- Iteration 14 --\nstring(32) \"ABCDEFGHIJKLMNOPQRSTUVWXYZ012345\"\n\n-- Iteration 15 --\nstring(33) \"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456\"\n\n-- Iteration 16 --\nstring(62) \"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\"\n\n*** Testing strtoupper() with two different case strings ***\nstrings are same, with Case Insensitive\n*** Done ***");
}

/// php-src `ext/standard/tests/strings/strval_basic.phpt`: Test strval() function : basic functionality
#[test]
fn phpt_strval_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strval() : basic variations ***\n";

error_reporting(E_ALL ^ E_NOTICE);

$simple_heredoc =<<<EOT
Simple HEREDOC string
EOT;


//array of values to iterate over
$values = array(
            // Simple strings
/*1*/		"Hello World",
            'Hello World',

            // String with control chars
/*3*/		"String\nwith\ncontrol\ncharacters\r\n",

            // String with quotes
/*4*/		"String with \"quotes\"",

            //Numeric String
/*5*/		"123456",

            // Hexadecimal string
/*6*/		"0xABC",

            //Heredoc String
/*7*/		$simple_heredoc
);

// loop through each element of the array for strval
$iterator = 1;
foreach($values as $value) {
      echo "\n-- Iteration $iterator --\n";
      var_dump( strval($value) );
      $iterator++;
};
?>
"#);
    assert_eq!(out, "*** Testing strval() : basic variations ***\n\n-- Iteration 1 --\nstring(11) \"Hello World\"\n\n-- Iteration 2 --\nstring(11) \"Hello World\"\n\n-- Iteration 3 --\nstring(32) \"String\nwith\ncontrol\ncharacters\r\n\"\n\n-- Iteration 4 --\nstring(20) \"String with \"quotes\"\"\n\n-- Iteration 5 --\nstring(6) \"123456\"\n\n-- Iteration 6 --\nstring(5) \"0xABC\"\n\n-- Iteration 7 --\nstring(21) \"Simple HEREDOC string\"\n");
}

/// php-src `ext/standard/tests/strings/strval_variation2.phpt`: Test strval() function : usage variations  - Pass all valid char codes
#[test]
fn phpt_strval_variation2() {
    let out = compile_and_run(r#"<?php
echo "*** Testing strval() : usage variations  - Pass all valid char codes ***\n";

$s0 = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f";
$s1 = "\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f";
$s2 = "\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2a\x2b\x2c\x2d\x2e\x2f";
$s3 = "\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3a\x3b\x3c\x3d\x3e\x3f";
$s4 = "\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f";
$s5 = "\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f";
$s6 = "\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f";
$s7 = "\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f";
$s8 = "\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f";
$s9 = "\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f";
$sa = "\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf";
$sb = "\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\xbf";
$sc = "\xc0\xc1\xc2\xc3\xc4\xc5\xc6\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf";
$sd = "\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf";
$se = "\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\xe8\xe9\xea\xeb\xec\xed\xee\xef";
$sf = "\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff";

echo bin2hex(strval($s0));
echo "\n";
echo bin2hex(strval($s1));
echo "\n";
echo bin2hex(strval($s2));
echo "\n";
echo bin2hex(strval($s3));
echo "\n";
echo bin2hex(strval($s4));
echo "\n";
echo bin2hex(strval($s5));
echo "\n";
echo bin2hex(strval($s6));
echo "\n";
echo bin2hex(strval($s7));
echo "\n";
echo bin2hex(strval($s8));
echo "\n";
echo bin2hex(strval($s9));
echo "\n";
echo bin2hex(strval($sa));
echo "\n";
echo bin2hex(strval($sb));
echo "\n";
echo bin2hex(strval($sc));
echo "\n";
echo bin2hex(strval($sd));
echo "\n";
echo bin2hex(strval($se));
echo "\n";
echo bin2hex(strval($sf));
echo "\n";

?>
"#);
    assert_eq!(out, "*** Testing strval() : usage variations  - Pass all valid char codes ***\n000102030405060708090a0b0c0d0e0f\n101112131415161718191a1b1c1d1e1f\n202122232425262728292a2b2c2d2e2f\n303132333435363738393a3b3c3d3e3f\n404142434445464748494a4b4c4d4e4f\n505152535455565758595a5b5c5d5e5f\n606162636465666768696a6b6c6d6e6f\n707172737475767778797a7b7c7d7e7f\n808182838485868788898a8b8c8d8e8f\n909192939495969798999a9b9c9d9e9f\na0a1a2a3a4a5a6a7a8a9aaabacadaeaf\nb0b1b2b3b4b5b6b7b8b9babbbcbdbebf\nc0c1c2c3c4c5c6c7c8c9cacbcccdcecf\nd0d1d2d3d4d5d6d7d8d9dadbdcdddedf\ne0e1e2e3e4e5e6e7e8e9eaebecedeeef\nf0f1f2f3f4f5f6f7f8f9fafbfcfdfeff\n");
}

/// php-src `ext/standard/tests/strings/substr_count_variation_002.phpt`: Test substr_count() function (variation - 2)
#[test]
fn phpt_substr_count_variation_002() {
    let out = compile_and_run(r#"<?php

echo "\n*** Testing possible variations ***\n";
echo "\n-- complex strings containing other than 7-bit chars --\n";
$str = chr(128).chr(129).chr(128).chr(0).chr(255).chr(254).chr(255);
var_dump(substr_count($str, chr(128)));
var_dump(substr_count($str, chr(255)));
var_dump(substr_count($str, chr(0)));

echo "\n-- heredoc string --\n";
$string = <<<EOD
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
acdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
acdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789
EOD;
var_dump(substr_count($string, "abcd"));
var_dump(substr_count($string, "1234"));

echo "\n-- heredoc null string --\n";
$str = <<<EOD
EOD;
var_dump(substr_count($str, "\0"));
var_dump(substr_count($str, "\x000"));
var_dump(substr_count($str, "0"));

echo "Done\n";

?>
"#);
    assert_eq!(out, "\n*** Testing possible variations ***\n\n-- complex strings containing other than 7-bit chars --\nint(2)\nint(2)\nint(1)\n\n-- heredoc string --\nint(14)\nint(16)\n\n-- heredoc null string --\nint(0)\nint(0)\nint(0)\nDone\n");
}

/// php-src `ext/standard/tests/strings/substr_int_min.phpt`: substr() with PHP_INT_MIN offset or length
#[test]
fn phpt_substr_int_min() {
    let out = compile_and_run(r#"<?php
var_dump(substr('x', PHP_INT_MIN));
var_dump(substr('x', 0, PHP_INT_MIN));
?>
"#);
    assert_eq!(out, "string(1) \"x\"\nstring(0) \"\"\n");
}

/// php-src `ext/standard/tests/strings/trim1.phpt`: Test trim() function
#[test]
fn phpt_trim1() {
    let out = compile_and_run(r#"<?php
/* trim with unset/null/boolean variable - returns an empty string */
echo "\n";
$null_var = "";
var_dump( trim($null_var) );
$null_var = 0;
var_dump( trim($null_var) );
$bool_val = true;
var_dump( trim($null_var) );

/* second argument charlist as empty - does not trim any white spaces */
var_dump( trim("\ttesting trim", "") );
var_dump( trim("\ttesting trim  ", true) );

/* Use of class and objects */
echo "\n*** Testing with OBJECTS ***\n";
class string1
{
  public function __toString() {
    return "Object";
  }
}
$obj = new string1;
var_dump( trim($obj, "Ot") );

/* String with embedded NULL */
echo "\n*** Testing with String with embedded NULL ***\n";
var_dump( trim("\x0n1234\x0005678\x0000efgh\xijkl\x0n1", "\x0n1") );

/* heredoc string */
$str = <<<EOD
us
ing heredoc string
EOD;

echo "\n*** Testing with heredoc string ***\n";
var_dump( trim($str, "us\ning") );

echo "\nDone";
?>
"#);
    assert_eq!(out, "\nstring(0) \"\"\nstring(1) \"0\"\nstring(1) \"0\"\nstring(13) \"\ttesting trim\"\nstring(15) \"\ttesting trim  \"\n\n*** Testing with OBJECTS ***\nstring(4) \"bjec\"\n\n*** Testing with String with embedded NULL ***\nstring(22) \"234\u{0}05678\u{0}00efgh\\xijkl\"\n\n*** Testing with heredoc string ***\nstring(12) \" heredoc str\"\n\nDone");
}

/// php-src `ext/standard/tests/strings/ucwords_variation3.phpt`: Test ucwords() function : usage variations - single quoted string
#[test]
fn phpt_ucwords_variation3() {
    let out = compile_and_run(r#"<?php
/*
 * test ucwords() with different string prepared using single quote
*/

echo "*** Testing ucwords() : usage variations ***\n";

// different strings containing regular chars and special chars
$str_array = array(
  // multiple spaces
  'testing    ucwords',
  't e s t i n g   u c w o r d s ',

  // brackets in sentence
  'testing function(ucwords)',
  '(testing ( function (ucwords) )a )test',
  '(t)',
  ' ( t )t',

  // using quote chars in sentence
  '"testing",ucword,"test"',
  '"t""t",test, t',
  '\'t \'t\',test',

  // using other white spaces
  '\ttesting\ttesting\tucwords',
  'testing\rucwords testing ucwords',
  'testing\fucwords \f testing \nucwords',
  '\ntesting\nucwords\n testing \n ucwords',
  'using\vvertical\vtab',

  //using special chars in sentence
  't@@#$% %test ^test &test *test +test -test',
  '!test ~test `test` =test= @test@test.com',
  '/test/r\test\ucwords\t\y\y\u\3 \yy\ /uu/',

  //only special chars
  '!@#$%^&*()_+=-`~'
);

// loop through the $str_array array to test ucwords on each element
$iteration = 1;
for($index = 0; $index < count($str_array); $index++) {
  echo "-- Iteration $iteration --\n";
  var_dump( ucwords($str_array[$index]) );
  $iteration++;
}

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing ucwords() : usage variations ***\n-- Iteration 1 --\nstring(18) \"Testing    Ucwords\"\n-- Iteration 2 --\nstring(30) \"T E S T I N G   U C W O R D S \"\n-- Iteration 3 --\nstring(25) \"Testing Function(ucwords)\"\n-- Iteration 4 --\nstring(38) \"(testing ( Function (ucwords) )a )test\"\n-- Iteration 5 --\nstring(3) \"(t)\"\n-- Iteration 6 --\nstring(7) \" ( T )t\"\n-- Iteration 7 --\nstring(23) \"\"testing\",ucword,\"test\"\"\n-- Iteration 8 --\nstring(14) \"\"t\"\"t\",test, T\"\n-- Iteration 9 --\nstring(11) \"'t 't',test\"\n-- Iteration 10 --\nstring(27) \"\\ttesting\\ttesting\\tucwords\"\n-- Iteration 11 --\nstring(32) \"Testing\\rucwords Testing Ucwords\"\n-- Iteration 12 --\nstring(37) \"Testing\\fucwords \\f Testing \\nucwords\"\n-- Iteration 13 --\nstring(39) \"\\ntesting\\nucwords\\n Testing \\n Ucwords\"\n-- Iteration 14 --\nstring(20) \"Using\\vvertical\\vtab\"\n-- Iteration 15 --\nstring(42) \"T@@#$% %test ^test &test *test +test -test\"\n-- Iteration 16 --\nstring(40) \"!test ~test `test` =test= @test@test.com\"\n-- Iteration 17 --\nstring(40) \"/test/r\\test\\ucwords\\t\\y\\y\\u\\3 \\yy\\ /uu/\"\n-- Iteration 18 --\nstring(16) \"!@#$%^&*()_+=-`~\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic1.phpt`: Test vprintf() function : basic functionality - string format
#[test]
fn phpt_vprintf_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vprintf() : basic functionality - using string format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%s";
$format2 = "%s %s";
$format3 = "%s %s %s";
$arg1 = array("one");
$arg2 = array("one","two");
$arg3 = array("one","two","three");


$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);
$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);
$result = vprintf($format3,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using string format ***\none\nint(3)\none two\nint(7)\none two three\nint(13)\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic2.phpt`: Test vprintf() function : basic functionality - integer format
#[test]
fn phpt_vprintf_basic2() {
    let out = compile_and_run(r#"<?php
/*
 *  Testing vprintf() : basic functionality - using integer format
*/

echo "*** Testing vprintf() : basic functionality - using integer format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%d";
$format2 = "%d %d";
$format3 = "%d %d %d";
$arg1 = array(111);
$arg2 = array(111,222);
$arg3 = array(111,222,333);

$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);

$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);

$result = vprintf($format3,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using integer format ***\n111\nint(3)\n111 222\nint(7)\n111 222 333\nint(11)\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic4.phpt`: Test vprintf() function : basic functionality - bool format
#[test]
fn phpt_vprintf_basic4() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vprintf() : basic functionality - using bool format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%b";
$format2 = "%b %b";
$format3 = "%b %b %b";
$arg1 = array(TRUE);
$arg2 = array(TRUE,FALSE);
$arg3 = array(TRUE,FALSE,TRUE);

$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);

$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);

$result = vprintf($format3,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using bool format ***\n1\nint(1)\n1 0\nint(3)\n1 0 1\nint(5)\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic5.phpt`: Test vprintf() function : basic functionality - char format
#[test]
fn phpt_vprintf_basic5() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vprintf() : basic functionality - using char format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%c";
$format2 = "%c %c";
$format3 = "%c %c %c";
$arg1 = array(65);
$arg2 = array(65,66);
$arg3 = array(65,66,67);

$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);

$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);

$result = vprintf($format3,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using char format ***\nA\nint(1)\nA B\nint(3)\nA B C\nint(5)\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic8.phpt`: Test vprintf() function : basic functionality - octal format
#[test]
fn phpt_vprintf_basic8() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vprintf() : basic functionality - using octal format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%o";
$format2 = "%o %o";
$format3 = "%o %o %o";
$arg1 = array(021);
$arg2 = array(021,0347);
$arg3 = array(021,0347,0567);

$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);

$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);

$result = vprintf($format3,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using octal format ***\n21\nint(2)\n21 347\nint(6)\n21 347 567\nint(10)\n");
}

/// php-src `ext/standard/tests/strings/vprintf_basic9.phpt`: Test vprintf() function : basic functionality - hexadecimal format
#[test]
fn phpt_vprintf_basic9() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vprintf() : basic functionality - using hexadecimal format ***\n";

// Initialising different format strings
$format = "format";
$format1 = "%x";
$format2 = "%x %x";
$format3 = "%x %x %x";

$format11 = "%X";
$format22 = "%X %X";
$format33 = "%X %X %X";

$arg1 = array(11);
$arg2 = array(11,132);
$arg3 = array(11,132,177);

$result = vprintf($format1,$arg1);
echo "\n";
var_dump($result);
$result = vprintf($format11,$arg1);
echo "\n";
var_dump($result);

$result = vprintf($format2,$arg2);
echo "\n";
var_dump($result);
$result = vprintf($format22,$arg2);
echo "\n";
var_dump($result);

$result = vprintf($format3,$arg3);echo "\n";
var_dump($result);
$result = vprintf($format33,$arg3);
echo "\n";
var_dump($result);

?>
"#);
    assert_eq!(out, "*** Testing vprintf() : basic functionality - using hexadecimal format ***\nb\nint(1)\nB\nint(1)\nb 84\nint(4)\nB 84\nint(4)\nb 84 b1\nint(7)\nB 84 B1\nint(7)\n");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic1.phpt`: Test vsprintf() function : basic functionality - string format
#[test]
fn phpt_vsprintf_basic1() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vsprintf() : basic functionality - using string format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%s";
$format2 = "%s %s";
$format3 = "%s %s %s";
$arg1 = array("one");
$arg2 = array("one","two");
$arg3 = array("one","two","three");


var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format3,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using string format ***\nstring(3) \"one\"\nstring(7) \"one two\"\nstring(13) \"one two three\"\nDone");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic2.phpt`: Test vsprintf() function : basic functionality - integer format
#[test]
fn phpt_vsprintf_basic2() {
    let out = compile_and_run(r#"<?php
/*
 *  Testing vsprintf() : basic functionality - using integer format
*/

echo "*** Testing vsprintf() : basic functionality - using integer format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%d";
$format2 = "%d %d";
$format3 = "%d %d %d";
$arg1 = array(111);
$arg2 = array(111,222);
$arg3 = array(111,222,333);

var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format3,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using integer format ***\nstring(3) \"111\"\nstring(7) \"111 222\"\nstring(11) \"111 222 333\"\nDone");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic4.phpt`: Test vsprintf() function : basic functionality - bool format
#[test]
fn phpt_vsprintf_basic4() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vsprintf() : basic functionality - using bool format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%b";
$format2 = "%b %b";
$format3 = "%b %b %b";
$arg1 = array(TRUE);
$arg2 = array(TRUE,FALSE);
$arg3 = array(TRUE,FALSE,TRUE);

var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format3,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using bool format ***\nstring(1) \"1\"\nstring(3) \"1 0\"\nstring(5) \"1 0 1\"\nDone");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic5.phpt`: Test vsprintf() function : basic functionality - char format
#[test]
fn phpt_vsprintf_basic5() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vsprintf() : basic functionality - using char format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%c";
$format2 = "%c %c";
$format3 = "%c %c %c";
$arg1 = array(65);
$arg2 = array(65,66);
$arg3 = array(65,66,67);

var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format3,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using char format ***\nstring(1) \"A\"\nstring(3) \"A B\"\nstring(5) \"A B C\"\nDone");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic8.phpt`: Test vsprintf() function : basic functionality - octal format
#[test]
fn phpt_vsprintf_basic8() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vsprintf() : basic functionality - using octal format ***\n";

// Initialise all required variables
$format = "format";
$format1 = "%o";
$format2 = "%o %o";
$format3 = "%o %o %o";
$arg1 = array(021);
$arg2 = array(021,0347);
$arg3 = array(021,0347,0567);

var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format3,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using octal format ***\nstring(2) \"21\"\nstring(6) \"21 347\"\nstring(10) \"21 347 567\"\nDone");
}

/// php-src `ext/standard/tests/strings/vsprintf_basic9.phpt`: Test vsprintf() function : basic functionality - hexadecimal format
#[test]
fn phpt_vsprintf_basic9() {
    let out = compile_and_run(r#"<?php
echo "*** Testing vsprintf() : basic functionality - using hexadecimal format ***\n";

// Initialising different format strings
$format = "format";
$format1 = "%x";
$format2 = "%x %x";
$format3 = "%x %x %x";

$format11 = "%X";
$format22 = "%X %X";
$format33 = "%X %X %X";

$arg1 = array(11);
$arg2 = array(11,132);
$arg3 = array(11,132,177);

var_dump( vsprintf($format1,$arg1) );
var_dump( vsprintf($format11,$arg1) );

var_dump( vsprintf($format2,$arg2) );
var_dump( vsprintf($format22,$arg2) );

var_dump( vsprintf($format3,$arg3) );
var_dump( vsprintf($format33,$arg3) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing vsprintf() : basic functionality - using hexadecimal format ***\nstring(1) \"b\"\nstring(1) \"B\"\nstring(4) \"b 84\"\nstring(4) \"B 84\"\nstring(7) \"b 84 b1\"\nstring(7) \"B 84 B1\"\nDone");
}

/// php-src `ext/standard/tests/strings/wordwrap_basic.phpt`: Test wordwrap() function : basic functionality
#[test]
fn phpt_wordwrap_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing wordwrap() : basic functionality ***\n";

// Initialize all required variables
$str = "The quick brown foooooooooox jummmmmmmmmmmmped over the lazzzzzzzzzzzy doooooooooooooooooooooog.";
$width = 80;
$break = '<br />\n';

// Calling wordwrap() with default arguments
var_dump( wordwrap($str) );

// Calling wordwrap() with all possible optional arguments
// with $width arg
var_dump( wordwrap($str, $width) );
// with $break arg
var_dump( wordwrap($str, $width, $break) );

// Calling wordwrap() with all arguments
// $cut as true
$width = 10;
$cut = true;
var_dump( wordwrap($str, $width, $break, $cut) );

// $cut as false
$width = 10;
$cut = false;
var_dump( wordwrap($str, $width, $break, $cut) );
echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing wordwrap() : basic functionality ***\nstring(96) \"The quick brown foooooooooox jummmmmmmmmmmmped over the lazzzzzzzzzzzy\ndoooooooooooooooooooooog.\"\nstring(96) \"The quick brown foooooooooox jummmmmmmmmmmmped over the lazzzzzzzzzzzy\ndoooooooooooooooooooooog.\"\nstring(103) \"The quick brown foooooooooox jummmmmmmmmmmmped over the lazzzzzzzzzzzy<br />\\ndoooooooooooooooooooooog.\"\nstring(178) \"The quick<br />\\nbrown<br />\\nfooooooooo<br />\\nox<br />\\njummmmmmmm<br />\\nmmmmped<br />\\nover the<br />\\nlazzzzzzzz<br />\\nzzzy<br />\\ndooooooooo<br />\\noooooooooo<br />\\nooog.\"\nstring(138) \"The quick<br />\\nbrown<br />\\nfoooooooooox<br />\\njummmmmmmmmmmmped<br />\\nover the<br />\\nlazzzzzzzzzzzy<br />\\ndoooooooooooooooooooooog.\"\nDone\n");
}

/// php-src `ext/standard/tests/strings/wordwrap_variation5.phpt`: Test wordwrap() function : usage variations  - valid break arguments(spaces)
#[test]
fn phpt_wordwrap_variation5() {
    let out = compile_and_run(r#"<?php
/*
 *test wordwrap() with break arguments as single spaces
*/

echo "*** Testing wordwrap() : usage variations ***\n";

// Initialize all required variables
$str = "Testing wordrap function";
$width = 1;
$cut = false;

echo "\n-- Testing wordwrap() with default break value and single space as value --\n";
echo "-- with default break and cut value --\n";
var_dump( wordwrap($str, $width) );  // default break and cut value

echo "-- with default cut value --\n";
$break = ' ';
$break1 = "  ";
var_dump( wordwrap($str, $width, $break) );
var_dump( wordwrap($str, $width, $break1) );

echo "-- with cut value as false --\n";
$cut = false;
var_dump( wordwrap($str, $width, $break, $cut) );
var_dump( wordwrap($str, $width, $break1, $cut) );

echo "-- with cut value as true --\n";
$cut = true;
var_dump( wordwrap($str, $width, $break, $cut) );
var_dump( wordwrap($str, $width, $break1, $cut) );

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing wordwrap() : usage variations ***\n\n-- Testing wordwrap() with default break value and single space as value --\n-- with default break and cut value --\nstring(24) \"Testing\nwordrap\nfunction\"\n-- with default cut value --\nstring(24) \"Testing wordrap function\"\nstring(26) \"Testing  wordrap  function\"\n-- with cut value as false --\nstring(24) \"Testing wordrap function\"\nstring(26) \"Testing  wordrap  function\"\n-- with cut value as true --\nstring(43) \"T e s t i n g w o r d r a p f u n c t i o n\"\nstring(64) \"T  e  s  t  i  n  g  w  o  r  d  r  a  p  f  u  n  c  t  i  o  n\"\nDone\n");
}

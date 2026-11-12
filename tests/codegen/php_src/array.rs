//! Purpose:
//! php-src `.phpt` conformance tests for array, migrated to Rust: each test compiles the
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

/// php-src `ext/standard/tests/array/array_change_key_case_variation3.phpt`: Test array_change_key_case() function : usage variations - different data types as keys
#[test]
fn phpt_array_change_key_case_variation3() {
    let out = compile_and_run(r#"<?php
/*
 * Pass arrays with different data types as keys to array_change_key_case()
 * to test conversion
 */

echo "*** Testing array_change_key_case() : usage variations ***\n";

// arrays of different data types to be passed to $input argument
$inputs = [
	'int' => [
       0 => 'zero',
       1 => 'one',
       12345 => 'positive',
       -2345 => 'negative',
    ],
    'bool' => [
       true => 'true',
       false => 'false',
    ],
    'string' => [
       'strings' => 'strings',
       '' => 'emptys',
    ],
];

// loop through each sub-array of $inputs to check the behavior of array_change_key_case()
foreach($inputs as $key => $input) {
  echo "\n-- $key data --\n";
  var_dump( array_change_key_case($input, CASE_UPPER) );
};

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_change_key_case() : usage variations ***\n\n-- int data --\narray(4) {\n  [0]=>\n  string(4) \"zero\"\n  [1]=>\n  string(3) \"one\"\n  [12345]=>\n  string(8) \"positive\"\n  [-2345]=>\n  string(8) \"negative\"\n}\n\n-- bool data --\narray(2) {\n  [1]=>\n  string(4) \"true\"\n  [0]=>\n  string(5) \"false\"\n}\n\n-- string data --\narray(2) {\n  [\"STRINGS\"]=>\n  string(7) \"strings\"\n  [\"\"]=>\n  string(6) \"emptys\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_change_key_case_variation7.phpt`: Test array_change_key_case() function : usage variations - referenced variables
#[test]
fn phpt_array_change_key_case_variation7() {
    let out = compile_and_run(r#"<?php
/*
 * Test array_change_key_case() when:
 * 1. Passed a referenced variable
 * 2. Passed an argument by reference
 */

echo "*** Testing array_change_key_case() : usage variations ***\n";

$input = array('one' => 1, 'two' => 2, 'ABC' => 'xyz');

echo "\n-- \$input argument is a reference to array --\n";
$new_input = &$input;
echo "Result:\n";
var_dump(array_change_key_case($new_input, CASE_UPPER));
echo "Original:\n";
var_dump($input);
echo "Referenced:\n";
var_dump($new_input);

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_change_key_case() : usage variations ***\n\n-- $input argument is a reference to array --\nResult:\narray(3) {\n  [\"ONE\"]=>\n  int(1)\n  [\"TWO\"]=>\n  int(2)\n  [\"ABC\"]=>\n  string(3) \"xyz\"\n}\nOriginal:\narray(3) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n  [\"ABC\"]=>\n  string(3) \"xyz\"\n}\nReferenced:\narray(3) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n  [\"ABC\"]=>\n  string(3) \"xyz\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_combine_variation3.phpt`: Test array_combine() function : usage variations - different arrays(Bug#43424)
#[test]
fn phpt_array_combine_variation3() {
    let out = compile_and_run(r#"<?php
/*
* Passing different types of arrays to both $keys and $values arguments and testing whether
* array_combine() behaves in an expected way with the arguments passed to the function
*/

echo "*** Testing array_combine() : Passing different types of arrays to both \$keys and \$values argument ***\n";
/* Different heredoc strings passed as argument to arrays */
// heredoc with blank line
$blank_line = <<<EOT


EOT;

// heredoc with multiline string
$multiline_string = <<<EOT
hello world
The quick brown fox jumped over;
the lazy dog
This is a double quoted string
EOT;

// heredoc with quoted strings and numeric values
$numeric_string = <<<EOT
11 < 12. 123 >22
'single quoted string'
"double quoted string"
2222 != 1111.\t 0000 = 0000\n
EOT;

// arrays passed to $keys argument
$arrays = array (
/*1*/  array(1, 2), // with default keys and numeric values
       array(1.1, 2.2), // with default keys & float values
       array(false,true), // with default keys and boolean values
       array(), // empty array
/*5*/  array(NULL), // with NULL
       array("a\v\f","aaaa\n","b","b\tbbb","c","\[\]\!\@\#\$\%\^\&\*\(\)\{\}"),  // with double quoted strings
       array('a\v\f','aaaa\n','b','b\tbbb','c','\[\]\!\@\#\$\%\^\&\*\(\)\{\}'),  // with single quoted strings
       array("h1" => $blank_line, "h2" => $multiline_string, $numeric_string),  // with heredocs

       // associative arrays
/*9*/  array(1 => "one", 2 => "two", 3 => "three"),  // explicit numeric keys, string values
       array("one" => 1, "two" => 2, "three" => 3 ),  // string keys & numeric values
       array( 1 => 10, 2 => 20, 4 => 40, 3 => 30),  // explicit numeric keys and numeric values
       array( "one" => "ten", "two" => "twenty", "three" => "thirty"),  // string key/value
       array("one" => 1, 2 => "two", 4 => "four"),  //mixed

       // associative array, containing null/empty/boolean values as key/value
/*14*/
       array(true => "true", false => "false", "false" => false, "true" => true),
       array("" => "emptyd", '' => 'emptys', "emptyd" => "", 'emptys' => ''),
       array(1 => '', 2 => "", 5 => false, 6 => true),
       array('' => 1, "" => 2, false => 5, true => 6),

       // array with repetitive keys
/*19*/ array("One" => 1, "two" => 2, "One" => 10, "two" => 20, "three" => 3)
);

// loop through each sub-array within $arrays to check the behavior of array_combine()
// same arrays are passed to both $keys and $values
$iterator = 1;
foreach($arrays as $array) {
  echo "-- Iteration $iterator --\n";
  var_dump( array_combine($array, $array) );
  $iterator++;
}

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_combine() : Passing different types of arrays to both $keys and $values argument ***\n-- Iteration 1 --\narray(2) {\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n}\n-- Iteration 2 --\narray(2) {\n  [\"1.1\"]=>\n  float(1.1)\n  [\"2.2\"]=>\n  float(2.2)\n}\n-- Iteration 3 --\narray(2) {\n  [\"\"]=>\n  bool(false)\n  [1]=>\n  bool(true)\n}\n-- Iteration 4 --\narray(0) {\n}\n-- Iteration 5 --\narray(1) {\n  [\"\"]=>\n  NULL\n}\n-- Iteration 6 --\narray(6) {\n  [\"a\u{b}\u{c}\"]=>\n  string(3) \"a\u{b}\u{c}\"\n  [\"aaaa\n\"]=>\n  string(5) \"aaaa\n\"\n  [\"b\"]=>\n  string(1) \"b\"\n  [\"b\tbbb\"]=>\n  string(5) \"b\tbbb\"\n  [\"c\"]=>\n  string(1) \"c\"\n  [\"\\[\\]\\!\\@\\#$\\%\\^\\&\\*\\(\\)\\{\\}\"]=>\n  string(27) \"\\[\\]\\!\\@\\#$\\%\\^\\&\\*\\(\\)\\{\\}\"\n}\n-- Iteration 7 --\narray(6) {\n  [\"a\\v\\f\"]=>\n  string(5) \"a\\v\\f\"\n  [\"aaaa\\n\"]=>\n  string(6) \"aaaa\\n\"\n  [\"b\"]=>\n  string(1) \"b\"\n  [\"b\\tbbb\"]=>\n  string(6) \"b\\tbbb\"\n  [\"c\"]=>\n  string(1) \"c\"\n  [\"\\[\\]\\!\\@\\#\\$\\%\\^\\&\\*\\(\\)\\{\\}\"]=>\n  string(28) \"\\[\\]\\!\\@\\#\\$\\%\\^\\&\\*\\(\\)\\{\\}\"\n}\n-- Iteration 8 --\narray(3) {\n  [\"\n\"]=>\n  string(1) \"\n\"\n  [\"hello world\nThe quick brown fox jumped over;\nthe lazy dog\nThis is a double quoted string\"]=>\n  string(88) \"hello world\nThe quick brown fox jumped over;\nthe lazy dog\nThis is a double quoted string\"\n  [\"11 < 12. 123 >22\n'single quoted string'\n\"double quoted string\"\n2222 != 1111.\t 0000 = 0000\n\"]=>\n  string(90) \"11 < 12. 123 >22\n'single quoted string'\n\"double quoted string\"\n2222 != 1111.\t 0000 = 0000\n\"\n}\n-- Iteration 9 --\narray(3) {\n  [\"one\"]=>\n  string(3) \"one\"\n  [\"two\"]=>\n  string(3) \"two\"\n  [\"three\"]=>\n  string(5) \"three\"\n}\n-- Iteration 10 --\narray(3) {\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n  [3]=>\n  int(3)\n}\n-- Iteration 11 --\narray(4) {\n  [10]=>\n  int(10)\n  [20]=>\n  int(20)\n  [40]=>\n  int(40)\n  [30]=>\n  int(30)\n}\n-- Iteration 12 --\narray(3) {\n  [\"ten\"]=>\n  string(3) \"ten\"\n  [\"twenty\"]=>\n  string(6) \"twenty\"\n  [\"thirty\"]=>\n  string(6) \"thirty\"\n}\n-- Iteration 13 --\narray(3) {\n  [1]=>\n  int(1)\n  [\"two\"]=>\n  string(3) \"two\"\n  [\"four\"]=>\n  string(4) \"four\"\n}\n-- Iteration 14 --\narray(4) {\n  [\"true\"]=>\n  string(4) \"true\"\n  [\"false\"]=>\n  string(5) \"false\"\n  [\"\"]=>\n  bool(false)\n  [1]=>\n  bool(true)\n}\n-- Iteration 15 --\narray(2) {\n  [\"emptys\"]=>\n  string(6) \"emptys\"\n  [\"\"]=>\n  string(0) \"\"\n}\n-- Iteration 16 --\narray(2) {\n  [\"\"]=>\n  bool(false)\n  [1]=>\n  bool(true)\n}\n-- Iteration 17 --\narray(3) {\n  [2]=>\n  int(2)\n  [5]=>\n  int(5)\n  [6]=>\n  int(6)\n}\n-- Iteration 18 --\narray(3) {\n  [10]=>\n  int(10)\n  [20]=>\n  int(20)\n  [3]=>\n  int(3)\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_diff_assoc_variation8.phpt`: Test array_diff_assoc() function : usage variations - array containing duplicate keys and values
#[test]
fn phpt_array_diff_assoc_variation8() {
    let out = compile_and_run(r#"<?php
/*
 * Test how array_diff_assoc() behaves when comparing:
 * 1. the order of the array
 * 2. duplicate values
 * 3. duplicate key names
 */

echo "*** Testing array_diff_assoc() : variation ***\n";

$array_index = array('a', 'b', 'c', 0 => 'd', 'b');   //duplicate key (0), duplicate value (b)
$array_assoc = array ('2' => 'c',   //same key=>value pair, different order
                      '1' => 'b',
                      '0' => 'a',
                      'b' => '3',   //key and value from array_index swapped
                      'c' => 2);    //same as above, using integer

var_dump(array_diff_assoc($array_index, $array_assoc));
var_dump(array_diff_assoc($array_assoc, $array_index));

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_diff_assoc() : variation ***\narray(2) {\n  [0]=>\n  string(1) \"d\"\n  [3]=>\n  string(1) \"b\"\n}\narray(3) {\n  [0]=>\n  string(1) \"a\"\n  [\"b\"]=>\n  string(1) \"3\"\n  [\"c\"]=>\n  int(2)\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_diff_key_basic.phpt`: Test array_diff_key() : basic functionality
#[test]
fn phpt_array_diff_key_basic() {
    let out = compile_and_run(r#"<?php
/*
* Function is implemented in ext/standard/array.c
*/
$array1 = array('blue' => 1, 'red' => 2, 'green' => 3, 'purple' => 4);
$array2 = array('green' => 5, 'blue' => 6, 'yellow' => 7, 'cyan' => 8);
var_dump(array_diff_key($array1, $array2));
?>
"#);
    assert_eq!(out, "array(2) {\n  [\"red\"]=>\n  int(2)\n  [\"purple\"]=>\n  int(4)\n}\n");
}

/// php-src `ext/standard/tests/array/array_diff_key_variation6.phpt`: Test array_diff_key() function : usage variation - Passing boolean indexed array
#[test]
fn phpt_array_diff_key_variation6() {
    let out = compile_and_run(r#"<?php
echo "*** Testing array_diff_key() : usage variation ***\n";

// Initialise function arguments not being substituted (if any)
$input_array = array(0 => '0', 1 => '1', -10 => '-10', 'true' => 1, 'false' => 0);
$boolean_indx_array = array(true => 'boolt', false => 'boolf', TRUE => 'boolT', FALSE => 'boolF');

echo "\n-- Testing array_diff_key() function with boolean indexed array --\n";
// loop through each element of the array for arr1
var_dump( array_diff_key($input_array, $boolean_indx_array) );
var_dump( array_diff_key($boolean_indx_array, $input_array) );
?>
"#);
    assert_eq!(out, "*** Testing array_diff_key() : usage variation ***\n\n-- Testing array_diff_key() function with boolean indexed array --\narray(3) {\n  [-10]=>\n  string(3) \"-10\"\n  [\"true\"]=>\n  int(1)\n  [\"false\"]=>\n  int(0)\n}\narray(0) {\n}\n");
}

/// php-src `ext/standard/tests/array/array_fill_basic.phpt`: Test array_fill() function : basic functionality
#[test]
fn phpt_array_fill_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing array_fill() : basic functionality ***\n";

// calling the array_fill with all possible valid values for 'val' argument
$start_key = 0 ;
$num = 2;
$heredoc = <<<HERE_DOC
Hello
HERE_DOC;

// array of possible valid values for 'val' argument
$values = array (

  /* 1  */  NULL,
            0,
            1,
  /* 4  */  1.5,
            'hi',
            "hi",
  /* 7  */  $heredoc

);

$counter = 1;
for($i = 0; $i < count($values); $i ++)
{
  echo "-- Iteration $counter --\n";
  $val = $values[$i];

  var_dump( array_fill($start_key,$num,$val) );

  $counter++;
}

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_fill() : basic functionality ***\n-- Iteration 1 --\narray(2) {\n  [0]=>\n  NULL\n  [1]=>\n  NULL\n}\n-- Iteration 2 --\narray(2) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(0)\n}\n-- Iteration 3 --\narray(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(1)\n}\n-- Iteration 4 --\narray(2) {\n  [0]=>\n  float(1.5)\n  [1]=>\n  float(1.5)\n}\n-- Iteration 5 --\narray(2) {\n  [0]=>\n  string(2) \"hi\"\n  [1]=>\n  string(2) \"hi\"\n}\n-- Iteration 6 --\narray(2) {\n  [0]=>\n  string(2) \"hi\"\n  [1]=>\n  string(2) \"hi\"\n}\n-- Iteration 7 --\narray(2) {\n  [0]=>\n  string(5) \"Hello\"\n  [1]=>\n  string(5) \"Hello\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_filter_variation8.phpt`: Test array_filter() function : usage variations - Callback function with different return values
#[test]
fn phpt_array_filter_variation8() {
    let out = compile_and_run(r#"<?php
/*
* callback functions are expected to return bool value for array_filter()
* here testing callback functions for return values other than bool
*/

echo "*** Testing array_filter() : usage variations - callback function with different return values***\n";

$input = array(0, 1, -1, 10, 100, 1000, 'Hello', null, true);

function callback1($input)
{
  return 5;
}
echo "callback function with int return value\n";
var_dump( array_filter($input, 'callback1') );

// float as return value
function callback2($input)
{
  return 3.4;
}
echo "callback function with float return value\n";
var_dump( array_filter($input, 'callback2') );

// string as return value
function callback3($input)
{
  return 'value';
}
echo "callback function with string return value\n";
var_dump( array_filter($input, 'callback3') );

// null as return value
function callback4($input)
{
  return null;
}
echo "callback function with null return value\n";
var_dump( array_filter($input, 'callback4') );

// array as return value
function callback5($input)
{
  return array(8);
}
echo "callback function with array as return value\n";
var_dump( array_filter($input, 'callback5') );

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing array_filter() : usage variations - callback function with different return values***\ncallback function with int return value\narray(9) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(-1)\n  [3]=>\n  int(10)\n  [4]=>\n  int(100)\n  [5]=>\n  int(1000)\n  [6]=>\n  string(5) \"Hello\"\n  [7]=>\n  NULL\n  [8]=>\n  bool(true)\n}\ncallback function with float return value\narray(9) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(-1)\n  [3]=>\n  int(10)\n  [4]=>\n  int(100)\n  [5]=>\n  int(1000)\n  [6]=>\n  string(5) \"Hello\"\n  [7]=>\n  NULL\n  [8]=>\n  bool(true)\n}\ncallback function with string return value\narray(9) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(-1)\n  [3]=>\n  int(10)\n  [4]=>\n  int(100)\n  [5]=>\n  int(1000)\n  [6]=>\n  string(5) \"Hello\"\n  [7]=>\n  NULL\n  [8]=>\n  bool(true)\n}\ncallback function with null return value\narray(0) {\n}\ncallback function with array as return value\narray(9) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(-1)\n  [3]=>\n  int(10)\n  [4]=>\n  int(100)\n  [5]=>\n  int(1000)\n  [6]=>\n  string(5) \"Hello\"\n  [7]=>\n  NULL\n  [8]=>\n  bool(true)\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_flip_basic.phpt`: Test array_flip() function : basic functionality
#[test]
fn phpt_array_flip_basic() {
    let out = compile_and_run(r#"<?php
echo "*** Testing array_flip() : basic functionality ***\n";

// array with default keys - numeric values
$input = array(1, 2);
var_dump( array_flip($input) );

// array with default keys - string values
$input = array('value1', "value2");
var_dump( array_flip($input) );

// associative arrays - key as string
$input = array('key1' => 1, "key2" => 2);
var_dump( array_flip($input) );

// associative arrays - key as numeric
$input = array(1 => 'one', 2 => "two");
var_dump( array_flip($input) );

// combination of associative and non-associative array
$input = array(1 => 'one','two', 3 => 'three', 4, "five" => 5);
var_dump( array_flip($input) );
echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing array_flip() : basic functionality ***\narray(2) {\n  [1]=>\n  int(0)\n  [2]=>\n  int(1)\n}\narray(2) {\n  [\"value1\"]=>\n  int(0)\n  [\"value2\"]=>\n  int(1)\n}\narray(2) {\n  [1]=>\n  string(4) \"key1\"\n  [2]=>\n  string(4) \"key2\"\n}\narray(2) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n}\narray(5) {\n  [\"one\"]=>\n  int(1)\n  [\"two\"]=>\n  int(2)\n  [\"three\"]=>\n  int(3)\n  [4]=>\n  int(4)\n  [5]=>\n  string(4) \"five\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_intersect_key_basic.phpt`: array_intersect_key(): Basic Test
#[test]
fn phpt_array_intersect_key_basic() {
    let out = compile_and_run(r#"<?php
/*
* Function is implemented in ext/standard/array.c
*/
$array1 = array('blue' => 1, 'red' => 2, 'green' => 3, 'purple' => 4);
$array2 = array('green' => 5, 'blue' => 6, 'yellow' => 7, 'cyan' => 8);
var_dump(array_intersect_key($array1, $array2));
?>
"#);
    assert_eq!(out, "array(2) {\n  [\"blue\"]=>\n  int(1)\n  [\"green\"]=>\n  int(3)\n}\n");
}

/// php-src `ext/standard/tests/array/array_intersect_key_variation6.phpt`: Test array_intersect_key() function : usage variation - Passing boolean indexed array
#[test]
fn phpt_array_intersect_key_variation6() {
    let out = compile_and_run(r#"<?php
echo "*** Testing array_intersect_key() : usage variation ***\n";

// Initialise function arguments not being substituted (if any)
$input_array = array(0 => '0', 1 => '1' , -10 => '-10');
$boolean_indx_array = array(true => 'boolt', false => 'boolf', TRUE => 'boolT', FALSE => 'boolF');

echo "\n-- Testing array_intersect_key() function with boolean indexed array --\n";
var_dump( array_intersect_key($input_array, $boolean_indx_array) );
var_dump( array_intersect_key($boolean_indx_array,$input_array ) );
?>
"#);
    assert_eq!(out, "*** Testing array_intersect_key() : usage variation ***\n\n-- Testing array_intersect_key() function with boolean indexed array --\narray(2) {\n  [0]=>\n  string(1) \"0\"\n  [1]=>\n  string(1) \"1\"\n}\narray(2) {\n  [1]=>\n  string(5) \"boolT\"\n  [0]=>\n  string(5) \"boolF\"\n}\n");
}

/// php-src `ext/standard/tests/array/array_key_exists_basic.phpt`: Test array_key_exists() function : basic functionality
#[test]
fn phpt_array_key_exists_basic() {
    let out = compile_and_run(r#"<?php
/*
 * Test basic functionality of array_key_exists()
 */

echo "*** Testing array_key_exists() : basic functionality ***\n";

$key1 = 'key';
$key2 = 'val';
$search = array('one', 'key' => 'value', 'val');
var_dump(array_key_exists($key1, $search));
var_dump(array_key_exists($key2, $search));

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_key_exists() : basic functionality ***\nbool(true)\nbool(false)\nDone");
}

/// php-src `ext/standard/tests/array/array_key_exists_variation4.phpt`: Test array_key_exists() function : usage variations - referenced variables
#[test]
fn phpt_array_key_exists_variation4() {
    let out = compile_and_run(r#"<?php
/*
 * Pass referenced variables as arguments to array_key_exists() to test behaviour
 */

echo "*** Testing array_key_exists() : usage variations ***\n";

$array = array('one' => 1, 'two' => 2, 'three' => 3);

echo "\n-- \$search is a reference to \$array --\n";
$search = &$array;
var_dump(array_key_exists('one', $search));

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_key_exists() : usage variations ***\n\n-- $search is a reference to $array --\nbool(true)\nDone");
}

/// php-src `ext/standard/tests/array/array_key_exists_variation5.phpt`: Test array_key_exists() function : usage variations - multidimensional arrays
#[test]
fn phpt_array_key_exists_variation5() {
    let out = compile_and_run(r#"<?php
/*
 * Test how array_key_exists() behaves with multi-dimensional arrays
 */

echo "*** Testing array_key_exists() : usage variations ***\n";

$multi_array = array ('zero' => 'val1',
                      'one' => 'val2',
                      'sub1' => array (1, 2, 3));

echo "\n-- Attempt to match key in sub-array --\n";
// this key is in the sub-array
var_dump(array_key_exists(0, $multi_array));

echo "\n-- \$search arg points to sub-array --\n";
var_dump(array_key_exists(0, $multi_array['sub1']));

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_key_exists() : usage variations ***\n\n-- Attempt to match key in sub-array --\nbool(false)\n\n-- $search arg points to sub-array --\nbool(true)\nDone");
}

/// php-src `ext/standard/tests/array/array_key_exists_variation8.phpt`: Test array_key_exists() function : usage variations - array keys are different data types
#[test]
fn phpt_array_key_exists_variation8() {
    let out = compile_and_run(r#"<?php
/*
 * Pass an array where the keys are different data types as the $search argument
 * then pass many different data types as $key argument to test where array_key_exist()
 * returns true.
 */

echo "*** Testing array_key_exists() : usage variations ***\n";

// heredoc string
$heredoc = <<<EOT
string
EOT;

// different data types to be iterated over
$inputs = array(

       // int data
/*1*/  'int' => array(
       0 => 'zero',
       1 => 'one',
       12345 => 'positive',
       -2345 => 'negative',
       ),

       // boolean data
/*4*/ 'bool lowercase' => array(
       true => 'lowert',
       false => 'lowerf',
       ),

       // empty data
/*5*/ 'empty single quotes' => array(
       '' => 'emptys',
       ),

       // string data
/*6*/ 'string' => array(
       "stringd" => 'stringd',
       'strings' => 'strings',
       $heredoc => 'stringh',
       ),
);

// loop through each element of $inputs to check the behavior of array_key_exists()
$iterator = 1;
foreach($inputs as $type => $input) {
    echo "\n-- $type data --\n";

    //iterate over again to get all different key values
    foreach ($inputs as $new_type => $new_input) {
        echo "-- \$key arguments are $new_type data:\n";
        foreach ($new_input as $key => $search) {
            var_dump(array_key_exists($key, $input));
        }
    }
    $iterator++;
};

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_key_exists() : usage variations ***\n\n-- int data --\n-- $key arguments are int data:\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n-- $key arguments are bool lowercase data:\nbool(true)\nbool(true)\n-- $key arguments are empty single quotes data:\nbool(false)\n-- $key arguments are string data:\nbool(false)\nbool(false)\nbool(false)\n\n-- bool lowercase data --\n-- $key arguments are int data:\nbool(true)\nbool(true)\nbool(false)\nbool(false)\n-- $key arguments are bool lowercase data:\nbool(true)\nbool(true)\n-- $key arguments are empty single quotes data:\nbool(false)\n-- $key arguments are string data:\nbool(false)\nbool(false)\nbool(false)\n\n-- empty single quotes data --\n-- $key arguments are int data:\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n-- $key arguments are bool lowercase data:\nbool(false)\nbool(false)\n-- $key arguments are empty single quotes data:\nbool(true)\n-- $key arguments are string data:\nbool(false)\nbool(false)\nbool(false)\n\n-- string data --\n-- $key arguments are int data:\nbool(false)\nbool(false)\nbool(false)\nbool(false)\n-- $key arguments are bool lowercase data:\nbool(false)\nbool(false)\n-- $key arguments are empty single quotes data:\nbool(false)\n-- $key arguments are string data:\nbool(true)\nbool(true)\nbool(true)\nDone");
}

/// php-src `ext/standard/tests/array/array_keys_basic.phpt`: Test array_keys() function (basic)
#[test]
fn phpt_array_keys_basic() {
    let out = compile_and_run(r#"<?php

echo "*** Testing array_keys() on basic array operation ***\n";
$basic_arr = array("a" => 1, "b" => 2, 2 => 2.0, -23 => "asdasd",
                   array(1,2,3));
var_dump(array_keys($basic_arr));

echo "Done\n";
?>
"#);
    assert_eq!(out, "*** Testing array_keys() on basic array operation ***\narray(5) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  int(2)\n  [3]=>\n  int(-23)\n  [4]=>\n  int(3)\n}\nDone\n");
}

/// php-src `ext/standard/tests/array/array_keys_variation_002.phpt`: Test array_keys() function (variation - 2)
#[test]
fn phpt_array_keys_variation_002() {
    let out = compile_and_run(r#"<?php

echo "\n*** Testing array_keys() on an array created on the fly ***\n";
var_dump(array_keys(array("a" => 1, "b" => 2, "c" => 3)));
var_dump(array_keys(array()));  // null array

echo "Done\n";
?>
"#);
    assert_eq!(out, "\n*** Testing array_keys() on an array created on the fly ***\narray(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"c\"\n}\narray(0) {\n}\nDone\n");
}

/// php-src `ext/standard/tests/array/array_product_empty_array.phpt`: Test array_product() function with empty array
#[test]
fn phpt_array_product_empty_array() {
    let out = compile_and_run(r#"<?php
$input = [];

echo "array_product() version:\n";
var_dump(array_product($input));

echo "array_reduce() version:\n";
var_dump(array_reduce($input, fn($carry, $value) => $carry * $value, 1));
?>
"#);
    assert_eq!(out, "array_product() version:\nint(1)\narray_reduce() version:\nint(1)\n");
}

/// php-src `ext/standard/tests/array/array_reduce_return_by_ref.phpt`: Return by reference from array_reduce() callback
#[test]
fn phpt_array_reduce_return_by_ref() {
    let out = compile_and_run(r#"<?php
$array = [1, 2];
var_dump(array_reduce($array, function &($a, $b) {
    return $b;
}, 0));
?>
"#);
    assert_eq!(out, "int(2)\n");
}

/// php-src `ext/standard/tests/array/array_shift_variation3.phpt`: Test array_shift() function : usage variations - Pass array with different data types as keys
#[test]
fn phpt_array_shift_variation3() {
    let out = compile_and_run(r#"<?php
/*
 * Pass arrays with different data types as keys to test how array_shift() re-assigns keys
 */

echo "*** Testing array_shift() : usage variations ***\n";

// heredoc string
$heredoc = <<<EOT
hello world
EOT;

// unexpected values to be passed to $stack argument
$inputs = array(

       // int data
/*1*/  'int' => array(
       0 => 'zero',
       1 => 'one',
       12345 => 'positive',
       -2345 => 'negative',
       ),

       // boolean data
/*6*/  'bool lowercase' => array(
       true => 'lowert',
       false => 'lowerf',
       ),

       // empty data

/*9*/  'empty single quotes' => array(
       '' => 'emptys',
       ),

       // string data
/*10*/ 'string' => array(
       "stringd" => 'stringd',
       'strings' => 'strings',
       $heredoc => 'stringh',
       ),
);

// loop through each element of $inputs to check the behavior of array_shift()
$iterator = 1;
foreach($inputs as $key => $input) {
  echo "\n-- Iteration $iterator : $key data --\n";
  var_dump( array_shift($input) );
  var_dump($input);
  $iterator++;
};

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_shift() : usage variations ***\n\n-- Iteration 1 : int data --\nstring(4) \"zero\"\narray(3) {\n  [0]=>\n  string(3) \"one\"\n  [1]=>\n  string(8) \"positive\"\n  [2]=>\n  string(8) \"negative\"\n}\n\n-- Iteration 2 : bool lowercase data --\nstring(6) \"lowert\"\narray(1) {\n  [0]=>\n  string(6) \"lowerf\"\n}\n\n-- Iteration 3 : empty single quotes data --\nstring(6) \"emptys\"\narray(0) {\n}\n\n-- Iteration 4 : string data --\nstring(7) \"stringd\"\narray(2) {\n  [\"strings\"]=>\n  string(7) \"strings\"\n  [\"hello world\"]=>\n  string(7) \"stringh\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_sum_empty_array.phpt`: Test array_sum() function with empty array
#[test]
fn phpt_array_sum_empty_array() {
    let out = compile_and_run(r#"<?php
$input = [];

echo "array_sum() version:\n";
var_dump(array_sum($input));

echo "array_reduce() version:\n";
var_dump(array_reduce($input, fn($carry, $value) => $carry + $value, 0));
?>
"#);
    assert_eq!(out, "array_sum() version:\nint(0)\narray_reduce() version:\nint(0)\n");
}

/// php-src `ext/standard/tests/array/array_sum_variation2.phpt`: Test array_sum() function : usage variations - array with different integer value
#[test]
fn phpt_array_sum_variation2() {
    let out = compile_and_run(r#"<?php
/*
* Testing array_sum() with different types of integer arrays containing data of following type:
*  integer, octal, hexadecimal, maximum and minimum integer values & mixed of all integers
*/

echo "*** Testing array_sum() : different integer array ***\n";

// Int array
$int_values = array(3, 2, 100, 150, 25, 350, 0, -3, -1200);
echo "-- Sum of Integer array --\n";
var_dump( array_sum($int_values) );

// Octal array
$octal_values = array(056, 023, 00, 015, -045, 01, -07);
echo "-- Sum of Octal array --\n";
var_dump( array_sum($octal_values) );

// Hexadecimal array
$hex_values = array(0xAE, 0x2B, 0X10, -0xCF, 0X12, -0XF2);
echo "-- Sum of Hex array --\n";
var_dump( array_sum($hex_values) );

// Mixed values int, octal & hex
$mixed_int_value = array(2, 5, -1, 054, 0X3E, 0, -014, -0x2A);
echo "-- Sum of mixed integer values --\n";
var_dump( array_sum($mixed_int_value) );

echo "Done"
?>
"#);
    assert_eq!(out, "*** Testing array_sum() : different integer array ***\n-- Sum of Integer array --\nint(-573)\n-- Sum of Octal array --\nint(35)\n-- Sum of Hex array --\nint(-198)\n-- Sum of mixed integer values --\nint(58)\nDone");
}

/// php-src `ext/standard/tests/array/array_values_basic.phpt`: Test array_values() function : basic functionality
#[test]
fn phpt_array_values_basic() {
    let out = compile_and_run(r#"<?php
/*
 * Test basic functionality of array_values()
 */

echo "*** Testing array_values() : basic functionality ***\n";


// Initialise all required variables
$input = array('zero', 'one', 'two', 'three' => 3, 10 => 'ten');

// Calling array_values() with all possible arguments
var_dump( array_values($input) );

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_values() : basic functionality ***\narray(5) {\n  [0]=>\n  string(4) \"zero\"\n  [1]=>\n  string(3) \"one\"\n  [2]=>\n  string(3) \"two\"\n  [3]=>\n  int(3)\n  [4]=>\n  string(3) \"ten\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/array_values_variation7.phpt`: Test array_values() function : usage variations - Internal order check
#[test]
fn phpt_array_values_variation7() {
    let out = compile_and_run(r#"<?php
/*
 * Check that array_values is re-assigning keys according to the internal order of the array,
 * and is not dependent on the \$input argument's keys
 */

echo "*** Testing array_values() : usage variations ***\n";

// populate array with 'default' keys in reverse order
$input = array(3 => 'three', 2 => 'two', 1 => 'one', 0 => 'zero');

echo "\n-- \$input argument: --\n";
var_dump($input);

echo "\n-- Result of array_values() --\n";
var_dump(array_values($input));

echo "Done";
?>
"#);
    assert_eq!(out, "*** Testing array_values() : usage variations ***\n\n-- $input argument: --\narray(4) {\n  [3]=>\n  string(5) \"three\"\n  [2]=>\n  string(3) \"two\"\n  [1]=>\n  string(3) \"one\"\n  [0]=>\n  string(4) \"zero\"\n}\n\n-- Result of array_values() --\narray(4) {\n  [0]=>\n  string(5) \"three\"\n  [1]=>\n  string(3) \"two\"\n  [2]=>\n  string(3) \"one\"\n  [3]=>\n  string(4) \"zero\"\n}\nDone");
}

/// php-src `ext/standard/tests/array/bug21918.phpt`: Bug #21918 (different handling of positive vs. negative array indexes)
#[test]
fn phpt_bug21918() {
    let out = compile_and_run(r#"<?php

echo "==Mixed==\n";
$a = array(-1=>'a', '-2'=>'b', 3=>'c', '4'=>'d', 5=>'e', '6001'=>'f', '07'=>'g');

foreach($a as $k => $v) {
    var_dump($k);
    var_dump($v);
}

echo "==Normal==\n";
$b = array();
$b[] = 'a';

foreach($b as $k => $v) {
    var_dump($k);
    var_dump($v);
}

echo "==Negative==\n";
$c = array('-2' => 'a');

foreach($c as $k => $v) {
    var_dump($k);
    var_dump($v);
}

?>
"#);
    assert_eq!(out, "==Mixed==\nint(-1)\nstring(1) \"a\"\nint(-2)\nstring(1) \"b\"\nint(3)\nstring(1) \"c\"\nint(4)\nstring(1) \"d\"\nint(5)\nstring(1) \"e\"\nint(6001)\nstring(1) \"f\"\nstring(2) \"07\"\nstring(1) \"g\"\n==Normal==\nint(0)\nstring(1) \"a\"\n==Negative==\nint(-2)\nstring(1) \"a\"\n");
}

/// php-src `ext/standard/tests/array/bug25758.phpt`: Bug #25758 (var_export does not escape ' & \ inside array keys)
#[test]
fn phpt_bug25758() {
    let out = compile_and_run(r#"<?php
    $a = array ("quote'" => array("quote'"));
    echo var_export($a, true);
?>
"#);
    assert_eq!(out, "array (\n  'quote\\'' => \n  array (\n    0 => 'quote\\'',\n  ),\n)");
}

/// php-src `ext/standard/tests/array/bug28974.phpt`: Bug #28974 (array_(p)slice() treats large lengths incorrectly - overflow)
#[test]
fn phpt_bug28974() {
    let out = compile_and_run(r#"<?php
$a = $b = $c = array(0,1,2,3,4,5);
print_r($a);
// this is ok:
print_r(array_slice($a,2,2147483645));

// this is wrong:
print_r(array_slice($a,2,2147483646));
echo 'print_r(array_splice($a,2,1));'."\n";
print_r(array_splice($a,2,1));
echo "\$a is :";
print_r($a);
echo 'print_r(array_splice($b,2,2147483645));'."\n";
print_r(array_splice($b,2,2147483645));
echo "\$b is :";
print_r($b);

// this is wrong:
echo 'print_r(array_splice($c,2,2147483646));'."\n";
print_r(array_splice($c,2,2147483646));
echo "\$c is :";
print_r($c);
?>
"#);
    assert_eq!(out, "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n    [3] => 3\n    [4] => 4\n    [5] => 5\n)\nArray\n(\n    [0] => 2\n    [1] => 3\n    [2] => 4\n    [3] => 5\n)\nArray\n(\n    [0] => 2\n    [1] => 3\n    [2] => 4\n    [3] => 5\n)\nprint_r(array_splice($a,2,1));\nArray\n(\n    [0] => 2\n)\n$a is :Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 3\n    [3] => 4\n    [4] => 5\n)\nprint_r(array_splice($b,2,2147483645));\nArray\n(\n    [0] => 2\n    [1] => 3\n    [2] => 4\n    [3] => 5\n)\n$b is :Array\n(\n    [0] => 0\n    [1] => 1\n)\nprint_r(array_splice($c,2,2147483646));\nArray\n(\n    [0] => 2\n    [1] => 3\n    [2] => 4\n    [3] => 5\n)\n$c is :Array\n(\n    [0] => 0\n    [1] => 1\n)\n");
}

/// php-src `ext/standard/tests/array/bug34227.phpt`: Bug #34277 (array_filter() crashes with references and objects)
#[test]
fn phpt_bug34227() {
    let out = compile_and_run(r#"<?php

class C
{
  function m1()
  {
    $this->m2();
  }

  function m2()
  {
    $this->m3();
  }

  function m3()
  {
    $this->m4();
  }

  function m4()
  {
    $this->m5();
  }

  function m5()
  {
    $this->m6();
  }

  function m6()
  {
    $this->m7();
  }

  function m7()
  {
    $this->m8();
  }

  function m8()
  {
    $this->m9();
  }

  function m9()
  {
    $this->m10();
  }

  function m10()
  {
    $this->m11(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
  }

  function m11($a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9, $a10)
  {
    $arr = explode('a', 'b');
  }
}

function f($str)
{
  $obj = new C;
  $obj->m1();
  return TRUE;
}

function p5($a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9, $a10, $a11, $a12)
{
  $ret = array_filter(array(0), 'f');
}

function p4()
{
  p5(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
}

function p3()
{
  p4();
}

function p2()
{
  p3();
}

function p1()
{
  p2();
}

p1();
echo "ok\n";
?>
"#);
    assert_eq!(out, "ok\n");
}

/// php-src `ext/standard/tests/array/bug52534.phpt`: Bug #52534 (var_export array with negative key)
#[test]
fn phpt_bug52534() {
    let out = compile_and_run(r#"<?php

$aArray = array ( -1 => 'Hello');

var_export($aArray);

?>
"#);
    assert_eq!(out, "array (\n  -1 => 'Hello',\n)");
}

/// php-src `ext/standard/tests/array/bug69166.phpt`: Fixed #69166 (Assigning array_values() to array does not reset key counter)
#[test]
fn phpt_bug69166() {
    let out = compile_and_run(r#"<?php

$array = [0];
$ar = array_values($array);
$ar[] = 1;
var_dump($ar);
?>
"#);
    assert_eq!(out, "array(2) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n}\n");
}

/// php-src `ext/standard/tests/array/bug69723.phpt`: Bug #69723 (Passing parameters by reference and array_column)
#[test]
fn phpt_bug69723() {
    let out = compile_and_run(r#"<?php
function byReference( & $array){
    foreach($array as &$item){
        $item['nanana'] = 'batman';
        $item['superhero'] = 'robin';
    }
}

$array = [
    [
    'superhero'=> 'superman',
    'nanana' => 'no nana'
    ],
    [
    'superhero'=> 'acuaman',
    'nanana' => 'no nana'
    ],

    ];

var_dump(array_column($array, 'superhero'));
byReference($array);
var_dump(array_column($array, 'superhero'));
?>
"#);
    assert_eq!(out, "array(2) {\n  [0]=>\n  string(8) \"superman\"\n  [1]=>\n  string(7) \"acuaman\"\n}\narray(2) {\n  [0]=>\n  string(5) \"robin\"\n  [1]=>\n  string(5) \"robin\"\n}\n");
}

/// php-src `ext/standard/tests/array/bug72116.phpt`: Bug #72116 (insertion after array_fill fails)
#[test]
fn phpt_bug72116() {
    let out = compile_and_run(r#"<?php

$x = array_fill(0, 1, '..');
$x[] = 'a';
var_dump($x);

?>
"#);
    assert_eq!(out, "array(2) {\n  [0]=>\n  string(2) \"..\"\n  [1]=>\n  string(1) \"a\"\n}\n");
}

/// php-src `ext/standard/tests/array/bug75433.phpt`: array_values() preserves next index from source array when shallow-copying
#[test]
fn phpt_bug75433() {
    let out = compile_and_run(r#"<?php

$a = [1,2,3];
unset($a[2]);
$b = array_values($a);
$b[] = 4;
print_r($b);
?>
"#);
    assert_eq!(out, "Array\n(\n    [0] => 1\n    [1] => 2\n    [2] => 4\n)\n");
}

/// php-src `ext/standard/tests/array/max_variation1.phpt`: Test variations in usage of max()
#[test]
fn phpt_max_variation1() {
    let out = compile_and_run(r#"<?php
/*
 * Function is implemented in ext/standard/array.c
*/

echo "\n*** Testing boundary conditions ***\n";
var_dump(max(2147483645, 2147483646) );
var_dump(max(2147483647, 2147483648) );
var_dump(max(2147483646, 2147483648) );
var_dump(max(-2147483647, -2147483646) );
var_dump(max(-2147483648, -2147483647) );
var_dump(max(-2147483649, -2147483647) );

echo "\n*** Testing large number of arguments ***\n";

var_dump(max(0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,21));

echo "\nDone\n";
?>
"#);
    assert_eq!(out, "\n*** Testing boundary conditions ***\nint(2147483646)\nint(2147483648)\nint(2147483648)\nint(-2147483646)\nint(-2147483647)\nint(-2147483647)\n\n*** Testing large number of arguments ***\nint(21)\n\nDone\n");
}

/// php-src `ext/standard/tests/array/min_variation1.phpt`: Test variations in usage of min()
#[test]
fn phpt_min_variation1() {
    let out = compile_and_run(r#"<?php
/*
 * Function is implemented in ext/standard/array.c
*/

echo "\n*** Testing boundary conditions ***\n";
var_dump(min(2147483645, 2147483646) );
var_dump(min(2147483647, 2147483648) );
var_dump(min(2147483646, 2147483648) );
var_dump(min(-2147483647, -2147483646) );
var_dump(min(-2147483648, -2147483647) );
var_dump(min(-2147483649, -2147483647) );

echo "\n*** Testing large number of arguments ***\n";

var_dump(min(0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,0, 1 , 2, 3, 4, 5, 6, 7, 8, 9, 0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,21));

echo "\nDone\n";
?>
"#);
    assert_eq!(out, "\n*** Testing boundary conditions ***\nint(2147483645)\nint(2147483647)\nint(2147483646)\nint(-2147483647)\nint(-2147483648)\nint(-2147483649)\n\n*** Testing large number of arguments ***\nint(0)\n\nDone\n");
}

/// php-src `ext/standard/tests/array/negative_index_empty_array.phpt`: Test empty arrays with first added index being negative
#[test]
fn phpt_negative_index_empty_array() {
    let out = compile_and_run(r#"<?php

$a = [];
$a[-5] = "-5";
$a[] = "after -5";

var_dump($a);
?>
"#);
    assert_eq!(out, "array(2) {\n  [-5]=>\n  string(2) \"-5\"\n  [-4]=>\n  string(8) \"after -5\"\n}\n");
}

/// php-src `ext/standard/tests/array/range/range_bug71132.phpt`: Bug #71132 (range function produces 2 segfaults with long integers)
#[test]
fn phpt_range_bug71132() {
    let out = compile_and_run(r#"<?php
var_dump(count(range(PHP_INT_MIN + 513, PHP_INT_MIN)));
var_dump(count(range(PHP_INT_MIN, PHP_INT_MIN + 513)));
?>
"#);
    assert_eq!(out, "int(514)\nint(514)\n");
}

/// php-src `ext/standard/tests/array/sort/bug33382.phpt`: Bug #33382 (array_reverse() fails after *sort() )
#[test]
fn phpt_bug33382() {
    let out = compile_and_run(r#"<?php

$array = array(1,2,3,4,5);

sort($array);

var_dump(array_reverse($array));

echo "Done\n";
?>
"#);
    assert_eq!(out, "array(5) {\n  [0]=>\n  int(5)\n  [1]=>\n  int(4)\n  [2]=>\n  int(3)\n  [3]=>\n  int(2)\n  [4]=>\n  int(1)\n}\nDone\n");
}

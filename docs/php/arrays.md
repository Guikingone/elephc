---
title: "Arrays"
description: "Indexed arrays, associative arrays, copy-on-write, and built-in array functions."
sidebar:
  order: 7
---

## Indexed arrays
```php
<?php
$arr = [10, 20, 30];
echo $arr[0];          // 10
echo count($arr);      // 3
$arr[1] = 99;          // modify
$arr[] = 40;           // push
```

## Long-form `array()` syntax
The short `[...]` form and the long-form `array(...)` construct are exactly equivalent — `array(...)` is a language construct, not a function call. Both produce the same array and may be mixed freely, including `key => value` entries, `...` spreads, and nesting. The keyword is case-insensitive.

```php
<?php
$indexed = array(10, 20, 30);
$assoc   = array("name" => "Alice", "city" => "Paris");
$nested  = array("point" => array(1, 2), "label" => "p");
$spread  = array(...$indexed, 40);   // 10, 20, 30, 40
```

## String arrays
```php
<?php
$names = ["Alice", "Bob", "Charlie"];
foreach ($names as $name) {
    echo "Hello, " . $name . "\n";
}
```

## Heterogeneous indexed arrays
Indexed arrays can contain different value types. When element types differ, elephc stores the payloads as boxed `mixed` values internally.

```php
<?php
$items = [1, "two", true];
$items[] = 3.5;

echo $items[0]; // 1
echo $items[1]; // two
```

## Associative arrays
```php
<?php
$map = ["name" => "Alice", "city" => "Paris"];
echo $map["name"];       // Alice
$map["age"] = "30";      // add new key
```

Associative arrays use a hash table runtime. If later values do not match the first value type, the checker widens to internal `mixed` runtime shape.

Keys follow PHP's array-key normalization. Integer keys remain integers, booleans and floats normalize to integer keys, numeric strings such as `"1"` normalize to the integer key `1`, and strings with leading zeroes such as `"01"` remain string keys. This applies to literals, reads and writes, `foreach`, `array_keys()`, `array_search()`, `array_key_exists()`, `array_flip()`, JSON object keys, and array union.

```php
<?php
$map = [1 => "one", "2" => "two", "02" => "leading"];

echo $map["1"];  // one
echo $map[2];    // two
echo $map["02"]; // leading
```

## Removing elements with unset

`unset($map[$key])` removes a single entry from an associative array. The removed key's owned
key and value storage is released, the live `count()` drops by one, and `isset()`, `foreach`,
and re-insertion all observe the entry as gone. Iteration order follows PHP: surviving entries
keep their original order, and re-adding a removed key appends it at the end.

```php
<?php
$map = ["a" => 1, "b" => 2, "c" => 3];
unset($map["b"]);

echo count($map);          // 2
echo isset($map["b"]) ? "y" : "n"; // n
$map["b"] = 9;             // re-added at the end
foreach ($map as $k => $v) { echo "$k=$v "; } // a=1 c=3 b=9
```

`unset()` also works on indexed arrays. PHP removes the key **without renumbering** the survivors,
so the array becomes sparse (a hole is left). The remaining keys keep their original values, and a
later `$arr[] = ...` append continues at `max_key + 1`.

```php
<?php
$arr = [1, 2, 3];
unset($arr[1]);
foreach ($arr as $k => $v) { echo "$k=$v "; } // 0=1 2=3 (no key 1)
$arr[] = 9;                                    // appended at key 3
echo isset($arr[1]) ? "y" : "n";               // n
```

`unset()` respects copy-on-write: removing a key from one array never mutates another array that
was assigned from it. Unsetting a key that is not present is a no-op.

```php
<?php
$a = ["x" => 1, "y" => 2];
$b = $a;
unset($b["x"]);
echo count($a); // 2 — original is untouched
echo count($b); // 1
```

> Removing an element from an array passed **by reference** (`function f(array &$a)`) is not yet
> supported and reports a compile error.

## Reference into a local array element

An element of a **local array** can be reference-aliased to a plain variable with `= &$var`, so the
element and the variable then share one storage cell — a write through either side is observed through
the other. This works for an explicit key (`$a[$k] = &$var`), an appended element (`$a[] = &$var`), and
an appended element of a nested local array (`$loops[$k][] = &$var`, auto-vivifying `$loops[$k]` when
absent):

```php
<?php
$loops = [];
$path  = [1];
$loops["c"][] = &$path;  // the appended element aliases $path
$path[] = 2;             // mutating $path is visible through the element
echo count($loops["c"][0]); // 2
```

The reference **source** must be a plain variable whose value fits in one machine word (an array,
object, or scalar). A source that is itself an array element (`&$b[$j]`), a property (`&$o->p`), or a
string-valued variable is a compile-time error, as is appending a reference into a static-property or
instance-property array (`self::$a[] = &$var`, `$o->p[] = &$var`) — those forms are follow-up features.

### Nested write through a reference-bound variable

A **nested** lvalue write (two or more index levels) whose base is a reference-bound local
writes through the shared cell so the mutation is observable via both the alias and the source
array:

```php
<?php
$arr = [[1, [2, 3]]];
$x = &$arr[0];        // $x aliases $arr[0]
$x[1][0] = 9;        // nested write through the alias
echo $arr[0][1][0];  // 9 — the source array sees the mutation
```

This works for any depth (`$x[1][0][1] = 9`), nested append (`$x[1][] = 9`), nested string-key
write (`$x["a"]["b"] = 9`), and nested write through a ref-promoted source (`$a[] = &$p;
$p[1][0] = 9`). Writing into a scalar alias (`$x = 5; $x[0] = 9`) is a compile-time error
("Cannot use a scalar value as an array").

A whole-value reassign of the alias (`$x = [5, 6]`) or `unset($x)` breaks the alias as in PHP:
the source array element is updated on reassign, and the alias is detached on unset.

### By-reference entries in array literals

An array literal may bind entries **by reference** with `&`, both keyed (`['s' => &$v]`) and
positional (`[&$v]`), in the short `[...]` and long `array(...)` forms alike. The entry aliases the
source variable exactly like the statement forms above — a later write to the source is visible
through the entry, copies of the array share the reference cell, and `unset($v)` keeps the entry
alive:

```php
<?php
$v = 5;
$arr = ['a' => 1, 's' => &$v, 'b' => 2];
$v = 9;
echo $arr['s'], " ", $arr['a'], $arr['b']; // 9 12
$b = $arr;                                  // the copy shares the reference cell
$v = 42;
echo $arr['s'], " ", $b['s'];              // 42 42
```

Entries are evaluated in source order, and positional entries follow PHP's next-integer-key rule
even when mixed with explicit integer keys (`[&$a, 5 => &$b, &$c]` produces keys 0, 5, and 6).
Nested literals (`['x' => ['y' => &$v]]`), return position (`return ['session' => &$v];`,
including under a declared `: array` return type and through `??`/ternary chains),
call-argument position (`takes(['s' => &$v])`), and conditional value positions
(`$c ? ['s' => &$v] : ['s' => &$w]`, `$x ?? ['s' => &$v]`, `match` arms) work too.

Duplicate keys follow PHP's replace semantics: a later entry with the same key **replaces** the
bucket, so `['s' => &$v, 's' => 2]` discards the reference (leaving `$v` untouched) and stores
`2`, while `['t' => 1, 't' => &$w]` leaves the reference binding in place.

The reference source rules match the statement forms: the source must be a variable-rooted
lvalue immediately after `&` (a parenthesized source like `[&($v)]` is a parse error, as in
PHP), `&f()` is rejected with PHP's "Can't use function return value in write context", a
string-valued source is a compile-time error, and a superglobal (`&$_GET`) or `global`-imported
source is a loud "not yet supported" error. A spread (`...$xs`) inside a literal that also has a
by-reference entry is not yet supported and reports a compile error.

Function, method, and closure **parameters** work as reference sources (`function mk(int $x):
array { return ['s' => &$x]; }` — the entry aliases the parameter with its incoming argument
value), including `mixed`-typed, `array`-typed, and variadic (`...$rest`) parameters. A
**by-reference parameter** (`int &$x`) or a by-reference closure capture (`use (&$b)`) as the
reference source is a loud "not yet supported" compile error.


## Array union

`+` between arrays follows PHP union semantics: keys from the left operand win, and only keys that are missing from the left are copied from the right.

```php
<?php
$left = ["a" => "left", "b" => "keep"];
$right = ["a" => "right", "c" => "new"];
$result = $left + $right;

echo $result["a"]; // left
echo $result["c"]; // new
```

For indexed arrays, numeric keys are preserved. In elephc's dense indexed-array representation, this means the left side keeps indexes `0..count($left)-1`, and only the right suffix with higher numeric indexes is appended.

```php
<?php
$result = [10, 20] + [99, 88, 77];
echo $result[0]; // 10
echo $result[1]; // 20
echo $result[2]; // 77
```

Union also works across indexed and associative representations. Indexed positions become integer keys in the shared PHP key space, so an associative key `"0"` blocks right index `0`, while `"01"` remains a distinct string key.

```php
<?php
$left = ["0" => "left zero", "01" => "leading"];
$right = ["right zero", "right one"];
$result = $left + $right;

echo $result[0];    // left zero
echo $result[1];    // right one
echo $result["01"]; // leading
```

## Copy-on-write semantics
Arrays are shared until modified, matching PHP's by-value behavior:
```php
<?php
$a = [1, 2];
$b = $a;      // shares storage
$b[0] = 9;    // first write detaches $b
echo $a[0];   // 1
echo $b[0];   // 9
```

The same applies to function parameters and mutating built-ins (`array_push()`, `sort()`, `shuffle()`, etc.).

## Multi-dimensional arrays
```php
<?php
$matrix = [[1, 2], [3, 4]];
echo $matrix[0][1];    // 2
```

## Array destructuring

Array destructuring assigns array elements to writable targets. Both short syntax and `list(...)` are supported.

```php
<?php
[$first, , $third] = [10, 20, 30];
echo $first; // 10
echo $third; // 30

list($left, $right) = [1, 2];
```

Patterns can be nested, keyed, and can write to the same target forms as ordinary assignments.

```php
<?php
[[$a, $b], [$c, $d]] = [[1, 2], [3, 4]];

["name" => $name, "role" => $role] = ["role" => "admin", "name" => "Ada"];

$items = [0];
[$items[0], $items[]] = [5, 6];
```

PHP does not allow keyed and unkeyed entries in the same destructuring pattern, and elephc reports that as a compile-time error.

Destructuring can also be used in **expression position** — for example as a condition. It binds the targets and evaluates to `EXPR` (the whole right-hand side), exactly as in PHP, so the classic assign-and-test idiom works:

```php
<?php
if ([$id, $name] = $row) {
    echo $id . ":" . $name;
}
```

Every pattern the statement form supports works in expression position too: skipped slots, keyed entries, nested patterns, non-variable targets, and the `list(...)` construct. A falsy right-hand side (such as a missing key falling back to `?? null`) makes the whole expression falsy, and destructuring a runtime `null` binds `null` to every target, as in PHP:

```php
<?php
// Skipped slots + a nullable source: take the branch only when the row exists.
if ([, , , $access] = $propertyScopes[$name] ?? null) {
    echo $access;
}

// Keyed, nested, and list() patterns work the same way.
if (["level" => $level] = $config) { echo $level; }
if ([[$a, $b], [$c, $d]] = $pairs) { echo $a + $c; }
while (list(, $tag) = $queue[$i] ?? null) { $i++; }

// The expression's value is the full right-hand side.
$row = [, $second] = [1, 2];
echo count($row); // 2
```

Inside a ternary branch or the right operand of `&&`/`||`, the destructuring only runs when that branch is actually taken, matching PHP's evaluation order. A right-hand side the compiler can prove is `null` at compile time is rejected as a compile-time error (`Cannot index non-array`); runtime-nullable sources are fine.

Destructuring can also appear directly in a `foreach` value pattern. The pattern is bound from each element (or each value, when a key is present), so you can unpack rows while iterating.

```php
<?php
// Positional: bind each element pair.
foreach ([["a", "b"], ["c", "d"]] as [$x, $y]) {
    echo $x . $y; // abcd
}

// Keyed: pick fields by name from each row.
foreach ([["id" => 1, "name" => "Ada"]] as ["id" => $id, "name" => $name]) {
    echo $id . ':' . $name; // 1:Ada
}

// Key with a destructured value.
foreach (["k" => [1, 2]] as $key => [$m, $n]) {
    echo $key . $m . $n; // k12
}
```

By-reference destructuring targets (`foreach ($arr as [&$a, $b])` and `[&$a, $b] = $arr`) are not supported; the pattern must bind by value.

## Built-in array functions

| Function | Signature | Description |
|---|---|---|
| `count()` | `count($arr_or_countable): int` | Number of elements; on objects implementing `Countable`, dispatches to `count()` |
| `array_push()` | `array_push($arr, $val): void` | Add element to end |
| `array_pop()` | `array_pop($arr): mixed` | Remove and return last element |
| `in_array()` | `in_array($needle, $arr [, $strict]): bool` | Search for value. Pass `$strict = true` for identity (`===`) comparison so a value only matches an element of the same type |
| `array_keys()` | `array_keys($arr): array` | Returns the array keys |
| `array_values()` | `array_values($arr): array` | Returns copy of values |
| `array_key_exists()` | `array_key_exists($key, $arr): bool` | Check if key exists |
| `array_search()` | `array_search($needle, $arr): int\|string\|false` | Search for value, returning an integer index for indexed arrays, the first matching associative-array key, or `false` if not found |
| `array_slice()` | `array_slice($arr, $offset [, $length]): array` | Extract a slice |
| `array_splice()` | `array_splice($arr, $offset [, $length]): array` | Remove a slice in place and return the removed elements |
| `array_chunk()` | `array_chunk($arr, $size): array` | Split into chunks |
| `array_merge()` | `array_merge(...$arrays): array` | Merge one or more arrays (variadic; `array_merge()` returns an empty array) |
| `array_replace()` | `array_replace($arr, ...$replacements): array` | Overlay later arrays onto the first, last-wins by key (integer keys are preserved, not renumbered), keeping the first array's insertion order. Associative-array arguments that share one element type are supported |
| `array_is_list()` | `array_is_list($arr): bool` | Returns `true` when the array's keys are exactly the integers `0..count-1` in order (an empty array is a list) |
| `array_combine()` | `array_combine($keys, $values): array` | Create array from keys/values |
| `array_fill()` | `array_fill($start, $num, $value): array` | Fill with values |
| `array_fill_keys()` | `array_fill_keys($keys, $value): array` | Fill with values using keys |
| `array_pad()` | `array_pad($arr, $size, $value): array` | Pad to length |
| `range()` | `range($start, $end): array` | Sequential integers |
| `array_diff()` | `array_diff($arr1, $arr2): array` | Values in $arr1 not in $arr2 |
| `array_intersect()` | `array_intersect($arr1, $arr2): array` | Values in both |
| `array_diff_key()` | `array_diff_key($arr1, $arr2): array` | Keys in $arr1 not in $arr2 |
| `array_intersect_key()` | `array_intersect_key($arr1, $arr2): array` | Keys in both |
| `array_unique()` | `array_unique($arr): array` | Remove duplicates |
| `array_reverse()` | `array_reverse($arr): array` | Reverse order |
| `array_flip()` | `array_flip($arr): array` | Exchange keys and values, normalizing integer and numeric-string result keys |
| `array_shift()` | `array_shift($arr): mixed` | Remove and return first |
| `array_unshift()` | `array_unshift($arr, $value): int` | Prepend element |
| `array_sum()` | `array_sum($arr): int\|float` | Sum of values |
| `array_product()` | `array_product($arr): int\|float` | Product of values |
| `array_column()` | `array_column($arr, $column_key): array` | Extract column from array of assoc rows |
| `sort()` | `sort($arr): void` | Sort ascending (in-place) |
| `rsort()` | `rsort($arr): void` | Sort descending |
| `asort()` | `asort($arr): void` | Sort by value, maintain keys |
| `arsort()` | `arsort($arr): void` | Sort by value desc, maintain keys |
| `ksort()` | `ksort($arr): void` | Sort by key ascending |
| `krsort()` | `krsort($arr): void` | Sort by key descending |
| `natsort()` | `natsort($arr): void` | Natural order sort |
| `natcasesort()` | `natcasesort($arr): void` | Case-insensitive natural sort |
| `shuffle()` | `shuffle($arr): void` | Randomly shuffle (in-place) |
| `array_rand()` | `array_rand($arr): int` | Pick one random key |
| `array_map()` | `array_map($callback, $arr): array` | Apply callback to each element |
| `array_filter()` | `array_filter($arr, $callback, $mode = ARRAY_FILTER_USE_VALUE): array` | Filter where callback is truthy; mode selects value, key, or both callback args |
| `array_reduce()` | `array_reduce($arr, $callback, $init): int` | Reduce to single value |
| `array_walk()` | `array_walk($arr, $callback): void` | Call callback on each element |
| `usort()` | `usort($arr, $callback): void` | Sort with user comparison |
| `uksort()` | `uksort($arr, $callback): void` | Sort by key with user comparison |
| `uasort()` | `uasort($arr, $callback): void` | Sort with user comparison, maintain keys |
| `call_user_func()` | `call_user_func($callback, ...): mixed` | Call a callback value |
| `call_user_func_array()` | `call_user_func_array($callback, $args): mixed` | Call with args from array |
| `function_exists()` | `function_exists("name"): bool` | Check if a literal global or fully-qualified function name is defined |
| `isset()` | `isset($var, ...$vars): int` | Check that every variable or offset is defined and not null |

`array_filter()` accepts `ARRAY_FILTER_USE_VALUE` (`0`), `ARRAY_FILTER_USE_BOTH` (`1`), and `ARRAY_FILTER_USE_KEY` (`2`). Invalid mode values throw `ValueError`.

> Callback arguments can be string literals, runtime string names for user functions, first-class callable values, anonymous functions, arrow functions, or variables holding captured closures. `array_map()`, `array_filter()`, `array_reduce()`, `array_walk()`, `usort()`, `uksort()`, and `uasort()` resolve runtime string callback variables through descriptor dispatch. `array_map()` stores mixed result elements when the selected callback return shape is only known at runtime. `array_map()` also runs over a heterogeneous (boxed `mixed`) input array: each element is passed to the callback as a `mixed` value, so a callback with a `mixed` (or untyped) parameter sees and can return each element with its original runtime type.
> `call_user_func_array()` also accepts dynamic indexed and associative argument arrays for callbacks with a known signature, including userland variadic callbacks. When a callable value has no single static signature at the call site, elephc emits an AOT runtime dispatch over user functions and closure/FCC wrappers available in that codegen context, then applies the matched target's descriptor metadata: parameter names, defaults, by-reference flags, variadic position, return shape, captures, hidden receiver arguments, and callable shape. Runtime string callback names dispatch over user functions, supported builtins, and public static-method strings by case-insensitive name matching, materialize the matched descriptor, and invoke its generated descriptor invoker. Descriptor invokers receive a temporary boxed Mixed clone of the argument container and inspect its runtime tag to handle indexed arrays and associative hashes through the same signature-level wrapper, so the source `$args` remains usable with its original static layout after the call. String keys bind named parameters; unconsumed string and numeric keys are copied into `...$rest` for variadic callbacks. Dynamic arrays passed to by-reference callback parameters use temporary reference cells, so callback writes do not mutate the source argument array.

`usort()` and `uasort()` sort arrays of **objects** as well as scalars. The comparator receives each element as its object handle, so an unannotated comparator's parameters are typed from the array element automatically — `usort($items, fn($a, $b) => $a->weight <=> $b->weight)` works without writing `($a, $b)` type hints, and `usort($dates, fn($a, $b) => $a <=> $b)` over `DateTime`/`DateTimeImmutable` compares by instant. Explicit hints (`function (Item $a, Item $b)`) are equally accepted. Sorting an array of **strings** with a user comparator is not yet supported and reports a clear unsupported-feature error.

**Not supported by design:** `compact()`, `extract()` require runtime variable-name tables and are listed in the roadmap's "Will not implement" section.

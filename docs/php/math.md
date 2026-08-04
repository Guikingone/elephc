---
title: "Math"
description: "Mathematical functions: abs, floor, ceil, round, clamp, trigonometry, logarithms, and more."
sidebar:
  order: 9
---

## Built-in math functions

| Function | Signature | Description |
|---|---|---|
| `abs()` | `abs($val): int\|float` | Absolute value (preserves type); `abs(PHP_INT_MIN)` has no `int` result and promotes to `float` |
| `floor()` | `floor($val): float` | Round down |
| `ceil()` | `ceil($val): float` | Round up |
| `round()` | `round($val [, $precision]): float` | Round to nearest |
| `sqrt()` | `sqrt($val): float` | Square root |
| `pow()` | `pow($base, $exp): float` | Exponentiation |
| `min()` | `min($value, ...$values): mixed` | Minimum. Either one array, or two or more values |
| `max()` | `max($value, ...$values): mixed` | Maximum. Either one array, or two or more values |
| `clamp()` | `clamp(?mixed $value, ?mixed $min, ?mixed $max): ?mixed` | Clamp a value to inclusive bounds |
| `intdiv()` | `intdiv($a, $b): int` | Integer division; a zero divisor raises a catchable `DivisionByZeroError`, and `intdiv(PHP_INT_MIN, -1)` an `ArithmeticError` |
| `fmod()` | `fmod($a, $b): float` | Float modulo |
| `fdiv()` | `fdiv($a, $b): float` | Float division (returns INF for /0) |
| `rand()` | `rand([$min, $max]): int` | Random integer |
| `mt_rand()` | `mt_rand([$min, $max]): int` | Alias for rand(), except that `$min > $max` throws `\ValueError` instead of swapping the bounds |
| `random_int()` | `random_int($min, $max): int` | Cryptographic random. `$min > $max` throws `\ValueError`. |
| `sin()` | `sin($angle): float` | Sine (radians) |
| `cos()` | `cos($angle): float` | Cosine (radians) |
| `tan()` | `tan($angle): float` | Tangent (radians) |
| `asin()` | `asin($val): float` | Arc sine |
| `acos()` | `acos($val): float` | Arc cosine |
| `atan()` | `atan($val): float` | Arc tangent |
| `atan2()` | `atan2($y, $x): float` | Two-argument arc tangent |
| `sinh()` | `sinh($val): float` | Hyperbolic sine |
| `cosh()` | `cosh($val): float` | Hyperbolic cosine |
| `tanh()` | `tanh($val): float` | Hyperbolic tangent |
| `log()` | `log($num [, $base]): float` | Logarithm |
| `log2()` | `log2($num): float` | Base-2 logarithm |
| `log10()` | `log10($num): float` | Base-10 logarithm |
| `exp()` | `exp($val): float` | e^x |
| `hypot()` | `hypot($x, $y): float` | Hypotenuse |
| `deg2rad()` | `deg2rad($degrees): float` | Degrees to radians |
| `rad2deg()` | `rad2deg($radians): float` | Radians to degrees |
| `pi()` | `pi(): float` | Returns M_PI |

`clamp()` validates the bounds before selecting a result. It throws `ValueError` if `$min > $max` or if either bound is `NAN`. Selection checks the upper bound first, then the lower bound.

```php
echo clamp(15, 0, 10);      // 10
echo clamp(3.5, 0.0, 10.0); // 3.5
echo clamp("P", "A", "Z");  // "P"
```

### min() and max()

Both accept PHP's two call forms: a single array whose elements are compared, or
two or more values compared against each other.

```php
var_dump(min([1, 2, 3]));   // int(1)
var_dump(max([1, 2, 3]));   // int(3)
var_dump(min([3.5, 1.25])); // float(1.25)
var_dump(min(4, 9, 2));     // int(2)
```

An empty array has no element to return, so it throws a catchable `ValueError`
exactly like PHP:

```php
try {
    min([]);
} catch (ValueError $e) {
    echo $e->getMessage(); // min(): Argument #1 ($value) must contain at least one element
}
```

The single-array form currently compiles when every element has the same scalar type:
an indexed array of `int`, of `float`, or of `bool`. It is rejected at compile time for

- an array of strings (`min(["a", "b"])`),
- an associative array (`min(["a" => 3, "b" => 1])`),
- and any array with heterogeneous elements — including a mixed int/float literal such
  as `min([1, 2.5])`, whose elements are stored as boxed `mixed` values.

Selecting the smallest element there needs PHP's full comparison table over boxed heap
values, which the reduction does not implement; the diagnostic names the array shape it
saw. Write `min(1, 2.5)` (the variadic form, which has no such restriction) or make the
array uniform (`min([1.0, 2.5])`).

## Math constants

| Constant | Type | Value |
|---|---|---|
| `M_PI` | float | 3.14159265358979... |
| `M_E` | float | 2.71828182845904... |
| `M_SQRT2` | float | 1.41421356237309... |
| `M_PI_2` | float | 1.57079632679489... |
| `M_PI_4` | float | 0.78539816339744... |
| `M_LOG2E` | float | 1.44269504088896... |
| `M_LOG10E` | float | 0.43429448190325... |
| `INF` | float | Positive infinity |
| `NAN` | float | Not a Number |
| `PHP_INT_MAX` | int | 9223372036854775807 |
| `PHP_INT_MIN` | int | -9223372036854775808 |
| `PHP_FLOAT_MAX` | float | ~1.8e308 |
| `PHP_FLOAT_MIN` | float | ~2.2e-308 |
| `PHP_FLOAT_EPSILON` | float | ~2.2e-16 |

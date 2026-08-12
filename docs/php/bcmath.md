---
title: "BCMath"
description: "Arbitrary-precision decimal arithmetic: the 14 PHP bcmath functions."
sidebar:
  order: 22
---

elephc implements PHP's 14 procedural `bcmath` functions through a pure-Rust
decimal bridge. Operations use base-10 digits directly, so values are not
converted through binary floating point and result strings preserve the requested
number of fractional digits.

## Functions

| Function | Signature | Result |
|---|---|---|
| `bcadd()` | `bcadd(string $num1, string $num2, ?int $scale = null): string` | Sum |
| `bcsub()` | `bcsub(string $num1, string $num2, ?int $scale = null): string` | Difference |
| `bcmul()` | `bcmul(string $num1, string $num2, ?int $scale = null): string` | Product |
| `bcdiv()` | `bcdiv(string $num1, string $num2, ?int $scale = null): string` | Quotient |
| `bcmod()` | `bcmod(string $num1, string $num2, ?int $scale = null): string` | Remainder |
| `bcdivmod()` | `bcdivmod(string $num1, string $num2, ?int $scale = null): array` | Indexed `[quotient, remainder]` pair |
| `bcpow()` | `bcpow(string $num, string $exponent, ?int $scale = null): string` | Integral power |
| `bcpowmod()` | `bcpowmod(string $num, string $exponent, string $modulus, ?int $scale = null): string` | Integral modular power |
| `bcsqrt()` | `bcsqrt(string $num, ?int $scale = null): string` | Square root |
| `bccomp()` | `bccomp(string $num1, string $num2, ?int $scale = null): int` | `-1`, `0`, or `1` |
| `bcscale()` | `bcscale(?int $scale = null): int` | Current scale, or previous scale when setting |
| `bcceil()` | `bcceil(string $num): string` | Least integer greater than or equal to the number |
| `bcfloor()` | `bcfloor(string $num): string` | Greatest integer less than or equal to the number |
| `bcround()` | `bcround(string $num, int $precision = 0, int $mode = 1): string` | Rounded decimal string |

The four PHP 8.4 additions—`bcceil()`, `bcfloor()`, `bcround()`, and
`bcdivmod()`—are available in every elephc PHP-version profile, like other
registry builtins.

## Scale and formatting

The process scale starts at `0`. Omitting `$scale`, or passing `null`, reads the
current value set by `bcscale()`; an explicit `0` always means zero fractional
digits. Setting the scale returns its previous value.

```php
echo bcscale(4);                 // 0
echo bcadd('1.234', '5');        // 6.2340
echo bcadd('1.234', '5', 0);     // 6
```

`bcadd()`, `bcsub()`, `bcmul()`, `bcdiv()`, and `bcmod()` truncate to the
selected scale; they do not round. Only `bcround()`, `bcceil()`, and
`bcfloor()` apply rounding. Output is normalized without a leading `+` or
negative zero, and is padded with trailing zeros to the selected scale.

`bcround()` accepts the same integer mode values as elephc's `round()`:
`1` through `8`, with `1` (`PHP_ROUND_HALF_UP` /
`RoundingMode::HalfAwayFromZero`) as the default.

## Accepted numbers and errors

Numeric strings may contain surrounding ASCII whitespace, an optional sign,
and decimal digits with an optional point. Forms such as `.5`, `5.`, and
`+1.20` are valid; empty strings, exponent notation such as `1e2`, multiple
decimal points, and non-numeric text are not.

Malformed numbers, negative or out-of-range scales, invalid powers or square
roots, and unsupported rounding modes throw a catchable `ValueError`.
Division or modulo by zero, including a negative power of zero, throws a
catchable `DivisionByZeroError`. Error messages retain PHP function and
argument names.

## Linking and eval

Using any `bc*` function auto-links `libelephc_bcmath`. Use `--with-bcmath` to
force-link the bridge when calls are reached only through indirection:

```bash
elephc --with-bcmath app.php
```

`extension_loaded('bcmath')` reports `true` when the AOT binary links the
bridge. Dynamic `eval()` also exposes all 14 functions and shares the same
process scale with surrounding AOT code.

## Current scope

The procedural PHP 8.4 surface is supported. `BcMath\Number`, decimal operator
overloading, and the `bcmath.scale` INI directive are not implemented;
`bcscale()` is the supported process-scale interface.

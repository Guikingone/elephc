---
title: "FFI & Extern"
description: "Foreign Function Interface: calling C libraries, extern functions, globals, classes, and callbacks."
sidebar:
  order: 4
---

FFI lets elephc programs call C library functions directly, with automatic type marshalling.

> **Strict mode:** `extern` declarations are an elephc extension with no PHP equivalent. Compiling with [`--strict-php`](../compiling/cli-reference.md#strict-php-mode) rejects them.

## Declaring extern functions
```php
<?php
extern function abs(i32 $n): i32;
extern function getpid(): i32;

// With explicit library
extern "curl" function curl_easy_init(): ptr;

// Block syntax
extern "SDL2" {
    function SDL_Init(u32 $flags): i32;
    function SDL_Quit(): void;
}
```

## Supported C types

| elephc type | C equivalent | Passed in |
|---|---|---|
| `int` (alias `i64`) | `int64_t` / `long` / `ssize_t` | integer argument register |
| `u64` | `uint64_t` / `size_t` | integer argument register |
| `i32` / `u32` | `int32_t` / `uint32_t` (C `int` / `unsigned`) | integer argument register |
| `i16` / `u16` | `int16_t` / `uint16_t` (`short`) | integer argument register |
| `i8` / `u8` | `int8_t` / `uint8_t` (`signed char` / `unsigned char`) | integer argument register |
| `float` (alias `f64`) | `double` | floating-point argument register |
| `f32` | `float` | floating-point argument register |
| `bool` | `bool` (`_Bool`, one byte) | integer argument register |
| `string` | `char*` (auto null-terminated) | integer argument register |
| `ptr` | `void*` | integer argument register |
| `ptr<T>` | `T*` | integer argument register |
| `void` | void (return only) | — |
| `callable` | function pointer | integer argument register |

Argument registers follow the selected target's C ABI: AArch64 uses
`x0`-`x7` for integers/pointers and `d0`-`d7` for doubles, while Linux
x86_64 uses the System V registers (`rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
for integers/pointers and `xmm0`-`xmm7` for doubles).

## Integer widths

On the PHP side every C integer is an `int`: `$t = SDL_GetTicks()` is a plain
64-bit PHP integer whatever width the declaration names, and it can be passed
to any extern parameter declared `int`, `u32`, `i64`, and so on. The width only
describes the C side, and it matters because both supported ABIs leave the bits
above a narrow value unspecified: a function returning a C `int` may leave
garbage in the upper 32 bits of the return register, and a callee compiled by
clang expects the caller to extend arguments narrower than 32 bits.

Declare the width the C header uses:

| C type | elephc extern type |
|---|---|
| `int`, `int32_t` | `i32` |
| `unsigned`, `uint32_t`, `Uint32` | `u32` |
| `long`, `int64_t`, `ssize_t`, `intptr_t`, `off_t` | `int` (or `i64`) |
| `size_t`, `uint64_t`, `Uint64`, `uintptr_t` | `u64` |
| `short`, `int16_t` | `i16` |
| `unsigned short`, `uint16_t`, `Uint16` | `u16` |
| `signed char`, `int8_t` | `i8` |
| `unsigned char`, `uint8_t`, `Uint8` | `u8` |
| `bool`, `_Bool` | `bool` |
| `float` | `f32` |
| `double` | `float` (or `f64`) |

Plain `char` is signed on linux-x86_64 and on the Apple targets but unsigned on
linux-aarch64, so declare it `i8` or `u8` from the header's intent, not from
the keyword.

What elephc does at the call boundary:

- **Returns** are widened from the declared width before PHP sees them: `i32`
  sign-extends, `u32` zero-extends, `i16`/`u16` and `i8`/`u8` do the same for
  their widths, `int`/`i64`/`u64` pass through, and `bool` keeps only its low
  byte. A `u64` above `PHP_INT_MAX` reads as a negative `int`, exactly like
  reinterpreting the bits.
- **Arguments** of 32 or 64 bits are passed as they are; the callee reads only
  the bits it declared. Arguments of 8 or 16 bits are reduced modulo 2^width
  and extended to 32 bits by the caller, as C would convert them: `300` passed
  to a `u8` parameter arrives as `44`.
- Nothing is checked at runtime: like C, an out-of-range value truncates
  silently.

Declaring a C `int` function as `int` is a mistake the compiler cannot detect:
the call works whenever the callee happens to leave the upper bits clean and
reads garbage otherwise. Explicit widths are accepted on `extern function`
parameters and returns only; `extern class` fields, `extern global`
declarations and callback signatures still use `int`.

## Float widths

`float` is a C `double`. Declare a C `float` parameter or return as `f32`: the
PHP value is still a 64-bit `float`, and elephc rounds it to single precision
before the call (IEEE round-to-nearest, values outside the single range become
infinity) and widens a returned single back to a double. The C side only ever
reads the low 32 bits of its floating-point register, so a C `float` function
declared as `float` passes and reads garbage.

```php
<?php
extern function sqrtf(f32 $x): f32;
var_dump(sqrtf(2.0));   // float(1.4142135381698608): the single-precision root, widened
```

Like the integer widths, `f32` is accepted on `extern function` parameters and
returns only. To fill C structs that hold `float` fields (`SDL_FRect`,
`SDL_Vertex`), write them through a pointer with `ptr_write_f32()`; see
[Pointers](pointers.md).

`callable` in an `extern function` signature is a special FFI-only path. The
call site can provide either a string-literal elephc function name (resolved
case-insensitively like PHP function names) or a callable descriptor. Descriptor
callbacks are bound into generated C-ABI trampoline symbols, including values
with closure captures, object receivers, late-static-binding context, or
branch-selected descriptor state. The trampoline receives the C callback arguments, reloads the retained
descriptor from global storage, invokes the descriptor's runtime invoker, and
casts the boxed result back to the C-compatible callback return type.

FFI callback descriptors are limited to fixed scalar/pointer signatures:
`int`, `float`, `bool`, `ptr`, and `void` return values, with parameters drawn
from `int`, `float`, `bool`, and `ptr`. `string`, arrays, objects, variadics,
defaults, and by-reference callback parameters are rejected because ownership
and temporary lifetime across a C callback boundary are not modeled safely yet.
For C APIs without a userdata/context parameter, each generated trampoline owns
one mutable descriptor slot for that callsite; registering a new descriptor at
the same callsite replaces the previous slot owner.

## String conversion
- **Calling C**: elephc creates temporary null-terminated copy, frees after call
- **C returns string**: elephc scans for `\0`, copies to owned storage

Extern calls support the declared parameter names:

```php
<?php
extern function strcmp(string $left, string $right): i32;

$left = ["a"];
echo strcmp(...$left, right: "b");
```

Argument expressions are evaluated in PHP source order, then elephc loads the
resulting values into the target C ABI registers. This matters when positional,
named, or spread arguments have side effects.

Declared extern functions can also be selected by dynamic PHP string callbacks
such as `call_user_func($cb, ...)` and `call_user_func_array($cb, $args)`.
The runtime descriptor uses an extern invocation shape and a generated PHP-ABI
wrapper; the wrapper performs the C ABI call after PHP argument normalization.

## Callbacks
```php
<?php
extern function signal(i32 $sig, callable $handler): ptr;

function on_signal($sig) {
    echo "caught signal " . $sig . "\n";
}

signal(15, "on_signal");
```

Descriptor-backed callbacks can carry state:

```php
<?php
extern function signal(i32 $sig, callable $handler): ptr;

$delta = 3;
$handler = function (int $sig) use ($delta): void {
    echo $sig + $delta;
};

signal(15, $handler);
```
Callbacks must use C-compatible types only. No strings, arrays, variadic, defaults, or pass-by-reference.

Descriptor-backed callbacks are accepted when no environment is needed:

```php
<?php
extern function signal(i32 $sig, callable $handler): ptr;

function on_signal(int $sig): void {
    echo $sig;
}

$handler = on_signal(...);
signal(15, $handler);
```

## Extern globals
```php
<?php
extern global ptr $environ;
```
Uses GOT-relative addressing. String globals auto-converted.

## Extern classes (C structs)
```php
<?php
extern class Point {
    public int $x;
    public int $y;
}

$p = ptr_cast<Point>(malloc(16));
$p->x = 10;
echo $p->x;   // 10
```
Flat sequential layout, no class_id/vtable. 8-byte alignment.

## CLI linker flags

| Flag | Description |
|---|---|
| `--link LIB` / `-lLIB` | Link additional library |
| `--link-path DIR` / `-LDIR` | Add library search path |
| `--framework NAME` | Link macOS framework |

Libraries in `extern "lib" {}` blocks are linked automatically.

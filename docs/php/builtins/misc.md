---
title: "Misc builtins"
description: "Builtins in the Misc category."
sidebar:
  order: 118
---

## Misc builtins

| Function | Signature | Returns | AOT | eval() |
|---|---|---|:-:|:-:|
| [`constant()`](./misc/constant.md) | `(string $name): mixed` | `mixed` | ✓ | ✓ |
| [`define()`](./misc/define.md) | `(string $constant_name, mixed $value): bool` | `bool` | ✓ | ✓ |
| [`defined()`](./misc/defined.md) | `(string $constant_name): bool` | `bool` | ✓ | ✓ |
| [`empty()`](./misc/empty.md) | `(mixed $value): bool` | `bool` | ✓ | ✓ |
| [`error_log()`](./misc/error_log.md) | `(string $message, int $message_type = 0, string $destination = '', string $additional_headers = ''): bool` | `bool` | ✓ | — |
| [`extension_loaded()`](./misc/extension_loaded.md) | `(string $extension): bool` | `bool` | ✓ | ✓ |
| [`filter_var()`](./misc/filter_var.md) | `(mixed $value, int $filter = 516, mixed $options = 0): mixed` | `mixed` | ✓ | ✓ |
| [`get_loaded_extensions()`](./misc/get_loaded_extensions.md) | `(bool $zend_extensions = false): array` | `array` | ✓ | ✓ |
| [`header()`](./misc/header.md) | `(string $header, bool $replace = true, int $response_code = 0): void` | `void` | ✓ | ✓ |
| [`header_remove()`](./misc/header_remove.md) | `(string $name = null): void` | `void` | ✓ | — |
| [`headers_sent()`](./misc/headers_sent.md) | `(mixed $filename = null, mixed $line = null): bool` | `bool` | ✓ | — |
| [`http_response_code()`](./misc/http_response_code.md) | `(int $response_code = 0): int` | `int` | ✓ | ✓ |
| [`isset()`](./misc/isset.md) | `(mixed $var, ...$vars): bool` | `bool` | ✓ | ✓ |
| [`php_uname()`](./misc/php_uname.md) | `(string $mode = 'a'): string` | `string` | ✓ | ✓ |
| [`phpversion()`](./misc/phpversion.md) | `(string $extension = null): string|false` | `string|false` | ✓ | ✓ |
| [`preg_grep()`](./misc/preg_grep.md) | `(string $pattern, mixed $array, int $flags = 0): array` | `array` | ✓ | — |
| [`print_r()`](./misc/print_r.md) | `(mixed $value, bool $return = false): mixed` | `mixed` | ✓ | ✓ |
| [`register_tick_function()`](./misc/register_tick_function.md) | `(mixed $callback, ...$args): bool` | `bool` | — | ✓ |
| [`serialize()`](./misc/serialize.md) | `(mixed $value): string` | `string` | ✓ | — |
| [`setlocale()`](./misc/setlocale.md) | `(int $category, mixed $locales, ...$rest): mixed` | `mixed` | ✓ | — |
| [`unregister_tick_function()`](./misc/unregister_tick_function.md) | `(mixed $callback): void` | `void` | — | ✓ |
| [`unserialize()`](./misc/unserialize.md) | `(string $data, mixed $options = []): mixed` | `mixed` | ✓ | — |
| [`unset()`](./misc/unset.md) | `(mixed $var, ...$vars): void` | `void` | ✓ | ✓ |
| [`var_dump()`](./misc/var_dump.md) | `(mixed $value, ...$values): void` | `void` | ✓ | ✓ |

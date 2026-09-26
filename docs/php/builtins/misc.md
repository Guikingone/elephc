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
| [`error_log()`](./misc/error_log.md) | `(string $message, int $message_type = 0, string $destination = '', string $additional_headers = ''): bool` | `bool` | ✓ | ✓ |
| [`extension_loaded()`](./misc/extension_loaded.md) | `(string $extension): bool` | `bool` | ✓ | ✓ |
| [`filter_var()`](./misc/filter_var.md) | `(mixed $value, int $filter = 516, mixed $options = 0): mixed` | `mixed` | ✓ | ✓ |
| [`gc_collect_cycles()`](./misc/gc_collect_cycles.md) | `(): int` | `int` | ✓ | ✓ |
| [`gc_disable()`](./misc/gc_disable.md) | `(): void` | `void` | ✓ | ✓ |
| [`gc_enable()`](./misc/gc_enable.md) | `(): void` | `void` | ✓ | ✓ |
| [`gc_enabled()`](./misc/gc_enabled.md) | `(): bool` | `bool` | ✓ | ✓ |
| [`gc_mem_caches()`](./misc/gc_mem_caches.md) | `(): int` | `int` | ✓ | ✓ |
| [`get_cfg_var()`](./misc/get_cfg_var.md) | `(string $option): mixed` | `mixed` | ✓ | ✓ |
| [`get_loaded_extensions()`](./misc/get_loaded_extensions.md) | `(bool $zend_extensions = false): array` | `array` | ✓ | ✓ |
| [`header()`](./misc/header.md) | `(string $header, bool $replace = true, int $response_code = 0): void` | `void` | ✓ | ✓ |
| [`header_remove()`](./misc/header_remove.md) | `(string $name = null): void` | `void` | ✓ | — |
| [`headers_sent()`](./misc/headers_sent.md) | `(mixed $filename = null, mixed $line = null): bool` | `bool` | ✓ | ✓ |
| [`http_response_code()`](./misc/http_response_code.md) | `(int $response_code = 0): int` | `int` | ✓ | ✓ |
| [`ini_restore()`](./misc/ini_restore.md) | `(string $option): void` | `void` | ✓ | — |
| [`isset()`](./misc/isset.md) | `(mixed $var, ...$vars): bool` | `bool` | ✓ | ✓ |
| [`memory_get_peak_usage()`](./misc/memory_get_peak_usage.md) | `(bool $real_usage = false): int` | `int` | ✓ | ✓ |
| [`memory_get_usage()`](./misc/memory_get_usage.md) | `(bool $real_usage = false): int` | `int` | ✓ | ✓ |
| [`opcache_compile_file()`](./misc/opcache_compile_file.md) | `(mixed $filename): bool` | `bool` | ✓ | — |
| [`opcache_get_configuration()`](./misc/opcache_get_configuration.md) | `(): array` | `array` | ✓ | — |
| [`opcache_get_status()`](./misc/opcache_get_status.md) | `(mixed $include_scripts = true): mixed` | `mixed` | ✓ | — |
| [`opcache_invalidate()`](./misc/opcache_invalidate.md) | `(mixed $filename, mixed $force = false): bool` | `bool` | ✓ | — |
| [`opcache_is_script_cached()`](./misc/opcache_is_script_cached.md) | `(mixed $filename): bool` | `bool` | ✓ | — |
| [`opcache_is_script_cached_in_file_cache()`](./misc/opcache_is_script_cached_in_file_cache.md) | `(mixed $filename): bool` | `bool` | ✓ | — |
| [`opcache_jit_blacklist()`](./misc/opcache_jit_blacklist.md) | `(mixed $closure): void` | `void` | ✓ | — |
| [`opcache_reset()`](./misc/opcache_reset.md) | `(): bool` | `bool` | ✓ | — |
| [`pcntl_alarm()`](./misc/pcntl_alarm.md) | `(int $seconds): int` | `int` | ✓ | ✓ |
| [`pcntl_async_signals()`](./misc/pcntl_async_signals.md) | `(bool $enable = null): bool` | `bool` | ✓ | ✓ |
| [`pcntl_daemon()`](./misc/pcntl_daemon.md) | `(bool $no_chdir = false, bool $no_close = false): bool` | `bool` | ✓ | ✓ |
| [`pcntl_errno()`](./misc/pcntl_errno.md) | `(): int` | `int` | ✓ | ✓ |
| [`pcntl_exec()`](./misc/pcntl_exec.md) | `(string $path, mixed $args = [], mixed $env_vars = []): bool` | `bool` | ✓ | ✓ |
| [`pcntl_fork()`](./misc/pcntl_fork.md) | `(): int` | `int` | ✓ | ✓ |
| [`pcntl_get_last_error()`](./misc/pcntl_get_last_error.md) | `(): int` | `int` | ✓ | ✓ |
| [`pcntl_getcpu()`](./misc/pcntl_getcpu.md) | `(): int` | `int` | ✓ | ✓ |
| [`pcntl_getcpuaffinity()`](./misc/pcntl_getcpuaffinity.md) | `(int $process_id = null): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_getpriority()`](./misc/pcntl_getpriority.md) | `(int $process_id = null, int $mode = 0): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_getqos_class()`](./misc/pcntl_getqos_class.md) | `(): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_setcpuaffinity()`](./misc/pcntl_setcpuaffinity.md) | `(int $process_id = null, mixed $cpu_ids = []): bool` | `bool` | ✓ | ✓ |
| [`pcntl_setns()`](./misc/pcntl_setns.md) | `(int $process_id = null, int $nstype = 1073741824): bool` | `bool` | ✓ | ✓ |
| [`pcntl_setpriority()`](./misc/pcntl_setpriority.md) | `(int $priority, int $process_id = null, int $mode = 0): bool` | `bool` | ✓ | ✓ |
| [`pcntl_setqos_class()`](./misc/pcntl_setqos_class.md) | `(mixed $qos_class): void` | `void` | ✓ | ✓ |
| [`pcntl_signal()`](./misc/pcntl_signal.md) | `(int $signal, mixed $handler, bool $restart_syscalls = true): bool` | `bool` | ✓ | ✓ |
| [`pcntl_signal_dispatch()`](./misc/pcntl_signal_dispatch.md) | `(): bool` | `bool` | ✓ | ✓ |
| [`pcntl_signal_get_handler()`](./misc/pcntl_signal_get_handler.md) | `(int $signal): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_sigprocmask()`](./misc/pcntl_sigprocmask.md) | `(int $mode, mixed $signals, mixed $old_signals = []): bool` | `bool` | ✓ | ✓ |
| [`pcntl_sigtimedwait()`](./misc/pcntl_sigtimedwait.md) | `(mixed $signals, mixed $info = [], int $seconds = 0, int $nanoseconds = 0): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_sigwaitinfo()`](./misc/pcntl_sigwaitinfo.md) | `(mixed $signals, mixed $info = []): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_strerror()`](./misc/pcntl_strerror.md) | `(int $error_code): string` | `string` | ✓ | ✓ |
| [`pcntl_unshare()`](./misc/pcntl_unshare.md) | `(int $flags): bool` | `bool` | ✓ | ✓ |
| [`pcntl_wait()`](./misc/pcntl_wait.md) | `(mixed $status, int $flags = 0, mixed $resource_usage = []): int` | `int` | ✓ | ✓ |
| [`pcntl_waitid()`](./misc/pcntl_waitid.md) | `(int $idtype = 0, int $id = null, mixed $info = [], int $flags = 4, mixed $resource_usage = []): bool` | `bool` | ✓ | ✓ |
| [`pcntl_waitpid()`](./misc/pcntl_waitpid.md) | `(int $process_id, mixed $status, int $flags = 0, mixed $resource_usage = []): int` | `int` | ✓ | ✓ |
| [`pcntl_wexitstatus()`](./misc/pcntl_wexitstatus.md) | `(int $status): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_wifcontinued()`](./misc/pcntl_wifcontinued.md) | `(int $status): bool` | `bool` | ✓ | ✓ |
| [`pcntl_wifexited()`](./misc/pcntl_wifexited.md) | `(int $status): bool` | `bool` | ✓ | ✓ |
| [`pcntl_wifsignaled()`](./misc/pcntl_wifsignaled.md) | `(int $status): bool` | `bool` | ✓ | ✓ |
| [`pcntl_wifstopped()`](./misc/pcntl_wifstopped.md) | `(int $status): bool` | `bool` | ✓ | ✓ |
| [`pcntl_wstopsig()`](./misc/pcntl_wstopsig.md) | `(int $status): mixed` | `mixed` | ✓ | ✓ |
| [`pcntl_wtermsig()`](./misc/pcntl_wtermsig.md) | `(int $status): mixed` | `mixed` | ✓ | ✓ |
| [`php_sapi_name()`](./misc/php_sapi_name.md) | `(): string` | `string` | ✓ | — |
| [`php_uname()`](./misc/php_uname.md) | `(string $mode = 'a'): string` | `string` | ✓ | ✓ |
| [`phpversion()`](./misc/phpversion.md) | `(?string $extension = null): string|false` | `string|false` | ✓ | ✓ |
| [`posix_setpgid()`](./misc/posix_setpgid.md) | `(int $process_id, int $process_group_id): bool` | `bool` | ✓ | ✓ |
| [`posix_setsid()`](./misc/posix_setsid.md) | `(): int` | `int` | ✓ | ✓ |
| [`preg_grep()`](./misc/preg_grep.md) | `(string $pattern, mixed $array, int $flags = 0): array` | `array` | ✓ | — |
| [`print_r()`](./misc/print_r.md) | `(mixed $value, bool $return = false): mixed` | `mixed` | ✓ | ✓ |
| [`register_tick_function()`](./misc/register_tick_function.md) | `(mixed $callback, ...$args): bool` | `bool` | — | ✓ |
| [`serialize()`](./misc/serialize.md) | `(mixed $value): string` | `string` | ✓ | ✓ |
| [`set_time_limit()`](./misc/set_time_limit.md) | `(int $seconds): bool` | `bool` | ✓ | ✓ |
| [`setlocale()`](./misc/setlocale.md) | `(int $category, mixed $locales, ...$rest): mixed` | `mixed` | ✓ | ✓ |
| [`unregister_tick_function()`](./misc/unregister_tick_function.md) | `(mixed $callback): void` | `void` | — | ✓ |
| [`unserialize()`](./misc/unserialize.md) | `(string $data, mixed $options = []): mixed` | `mixed` | ✓ | ✓ |
| [`unset()`](./misc/unset.md) | `(mixed $var, ...$vars): void` | `void` | ✓ | ✓ |
| [`var_dump()`](./misc/var_dump.md) | `(mixed $value, ...$values): void` | `void` | ✓ | ✓ |
| [`zend_version()`](./misc/zend_version.md) | `(): string` | `string` | ✓ | — |

---
title: "Streams"
description: "Stream resources, wrappers, contexts, filters, sockets, TLS, and process pipes."
sidebar:
  order: 14
---

## Resource model

Streams are PHP `resource` values. File handles, standard streams, directory
streams, socket streams, process pipes, stream contexts, and stream filters all
use the resource runtime tag instead of plain integers.

`fopen()` returns `resource|false`: successful opens produce a stream resource,
while failed opens emit a suppressible runtime warning and return `false`.
Passing that `false` value to a stream builtin is a fatal runtime TypeError, so
code should guard failed opens before using the handle.

`STDIN`, `STDOUT`, and `STDERR` are stream resources. `gettype(STDIN)` returns
`"resource"`, `is_resource(STDIN)` returns `true`, and
`get_resource_type(STDIN)` returns `"stream"`.

### Resource ids

Ids follow PHP's reference numbering:

| id | holder |
|---|---|
| 1, 2, 3 | `STDIN`, `STDOUT`, `STDERR` |
| 4 | the request's default stream context |
| 5 and up | resources the program opens |

Id 4 is not reserved by the runtime at startup: PHP creates the default stream
context lazily, at the **first stream open of any kind**, and keeps it for the
rest of the request. `stream_context_get_default()` therefore reports a LOWER id
than a stream opened before it, and a program that never opens a stream never
mints it.

Ids are never reused. Closing a handle and opening another gives the new one the
next number, even when the operating system hands back the same descriptor.
`var_dump()`, `get_resource_id()`, an `(int)` cast, and `"$handle"` all report the
same id, and streams opened inside `eval()` draw from the same counter, so a mixed
program numbers its resources exactly as PHP does.

A `php://stdout` (or `php://stdin`, `php://stderr`, `php://fd/N`) handle is a
DUPLICATE of the descriptor, as in php-src, so `fclose()` on it closes only that
copy and the program's own output keeps working.

The table backing these ids costs no runtime heap: its first slots are reserved
statically, so a program compiled with a small `--heap-size` still starts, and one
that never opens a resource reports `allocs=0` under `--heap-debug`. Opening more
resources than the initial reservation holds grows the table on the heap as usual.

## Basic stream I/O

| Function | Signature | Description |
|---|---|---|
| `fopen()` | `fopen($filename, $mode, $use_include_path = false, $context = null): resource\|false` | Open a file, wrapper URL, socket-like wrapper, or temporary/memory stream. Modes `r`, `w`, `a`, `r+`, `w+`, and `a+` are supported. The optional args are evaluated in source order. `fopen()`, `file_get_contents()` and `readfile()` each publish their own `$context` for the duration of the call and restore the previous one afterwards. |
| `fclose()` | `fclose(resource $handle): bool` | Close a stream. Closing a `phar://` write stream finalizes the archive, and closing a filtered stream runs pending filter cleanup such as user-filter `onClose()`. |
| `fread()` | `fread(resource $handle, $length): string` | Read up to `$length` bytes. Attached read filters and user-wrapper `stream_read()` methods are honored. |
| `fwrite()` | `fwrite(resource $handle, $data): int` | Write bytes and return the byte count. Attached write filters and user-wrapper `stream_write()` methods are honored. |
| `fprintf()` | `fprintf(resource $handle, string $format, ...$values): int` | Format like `sprintf()` and write to the stream. |
| `vfprintf()` | `vfprintf(resource $handle, string $format, array $values): int` | Like `fprintf()`, with format values supplied as an array. |
| `fscanf()` | `fscanf(resource $handle, string $format): array` | Read one line and parse it with the `sscanf()` engine. v1 supports the two-argument array-returning form and conversions `%d`, `%f`, `%s`, and `%%`. |
| `fgets()` | `fgets(resource $handle, int $length = null): string\|false` | Read a line until newline, EOF, or `$length`; returns `false` at EOF. |
| `fgetc()` | `fgetc(resource $handle): string\|false` | Read one byte, or `false` at EOF/failure. |
| `feof()` | `feof(resource $handle): bool` | Report whether the stream is at EOF. |
| `fseek()` | `fseek(resource $handle, $offset [, $whence]): int` | Seek a stream. User wrappers route through `stream_seek()`. |
| `ftell()` | `ftell(resource $handle): int` | Return the current stream position. User wrappers route through `stream_tell()`. |
| `rewind()` | `rewind(resource $handle): bool` | Seek to the start of the stream. |
| `fgetcsv()` | `fgetcsv(resource $handle, ?int $length = null, string $separator = ',', string $enclosure = '"', string $escape = ''): array` | Read and parse one CSV line. Custom separator, enclosure, and escape (PHP 8.4 default `''` = RFC 4180 doubling mode) are honored. User wrappers are read through `stream_read()`. |
| `fputcsv()` | `fputcsv(resource $handle, array $fields, string $separator = ',', string $enclosure = '"', string $escape = '\\', string $eol = "\n"): int` | Format and write one CSV line with custom separator, enclosure, escape, and end-of-line (PHP 8.1+ `eol`). User wrappers are written through `stream_write()`. |
| `readline()` | `readline([$prompt]): string` | Read a line from standard input. |
| `readfile()` | `readfile($filename, bool $use_include_path = false, $context = null): int\|false` | Open a path or wrapper URL, stream it to stdout, and return copied bytes; returns `false` when open fails. Remote URLs are read through the same wrapper `file_get_contents()` uses, and `$context` is honored. |
| `fpassthru()` | `fpassthru(resource $handle): int` | Stream the remaining bytes of an open handle to stdout, returning `-1` on read failure. |
| `stream_get_contents()` | `stream_get_contents(resource $handle, ?int $length = null, int $offset = -1): string\|false` | Read remaining bytes from the stream. `$offset >= 0` seeks there first (seekable streams / user wrappers via `stream_seek()`) and returns `false` if that seek fails; a finite `$length` reads at most that many bytes (a `null`/negative `$length` reads to EOF). The bounded form loops through `fread` until it fills `$length`, reaches EOF, or receives an empty read. |
| `stream_copy_to_stream()` | `stream_copy_to_stream(resource $from, resource $to, ?int $length = null, int $offset = -1): int\|false` | Copy bytes from one stream to another, returning the count. `$offset >= 0` seeks the source first (seekable streams / user wrappers via `stream_seek()`) and returns `false` if that seek fails; a finite `$length` copies at most that many bytes (a `null`/negative `$length` copies to EOF). The bounded form drives a chunked read/write loop and clamps wrapper chunks that exceed the requested count. |
| `stream_get_line()` | `stream_get_line(resource $handle, int $length [, string $ending]): string` | Read up to `$length` bytes, stopping at and consuming `$ending` when supplied. |
| `flock()` | `flock(resource $handle, int $op, &$would_block = null): bool` | Advisory locking. `LOCK_SH`, `LOCK_EX`, `LOCK_UN`, and `LOCK_NB` are supported; user wrappers route through `stream_lock(int $operation)`. |
| `tmpfile()` | `tmpfile(): resource\|false` | Create an anonymous temporary stream backed by a `/tmp/elephc-XXXXXX` file that is immediately unlinked. |
| `fstat()` | `fstat(resource $handle): array\|false` | Return the same stat shape as `stat()`, but for an open stream. User wrappers route through `stream_stat()`. |
| `ftruncate()` | `ftruncate(resource $handle, $size): bool` | Truncate or extend a stream. User wrappers route through `stream_truncate(int $new_size)`. |
| `fflush()` | `fflush(resource $handle): bool` | Flush buffered output. elephc streams are unbuffered, so this maps to `fsync()`. |
| `fsync()` | `fsync(resource $handle): bool` | Flush data and metadata to durable storage. |
| `fdatasync()` | `fdatasync(resource $handle): bool` | Flush data only. On macOS, this falls back to `fsync()`. |

`stream_set_chunk_size($stream, $size)` returns the previous chunk size for the
stream. The first call reports the default `8192`; later calls report the value
set by the previous call. v1 tracks the value per fd but does not yet change read
granularity.

`stream_set_read_buffer()` and `stream_set_write_buffer()` return `0`. elephc
streams are unbuffered, so the accepted buffer size does not change behavior.

## Built-in wrappers

| Wrapper | Description |
|---|---|
| `file` | Normal filesystem streams. |
| `php://stdin`, `php://stdout`, `php://stderr` | Standard descriptors 0, 1, and 2. `php://input` aliases stdin, and `php://output` aliases stdout. |
| `php://memory`, `php://temp` | Seekable in-memory streams backed by an anonymous temporary buffer. `php://temp/maxmemory:N` is accepted and ignored. |
| `php://filter` | Opens an underlying resource and attaches one built-in filter at open time, for example `php://filter/read=string.toupper/resource=php://temp`. The resource may be any path or wrapper URL. ⚠️ The URL must be a literal in the `fopen()` call: it is parsed at compile time, so a string built at run time — the common shape, since the resource is usually a variable — falls through to the generic wrapper open and returns `false`. |
| `data://` | RFC 2397 inline payload streams. Base64 and percent-decoded payloads are supported. The URI must be a string literal. |
| `phar://` | Read or write PHAR entries. Literal reads happen at compile time and embed the entry in the binary; non-literal reads happen at runtime. Native PHAR, tar-based PHAR, and zip-based PHAR containers are readable; native PHAR gzip/bzip2 entries and ZIP deflate entries are decoded transparently. |
| `ftp://` | Anonymous binary passive FTP read streams. `fopen()` requires a literal URL; `file_get_contents()` also accepts runtime string URLs. Credentials in the URL are ignored in v1. |
| `ftps://` | Explicit FTP over TLS using `AUTH TLS`, with TLS on both control and data channels. `fopen()` requires a literal URL; `file_get_contents()` also accepts runtime string URLs. |
| `http://` | HTTP read streams. `fopen()` requires a literal URL; `file_get_contents()` also accepts runtime string URLs. Redirects are followed as PHP does — `follow_location` defaults to on and `max_redirects` to 20 — and a response buffers up to 1 MiB. |
| `https://` | Same as `http://`, but over TLS through the `elephc-tls` static library. Programs using it auto-link `-lelephc_tls`; programs that do not use TLS pay no extra link cost. |
| `compress.zlib://` | Read-only wrapper that opens the underlying file and applies `zlib.inflate`. |
| `compress.bzip2://` | Read-only wrapper that opens the underlying file and decompresses it through libbz2. |
| `glob://` | Directory-style wrapper for iterating paths matching a glob pattern through `opendir()` / `readdir()`. |

`phar://` write streams buffer one uncompressed entry in memory. `fclose()` and
`file_put_contents("phar://archive.phar/entry", $data)` insert or replace that
entry in a SHA1-signed PHAR archive while preserving existing entries.
`file_put_contents()` and write-mode `fopen()` also accept runtime-built
`phar://` URLs. Native PHAR, tar-based PHAR, and zip-based PHAR containers are
writable; ZIP writes preserve stored/deflated entries and compression controls
can rewrite ZIP entries between stored and deflated forms. ZIP entries written
with a streaming data descriptor are read transparently, and ZIP64 archives
(over 65535 entries, or sizes/offsets over 4 GiB) are both read and written.
Traditional-PKWARE (ZipCrypto) encrypted ZIP entries can be read and written
after calling the `setZipPassword(string $password)` compiler extension on a
`Phar`/`PharData` object: once a password is set, encrypted entries are decrypted
on read and zip entries are encrypted on write (the stub is encrypted too; the
`.phar/signature.bin` entry stays in the clear). ZipCrypto is cryptographically
weak — it is kept for compatibility with legacy archives, not as a real
confidentiality mechanism. `Phar` and `PharData` expose a
baseline OOP surface with constructors, format/compression/signature constants,
`addFromString()`, `delete()`, `compressFiles()`, `decompressFiles()`,
mixed metadata/string stub accessors, path helpers, and ArrayAccess read/write/isset
over the same `phar://` paths. ArrayAccess reads return `PharFileInfo` objects
with `getContent()` for payload reads and
`setMetadata()`/`getMetadata()`/`hasMetadata()`/`delMetadata()` for per-file
metadata. `foreach` over a `Phar` / `PharData`
object visits entries scanned from the archive at construction time plus entries
written through that object, yielding `entryName => PharFileInfo`.
`unlink("phar://archive/entry")` and `unset($phar["entry"])` remove entries
while preserving sibling entries. Native PHAR compression controls support
`Phar::GZ`, `Phar::BZ2`, and `Phar::NONE`; ZIP compression controls support
`Phar::GZ` and `Phar::NONE`.

`setMetadata()`/`getMetadata()`/`hasMetadata()`/`delMetadata()` and
`setStub()`/`getStub()` **persist into the archive file** for all three families,
so the global metadata and stub round-trip across fresh `Phar`/`PharData` objects
and across processes (and are interchangeable with the PHP interpreter). Metadata
is stored PHP-`serialize()`d — in the manifest metadata field for native PHAR, in a
`.phar/.metadata.bin` entry for tar, and in the ZIP archive comment for zip; the stub
is stored as the byte prefix for native PHAR and as a `.phar/stub.php` entry for
tar/zip. `setStub()` requires the stub to contain `__HALT_COMPILER();` (matching PHP).
The reserved `.phar/*` control entries are hidden from the entry listing and iteration.

Per-file metadata persists the same way through the `PharFileInfo` returned by
ArrayAccess: `$phar["entry"]->setMetadata(...)`, `getMetadata()`, `hasMetadata()`,
and `delMetadata()` round-trip across fresh objects and the PHP interpreter. It is
stored in the per-entry manifest field for native PHAR, in a
`.phar/.metadata/<entry>/.metadata.bin` side entry for tar, and in the per-entry ZIP
central-directory file comment for zip.

Whole-archive compression is supported on tar-based `PharData`: `compress(Phar::GZ)`
and `compress(Phar::BZ2)` write a sibling `.tar.gz` / `.tar.bz2` and return a fresh
`PharData` for it, while `decompress()` writes the plain `.tar` back; the compressed
archives are read transparently (and are interchangeable with the PHP interpreter).
Per-entry compression for native PHAR / zip stays on `compressFiles()` /
`decompressFiles()`.

Signatures are supported through `setSignatureAlgorithm()` / `getSignature()` across
native PHAR, tar, and zip phars. `setSignatureAlgorithm(Phar::MD5|Phar::SHA1|Phar::SHA256|Phar::SHA512)`
applies a hash signature, and `setSignatureAlgorithm(Phar::OPENSSL, $privateKey)` signs
with RSA-SHA1 using a PEM private key (PKCS#1 or PKCS#8). Native PHARs store the signature
in their trailer; tar and zip phars store it in a `.phar/signature.bin` control entry. The
resulting signature is verifiable by the PHP interpreter (for OpenSSL, place the matching
public key in `<archive>.pubkey`). `getSignature()` returns `['hash' => <uppercase hex>,
'hash_type' => 'MD5'|'SHA-1'|'SHA-256'|'SHA-512'|'OpenSSL']`.

Metadata persistence covers the same scalar+array subset as
[`serialize()`/`unserialize()`](system-and-io.md#serialization); object metadata is not
serialized.

`file_get_contents($url)` recognizes runtime `http://`, `https://`, `ftp://`,
and `ftps://` strings before falling back to `phar://`/filesystem handling.
Because the scheme is not known statically, non-literal `file_get_contents()`
conservatively links `elephc-tls`, `elephc-phar`, zlib, and libbz2.

`https://`, `ftps://`, and `stream_socket_enable_crypto()` use `elephc-tls`
(rustls, the `ring` crypto provider, and Mozilla webpki roots). TLS contexts can
override trust with `ssl.cafile` or `ssl.capath`, set `ssl.peer_name`, or relax
verification with `ssl.verify_peer = "0"`, `ssl.allow_self_signed`, or
`ssl.verify_peer_name = "0"`. Client certificates are supported when both
`ssl.local_cert` and `ssl.local_pk` point at readable PEM files; encrypted keys
and `ssl.passphrase` are not supported. `ssl.ciphers` and
`ssl.security_level` are accepted as context options for source compatibility
but are no-ops: rustls does not consume OpenSSL cipher-list strings and chooses
its TLS 1.2/1.3 policy internally.

## Stream contexts

| Function | Signature | Description |
|---|---|---|
| `stream_context_create()` | `stream_context_create(array $options = [], array $params = []): resource` | Create a stream-context resource and persist `$options` in the single global context slot. A literal `['notification' => <closure>]` in `$params` is captured for HTTP notification callbacks. |
| `stream_context_get_default()` | `stream_context_get_default(array $options = []): resource` | Return the default context resource. The optional arg is evaluated for side effects; v1 does not apply it. |
| `stream_context_set_default()` | `stream_context_set_default(array $options): resource` | Merge `$options` into the request's default context and return it. Later opens without an explicit context read them. |
| `stream_context_set_option()` | `stream_context_set_option(resource $context, ...): bool` | Accepts PHP's two forms: `(ctx, options_array)` replaces the persisted options hash, while `(ctx, wrapper, option, value)` sets one nested option. In the four-arg form, values are stored as strings in v1. |
| `stream_context_set_params()` | `stream_context_set_params(resource $context, array $params): bool` | Captures a literal `notification` closure or first-class callable into the global notification slot and returns `true`. |
| `stream_context_get_options()` | `stream_context_get_options(resource $context): array` | Return the persisted options hash, or an empty hash when no context has been created. |
| `stream_context_get_params()` | `stream_context_get_params(resource $context): array` | Return the context's `options` (and `notification` when one is set). |
| `stream_resolve_include_path()` | `stream_resolve_include_path(string $filename): string\|false` | elephc has no runtime `include_path`, so this is equivalent to `realpath($filename)`: canonical path on success, `false` otherwise. |

Active stream-context consumers:

- `fopen("http://...")` reads `http.method`, `http.header`, and `http.content`.
- `fopen("https://...")` reads the `ssl` trust and peer-name options.
- `fopen("ftp://...")` reads `ftp.resume_pos`.
- `file_get_contents()` over `https://` reads the same `ssl` options; over
  `ftp://` or `ftps://` it reads `ftp.resume_pos`.
- `stream_socket_server()` reads `socket.backlog`.
- `stream_socket_enable_crypto()` reads TLS peer and client-certificate options.
  `ssl.peer_name` becomes the SNI the handshake sends and the name the certificate
  is checked against; without it the connection host is used, and a host that is an
  IP address means no SNI is sent at all.

Contexts are independent values: creating or modifying one does not disturb another.
`fopen()`, `file_get_contents()` and `readfile()` publish their own `$context` for the
duration of the call and restore the previous one, so a context passed to one call
cannot leak into the next.

## Notification callbacks

HTTP streams can fire a context notification callback:

```php
$ctx = stream_context_create([], [
    'notification' => function (int $code, int $severity, ?string $message,
                                int $message_code, int $bytes_transferred,
                                int $bytes_max): void {
        if ($code === STREAM_NOTIFY_CONNECT)   { echo "connected\n"; }
        if ($code === STREAM_NOTIFY_COMPLETED) { echo "done\n"; }
        if ($code === STREAM_NOTIFY_FAILURE)   { echo "failed\n"; }
    },
]);

$body = fopen('http://example.com/', 'r');
```

`http://` fires `STREAM_NOTIFY_CONNECT`, `STREAM_NOTIFY_COMPLETED`, and
`STREAM_NOTIFY_FAILURE`. v1 captures only literal closure or first-class
callable entries. String function names, `[object, method]` arrays, variable
callbacks, HTTPS/FTP notifications, progress/file-size/mime/redirect/auth
milestones, `$message`, `$message_code`, and `$bytes_max` are deferred.

## Filters and buckets

| Function | Signature | Description |
|---|---|---|
| `stream_get_filters()` | `stream_get_filters(): array` | Return built-in filters: `string.toupper`, `string.tolower`, `string.rot13`, `string.strip_tags`, `convert.base64-encode`, `convert.base64-decode`, `convert.quoted-printable-encode`, `convert.quoted-printable-decode`, `convert.iconv.*`, `dechunk`, `zlib.deflate`, `zlib.inflate`, `bzip2.compress`, and `bzip2.decompress`, followed by every name `stream_filter_register()` added. PHP collapses its compression and conversion families into wildcard entries (`zlib.*`, `bzip2.*`, `convert.*`) where elephc lists each concrete name, so an `in_array()` availability check succeeds here on names PHP does not list. |
| `stream_filter_append()` | `stream_filter_append(resource $stream, string $filtername, int $read_write = STREAM_FILTER_ALL, mixed $params = null): resource\|false` | Attach a built-in or user-registered filter. `STREAM_FILTER_READ`, `STREAM_FILTER_WRITE`, and `STREAM_FILTER_ALL` select directions. |
| `stream_filter_prepend()` | `stream_filter_prepend(resource $stream, string $filtername, int $read_write = STREAM_FILTER_ALL, mixed $params = null): resource` | Same behavior as append in v1's one-filter-per-direction model. |
| `stream_filter_remove()` | `stream_filter_remove(resource $filter): bool` | Detach the filter from its chain. The filter is flushed first with `$closing = true`; if that flush answers `PSFS_ERR_FATAL`, the filter stays attached and the call returns `false`. |
| `stream_filter_register()` | `stream_filter_register(string $filter_name, string $class): bool` | Register a user filter class. Up to 128 registrations are stored; a literal class name is validated at compile time. |
| `stream_bucket_new()` | `stream_bucket_new(resource $stream, string $data): object` | Create a stdClass-backed bucket with public `data` and `datalen` properties. |
| `stream_bucket_make_writeable()` | `stream_bucket_make_writeable(resource $brigade): object\|null` | Pop the next bucket from a brigade. |
| `stream_bucket_append()` | `stream_bucket_append(resource $brigade, object $bucket): void` | Push a bucket to the end of a brigade. |
| `stream_bucket_prepend()` | `stream_bucket_prepend(resource $brigade, object $bucket): void` | Push a bucket to the beginning of a brigade. |

`zlib.deflate` and `gzcompress()` use system `libz`. `bzip2.compress`,
`bzip2.decompress`, and `compress.bzip2://` use `libbz2`. `convert.iconv.*`
uses libc `iconv` and auto-links `-liconv` on macOS.

Compression filter params can be a bare integer or a literal array. `zlib.deflate`
reads `level` (`-1..9`). `bzip2.compress` reads `blocks` (`1..9`) and `work`
(`0..250`). Non-literal params keep defaults.

User filters can implement either `filter(string $data): string` or PHP's
four-argument `filter($in, $out, &$consumed, $closing): int` bucket form.
Classes may extend PHP's `php_user_filter` base class; the fourth
`stream_filter_append`/`prepend` `$params` argument is available as
`$this->params` before `onCreate()` runs. The base class also declares
`$filtername` and `$stream`, so the manual's `stream_bucket_new($this->stream, …)`
idiom compiles; neither is seeded with a value yet. Optional `onCreate(): bool` and
`onClose(): void` hooks are honored; `onClose()` fires exactly once, whether the
filter is removed with `stream_filter_remove()` or carried off by `fclose()`.
`$closing` is `false` on read and write dispatches and `true` on the single
closing flush a removal performs. `PSFS_ERR_FATAL` cancels a removal but does not
otherwise propagate as a stream error.

⚠️ **`PSFS_FEED_ME` currently passes the raw input through** instead of withholding it.
A filter that buffers across dispatches — returning `PSFS_FEED_ME` until it has enough
bytes — therefore leaks its unfiltered input to the caller: reading `abcdefghi` in
three-byte chunks through such a filter yields `abc`, `ABCDEF`, `ghi` where PHP yields
`ABC`, `DEF`, `GHI`. Filters that answer `PSFS_PASS_ON` on every dispatch are
unaffected. The fix needs three things together: `PSFS_FEED_ME` returning nothing, the
read path fetching more input and dispatching again rather than reporting a short read,
and a filtered-read buffer on the stream so a filter that emits more than the caller
asked for keeps the remainder — plus a closing dispatch at end-of-stream so bytes still
held by the filter are flushed. Measured against php 8.5.6: a filter that triples `"ab"`
answers three `fread($f, 2)` calls with `ab`, `ab`, `ab`, and a filter still buffering
when the stream ends has its `$closing` output reach the reader.

## User stream wrappers

| Function | Signature | Description |
|---|---|---|
| `stream_get_wrappers()` | `stream_get_wrappers(): array` | Return built-in wrappers: `file`, `php`, `data`, `ftp`, `http`, `https`, `ftps`, `compress.zlib`, `compress.bzip2`, `phar`, and `glob`, followed by every scheme `stream_wrapper_register()` added. PHP also lists `zip`, which elephc does not implement. |
| `stream_wrapper_register()` | `stream_wrapper_register(string $protocol, string $class, int $flags = 0): bool` | Register a userspace wrapper class for `$protocol://` URLs. Up to 16 registrations are stored. |
| `stream_wrapper_unregister()` | `stream_wrapper_unregister(string $protocol): bool` | Remove a user-registered wrapper; built-in wrappers cannot be unregistered in v1. |
| `stream_wrapper_restore()` | `stream_wrapper_restore(string $protocol): bool` | Clears the disabled bit `stream_wrapper_unregister()` set on a built-in wrapper, and answers PHP's three cases. A wrapper that really was unregistered is restored silently and reports `true`. One that was never unregistered reports `true` with `Notice: stream_wrapper_restore(): <proto>:// was never changed, nothing to restore`. A scheme that never existed reports `false` with `Warning: stream_wrapper_restore(): <proto>:// never existed, nothing to restore`, which `@` suppresses. Following elephc's convention for every diagnostic, the Notice goes to stdout through the output-buffer funnel and the Warning to stderr; PHP CLI puts both on stdout, a repo-wide divergence rather than one specific to this builtin. |

When `fopen("$protocol://...")` matches a registered wrapper, elephc creates an
instance through the runtime class registry. Declared property defaults are
applied, but `__construct` is not invoked on this path.

Supported wrapper methods include `stream_open`, `stream_read`, `stream_write`,
`stream_close`, `stream_eof`, `stream_seek`, `stream_tell`, `stream_flush`,
`stream_stat`, `stream_lock`, `stream_truncate`, `stream_metadata`,
`stream_set_option`, `stream_cast`, `url_stat`, and the directory methods
`dir_opendir`, `dir_readdir`, `dir_rewinddir`, and `dir_closedir`.

Wrapper methods should declare return types that match their PHP contracts.
`stream_stat()` and `url_stat()` are exceptions: declare them without a return
type, or as `mixed`, when returning associative stat arrays with string keys.

## Sockets and process streams

| Function | Signature | Description |
|---|---|---|
| `stream_get_transports()` | `stream_get_transports(): array` | Return recognized socket transports: `tcp`, `udp`, `unix`, `udg`, `tls`, `ssl`, `sslv2`, `sslv3`, `tlsv1.0`, `tlsv1.1`, `tlsv1.2`, and `tlsv1.3`. TLS-version names all use rustls default negotiation. |
| `stream_socket_server()` | `stream_socket_server($address): resource\|false` | Bind a server socket for `[tcp://]host:port`, `udp://host:port`, `unix:///path`, or `udg:///path`. TCP and Unix-stream sockets listen; UDP and Unix-datagram sockets only bind. |
| `stream_socket_client()` | `stream_socket_client($address): resource\|false` | Open a client stream for `[tcp://]host:port`, `udp://host:port`, `unix:///path`, or `udg:///path`. ⚠️ `&$errno` and `&$errstr` must be declared before the call: a variable passed to a by-ref builtin parameter is treated as a use rather than a definition, so PHP's own idiom of passing them undeclared is a compile error. The same applies to every by-ref builtin parameter, `flock()`'s `&$would_block` included. |
| `stream_socket_accept()` | `stream_socket_accept($socket): resource\|false` | Accept the next pending connection from a listening stream. |
| `stream_socket_enable_crypto()` | `stream_socket_enable_crypto(resource $stream, bool $enable, int $crypto_method = null, resource $session_stream = null): bool` | Attach TLS to an already-connected TCP fd. `$enable=false` unwinds the session (sends `close_notify` and detaches it from the stream), leaving the fd a plain TCP socket, and reports `false` as PHP does — php-src performs the shutdown and still returns -1. On a handle that never had crypto it is a no-op and reports `true`. |
| `fsockopen()` | `fsockopen(string $hostname, int $port, int &$error_code = null, string &$error_message = null, float $timeout = null): resource\|false` | Open a TCP connection to `$hostname:$port`, writing optional by-reference error outputs. The timeout arg is evaluated but the OS default connect timeout is used in v1. |
| `pfsockopen()` | `pfsockopen(string $hostname, int $port, int &$error_code = null, string &$error_message = null, float $timeout = null): resource\|false` | Alias of `fsockopen()`; persistent connections are not meaningful for standalone native binaries. |
| `stream_set_blocking()` | `stream_set_blocking($stream, bool $enable): bool` | Toggle `O_NONBLOCK`. Non-blocking read misses return an empty `fread()` result or `false` from `fgetc()`/`fgets()` without setting EOF. User wrappers route through `stream_set_option(STREAM_OPTION_BLOCKING, ...)`. |
| `stream_set_timeout()` | `stream_set_timeout($stream, int $seconds, int $microseconds = 0): bool` | Set `SO_RCVTIMEO` on socket streams. User wrappers route through `stream_set_option(STREAM_OPTION_READ_TIMEOUT, ...)`. |
| `stream_select()` | `stream_select(array &$read, array &$write, array &$except, int $seconds, int $microseconds = 0): int` | Wait until stream arrays are ready and rewrite each array to its ready subset. Word-0 only: descriptors must be `0..63`. User wrappers are selectable when `stream_cast(STREAM_CAST_FOR_SELECT)` returns a real stream resource. |
| `stream_socket_shutdown()` | `stream_socket_shutdown($stream, int $mode): bool` | Shut down socket reads (`0`), writes (`1`), or both (`2`). |
| `stream_socket_sendto()` | `stream_socket_sendto($socket, string $data, int $flags = 0, string $address = ""): int\|false` | Send bytes to the connected peer or to an explicit datagram address. |
| `stream_socket_recvfrom()` | `stream_socket_recvfrom($socket, int $length, int $flags = 0, string &$address = ""): string\|false` | Receive bytes and optionally write back the sender address as `host:port`. |
| `stream_socket_get_name()` | `stream_socket_get_name($socket, bool $remote): string\|false` | Return local or remote socket name as `host:port`. |
| `stream_socket_pair()` | `stream_socket_pair(int $domain, int $type, int $protocol): array` | Create a pair of connected socket streams, for example `STREAM_PF_UNIX`, `STREAM_SOCK_STREAM`, `0`. |
| `popen()` | `popen(string $command, string $mode): resource\|false` | Open a pipe to a process in read (`"r"`) or write (`"w"`) mode. |
| `pclose()` | `pclose($handle): int` | Close a process pipe and return its termination status. |

Socket addresses use `[tcp://]host:port`, `udp://host:port`, `unix:///path`, or
`udg:///path`. Host names are resolved through the system resolver to IPv4.

## Directory streams

Directory handles are stream resources too. `opendir()`, `readdir()`,
`rewinddir()`, and `closedir()` are documented with filesystem functions in
[System & I/O](system-and-io.md). Registered userspace wrappers can implement
`dir_opendir`, `dir_readdir`, `dir_rewinddir`, and `dir_closedir`; the `glob://`
wrapper exposes glob matches through the same directory-stream API.

## Stream metadata and introspection

| Function | Signature | Description |
|---|---|---|
| `get_resource_type()` | `get_resource_type(resource $handle): string` | Return the resource's PHP type name — `"stream"`, `"stream-context"` or `"stream filter"` — and `"Unknown"` once the handle has been closed. |
| `get_resource_id()` | `get_resource_id(resource $handle): int` | Return the numeric id shown in `Resource id #N`. |
| `stream_isatty()` | `stream_isatty(resource $stream): bool` | Report whether the stream is connected to an interactive terminal. |
| `stream_is_local()` | `stream_is_local(resource\|string $stream): bool` | Return `true` for local streams. |
| `stream_supports_lock()` | `stream_supports_lock(resource $stream): bool` | Return `true` when a stream supports `flock()`. |
| `stream_get_meta_data()` | `stream_get_meta_data(resource $stream): array` | Return metadata keys `timed_out`, `blocked`, `eof`, `unread_bytes`, `stream_type`, `wrapper_type`, `mode`, `seekable`, and `uri`. |

Closing a handle changes its reported type but not its id. After `fclose()`,
`pclose()` or `closedir()`, `get_resource_type($handle)` returns `"Unknown"` and
`var_dump($handle)` prints `resource(N) of type (Unknown)` — matching PHP 8.5.6,
which renames every closed resource that way regardless of what it was. The id
`N` is unchanged, `get_resource_id()` still answers it, and `"$handle"` still
renders `Resource id #N`, because php-src leaves `zend_resource.handle` alone on
close. A resource reports its own type name — `"stream"`, `"stream-context"` or
`"stream filter"` — whether it is held directly or has travelled through an
untyped parameter, and `"Unknown"` once closed in either case.

`stream_get_meta_data()` derives `eof`, `seekable`, `blocked`, and `mode` from
the live descriptor. `wrapper_type` and `uri` are recorded per handle when the
stream is opened, so a file reports `plainfile` with its path, a `php://` stream
reports `PHP`, and a `data:` URL reports `RFC2397` — PHP's name for that wrapper,
which is the RFC rather than the scheme.

Known differences from PHP 8.5.6, all in reported names rather than behavior:

- `stream_type` is `"STDIO"` for seekable streams and `"tcp_socket"` for
  non-seekable ones. PHP distinguishes further: `MEMORY` for `php://memory`,
  `TEMP` for `php://temp`, `Output` for `php://output`, and `RFC2397` for a
  `data:` URL.
- `uri` is empty for a stream with no user-supplied path, such as `tmpfile()`,
  where PHP reports the temporary file it created.
- `php://` targets are resolved for LITERAL paths. `fopen($url, …)` with the URL
  in a variable goes through the ordinary filesystem open and fails.

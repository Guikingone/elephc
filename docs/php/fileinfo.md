---
title: "File information"
description: "PHP fileinfo functions, finfo objects, MIME types and character encodings."
sidebar:
  order: 28
---

PHP's fileinfo extension inspects file contents with libmagic. Elephc provides all
six functions, the `finfo` class and the eleven `FILEINFO_*` constants on every
supported target, including inside `eval()`.

## Setup

Declare the pinned native package once in the PHP project:

```bash
elephc native add libmagic
elephc main.php
```

Static references automatically link `elephc_fileinfo` and the managed libmagic
package. For opaque dynamic source use `--with-fileinfo`, which also retains the
PHP declarations. The compiled binary includes the complete libmagic 5.46 magic
database. Deployment needs no system libmagic or database file.

## Identify content

```php
$info = new finfo(FILEINFO_MIME_TYPE);
echo $info->buffer("Hello"), "\n"; // text/plain
echo $info->buffer("Hello", FILEINFO_MIME_ENCODING), "\n"; // us-ascii
echo $info->buffer("Hello", FILEINFO_MIME), "\n"; // text/plain; charset=us-ascii
echo $info->file(__FILE__), "\n";
```

`finfo_buffer()` and `finfo_file()` are the procedural equivalents. Both return a
string on success or `false` on a recoverable identification/read failure. Buffer
contents are binary-safe. `finfo_file()` accepts stream-wrapper URLs and a nullable
stream context. The context argument of `finfo_buffer()` has no effect, as in PHP.
Directories return `directory`.

`mime_content_type($filename)` accepts a filename or an open stream resource and
returns its MIME type. Stream identification seeks to the beginning and restores
the previous stream position.

## Configuration and ownership

`finfo_open(int $flags = FILEINFO_NONE, ?string $magic_database = null)` returns a
`finfo` object or `false`. `new finfo(...)` throws an exception when database
initialization fails. Null or empty database names select the embedded default;
custom text and compiled magic databases are accepted.

`$info->set_flags($flags)` and `finfo_set_flags($info, $flags)` change persistent
options. A nonzero flags argument on an individual query temporarily replaces
those options, including on failure. Passing `FILEINFO_NONE` to a query uses the
instance's persistent options.

Like PHP 8, `finfo_close($info)` returns true and leaves the object usable. The
object releases its native state automatically. Cloning and serialization are
refused. Content descriptions and the set of recognized file formats follow the
pinned upstream libmagic database, which can differ from the database bundled by
a particular PHP release.

---
title: "proc_open()"
description: "Execute a command and open file pointers for I/O."
sidebar:
  order: 307
---

## proc_open()

```php
function proc_open(array $descriptor_spec, string $command, array &$pipes): mixed
```

Executes a command and opens file pointers for I/O. The C1a implementation
ships a stub runtime that always returns `false`; the real fork/pipe/exec
implementation lands in a later slice.

**Parameters**:
- `$descriptor_spec` (`array`) — indexed array describing the child's pipes. Each
  entry is `[fd => ["pipe", "r"|"w"]]` (pipe-only in C1a).
- `$command` (`string`) — the command to execute.
- `&$pipes` (`array`) — by-reference array populated by the runtime with the
  parent-side pipe descriptors.

**Returns**: `mixed` — a process resource on success, or `false` on failure. The
C1a stub always returns `false`.

> **Note**: The `cwd`, `env`, and `options` parameters from PHP are not yet
> supported and will be added in a later slice.

```php
<?php
$pipes = [];
$r = proc_open([0 => ["pipe", "r"], 1 => ["pipe", "w"]], "echo hi", $pipes);
if ($r === false) {
    echo "failed to start process";
}
proc_close($r);
```
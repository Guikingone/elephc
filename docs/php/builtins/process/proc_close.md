---
title: "proc_close()"
description: "Close a process opened by proc_open and return the exit status."
sidebar:
  order: 308
---

## proc_close()

```php
function proc_close(mixed $process): int
```

Closes a process opened by `proc_open` and returns the child exit status. The
C1a implementation ships a stub runtime that always returns `-1`; the real
waitpid/reap implementation lands in a later slice.

**Parameters**:
- `$process` (`mixed`) — the `resource|false` value returned by `proc_open`.

**Returns**: `int` — the child process exit status, or `-1` on failure. The C1a
stub always returns `-1`.

```php
<?php
$pipes = [];
$r = proc_open([0 => ["pipe", "r"], 1 => ["pipe", "w"]], "echo hi", $pipes);
proc_close($r);
```
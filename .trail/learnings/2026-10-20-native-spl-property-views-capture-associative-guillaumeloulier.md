---
title: Native SPL property views capture associative keys without retaining PHP payloads
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
Native property retirement uses one raw graph memo shared by packed arrays and hashes. Nodes contain next, source identity, count and tag, followed by key/value snapshot pairs. Packed keys are absent; associative keys retain only inert strings. Capture hash values through the borrowed dereferencing iterator in insertion order. Reconstruction acquires Mixed values and transfers their ownership through hash_set_value. Invalidation and retirement must use the same entry layout.

Static and boxed clone must acquire logical native property values before retaining a byte-copied retiring Mixed cell. Reference slots keep their existing ABI.

## Tried and ruled out
Keeping the old array's Mixed box alive changes destructor timing. Retaining a zero-owner old box during clone reads freed storage. Reusing generic object_restore_property unchanged would reject PHP-accepted empty or unknown mangled keys and enforce ordinary property types.

## Watch out
M36 in the SPL/OPcache integration worktree passes 64 focused tests, 84 actual native executions and 56 iOS assembly cases. These prove only the private property primitive; unchanged heap PHPT remains 27/36 and full 1710-case compliance is false. Writable reference identity, cyclic acquired-graph controls and public native loading remain open.

Pinned PHP accepts integer keys as decimal property names and valid unknown mangled names dynamically. A plain protected name targets its declared slot; a plain private name becomes dynamic. Invalid NUL names emit a notice but are still inserted and later writes continue. Keep the dedicated native key protocol separate from ordinary checked restoration.

---
title: Heap clone materialization must use the target array allocation ABI
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
The receiver-bound SPL heap clone hook materializes a logical array before exposing the clone to PHP callbacks. Pass capacity through ABI integer argument 0 and element width 8 through argument 1, using abi::int_arg_reg_name for both. __rt_array_new uses x0/x1 on Apple/Linux ARM64 and rdi/rsi on Linux x86_64, not the general result register. Keep static and boxed clone paths in the native regression matrix, with both optimizer modes. A patched M23 assembly probe isolated this cause; the M25 source fix and boxed-receiver Rust regression replace that diagnostic patch.

## Tried and ruled out
The first manual cross-target harness omitted object_clone and regex runtime features. Clone-linked shared string dispatch referenced preg_replace/preg_replace_callback/preg_split and failed linking even though the public compiler pipeline passed. Dead stripping alone did not fix the omitted runtime families. Emit the actual required families and link the repository PCRE2 shim with the platform test provider; do not add placeholder runtime symbols or weaken the PHP fixture to hide missing dependencies. Once linking worked, both Linux x86_64 clone controls failed with heap memory exhausted because rsi still contained the source array pointer.

## Watch out
Filtered replay uses a Python regular expression, so (?i)(heap|priorityqueue|bug69264) selects 36 unchanged upstream cases. M25 passes 25; ten failures are missing heap serialization semantics, one is a by-reference compare representation rejection. These results do not close the 1710-test SPL/OPcache campaign. Native PHP 8.5.10 heap __unserialize copies property zvals without ordinary type checks: an int property can read back string:bad, while initialized readonly properties still reject the write and chain the Error beneath Invalid serialization data for H object. A checked generic hydration writer therefore cannot alone implement full native SPL loading parity. Preserve that checked writer's ordinary contracts and design the native authority explicitly.

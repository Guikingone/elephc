---
title: Element writes on null/false variables (php vivification)
author: Guillaume
date: 2026-10-28
branch: work/spl-opcache-main-integration
---

## Decided
- php converts a null/false variable to an array on its first element write (false: "Deprecated: Automatic conversion of false to array is deprecated"); true/int/float/string stay errors.
- Untyped params: `respecialized_param_types_for_call` widens a null/bool/false call-site type to Mixed when `undefined_reads::writes_elements_of(body, param)` — a Bool/Void-specialized slot cannot hold the array (it silently returned the scalar).
- Locals: `mixed_storage_scan` records element writes apart from unsafe disqualifications (`record_element_write`, `unsafe_to_box`, `inexact_assign`); a local assigned only literals incl. null/false AND element-written is marked boxed Mixed silently (`scalar_seeded_element_write`).
- Boxed locals: `static_false_parent::normalize_for_update` now runs before `lower_array_push`'s guard and inside `lower_boxed_array_set` (it was only on nested writes).
- Runtime `__rt_mixed_array_get_for_write`: integer key > length on packed storage promotes to hash (was null-filled: `$a[3][] = 1` gave `[null,null,null,[1]]`).

## Tried and ruled out
- Treating a Void/Bool env type as an empty array in `check_array_push`/`check_array_assign`: the frame slot type is fixed at the first store, lowering still failed ("runtime_call with receiver PHP type Void").

## Watch out
- Compound element updates (`$a[] .= $x`, `$a[] += 1`) bind a slot alias with RefAssign, which the scan treats as unsafe: null/false-seeded locals with those still fail (opcache jit assign_dim_op_001/004, inference_010).
- Functions are re-specialized per call site: `h(false); h(null)` exercise different param types than either call alone.

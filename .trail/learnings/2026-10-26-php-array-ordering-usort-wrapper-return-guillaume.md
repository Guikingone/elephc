---
title: "PHP array ordering, usort wrapper return, hydrated Mixed overwrite"
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 82, tokens: 217865 }
touches:
  - src/codegen_support/runtime/compare/array_loose_eq.rs
  - src/codegen_support/runtime/compare/php_compare.rs
  - src/types/checker/inference/ops.rs
  - src/ir_lower/function/builtin_wrappers.rs
  - src/codegen/eval_property_helpers/hydrate.rs
  - src/codegen_support/runtime/system/unserialize/property_cleanup.rs
---

## Decided
- Array ordering (<, >, <=, >=, <=>) follows zend_compare_arrays: smaller count first, then each left key in insertion order must exist on the right (absent key = 1, uncomparable), first non-zero element compare decides. Implemented by parameterizing the == walker (array_loose_eq.rs WalkMode::LooseEq|Order; LooseEq emission unchanged) as __rt_mixed_array_compare, reached from __rt_php_compare when both tags are 4/5 through two borrowed stack cells. __rt_php_truthy now treats an empty array as falsy (null <=> [] is 0). The checker accepts an array against an array/string/numeric/Mixed operand; IR and codegen route array operands to the runtime ordering path. SplMaxHeap::compare over arrays now orders like php.
- Main's usort(...) first-class wrapper body was 'usort(...);' with a declared bool return; this branch's run-time 'none returned' TypeError fired. The wrapper now returns the call.
- Hydrated Mixed properties share their box with the parser's value slots (the writer increfs the parsed box). A duplicate property key (php overwrites the zval in place, so R:N to the first value sees the replacement) now overwrites the existing box in place via __rt_unser_overwrite_box when the box refcount > 1 (shared with the parser); private default boxes take the ordinary store. The old value is deferred like __rt_unser_reuse_box; the borrowed parsed box is cloned (resource clones share the cell, so that case is declined).

## Watch out
- max()/min() with array arguments are still refused by the backend (bead elephc-4ml.7).
- __rt_mixed_clone returns the SAME retained cell for resources: never empty a clone shell without checking that.

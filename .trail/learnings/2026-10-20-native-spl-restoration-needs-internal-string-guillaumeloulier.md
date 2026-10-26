---
title: Native SPL restoration needs internal string ABI and runtime class scope
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
Use abi::string_result_regs for the internal str_persist helper: ARM64 uses x1/x2, x86_64 uses rax/rdx. A C-ABI name/value dispatcher still expands its Str operand into four C arguments; that does not change the ABI of string helpers it calls internally. Verify readonly Error construction by actual x86 execution, not only assembly.

Native SPL object_properties_load bypasses property types but keeps runtime scope lookup, readonly flags, and pending exception ordering. Preserve original key bytes for dynamic fields. The current private name dispatcher is partial until runtime lookup and these flags are implemented.

## Tried and ruled out
M38 passed 14 host controls and 80 target cases before readonly_initialized hung under x86_64: Error construction passed rdi/rsi to str_persist. M39 fixes the internal string pair; both optimizer modes now execute exactly with clean heaps. The old verifier was deliberately stopped, with its failure evidence retained. Native verification now bounds runs and cleans up only named own containers.

Simple exact mangled-key matching misses lowercase class scopes. Unconditionally matching every protected suffix is also wrong: an unknown scope resolves to NULL, while a registered unrelated scope creates a separate raw dynamic key.

## Watch out
Pinned PHP controls show autoload runs for unknown mangled scopes, class_alias names select the original private slot, and readonly clone reinitialization is allowed once. Throwing notice/deprecation handlers still allow the property loop to publish values and continue before the outer Invalid serialization wrapper. Dynamic destructor reentrant write yields callback|new, but reentrant unset yields absent|absent: do not blindly reinsert deleted buckets.

Evidence is in the SPL/OPcache integration worktree under native-property-edge-oracles-m39, native-dynamic-property-oracles-m39 and main-m39-native-property-error-abi. Host regressions pass 78/78; the expanded target matrix is still running when this note is written. Heap PHPT remains 27/36; public loading, references, dynamic retirement, runtime scope and full 1710-case compliance remain open.

---
title: Magic debug invokers preserve hidden arguments and reorder the receiver
author: Guillaumeloulier
date: 2026-10-19
branch: work/spl-opcache-main-integration
cost: { minutes: 2100, tokens: 12790252 }
touches:
  - ".plans/spl-opcache-php-src-1806-checkpoint.md"
  - ".plans/spl-opcache-php-src-1806-diagnostics.md"
  - ".plans/spl-opcache-php-src-1806.md"
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - crates/elephc-builtin-contract/src/lib.rs
  - crates/elephc-builtin-contract/src/method_returns.rs
  - crates/elephc-builtin-contract/src/registry.rs
  - crates/elephc-builtin-contract/src/support.rs
  - crates/elephc-magician/src/context.rs
  - crates/elephc-magician/src/context/classlike_objects.rs
---

## Decided
Native __debugInfo descriptors validate PHP-visible arity through types::call_args::regular_param_count and ignore the compiler-owned HIDDEN_ARGS_PARAM variadic collector. Preserve the complete physical signature: the ordinary callable invoker synthesizes an empty func_get_args collector and owns its cleanup. When hidden parameters exist, emit a target-aware tail adapter rotating the captured receiver from after those parameters into argument zero before entering the instance method. Use ABI register and branch helpers on both architectures.

## Tried and ruled out
Rejecting any nonempty signature.params blocks opaque eval, because SensitiveParameterValue::__debugInfo uses generated argument introspection. Merely accepting the hidden collector compiles but misorders the method ABI: a native direct control reported six null arguments instead of PHP's zero. Stripping the hidden signature would discard the required collector. Adding descriptor captures is unsafe because native debug views reserve a bounded single-receiver descriptor.

## Watch out
Keep receiver-first physical method entry distinct from capture-last callable invocation. Test inherited hooks reading a property as well as func_num_args and func_get_args; native M8 direct and inherited output matches PHP with a clean heap. All five supported target user assemblies pass. These checks are separate from opaque eval ownership replay, full linked target execution and the unchanged 1710-PHPT corpus score. No full compliance claim follows from this regression.

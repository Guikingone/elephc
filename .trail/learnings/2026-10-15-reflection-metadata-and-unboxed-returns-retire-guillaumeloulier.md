---
title: Reflection metadata and unboxed returns retire their producer owners
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-reflection-metadata-owners
touches:
  - src/codegen/lower_inst/objects/reflection/member_properties_emit.rs
  - src/codegen/lower_inst/objects/reflection/owner_emission.rs
  - src/codegen/lower_inst/objects/reflection/parameter_property_emit.rs
  - src/codegen/lower_inst/objects/reflection/string_attrs_emit.rs
  - src/ir_lower/stmt/return_coercions.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
---

## Decided
Fresh Reflection children and associative metadata adopt their producer reference through emit_box_current_owned_value_as_mixed. Fourteen fresh sites are covered; borrowed static values and lazy enum singletons keep borrowed boxing. ReflectionFunction name parts are initialized once. Mixed-to-concrete heap returns root an owning source cell until runtime conversion has acquired the payload, then retire the source through the existing unwind-aware call-operand owner protocol.

## Tried and ruled out
Changing Reflection boxing alone left getPrototype leaking 130 blocks/27416 bytes: EIR showed an owned property-read Mixed cell surviving conversion to the object return type. Constructing ReflectionMethod without getPrototype was clean. Initializing __short_name twice independently abandoned two strings. Do not fix borrowed enum singleton retention by transferring its global owner.

## Watch out
Eleven focused heap regressions pass with PHP cross-check. All eleven produce matching stdout and clean heaps on macOS ARM64, Linux ARM64 and Linux x86_64 (33 executions); runtime/user assembly passes for all five supported targets. iOS used the host SDK for assembly and still needs real SDK/native evidence. The broader 430-test Reflection batch is still running and must not be reported as complete. Separate intersection-type representation, static object assignment, and enum singleton shutdown gates remain open. The full SPL/OPcache campaign remains 639/1710 on its prior frozen compiler; these controls do not establish full compliance.

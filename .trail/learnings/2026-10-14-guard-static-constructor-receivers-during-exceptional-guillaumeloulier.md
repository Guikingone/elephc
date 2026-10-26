---
title: Guard static constructor receivers during exceptional unwinding
author: Guillaumeloulier
date: 2026-10-14
branch: fix/main-constructor-unwind-owners
touches:
  - src/codegen/lower_inst/objects/fixed_new.rs
  - src/codegen/lower_inst/callable_guards.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/constructor_owners.rs
---

## Decided
Static generic object construction publishes the fresh receiver through the existing begin_unconstructed_object guard after initializing defaults and before invoking __construct. Normal return removes the guard while retaining the result owner. Exceptional unwind suppresses the failed receiver's PHP destructor and releases its owned properties. Dynamic construction already uses this helper. Preserve the SPL branch's reserved-handle allocation order when adapting the same fix to its refactored generic emitter.

## Tried and ruled out
Clean main 62cfeb85ef leaked one 24-byte receiver per failed constructor. A failed receiver retaining a child leaked both objects. A focused property-allocation fixture also reproduced the two-object leak on frozen a7364c782a. Three focused regressions and nine native executions now match PHP and have clean heaps, covering repeated failure, fresh property ownership and successful construction. All five targets assemble.

The initial fresh-constructor-argument fixture revealed a separate observable gap: PHP retains that argument in the exception trace until the caught exception is released; native execution destroys it before catch. Preserve that source and its red output as an open trace-argument ownership gate. The receiver repair does not claim to fix Throwable trace retention. A check accidentally run before patch application is invalid evidence and is retained separately.

## Watch out
The main emitter and SPL emitter differ: the feature branch factors construction and reserves handles before nested argument evaluation. Integrate the guard in each actual body rather than copying the whole main file. Never invoke the failed receiver's destructor, and keep child-property destruction observable. Native error-handler activation records carry a self-pointer at offset 16; backtrace skips them before interpreting offset 24, so the diagnostic ownership flag is not a frame-reader pointer. Real iOS SDK/archive acceptance and full php-src compliance remain separate open gates.

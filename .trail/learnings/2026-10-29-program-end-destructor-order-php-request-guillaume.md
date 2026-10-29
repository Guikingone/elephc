---
title: Program-end destructor order (php_request_shutdown)
author: Guillaume
date: 2026-10-29
branch: fix/php-ext-teardown-order
---

## Decided
- The main epilogue follows php-src's php_request_shutdown when it is observable (a class with __destruct, a hosted extension, or eval): top-level variables holding an object's only reference are released newest first (emit_main_sole_object_release), then __rt_shutdown_destructors runs every remaining __destruct in object-handle order, then RSHUTDOWN, then static properties, static locals, the remaining locals and globals. Other programs keep the old order and pay no heap walk.
- __rt_shutdown_destructors walks by handle through a calloc'd handle->granule buffer filled from _obj_handle_index; it re-checks the slot still holds the handle before and after each call.

## Watch out
- __rt_call_object_destructor leaves bit 31 of the refcount set (the free path that normally follows never reads it). Calling it outside the free path requires clearing it, or the object can never be freed.
- A top-level object slot can hold a null sentinel (0x7ffffffffffffffe), not only 0: range-check against _heap_buf/_heap_off before reading a header.
- --web request teardown still releases globals/statics before RSHUTDOWN (documented).

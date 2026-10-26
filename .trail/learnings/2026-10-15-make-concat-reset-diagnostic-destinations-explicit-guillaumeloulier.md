---
title: Make concat-reset diagnostic destinations explicit
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - tests/codegen/optimizer/concat_reset.rs
---

## Decided
Concat-reset integration fixtures compare stdout with an explicit display_errors=stderr setting. Compare optimized and unoptimized stderr as well, and assert the expected float-key and undefined-key diagnostics are present. Preserve all scratch-state IR assertions and the original stdout values.

## Tried and ruled out
Two upstream tests failed their hardcoded stdout expectation after the PHP-compatible default diagnostic channel was integrated; optimized and unoptimized outputs were already identical. Pinned PHP 8.5.10 produces the same warnings on default stdout and the expected clean stdout with display_errors=stderr. This is a fixture destination assumption, not evidence of an optimizer mismatch or a reason to suppress warnings.

## Watch out
All seven concat-reset controls pass with both streams checked. The warning-producing operations must still execute: masking error_reporting or discarding stderr would hide the exact concat-scratch regression these tests protect. The production default channel and its existing PHP/native controls remain unchanged.

//! Purpose:
//! Regression tests for flow-sensitive narrowing between sequential `match (true)` arms.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Reaching a later single-condition arm proves every earlier single-condition guard false.
//! - These remain checker-level because subtype dispatch selected only by narrowing is not yet
//!   represented in EIR method-call metadata.

use super::*;

/// Verifies a negated property `instanceof` guard narrows the same property for the next
/// `match (true)` arm after the first arm failed.
#[test]
fn test_match_true_property_guard_complement_narrows_next_arm() {
    expect_ok(
        "<?php \
         interface GeneralContainer { public function has(string $id): bool; } \
         interface ParameterContainer extends GeneralContainer { \
             public function hasParameter(string $id): bool; \
         } \
         class MatchHolder { \
             public function __construct(private GeneralContainer $container) {} \
             public function contains(string $id): bool { \
                 return match (true) { \
                     !$this->container instanceof ParameterContainer => $this->container->has($id), \
                     $this->container->hasParameter($id) => true, \
                     default => false, \
                 }; \
             } \
         }",
    );
}

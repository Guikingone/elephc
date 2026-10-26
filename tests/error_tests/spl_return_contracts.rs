//! Purpose:
//! PHP declaration regressions for return covariance and ReturnTypeWillChange boundaries.
//!
//! Called from:
//! - `cargo test --test error_tests return_contract_tdd`.
//!
//! Key details:
//! - Source contracts remain strict even when an internal tentative-return attribute is present.
//! - Every union arm must be a subtype; a matching self arm cannot hide null or scalar arms.

use super::*;

/// The attribute cannot waive an explicit incompatible return on a source parent or interface.
#[test]
fn return_contract_tdd_marker_does_not_waive_source_types() {
    for source in [
        "<?php class P {function f(): int {return 1;}} class C extends P {#[ReturnTypeWillChange] function f(): string {return 'x';}}",
        "<?php class P {static function f(): int {return 1;}} class C extends P {#[ReturnTypeWillChange] static function f(): string {return 'x';}}",
        "<?php interface I {function f(): int;} class C implements I {#[ReturnTypeWillChange] function f(): string {return 'x';}}",
        "<?php interface I {static function f(): int;} class C implements I {#[ReturnTypeWillChange] static function f(): string {return 'x';}}",
        "<?php interface P {function f(): int;} interface I extends P {} class C implements I {#[ReturnTypeWillChange] function f(): string {return 'x';}}",
    ] {
        expect_error(source, "incompatible return type");
    }
}

/// Source return declarations cannot be omitted through ReturnTypeWillChange.
#[test]
fn return_contract_tdd_marker_does_not_remove_source_returns() {
    for source in [
        "<?php class P {function f(): int {return 1;}} class C extends P {#[ReturnTypeWillChange] function f() {return 1;}}",
        "<?php interface I {function f(): int;} class C implements I {#[ReturnTypeWillChange] function f() {return 1;}}",
        "<?php interface I {static function f(): int;} class C implements I {#[ReturnTypeWillChange] static function f() {return 1;}}",
    ] {
        expect_error(source, "without declaring a compatible return type");
    }
}

/// A self-return arm cannot make extra nullable, scalar, or unrelated-object arms covariant.
#[test]
fn return_contract_tdd_every_union_arm_must_conform() {
    for source in [
        "<?php interface I {function get(): I;} class C implements I {function get(): ?C {return null;}}",
        "<?php interface I {function get(): I;} class C implements I {function get(): C|string {return 'x';}}",
        "<?php interface I {function get(): I;} class D {} class C implements I {function get(): C|D {return new D();}}",
        "<?php interface I {static function get(): I;} class C implements I {static function get(): C|int {return 1;}}",
    ] {
        expect_error(source, "incompatible return type");
    }
}

/// Legal nullable and scalar unions stay valid when every actual arm has a required counterpart.
#[test]
fn return_contract_tdd_valid_union_covariance_survives() {
    for source in [
        "<?php interface I {function get(): ?I;} class C implements I {function get(): ?C {return null;}}",
        "<?php interface I {function get(): I|string;} class C implements I {function get(): C|string {return 'x';}}",
        "<?php class P {} interface I {function get(): P;} class C extends P implements I {function get(): C {return $this;}}",
    ] {
        check_source_full(source).expect("every union member is covariant");
    }
}

/// The attribute still suppresses tentative internal return constraints, including explicit types.
#[test]
fn return_contract_tdd_tentative_internal_returns_still_allow_marker() {
    for source in [
        "<?php class C extends ArrayObject {#[ReturnTypeWillChange] function count(): string {return 'x';}}",
        "<?php class C implements Countable {#[ReturnTypeWillChange] function count() {return 1;}}",
        "<?php class P {#[ReturnTypeWillChange] function count() {return 1;}} class C extends P implements Countable {}",
    ] {
        check_source_full(source).expect("valid PHP tentative-return suppression");
    }
}

/// The marker cannot waive mandatory internal SPL returns that PHP does not mark tentative.
#[test]
fn return_contract_tdd_hard_internal_spl_returns_stay_strict() {
    for source in [
        "<?php class C extends SplObjectStorage {#[ReturnTypeWillChange] function seek(int $offset): int {return 1;}}",
        "<?php class C extends SplFixedArray {#[ReturnTypeWillChange] function getIterator(): int {return 1;}}",
        "<?php class C extends SplFixedArray {#[ReturnTypeWillChange] function jsonSerialize(): int {return 1;}}",
    ] {
        expect_error(source, "incompatible return type");
    }
}

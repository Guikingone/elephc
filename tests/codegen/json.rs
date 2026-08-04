//! Purpose:
//! Provides JSON codegen test module wiring.
//! Exercises the JSON implementation through end-to-end PHP compilation and execution.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the JSON codegen test module.
//!
//! Key details:
//! - Submodules are grouped by JSON surface area so focused filters can run one feature slice.

use crate::support::*;

#[path = "json/constants.rs"]
mod constants;
#[path = "json/last_error.rs"]
mod last_error;
#[path = "json/last_error_msg.rs"]
mod last_error_msg;
#[path = "json/validate.rs"]
mod validate;
#[path = "json/extended_signatures.rs"]
mod extended_signatures;
#[path = "json/jsonserializable.rs"]
mod jsonserializable;
#[path = "json/encode_object.rs"]
mod encode_object;
#[path = "json/encode_jsonserializable.rs"]
mod encode_jsonserializable;
#[path = "json/exception.rs"]
mod exception;
#[path = "json/encode_flags.rs"]
mod encode_flags;
#[path = "json/encode_inf_nan.rs"]
mod encode_inf_nan;
#[path = "json/encode_float_precision.rs"]
mod encode_float_precision;
#[path = "json/encode_depth.rs"]
mod encode_depth;
#[path = "json/encode_invalid_utf8.rs"]
mod encode_invalid_utf8;
#[path = "json/encode_control_chars.rs"]
mod encode_control_chars;
#[path = "json/encode_list_shape.rs"]
mod encode_list_shape;
#[path = "json/decode_mixed.rs"]
mod decode_mixed;
#[path = "json/decode_stdclass.rs"]
mod decode_stdclass;
#[path = "json/mixed_index_access.rs"]
mod mixed_index_access;
#[path = "json/decode_errors.rs"]
mod decode_errors;
#[path = "json/decode_bigint.rs"]
mod decode_bigint;
#[path = "json/case_insensitive.rs"]
mod case_insensitive;
#[path = "json/evaluation_order.rs"]
mod evaluation_order;

/// `json_encode()`'s flags argument accepts a BOXED value, and the bitmask survives the boxing.
///
/// `$flags = $options['json_encoding'] ?? 0;` — a hash read merged with a scalar default — reaches
/// the builtin as `Mixed`, and PHP casts it to int at the boundary. Demanding a bare `Int` rejected
/// `Console\Descriptor\JsonDescriptor::write` on a program `php -n` runs. Asserting the PRETTY_PRINT
/// output rather than just "it compiles" is what pins the bitmask itself: a flags word that arrived
/// as garbage would still produce valid JSON, just not this shape.
#[test]
fn test_json_encode_accepts_boxed_flags_and_keeps_the_bitmask() {
    let out = compile_and_run(
        r#"<?php
function w(array $data, array $options = []): string {
    $flags = $options['json_encoding'] ?? 0;
    return json_encode($data, $flags);
}
echo w(['a' => 1], ['json_encoding' => \JSON_PRETTY_PRINT]);
"#,
    );
    assert_eq!(out, "{\n    \"a\": 1\n}");
}

/// The default arm of the same `??`, so the widened boundary is pinned on both sides: a boxed zero
/// must encode exactly as an unflagged call does.
#[test]
fn test_json_encode_boxed_zero_flags_encodes_unflagged() {
    let out = compile_and_run(
        r#"<?php
function w(array $data, array $options = []): string {
    $flags = $options['json_encoding'] ?? 0;
    return json_encode($data, $flags);
}
echo w(['a/b']);
"#,
    );
    assert_eq!(out, "[\"a\\/b\"]");
}

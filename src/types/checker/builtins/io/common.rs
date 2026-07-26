//! Purpose:
//! Type-checks PHP IO builtin common helpers and signatures.
//! Validates argument categories, resource handling, and return types before codegen sees calls.
//!
//! Called from:
//! - `crate::types::checker::builtins::io::common::ensure_stream_resource()` — used by
//!   check hooks in `src/builtins/io/` (fstat, stream_socket_shutdown, stream_socket_get_name,
//!   stream_socket_sendto, stream_socket_enable_crypto, stream_socket_accept,
//!   stream_socket_recvfrom, stream_filter_append, stream_filter_prepend, flock, and others).
//!
//! Key details:
//! - Return types and diagnostics must stay aligned with `crate::types::signatures` and builtin codegen emitters.

use crate::errors::CompileError;
use crate::parser::ast::Expr;
use crate::types::{PhpType, TypeEnv};

use super::super::super::Checker;

/// Validates that `arg` is a stream resource (or a type that accepts a stream resource).
///
/// Emits a type error if the argument is not a compatible stream type. Widened to
/// `pub(crate)` so `fstat`'s check hook in `src/builtins/io/fstat.rs` can call it;
/// `streams.rs` continues to use it via `super::common::ensure_stream_resource`.
pub(crate) fn ensure_stream_resource(
    checker: &mut Checker,
    name: &str,
    arg: &Expr,
    env: &TypeEnv,
) -> Result<(), CompileError> {
    let actual = checker.infer_type(arg, env)?;
    let expected = PhpType::stream_resource();
    if stream_arg_accepts(checker, &expected, &actual) {
        Ok(())
    } else {
        Err(CompileError::new(
            arg.span,
            &format!("{}() expects resource, got {}", name, actual),
        ))
    }
}

/// Checks whether `actual` can satisfy a stream resource expectation.
///
/// Returns true if `checker.type_accepts(expected, actual)` is true, if `actual` is `Mixed`,
/// or if `actual` is a `Union` containing at least one resource-accepting member while all
/// members are either resource-accepting, `Bool`/`False`, or `Void` (PHP null). Called only
/// by `ensure_stream_resource`.
///
/// The `Null`/`Bool` members are gradually accepted because a resource union such as
/// `fopen()`'s `resource|false` routinely widens to `resource<stream>|bool|null` once the
/// handle flows through an `if ($h = fopen(...))` guard whose sibling branch assigns
/// `$h = null` (Symfony's `KernelTrait::warmUp` is the canonical shape). PHP itself only
/// rejects a non-resource argument at runtime, and the stream builtins lower through
/// `emit_unbox_stream_or_type_error`, which unboxes a real resource from the Mixed cell and
/// otherwise raises PHP's exact `must be of type resource, <type> given` TypeError. Accepting
/// the union therefore defers to the same runtime check PHP performs, without ever passing a
/// non-resource fd to the syscall. A bare `Null` (not a union) still stays loud because it
/// can never carry a resource.
fn stream_arg_accepts(checker: &Checker, expected: &PhpType, actual: &PhpType) -> bool {
    if checker.type_accepts(expected, actual) || matches!(actual, PhpType::Mixed) {
        return true;
    }
    match actual {
        PhpType::Union(members) => {
            let has_resource = members
                .iter()
                .any(|member| checker.type_accepts(expected, member));
            let only_resource_bool_or_null = members
                .iter()
                .all(|member| {
                    checker.type_accepts(expected, member)
                        || matches!(member, PhpType::Bool | PhpType::False | PhpType::Void)
                });
            has_resource && only_resource_bool_or_null
        }
        _ => false,
    }
}

//! Purpose:
//! Defines the aggregate result returned by type checking to the pipeline.
//! Carries type environments, declarations, class metadata, warnings, FFI data, and required libraries forward.
//!
//! Called from:
//! - `crate::types::check()`
//! - `crate::pipeline::compile()`
//!
//! Key details:
//! - Fields are consumed by optimizer, codegen, and linker setup; keep additions explicit and phase-owned.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::{Platform, Target};
use crate::errors::{CompileError, CompileWarning};
use crate::parser::ast::Program;
use crate::span::Span;

use super::{
    checker, AttrArgEntry, ClassInfo, EnumInfo, ExternClassInfo, ExternFunctionSig, FunctionSig,
    InterfaceInfo, PackedClassInfo, PhpType, ReturnAliasSummaries, TypeEnv,
};

/// Fixed-point loop storage contracts keyed by `(function-like scope, loop span)`.
pub(crate) type LoopStorageTypes = HashMap<(String, Span), TypeEnv>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Describes a statically-known access violation that PHP raises as a catchable
/// `Error` at runtime instead of a compile-time rejection.
pub struct ThrowAccessInfo {
    /// Source span of the offending call/assignment, used as the lookup key.
    pub span: Span,
    /// The kind of violation, selecting the PHP error message template.
    pub kind: ThrowAccessKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Categorizes statically-decided access violations lowered to runtime `Error` throws.
pub enum ThrowAccessKind {
    /// A private/protected method call from an inaccessible scope.
    PrivateMethod {
        /// Visibility label (`private` or `protected`).
        visibility: String,
        /// Class holding the method (e.g. `C`).
        class_name: String,
        /// Method name without the trailing parentheses (e.g. `secret`).
        method: String,
    },
    /// A write to a readonly property outside its declaring constructor.
    ReadonlyProperty {
        /// Class holding the property (e.g. `Box`).
        class_name: String,
        /// Property name without the leading `$` (e.g. `x`).
        property: String,
    },
}

#[derive(Debug)]
/// Aggregate result of type checking, carrying type environments, declarations,
/// class metadata, warnings, FFI data, and required libraries forward to optimizer,
/// codegen, and linker setup.
pub struct CheckResult {
    pub global_env: TypeEnv,
    pub functions: HashMap<String, FunctionSig>,
    pub function_attribute_names: HashMap<String, Vec<String>>,
    pub function_attribute_args: HashMap<String, Vec<Option<Vec<AttrArgEntry>>>>,
    pub callable_param_sigs: HashMap<(String, String), FunctionSig>,
    /// Proven return-to-parameter storage aliases for source-declared callables.
    pub(crate) return_alias_summaries: ReturnAliasSummaries,
    #[allow(dead_code)]
    pub callable_return_sigs: HashMap<String, FunctionSig>,
    #[allow(dead_code)]
    pub callable_array_return_sigs: HashMap<String, FunctionSig>,
    pub interfaces: HashMap<String, InterfaceInfo>,
    pub classes: HashMap<String, ClassInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub packed_classes: HashMap<String, PackedClassInfo>,
    pub extern_functions: HashMap<String, ExternFunctionSig>,
    pub extern_classes: HashMap<String, ExternClassInfo>,
    pub extern_globals: HashMap<String, PhpType>,
    pub required_libraries: Vec<String>,
    pub warnings: Vec<CompileWarning>,
    /// Statically-decided access violations lowered to runtime `Error` throws,
    /// keyed by the source span of the offending call/assignment.
    pub throw_access_sites: HashMap<Span, ThrowAccessInfo>,
    /// Authoritative checker result types for builtin calls, keyed by call span.
    pub builtin_call_types: HashMap<Span, PhpType>,
    /// Fixed-point array-local storage contracts keyed by function-like scope and loop span.
    pub loop_storage_types: LoopStorageTypes,
    /// `(function-like scope, local name)` pairs for `string` locals that are a `++`/`--`
    /// target, so EIR lowering can give them boxed `Mixed` storage from their first store.
    pub string_incdec_locals: HashSet<(String, String)>,
    /// The `unset()` arguments whose local binding the checker killed, as span -> local NAME, so
    /// EIR lowering abandons the old frame slot (after releasing its value) instead of
    /// null-storing into it.
    ///
    /// The name is not decoration, it is half the key. A `Span` carries line/col and NOTHING
    /// about which file they are in, and include resolution splices every included file's AST
    /// into one program without rebasing line numbers, so line 4 column 1 of `main.php` and of
    /// `lib.php` are the SAME `Span`. Lowering must therefore check that the decision it found
    /// is about the variable in front of it; without that check a retype recorded in one file
    /// re-binds an unrelated same-position assignment in another (measured: a conditional
    /// `$w = …` in an included file lost its whole binding, printing `|s` where PHP prints
    /// `a1|s`). Same-name-same-position across two files stays ambiguous — closing that needs
    /// file identity in `Span` itself, which every span-keyed map here would want.
    #[allow(dead_code)]
    pub local_bind_kill_sites: HashMap<Span, String>,
    /// The statement-form assignments the checker re-bound to a fresh binding of an incompatible
    /// type, as span -> local NAME, so EIR lowering mints a new slot there. Read by
    /// `crate::ir_lower` alongside `local_bind_kill_sites`, and keyed the same way for the same
    /// reason — see that field.
    #[allow(dead_code)]
    pub local_retype_sites: HashMap<Span, String>,
}

/// Runs type checking using the host platform (auto-detected from the build environment).
/// Returns `Ok(CheckResult)` on success, or `Err(CompileError)` if type checking fails.
/// The `CheckResult` carries the resolved type environment, function signatures, class
/// metadata, warnings, and any required native libraries for linking.
/// Delegates to `check_with_options` with default `CheckOptions`.
#[allow(dead_code)]
pub fn check(program: &Program) -> Result<CheckResult, CompileError> {
    check_with_options(program, checker::CheckOptions::default())
}

/// Runs type checking using the host platform with explicit `CheckOptions`
/// (e.g. `--strict-locals`). Returns `Ok(CheckResult)` on success, or
/// `Err(CompileError)` if type checking fails.
#[allow(dead_code)]
pub fn check_with_options(
    program: &Program,
    options: checker::CheckOptions,
) -> Result<CheckResult, CompileError> {
    checker::check_types_with_options(program, Platform::detect_host(), options)
}

/// Runs type checking targeting a specific platform (e.g., Linux instead of the host macOS).
/// Returns `Ok(CheckResult)` on success, or `Err(CompileError)` if type checking fails.
/// The target affects FFI metadata, required library resolution, and platform-specific type behavior.
/// Delegates to `check_with_target_and_options` with default `CheckOptions`.
///
/// No longer called by the compile pipeline directly (it now threads `CheckOptions`
/// via `check_with_target_and_options`), so this is dead code outside of tests.
#[allow(dead_code)]
pub fn check_with_target(program: &Program, target: Target) -> Result<CheckResult, CompileError> {
    check_with_target_and_options(program, target, checker::CheckOptions::default())
}

/// Runs type checking targeting a specific platform with explicit `CheckOptions`
/// (e.g. `--strict-locals`). Returns `Ok(CheckResult)` on success, or
/// `Err(CompileError)` if type checking fails.
pub fn check_with_target_and_options(
    program: &Program,
    target: Target,
    options: checker::CheckOptions,
) -> Result<CheckResult, CompileError> {
    checker::check_types_with_options(program, target.platform, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{Arch, Target};

    /// Parses PHP source text into an AST for use in tests.
    fn parse_program(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        crate::parser::parse(&tokens).expect("parse failed")
    }

    /// Verifies that hashing builtins (here `md5`) require the pure-Rust
    /// `elephc_crypto` bridge on EVERY target after the Phase 2 migration off
    /// CommonCrypto/libcrypto — not the old linux-only system `crypto` library
    /// (which previously left macOS with no required library).
    #[test]
    fn test_hash_builtin_requires_elephc_crypto_on_all_targets() {
        let program = parse_program("<?php echo md5(\"abc\");");

        let linux = check_with_target(&program, Target::new(Platform::Linux, Arch::AArch64))
            .expect("linux type check failed");
        assert_eq!(linux.required_libraries, vec!["elephc_crypto"]);

        let mac = check_with_target(&program, Target::new(Platform::MacOS, Arch::AArch64))
            .expect("mac type check failed");
        assert_eq!(mac.required_libraries, vec!["elephc_crypto"]);
    }

    /// Verifies enum class metadata preserves flattened trait relation data for runtime reflection.
    #[test]
    fn test_enum_class_info_preserves_trait_metadata() {
        let program = parse_program(
            r#"<?php
trait EnumMetaTrait {
    public function original() {}
}
enum EnumMetaTarget {
    use EnumMetaTrait {
        original as aliasOriginal;
    }
    case Ready;
}
"#,
        );

        let result = check(&program).expect("type check failed");
        let enum_class = result
            .classes
            .get("EnumMetaTarget")
            .expect("missing enum class metadata");

        assert_eq!(enum_class.used_traits, vec!["EnumMetaTrait"]);
        assert_eq!(
            enum_class.trait_aliases,
            vec![(
                "aliasOriginal".to_string(),
                "EnumMetaTrait::original".to_string()
            )]
        );
    }
}

//! Purpose:
//! Records expected AOT/eval backend coverage and the reason for every intentional
//! absence in the shared builtin catalog.
//!
//! Called from:
//! - Compiler and Magician registry audits after joining implementation bindings.
//! - Cross-backend parity tests that must not maintain independent name allowlists.
//!
//! Key details:
//! - Implemented registry entries are proven by backend inventory, not duplicated here.
//! - Only non-registry routes and intentional unsupported surfaces need explicit data.

use crate::{
    eval_signature, runtime_builtin_id, Area, BuiltinContract, BuiltinId, BuiltinKind,
    RuntimeBuiltinId,
};

/// Backend whose support contract is being queried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinBackend {
    /// Static compiler checker/EIR/codegen path.
    Aot,
    /// Magician dynamic eval path.
    Eval,
}

/// How a supported contract reaches one backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendImplementation {
    /// Backend-owned inventory binding joined by `BuiltinId`.
    Registry,
    /// Parser/checker/lowering path for a PHP language construct.
    LanguageConstruct,
    /// Dedicated syntax node rather than an ordinary function call.
    DedicatedSyntax,
    /// Injected elephc-PHP prelude backed by internal compiler builtins.
    Prelude,
}

/// Why a shared catalog surface is deliberately absent from one backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedReason {
    /// Compiler-internal helper that is not part of Magician's PHP surface.
    InternalCompilerSurface,
    /// PHP-visible AOT implementation whose Magician implementation has not landed.
    EvalImplementationPending,
    /// Reflection behavior currently exists only for eval-declared/runtime objects.
    EvalOnlyReflection,
    /// PHP-visible Magician implementation whose AOT counterpart has not landed.
    ///
    /// The mirror of `EvalImplementationPending`. The interpreter can grow a surface the compiler
    /// has no machinery for at all -- `declare(ticks=N)` needs a per-statement hook the AOT
    /// backend does not emit -- and the registry gate demands that every `Function` contract have
    /// an AOT binding, so the absence has to be declared rather than left to panic.
    AotImplementationPending,
}

/// Expected support for one contract/backend pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendSupport {
    /// The backend must expose exactly one implementation through this route.
    Implemented(BackendImplementation),
    /// Absence is intentional and carries a machine-auditable reason.
    Unsupported(UnsupportedReason),
}

/// Why Magician must retain an interpreter-level adapter instead of using only
/// the versioned boxed-runtime dispatcher.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalAdapterReason {
    /// Caller-addressable storage, writeback, or lvalue evaluation is required.
    ByReferenceOrLvalue,
    /// Runtime callable resolution or class/member reflection is required.
    CallableOrReflection,
    /// Object conversion depends on declarations owned by the eval program.
    DynamicObjectCoercion,
    /// Language-construct, dedicated-syntax, or prelude behavior is eval-owned.
    DynamicLanguageSurface,
    /// Availability or behavior depends on an optional runtime capability.
    CapabilityDependent,
    /// Files, resources, process state, output state, or opaque handles are involved.
    RuntimeStateOrResource,
    /// The boxed-value algorithm remains an interpreter implementation rather than
    /// a generated-runtime helper with an equivalent ownership/error contract.
    InterpreterSpecificValueSemantics,
    /// A shared runtime helper covers only a strict subset of the PHP signature.
    AdditionalSignatureSemantics,
}

/// Expected Magician execution route after joining one implementation binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalExecution {
    /// All supported arities dispatch through the versioned boxed-runtime ABI.
    SharedRuntime(RuntimeBuiltinId),
    /// Magician retains a documented adapter, optionally falling back from a
    /// shared runtime helper for unsupported signature variants.
    Adapter {
        /// Shared runtime subset used before the adapter, when one exists.
        runtime_builtin: Option<RuntimeBuiltinId>,
        /// Dynamic/interpreter-specific reason the adapter remains.
        reason: EvalAdapterReason,
    },
}

impl EvalExecution {
    /// Returns the typed runtime subset used by this execution route.
    pub const fn runtime_builtin(self) -> Option<RuntimeBuiltinId> {
        match self {
            Self::SharedRuntime(runtime_builtin) => Some(runtime_builtin),
            Self::Adapter {
                runtime_builtin, ..
            } => runtime_builtin,
        }
    }
}

/// Returns the expected support route for one shared contract and backend.
pub fn backend_support(contract: &BuiltinContract, backend: BuiltinBackend) -> BackendSupport {
    match backend {
        BuiltinBackend::Aot => aot_support(contract),
        BuiltinBackend::Eval => eval_support(contract),
    }
}

/// Returns the expected compiler route for one shared contract.
pub fn aot_support(contract: &BuiltinContract) -> BackendSupport {
    if is_eval_only_reflection(contract.id) {
        return BackendSupport::Unsupported(UnsupportedReason::EvalOnlyReflection);
    }
    if is_aot_implementation_pending(contract.id) {
        return BackendSupport::Unsupported(UnsupportedReason::AotImplementationPending);
    }
    let implementation = match contract.kind {
        BuiltinKind::Function => BackendImplementation::Registry,
        BuiltinKind::LanguageConstruct => BackendImplementation::LanguageConstruct,
        BuiltinKind::DedicatedSyntax => BackendImplementation::DedicatedSyntax,
        BuiltinKind::PreludeProvided => BackendImplementation::Prelude,
    };
    BackendSupport::Implemented(implementation)
}

/// Returns the expected Magician route for one shared contract.
pub fn eval_support(contract: &BuiltinContract) -> BackendSupport {
    if contract.internal {
        return BackendSupport::Unsupported(UnsupportedReason::InternalCompilerSurface);
    }
    if EVAL_IMPLEMENTATION_PENDING
        .iter()
        .any(|name| contract.id == BuiltinId::from_canonical_name(name))
    {
        return BackendSupport::Unsupported(UnsupportedReason::EvalImplementationPending);
    }
    BackendSupport::Implemented(BackendImplementation::Registry)
}

/// Returns the documented execution route for an eval-supported contract.
pub fn eval_execution(contract: &BuiltinContract) -> Option<EvalExecution> {
    if !matches!(
        eval_support(contract),
        BackendSupport::Implemented(BackendImplementation::Registry)
    ) {
        return None;
    }

    if let Some(runtime_builtin) = runtime_builtin_id(contract.id) {
        return Some(if matches!(
            runtime_builtin,
            RuntimeBuiltinId::Intval | RuntimeBuiltinId::Round
        ) {
            EvalExecution::Adapter {
                runtime_builtin: Some(runtime_builtin),
                reason: EvalAdapterReason::AdditionalSignatureSemantics,
            }
        } else {
            EvalExecution::SharedRuntime(runtime_builtin)
        });
    }

    let reason = if contract.name == "strval" {
        EvalAdapterReason::DynamicObjectCoercion
    } else if eval_signature(contract)
        .params
        .iter()
        .any(|param| param.by_ref)
    {
        EvalAdapterReason::ByReferenceOrLvalue
    } else if matches!(contract.area, Area::Callables | Area::Spl) {
        EvalAdapterReason::CallableOrReflection
    } else if !matches!(contract.kind, BuiltinKind::Function) {
        EvalAdapterReason::DynamicLanguageSurface
    } else if !contract.requirements.is_empty() {
        EvalAdapterReason::CapabilityDependent
    } else if matches!(contract.area, Area::Io | Area::System | Area::Pointers) {
        EvalAdapterReason::RuntimeStateOrResource
    } else {
        EvalAdapterReason::InterpreterSpecificValueSemantics
    };
    Some(EvalExecution::Adapter {
        runtime_builtin: None,
        reason,
    })
}

/// Returns whether a function contract is intentionally available only in Magician.
fn is_aot_implementation_pending(id: BuiltinId) -> bool {
    AOT_IMPLEMENTATION_PENDING
        .iter()
        .any(|name| id == BuiltinId::from_canonical_name(name))
}

/// PHP-visible Magician contracts with no AOT implementation yet.
///
/// `declare(ticks=N)` runs a handler after every statement of the declared scope. The interpreter
/// walks statements and can do it; the compiled backend emits no per-statement hook, so these two
/// have nowhere to bind. Listed rather than silently registered, because a registry binding that
/// did nothing would report a handler installed and never call it.
///
/// `parse_str` joins them for a different reason: its `$result` parameter is by-reference with NO
/// compiled-side counterpart at all (no `RuntimeFnId`, no lowering, no prelude declaration --
/// confirmed by grep across `crates/` and `src/` before this contract was added). It already had
/// an interpreter implementation, dispatched from `eval_call`'s own hard-coded ladder rather than
/// the registry, before this contract made it visible to `function_exists()`/`is_callable()`.
const AOT_IMPLEMENTATION_PENDING: &[&str] =
    &["parse_str", "register_tick_function", "unregister_tick_function"];

fn is_eval_only_reflection(id: BuiltinId) -> bool {
    [
        "get_called_class",
        "get_class_methods",
        "get_class_vars",
    ]
    .into_iter()
    .any(|name| id == BuiltinId::from_canonical_name(name))
}

/// PHP-visible AOT contracts that do not yet have a Magician implementation binding.
const EVAL_IMPLEMENTATION_PENDING: &[&str] = &[
    "array_all",
    "array_any",
    "array_diff_assoc",
    "array_find",
    "array_intersect_assoc",
    "array_is_list",
    "array_key_first",
    "array_key_last",
    "array_merge_recursive",
    "array_multisort",
    "array_replace",
    "array_replace_recursive",
    "array_udiff",
    "array_uintersect",
    "array_walk_recursive",
    "bindec",
    "decbin",
    "dechex",
    "decoct",
    "error_log",
    "header_remove",
    "headers_sent",
    "hexdec",
    "join",
    "octdec",
    "preg_grep",
    "serialize",
    "setlocale",
    "strncasecmp",
    "strncmp",
    "unpack",
    "unserialize",
    "zval_free",
    "zval_pack",
    "zval_type",
    "zval_unpack",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contracts, lookup};

    /// Verifies every catalog entry has one explicit support result per backend.
    #[test]
    fn every_contract_has_a_backend_support_record() {
        let mut eval_registry = 0;
        let mut eval_internal = 0;
        let mut eval_pending = 0;
        let mut aot_registry = 0;
        let mut aot_external = 0;
        let mut aot_unsupported = 0;

        for contract in contracts() {
            match eval_support(contract) {
                BackendSupport::Implemented(BackendImplementation::Registry) => {
                    eval_registry += 1;
                }
                BackendSupport::Unsupported(UnsupportedReason::InternalCompilerSurface) => {
                    eval_internal += 1;
                }
                BackendSupport::Unsupported(UnsupportedReason::EvalImplementationPending) => {
                    eval_pending += 1;
                }
                other => panic!("unexpected eval support for {}: {other:?}", contract.name),
            }
            match aot_support(contract) {
                BackendSupport::Implemented(BackendImplementation::Registry) => {
                    aot_registry += 1;
                }
                BackendSupport::Implemented(_) => aot_external += 1,
                BackendSupport::Unsupported(UnsupportedReason::EvalOnlyReflection)
                | BackendSupport::Unsupported(UnsupportedReason::AotImplementationPending) => {
                    aot_unsupported += 1;
                }
                other => panic!("unexpected AOT support for {}: {other:?}", contract.name),
            }
        }

        // Recomputed against THIS tree, not typed in from a stale plan: the pinned numbers this
        // assertion inherited (`eval_registry: 491`, `eval_pending: 40`) were ALREADY WRONG on
        // this branch before the substr_count/get_debug_type commit touched anything --
        // reverting every file that commit touched and re-running this test measured 493 / 38,
        // not 491 / 40, so a prior commit on this branch drifted the catalog without updating
        // this census. The AOT-side numbers (`aot_registry: 555`, `aot_external: 10`,
        // `aot_unsupported: 5`) were still correct.
        // 496 (substr_count + get_debug_type + parse_str already landed) + levenshtein, a
        // brand-new `PreludeProvided` contract that is also NOT eval-pending.
        assert_eq!(eval_registry, 497);
        assert_eq!(eval_internal, 39);
        assert_eq!(eval_pending, 36);
        // Unchanged: `parse_str` raises `aot_unsupported` (it joined
        // `AOT_IMPLEMENTATION_PENDING`), and `levenshtein` raises `aot_external` (it is
        // `PreludeProvided`) -- neither is a `Registry` binding.
        assert_eq!(aot_registry, 555);
        // 10 + `levenshtein` (`PreludeProvided` -> `BackendImplementation::Prelude`).
        assert_eq!(aot_external, 11);
        // 5 + `parse_str`.
        assert_eq!(aot_unsupported, 6);
    }

    /// Verifies representative exceptional routes are attached to their contracts.
    #[test]
    fn exceptional_backend_routes_are_explicit() {
        assert_eq!(
            aot_support(lookup("hash_init").expect("hash_init contract")),
            BackendSupport::Implemented(BackendImplementation::Prelude)
        );
        assert_eq!(
            aot_support(lookup("get_object_vars").expect("get_object_vars contract")),
            BackendSupport::Implemented(BackendImplementation::Registry)
        );
        assert_eq!(
            eval_support(lookup("array_all").expect("array_all contract")),
            BackendSupport::Unsupported(UnsupportedReason::EvalImplementationPending)
        );
    }

    /// Verifies runtime-only, hybrid, and interpreter adapter routes cover all eval bindings.
    #[test]
    fn every_eval_binding_has_a_documented_execution_route() {
        let mut shared_runtime = 0;
        let mut hybrid_adapter = 0;
        let mut interpreter_adapter = 0;
        let mut unsupported = 0;

        for contract in contracts() {
            match eval_execution(contract) {
                Some(EvalExecution::SharedRuntime(_)) => shared_runtime += 1,
                Some(EvalExecution::Adapter {
                    runtime_builtin: Some(_),
                    reason: EvalAdapterReason::AdditionalSignatureSemantics,
                }) => hybrid_adapter += 1,
                Some(EvalExecution::Adapter {
                    runtime_builtin: None,
                    ..
                }) => interpreter_adapter += 1,
                Some(other) => panic!("invalid eval execution for {}: {other:?}", contract.name),
                None => unsupported += 1,
            }
        }

        // Recomputed against THIS tree; the pinned `interpreter_adapter: 470` / `unsupported: 79`
        // this assertion inherited were likewise already stale (see the census above).
        assert_eq!(shared_runtime, 19);
        assert_eq!(hybrid_adapter, 2);
        // 475 + levenshtein (brand new, `PreludeProvided`, no `RuntimeBuiltinId`).
        assert_eq!(interpreter_adapter, 476);
        // eval_internal (39) + eval_pending (36) above.
        assert_eq!(unsupported, 75);
        assert_eq!(
            eval_execution(lookup("strval").expect("strval contract")),
            Some(EvalExecution::Adapter {
                runtime_builtin: None,
                reason: EvalAdapterReason::DynamicObjectCoercion,
            })
        );
    }
}

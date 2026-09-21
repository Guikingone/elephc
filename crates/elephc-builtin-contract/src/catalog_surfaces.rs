//! Purpose:
//! Canonical contracts for PHP surfaces implemented outside the AOT `builtin!`
//! registry, including language constructs, dedicated syntax, preludes, and
//! currently eval-only reflection functions.
//!
//! Called from:
//! - `crate::registry` when assembling the complete shared contract catalog.
//!
//! Key details:
//! - These entries are ordinary shared contracts even though their AOT route is
//!   not a registry binding.
//! - Backend support is joined separately and must not be inferred from this file.

use crate::{
    Area, BuiltinContract, BuiltinId, BuiltinKind, DefaultSpec, ParamSpec, PhpModule, TypeSpec,
    VariadicSpec,
};

macro_rules! param {
    ($name:literal, $ty:ident) => {
        ParamSpec {
            name: $name,
            ty: TypeSpec::$ty,
            default: None,
            by_ref: false,
        }
    };
    ($name:literal, $ty:ident = $default:expr) => {
        ParamSpec {
            name: $name,
            ty: TypeSpec::$ty,
            default: Some($default),
            by_ref: false,
        }
    };
    (ref $name:literal, $ty:ident) => {
        ParamSpec {
            name: $name,
            ty: TypeSpec::$ty,
            default: None,
            by_ref: true,
        }
    };
}

macro_rules! surface {
    (
        $name:literal, $area:ident, $module:ident, $kind:ident,
        [$($param:expr),* $(,)?], $variadic:expr, $returns:ident,
        $summary:literal $(, extension: $extension:expr)?
    ) => {
        BuiltinContract {
            id: BuiltinId::from_canonical_name($name),
            name: $name,
            area: Area::$area,
            module: PhpModule::$module,
            since: None,
            kind: BuiltinKind::$kind,
            params: &[$($param),*],
            variadic: $variadic,
            min_args: None,
            max_args: None,
            arity_error: None,
            returns: TypeSpec::$returns,
            by_ref_return: false,
            summary: $summary,
            examples: &[],
            php_manual: None,
            deprecation: None,
            extension: surface!(@bool $($extension)?),
            internal: false,
            requirements: &[],
        }
    };
    (@bool $value:expr) => { $value };
    (@bool) => { false };
}

pub(crate) static SURFACE_CONTRACTS: &[BuiltinContract] = &[
    surface!(
        "buffer_new",
        Pointers,
        Elephc,
        DedicatedSyntax,
        [param!("length", Int)],
        None,
        Mixed,
        "Allocates a raw byte buffer.",
        extension: true
    ),
    surface!(
        "die",
        System,
        Core,
        LanguageConstruct,
        [param!("status", Int = DefaultSpec::Int(0))],
        None,
        Void,
        "Terminates execution with an optional status."
    ),
    surface!(
        "empty",
        Types,
        Core,
        LanguageConstruct,
        [param!("value", Mixed)],
        None,
        Bool,
        "Determines whether a variable is considered empty."
    ),
    surface!(
        "exit",
        System,
        Core,
        LanguageConstruct,
        [param!("status", Int = DefaultSpec::Int(0))],
        None,
        Void,
        "Terminates execution with an optional status."
    ),
    surface!(
        "get_called_class",
        Callables,
        Core,
        Function,
        [],
        None,
        Mixed,
        "Returns the late-static-binding class name in eval context."
    ),
    surface!(
        "get_class_methods",
        Callables,
        Core,
        Function,
        [param!("object_or_class", Mixed)],
        None,
        Mixed,
        "Returns method names visible on an object or class."
    ),
    surface!(
        "get_class_vars",
        Callables,
        Core,
        Function,
        [param!("class", Mixed)],
        None,
        Mixed,
        "Returns visible default properties for a class."
    ),
    surface!(
        "hash_copy",
        String,
        Hash,
        PreludeProvided,
        [param!("context", Mixed)],
        None,
        Mixed,
        "Clones an incremental hashing context."
    ),
    surface!(
        "hash_final",
        String,
        Hash,
        PreludeProvided,
        [
            param!("context", Mixed),
            param!("binary", Bool = DefaultSpec::Bool(false)),
        ],
        None,
        Mixed,
        "Finalizes an incremental hashing context."
    ),
    surface!(
        "hash_init",
        String,
        Hash,
        PreludeProvided,
        [
            param!("algo", Str),
            param!("flags", Int = DefaultSpec::Int(0)),
            param!("key", Str = DefaultSpec::Str("")),
        ],
        None,
        Mixed,
        "Opens an incremental hashing context."
    ),
    surface!(
        "hash_update",
        String,
        Hash,
        PreludeProvided,
        [param!("context", Mixed), param!("data", Str)],
        None,
        Mixed,
        "Feeds data into an incremental hashing context."
    ),
    // The `--web` request prelude's error/exception-handler stack and shutdown queue. All six
    // are PHP declarations over the request-local state helpers (`__elephc_error_handler_state`
    // and friends), so they have no `builtin!` binding. The prelude widens PHP's `?callable`
    // to `mixed` because a handler slot also holds `null` and a callable array; the contracts
    // record what the prelude declares, which is what prelude parity compares.
    surface!(
        "get_error_handler",
        Web,
        Core,
        PreludeProvided,
        [],
        None,
        Mixed,
        "Returns the current user-defined error handler."
    ),
    surface!(
        "register_shutdown_function",
        Web,
        Standard,
        PreludeProvided,
        [param!("callback", Callable)],
        Some(VariadicSpec::value("args")),
        Void,
        "Registers a callback to run when the script terminates."
    ),
    surface!(
        "restore_error_handler",
        Web,
        Core,
        PreludeProvided,
        [],
        None,
        Bool,
        "Restores the previous user-defined error handler."
    ),
    surface!(
        "restore_exception_handler",
        Web,
        Core,
        PreludeProvided,
        [],
        None,
        Bool,
        "Restores the previous user-defined exception handler."
    ),
    surface!(
        "set_error_handler",
        Web,
        Core,
        PreludeProvided,
        [
            param!("callback", Mixed),
            param!("error_levels", Int = DefaultSpec::Constant("E_ALL")),
        ],
        None,
        Mixed,
        "Installs a user-defined error handler and returns the previous one."
    ),
    surface!(
        "set_exception_handler",
        Web,
        Core,
        PreludeProvided,
        [param!("callback", Mixed)],
        None,
        Mixed,
        "Installs a user-defined exception handler and returns the previous one."
    ),
    surface!(
        "isset",
        Types,
        Core,
        LanguageConstruct,
        [param!("var", Mixed)],
        Some(VariadicSpec::value("vars")),
        Bool,
        "Determines whether variables are set and are not null."
    ),
    // `PreludeProvided`, not a `catalog_data.rs`/`catalog_data_additional.rs` `Function`
    // contract: the AOT side serves ONLY the plain two-argument call shape, as a conditionally
    // injected PHP-source helper (`__elephc_levenshtein_two_arg` /
    // `src/backend_gap_prelude.rs`), not a `builtin!` registry binding -- the same asymmetry
    // `hash_init` above models. `min_args`/`max_args` stay `None` (php's own arity errors are
    // "at least 2" / "at most 5", not a fixed count), matching the contract's five declared
    // parameters.
    surface!(
        "levenshtein",
        String,
        Standard,
        PreludeProvided,
        [
            param!("string1", Str),
            param!("string2", Str),
            param!("insertion_cost", Int = DefaultSpec::Int(1)),
            param!("replacement_cost", Int = DefaultSpec::Int(1)),
            param!("deletion_cost", Int = DefaultSpec::Int(1)),
        ],
        None,
        Int,
        "Computes the Levenshtein edit distance between two strings."
    ),
    // `PreludeProvided` for the same asymmetry as `levenshtein` above: the AOT side declares
    // `parse_str` as a conditionally injected prelude (`src/parse_str_prelude.rs`) rather than a
    // `builtin!` registry binding, because its mandatory by-reference `$result` is an ordinary
    // by-reference PARAMETER there and needs no runtime symbol. The INTERPRETER has its own
    // implementation (`interpreter::builtins::string::parse_str`), reached from `eval_call`'s
    // ladder, because interpreted code never runs through a compiler prelude.
    surface!(
        "parse_str",
        String,
        Standard,
        PreludeProvided,
        [param!("string", Str), param!(ref "result", Mixed)],
        None,
        Void,
        "Parses a query string into an array of variables."
    ),
    // `PreludeProvided` for the same asymmetry as `levenshtein` above: the AOT side serves
    // `var_export` as a conditionally injected PHP-source prelude (`src/var_export_prelude.rs`),
    // not a `builtin!` registry binding, so a `Function` contract would wrongly demand one. The
    // INTERPRETER has a real implementation
    // (`interpreter::builtins::core::var_export`) because interpreted code cannot reach that
    // prelude at all -- Symfony's routing dumper is interpreted and called it.
    surface!(
        "var_export",
        Io,
        Standard,
        PreludeProvided,
        [
            param!("value", Mixed),
            param!("return", Bool = DefaultSpec::Bool(false)),
        ],
        None,
        Mixed,
        "Renders a parsable string representation of a variable."
    ),
    surface!(
        "unset",
        Types,
        Core,
        LanguageConstruct,
        [param!("var", Mixed)],
        Some(VariadicSpec::value("vars")),
        Void,
        "Unsets the given variables."
    ),
];

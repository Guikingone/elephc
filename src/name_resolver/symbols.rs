//! Purpose:
//! Collects declared symbols needed for PHP namespace fallback and redeclaration-aware lookup.
//! Records functions, constants, and class-like declarations using case-folded lookup keys.
//!
//! Called from:
//! - `crate::name_resolver::resolve()` before rewriting references.
//!
//! Key details:
//! - Builtin class-like symbols are seeded so unresolved user names can still bind to PHP builtins.

use crate::names::{canonical_name_for_decl, php_symbol_key, Name, NameKind};
use crate::parser::ast::{ExprKind, Stmt, StmtKind};

use super::{canonical_builtin_function_name, namespace_name, Symbols};

/// Returns `true` if `name` refers to the builtin `define()` function, i.e. an
/// unqualified or fully-qualified single-segment name equal to `"define"`.
/// Inlined here (mirroring `src/resolver/state.rs::is_define_call_name`) because
/// the resolver helper is `pub(super)`-private and must not be widened.
fn is_define_call_name(name: &Name) -> bool {
    matches!(name.kind, NameKind::Unqualified | NameKind::FullyQualified)
        && name.parts.len() == 1
        && name.parts[0] == "define"
}

const BUILTIN_CLASS_LIKE_SYMBOLS: &[&str] = &[
    "ArrayAccess",
    "AppendIterator",
    "ArrayIterator",
    "ArrayObject",
    "CachingIterator",
    "CallbackFilterIterator",
    "Countable",
    "EmptyIterator",
    "Throwable",
    "Error",
    "Exception",
    "FilterIterator",
    "InfiniteIterator",
    "InternalIterator",
    "Iterator",
    "IteratorAggregate",
    "IteratorIterator",
    "LimitIterator",
    "MultipleIterator",
    "NoRewindIterator",
    "OuterIterator",
    "ParentIterator",
    "RecursiveArrayIterator",
    "RecursiveCallbackFilterIterator",
    "RecursiveFilterIterator",
    "RecursiveIterator",
    "RecursiveIteratorIterator",
    "SeekableIterator",
    "SortDirection",
    "SplDoublyLinkedList",
    "SplFixedArray",
    "SplObserver",
    "SplQueue",
    "SplStack",
    "SplSubject",
    "Stringable",
    "Traversable",
    "UnitEnum",
    "BackedEnum",
];

impl Symbols {
    /// canonical_function
    pub(super) fn canonical_function(&self, name: &str) -> Option<String> {
        let key = php_symbol_key(name);
        self.functions
            .get(&key)
            .or_else(|| self.extern_functions.get(&key))
            .cloned()
            .or_else(|| canonical_builtin_function_name(name))
            .or_else(|| super::canonical_prelude_global_function_name(name))
    }

    /// Returns whether `name` resolves to a user-declared (or extern) function,
    /// ignoring compiler builtins. Used to stop procedural date/time alias
    /// rewriting from hijacking a user function whose name collides with an
    /// alias (e.g. a namespaced `App\date_diff`).
    pub(super) fn declares_function(&self, name: &str) -> bool {
        let key = php_symbol_key(name);
        self.functions.contains_key(&key) || self.extern_functions.contains_key(&key)
    }

    /// canonical_class_like
    pub(super) fn canonical_class_like(&self, name: &str) -> Option<String> {
        let key = php_symbol_key(name);
        self.classes
            .get(&key)
            .or_else(|| self.interfaces.get(&key))
            .or_else(|| self.traits.get(&key))
            .or_else(|| self.extern_classes.get(&key))
            .cloned()
            .or_else(|| {
                BUILTIN_CLASS_LIKE_SYMBOLS
                    .iter()
                    .find(|builtin| php_symbol_key(builtin) == key)
                    .map(|builtin| (*builtin).to_string())
            })
    }

    /// Returns whether `name` resolves to a user-declared (or extern) class,
    /// interface, or trait, ignoring compiler builtins. Used to stop builtin
    /// static-method desugaring (e.g. `DateTimeZone::listIdentifiers`) from
    /// hijacking a user class that happens to share the name.
    pub(super) fn declares_class_like(&self, name: &str) -> bool {
        let key = php_symbol_key(name);
        self.classes.contains_key(&key)
            || self.interfaces.contains_key(&key)
            || self.traits.contains_key(&key)
            || self.extern_classes.contains_key(&key)
    }

    /// has_constant
    pub(super) fn has_constant(&self, name: &str) -> bool {
        self.constants.contains(name)
    }
}

/// collect_symbols
pub(super) fn collect_symbols(
    stmts: &[Stmt],
    current_namespace: Option<&str>,
    symbols: &mut Symbols,
) {
    let mut namespace = current_namespace.map(str::to_string);
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::NamespaceDecl { name } => {
                namespace = Some(namespace_name(name));
            }
            StmtKind::NamespaceBlock { name, body } => {
                let block_namespace = Some(namespace_name(name));
                collect_symbols(body, block_namespace.as_deref(), symbols);
            }
            StmtKind::FunctionDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.functions,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::FunctionVariantGroup { name, .. } => {
                insert_folded_symbol(&mut symbols.functions, name.clone());
            }
            StmtKind::ClassDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::PackedClassDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.classes,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::InterfaceDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.interfaces,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::TraitDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.traits,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::ExternFunctionDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.extern_functions,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::ExternClassDecl { name, .. } => {
                insert_folded_symbol(
                    &mut symbols.extern_classes,
                    canonical_name_for_decl(namespace.as_deref(), name),
                );
            }
            StmtKind::ConstDecl { name, .. } => {
                symbols
                    .constants
                    .insert(canonical_name_for_decl(namespace.as_deref(), name));
            }
            // Top-level `\define('LITERAL', ...)` calls create GLOBAL constants
            // (or FQN constants when the literal name contains `\`). Register the
            // name so the namespace global-fallback in `resolve_constant_name`
            // (names.rs step 5) can resolve unqualified references inside a
            // namespace to this global. The value's TYPE is registered separately
            // by the checker's `define` handler (`builtins/system.rs`); the
            // name_resolver only needs the name for fallback resolution.
            StmtKind::ExprStmt(expr) => {
                if let ExprKind::FunctionCall { name, args } = &expr.kind {
                    if args.len() >= 2 && is_define_call_name(name) {
                        if let ExprKind::StringLiteral(const_name) = &args[0].kind {
                            symbols
                                .constants
                                .insert(const_name.trim_start_matches('\\').to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Inserts folded symbol into the supplied builtin metadata registry.
fn insert_folded_symbol(symbols: &mut std::collections::HashMap<String, String>, name: String) {
    symbols.entry(php_symbol_key(&name)).or_insert(name);
}

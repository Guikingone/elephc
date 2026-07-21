//! Purpose:
//! Parses `[...]` and `array(...)` array literals, including by-reference entries
//! (`['k' => &$v]`, `[&$v]`). A literal containing at least one by-reference entry is
//! desugared at parse time into prelude statements on a hidden temporary; literals
//! without reference entries keep the exact pre-existing `ArrayLiteral(Assoc)` AST.
//!
//! Called from:
//! - `crate::parser::expr::prefix` for both literal forms.
//!
//! Key details:
//! - The desugar reuses the committed statement forms: `StmtKind::Assign` (empty-literal
//!   init), `StmtKind::ArrayAssign`/`StmtKind::ArrayPush` (plain entries), and
//!   `StmtKind::RefAssignToTarget` (reference entries), yielded through the
//!   `ExprKind::Assignment` prelude machinery. Source evaluation order is preserved.
//! - Positional entries in a ref-bearing literal become runtime appends so PHP's
//!   next-integer-key rule holds for mixed keyed/positional literals (`[&$a, 5 => &$b, &$c]`
//!   produces keys 0/5/6, cross-checked with `php -r`).
//! - Duplicate keys keep PHP's replace semantics: a plain KEYED entry after a reference
//!   entry emits an `unset($tmp[key]);` guard first so a colliding ref bucket is REPLACED
//!   (reference discarded, source untouched) instead of written through; non-literal guard
//!   keys are bound to a hidden temporary once so they are never evaluated twice.
//! - A reference-entry source must start with a variable-rooted token (`[&($v)]` is a parse
//!   error, as in PHP); `&f()` mirrors PHP's "Can't use function return value in write
//!   context" compile-time fatal.
//! - The plain (no-ref) construction replays `prefix.rs`'s exact key bookkeeping
//!   (`explicit_integer_array_key`-driven auto-key updates with saturating advance), so
//!   literals without reference entries are byte-identical to the pre-existing AST.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;

use super::parse_expr;
use super::prefix::{
    promote_indexed_array_items_to_assoc, update_next_auto_key_from_explicit_key,
};

/// One source-order entry of an array literal, recorded before deciding between the
/// plain `ArrayLiteral(Assoc)` construction and the by-reference desugar.
enum ArrayLiteralEntry {
    /// A positional element (`expr` or `&$lvalue` when `by_ref` is set).
    Positional { value: Expr, by_ref: bool },
    /// A keyed element (`key => expr` or `key => &$lvalue` when `by_ref` is set).
    Keyed { key: Expr, value: Expr, by_ref: bool },
    /// A `...$xs` spread element; `span` points at the `...` token for diagnostics.
    Spread { value: Expr, span: Span },
}

impl ArrayLiteralEntry {
    /// Returns whether this entry binds by reference (`&$lvalue` positional or keyed).
    fn is_by_ref(&self) -> bool {
        matches!(
            self,
            ArrayLiteralEntry::Positional { by_ref: true, .. }
                | ArrayLiteralEntry::Keyed { by_ref: true, .. }
        )
    }
}

/// Parses the comma-separated element list of an array literal up to its closing delimiter,
/// shared by the short `[...]` and long `array(...)` forms. Assumes the opening delimiter has
/// already been consumed and that `*pos` points at the first element (or the closing delimiter
/// for an empty literal). Handles positional elements, `key => value` keyed entries, `...`
/// spreads, and by-reference entries (`&$v`, `key => &$v`). `close` is the delimiter that
/// terminates the list (`]` or `)`) and `missing_close_msg` is the diagnostic emitted when it
/// is absent.
///
/// Entries are recorded in source order first; when no entry binds by reference the exact
/// pre-existing `ArrayLiteral`/`ArrayLiteralAssoc` construction is replayed (no regression
/// surface), while any by-reference entry routes the whole literal through the hidden-temp
/// prelude desugar so reference binds reuse the committed `RefAssignToTarget` machinery.
pub(super) fn parse_array_entries(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    close: &Token,
    missing_close_msg: &str,
) -> Result<Expr, CompileError> {
    let mut entries = Vec::new();
    let mut first = true;
    while *pos < tokens.len() && &tokens[*pos].0 != close {
        if !first {
            if tokens[*pos].0 != Token::Comma {
                return Err(CompileError::new(
                    tokens[*pos].1.span,
                    "Expected ',' between array elements",
                ));
            }
            *pos += 1;
            if *pos < tokens.len() && &tokens[*pos].0 == close {
                break;
            }
        }
        first = false;
        if *pos < tokens.len() && tokens[*pos].0 == Token::Ellipsis {
            let spread_span = tokens[*pos].1.span;
            *pos += 1;
            let inner = parse_expr(tokens, pos)?;
            entries.push(ArrayLiteralEntry::Spread {
                value: inner,
                span: spread_span,
            });
            continue;
        }
        // A leading `&` starts a by-reference positional entry (`[&$v]`): PHP's grammar only
        // allows `&` before a variable-rooted lvalue in this position, never as a unary operator.
        if *pos < tokens.len() && tokens[*pos].0 == Token::Ampersand {
            let source = parse_ref_entry_source(tokens, pos)?;
            entries.push(ArrayLiteralEntry::Positional {
                value: source,
                by_ref: true,
            });
            continue;
        }
        let expr = parse_expr(tokens, pos)?;
        if *pos < tokens.len() && tokens[*pos].0 == Token::DoubleArrow {
            *pos += 1;
            // `key => &$v`: a by-reference keyed entry.
            if *pos < tokens.len() && tokens[*pos].0 == Token::Ampersand {
                let source = parse_ref_entry_source(tokens, pos)?;
                entries.push(ArrayLiteralEntry::Keyed {
                    key: expr,
                    value: source,
                    by_ref: true,
                });
            } else {
                let value = parse_expr(tokens, pos)?;
                entries.push(ArrayLiteralEntry::Keyed {
                    key: expr,
                    value,
                    by_ref: false,
                });
            }
        } else {
            entries.push(ArrayLiteralEntry::Positional {
                value: expr,
                by_ref: false,
            });
        }
    }
    if *pos >= tokens.len() || &tokens[*pos].0 != close {
        return Err(CompileError::new(span, missing_close_msg));
    }
    *pos += 1;
    if entries.iter().any(ArrayLiteralEntry::is_by_ref) {
        build_ref_entry_literal_desugar(entries, span)
    } else {
        Ok(build_plain_array_literal(entries, span))
    }
}

/// Parses the source lvalue of a by-reference array-literal entry after `&` has been seen.
///
/// Consumes the `&`, parses one expression, and validates that it is a variable-rooted
/// lvalue shape PHP accepts in this position (`$v`, `$a[$k]`, `$o->p`, `$o->{$p}`,
/// `C::$p`, `C::${$p}`). PHP's grammar only accepts a variable-rooted token immediately
/// after `&` here — a parenthesized source (`[&($v)]`) is a PARSE error in PHP, so the
/// first token is checked before any expression parsing. Call results mirror PHP's
/// "Can't use function return value in write context" compile-time fatal; any other shape
/// is rejected with a clear message. Which lvalue families actually SUPPORT reference
/// binding is enforced later by the checker's `RefAssignToTarget` validation, so
/// unsupported-but-well-formed sources keep their committed loud errors instead of
/// silently value-copying.
fn parse_ref_entry_source(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Expr, CompileError> {
    let amp_span = tokens[*pos].1.span;
    *pos += 1; // consume '&'
    // PHP parse-rejects a non-variable-rooted token after `&` in this position (e.g.
    // `[&($v)]`, `[&1]`): only `$var`, `Name::$prop`, `self/static/parent::$prop`, and
    // `\Qualified\Name::$prop` starts are grammatically valid reference sources.
    let starts_variable_rooted = matches!(
        tokens.get(*pos).map(|(token, _)| token),
        Some(
            Token::Variable(_)
                | Token::Identifier(_)
                | Token::Static
                | Token::Self_
                | Token::Parent
                | Token::Backslash
        )
    );
    if !starts_variable_rooted {
        return Err(CompileError::new(
            amp_span,
            "By-reference array entry source must be a variable, array element, or property (e.g. &$x)",
        ));
    }
    let source = parse_expr(tokens, pos)?;
    match &source.kind {
        ExprKind::Variable(_)
        | ExprKind::ArrayAccess { .. }
        | ExprKind::PropertyAccess { .. }
        | ExprKind::DynamicPropertyAccess { .. }
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::DynamicStaticPropertyAccess { .. } => Ok(source),
        ExprKind::FunctionCall { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::StaticMethodCall { .. }
        | ExprKind::ClosureCall { .. }
        | ExprKind::ExprCall { .. } => Err(CompileError::new(
            amp_span,
            "Can't use function return value in write context",
        )),
        _ => Err(CompileError::new(
            amp_span,
            "By-reference array entry source must be a variable, array element, or property (e.g. &$x)",
        )),
    }
}

/// Builds the plain (no by-reference entry) array-literal AST from the source-order entries,
/// replaying the exact `prefix.rs` construction: positional elements accumulate in an indexed
/// list until the first keyed entry promotes them to integer-keyed pairs, later positional
/// elements receive the statically tracked next automatic integer key, spreads survive only
/// in the indexed form, and explicit integer keys advance the auto-key cursor through
/// `update_next_auto_key_from_explicit_key`. Returns `ArrayLiteralAssoc` when any keyed
/// entry was seen, otherwise `ArrayLiteral`.
fn build_plain_array_literal(entries: Vec<ArrayLiteralEntry>, span: Span) -> Expr {
    let mut elems = Vec::new();
    let mut assoc_elems = Vec::new();
    let mut is_assoc = false;
    let mut next_auto_key = 0i64;
    let mut auto_key_initialized = false;
    for entry in entries {
        match entry {
            ArrayLiteralEntry::Spread {
                value,
                span: spread_span,
            } => {
                if !is_assoc {
                    elems.push(Expr::new(ExprKind::Spread(Box::new(value)), spread_span));
                }
            }
            ArrayLiteralEntry::Keyed { key, value, .. } => {
                if !is_assoc {
                    promote_indexed_array_items_to_assoc(&mut elems, &mut assoc_elems);
                }
                is_assoc = true;
                update_next_auto_key_from_explicit_key(
                    &key,
                    &mut next_auto_key,
                    &mut auto_key_initialized,
                );
                assoc_elems.push((key, value));
            }
            ArrayLiteralEntry::Positional { value, .. } => {
                if is_assoc {
                    let key = Expr::new(ExprKind::IntLiteral(next_auto_key), value.span);
                    assoc_elems.push((key, value));
                } else {
                    elems.push(value);
                }
                next_auto_key += 1;
                auto_key_initialized = true;
            }
        }
    }
    if is_assoc {
        Expr::new(ExprKind::ArrayLiteralAssoc(assoc_elems), span)
    } else {
        Expr::new(ExprKind::ArrayLiteral(elems), span)
    }
}

/// Desugars an array literal containing at least one by-reference entry into prelude
/// statements on a hidden temporary, yielded through the `ExprKind::Assignment` prelude
/// machinery (the same shape the expression-position append desugar uses):
///
/// ```php
/// $arr = ['a' => 1, 's' => &$v, 'b' => f()];
/// // becomes:
/// $__elephc_lit_L_C = [];
/// $__elephc_lit_L_C['a'] = 1;          // ArrayAssign
/// $__elephc_lit_L_C['s'] = &$v;        // RefAssignToTarget (keyed)
/// $__elephc_lit_L_C['b'] = f();
/// // expression value = ($__elephc_lit_yield_L_C = $__elephc_lit_L_C)
/// ```
///
/// Entries are lowered IN SOURCE ORDER so side effects match PHP. Positional entries become
/// runtime appends (`ArrayPush` / append-form `RefAssignToTarget`) so PHP's
/// next-integer-key rule holds when explicit integer keys interleave. The yield copies the
/// temp into a DISTINCT fresh local (never `$t = $t`), matching the append-expression
/// desugar's two-slot rule so `store_local`'s release-then-acquire never frees the value it
/// hands back. Spread entries inside a ref-bearing literal are not expressible through the
/// per-entry statement forms (a positional-only `array_push` would drop string keys), so
/// they are a loud error rather than a silent mis-lowering.
fn build_ref_entry_literal_desugar(
    entries: Vec<ArrayLiteralEntry>,
    span: Span,
) -> Result<Expr, CompileError> {
    let temp_name = format!("__elephc_lit_{}_{}", span.line, span.col);
    let mut prelude = Vec::with_capacity(entries.len() + 1);
    // Hidden-temp init: `$tmp = [];` — the per-entry statements below populate it.
    prelude.push(Stmt::new(
        StmtKind::Assign {
            name: temp_name.clone(),
            value: Expr::new(ExprKind::ArrayLiteral(Vec::new()), span),
        },
        span,
    ));
    // Tracks whether a by-reference entry has been emitted yet: a later plain KEYED entry
    // whose key collides with a ref bucket must REPLACE the bucket (PHP literal
    // construction uses zend_hash_update — the duplicate discards the reference without
    // writing through it), so an `unset($tmp[key])` is emitted first (see the keyed arm).
    let mut seen_ref_entry = false;
    // Counter for hidden key temporaries binding a non-literal duplicate-guard key once.
    let mut key_temp_index = 0usize;
    for entry in entries {
        match entry {
            // Plain positional entry: `$tmp[] = value;` (runtime next-integer-key rule).
            ArrayLiteralEntry::Positional {
                value,
                by_ref: false,
            } => {
                let entry_span = value.span;
                prelude.push(Stmt::new(
                    StmtKind::ArrayPush {
                        array: temp_name.clone(),
                        value,
                    },
                    entry_span,
                ));
            }
            // By-reference positional entry: `$tmp[] = &$src;` (committed append-ref form).
            ArrayLiteralEntry::Positional {
                value: source,
                by_ref: true,
            } => {
                let entry_span = source.span;
                prelude.push(Stmt::new(
                    StmtKind::RefAssignToTarget {
                        target: Expr::new(ExprKind::Variable(temp_name.clone()), entry_span),
                        source,
                        append: true,
                    },
                    entry_span,
                ));
                seen_ref_entry = true;
            }
            // Plain keyed entry: `$tmp[key] = value;`. After the first by-reference
            // entry, a duplicate key could hit a ref bucket; PHP's literal construction
            // REPLACES the bucket (discarding the reference, source untouched), while the
            // desugared `$tmp[key] = value` would write THROUGH the reference. Clearing
            // the bucket first (`unset($tmp[key])`, a no-op for a missing key) restores
            // the replace semantics unconditionally. A non-literal key is bound to a
            // hidden temporary once so the guard + assignment cannot evaluate it twice.
            ArrayLiteralEntry::Keyed {
                key,
                value,
                by_ref: false,
            } => {
                let entry_span = key.span;
                let key = if seen_ref_entry {
                    let key = bind_key_once_for_duplicate_guard(
                        &mut prelude,
                        key,
                        span,
                        &mut key_temp_index,
                    );
                    prelude.push(build_unset_element_stmt(&temp_name, &key, entry_span));
                    key
                } else {
                    key
                };
                prelude.push(Stmt::new(
                    StmtKind::ArrayAssign {
                        array: temp_name.clone(),
                        index: key,
                        value,
                    },
                    entry_span,
                ));
            }
            // By-reference keyed entry: `$tmp[key] = &$src;` (committed keyed-ref form).
            // No duplicate guard is needed: `__rt_hash_bind_ref_element` already unsets
            // the key's prior value before installing the shared cell.
            ArrayLiteralEntry::Keyed {
                key,
                value: source,
                by_ref: true,
            } => {
                let entry_span = key.span;
                prelude.push(Stmt::new(
                    StmtKind::RefAssignToTarget {
                        target: Expr::new(
                            ExprKind::ArrayAccess {
                                array: Box::new(Expr::new(
                                    ExprKind::Variable(temp_name.clone()),
                                    entry_span,
                                )),
                                index: Box::new(key),
                            },
                            entry_span,
                        ),
                        source,
                        append: false,
                    },
                    entry_span,
                ));
                seen_ref_entry = true;
            }
            ArrayLiteralEntry::Spread {
                span: spread_span, ..
            } => {
                return Err(CompileError::new(
                    spread_span,
                    "Spread (...) inside an array literal with a by-reference entry is not supported",
                ));
            }
        }
    }
    // Yield the built array by copying the temp into a DISTINCT fresh local: the two-slot
    // copy is the everyday `$b = $t` shape, and the array copy shares tag-11 reference cells
    // (incref, not deep copy) so entry aliasing survives, matching PHP.
    let yield_name = format!("__elephc_lit_yield_{}_{}", span.line, span.col);
    Ok(Expr::new(
        ExprKind::Assignment {
            target: Box::new(Expr::new(ExprKind::Variable(yield_name), span)),
            value: Box::new(Expr::new(ExprKind::Variable(temp_name), span)),
            result_target: None,
            prelude,
            conditional_value_temp: None,
        },
        span,
    ))
}

/// Binds a duplicate-guard key to a hidden temporary when it is not a replayable literal.
///
/// The duplicate-key guard (`unset($tmp[key]);` followed by `$tmp[key] = value;`) mentions
/// the key expression twice, but PHP evaluates a literal-entry key exactly once. Int/string
/// literals replay for free; any other key expression (including a plain variable, whose
/// value the ENTRY VALUE expression could mutate between the two mentions) is evaluated once
/// into `$__elephc_lit_key_L_C_i` and both mentions read the temporary.
fn bind_key_once_for_duplicate_guard(
    prelude: &mut Vec<Stmt>,
    key: Expr,
    literal_span: Span,
    key_temp_index: &mut usize,
) -> Expr {
    if matches!(
        key.kind,
        ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_)
    ) {
        return key;
    }
    let key_span = key.span;
    let key_temp = format!(
        "__elephc_lit_key_{}_{}_{}",
        literal_span.line, literal_span.col, *key_temp_index
    );
    *key_temp_index += 1;
    prelude.push(Stmt::new(
        StmtKind::Assign {
            name: key_temp.clone(),
            value: key,
        },
        key_span,
    ));
    Expr::new(ExprKind::Variable(key_temp), key_span)
}

/// Builds the `unset($tmp[key]);` duplicate-key guard statement for the ref-literal desugar.
///
/// `unset()` on a missing key is a no-op, so the guard is unconditionally safe; on a key
/// that a previous by-reference entry bound, it discards the bucket and its cell share so
/// the following plain assignment recreates an ordinary bucket instead of writing through
/// the reference (PHP duplicate-key replace semantics).
fn build_unset_element_stmt(temp_name: &str, key: &Expr, entry_span: Span) -> Stmt {
    let element = Expr::new(
        ExprKind::ArrayAccess {
            array: Box::new(Expr::new(
                ExprKind::Variable(temp_name.to_string()),
                entry_span,
            )),
            index: Box::new(key.clone()),
        },
        entry_span,
    );
    Stmt::new(
        StmtKind::ExprStmt(Expr::new(
            ExprKind::FunctionCall {
                name: crate::names::Name::unqualified("unset"),
                args: vec![element],
            },
            entry_span,
        )),
        entry_span,
    )
}

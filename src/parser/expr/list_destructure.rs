//! Purpose:
//! Desugars expression-position list/bracket destructuring with skipped, keyed, nested, or
//! non-variable-target patterns (`if ([, , , $access] = $scopes[$name] ?? null)`) onto the
//! committed statement-form destructuring via the `ExprKind::Assignment` prelude machinery.
//!
//! Called from:
//! - `crate::parser::expr::prefix::parse_prefix()` for `[pattern] = RHS` and
//!   `list(pattern) = RHS` in expression position.
//!
//! Key details:
//! - The all-simple-positional bracket form (`[$a, $b] = RHS`) is deliberately left to the
//!   pre-existing array-literal + Pratt `ExprKind::ListUnpack` path so its AST stays
//!   byte-identical (regression guard).
//! - The desugar evaluates the RHS once into a hidden `__elephc_destr_{line}_{col}` local,
//!   replays the exact statement-form destructuring from that local (the same
//!   `lower_list_unpack` builder the statement parser uses), and yields the local through a
//!   distinct `__elephc_destr_yield_*` local — PHP: a destructuring assignment evaluates to
//!   its full right-hand side, so `if ([, $b] = $x ?? null)` tests the RHS.
//! - Prelude statements ride `ExprKind::Assignment`, so conditional-context placement rules
//!   (ternary-branch confinement, `&&` right-operand confinement, per-iteration `while`
//!   re-evaluation) apply unchanged.

use crate::errors::CompileError;
use crate::lexer::token::SpannedToken;
use crate::lexer::Token;
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;

use super::pratt::parse_expr_bp;

/// Right binding power for the right-hand side of `=`, mirroring `assignment_bp`'s
/// `(l_bp, r_bp) = (7, 6)` so `[pattern] = $a ?? null` swallows the same RHS an ordinary
/// `$x = ...` assignment would.
const ASSIGN_RHS_BP: u8 = 6;

/// Tries to parse `[pattern] = RHS` in expression position as a destructuring assignment.
///
/// Expects `*pos` at the opening `[`. Returns `Ok(None)` — with `*pos` unchanged — when the
/// brackets are not followed by a plain `=` (an ordinary array literal) or when the pattern
/// is the all-simple-positional shape, which keeps its pre-existing array-literal + Pratt
/// `ExprKind::ListUnpack` path byte-identical. Otherwise consumes through the RHS and returns
/// the prelude-desugared expression; malformed patterns propagate the statement parser's
/// loud diagnostics (invalid target, mixed keyed/unkeyed, empty list).
pub(super) fn try_parse_bracket_destructure_expr(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<Expr>, CompileError> {
    let span = tokens[*pos].1.span;
    let Some(close) = find_matching_delimiter(tokens, *pos, &Token::LBracket, &Token::RBracket)
    else {
        return Ok(None);
    };
    if !matches!(tokens.get(close + 1).map(|(token, _)| token), Some(Token::Assign)) {
        return Ok(None);
    }

    let snapshot = *pos;
    let temp_name = destructure_temp_name(span);
    let source = Expr::new(ExprKind::Variable(temp_name.clone()), span);
    let destructure =
        crate::parser::stmt::parse_and_lower_bracket_destructure(tokens, pos, span, source)?;
    if matches!(destructure.kind, StmtKind::ListUnpack { .. }) {
        // All-simple positional (`[$a, $b] = ...`): rewind so the array-literal + Pratt
        // `ExprKind::ListUnpack` path produces the exact pre-existing AST.
        *pos = snapshot;
        return Ok(None);
    }

    let rhs = parse_destructure_rhs(tokens, pos, span)?;
    Ok(Some(build_destructure_expr(destructure, temp_name, rhs, span)))
}

/// Tries to parse `list(pattern) = RHS` in expression position as a destructuring assignment
/// (e.g. `if (list(, $b) = $arr)`), which PHP permits like the bracket form.
///
/// Expects `*pos` at the `list` identifier with `(` at `*pos + 1` (guaranteed by the caller's
/// match guard). Returns `Ok(None)` — with `*pos` unchanged — when the matching `)` is not
/// followed by a plain `=`, letting the identifier proceed through the ordinary named-expression
/// path for its usual diagnostics. An all-simple-positional pattern maps directly onto
/// `ExprKind::ListUnpack` (the same node the bracket form yields); other patterns take the
/// prelude desugar.
pub(super) fn try_parse_list_construct_destructure_expr(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<Expr>, CompileError> {
    let span = tokens[*pos].1.span;
    let Some(close) =
        find_matching_delimiter(tokens, *pos + 1, &Token::LParen, &Token::RParen)
    else {
        return Ok(None);
    };
    if !matches!(tokens.get(close + 1).map(|(token, _)| token), Some(Token::Assign)) {
        return Ok(None);
    }

    let temp_name = destructure_temp_name(span);
    let source = Expr::new(ExprKind::Variable(temp_name.clone()), span);
    let destructure = crate::parser::stmt::parse_and_lower_list_construct_destructure(
        tokens, pos, span, source,
    )?;
    let rhs = parse_destructure_rhs(tokens, pos, span)?;
    if let StmtKind::ListUnpack { vars, .. } = destructure.kind {
        // All-simple positional `list($a, $b) = RHS`: emit the same expression node the
        // bracket form produces, so both spellings share one lowering path.
        return Ok(Some(Expr::new(
            ExprKind::ListUnpack {
                vars,
                value: Box::new(rhs),
            },
            span,
        )));
    }
    Ok(Some(build_destructure_expr(destructure, temp_name, rhs, span)))
}

/// Consumes the `=` after a destructuring pattern and parses the right-hand side with the
/// standard assignment right binding power. The `=` is guaranteed by the caller's lookahead;
/// the check remains as a defensive diagnostic.
fn parse_destructure_rhs(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Expr, CompileError> {
    if !matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Assign)) {
        return Err(CompileError::new(span, "Expected '=' after list pattern"));
    }
    *pos += 1; // consume '='
    parse_expr_bp(tokens, pos, ASSIGN_RHS_BP)
}

/// Builds the desugared expression for a non-simple destructuring assignment: the prelude
/// evaluates the RHS once into the hidden temporary, then replays the statement-form
/// destructuring from it; the expression yields the temporary through a DISTINCT fresh
/// `__elephc_destr_yield_*` local (the two-slot `$b = $t` copy shape, never a `$t = $t`
/// self-assignment whose release-then-acquire would hand back freed memory).
fn build_destructure_expr(destructure: Stmt, temp_name: String, rhs: Expr, span: Span) -> Expr {
    let assign_temp = Stmt::new(
        StmtKind::Assign {
            name: temp_name.clone(),
            value: rhs,
        },
        span,
    );
    let prelude = vec![assign_temp, destructure];
    let yield_name = format!("__elephc_destr_yield_{}_{}", span.line, span.col);
    Expr::new(
        ExprKind::Assignment {
            target: Box::new(Expr::new(ExprKind::Variable(yield_name), span)),
            value: Box::new(Expr::new(ExprKind::Variable(temp_name), span)),
            result_target: None,
            prelude,
            conditional_value_temp: None,
        },
        span,
    )
}

/// Returns the hidden temporary name holding the evaluated RHS of an expression-position
/// destructuring, unique per source location (`__elephc_destr_{line}_{col}`).
fn destructure_temp_name(span: Span) -> String {
    format!("__elephc_destr_{}_{}", span.line, span.col)
}

/// Scans tokens starting at `open_pos` for the close delimiter matching the open delimiter at
/// `open_pos`, tracking nesting depth. Returns the index of the matching close token, or
/// `None` if the delimiters never balance before the token stream ends.
fn find_matching_delimiter(
    tokens: &[SpannedToken],
    open_pos: usize,
    open: &Token,
    close: &Token,
) -> Option<usize> {
    let mut depth = 0usize;
    for (i, (token, _)) in tokens.iter().enumerate().skip(open_pos) {
        if token == open {
            depth += 1;
        } else if token == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

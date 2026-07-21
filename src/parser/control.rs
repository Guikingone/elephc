//! Purpose:
//! Parses PHP control-flow statements and inline loop/header expressions.
//! Covers if/ifdef, loops, foreach, try/catch/finally, switch, and control headers.
//!
//! Called from:
//! - `crate::parser::stmt::parse_stmt()`.
//!
//! Key details:
//! - Control parsers must preserve PHP statement nesting and spans for later flow and diagnostic passes.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{BinOp, CatchClause, Expr, ExprKind, Stmt, StmtKind};
use crate::parser::expr::{parse_assignment_value_expr, parse_expr};
use crate::parser::foreach_target::{lower_foreach_binding, parse_foreach_binding, ForeachBinding};
use crate::parser::stmt::{expect_semicolon, expect_token, name_starts_at, parse_block, parse_body, parse_name};
use crate::span::Span;

/// Parse: if (expr) { stmts } (elseif (expr) { stmts })* (else { stmts })?
pub fn parse_if(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;

    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'if'")?;
    let condition = parse_expr(tokens, pos)?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after if condition")?;
    let then_body = parse_body(tokens, pos)?;

    let mut elseif_clauses = Vec::new();
    let mut else_body = None;

    loop {
        if *pos >= tokens.len() {
            break;
        }
        if tokens[*pos].0 == Token::ElseIf {
            *pos += 1;
            expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'elseif'")?;
            let cond = parse_expr(tokens, pos)?;
            expect_token(tokens, pos, &Token::RParen, "Expected ')' after elseif condition")?;
            let body = parse_body(tokens, pos)?;
            elseif_clauses.push((cond, body));
        } else if tokens[*pos].0 == Token::Else {
            *pos += 1;
            else_body = Some(parse_body(tokens, pos)?);
            break;
        } else {
            break;
        }
    }

    Ok(Stmt::new(
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        },
        span,
    ))
}

/// Parse: ifdef SYMBOL { stmts } (else { stmts })?
pub fn parse_ifdef(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;

    let symbol = match tokens.get(*pos).map(|(t, _)| t) {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(CompileError::new(span, "Expected symbol name after 'ifdef'")),
    };
    *pos += 1;

    let then_body = parse_block(tokens, pos)?;
    let else_body = if *pos < tokens.len() && tokens[*pos].0 == Token::Else {
        *pos += 1;
        Some(parse_block(tokens, pos)?)
    } else {
        None
    };

    Ok(Stmt::new(
        StmtKind::IfDef {
            symbol,
            then_body,
            else_body,
        },
        span,
    ))
}

/// Parse: while (expr) { stmts }
pub fn parse_while(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'while'")?;
    let condition = parse_expr(tokens, pos)?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after while condition")?;
    let body = parse_body(tokens, pos)?;
    Ok(Stmt::new(StmtKind::While { condition, body }, span))
}

/// Parses a foreach loop: `foreach ($array as $value)` or `foreach ($array as $key => $value)`.
/// Supports by-reference values via `&` prefix and by-reference loop variables.
///
/// Also supports PHP 7.1+ array-destructuring value patterns: `foreach ($arr as [$a, $b])`
/// and `foreach ($arr as $k => ['key' => $v])`. The bracket pattern is parsed and lowered
/// (via the standalone list-destructuring lowering) against a synthetic per-iteration
/// element variable, and the resulting destructure statement is prepended to the body so
/// the rest of the `Foreach` node — and every pass that reads its `value_var` — is unchanged.
///
/// Non-plain writable lvalue targets (`foreach ($defs as $this->id => $d)`,
/// `foreach ($rows as $out["k"])`, `foreach ($m as R::$k => $v)`) desugar the same way:
/// the `Foreach` node binds a hidden loop variable and a `<lvalue> = $hidden;` statement is
/// prepended to the body (value store before key store, matching PHP's per-iteration
/// assignment order). By-ref bindings stay plain-variable-only: `as &$this->v` remains a
/// loud error until foreach by-ref write-through is implemented.
pub fn parse_foreach(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'foreach'")?;
    let array = parse_expr(tokens, pos)?;
    expect_token(tokens, pos, &Token::As, "Expected 'as' in foreach")?;

    // Destructure value pattern: `foreach ($arr as [pattern])`.
    if matches!(
        tokens.get(*pos).map(|(token, _)| token),
        Some(Token::LBracket)
    ) {
        return finish_foreach_destructure(tokens, pos, span, array, None);
    }

    let first_by_ref = if matches!(
        tokens.get(*pos).map(|(token, _)| token),
        Some(Token::Ampersand)
    ) {
        *pos += 1;
        true
    } else {
        false
    };

    let first = if first_by_ref {
        // A by-ref binding must be a plain variable: `as &$this->v` (by-ref write-through
        // into a complex lvalue) is intentionally unsupported and stays a loud error.
        match tokens.get(*pos).map(|(t, _)| t) {
            Some(Token::Variable(n)) => {
                let name = n.clone();
                *pos += 1;
                ForeachBinding::Plain(name)
            }
            _ => return Err(CompileError::new(span, "Expected variable after 'as'")),
        }
    } else {
        parse_foreach_binding(tokens, pos, span, "Expected variable after 'as'")?
    };

    // Check for => (foreach $arr as $key => $value)
    let (key_binding, value_binding, value_by_ref) =
        if *pos < tokens.len() && tokens[*pos].0 == Token::DoubleArrow {
        if first_by_ref {
            return Err(CompileError::new(
                span,
                "Key element cannot be a reference in foreach",
            ));
        }
        *pos += 1;
        // Destructure value pattern: `foreach ($arr as $k => [pattern])`.
        if matches!(
            tokens.get(*pos).map(|(token, _)| token),
            Some(Token::LBracket)
        ) {
            let (key_name, key_store) = lower_foreach_binding(first, "key", span)?;
            let mut stmt = finish_foreach_destructure(tokens, pos, span, array, Some(key_name))?;
            // A desugared key store runs after the destructure statement (PHP assigns the
            // value binding before the key binding each iteration).
            if let Some(store) = key_store {
                if let StmtKind::Foreach { body, .. } = &mut stmt.kind {
                    body.insert(1, store);
                }
            }
            return Ok(stmt);
        }
        let value_by_ref = if matches!(
            tokens.get(*pos).map(|(token, _)| token),
            Some(Token::Ampersand)
        ) {
            *pos += 1;
            true
        } else {
            false
        };
        let value = if value_by_ref {
            // Same plain-variable-only rule for by-ref value bindings as after 'as'.
            match tokens.get(*pos).map(|(t, _)| t) {
                Some(Token::Variable(n)) => {
                    let name = n.clone();
                    *pos += 1;
                    ForeachBinding::Plain(name)
                }
                _ => return Err(CompileError::new(span, "Expected variable after '=>'")),
            }
        } else {
            parse_foreach_binding(tokens, pos, span, "Expected variable after '=>'")?
        };
        (Some(first), value, value_by_ref)
    } else {
        (None, first, first_by_ref)
    };

    expect_token(tokens, pos, &Token::RParen, "Expected ')' after foreach")?;
    let mut body = parse_body(tokens, pos)?;

    let (value_var, value_store) = lower_foreach_binding(value_binding, "val", span)?;
    let (key_var, key_store) = match key_binding {
        Some(binding) => {
            let (name, store) = lower_foreach_binding(binding, "key", span)?;
            (Some(name), store)
        }
        None => (None, None),
    };
    // PHP assigns the value binding first and the key binding second each iteration
    // (`foreach ([7 => 9] as $x => $x)` leaves $x == 7), so the desugared stores are
    // prepended in value-then-key order ahead of the user body.
    let mut desugared_stores = Vec::new();
    desugared_stores.extend(value_store);
    desugared_stores.extend(key_store);
    if !desugared_stores.is_empty() {
        body.splice(0..0, desugared_stores);
    }

    Ok(Stmt::new(
        StmtKind::Foreach {
            array,
            key_var,
            value_var,
            value_by_ref,
            body,
        },
        span,
    ))
}

/// Builds a `Foreach` whose value is destructured by a bracket pattern.
///
/// `key_var` is `Some(name)` for the `$k => [pattern]` form, `None` for the `as [pattern]`
/// form. The bracket pattern at `*pos` is parsed and lowered against a fresh synthetic
/// element variable (`__elephc_foreach_destructure_{line}_{col}`, unique per foreach by
/// its starting span) and the resulting destructure statement is prepended to the parsed
/// body. The `Foreach` node itself uses the synthetic variable as `value_var`, so every
/// downstream pass that reads `value_var` continues to work unchanged.
fn finish_foreach_destructure(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    array: Expr,
    key_var: Option<String>,
) -> Result<Stmt, CompileError> {
    let temp = format!("__elephc_foreach_destructure_{}_{}", span.line, span.col);
    let destructure_stmt = crate::parser::stmt::parse_and_lower_bracket_destructure(
        tokens,
        pos,
        span,
        Expr::new(ExprKind::Variable(temp.clone()), span),
    )?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after foreach")?;
    let mut body = parse_body(tokens, pos)?;
    body.insert(0, destructure_stmt);
    Ok(Stmt::new(
        StmtKind::Foreach {
            array,
            key_var,
            value_var: temp,
            value_by_ref: false,
            body,
        },
        span,
    ))
}

/// Parse: do { stmts } while (expr);
pub fn parse_do_while(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let body = parse_block(tokens, pos)?;
    expect_token(tokens, pos, &Token::While, "Expected 'while' after do block")?;
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'while'")?;
    let condition = parse_expr(tokens, pos)?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after condition")?;
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::DoWhile { body, condition }, span))
}

/// Parse: for (init; condition; update) { stmts }
pub fn parse_for(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'for'")?;

    let init = parse_for_clause_list(tokens, pos, &Token::Semicolon)?;
    expect_semicolon(tokens, pos)?;

    let condition = if *pos < tokens.len() && tokens[*pos].0 != Token::Semicolon {
        Some(parse_expr(tokens, pos)?)
    } else {
        None
    };
    expect_semicolon(tokens, pos)?;

    let update = parse_for_clause_list(tokens, pos, &Token::RParen)?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after for clauses")?;

    let body = parse_body(tokens, pos)?;

    Ok(Stmt::new(
        StmtKind::For {
            init,
            condition,
            update,
            body,
        },
        span,
    ))
}

/// Parse: try { stmts } (catch (TypeA|TypeB $e) { stmts })+ (finally { stmts })?
///     or: try { stmts } finally { stmts }
pub fn parse_try(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let try_body = parse_body(tokens, pos)?;

    let mut catches = Vec::new();
    while *pos < tokens.len() && tokens[*pos].0 == Token::Catch {
        *pos += 1;
        expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'catch'")?;
        let mut exception_types = Vec::new();
        loop {
            if *pos < tokens.len() && tokens[*pos].0 == Token::Self_ {
                exception_types.push(crate::names::Name::unqualified("self"));
                *pos += 1;
            } else if *pos < tokens.len() && tokens[*pos].0 == Token::Parent {
                exception_types.push(crate::names::Name::unqualified("parent"));
                *pos += 1;
            } else if name_starts_at(tokens, *pos) {
                exception_types.push(parse_name(
                    tokens,
                    pos,
                    span,
                    "Expected exception class name in catch clause",
                )?);
            } else {
                return Err(CompileError::new(
                    span,
                    "Expected exception class name in catch clause",
                ));
            }
            if *pos < tokens.len() && tokens[*pos].0 == Token::Pipe {
                *pos += 1;
                continue;
            }
            break;
        }
        let variable = match tokens.get(*pos).map(|(t, _)| t) {
            Some(Token::Variable(name)) => {
                *pos += 1;
                Some(name.clone())
            }
            Some(Token::RParen) => None,
            _ => {
                return Err(CompileError::new(
                    span,
                    "Expected catch variable or ')' after exception type",
                ))
            }
        };
        expect_token(tokens, pos, &Token::RParen, "Expected ')' after catch clause")?;
        let body = parse_body(tokens, pos)?;
        catches.push(CatchClause {
            exception_types,
            variable,
            body,
        });
    }

    let finally_body = if *pos < tokens.len() && tokens[*pos].0 == Token::Finally {
        *pos += 1;
        Some(parse_body(tokens, pos)?)
    } else {
        None
    };

    if catches.is_empty() && finally_body.is_none() {
        return Err(CompileError::new(
            span,
            "Expected at least one catch or a finally block after try",
        ));
    }

    Ok(Stmt::new(
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        },
        span,
    ))
}

/// Parses a `for` init or update clause, which may be a comma-separated list of arbitrary
/// expressions and inline assignments (PHP's `expr_list` grammar for these clauses).
///
/// Stops at `terminator` (a `;` for the init clause, a `)` for the update clause). An empty clause
/// yields `None`; a single item is returned directly; several comma-separated items are wrapped in
/// a `Synthetic` block so the `for` lowering runs them in order (the init list once, the update
/// list after each iteration), matching PHP's `for ($i = 0, $j = 10; ...; $i++, $j--)` and
/// `for (next($paths); ...; next($paths))`.
fn parse_for_clause_list(
    tokens: &[SpannedToken],
    pos: &mut usize,
    terminator: &Token,
) -> Result<Option<Box<Stmt>>, CompileError> {
    if *pos >= tokens.len() || tokens[*pos].0 == *terminator {
        return Ok(None);
    }
    let list_span = tokens[*pos].1.span;
    let mut stmts = Vec::new();
    loop {
        stmts.push(parse_for_clause_item(tokens, pos)?);
        if *pos < tokens.len() && tokens[*pos].0 == Token::Comma {
            *pos += 1; // consume ','
            continue;
        }
        break;
    }
    if stmts.len() == 1 {
        Ok(Some(Box::new(stmts.pop().expect("one statement present"))))
    } else {
        Ok(Some(Box::new(Stmt::new(StmtKind::Synthetic(stmts), list_span))))
    }
}

/// Parses one item of a `for` init/update clause list.
///
/// Historical inline-assignment shapes (`$v = expr`, compound assigns, `$v ??= expr`, and
/// whole-item `++$v` / `--$v` / `$v++` / `$v--`) keep their dedicated statement AST via
/// `parse_assign_inline` so existing programs parse byte-identically. Every other item is a
/// full expression (a call like `next($paths)`, a method call, a complex-lvalue assignment,
/// ...) parsed with `parse_expr` and wrapped in an effect-only `ExprStmt`, matching PHP's
/// arbitrary-expression `for` clauses.
fn parse_for_clause_item(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Stmt, CompileError> {
    let span = tokens[*pos].1.span;
    if for_clause_fast_path_applies(tokens, *pos) {
        return parse_assign_inline(tokens, pos, span);
    }
    let expr = parse_expr(tokens, pos)?;
    Ok(Stmt::new(StmtKind::ExprStmt(expr), span))
}

/// Reports whether the for-clause item starting at `pos` matches one of the shapes the
/// historical inline-assignment parser accepted: a whole-item `++$v` / `--$v` / `$v++` /
/// `$v--` (the inc/dec must end the item so longer expressions like `$i++ + 1` fall back
/// to the full-expression path), or a `$v` head directly followed by `=`, a compound
/// assignment, or `??=`. Only these shapes take `parse_assign_inline`, keeping their AST
/// byte-identical to the pre-expression-list parser.
fn for_clause_fast_path_applies(tokens: &[SpannedToken], pos: usize) -> bool {
    match tokens.get(pos).map(|(t, _)| t) {
        Some(Token::PlusPlus | Token::MinusMinus) => {
            matches!(tokens.get(pos + 1).map(|(t, _)| t), Some(Token::Variable(_)))
                && for_clause_item_boundary(tokens.get(pos + 2).map(|(t, _)| t))
        }
        Some(Token::Variable(_)) => match tokens.get(pos + 1).map(|(t, _)| t) {
            Some(Token::PlusPlus | Token::MinusMinus) => {
                for_clause_item_boundary(tokens.get(pos + 2).map(|(t, _)| t))
            }
            Some(
                Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::StarAssign
                | Token::StarStarAssign
                | Token::SlashAssign
                | Token::PercentAssign
                | Token::DotAssign
                | Token::AmpAssign
                | Token::PipeAssign
                | Token::CaretAssign
                | Token::LessLessAssign
                | Token::GreaterGreaterAssign
                | Token::QuestionQuestionAssign,
            ) => true,
            _ => false,
        },
        _ => false,
    }
}

/// Reports whether `token` ends a for-clause item: a list `,`, the init-clause `;`, the
/// update-clause `)`, or end of input (which the clause parsers report as a loud error).
fn for_clause_item_boundary(token: Option<&Token>) -> bool {
    matches!(token, Some(Token::Comma | Token::Semicolon | Token::RParen) | None)
}

/// Parses one inline assignment or increment/decrement statement without a trailing
/// semicolon, for use inside `for` clauses: `++$v` / `--$v` / `$v++` / `$v--` become
/// inc/dec `ExprStmt`s, and `$v = expr` / compound assigns / `$v ??= expr` become
/// `StmtKind::Assign`. Callers gate entry through `for_clause_fast_path_applies`; other
/// shapes error loudly here.
pub fn parse_assign_inline(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    if *pos < tokens.len() {
        match &tokens[*pos].0 {
            Token::PlusPlus => {
                *pos += 1;
                let name = match tokens.get(*pos).map(|(t, _)| t) {
                    Some(Token::Variable(n)) => n.clone(),
                    _ => return Err(CompileError::new(span, "Expected variable after '++'")),
                };
                *pos += 1;
                let expr = Expr::new(ExprKind::PreIncrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            Token::MinusMinus => {
                *pos += 1;
                let name = match tokens.get(*pos).map(|(t, _)| t) {
                    Some(Token::Variable(n)) => n.clone(),
                    _ => return Err(CompileError::new(span, "Expected variable after '--'")),
                };
                *pos += 1;
                let expr = Expr::new(ExprKind::PreDecrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            _ => {}
        }
    }

    let name = match &tokens[*pos].0 {
        Token::Variable(n) => n.clone(),
        _ => return Err(CompileError::new(span, "Expected variable in for clause")),
    };
    *pos += 1;

    if *pos < tokens.len() {
        match &tokens[*pos].0 {
            Token::PlusPlus => {
                *pos += 1;
                let expr = Expr::new(ExprKind::PostIncrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            Token::MinusMinus => {
                *pos += 1;
                let expr = Expr::new(ExprKind::PostDecrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            _ => {}
        }
    }

    if *pos >= tokens.len() {
        return Err(CompileError::new(span, "Expected '=' after variable name"));
    }

    let compound_op = match &tokens[*pos].0 {
        Token::PlusAssign => Some(BinOp::Add),
        Token::MinusAssign => Some(BinOp::Sub),
        Token::StarAssign => Some(BinOp::Mul),
        Token::StarStarAssign => Some(BinOp::Pow),
        Token::SlashAssign => Some(BinOp::Div),
        Token::PercentAssign => Some(BinOp::Mod),
        Token::DotAssign => Some(BinOp::Concat),
        Token::AmpAssign => Some(BinOp::BitAnd),
        Token::PipeAssign => Some(BinOp::BitOr),
        Token::CaretAssign => Some(BinOp::BitXor),
        Token::LessLessAssign => Some(BinOp::ShiftLeft),
        Token::GreaterGreaterAssign => Some(BinOp::ShiftRight),
        Token::Assign => None,
        Token::QuestionQuestionAssign => {
            *pos += 1;
            let rhs = parse_assignment_value_expr(tokens, pos)?;
            let value = Expr::new(
                ExprKind::NullCoalesce {
                    value: Box::new(Expr::new(ExprKind::Variable(name.clone()), span)),
                    default: Box::new(rhs),
                },
                span,
            );
            return Ok(Stmt::new(StmtKind::Assign { name, value }, span));
        }
        _ => return Err(CompileError::new(span, "Expected '=' after variable name")),
    };
    *pos += 1;

    let rhs = parse_assignment_value_expr(tokens, pos)?;
    let value = if let Some(op) = compound_op {
        Expr::new(
            ExprKind::BinaryOp {
                left: Box::new(Expr::new(ExprKind::Variable(name.clone()), span)),
                op,
                right: Box::new(rhs),
            },
            span,
        )
    } else {
        rhs
    };
    Ok(Stmt::new(StmtKind::Assign { name, value }, span))
}

/// Parse: switch (expr) { case expr: stmts... case expr: stmts... default: stmts... }
pub fn parse_switch(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1; // consume 'switch'
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'switch'")?;
    let subject = parse_expr(tokens, pos)?;
    expect_token(tokens, pos, &Token::RParen, "Expected ')' after switch expression")?;
    expect_token(tokens, pos, &Token::LBrace, "Expected '{' after switch")?;

    let mut cases: Vec<(Vec<Expr>, Vec<Stmt>)> = Vec::new();
    let mut default: Option<Vec<Stmt>> = None;

    while *pos < tokens.len() && tokens[*pos].0 != Token::RBrace {
        if tokens[*pos].0 == Token::Case {
            // Parse one or more case values
            let mut values = Vec::new();
            while *pos < tokens.len() && tokens[*pos].0 == Token::Case {
                *pos += 1;
                values.push(parse_expr(tokens, pos)?);
                expect_token(tokens, pos, &Token::Colon, "Expected ':' after case value")?;
            }
            // Parse case body (statements until next case/default/})
            let mut body = Vec::new();
            while *pos < tokens.len()
                && tokens[*pos].0 != Token::Case
                && tokens[*pos].0 != Token::Default
                && tokens[*pos].0 != Token::RBrace
            {
                body.push(crate::parser::stmt::parse_stmt(tokens, pos)?);
            }
            cases.push((values, body));
        } else if tokens[*pos].0 == Token::Default {
            *pos += 1;
            expect_token(tokens, pos, &Token::Colon, "Expected ':' after 'default'")?;
            let mut body = Vec::new();
            while *pos < tokens.len()
                && tokens[*pos].0 != Token::Case
                && tokens[*pos].0 != Token::RBrace
            {
                body.push(crate::parser::stmt::parse_stmt(tokens, pos)?);
            }
            default = Some(body);
        } else {
            return Err(CompileError::new(
                tokens[*pos].1.span,
                "Expected 'case' or 'default' inside switch",
            ));
        }
    }

    expect_token(tokens, pos, &Token::RBrace, "Expected '}' to close switch")?;

    Ok(Stmt::new(
        StmtKind::Switch {
            subject,
            cases,
            default,
        },
        span,
    ))
}

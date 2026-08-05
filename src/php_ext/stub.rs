//! Purpose:
//! Derives a hosted PHP extension's callable surface from its own `*.stub.php`
//! file — the same file PECL generates `arginfo` from — so signatures never have
//! to be transcribed into a hand-written catalogue that can drift from the
//! extension it describes.
//!
//! Called from:
//! - `crate::php_ext` when an extension is declared, to feed the checker and EIR.
//!
//! Key details:
//! - A stub is ordinary PHP, so it is lexed and parsed with the normal front end;
//!   no second parser exists to disagree with the first.
//! - **`#` starts a comment in PHP.** A stub's `#ifdef APC_DEBUG` guards are
//!   therefore invisible to the parser, and the functions they wrap would look
//!   unconditional. Those regions are detected textually and their functions are
//!   marked [`StubFunction::conditional`] rather than silently admitted.
//! - `UNKNOWN` is a stub-only marker for "the real default lives in C". It is
//!   recorded as [`StubDefault::Unknown`], never as a value.

use crate::errors::CompileError;
use crate::lexer;
use crate::parser;
use crate::parser::ast::{StmtKind, TypeExpr};

/// What a parameter's default tells us — which is sometimes "nothing".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StubDefault {
    /// No default: the argument is required.
    None,
    /// A real default the stub spells out.
    Value,
    /// `UNKNOWN`: the stub declines to state it; only the C code knows.
    Unknown,
}

#[derive(Debug, Clone)]
pub struct StubParam {
    pub name: String,
    pub type_hint: Option<String>,
    pub default: StubDefault,
    pub by_ref: bool,
}

#[derive(Debug, Clone)]
pub struct StubFunction {
    pub name: String,
    pub params: Vec<StubParam>,
    pub variadic: Option<String>,
    pub return_type: Option<String>,
    pub by_ref_return: bool,
    /// True when the declaration sat inside a `#if`/`#ifdef` region: the
    /// extension may or may not export it, depending on how it was built.
    pub conditional: bool,
}

#[derive(Debug, Clone)]
pub struct StubClass {
    pub name: String,
    pub methods: Vec<StubFunction>,
    pub conditional: bool,
}

/// Everything a stub can tell us, plus everything it cannot.
#[derive(Debug, Clone, Default)]
pub struct ExtensionSurface {
    pub functions: Vec<StubFunction>,
    pub classes: Vec<StubClass>,
    /// Facts the stub cannot express, which must be resolved elsewhere (against
    /// the built object file, or by the recipe's own configure step) before the
    /// surface is trusted.
    pub caveats: Vec<String>,
}

impl ExtensionSurface {
    /// Functions safe to expose without further checking.
    pub fn unconditional_functions(&self) -> impl Iterator<Item = &StubFunction> {
        self.functions.iter().filter(|f| !f.conditional)
    }
}

/// Line ranges guarded by C preprocessor directives. PHP treats `#` as a
/// comment, so the parser cannot see these; they are found textually.
fn conditional_line_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("#if") {
            open.push(idx + 1);
        } else if t.starts_with("#endif") {
            if let Some(start) = open.pop() {
                ranges.push((start, idx + 1));
            }
        }
    }
    // An unterminated #if guards everything after it.
    let total = source.lines().count();
    for start in open {
        ranges.push((start, total));
    }
    ranges
}

fn is_conditional(line: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(a, b)| line >= *a && line <= *b)
}

fn render_type(t: &TypeExpr) -> String {
    format!("{t:?}")
}

fn convert_function(
    name: &str,
    params: &[(String, Option<TypeExpr>, Option<crate::parser::ast::Expr>, bool)],
    variadic: Option<&String>,
    return_type: Option<&TypeExpr>,
    by_ref_return: bool,
    conditional: bool,
) -> StubFunction {
    let params = params
        .iter()
        .map(|(pname, ty, default, by_ref)| StubParam {
            name: pname.clone(),
            type_hint: ty.as_ref().map(render_type),
            default: match default {
                None => StubDefault::None,
                Some(expr) => {
                    // `UNKNOWN` is a stub convention, not a PHP constant.
                    if format!("{expr:?}").contains("UNKNOWN") {
                        StubDefault::Unknown
                    } else {
                        StubDefault::Value
                    }
                }
            },
            by_ref: *by_ref,
        })
        .collect();

    StubFunction {
        name: name.to_string(),
        params,
        variadic: variadic.cloned(),
        return_type: return_type.map(render_type),
        by_ref_return,
        conditional,
    }
}

/// Parse a `*.stub.php` into the surface it declares.
///
/// `source` is the stub's full text; it is parsed with the ordinary PHP front
/// end, so a malformed stub fails the same way malformed user code does.
pub fn parse_stub(source: &str) -> Result<ExtensionSurface, CompileError> {
    let guarded = conditional_line_ranges(source);
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;

    let mut surface = ExtensionSurface::default();

    // `Program` is a `Vec<Stmt>`, and each `Stmt` carries its source span — so a
    // declaration's line is exact, not guessed from where its name appears.
    for stmt in &program {
        let line = stmt.span.line as usize;
        match &stmt.kind {
            StmtKind::FunctionDecl {
                name,
                params,
                variadic,
                return_type,
                by_ref_return,
                ..
            } => {
                surface.functions.push(convert_function(
                    name,
                    params,
                    variadic.as_ref(),
                    return_type.as_ref(),
                    *by_ref_return,
                    is_conditional(line, &guarded),
                ));
            }
            StmtKind::ClassDecl { name, .. } => {
                surface.classes.push(StubClass {
                    name: name.clone(),
                    methods: Vec::new(),
                    conditional: is_conditional(line, &guarded),
                });
            }
            _ => {}
        }
    }

    if surface.functions.iter().any(|f| f.conditional)
        || surface.classes.iter().any(|c| c.conditional)
    {
        surface.caveats.push(
            "stub contains #if/#ifdef guards: conditional declarations must be confirmed \
             against the built object file's symbol table"
                .to_string(),
        );
    }
    if surface
        .functions
        .iter()
        .any(|f| f.params.iter().any(|p| p.default == StubDefault::Unknown))
    {
        surface.caveats.push(
            "stub uses UNKNOWN defaults: the real values live in C and must come from the \
             extension's arginfo, not from this stub"
                .to_string(),
        );
    }
    surface.caveats.push(
        "a stub cannot express aliases or conditionally registered entries created in C; \
         cross-check the built symbol table before trusting this surface to be complete"
            .to_string(),
    );

    Ok(surface)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real APCu stub shape, including its `#ifdef`-guarded debug function.
    const APCU_STUB: &str = r#"<?php

function apcu_clear_cache(): bool {}

function apcu_cache_info(bool $limited = false): array|false {}

/** @param array|string $key */
function apcu_store($key, mixed $value = UNKNOWN, int $ttl = 0): array|bool {}

/** @param bool $success */
function apcu_inc(string $key, int $step = 1, &$success = null, int $ttl = 0): int|false {}

#ifdef APC_DEBUG
function apcu_inc_request_time(int $by = 1): void {}
#endif
"#;

    #[test]
    fn extracts_functions_from_a_real_stub() {
        let s = parse_stub(APCU_STUB).expect("stub parses");
        let names: Vec<_> = s.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"apcu_clear_cache"));
        assert!(names.contains(&"apcu_store"));
        assert!(names.contains(&"apcu_inc"));
    }

    /// The trap: `#` is a PHP comment, so without textual detection this
    /// debug-only function would look unconditionally available.
    #[test]
    fn marks_ifdef_guarded_functions_as_conditional() {
        let s = parse_stub(APCU_STUB).expect("stub parses");
        let guarded = s
            .functions
            .iter()
            .find(|f| f.name == "apcu_inc_request_time")
            .expect("guarded function is present in the stub");
        assert!(
            guarded.conditional,
            "an #ifdef-guarded function must not be reported as unconditional"
        );
        assert!(!s.unconditional_functions().any(|f| f.name == "apcu_inc_request_time"));
    }

    #[test]
    fn unguarded_functions_are_not_marked_conditional() {
        let s = parse_stub(APCU_STUB).expect("stub parses");
        let plain = s
            .functions
            .iter()
            .find(|f| f.name == "apcu_clear_cache")
            .expect("present");
        assert!(!plain.conditional);
    }

    #[test]
    fn records_unknown_defaults_rather_than_inventing_values() {
        let s = parse_stub(APCU_STUB).expect("stub parses");
        let store = s.functions.iter().find(|f| f.name == "apcu_store").unwrap();
        let value = store.params.iter().find(|p| p.name == "value").unwrap();
        assert_eq!(value.default, StubDefault::Unknown);
        assert!(s.caveats.iter().any(|c| c.contains("UNKNOWN")));
    }

    #[test]
    fn detects_by_reference_parameters() {
        let s = parse_stub(APCU_STUB).expect("stub parses");
        let inc = s.functions.iter().find(|f| f.name == "apcu_inc").unwrap();
        let success = inc.params.iter().find(|p| p.name == "success").unwrap();
        assert!(success.by_ref, "&$success must be recognised as by-reference");
    }

    #[test]
    fn always_warns_that_a_stub_is_not_the_whole_truth() {
        let s = parse_stub("<?php function f(): void {}").expect("parses");
        assert!(
            s.caveats.iter().any(|c| c.contains("symbol table")),
            "the surface must always carry the caveat that C-side registrations are invisible"
        );
    }
}

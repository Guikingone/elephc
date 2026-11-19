//! Purpose:
//! Splits the emitted user assembly into the program unit and the compiler-provided catalog unit.
//!
//! Called from:
//! - `crate::codegen::finalize_user_asm()` before the visibility pass, so each unit gets the
//!   directives covering the symbols it declares.
//!
//! Key details:
//! - The candidate catalog is every function the compiler emits without a user source
//!   declaration: builtins, their helpers, and the opaque-`eval` wrappers. They do not depend on
//!   the user's program, which is what will let one unit serve many compilations.
//! - Two objects exchange symbols only through the global symbol table, and the generator marks
//!   most function-level labels `.globl` but leaves shared helpers local. `promote_crossing_symbols`
//!   therefore declares exactly the symbols a moved span needs — and the symbols it must expose —
//!   as global, so the existing visibility pass turns them into `.hidden`/`.private_extern`.
//! - Mach-O drops `L`-prefixed labels from the symbol table whatever their declared binding, so a
//!   span that reaches an `L` label across the boundary cannot move and stays in the program unit.
//! - The remainder of the assembly (prelude, `.data`, runtime metadata) stays in the program unit.

use std::collections::HashSet;
use std::ops::Range;

/// One split result: the two units and how many functions left the program unit.
pub(crate) struct CatalogSplit {
    /// Assembly that stays in the program object.
    pub(crate) program: String,
    /// Assembly of the catalog object, empty when nothing could move.
    pub(crate) catalog: String,
    /// Number of function spans that moved.
    pub(crate) moved_functions: usize,
}

/// Splits `asm` into the program text and the catalog text described by `spans`.
///
/// Returns the program unit first and the catalog unit second. The catalog unit is empty when
/// nothing can move, which is what a program without compiler-internal functions produces.
pub(crate) fn split_catalog_unit(asm: &str, spans: &[Range<usize>]) -> CatalogSplit {
    if spans.is_empty() {
        return CatalogSplit {
            program: asm.to_string(),
            catalog: String::new(),
            moved_functions: 0,
        };
    }
    let symbols = scan_symbols(asm, spans);
    let selected = select_splittable(&symbols, spans.len());
    if !selected.iter().any(|keep| *keep) {
        return CatalogSplit {
            program: asm.to_string(),
            catalog: String::new(),
            moved_functions: 0,
        };
    }
    let (mut program, catalog, moved) = slice_spans(asm, spans, &selected);
    let (program_promotions, catalog_promotions) = crossing_symbols(&symbols, &selected);
    program = promote_crossing_symbols(&program, &program_promotions);
    // The generator opens the file's text section once and then emits functions into it, so a
    // unit made only of function bodies must open its own: without this the catalog code inherits
    // whatever section the previous text left behind (the data section, for a recombined file).
    let catalog = format!(".text\n{}", promote_crossing_symbols(&catalog, &catalog_promotions));
    CatalogSplit {
        program,
        catalog,
        moved_functions: moved,
    }
}

/// Returns the subset of spans that can move, as a keep mask.
///
/// A span moves unless it depends on an `L` label defined outside the moving set, because those
/// labels never reach the symbol table of a second object.
fn select_splittable(symbols: &SymbolScan, span_count: usize) -> Vec<bool> {
    let mut kept = vec![true; span_count];
    loop {
        let mut movable: HashSet<&str> = HashSet::new();
        for (index, keep) in kept.iter().enumerate() {
            if *keep {
                movable.extend(symbols.defs[index].iter().map(String::as_str));
            }
        }
        let mut changed = false;
        for index in 0..span_count {
            if !kept[index] {
                continue;
            }
            let unresolved = symbols.refs[index].iter().any(|symbol| {
                is_linker_local(symbol)
                    && !movable.contains(symbol.as_str())
                    && !symbols.globals.contains(symbol)
            });
            if unresolved {
                kept[index] = false;
                changed = true;
            }
        }
        if !changed {
            return kept;
        }
    }
}

/// Returns the symbols each unit must declare global so the other can reach them.
///
/// The first set belongs to the program unit (referenced from a moved span), the second to the
/// catalog unit (defined in a moved span and referenced from outside it).
fn crossing_symbols(symbols: &SymbolScan, kept: &[bool]) -> (Vec<String>, Vec<String>) {
    let mut catalog_refs: HashSet<&str> = HashSet::new();
    let mut catalog_defs: HashSet<&str> = HashSet::new();
    for (index, keep) in kept.iter().enumerate() {
        if *keep {
            catalog_refs.extend(symbols.refs[index].iter().map(String::as_str));
            catalog_defs.extend(symbols.defs[index].iter().map(String::as_str));
        }
    }
    let mut program_promotions: HashSet<&str> = HashSet::new();
    for symbol in &catalog_refs {
        if !symbols.globals.contains(*symbol) && !catalog_defs.contains(symbol) {
            program_promotions.insert(symbol);
        }
    }
    let mut escapes: HashSet<&str> = symbols.outside_refs.iter().map(String::as_str).collect();
    for (index, keep) in kept.iter().enumerate() {
        if !keep {
            escapes.extend(symbols.refs[index].iter().map(String::as_str));
        }
    }
    let mut catalog_promotions: HashSet<&str> = HashSet::new();
    for symbol in &catalog_defs {
        if !symbols.globals.contains(*symbol) && escapes.contains(symbol) {
            catalog_promotions.insert(symbol);
        }
    }
    let mut program: Vec<String> = program_promotions.into_iter().map(str::to_string).collect();
    let mut catalog: Vec<String> = catalog_promotions.into_iter().map(str::to_string).collect();
    program.sort();
    catalog.sort();
    (program, catalog)
}

/// Prepends `.globl` declarations for `symbols` that the unit does not already declare.
fn promote_crossing_symbols(asm: &str, symbols: &[String]) -> String {
    if symbols.is_empty() {
        return asm.to_string();
    }
    let declared: HashSet<&str> = asm
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(".globl "))
        .map(str::trim)
        .collect();
    let promoted: Vec<&String> = symbols
        .iter()
        .filter(|symbol| !declared.contains(symbol.as_str()))
        .collect();
    if promoted.is_empty() {
        return asm.to_string();
    }
    let mut out = String::with_capacity(asm.len() + promoted.len() * 24);
    out.push_str("// -- catalog linkage: symbols reachable across the two objects --\n");
    for symbol in promoted {
        out.push_str(".globl ");
        out.push_str(symbol);
        out.push('\n');
    }
    out.push_str(asm);
    out
}

/// Removes the kept spans from `asm`, returning the program text, the catalog text, and the count.
fn slice_spans(asm: &str, spans: &[Range<usize>], kept: &[bool]) -> (String, String, usize) {
    let mut program = String::with_capacity(asm.len());
    let mut catalog = String::new();
    let mut cursor = 0usize;
    let mut moved = 0usize;
    for (span, keep) in spans.iter().zip(kept) {
        if !keep {
            continue;
        }
        let start = span.start.min(asm.len());
        let end = span.end.min(asm.len());
        // Overlapping or inverted spans would duplicate or drop text; the recording side never
        // produces them, and skipping keeps a bad range from corrupting the split.
        if start < cursor || end < start || !asm.is_char_boundary(start) || !asm.is_char_boundary(end)
        {
            continue;
        }
        program.push_str(&asm[cursor..start]);
        catalog.push_str(&asm[start..end]);
        if !catalog.ends_with('\n') {
            catalog.push('\n');
        }
        cursor = end;
        moved += 1;
    }
    program.push_str(&asm[cursor..]);
    (program, catalog, moved)
}

/// Returns whether the assembler keeps `symbol` out of the object's symbol table.
///
/// Mach-O treats a leading `L` as an assembler-local label: it is discarded at assembly time, so
/// no `.globl` declaration can make it visible to another object.
fn is_linker_local(symbol: &str) -> bool {
    symbol.starts_with('L') || symbol.starts_with(".L")
}

/// Symbol facts about one emission, indexed by span.
#[derive(Default)]
struct SymbolScan {
    /// Global symbols declared with `.globl` or `.comm`.
    globals: HashSet<String>,
    /// Symbols defined inside each span.
    defs: Vec<HashSet<String>>,
    /// Symbols referenced from inside each span.
    refs: Vec<HashSet<String>>,
    /// Symbols referenced from outside every span.
    outside_refs: HashSet<String>,
}

/// Collects the global, defined, and referenced symbols of an assembly text.
fn scan_symbols(asm: &str, spans: &[Range<usize>]) -> SymbolScan {
    let mut scan = SymbolScan {
        defs: vec![HashSet::new(); spans.len()],
        refs: vec![HashSet::new(); spans.len()],
        ..SymbolScan::default()
    };
    let mut ordered: Vec<(Range<usize>, usize)> = spans
        .iter()
        .enumerate()
        .map(|(index, span)| (span.clone(), index))
        .collect();
    ordered.sort_by_key(|(span, _)| span.start);
    // References are collected first and escapes derived afterwards, because a symbol's escape
    // status depends on references that appear later in the text.
    let mut outside_refs: HashSet<String> = HashSet::new();
    let mut line_start = 0usize;
    for line in asm.split_inclusive('\n') {
        let span = span_at(&ordered, line_start);
        line_start += line.len();
        let trimmed = line.trim_start();
        let code = match trimmed.find("//") {
            Some(index) => &trimmed[..index],
            None => trimmed,
        };
        let code = code.trim_end();
        if code.is_empty() {
            continue;
        }
        if let Some(rest) = code.strip_prefix(".globl ") {
            scan.globals.insert(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = code.strip_prefix(".comm ") {
            if let Some(symbol) = rest.split(',').next() {
                scan.globals.insert(symbol.trim().to_string());
            }
            continue;
        }
        // A definition owns the line's leading `name:`; anything after it is an instruction.
        let instruction = match split_definition(code) {
            Some((Some(symbol), rest)) => {
                if let Some(index) = span {
                    scan.defs[index].insert(symbol);
                }
                rest
            }
            Some((None, rest)) => rest,
            None => code,
        };
        for symbol in referenced_symbols(instruction) {
            match span {
                Some(index) => {
                    scan.refs[index].insert(symbol);
                }
                None => {
                    outside_refs.insert(symbol);
                }
            }
        }
    }
    scan.outside_refs = outside_refs;
    scan
}

/// Returns the index of the span containing `offset`, if any.
fn span_at(spans: &[(Range<usize>, usize)], offset: usize) -> Option<usize> {
    spans
        .iter()
        .find(|(span, _)| offset >= span.start && offset < span.end)
        .map(|(_, index)| *index)
}

/// Splits a `name: rest` line into its definition and the instruction that follows.
fn split_definition(code: &str) -> Option<(Option<String>, &str)> {
    let colon = code.find(':')?;
    let candidate = &code[..colon];
    if candidate.is_empty() || candidate.contains(|c: char| c.is_whitespace()) {
        return None;
    }
    Some((Some(candidate.to_string()), code[colon + 1..].trim()))
}

/// Returns the symbols a single instruction or data line refers to.
///
/// The generator's symbol tokens start with `_` or an uppercase `L`; registers and mnemonics never
/// do, so the filter does not need an instruction table.
fn referenced_symbols(code: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for token in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$'))
    {
        if token.is_empty() {
            continue;
        }
        let is_symbol = token.starts_with('_') || (token.starts_with('L') && token.len() > 1);
        if is_symbol {
            symbols.push(token.split('@').next().unwrap_or(token).to_string());
        }
    }
    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that one span moves exactly its slice into the catalog unit.
    #[test]
    fn moves_one_function_out_of_the_program_unit() {
        let asm = "prelude\n.globl _catalog\n_catalog:\n    nop\nepilogue\n";
        let start = asm.find(".globl _catalog").expect("catalog label");
        let end = start + ".globl _catalog\n_catalog:\n    nop\n".len();
        let split = split_catalog_unit(asm, &[start..end]);
        assert_eq!(split.catalog, ".text\n.globl _catalog\n_catalog:\n    nop\n");
        assert_eq!(split.program, "prelude\nepilogue\n");
        assert_eq!(split.moved_functions, 1);
    }

    /// Verifies that contiguous spans leave nothing behind in the program unit.
    #[test]
    fn joins_adjacent_spans_without_glue() {
        let asm = ".globl _a\n_a:\n.globl _b\n_b:\nC\n";
        let split = split_catalog_unit(asm, &[0..14, 14..28]);
        assert_eq!(split.catalog, ".text\n.globl _a\n_a:\n.globl _b\n_b:\n");
        assert_eq!(split.program, "C\n");
        assert_eq!(split.moved_functions, 2);
    }

    /// Verifies that an empty span list leaves the assembly untouched.
    #[test]
    fn empty_spans_keep_one_unit() {
        let asm = "only\n";
        let split = split_catalog_unit(asm, &[]);
        assert_eq!(split.program, asm);
        assert!(split.catalog.is_empty());
    }

    /// Verifies that a span reaching past the text is clamped instead of panicking.
    #[test]
    fn out_of_range_span_is_clamped() {
        let asm = "tail\n";
        let split = split_catalog_unit(asm, &[0..99]);
        assert_eq!(split.catalog, ".text\ntail\n");
        assert!(split.program.is_empty());
    }

    /// A span that reaches a local `L` label of the program must stay in the program unit.
    #[test]
    fn keeps_spans_reaching_program_local_labels() {
        let asm = "L_local:\n    ret\n.globl _inner\n_inner:\n    bl L_local\n";
        let start = asm.find(".globl _inner").expect("start");
        let split = split_catalog_unit(asm, &[start..asm.len()]);
        assert!(split.catalog.is_empty(), "an L label cannot cross objects");
        assert_eq!(split.program, asm);
    }

    /// A span reaching an ordinary local helper promotes it instead of staying behind.
    #[test]
    fn promotes_local_helpers_reached_from_the_catalog() {
        let asm = "_helper:\n    ret\n.globl _inner\n_inner:\n    bl _helper\n";
        let start = asm.find(".globl _inner").expect("start");
        let split = split_catalog_unit(asm, &[start..asm.len()]);
        assert_eq!(split.catalog, ".text\n.globl _inner\n_inner:\n    bl _helper\n");
        assert!(
            split.program.contains(".globl _helper\n"),
            "the program unit must expose the helper: {}",
            split.program
        );
        assert!(split.program.ends_with("_helper:\n    ret\n"));
    }

    /// A local helper defined in the catalog and used by the program is promoted there.
    #[test]
    fn promotes_catalog_helpers_used_by_the_program() {
        let asm = ".globl _inner\n_inner:\n    ret\n_helper:\n    ret\nprogram:\n    bl _helper\n";
        let start = asm.find(".globl _inner").expect("start");
        let end = start + ".globl _inner\n_inner:\n    ret\n_helper:\n    ret\n".len();
        let split = split_catalog_unit(asm, &[start..end]);
        assert!(
            split.catalog.contains(".globl _helper\n"),
            "the catalog unit must expose the helper: {}",
            split.catalog
        );
        assert_eq!(split.program, "program:\n    bl _helper\n");
    }
}

//! Purpose:
//! Cuts one finished user assembly file into N CONTIGUOUS slices that assemble
//! independently, so `as` can run N times in parallel instead of once on a
//! single multi-hundred-megabyte input.
//!
//! Called from:
//! - `crate::linker::assemble_parallel()`, on the exact bytes already written to
//!   `<stem>.s`. Never on the emitter's intermediate state.
//!
//! Key details:
//! - The split is a POST-PASS over the finished text, not an emitter change. It has to be:
//!   `codegen::aarch64_relax::relax_conditional_branches` (src/codegen/aarch64_relax.rs:30-71)
//!   decides whether a conditional branch needs an inverse-condition island from the SOURCE
//!   BYTE DISTANCE to its target in the whole file, so splitting before it would change which
//!   branches are relaxed. Splitting after it cannot: every slice is a byte range of the exact
//!   string that pass produced.
//! - Slices are contiguous ranges of the emission order and are handed to `ld` in that order.
//!   They are NEVER reordered or bin-packed by size: emitted `__data` records are adjacent by
//!   construction, and a size-greedy repack links fine and dies at startup.
//! - `jobs <= 1` returns the input unchanged, byte for byte. That is the identity gate.

use std::collections::{HashMap, HashSet};

use crate::codegen::platform::{AppleVariant, Arch, Platform, Target};
use crate::codegen::Emit;

/// Header line `codegen_support::visibility::append_visibility_directives`
/// (src/codegen_support/visibility.rs:87) writes before the FILE-GLOBAL block of
/// `.private_extern` directives it appends to every finished user object.
///
/// That block names every `.globl`/`.comm` symbol in the file except the export
/// allowlist, so it cannot be copied verbatim into a slice that does not define
/// those symbols: a `.private_extern` naming a symbol the object does not define
/// is a private external reference with no definition. It is therefore stripped
/// from the body and re-emitted per slice over exactly the symbols that slice
/// defines.
const VISIBILITY_FOOTER_MARKER: &str =
    "// -- internal symbols are local to the cdylib public ABI --";

/// Prefix given to an assembler temporary that must become a real symbol because
/// the split put its definition and one of its references in different slices.
///
/// A Mach-O `L`-prefixed name is an assembler temporary: it never reaches the
/// object's symbol table, so a reference to it from another object would be an
/// undefined temporary, and `.globl`-ing one is rejected outright ("non-local
/// symbol required" — the error `Emitter::label_global` already asserts against,
/// src/codegen_support/emit.rs:145-151). The only way to let such a reference
/// cross a slice boundary is to rename the label to a non-temporary name in
/// EVERY slice and publish it `.private_extern` in the defining one. The prefix
/// is deliberately unlike anything the emitter mints, so the rename cannot
/// collide, and greppable, so a symbolicated profile explains itself.
const PROMOTED_PREFIX: &str = "_elephc_xslice_";

/// Result of one split: the slice texts plus what had to be published to make them link.
pub(super) struct SplitOutcome {
    /// Slice bodies in emission order. Exactly one entry when the input was not split.
    pub(super) slices: Vec<String>,
    /// Assembler temporaries renamed because their span crossed a cut.
    pub(super) promoted: Vec<String>,
    /// Plain (non-`.globl`) labels published `.private_extern` because another
    /// slice references them.
    pub(super) published: usize,
}

impl SplitOutcome {
    /// Returns the untouched input as a single slice.
    fn unsplit(source: &str) -> Self {
        Self {
            slices: vec![source.to_string()],
            promoted: Vec::new(),
            published: 0,
        }
    }

    /// Returns whether the input was actually cut.
    pub(super) fn is_split(&self) -> bool {
        self.slices.len() > 1
    }
}

/// Returns whether this target/artifact pair is one the splitter is implemented for.
///
/// macOS AArch64 executables only. The measured cost is the single blocking `as`
/// on the macOS user object, and every hazard below was established against the
/// Mach-O assembler: ELF splits text per symbol (`Emitter::label_global`,
/// src/codegen_support/emit.rs:152-157) and would need its own `.section` replay
/// and its own `.L` locality rules, and a `Staticlib` goes through `ar` with a
/// fixed two-member argument list rather than through the link line. Every other
/// combination keeps today's exact single-`as` path.
pub(super) fn supports_split(target: Target, emit: Emit) -> bool {
    target.platform == Platform::MacOS
        && target.arch == Arch::AArch64
        && target.apple_variant == AppleVariant::MacOS
        && emit == Emit::Executable
}

/// One label definition and the span of lines that mention it.
#[derive(Clone, Copy)]
struct LabelSpan {
    /// Line index of the defining `name:` (or `.comm name, …`).
    def: usize,
    /// Lowest line index that defines or references the name.
    lo: usize,
    /// Highest line index that defines or references the name.
    hi: usize,
}

/// Everything one scan of the body hands to cut selection and rendering.
struct Analysis<'a> {
    /// Body lines, `\n` included, in emission order. Concatenating them reproduces the body.
    lines: Vec<&'a str>,
    /// Byte offset of each line plus a final total, so cuts can be balanced by size.
    offsets: Vec<usize>,
    /// Names in first-definition order; the index into this vector is a label id.
    names: Vec<&'a str>,
    /// Label id per name.
    ids: HashMap<&'a str, usize>,
    /// Span per label id, parallel to `names`.
    spans: Vec<LabelSpan>,
    /// Line indices a slice may start at, ascending.
    candidates: Vec<usize>,
    /// `(line, directive)` for every section change, ascending by line.
    sections: Vec<(usize, &'a str)>,
    /// File-global directives replayed at the top of every later slice.
    file_prologue: Vec<&'a str>,
    /// Names the file-global visibility footer marks, in footer order.
    footer: Vec<&'a str>,
    /// The same names, for membership tests.
    footer_set: HashSet<&'a str>,
    /// Names declared `.globl` anywhere in the body.
    globl: HashSet<&'a str>,
}

/// Splits one finished assembly text into at most `jobs` contiguous slices.
///
/// `jobs <= 1`, an input with no legal cut, a promotion name collision, and a
/// visibility footer naming a symbol the body does not define all return the
/// input as a single slice, so the caller's fallback is always "do exactly what
/// we do today".
pub(super) fn split_assembly(source: &str, jobs: usize, local_prefix: &str) -> SplitOutcome {
    if jobs <= 1 || source.is_empty() {
        return SplitOutcome::unsplit(source);
    }
    let analysis = analyze(source, local_prefix);
    if analysis.footer.iter().any(|name| !analysis.ids.contains_key(name)) {
        // The footer is authoritative about visibility. If a name in it has no
        // definition this pass can find, partitioning it would silently drop a
        // directive; refuse instead.
        return SplitOutcome::unsplit(source);
    }
    let cuts = choose_cuts(&analysis, jobs);
    if cuts.is_empty() {
        return SplitOutcome::unsplit(source);
    }
    render(source, &analysis, &cuts, local_prefix)
}

/// Runs the single body scan that every later decision reads.
fn analyze<'a>(source: &'a str, local_prefix: &str) -> Analysis<'a> {
    let all: Vec<&str> = source.split_inclusive('\n').collect();
    let body_end = footer_start(&all);
    let footer = footer_names(&all[body_end..]);
    let footer_set: HashSet<&str> = footer.iter().copied().collect();
    let lines: Vec<&str> = all[..body_end].to_vec();

    let mut offsets = Vec::with_capacity(lines.len() + 1);
    let mut total = 0usize;
    for line in &lines {
        offsets.push(total);
        total += line.len();
    }
    offsets.push(total);

    let mut names: Vec<&str> = Vec::new();
    let mut ids: HashMap<&str, usize> = HashMap::new();
    let mut spans: Vec<LabelSpan> = Vec::new();
    let mut sections: Vec<(usize, &str)> = Vec::new();
    let mut file_prologue: Vec<&str> = Vec::new();
    let mut globl: HashSet<&str> = HashSet::new();
    let mut numeric: HashMap<&str, Vec<usize>> = HashMap::new();

    for (index, raw) in lines.iter().copied().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix(".globl ") {
            globl.insert(rest.trim());
            continue;
        }
        if let Some(rest) = line.strip_prefix(".comm ") {
            let name = rest.split(',').next().unwrap_or("").trim();
            define(name, index, &mut names, &mut ids, &mut spans);
            continue;
        }
        if let Some(name) = line.strip_suffix(':') {
            if is_label_name(name) {
                if name.bytes().all(|byte| byte.is_ascii_digit()) {
                    numeric.entry(name).or_default().push(index);
                } else {
                    define(name, index, &mut names, &mut ids, &mut spans);
                }
                continue;
            }
        }
        if is_section_directive(line) {
            sections.push((index, line));
            continue;
        }
        if line == ".intel_syntax noprefix" && !file_prologue.contains(&line) {
            file_prologue.push(line);
        }
    }

    let mut hard: Vec<(usize, usize)> = Vec::new();
    for (index, raw) in lines.iter().copied().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let is_definition = line.strip_suffix(':').is_some_and(is_label_name);
        if !is_definition
            && !line.starts_with(".globl ")
            && !line.starts_with(".private_extern ")
            && !line.starts_with(".comm ")
        {
            visit_identifiers(raw, |token| {
                if let Some(&id) = ids.get(token) {
                    let span = &mut spans[id];
                    span.lo = span.lo.min(index);
                    span.hi = span.hi.max(index);
                }
            });
        }
        // HAZARD 1 — `as` refuses a conditional branch to an external symbol
        // ("conditional branch requires assembler-local label"). Branch and target
        // therefore have to stay in the same slice; the pair is a hard interval and
        // no cut may fall inside it. Rewriting `b.cond t` as `b.!cond skip; b t` is
        // the alternative and is deliberately not taken: keeping the pair together
        // costs nothing, because every such pair is intra-function, and rewriting
        // would change instruction counts and therefore later branch distances.
        if let Some(target) = conditional_branch_target(line) {
            if let Some(&id) = ids.get(target) {
                let def = spans[id].def;
                hard.push((index.min(def), index.max(def)));
            }
        }
        // A numeric local (`1:`) is resolved by direction, so `1f`/`1b` reaches
        // the nearest definition and can never leave the object it was written in.
        for (position, direction) in numeric_references(raw) {
            let Some(definitions) = numeric.get(position) else {
                continue;
            };
            if direction == b'f' {
                if let Some(&next) = definitions.iter().find(|&&line| line >= index) {
                    hard.push((index, next));
                }
            } else if let Some(&previous) = definitions.iter().rev().find(|&&line| line <= index) {
                hard.push((previous, index));
            }
        }
    }
    hard.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(hard.len());
    for (lo, hi) in hard {
        let overlaps = merged.last().is_some_and(|&(_, last_hi)| lo <= last_hi);
        if overlaps {
            let last = merged.last_mut().expect("checked non-empty above");
            last.1 = last.1.max(hi);
        } else {
            merged.push((lo, hi));
        }
    }

    let candidates = boundary_candidates(&lines, local_prefix, &merged);
    Analysis {
        lines,
        offsets,
        names,
        ids,
        spans,
        candidates,
        sections,
        file_prologue,
        footer,
        footer_set,
        globl,
    }
}

/// Records a first definition of `name` at `index`.
fn define<'a>(
    name: &'a str,
    index: usize,
    names: &mut Vec<&'a str>,
    ids: &mut HashMap<&'a str, usize>,
    spans: &mut Vec<LabelSpan>,
) {
    if name.is_empty() || ids.contains_key(name) {
        return;
    }
    ids.insert(name, names.len());
    names.push(name);
    spans.push(LabelSpan {
        def: index,
        lo: index,
        hi: index,
    });
}

/// Returns the line index at which the file-global visibility footer starts.
fn footer_start(lines: &[&str]) -> usize {
    lines
        .iter()
        .rposition(|line| line.trim() == VISIBILITY_FOOTER_MARKER)
        .unwrap_or(lines.len())
}

/// Returns the symbols named by a visibility footer, in footer order.
fn footer_names<'a>(footer: &[&'a str]) -> Vec<&'a str> {
    footer
        .iter()
        .copied()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix(".private_extern ")
                .or_else(|| line.strip_prefix(".hidden "))
                .map(str::trim)
        })
        .filter(|name| !name.is_empty())
        .collect()
}

/// Returns whether a name is a plain assembly label identifier.
fn is_label_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
}

/// Returns whether a directive changes the current section.
fn is_section_directive(line: &str) -> bool {
    line == ".text"
        || line == ".data"
        || line == ".bss"
        || line == ".const"
        || line == ".cstring"
        || line.starts_with(".section")
}

/// Calls `visit` with every identifier token on one line, skipping quoted strings.
///
/// Quote handling matches `codegen_support::emit::localize_internal_labels`
/// (src/codegen_support/emit.rs:374-420): a quoted run is passed over verbatim
/// and an unmatched quote is bounded to its physical line, so a user string
/// constant can neither be read as a symbol nor hide the rest of the file.
fn visit_identifiers<'a>(line: &'a str, mut visit: impl FnMut(&'a str)) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            index += 1;
            let mut escaped = false;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if byte == b'\n' || (!escaped && byte == b'"') {
                    break;
                }
                escaped = !escaped && byte == b'\\';
            }
        } else if is_identifier_start(byte) {
            let start = index;
            while index < bytes.len() && is_identifier_byte(bytes[index]) {
                index += 1;
            }
            visit(&line[start..index]);
        } else {
            index += 1;
        }
    }
}

/// Returns whether a byte may begin an identifier token.
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$' | b'.')
}

/// Returns whether a byte may continue an identifier token.
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.')
}

/// Returns the symbolic target of an AArch64 conditional branch, if the line is one.
///
/// Mirrors the shapes `codegen::aarch64_relax::parse_conditional_branch`
/// (src/codegen/aarch64_relax.rs:74-126) recognises, because those are exactly
/// the instructions whose target `as` requires to be assembler-local.
fn conditional_branch_target(line: &str) -> Option<&str> {
    let (mnemonic, operands) = line.split_once(char::is_whitespace)?;
    let conditional =
        mnemonic.starts_with("b.") || matches!(mnemonic, "cbz" | "cbnz" | "tbz" | "tbnz");
    if !conditional {
        return None;
    }
    let target = operands.rsplit(',').next()?.trim();
    is_label_name(target).then_some(target)
}

/// Returns `(position, b'b' | b'f')` for every numeric local reference on a line.
fn numeric_references(line: &str) -> Vec<(&str, u8)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() || (index > 0 && is_identifier_byte(bytes[index - 1])) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let direction = match bytes.get(index).copied() {
            Some(b'b') => b'b',
            Some(b'f') => b'f',
            _ => continue,
        };
        if bytes.get(index + 1).copied().is_some_and(is_identifier_byte) {
            continue;
        }
        out.push((&line[start..index], direction));
        index += 1;
    }
    out
}

/// Returns every line index a slice may legally start at, ascending.
///
/// Two conditions, both conservative:
/// * the line opens a top-level non-temporary label, and control cannot fall
///   through into it — the previous meaningful line is an unconditional transfer
///   (`ret`, `b`, `br`, `brk`) or data. Two slices become two objects that `ld`
///   concatenates with section padding between them, so a fall-through across a
///   cut would run into the padding;
/// * no hard interval (a conditional branch pair, a numeric local's `1b`/`1f`
///   reach) spans the cut.
///
/// The `.align`/`.globl`/comment lines that introduce the label are pulled into
/// the following slice with it, so the label keeps its alignment and declaration.
fn boundary_candidates(lines: &[&str], local_prefix: &str, hard: &[(usize, usize)]) -> Vec<usize> {
    let mut candidates: Vec<usize> = Vec::new();
    for (index, raw) in lines.iter().copied().enumerate() {
        if raw.starts_with(' ') || raw.starts_with('\t') {
            continue;
        }
        let Some(name) = raw.trim().strip_suffix(':') else {
            continue;
        };
        if !is_label_name(name)
            || name.starts_with(local_prefix)
            || name.bytes().all(|byte| byte.is_ascii_digit())
        {
            continue;
        }
        if !cannot_fall_through(lines, index) {
            continue;
        }
        let mut start = index;
        while start > 0 && introduces_a_label(lines[start - 1].trim()) {
            start -= 1;
        }
        if start == 0 || inside_hard_interval(hard, start) {
            continue;
        }
        if candidates.last() != Some(&start) {
            candidates.push(start);
        }
    }
    candidates
}

/// Returns whether control cannot reach the label defined at `index` by falling through.
fn cannot_fall_through(lines: &[&str], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let line = lines[cursor].trim();
        if line.is_empty() || introduces_a_label(line) {
            continue;
        }
        let mnemonic = line.split_whitespace().next().unwrap_or("");
        return matches!(mnemonic, "ret" | "b" | "br" | "brk")
            || line.starts_with(".quad")
            || line.starts_with(".long")
            || line.starts_with(".short")
            || line.starts_with(".byte")
            || line.starts_with(".ascii")
            || line.starts_with(".space")
            || line.starts_with(".comm")
            || is_section_directive(line);
    }
    true
}

/// Returns whether a line only introduces the label that follows it.
fn introduces_a_label(line: &str) -> bool {
    line.is_empty()
        || line.starts_with(".align")
        || line.starts_with(".p2align")
        || line.starts_with(".globl")
        || line.starts_with(".private_extern")
        || line.starts_with(';')
        || line.starts_with("//")
        || line.starts_with('#')
}

/// Returns whether `line` falls strictly inside a merged hard interval.
fn inside_hard_interval(hard: &[(usize, usize)], line: usize) -> bool {
    match hard.binary_search_by(|probe| probe.0.cmp(&line)) {
        Ok(_) | Err(0) => false,
        Err(position) => {
            let (lo, hi) = hard[position - 1];
            lo < line && line <= hi
        }
    }
}

/// Picks up to `jobs - 1` cut lines, each as close as the legal boundaries allow
/// to an equal split of the body's bytes.
fn choose_cuts(analysis: &Analysis<'_>, jobs: usize) -> Vec<usize> {
    let total = *analysis.offsets.last().unwrap_or(&0);
    if total == 0 || analysis.candidates.is_empty() {
        return Vec::new();
    }
    let mut cuts: Vec<usize> = Vec::new();
    for part in 1..jobs {
        let ideal = total * part / jobs;
        let mut best: Option<usize> = None;
        for &candidate in &analysis.candidates {
            if cuts.last().is_some_and(|&last| candidate <= last) {
                continue;
            }
            let distance = analysis.offsets[candidate].abs_diff(ideal);
            let keep = best.is_some_and(|current| {
                analysis.offsets[current].abs_diff(ideal) <= distance
            });
            if !keep {
                best = Some(candidate);
            }
        }
        if let Some(candidate) = best {
            cuts.push(candidate);
        }
    }
    cuts
}

/// Builds the slice texts, the rename table, and the per-slice visibility footers.
fn render<'a>(
    source: &str,
    analysis: &Analysis<'a>,
    cuts: &[usize],
    local_prefix: &str,
) -> SplitOutcome {
    let slice_of = |line: usize| cuts.partition_point(|&cut| cut <= line);
    let parts = cuts.len() + 1;

    // HAZARD 2 — a top-level symbol the emitter wrote as a plain `name:` is a
    // Mach-O local: it is not in the object's symbol table, so a reference to it
    // from another slice would not resolve. Every such label that IS referenced
    // from another slice is published `.private_extern` in its defining slice —
    // external enough for the linker to bind, still not an export, therefore
    // still not a `-dead_strip` root (src/codegen_support/visibility.rs:18-21).
    let mut promoted: HashMap<&'a str, String> = HashMap::new();
    let mut promoted_names: Vec<String> = Vec::new();
    let mut published: Vec<Vec<&'a str>> = vec![Vec::new(); parts];
    let mut published_count = 0usize;
    for (id, &name) in analysis.names.iter().enumerate() {
        let span = analysis.spans[id];
        if slice_of(span.lo) == slice_of(span.hi) {
            continue;
        }
        if name.starts_with(local_prefix) {
            let renamed = format!("{PROMOTED_PREFIX}{name}");
            if analysis.ids.contains_key(renamed.as_str()) {
                // A collision would silently merge two symbols. Refuse the split
                // rather than guess a second name.
                return SplitOutcome::unsplit(source);
            }
            published[slice_of(span.def)].push(name);
            promoted.insert(name, renamed.clone());
            promoted_names.push(renamed);
            continue;
        }
        if analysis.footer_set.contains(name) || analysis.globl.contains(name) {
            continue;
        }
        published[slice_of(span.def)].push(name);
        published_count += 1;
    }

    let mut slices: Vec<String> = Vec::with_capacity(parts);
    for part in 0..parts {
        let start = if part == 0 { 0 } else { cuts[part - 1] };
        let end = cuts.get(part).copied().unwrap_or(analysis.lines.len());
        let bytes = analysis.offsets[end] - analysis.offsets[start];
        let mut out = String::with_capacity(bytes + 4096);
        if part > 0 {
            // HAZARD 4 — `.section`/`.data` state and the `.intel_syntax` switch
            // are file-global. A slice that starts mid-`.data` would otherwise
            // assemble its records into `__text`.
            for directive in &analysis.file_prologue {
                out.push_str(directive);
                out.push('\n');
            }
            let section = section_in_effect(&analysis.sections, start);
            out.push_str(section);
            out.push('\n');
            // HAZARD 3 — a `.quad` record whose own alignment directive sits in
            // the previous slice starts the new object unaligned, and `ld`
            // rejects the object outright ("pointer not aligned"). Restate the
            // section's alignment unless the slice already opens with one.
            let opening = analysis.lines[start].trim_start();
            if !opening.starts_with(".align") && !opening.starts_with(".p2align") {
                out.push_str(if section == ".text" {
                    ".p2align 2\n"
                } else {
                    ".p2align 3\n"
                });
            }
        }
        for line in analysis.lines[start..end].iter().copied() {
            if !promoted.is_empty() && line.contains(local_prefix) {
                push_renamed(&mut out, line, &promoted);
            } else {
                out.push_str(line);
            }
        }
        append_visibility_footer(&mut out, analysis, &published[part], &promoted, part, &slice_of);
        slices.push(out);
    }
    promoted_names.sort();
    SplitOutcome {
        slices,
        promoted: promoted_names,
        published: published_count,
    }
}

/// Appends this slice's share of the file-global visibility footer.
fn append_visibility_footer<'a>(
    out: &mut String,
    analysis: &Analysis<'a>,
    published: &[&'a str],
    promoted: &HashMap<&'a str, String>,
    part: usize,
    slice_of: &impl Fn(usize) -> usize,
) {
    let mut written: HashSet<&'a str> = HashSet::new();
    let mut block = String::new();
    for &name in analysis.footer.iter().chain(published.iter()) {
        let Some(&id) = analysis.ids.get(name) else {
            continue;
        };
        if slice_of(analysis.spans[id].def) != part || !written.insert(name) {
            continue;
        }
        block.push_str(".private_extern ");
        block.push_str(promoted.get(name).map(String::as_str).unwrap_or(name));
        block.push('\n');
    }
    if block.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(VISIBILITY_FOOTER_MARKER);
    out.push('\n');
    out.push_str(&block);
}

/// Returns the section directive in effect at `line`.
fn section_in_effect<'a>(sections: &[(usize, &'a str)], line: usize) -> &'a str {
    match sections.binary_search_by(|probe| probe.0.cmp(&line)) {
        Ok(position) => sections[position].1,
        Err(0) => ".text",
        Err(position) => sections[position - 1].1,
    }
}

/// Copies one line, replacing every whole-token occurrence of a promoted name.
///
/// Quoted runs are copied verbatim for the same reason `localize_internal_labels`
/// copies them: a label name that also appears inside a user string constant must
/// not be rewritten there.
fn push_renamed(out: &mut String, line: &str, promoted: &HashMap<&str, String>) {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            let start = index;
            index += 1;
            let mut escaped = false;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if byte == b'\n' || (!escaped && byte == b'"') {
                    break;
                }
                escaped = !escaped && byte == b'\\';
            }
            out.push_str(&line[start..index]);
        } else if is_identifier_start(byte) {
            let start = index;
            while index < bytes.len() && is_identifier_byte(bytes[index]) {
                index += 1;
            }
            let token = &line[start..index];
            match promoted.get(token) {
                Some(renamed) => out.push_str(renamed),
                None => out.push_str(token),
            }
        } else {
            let start = index;
            while index < bytes.len() && bytes[index] != b'"' && !is_identifier_start(bytes[index]) {
                index += 1;
            }
            out.push_str(&line[start..index]);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for the contiguous assembly splitter.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - The fixture reproduces the four shapes the real emitter produces that a
    //!   split can break: a conditional branch to an intra-function label, a plain
    //!   top-level local referenced from another function, an `L`-prefixed function
    //!   whose address a `.data` record takes, and a data tail behind a `.data`
    //!   directive that only the file prologue establishes.

    use super::*;

    /// The Mach-O assembler-local prefix, the one the splitter is written for.
    const MACOS_LOCAL: &str = "L";

    /// Returns every identifier token on a line, for the scanner tests.
    fn identifiers(line: &str) -> Vec<&str> {
        let mut out = Vec::new();
        visit_identifiers(line, |token| out.push(token));
        out
    }

    /// A hand-written fixture shaped like real emitter output.
    ///
    /// Every shape here is one a split can break, and each is taken from the real
    /// 8.8 MB assembly this module was checked against:
    /// * `_second` branches conditionally to `L_second_retry`, so the two may never
    ///   land in different slices;
    /// * `_second_tail` is a plain, non-`.globl`, non-footer label INSIDE another
    ///   atom that `_fourth` branches to — the shape that has to be published;
    /// * `L_thunk` is an assembler temporary whose only reference is a `.quad` in
    ///   `.data` — the shape that forces a rename, because `.globl` on an
    ///   `L`-prefixed name is an assembler error;
    /// * the `.data` tail exercises the section replay and the eight-byte alignment.
    ///
    /// The visibility footer names exactly the `.globl` symbols outside the export
    /// allowlist, which is what `append_visibility_directives` produces
    /// (src/codegen_support/visibility.rs:65-78) — `_main` is the only export.
    fn fixture() -> String {
        let mut asm = String::new();
        asm.push_str(".globl _main\n_main:\n    bl _first\n    ret\n");
        asm.push_str(".align 2\n.globl _first\n_first:\n    mov x0, #1\n    ret\n");
        asm.push_str(".align 2\n.globl _second\n_second:\n    cmp x0, #0\n");
        asm.push_str("    b.eq L_second_retry\n    ret\n");
        asm.push_str("L_second_retry:\n    mov x0, #2\n_second_tail:\n    ret\n");
        asm.push_str(".align 2\n.globl _third\n_third:\n    bl _first\n    bl _second\n    ret\n");
        asm.push_str(".align 2\nL_thunk:\n    mov x0, #3\n    ret\n");
        asm.push_str(".align 2\n.globl _fourth\n_fourth:\n    bl _third\n    b _second_tail\n");
        asm.push_str(".data\n.p2align 3\n.globl _data_0\n_data_0:\n");
        asm.push_str("    .quad L_thunk\n    .quad _first\n");
        asm.push_str(".globl _data_1\n_data_1:\n    .quad 0x0000000000000001\n");
        asm.push('\n');
        asm.push_str(VISIBILITY_FOOTER_MARKER);
        asm.push('\n');
        for name in ["_first", "_second", "_third", "_fourth", "_data_0", "_data_1"] {
            asm.push_str(".private_extern ");
            asm.push_str(name);
            asm.push('\n');
        }
        asm
    }

    /// One job must return the input byte for byte: that is the whole identity gate.
    #[test]
    fn one_job_returns_the_input_unchanged() {
        let source = fixture();
        for jobs in [0, 1] {
            let outcome = split_assembly(&source, jobs, MACOS_LOCAL);
            assert_eq!(outcome.slices.len(), 1);
            assert_eq!(outcome.slices[0], source);
            assert!(outcome.promoted.is_empty());
            assert!(!outcome.is_split());
        }
    }

    /// Concatenating the sliced line ranges must reproduce the body of the input.
    ///
    /// Everything a slice adds is a prologue (section state, alignment) or an
    /// epilogue (its own visibility footer); nothing in between may be dropped,
    /// duplicated, or reordered, because a `__data` record's meaning depends on
    /// its neighbours.
    #[test]
    fn slices_cover_the_body_in_order_without_gaps() {
        let source = fixture();
        let analysis = analyze(&source, MACOS_LOCAL);
        let cuts = choose_cuts(&analysis, 4);
        assert!(!cuts.is_empty(), "the fixture must offer at least one legal cut");
        let mut rebuilt = String::new();
        for part in 0..=cuts.len() {
            let start = if part == 0 { 0 } else { cuts[part - 1] };
            let end = cuts.get(part).copied().unwrap_or(analysis.lines.len());
            for line in &analysis.lines[start..end] {
                rebuilt.push_str(line);
            }
        }
        let all: Vec<&str> = source.split_inclusive('\n').collect();
        let body: String = all[..footer_start(&all)].concat();
        assert_eq!(rebuilt, body);
    }

    /// No conditional branch and no numeric local reach may cross a slice
    /// boundary: `as` rejects the first and silently rebinds the second.
    #[test]
    fn no_hard_interval_crosses_a_slice() {
        let source = fixture();
        let analysis = analyze(&source, MACOS_LOCAL);
        let cuts = choose_cuts(&analysis, 4);
        let mut checked = 0usize;
        for (index, raw) in analysis.lines.iter().enumerate() {
            let Some(target) = conditional_branch_target(raw.trim()) else {
                continue;
            };
            let Some(&id) = analysis.ids.get(target) else {
                continue;
            };
            let def = analysis.spans[id].def;
            let (lo, hi) = (index.min(def), index.max(def));
            checked += 1;
            assert!(
                !cuts.iter().any(|&cut| lo < cut && cut <= hi),
                "conditional branch on line {index} to {target} (line {def}) crosses {cuts:?}"
            );
        }
        assert!(checked > 0, "the fixture must contain a conditional branch");
    }

    /// A local referenced from another slice must be published, and an assembler
    /// temporary must be renamed everywhere it appears rather than published
    /// under its temporary name.
    #[test]
    fn cross_slice_locals_are_published_and_temporaries_renamed() {
        let source = fixture();
        let outcome = split_assembly(&source, 4, MACOS_LOCAL);
        assert!(outcome.is_split(), "the fixture must split at four jobs");
        let joined = outcome.slices.concat();

        // `L_thunk` is reached only from the `.data` tail, so any cut between the
        // two forces the rename. `.globl L_thunk` is not an alternative: the
        // assembler rejects it ("non-local symbol required").
        let renamed = format!("{PROMOTED_PREFIX}L_thunk");
        assert!(
            outcome.promoted.iter().any(|name| name == &renamed),
            "the cross-section temporary was not promoted: {:?}",
            outcome.promoted
        );
        assert!(
            !joined.contains("\nL_thunk:"),
            "a promoted temporary kept its temporary definition:\n{joined}"
        );
        assert!(
            joined.contains(&format!(".quad {renamed}")),
            "a promoted temporary kept a temporary reference:\n{joined}"
        );
        assert!(
            joined.contains(&format!(".private_extern {renamed}")),
            "a promoted temporary was not published:\n{joined}"
        );
        // `_second_tail` is a plain local inside another atom that `_fourth`
        // branches to, so it must be published from the slice that defines it.
        assert!(
            outcome.published >= 1,
            "a plain local crossing a slice was not published"
        );
        assert!(
            outcome
                .slices
                .iter()
                .any(|slice| slice.contains(".private_extern _second_tail\n")),
            "the cross-slice plain local was not published:\n{joined}"
        );

        let analysis = analyze(&source, MACOS_LOCAL);
        let cuts = choose_cuts(&analysis, 4);
        let slice_of = |line: usize| cuts.partition_point(|&cut| cut <= line);
        for (id, name) in analysis.names.iter().enumerate() {
            let span = analysis.spans[id];
            if slice_of(span.lo) == slice_of(span.hi) || name.starts_with(MACOS_LOCAL) {
                continue;
            }
            assert!(
                outcome.slices[slice_of(span.def)].contains(&format!(".private_extern {name}\n"))
                    || *name == "_main",
                "{name} crosses a slice but is not published from slice {}",
                slice_of(span.def)
            );
        }
    }

    /// Every slice after the first must restate its section and its alignment.
    #[test]
    fn every_later_slice_restates_section_and_alignment() {
        let source = fixture();
        let outcome = split_assembly(&source, 4, MACOS_LOCAL);
        assert!(outcome.is_split());
        for (index, slice) in outcome.slices.iter().enumerate().skip(1) {
            let mut lines = slice.lines();
            let section = lines.next().unwrap_or_default();
            assert!(
                is_section_directive(section),
                "slice {index} does not open with a section directive:\n{slice}"
            );
            let alignment = lines.next().unwrap_or_default();
            assert!(
                alignment.starts_with(".p2align") || alignment.starts_with(".align"),
                "slice {index} does not restate alignment:\n{slice}"
            );
        }
        // A slice that opens in `.data` must restate eight-byte alignment, or a
        // `.quad` record whose own `.p2align` stayed behind lands misaligned.
        for slice in outcome.slices.iter().skip(1) {
            if slice.starts_with(".data\n") {
                assert!(
                    slice.starts_with(".data\n.p2align 3\n") || slice.starts_with(".data\n.align"),
                    "a data slice did not restate eight-byte alignment:\n{slice}"
                );
            }
        }
    }

    /// The file-global footer must be partitioned, never copied: a
    /// `.private_extern` naming a symbol the slice does not define is a private
    /// external reference with no definition in that object.
    #[test]
    fn the_visibility_footer_is_partitioned_not_copied() {
        let source = fixture();
        let outcome = split_assembly(&source, 4, MACOS_LOCAL);
        assert!(outcome.is_split());
        let analysis = analyze(&source, MACOS_LOCAL);
        let cuts = choose_cuts(&analysis, 4);
        let slice_of = |line: usize| cuts.partition_point(|&cut| cut <= line);
        let mut seen = 0usize;
        for (index, slice) in outcome.slices.iter().enumerate() {
            for line in slice.lines() {
                let Some(name) = line.trim().strip_prefix(".private_extern ") else {
                    continue;
                };
                let name = name.trim();
                let original = name.strip_prefix(PROMOTED_PREFIX).unwrap_or(name);
                let id = *analysis
                    .ids
                    .get(original)
                    .unwrap_or_else(|| panic!("slice {index} published unknown symbol {name}"));
                assert_eq!(
                    slice_of(analysis.spans[id].def),
                    index,
                    "slice {index} published {name}, which it does not define"
                );
                seen += 1;
            }
        }
        assert!(seen >= analysis.footer.len(), "the footer lost entries: {seen}");
        assert!(
            !outcome
                .slices
                .iter()
                .any(|slice| slice.contains(".private_extern _main")),
            "the entry point must not be published private"
        );
    }

    /// A cut may only land where control cannot fall through into the next slice.
    #[test]
    fn cuts_land_only_where_control_cannot_fall_through() {
        let source = fixture();
        let analysis = analyze(&source, MACOS_LOCAL);
        assert!(!analysis.candidates.is_empty());
        for &candidate in &analysis.candidates {
            let mut cursor = candidate;
            while cursor > 0 {
                cursor -= 1;
                let line = analysis.lines[cursor].trim();
                if line.is_empty() || introduces_a_label(line) {
                    continue;
                }
                let mnemonic = line.split_whitespace().next().unwrap_or("");
                assert!(
                    matches!(mnemonic, "ret" | "b" | "br" | "brk") || line.starts_with('.'),
                    "candidate {candidate} follows a fall-through line: {line}"
                );
                break;
            }
        }
    }

    /// The tokenizer must not read a symbol out of a quoted assembly string.
    #[test]
    fn quoted_strings_hide_their_contents_from_the_scanner() {
        let line = "    .ascii \"_first b.eq L_second_retry\"\n";
        assert_eq!(identifiers(line), vec![".ascii"]);
        let mut promoted: HashMap<&str, String> = HashMap::new();
        promoted.insert("L_second_retry", format!("{PROMOTED_PREFIX}L_second_retry"));
        let mut out = String::new();
        push_renamed(&mut out, line, &promoted);
        assert_eq!(out, line, "a quoted string must survive a rename verbatim");
    }

    /// Renaming is whole-token: a longer identifier that merely starts with a
    /// promoted name must not be rewritten.
    #[test]
    fn renaming_is_whole_token() {
        let mut promoted: HashMap<&str, String> = HashMap::new();
        promoted.insert("L_thunk", format!("{PROMOTED_PREFIX}L_thunk"));
        let mut out = String::new();
        push_renamed(&mut out, "    .quad L_thunk_2\n", &promoted);
        assert_eq!(out, "    .quad L_thunk_2\n");
        out.clear();
        push_renamed(&mut out, "    .quad L_thunk@PAGE\n", &promoted);
        assert_eq!(out, format!("    .quad {PROMOTED_PREFIX}L_thunk@PAGE\n"));
    }

    /// A numeric local's `1f`/`1b` reach binds it to its definition inside one slice.
    #[test]
    fn numeric_local_references_are_recognised() {
        assert_eq!(numeric_references("    b.eq 1f\n"), vec![("1", b'f')]);
        assert_eq!(numeric_references("    b 12b\n"), vec![("12", b'b')]);
        assert!(numeric_references("    mov x0, #0x1f\n").is_empty());
        assert!(numeric_references("    ldr x0, [sp, #16]\n").is_empty());
    }

    /// Non-macOS targets and library artifacts keep today's single-`as` path.
    #[test]
    fn only_macos_aarch64_executables_are_split() {
        let macos = Target::new(Platform::MacOS, Arch::AArch64);
        assert!(supports_split(macos, Emit::Executable));
        assert!(!supports_split(macos, Emit::Cdylib));
        assert!(!supports_split(macos, Emit::Staticlib));
        assert!(!supports_split(
            Target::new(Platform::Linux, Arch::AArch64),
            Emit::Executable
        ));
        assert!(!supports_split(
            Target::new(Platform::Linux, Arch::X86_64),
            Emit::Executable
        ));
        assert!(!supports_split(
            Target::new(Platform::MacOS, Arch::X86_64),
            Emit::Executable
        ));
    }
}

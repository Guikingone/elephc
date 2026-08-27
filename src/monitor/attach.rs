//! Purpose:
//! Turns raw program counters read out of another process into the display
//! stacks the rest of `monitor` already renders, exports and diffs.
//!
//! Called from:
//! - `crate::monitor` on Linux, for `--attach`, which is handed a pid already
//!   running under someone else's control and has no channel to ask over.
//!
//! Key details:
//! - Naming only. The syscalls that produce the addresses live next to this and
//!   are the one part a host without `ptrace` cannot exercise; everything here
//!   takes numbers and returns names, and is tested on any host.
//! - The frame chain is walked innermost-first, which is the order a sampler
//!   collects it and the reverse of the order a profile displays it.

use super::elf::{symbolize, FuncSymbol};
use super::Kind;

/// The name given to a frame no symbol claims.
///
/// One bucket rather than an address, because an address is not a name: it says
/// nothing a reader can act on, and a hundred distinct ones would split a single
/// runtime helper into a hundred rows that each look insignificant.
pub(crate) const UNNAMED: &str = "<native>";

/// Turns one sampled frame chain into a display stack, outermost first.
///
/// The addresses arrive innermost-first — a sampler reads the interrupted `pc`
/// and then climbs — and every consumer downstream reads a stack the other way,
/// from the root inwards. Reversing here rather than at each of them is what
/// keeps `{main}` at the top of a table instead of the bottom.
///
/// A run of consecutive unnamed frames collapses to one. A profile is read to
/// find the code that costs, and six rows of `<native>` between two PHP
/// functions tell a reader nothing while pushing what they came for off the
/// screen — which is the same reason the exact profiler folds inlined bodies
/// into their caller.
pub(crate) fn display_stack(
    frames: &[u64],
    symbols: &[FuncSymbol],
    bias: u64,
) -> Vec<(String, Kind)> {
    let mut out: Vec<(String, Kind)> = Vec::new();
    for address in frames.iter().rev() {
        match symbolize(symbols, bias, *address) {
            Some(name) => out.push((name.to_string(), kind_of(name))),
            None => {
                if out.last().map(|(name, _)| name.as_str()) != Some(UNNAMED) {
                    out.push((UNNAMED.to_string(), Kind::Native));
                }
            }
        }
    }
    out
}

/// What kind of time a named frame represents.
///
/// The compiler emits PHP functions under `_php_`/`_fn_` prefixes and its own
/// helpers under `__rt_`, so the name itself says which is which — and the
/// distinction is what lets a reader tell their own hot function from the
/// runtime doing work on its behalf.
fn kind_of(symbol: &str) -> Kind {
    let bare = symbol.strip_prefix('_').unwrap_or(symbol);
    if bare.starts_with("_rt_") || bare.starts_with("rt_") {
        Kind::Helper
    } else if bare.starts_with("php_") || bare.starts_with("fn_") || bare == "main" {
        Kind::Php
    } else {
        Kind::Native
    }
}

/// Counts identical stacks, which is what a sample count IS.
///
/// A sampler produces one stack per interrupt and says nothing about time; the
/// weight of a stack is how many times it was seen. Folding here rather than
/// storing every sample keeps a long window bounded by the program's shapes
/// rather than by its duration.
pub(crate) fn fold(stacks: Vec<Vec<(String, Kind)>>) -> Vec<(Vec<(String, Kind)>, u64)> {
    let mut counted: std::collections::BTreeMap<Vec<(String, Kind)>, u64> =
        std::collections::BTreeMap::new();
    for stack in stacks {
        if stack.is_empty() {
            continue;
        }
        *counted.entry(stack).or_default() += 1;
    }
    counted.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols() -> Vec<FuncSymbol> {
        // Sorted by address, which is what `symbolize` binary-searches.
        vec![
            FuncSymbol { value: 0x1000, size: 0x100, name: "_php_spin".into() },
            FuncSymbol { value: 0x2000, size: 0x100, name: "_php_descend".into() },
            FuncSymbol { value: 0x3000, size: 0x100, name: "__rt_mixed_add".into() },
            FuncSymbol { value: 0x4000, size: 0x100, name: "main".into() },
        ]
    }

    /// The stack comes back outermost-first, whichever way the sampler collected
    /// it. Every consumer downstream reads a stack from the root inwards, and
    /// reversing at each of them instead is how `{main}` ends up at the bottom
    /// of a table it should head.
    #[test]
    fn a_chain_is_turned_around_so_the_root_comes_first() {
        let bias = 0xaaaa_0000_0000;
        // Innermost first, as a sampler reads it: spin called by descend called
        // by main.
        let frames = [bias + 0x1010, bias + 0x2010, bias + 0x4010];
        assert_eq!(
            display_stack(&frames, &symbols(), bias),
            vec![
                ("main".to_string(), Kind::Php),
                ("_php_descend".to_string(), Kind::Php),
                ("_php_spin".to_string(), Kind::Php),
            ]
        );
    }

    /// A runtime helper is named as one, so a reader can tell their own hot
    /// function from the runtime working on its behalf.
    #[test]
    fn a_helper_is_not_reported_as_php() {
        let bias = 0;
        assert_eq!(
            display_stack(&[0x3010], &symbols(), bias),
            vec![("__rt_mixed_add".to_string(), Kind::Helper)]
        );
    }

    /// Frames no symbol claims collapse into one row rather than one each.
    ///
    /// A profile is read to find the code that costs. Six rows of `<native>`
    /// between two PHP functions tell a reader nothing and push what they came
    /// for off the screen.
    #[test]
    fn a_run_of_unnamed_frames_is_one_row() {
        let bias = 0;
        // spin, then three addresses in nobody's range, then main.
        let frames = [0x1010, 0x9000, 0x9100, 0x9200, 0x4010];
        assert_eq!(
            display_stack(&frames, &symbols(), bias),
            vec![
                ("main".to_string(), Kind::Php),
                (UNNAMED.to_string(), Kind::Native),
                ("_php_spin".to_string(), Kind::Php),
            ]
        );
    }

    /// Two unnamed runs SEPARATED by a named frame stay two rows: collapsing
    /// those would join stretches of the program that are not adjacent.
    #[test]
    fn unnamed_runs_on_either_side_of_a_name_stay_apart() {
        let bias = 0;
        let frames = [0x9000, 0x2010, 0x9100, 0x4010];
        assert_eq!(
            display_stack(&frames, &symbols(), bias),
            vec![
                ("main".to_string(), Kind::Php),
                (UNNAMED.to_string(), Kind::Native),
                ("_php_descend".to_string(), Kind::Php),
                (UNNAMED.to_string(), Kind::Native),
            ]
        );
    }

    /// The weight of a stack is how many times it was seen, which is the only
    /// thing a sampler measures.
    #[test]
    fn identical_stacks_are_counted_rather_than_kept() {
        let one = vec![("main".to_string(), Kind::Php)];
        let two = vec![("main".to_string(), Kind::Php), ("_php_spin".to_string(), Kind::Php)];
        let folded = fold(vec![one.clone(), two.clone(), one.clone(), vec![]]);
        assert_eq!(folded.len(), 2, "an empty sample is not a shape: {folded:?}");
        assert!(folded.contains(&(one, 2)));
        assert!(folded.contains(&(two, 1)));
    }
}

//! Purpose:
//! Collects constants and common storage declarations before serializing the assembly data section.
//! Deduplicates string, float, and common symbols used by expression and runtime-facing emitters.
//!
//! Called from:
//! - `crate::codegen` and shared expression/statement emitters.
//!
//! Key details:
//! - Labels must stay stable within one compilation because code emission references them before final serialization.
//! - `.comm`'s alignment operand is target-dependent and must follow the object format, not the
//!   host: Mach-O reads it as a power-of-two exponent, ELF as a byte count. Emitting one spelling
//!   everywhere silently under-aligns every common symbol on ELF, which the assembler accepts and
//!   the linker then rejects with `relocation truncated to fit` for any 64-bit access.

use std::collections::HashMap;

use crate::codegen_support::platform::{Platform, Target};
use crate::types::PhpType;

/// Alignment every common symbol is emitted with: 8 bytes, i.e. `2^3`.
///
/// Common storage holds pointers, `Mixed` boxes and 64-bit scalars, all of which are reached
/// through 64-bit loads and stores. On AArch64 those assemble to `R_AARCH64_LDST64_ABS_LO12_NC`,
/// whose displacement is encoded pre-shifted by 3 — so anything less than 8-byte alignment cannot
/// be represented and the link fails.
const COMM_ALIGN_BYTES: usize = 8;

/// Renders `.comm`'s third operand for one requested byte alignment and target format.
///
/// Mach-O's assembler documents the operand as `log2(alignment)`; GNU as on ELF documents it as
/// the alignment in bytes. `alignment_bytes` must be a non-zero power of two.
fn comm_alignment_operand(target: Target, alignment_bytes: usize) -> usize {
    debug_assert!(alignment_bytes.is_power_of_two() && alignment_bytes > 0);
    match target.platform {
        Platform::MacOS => alignment_bytes.ilog2() as usize,
        Platform::Linux | Platform::Windows => alignment_bytes,
    }
}

/// Renders one complete `.comm` directive line, alignment included, for `target`.
///
/// Every common symbol in the program must go through here rather than spelling the directive
/// inline: the alignment operand is the one part of it that is not portable, and a hardcoded
/// spelling is accepted by both assemblers while only being right for one of them.
pub(crate) fn comm_directive(label: &str, size: usize, target: Target) -> String {
    comm_directive_aligned(label, size, target, COMM_ALIGN_BYTES)
}

/// Renders a `.comm` directive with an explicit power-of-two byte alignment.
///
/// Runtime allocations whose payload is addressed through 128-bit-safe cells use this to
/// preserve their own stronger alignment without weakening ordinary common symbols.
pub(crate) fn comm_directive_aligned(
    label: &str,
    size: usize,
    target: Target,
    alignment_bytes: usize,
) -> String {
    format!(
        ".comm {}, {}, {}\n",
        label,
        size,
        comm_alignment_operand(target, alignment_bytes)
    )
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DataWord {
    U64(u64),
    Symbol(String),
}

/// Symbol-backed metadata for one function static local recorded during EIR
/// lowering: the value symbol, the one-time init-marker symbol, and the codegen
/// PHP type. Consumed only by the `--web` `__rt_web_reset` generator, which must
/// release/zero every persistent static between requests.
#[derive(Clone, Debug)]
pub struct StaticLocalRecord {
    /// `.comm` value symbol (`_static_<fn>_<name>`, 16 bytes).
    pub symbol: String,
    /// `.comm` init-marker symbol (`<symbol>_init`, 8 bytes; 0 = not yet run).
    pub init_symbol: String,
    /// Codegen representation of the static's PHP type (drives release shape).
    pub php_type: PhpType,
}

/// Tracks constants and common symbols for the assembly `.data` section.
///
/// - `entries`: string constants as `(label, bytes)` pairs
/// - `float_entries`: float constants as `(label, IEEE-754 bits)` pairs
/// - `comm_entries`: common symbols as `(label, size)` pairs
/// - `counter`: monotonically increasing integer for generating unique labels
/// - `dedup`/`float_dedup`/`comm_dedup`: deduplication maps to avoid emitting duplicate constants
/// The size of every collection in a `DataSection` at one moment, for `rollback_to`.
#[derive(Clone, Copy)]
pub struct DataSectionCheckpoint {
    entries: usize,
    float_entries: usize,
    word_entries: usize,
    comm_entries: usize,
    static_locals: usize,
    counter: usize,
}

pub struct DataSection {
    entries: Vec<(String, Vec<u8>)>,
    float_entries: Vec<(String, u64)>,
    word_entries: Vec<(String, Vec<DataWord>)>,
    comm_entries: Vec<(String, usize)>,
    counter: usize,
    /// Distinguishes the generated labels of one shard from another's, so two sections built
    /// independently can be merged without renaming anything. Empty for a whole-module section.
    label_shard: String,
    dedup: HashMap<Vec<u8>, String>,
    float_dedup: HashMap<u64, String>,
    word_dedup: HashMap<Vec<DataWord>, String>,
    comm_dedup: HashMap<String, String>,
    static_locals: Vec<StaticLocalRecord>,
    static_local_dedup: HashMap<String, usize>,
}

impl DataSection {
    /// Creates a new empty data section. All collections start empty; the counter is zero.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            float_entries: Vec::new(),
            word_entries: Vec::new(),
            comm_entries: Vec::new(),
            counter: 0,
            label_shard: String::new(),
            dedup: HashMap::new(),
            float_dedup: HashMap::new(),
            word_dedup: HashMap::new(),
            comm_dedup: HashMap::new(),
            static_locals: Vec::new(),
            static_local_dedup: HashMap::new(),
        }
    }

    /// Creates a section whose generated labels carry `shard`, for a body emitted independently.
    ///
    /// The prefix is what makes two sections mergeable without renaming: `_str_s3_17` belongs to
    /// shard 3 and cannot be `_str_s4_17`. Caller-named entries (`.comm`, named symbols, static
    /// locals) are untouched — they are already unique by name, and `merge` dedups them.
    pub fn for_shard(shard: usize) -> Self {
        Self {
            label_shard: format!("s{}_", shard),
            ..Self::new()
        }
    }

    /// Records the current size of every collection, for a possible rollback.
    pub fn checkpoint(&self) -> DataSectionCheckpoint {
        DataSectionCheckpoint {
            entries: self.entries.len(),
            float_entries: self.float_entries.len(),
            word_entries: self.word_entries.len(),
            comm_entries: self.comm_entries.len(),
            static_locals: self.static_locals.len(),
            counter: self.counter,
        }
    }

    /// Discards everything recorded after `checkpoint`.
    ///
    /// Needed because a data entry can REFERENCE a text label — a callable descriptor points at
    /// the body that implements it — so text and data have to be discarded together or the
    /// assembler is handed a reference with no definition.
    ///
    /// Each dedup map is unwound from the entries being removed rather than by scanning what
    /// survives: every map is keyed by something the removed entry still carries.
    pub fn rollback_to(&mut self, checkpoint: DataSectionCheckpoint) {
        for (label, bytes) in self.entries.drain(checkpoint.entries.min(self.entries.len())..) {
            // A named symbol was never interned by bytes, so this is a no-op for those.
            if self.dedup.get(&bytes).is_some_and(|existing| existing == &label) {
                self.dedup.remove(&bytes);
            }
        }
        for (_, bits) in self
            .float_entries
            .drain(checkpoint.float_entries.min(self.float_entries.len())..)
        {
            self.float_dedup.remove(&bits);
        }
        for (_, words) in self
            .word_entries
            .drain(checkpoint.word_entries.min(self.word_entries.len())..)
        {
            self.word_dedup.remove(&words);
        }
        for (label, _) in self
            .comm_entries
            .drain(checkpoint.comm_entries.min(self.comm_entries.len())..)
        {
            self.comm_dedup.remove(&label);
        }
        for record in self
            .static_locals
            .drain(checkpoint.static_locals.min(self.static_locals.len())..)
        {
            self.static_local_dedup.remove(&record.symbol);
        }
        self.counter = checkpoint.counter;
    }

    /// Folds `other` into this section, keeping this section's entries first.
    ///
    /// Generated labels cannot collide when the two sections came from different shards, so
    /// those entries are appended as they are — including duplicate blobs, because each shard's
    /// labels are already baked into the assembly it emitted and repointing them would mean
    /// rewriting that text. Everything keyed by a caller-chosen name is deduplicated instead:
    /// two shards declaring the same `.comm`, the same named symbol or the same function static
    /// is a duplicate DEFINITION, and the first one wins.
    pub fn merge(&mut self, other: Self) {
        for (label, bytes) in other.entries {
            if self.entries.iter().any(|(existing, _)| existing == &label) {
                continue;
            }
            self.entries.push((label, bytes));
        }
        self.float_entries.extend(other.float_entries);
        self.word_entries.extend(other.word_entries);
        for (label, size) in other.comm_entries {
            if self.comm_dedup.contains_key(&label) {
                continue;
            }
            self.comm_dedup.insert(label.clone(), label.clone());
            self.comm_entries.push((label, size));
        }
        for record in other.static_locals {
            self.record_static_local(record);
        }
    }

    /// Records one function static local's storage metadata for the `--web`
    /// per-request reset routine. Deduplicates by value symbol because the same
    /// static is resolved on every load/store/init of that variable; only the
    /// first record per symbol is kept, preserving first-seen order.
    pub fn record_static_local(&mut self, record: StaticLocalRecord) {
        if self.static_local_dedup.contains_key(&record.symbol) {
            return;
        }
        self.static_local_dedup
            .insert(record.symbol.clone(), self.static_locals.len());
        self.static_locals.push(record);
    }

    /// Returns the recorded function static locals in first-seen order, used by
    /// the `--web` `__rt_web_reset` generator after all functions are emitted.
    pub fn static_locals(&self) -> &[StaticLocalRecord] {
        &self.static_locals
    }

    /// Looks up `value` in the float deduplication map; if found, returns the existing label.
    /// Otherwise generates `_float_N`, stores the IEEE-754 bit representation, and returns the new label.
    pub fn add_float(&mut self, value: f64) -> String {
        let bits = value.to_bits();
        if let Some(label) = self.float_dedup.get(&bits) {
            return label.clone();
        }
        let label = format!("_float_{}{}", self.label_shard, self.counter);
        self.counter += 1;
        self.float_dedup.insert(bits, label.clone());
        self.float_entries.push((label.clone(), bits));
        label
    }

    /// Looks up `bytes` in the string deduplication map; if found, returns the existing label and length.
    /// Otherwise generates `_str_N`, clones the bytes into `entries`, and returns the new label and length.
    pub fn add_string(&mut self, bytes: &[u8]) -> (String, usize) {
        if let Some(label) = self.dedup.get(bytes) {
            return (label.clone(), bytes.len());
        }

        let label = format!("_str_{}{}", self.label_shard, self.counter);
        self.counter += 1;
        let owned = bytes.to_vec();
        self.dedup.insert(owned.clone(), label.clone());
        self.entries.push((label.clone(), owned));
        (label, bytes.len())
    }

    /// Emits `bytes` under the caller-chosen global symbol `name` in the `.data`
    /// section (`.globl name` + `.ascii`), for a fixed-name blob other objects
    /// reference by symbol — e.g. the `--probe` build key. Idempotent by name.
    pub fn add_named_symbol(&mut self, name: String, bytes: &[u8]) {
        if self.entries.iter().any(|(label, _)| label == &name) {
            return;
        }
        self.entries.push((name, bytes.to_vec()));
    }

    /// Looks up `label` in the common-symbol deduplication map; if found, returns the existing label.
    /// Otherwise inserts `label` into `comm_entries` with the given `size` and returns `label` unchanged.
    pub fn add_comm(&mut self, label: String, size: usize) -> String {
        if let Some(existing) = self.comm_dedup.get(&label) {
            return existing.clone();
        }

        self.comm_dedup.insert(label.clone(), label.clone());
        self.comm_entries.push((label.clone(), size));
        label
    }

    /// Returns true when common storage has been declared for `label`.
    pub fn has_comm(&self, label: &str) -> bool {
        self.comm_dedup.contains_key(label)
    }

    /// Adds words to the current runtime or metadata collection.
    pub fn add_words(&mut self, words: Vec<DataWord>) -> String {
        if let Some(label) = self.word_dedup.get(&words) {
            return label.clone();
        }
        let label = format!("_data_{}{}", self.label_shard, self.counter);
        self.counter += 1;
        self.word_dedup.insert(words.clone(), label.clone());
        self.word_entries.push((label.clone(), words));
        label
    }

    /// Serializes all entries into a GNU assembly `.data` section string.
    /// Returns an empty string when no entries have been collected.
    /// Emits `.comm` directives first, then `.ascii` string literals, then `.p2align 3`/`quad` float entries.
    ///
    /// `target` is required because `.comm`'s alignment operand is spelled differently per object
    /// format; see [`comm_alignment_operand`].
    pub fn emit(&self, target: Target) -> String {
        if self.entries.is_empty()
            && self.float_entries.is_empty()
            && self.word_entries.is_empty()
            && self.comm_entries.is_empty()
        {
            return String::new();
        }

        let mut out = String::from(".data\n");
        let comm_align = comm_alignment_operand(target, COMM_ALIGN_BYTES);
        for (label, size) in &self.comm_entries {
            out.push_str(&format!(".comm {}, {}, {}\n", label, size, comm_align));
        }
        for (label, bytes) in &self.entries {
            out.push_str(&format!("{}:\n", label));
            out.push_str("    .ascii \"");
            for &b in bytes {
                match b {
                    b'\n' => out.push_str("\\n"),
                    b'\t' => out.push_str("\\t"),
                    b'\\' => out.push_str("\\\\"),
                    b'"' => out.push_str("\\\""),
                    0x20..=0x7e => out.push(b as char),
                    _ => out.push_str(&format!("\\{:03o}", b)),
                }
            }
            out.push_str("\"\n");
        }
        for (label, bits) in &self.float_entries {
            out.push_str(&format!(".p2align 3\n{}:\n    .quad 0x{:016x}\n", label, bits));
        }
        for (label, words) in &self.word_entries {
            out.push_str(&format!(".p2align 3\n{}:\n", label));
            for word in words {
                match word {
                    DataWord::U64(value) => {
                        out.push_str(&format!("    .quad 0x{:016x}\n", value));
                    }
                    DataWord::Symbol(symbol) => {
                        out.push_str(&format!("    .quad {}\n", symbol));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{comm_directive_aligned, DataSection, DataWord};
    use crate::codegen_support::platform::{Arch, Platform, Target};

    /// A Mach-O target, whose assembler reads `.comm`'s alignment operand as `log2(bytes)`.
    fn macos() -> Target {
        Target::new(Platform::MacOS, Arch::AArch64)
    }

    /// An ELF target, whose assembler reads `.comm`'s alignment operand as a byte count.
    fn linux(arch: Arch) -> Target {
        Target::new(Platform::Linux, arch)
    }

    /// Verifies that float constants use power of two alignment directive.
    #[test]
    fn test_float_constants_use_power_of_two_alignment_directive() {
        let mut data = DataSection::new();
        data.add_float(3.14);

        let asm = data.emit(macos());

        assert!(asm.contains(".p2align 3\n"));
        assert!(!asm.contains(".align 3\n"));
    }

    /// Verifies that non printable string bytes use bounded octal escapes.
    #[test]
    fn test_non_printable_string_bytes_use_bounded_octal_escapes() {
        let mut data = DataSection::new();
        data.add_string(b"a\0b");

        let asm = data.emit(macos());

        assert!(asm.contains(r#".ascii "a\000b""#));
        assert!(!asm.contains(r#"\x00b"#));
    }

    /// Verifies that symbol word records emit quad symbols.
    #[test]
    fn test_symbol_word_records_emit_quad_symbols() {
        let mut data = DataSection::new();
        let label = data.add_words(vec![
            super::DataWord::U64(1),
            super::DataWord::Symbol("_fn_demo".to_string()),
        ]);

        let asm = data.emit(macos());

        assert!(asm.contains(&format!("{}:\n", label)));
        assert!(!asm.contains(&format!(".globl {}\n", label)));
        assert!(asm.contains("    .quad 0x0000000000000001\n"));
        assert!(asm.contains("    .quad _fn_demo\n"));
    }

    /// Verifies `.comm` asks each object format for the same 8-byte alignment in the spelling
    /// that format's assembler understands: `log2` on Mach-O, bytes on ELF.
    ///
    /// Emitting the Mach-O spelling on ELF declares 3-byte alignment, which the assembler
    /// accepts and the linker then rejects — `R_AARCH64_LDST64_ABS_LO12_NC` encodes its
    /// displacement pre-shifted by 3, so a 64-bit load of an under-aligned common symbol fails
    /// with `relocation truncated to fit`. That took out every linux-aarch64 link once
    /// `_stack_limit` became a common symbol.
    #[test]
    fn test_comm_alignment_operand_follows_the_object_format() {
        let mut data = DataSection::new();
        data.add_comm("_stack_limit".to_string(), 8);

        assert!(data.emit(macos()).contains(".comm _stack_limit, 8, 3\n"));
        assert!(data
            .emit(linux(Arch::AArch64))
            .contains(".comm _stack_limit, 8, 8\n"));
        assert!(data
            .emit(linux(Arch::X86_64))
            .contains(".comm _stack_limit, 8, 8\n"));
    }

    /// Verifies callers can request the 16-byte storage alignment required by runtime heap cells.
    #[test]
    fn test_explicit_common_alignment_follows_the_object_format() {
        assert!(comm_directive_aligned("_heap_buf", 1024, macos(), 16)
            .contains(".comm _heap_buf, 1024, 4\n"));
        assert!(comm_directive_aligned("_heap_buf", 1024, linux(Arch::AArch64), 16)
            .contains(".comm _heap_buf, 1024, 16\n"));
        assert!(comm_directive_aligned("_heap_buf", 1024, linux(Arch::X86_64), 16)
            .contains(".comm _heap_buf, 1024, 16\n"));
    }

    /// Verifies two shards generate labels that cannot collide, so their sections can be merged
    /// without repointing the assembly each already emitted.
    #[test]
    fn test_shards_generate_labels_that_cannot_collide() {
        let mut first = DataSection::for_shard(0);
        let mut second = DataSection::for_shard(1);
        let (first_label, _) = first.add_string(b"same bytes");
        let (second_label, _) = second.add_string(b"same bytes");
        assert_ne!(first_label, second_label);
        assert_eq!(first_label, "_str_s0_0");
        assert_eq!(second_label, "_str_s1_0");
        assert_eq!(first.add_float(1.5), "_float_s0_1");
        assert_eq!(second.add_words(vec![DataWord::U64(7)]), "_data_s1_1");
    }

    /// Verifies a whole-module section still emits the labels it emits today: the shard prefix
    /// is empty unless a caller asks for one.
    #[test]
    fn test_an_unsharded_section_keeps_its_plain_labels() {
        let mut data = DataSection::new();
        assert_eq!(data.add_string(b"x").0, "_str_0");
        assert_eq!(data.add_float(2.5), "_float_1");
    }

    /// Verifies merge keeps both shards' generated entries — including two copies of the same
    /// blob, which is the deliberate cost of not rewriting emitted assembly — while collapsing
    /// everything named by the caller.
    #[test]
    fn test_merge_keeps_generated_entries_and_collapses_named_ones() {
        let mut first = DataSection::for_shard(0);
        let mut second = DataSection::for_shard(1);
        first.add_string(b"shared blob");
        second.add_string(b"shared blob");
        first.add_comm("_request_slot".to_string(), 8);
        second.add_comm("_request_slot".to_string(), 8);
        second.add_comm("_second_only".to_string(), 16);

        first.merge(second);
        let emitted = first.emit(macos());
        assert_eq!(emitted.matches("shared blob").count(), 2);
        assert_eq!(emitted.matches(".comm _request_slot").count(), 1);
        assert!(emitted.contains(".comm _second_only"));
        assert!(first.has_comm("_second_only"));
    }
}

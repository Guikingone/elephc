//! Purpose:
//! Emits `__rt_mixed_array_loose_eq`, the runtime implementation of PHP's `==`
//! between two arrays: equal element counts, and for every key of the left array a
//! matching key in the right array whose value is loosely equal. The same walk also
//! emits `__rt_mixed_array_compare`, PHP's ordering comparison of two arrays
//! (`zend_compare_arrays`, behind `<`, `>` and `<=>`).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::compare`.
//! - `__rt_mixed_loose_eq` once both operands unbox to an array-like tag.
//! - `__rt_php_compare` (ordering) once both operands are array-like tags.
//!
//! Key details:
//! - PHP's `==` on arrays is ORDER-INDEPENDENT (unlike `===`): the walk enumerates
//!   the left array's keys and looks each one up in the right array, so
//!   `["a"=>1,"b"=>2] == ["b"=>2,"a"=>1]` is true while `[1,2] == [2=>1,3=>2]` is
//!   false.
//! - The ordering walk follows `zend_hash_compare` with `ordered = 0`: a smaller array
//!   sorts first; otherwise each left key in insertion order must exist on the right
//!   (an absent key makes the arrays uncomparable, which PHP reports as `1`), and the
//!   first element pair whose `__rt_php_compare` result is non-zero decides.
//! - Key presence is checked BEFORE the value is read. `__rt_mixed_array_get`
//!   answers `null` both for "absent" and for "present but null", so a missing key
//!   would otherwise compare equal to a stored `null`.
//! - elephc has two array representations. Tag 4 (indexed) is always the list
//!   `0..count-1`, so its keys are enumerated by counting; tag 5 (hash) is walked
//!   with the shared `__rt_hash_iter_next_value` cursor protocol. Both feed the same
//!   per-entry comparison block.
//! - Every value read through `__rt_mixed_array_get` is owned by this helper and is
//!   released before the next entry.
//! - Operand cells that wrap another Mixed cell (tag 7) are peeled on entry, because the
//!   count and read helpers only recognize a container tag on the cell they are handed.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// Selects which PHP comparison one emitted array walk implements.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WalkMode {
    /// `==`: a boolean, false on the first difference.
    LooseEq,
    /// `<=>`: `-1`/`0`/`1`, decided by the first differing count, key or element.
    Order,
}

impl WalkMode {
    /// Returns the global symbol of this walk.
    fn symbol(self) -> &'static str {
        match self {
            Self::LooseEq => "__rt_mixed_array_loose_eq",
            Self::Order => "__rt_mixed_array_compare",
        }
    }

    /// Returns the local label prefix that keeps the two walks' labels distinct.
    fn prefix(self) -> &'static str {
        match self {
            Self::LooseEq => "__rt_male",
            Self::Order => "__rt_maco",
        }
    }

    /// Returns the label reached when the walk consumed every key without a difference.
    fn exhausted(self) -> String {
        match self {
            Self::LooseEq => format!("{}_true", self.prefix()),
            Self::Order => format!("{}_zero", self.prefix()),
        }
    }

    /// Returns the label reached when a left key is absent from the right array.
    fn absent(self) -> String {
        match self {
            Self::LooseEq => format!("{}_false", self.prefix()),
            // `zend_hash_compare` reports an absent key as uncomparable, which is `1`.
            Self::Order => format!("{}_pos", self.prefix()),
        }
    }
}

/// Emits `__rt_mixed_array_loose_eq` for the active target.
///
/// Input: AArch64 `x0`/`x1` = the two boxed array cells, `x2` = recursion depth;
/// x86_64 `rdi`/`rsi`/`rdx`. Output: `x0` / `rax` = 1 when loosely equal. Both
/// operands stay borrowed.
pub fn emit_mixed_array_loose_eq(emitter: &mut Emitter) {
    emit_array_walk(emitter, WalkMode::LooseEq);
}

/// Emits `__rt_mixed_array_compare` for the active target.
///
/// Input: AArch64 `x0`/`x1` = the two boxed array cells, `x2` = recursion depth;
/// x86_64 `rdi`/`rsi`/`rdx`. Output: `x0` / `rax` = `-1`, `0` or `1`. Both operands
/// stay borrowed; borrowed stack cells are accepted.
pub fn emit_mixed_array_compare(emitter: &mut Emitter) {
    emit_array_walk(emitter, WalkMode::Order);
}

/// Dispatches one array walk to the target-specific emitter.
fn emit_array_walk(emitter: &mut Emitter, mode: WalkMode) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_walk_x86_64(emitter, mode);
        return;
    }
    emit_array_walk_aarch64(emitter, mode);
}

/// Emits the AArch64 array comparison walker.
///
/// Frame (128 bytes for `==`, 160 for `<=>`): `[sp,#0]` left cell, `[sp,#8]` right cell,
/// `[sp,#16]` depth, `[sp,#24]` the left element count, `[sp,#32]`/`[sp,#40]` right tag and
/// payload, `[sp,#48]` index-or-cursor, `[sp,#56]`/`[sp,#64]` left tag and payload,
/// `[sp,#72]`/`[sp,#80]` the current key pair, `[sp,#88]`/`[sp,#96]` the two owned value
/// cells, `[sp,#104]` the comparison result; the ordering walk keeps the left element
/// triple at `[sp,#112..#136]`. Saved `x29`/`x30` close the frame.
fn emit_array_walk_aarch64(emitter: &mut Emitter, mode: WalkMode) {
    let p = mode.prefix();
    let (frame, link) = match mode {
        WalkMode::LooseEq => (128, 112),
        WalkMode::Order => (160, 144),
    };
    let exhausted = mode.exhausted();
    let absent = mode.absent();
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", mode.symbol().trim_start_matches("__rt_")));
    emitter.label_global(mode.symbol());
    emit_peel_nested_mixed_operands(emitter, p);

    emitter.instruction(&format!("sub sp, sp, #{frame}"));                      // allocate the array comparison frame
    emitter.instruction(&format!("stp x29, x30, [sp, #{link}]"));               // save frame pointer and return address
    emitter.instruction(&format!("add x29, sp, #{link}"));                      // establish the array comparison frame pointer
    emitter.instruction("stp x0, x1, [sp, #0]");                                // save both boxed array operands
    emitter.instruction("str x2, [sp, #16]");                                   // save the current recursion depth
    emitter.instruction("bl __rt_mixed_count");                                 // count the left array's elements
    emitter.instruction("str x0, [sp, #24]");                                   // save the left element count
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed array operand
    emitter.instruction("bl __rt_mixed_count");                                 // count the right array's elements
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the left element count
    emitter.instruction("cmp x9, x0");                                          // PHP requires both arrays to hold the same number of entries
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("b.ne __rt_male_false");                        // different sizes are never loosely equal
            emitter.instruction("cbz x9, __rt_male_true");                      // two empty arrays are loosely equal
        }
        WalkMode::Order => {
            emitter.instruction(&format!("b.lt {p}_neg"));                      // the array with fewer entries sorts first
            emitter.instruction(&format!("b.gt {p}_pos"));                      // the array with more entries sorts last
            emitter.instruction(&format!("cbz x9, {p}_zero"));                  // two empty arrays compare equal
        }
    }

    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed array operand
    emitter.instruction("bl __rt_mixed_unbox");                                 // left cell -> x0=tag, x1=payload pointer
    emitter.instruction("str x0, [sp, #56]");                                   // save the left container tag
    emitter.instruction("str x1, [sp, #64]");                                   // save the left container payload pointer
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed array operand
    emitter.instruction("bl __rt_mixed_unbox");                                 // right cell -> x0=tag, x1=payload pointer
    emitter.instruction("str x0, [sp, #32]");                                   // save the right container tag
    emitter.instruction("str x1, [sp, #40]");                                   // save the right container payload pointer
    emitter.instruction("mov x11, #0");                                         // the index/cursor starts at the first entry
    emitter.instruction("str x11, [sp, #48]");                                  // save the initial index/cursor
    emitter.instruction("ldr x9, [sp, #56]");                                   // reload the left container tag
    emitter.instruction("cmp x9, #4");                                          // is the left container an indexed array?
    emitter.instruction(&format!("b.ne {p}_hash_loop"));                        // hashes enumerate through the iterator protocol

    // -- indexed arrays are the list 0..count-1, so keys come from the index --
    emitter.label(&format!("{p}_indexed_loop"));
    emitter.instruction("ldr x11, [sp, #48]");                                  // reload the current element index
    emitter.instruction("ldr x12, [sp, #24]");                                  // reload the shared element count
    emitter.instruction("cmp x11, x12");                                        // has every element been compared?
    emitter.instruction(&format!("b.ge {exhausted}"));                          // every key matched without a difference
    emitter.instruction("str x11, [sp, #72]");                                  // the element index is the PHP key
    emitter.instruction("mov x13, #-1");                                        // key_hi = -1 marks an integer key
    emitter.instruction("str x13, [sp, #80]");                                  // save the integer-key marker
    emitter.instruction(&format!("b {p}_entry"));                               // compare this key against the right array

    emitter.label(&format!("{p}_indexed_next"));
    emitter.instruction("ldr x11, [sp, #48]");                                  // reload the current element index
    emitter.instruction("add x11, x11, #1");                                    // advance to the next list slot
    emitter.instruction("str x11, [sp, #48]");                                  // save the advanced element index
    emitter.instruction(&format!("b {p}_indexed_loop"));                        // keep walking the list

    // -- hashes enumerate in insertion order through the shared cursor protocol --
    emitter.label(&format!("{p}_hash_loop"));
    emitter.instruction("ldr x0, [sp, #64]");                                   // reload the left hash payload pointer
    emitter.instruction("ldr x1, [sp, #48]");                                   // reload the iteration cursor
    emitter.instruction("bl __rt_hash_iter_next_value");                        // x0=next cursor, x1=key pointer, x2=key length
    emitter.instruction("cmp x0, #-1");                                         // has the walk consumed every entry?
    emitter.instruction(&format!("b.eq {exhausted}"));                          // every key matched without a difference
    emitter.instruction("str x0, [sp, #48]");                                   // save the next iteration cursor
    emitter.instruction("str x1, [sp, #72]");                                   // save the current key low word
    emitter.instruction("str x2, [sp, #80]");                                   // save the current key high word
    emitter.instruction(&format!("b {p}_entry"));                               // compare this key against the right array

    emitter.label(&format!("{p}_hash_next"));
    emitter.instruction(&format!("b {p}_hash_loop"));                           // keep walking the hash in insertion order

    // -- one key: it must exist on the right before its values are compared --
    emitter.label(&format!("{p}_entry"));
    emitter.instruction("ldr x9, [sp, #32]");                                   // reload the right container tag
    emitter.instruction("cmp x9, #4");                                          // is the right container an indexed array?
    emitter.instruction(&format!("b.ne {p}_entry_hash_lookup"));                // hashes answer key presence themselves
    emitter.instruction("ldr x13, [sp, #80]");                                  // reload the current key high word
    emitter.instruction("cmp x13, #-1");                                        // is the key an integer key?
    emitter.instruction(&format!("b.ne {absent}"));                             // a string key cannot exist in a list
    emitter.instruction("ldr x13, [sp, #72]");                                  // reload the current key low word
    emitter.instruction("cmp x13, #0");                                         // is the integer key non-negative?
    emitter.instruction(&format!("b.lt {absent}"));                             // a negative key is outside every list
    emitter.instruction("ldr x12, [sp, #24]");                                  // reload the shared element count
    emitter.instruction("cmp x13, x12");                                        // is the integer key inside the list bounds?
    emitter.instruction(&format!("b.ge {absent}"));                             // an out-of-range key is absent
    emitter.instruction(&format!("b {p}_entry_compare"));                       // the key exists, compare the values
    emitter.label(&format!("{p}_entry_hash_lookup"));
    emitter.instruction("ldr x0, [sp, #40]");                                   // reload the right hash payload pointer
    emitter.instruction("ldr x1, [sp, #72]");                                   // reload the current key low word
    emitter.instruction("ldr x2, [sp, #80]");                                   // reload the current key high word
    emitter.instruction("bl __rt_hash_get");                                    // probe the right hash for this key
    emitter.instruction(&format!("cbz x0, {absent}"));                          // a key missing on the right ends the comparison

    emitter.label(&format!("{p}_entry_compare"));
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the left boxed array operand
    emitter.instruction("ldr x1, [sp, #72]");                                   // reload the current key low word
    emitter.instruction("ldr x2, [sp, #80]");                                   // reload the current key high word
    emitter.instruction("mov x3, #0");                                          // read quietly: a comparison must not warn
    emitter.instruction("bl __rt_mixed_array_get");                             // read the left value as an owned boxed cell
    emitter.instruction("str x0, [sp, #88]");                                   // save the owned left value cell
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the right boxed array operand
    emitter.instruction("ldr x1, [sp, #72]");                                   // reload the current key low word
    emitter.instruction("ldr x2, [sp, #80]");                                   // reload the current key high word
    emitter.instruction("mov x3, #0");                                          // read quietly: a comparison must not warn
    emitter.instruction("bl __rt_mixed_array_get");                             // read the right value as an owned boxed cell
    emitter.instruction("str x0, [sp, #96]");                                   // save the owned right value cell
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("ldr x0, [sp, #88]");                           // reload the left value cell
            emitter.instruction("ldr x1, [sp, #96]");                           // reload the right value cell
            emitter.instruction("ldr x2, [sp, #16]");                           // reload the current recursion depth
            emitter.instruction("bl __rt_mixed_loose_eq_d");                    // compare the two element values loosely
        }
        WalkMode::Order => {
            emitter.instruction("ldr x0, [sp, #88]");                           // reload the left value cell
            emitter.instruction("bl __rt_mixed_unbox");                         // left value -> x0=tag, x1=low word, x2=high word
            emitter.instruction("stp x0, x1, [sp, #112]");                      // save the left element tag and low word
            emitter.instruction("str x2, [sp, #128]");                          // save the left element high word
            emitter.instruction("ldr x0, [sp, #96]");                           // reload the right value cell
            emitter.instruction("bl __rt_mixed_unbox");                         // right value -> x0=tag, x1=low word, x2=high word
            emitter.instruction("mov x3, x0");                                  // pass the right element tag
            emitter.instruction("mov x4, x1");                                  // pass the right element low word
            emitter.instruction("mov x5, x2");                                  // pass the right element high word
            emitter.instruction("ldp x0, x1, [sp, #112]");                      // pass the left element tag and low word
            emitter.instruction("ldr x2, [sp, #128]");                          // pass the left element high word
            emitter.instruction("bl __rt_php_compare");                         // order the two element values the PHP way
        }
    }
    emitter.instruction("str x0, [sp, #104]");                                  // save the element comparison result
    emitter.instruction("ldr x0, [sp, #88]");                                   // reload the owned left value cell
    emitter.instruction("bl __rt_decref_mixed");                                // release the left element copy
    emitter.instruction("ldr x0, [sp, #96]");                                   // reload the owned right value cell
    emitter.instruction("bl __rt_decref_mixed");                                // release the right element copy
    emitter.instruction("ldr x0, [sp, #104]");                                  // reload the element comparison result
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("cbz x0, __rt_male_false");                     // one differing element ends the comparison
        }
        WalkMode::Order => {
            emitter.instruction(&format!("cbnz x0, {p}_done"));                 // the first unequal element pair decides the order
        }
    }
    emitter.instruction("ldr x9, [sp, #56]");                                   // reload the left container tag
    emitter.instruction("cmp x9, #4");                                          // did this entry come from the list walk?
    emitter.instruction(&format!("b.eq {p}_indexed_next"));                     // resume the list walk
    emitter.instruction(&format!("b {p}_hash_next"));                           // resume the hash walk

    match mode {
        WalkMode::LooseEq => {
            emitter.label("__rt_male_true");
            emitter.instruction("mov x0, #1");                                  // report that the two arrays are loosely equal
            emitter.instruction("b __rt_male_done");                            // return the true result

            emitter.label("__rt_male_false");
            emitter.instruction("mov x0, #0");                                  // report that the two arrays differ
        }
        WalkMode::Order => {
            emitter.label(&format!("{p}_neg"));
            emitter.instruction("mov x0, #-1");                                 // the left array sorts before the right one
            emitter.instruction(&format!("b {p}_done"));                        // return the ordering result
            emitter.label(&format!("{p}_pos"));
            emitter.instruction("mov x0, #1");                                  // the left array sorts after (or is uncomparable to) the right one
            emitter.instruction(&format!("b {p}_done"));                        // return the ordering result
            emitter.label(&format!("{p}_zero"));
            emitter.instruction("mov x0, #0");                                  // the two arrays compare equal
        }
    }

    emitter.label(&format!("{p}_done"));
    emitter.instruction(&format!("ldp x29, x30, [sp, #{link}]"));               // restore frame pointer and return address
    emitter.instruction(&format!("add sp, sp, #{frame}"));                      // release the array comparison frame
    emitter.instruction("ret");                                                 // return the array comparison result
}

/// Replaces each operand cell that wraps another Mixed cell (tag 7) with the cell it wraps.
///
/// `__rt_mixed_count` and `__rt_mixed_array_get` read the tag word of the cell they are given
/// and treat tag 7 as "not a container", so a wrapper operand counted 0 and read every entry as
/// null. `__rt_in_array_boxed` hands over exactly such cells — a Mixed needle, and each element
/// of a Mixed-slot haystack, is described by a borrowed tag-7 stack cell — which made
/// `in_array([1, 2], $mixed)` miss a real match and match an unrelated array of the same size.
/// The walker is only entered once both operands unboxed to array tags, so every chain ends at
/// a concrete array cell and no wrapper holds a null pointer. Only the operand registers are
/// rewritten; the cells stay borrowed.
fn emit_peel_nested_mixed_operands(emitter: &mut Emitter, p: &str) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.label(&format!("{p}_peel_left"));
            emitter.instruction("ldr x9, [x0]");                                // inspect the left operand cell tag
            emitter.instruction("cmp x9, #7");                                  // does the left cell wrap another Mixed cell?
            emitter.instruction(&format!("b.ne {p}_peel_right"));               // a concrete left cell is ready for the walk
            emitter.instruction("ldr x0, [x0, #8]");                            // follow the wrapped left cell pointer
            emitter.instruction(&format!("b {p}_peel_left"));                   // keep peeling nested left wrappers
            emitter.label(&format!("{p}_peel_right"));
            emitter.instruction("ldr x9, [x1]");                                // inspect the right operand cell tag
            emitter.instruction("cmp x9, #7");                                  // does the right cell wrap another Mixed cell?
            emitter.instruction(&format!("b.ne {p}_peeled"));                   // a concrete right cell is ready for the walk
            emitter.instruction("ldr x1, [x1, #8]");                            // follow the wrapped right cell pointer
            emitter.instruction(&format!("b {p}_peel_right"));                  // keep peeling nested right wrappers
            emitter.label(&format!("{p}_peeled"));
        }
        Arch::X86_64 => {
            emitter.label(&format!("{p}_peel_left"));
            emitter.instruction("cmp QWORD PTR [rdi], 7");                      // does the left cell wrap another Mixed cell?
            emitter.instruction(&format!("jne {p}_peel_right"));                // a concrete left cell is ready for the walk
            emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                // follow the wrapped left cell pointer
            emitter.instruction(&format!("jmp {p}_peel_left"));                 // keep peeling nested left wrappers
            emitter.label(&format!("{p}_peel_right"));
            emitter.instruction("cmp QWORD PTR [rsi], 7");                      // does the right cell wrap another Mixed cell?
            emitter.instruction(&format!("jne {p}_peeled"));                    // a concrete right cell is ready for the walk
            emitter.instruction("mov rsi, QWORD PTR [rsi + 8]");                // follow the wrapped right cell pointer
            emitter.instruction(&format!("jmp {p}_peel_right"));                // keep peeling nested right wrappers
            emitter.label(&format!("{p}_peeled"));
        }
    }
}

/// Emits the x86_64 array comparison walker.
///
/// Frame (112 bytes below `rbp` for `==`, 144 for `<=>`): `[rbp-8]` left cell, `[rbp-16]`
/// right cell, `[rbp-24]` depth, `[rbp-32]` the left element count, `[rbp-40]`/`[rbp-48]`
/// right tag and payload, `[rbp-56]` index-or-cursor, `[rbp-64]`/`[rbp-72]` left tag and
/// payload, `[rbp-80]`/`[rbp-88]` the current key pair, `[rbp-96]`/`[rbp-104]` the two owned
/// value cells, `[rbp-112]` the comparison result; the ordering walk keeps the left element
/// triple at `[rbp-120..-136]`.
fn emit_array_walk_x86_64(emitter: &mut Emitter, mode: WalkMode) {
    let p = mode.prefix();
    let frame = match mode {
        WalkMode::LooseEq => 112,
        WalkMode::Order => 144,
    };
    let exhausted = mode.exhausted();
    let absent = mode.absent();
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", mode.symbol().trim_start_matches("__rt_")));
    emitter.label_global(mode.symbol());
    emit_peel_nested_mixed_operands(emitter, p);

    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the array comparison frame pointer
    emitter.instruction(&format!("sub rsp, {frame}"));                          // allocate the aligned array comparison frame
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the left boxed array operand
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the right boxed array operand
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the current recursion depth
    emitter.instruction("mov rax, rdi");                                        // move the left cell into the count input register
    abi::emit_call_label(emitter, "__rt_mixed_count"); // count the left array's elements
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the left element count
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the right cell for counting
    abi::emit_call_label(emitter, "__rt_mixed_count"); // count the right array's elements
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the left element count
    emitter.instruction("cmp r10, rax");                                        // PHP requires both arrays to hold the same number of entries
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("jne __rt_male_false");                         // different sizes are never loosely equal
            emitter.instruction("test r10, r10");                               // are both arrays empty?
            emitter.instruction("jz __rt_male_true");                           // two empty arrays are loosely equal
        }
        WalkMode::Order => {
            emitter.instruction(&format!("jl {p}_neg"));                        // the array with fewer entries sorts first
            emitter.instruction(&format!("jg {p}_pos"));                        // the array with more entries sorts last
            emitter.instruction("test r10, r10");                               // are both arrays empty?
            emitter.instruction(&format!("jz {p}_zero"));                       // two empty arrays compare equal
        }
    }

    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the left cell for unboxing
    abi::emit_call_label(emitter, "__rt_mixed_unbox"); // left cell -> rax=tag, rdi=payload pointer
    emitter.instruction("mov QWORD PTR [rbp - 64], rax");                       // save the left container tag
    emitter.instruction("mov QWORD PTR [rbp - 72], rdi");                       // save the left container payload pointer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // reload the right cell for unboxing
    abi::emit_call_label(emitter, "__rt_mixed_unbox"); // right cell -> rax=tag, rdi=payload pointer
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the right container tag
    emitter.instruction("mov QWORD PTR [rbp - 48], rdi");                       // save the right container payload pointer
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // the index/cursor starts at the first entry
    emitter.instruction("cmp QWORD PTR [rbp - 64], 4");                         // is the left container an indexed array?
    emitter.instruction(&format!("jne {p}_hash_loop"));                         // hashes enumerate through the iterator protocol

    // -- indexed arrays are the list 0..count-1, so keys come from the index --
    emitter.label(&format!("{p}_indexed_loop"));
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                       // reload the current element index
    emitter.instruction("cmp r10, QWORD PTR [rbp - 32]");                       // has every element been compared?
    emitter.instruction(&format!("jge {exhausted}"));                           // every key matched without a difference
    emitter.instruction("mov QWORD PTR [rbp - 80], r10");                       // the element index is the PHP key
    emitter.instruction("mov QWORD PTR [rbp - 88], -1");                        // key_hi = -1 marks an integer key
    emitter.instruction(&format!("jmp {p}_entry"));                             // compare this key against the right array

    emitter.label(&format!("{p}_indexed_next"));
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                       // reload the current element index
    emitter.instruction("add r10, 1");                                          // advance to the next list slot
    emitter.instruction("mov QWORD PTR [rbp - 56], r10");                       // save the advanced element index
    emitter.instruction(&format!("jmp {p}_indexed_loop"));                      // keep walking the list

    // -- hashes enumerate in insertion order through the shared cursor protocol --
    emitter.label(&format!("{p}_hash_loop"));
    emitter.instruction("mov rdi, QWORD PTR [rbp - 72]");                       // reload the left hash payload pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // reload the iteration cursor
    abi::emit_call_label(emitter, "__rt_hash_iter_next_value"); // rax=next cursor, rdi=key pointer, rdx=key length
    emitter.instruction("cmp rax, -1");                                         // has the walk consumed every entry?
    emitter.instruction(&format!("je {exhausted}"));                            // every key matched without a difference
    emitter.instruction("mov QWORD PTR [rbp - 56], rax");                       // save the next iteration cursor
    emitter.instruction("mov QWORD PTR [rbp - 80], rdi");                       // save the current key low word
    emitter.instruction("mov QWORD PTR [rbp - 88], rdx");                       // save the current key high word
    emitter.instruction(&format!("jmp {p}_entry"));                             // compare this key against the right array

    emitter.label(&format!("{p}_hash_next"));
    emitter.instruction(&format!("jmp {p}_hash_loop"));                         // keep walking the hash in insertion order

    // -- one key: it must exist on the right before its values are compared --
    emitter.label(&format!("{p}_entry"));
    emitter.instruction("cmp QWORD PTR [rbp - 40], 4");                         // is the right container an indexed array?
    emitter.instruction(&format!("jne {p}_entry_hash_lookup"));                 // hashes answer key presence themselves
    emitter.instruction("cmp QWORD PTR [rbp - 88], -1");                        // is the key an integer key?
    emitter.instruction(&format!("jne {absent}"));                              // a string key cannot exist in a list
    emitter.instruction("mov r10, QWORD PTR [rbp - 80]");                       // reload the current key low word
    emitter.instruction("cmp r10, 0");                                          // is the integer key non-negative?
    emitter.instruction(&format!("jl {absent}"));                               // a negative key is outside every list
    emitter.instruction("cmp r10, QWORD PTR [rbp - 32]");                       // is the integer key inside the list bounds?
    emitter.instruction(&format!("jge {absent}"));                              // an out-of-range key is absent
    emitter.instruction(&format!("jmp {p}_entry_compare"));                     // the key exists, compare the values
    emitter.label(&format!("{p}_entry_hash_lookup"));
    emitter.instruction("mov rdi, QWORD PTR [rbp - 48]");                       // reload the right hash payload pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // reload the current key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // reload the current key high word
    abi::emit_call_label(emitter, "__rt_hash_get"); // probe the right hash for this key
    emitter.instruction("test rax, rax");                                       // did the right hash contain this key?
    emitter.instruction(&format!("jz {absent}"));                               // a key missing on the right ends the comparison

    emitter.label(&format!("{p}_entry_compare"));
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the left boxed array operand
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // reload the current key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // reload the current key high word
    emitter.instruction("xor ecx, ecx");                                        // read quietly: a comparison must not warn
    abi::emit_call_label(emitter, "__rt_mixed_array_get"); // read the left value as an owned boxed cell
    emitter.instruction("mov QWORD PTR [rbp - 96], rax");                       // save the owned left value cell
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // reload the right boxed array operand
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // reload the current key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // reload the current key high word
    emitter.instruction("xor ecx, ecx");                                        // read quietly: a comparison must not warn
    abi::emit_call_label(emitter, "__rt_mixed_array_get"); // read the right value as an owned boxed cell
    emitter.instruction("mov QWORD PTR [rbp - 104], rax");                      // save the owned right value cell
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("mov rdi, QWORD PTR [rbp - 96]");               // reload the left value cell
            emitter.instruction("mov rsi, QWORD PTR [rbp - 104]");              // reload the right value cell
            emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");               // reload the current recursion depth
            abi::emit_call_label(emitter, "__rt_mixed_loose_eq_d"); // compare the two element values loosely
        }
        WalkMode::Order => {
            emitter.instruction("mov rax, QWORD PTR [rbp - 96]");               // reload the left value cell for unboxing
            abi::emit_call_label(emitter, "__rt_mixed_unbox"); // left value -> rax=tag, rdi=low word, rdx=high word
            emitter.instruction("mov QWORD PTR [rbp - 120], rax");              // save the left element tag
            emitter.instruction("mov QWORD PTR [rbp - 128], rdi");              // save the left element low word
            emitter.instruction("mov QWORD PTR [rbp - 136], rdx");              // save the left element high word
            emitter.instruction("mov rax, QWORD PTR [rbp - 104]");              // reload the right value cell for unboxing
            abi::emit_call_label(emitter, "__rt_mixed_unbox"); // right value -> rax=tag, rdi=low word, rdx=high word
            emitter.instruction("mov rcx, rax");                                // pass the right element tag
            emitter.instruction("mov r8, rdi");                                 // pass the right element low word
            emitter.instruction("mov r9, rdx");                                 // pass the right element high word
            emitter.instruction("mov rdi, QWORD PTR [rbp - 120]");              // pass the left element tag
            emitter.instruction("mov rsi, QWORD PTR [rbp - 128]");              // pass the left element low word
            emitter.instruction("mov rdx, QWORD PTR [rbp - 136]");              // pass the left element high word
            abi::emit_call_label(emitter, "__rt_php_compare"); // order the two element values the PHP way
        }
    }
    emitter.instruction("mov QWORD PTR [rbp - 112], rax");                      // save the element comparison result
    emitter.instruction("mov rax, QWORD PTR [rbp - 96]");                       // reload the owned left value cell
    abi::emit_call_label(emitter, "__rt_decref_mixed"); // release the left element copy
    emitter.instruction("mov rax, QWORD PTR [rbp - 104]");                      // reload the owned right value cell
    abi::emit_call_label(emitter, "__rt_decref_mixed"); // release the right element copy
    match mode {
        WalkMode::LooseEq => {
            emitter.instruction("cmp QWORD PTR [rbp - 112], 0");                // did the two element values compare equal?
            emitter.instruction("je __rt_male_false");                          // one differing element ends the comparison
        }
        WalkMode::Order => {
            emitter.instruction("mov rax, QWORD PTR [rbp - 112]");              // reload the element comparison result
            emitter.instruction("test rax, rax");                               // did the two element values compare equal?
            emitter.instruction(&format!("jnz {p}_done"));                      // the first unequal element pair decides the order
        }
    }
    emitter.instruction("cmp QWORD PTR [rbp - 64], 4");                         // did this entry come from the list walk?
    emitter.instruction(&format!("je {p}_indexed_next"));                       // resume the list walk
    emitter.instruction(&format!("jmp {p}_hash_next"));                         // resume the hash walk

    match mode {
        WalkMode::LooseEq => {
            emitter.label("__rt_male_true");
            emitter.instruction("mov rax, 1");                                  // report that the two arrays are loosely equal
            emitter.instruction("jmp __rt_male_done");                          // return the true result

            emitter.label("__rt_male_false");
            emitter.instruction("xor rax, rax");                                // report that the two arrays differ
        }
        WalkMode::Order => {
            emitter.label(&format!("{p}_neg"));
            emitter.instruction("mov rax, -1");                                 // the left array sorts before the right one
            emitter.instruction(&format!("jmp {p}_done"));                      // return the ordering result
            emitter.label(&format!("{p}_pos"));
            emitter.instruction("mov rax, 1");                                  // the left array sorts after (or is uncomparable to) the right one
            emitter.instruction(&format!("jmp {p}_done"));                      // return the ordering result
            emitter.label(&format!("{p}_zero"));
            emitter.instruction("xor eax, eax");                                // the two arrays compare equal
        }
    }

    emitter.label(&format!("{p}_done"));
    emitter.instruction(&format!("add rsp, {frame}"));                          // release the array comparison frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the array comparison result
}

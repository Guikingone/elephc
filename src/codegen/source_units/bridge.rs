//! Compact native include-state lookup/reset callbacks over a table of physical sources.
//! Callbacks touch only static guard cells and never execute PHP or allocate PHP values.

use super::include_guard_symbol;
use crate::codegen::{abi, context::FunctionContext, data_section::{DataSection, DataWord}, emit::Emitter, platform::Arch};
use crate::ir::Module;

const LOOKUP: &str = "__rt_source_include_lookup";
const RESET: &str = "__rt_source_include_reset";
const PRIME: &str = "__rt_source_include_prime";
/// C-ABI name: Magician calls this one by symbol, so it carries the platform prefix.
const CLASS_LOOKUP: &str = "__elephc_eval_class_deferred_lookup";
const CLASS_RESET: &str = "__rt_class_deferred_reset";
/// Bytes per table record: path pointer, path length, guard cell, compiler-included flag.
const RECORD: usize = 32;
const FRAME: usize = 64;
const PATH: usize = 8;
const LENGTH: usize = 16;
const CURSOR: usize = 24;
const REMAINING: usize = 32;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{AppleVariant, Platform, Target};

    fn helper_asm(target: Target, count: usize) -> String {
        let catalog = crate::ir::SourceCatalog::from_units((0..count).map(|index| crate::resolver::SourceUnit {
            canonical_path: format!("/sources/{index}.php").into(),
            mode: crate::source::SourceMode::Php, source: "<?php".into(),
        })).unwrap();
        let mut module = Module::with_source_catalog(target, catalog);
        module.required_runtime_features.eval_bridge = true;
        let mut emitter = Emitter::new(target);
        let mut data = DataSection::new();
        emit_state_helpers(&module, &mut emitter, &mut data);
        for (_, source) in module.source_catalog().unwrap().iter() {
            assert!(data.has_comm(&include_guard_symbol(&source.canonical_path)));
        }
        emitter.output()
    }

    #[test]
    fn native_include_lookup_is_compact_on_every_supported_target() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
            Target { apple_variant: AppleVariant::IOS, ..Target::new(Platform::MacOS, Arch::AArch64) },
            Target { apple_variant: AppleVariant::IOSSimulator, ..Target::new(Platform::MacOS, Arch::AArch64) },
        ] {
            let small = helper_asm(target, 1);
            let large = helper_asm(target, 100);
            assert!(large.len() < small.len() + 128, "lookup code must not unroll per source");
            assert!(small.contains(LOOKUP) && small.contains(RESET));
            match target.arch {
                Arch::AArch64 => assert!(small.contains("cbnz w0")),
                Arch::X86_64 => assert!(small.contains("test eax, eax")),
            }
            assert!(helper_asm(target, 0).contains(LOOKUP));
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_include_callbacks_execute_on_host() {
        use std::os::unix::ffi::OsStringExt;
        struct Fixture { path: std::path::PathBuf, temporary: bool }
        impl Drop for Fixture {
            fn drop(&mut self) { if self.temporary { let _ = std::fs::remove_dir_all(&self.path); } }
        }
        let (path, temporary): (std::path::PathBuf, bool) = match std::env::var_os("ELEPHC_INCLUDE_ABI_FIXTURE_DIR") {
            Some(path) => (path.into(), false),
            None => (std::env::temp_dir().join(format!("elephc-include-abi-{}-{}", std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())), true),
        };
        // Acquire ownership before installing cleanup, never delete a pre-existing directory.
        if temporary { std::fs::create_dir(&path).unwrap(); }
        else { std::fs::create_dir_all(&path).unwrap(); }
        let fixture = Fixture { path, temporary };
        let c = fixture.path.join("check.c");
        std::fs::write(&c, r#"
typedef unsigned long long u64;
extern u64 *lookup(const unsigned char *, u64) __asm__("__rt_source_include_lookup");
extern void reset(void) __asm__("__rt_source_include_reset");
int main(void) {
    const unsigned char a[] = "/sources/0.php";
    const unsigned char b[] = "/sources/1.php";
    const unsigned char absent[] = "/sources/x.php";
    const unsigned char raw[] = {'/', 's', 255};
    u64 *first = lookup(a, sizeof(a)-1), *second = lookup(b, sizeof(b)-1);
    u64 *third = lookup(raw, sizeof(raw));
    if (!first || !second || !third || first == second || second == third || first == third) return 1;
    if (*first || *second || *third) return 2;
    if (lookup(absent, sizeof(absent)-1) || lookup(a, 0)) return 3;
    *first = 1; *second = 1; *third = 1;
    if (lookup(a, sizeof(a)-1) != first || *lookup(a, sizeof(a)-1) != 1) return 4;
    reset();
    return (*first || *second || *third) ? 5 : 0;
}
"#).unwrap();
        let host = Target::detect_host();
        for target in [host, Target::new(Platform::Linux, Arch::AArch64), Target::new(Platform::Linux, Arch::X86_64)] {
            let paths = [std::path::PathBuf::from("/sources/0.php"), "/sources/1.php".into(),
                std::ffi::OsString::from_vec(vec![b'/', b's', 255]).into()];
            let catalog = crate::ir::SourceCatalog::from_units(paths.map(|path| crate::resolver::SourceUnit {
                canonical_path: path, mode: crate::source::SourceMode::Php, source: "<?php".into(),
            })).unwrap();
            let mut module = Module::with_source_catalog(target, catalog);
            module.required_runtime_features.eval_bridge = true;
            let mut emitter = Emitter::new(target);
            let mut data = DataSection::new();
            emit_state_helpers(&module, &mut emitter, &mut data);
            let syntax = if target.arch == Arch::X86_64 { ".intel_syntax noprefix\n" } else { "" };
            let asm = fixture.path.join(format!("{target}.s"));
            std::fs::write(&asm, format!(".text\n{syntax}{}{}", emitter.output(), data.emit(target))).unwrap();
        }
        let executable = fixture.path.join("check");
        let compile = std::process::Command::new("cc").arg(&c).arg(fixture.path.join(format!("{host}.s")))
            .arg("-o").arg(&executable).output().unwrap();
        assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
        let run = std::process::Command::new(&executable).output().unwrap();
        assert!(run.status.success(), "callback ABI failed: {:?}, {}", run.status.code(), String::from_utf8_lossy(&run.stderr));
    }
}

/// Emits one compact table and two callbacks, independent of the number of include sites.
pub(in crate::codegen) fn emit_state_helpers(module: &Module, emitter: &mut Emitter, data: &mut DataSection) {
    if !module.required_runtime_features.eval_bridge { return; }
    let mut words = Vec::new();
    if let Some(catalog) = module.source_catalog() {
        for (_, source) in catalog.iter() {
            let (path, length) = data.add_string(source.canonical_path.as_os_str().as_encoded_bytes());
            let cell = data.add_comm(include_guard_symbol(&source.canonical_path), 8);
            let preincluded = u64::from(module.preincluded_sources.contains(&source.canonical_path));
            words.extend([
                DataWord::Symbol(path),
                DataWord::U64(length as u64),
                DataWord::Symbol(cell),
                DataWord::U64(preincluded),
            ]);
        }
    }
    let count = words.len() / 4;
    let table = data.add_words(words);
    emit_lookup(emitter, LOOKUP, "source_lookup", &table, count);
    emit_reset(emitter, RESET, "source_reset", &table, count);
    emit_prime(emitter, &table, count);
    emit_deferred_class_helpers(module, emitter, data);
}

/// Emits the load-state table for classes the closed world carries only to answer a probe.
///
/// Same shape as the source table: one record of {name pointer, name length, flag cell,
/// placeholder} so both share the compact lookup and reset loops. Names are LOWERCASE, because
/// php class names are case-insensitive and the caller lowercases before asking.
fn emit_deferred_class_helpers(module: &Module, emitter: &mut Emitter, data: &mut DataSection) {
    let mut words = Vec::new();
    for name in &module.deferred_class_loads {
        let (label, length) = data.add_string(name.as_bytes());
        let cell = data.add_comm(deferred_class_symbol(name), 8);
        words.extend([
            DataWord::Symbol(label),
            DataWord::U64(length as u64),
            DataWord::Symbol(cell),
            DataWord::U64(0),
        ]);
    }
    let count = words.len() / 4;
    let table = data.add_words(words);
    let lookup_symbol = emitter.target.extern_symbol(CLASS_LOOKUP);
    emit_lookup(emitter, &lookup_symbol, "class_lookup", &table, count);
    emit_reset(emitter, CLASS_RESET, "class_reset", &table, count);
}

/// Encodes one lowercase class name into an assembly-safe, collision-free flag symbol.
pub(in crate::codegen) fn deferred_class_symbol(name: &str) -> String {
    use std::fmt::Write;
    let mut symbol = String::from("_class_deferred_");
    for byte in name.as_bytes() {
        let _ = write!(symbol, "{byte:02x}");
    }
    symbol
}

/// Installs the program's state access before PHP execution; also safe to repeat at eval entry.
pub(in crate::codegen) fn emit_state_install(ctx: &mut FunctionContext<'_>) {
    if !ctx.module.required_runtime_features.eval_bridge { return; }
    let a0 = abi::int_arg_reg_name(ctx.emitter.target, 0);
    let a1 = abi::int_arg_reg_name(ctx.emitter.target, 1);
    let a2 = abi::int_arg_reg_name(ctx.emitter.target, 2);
    abi::emit_load_int_immediate(ctx.emitter, a0, 0);
    abi::emit_symbol_address(ctx.emitter, a1, LOOKUP);
    abi::emit_symbol_address(ctx.emitter, a2, RESET);
    let install = ctx.emitter.target.extern_symbol("__elephc_eval_register_native_include_state");
    abi::emit_call_label(ctx.emitter, &install);
    let ready = ctx.next_label("include_state_ready");
    abi::emit_branch_if_int_result_zero(ctx.emitter, &ready);
    abi::emit_exit(ctx.emitter, 1);
    ctx.emitter.label(&ready);
    // Re-assert the inclusions the COMPILER performed. This runs at CLI entry, at every `--web`
    // request (after `__rt_web_reset` cleared the table) and at eval entry, and only ever raises
    // a flag the request reset lowered, so repeating it is exactly a no-op.
    abi::emit_call_label(ctx.emitter, PRIME);
}

fn emit_lookup(emitter: &mut Emitter, symbol: &str, tag: &str, table: &str, count: usize) {
    let prefix = emitter.target.platform.local_label_prefix();
    let again = format!("{prefix}{tag}_again");
    let next = format!("{prefix}{tag}_next");
    let missing = format!("{prefix}{tag}_missing");
    let done = format!("{prefix}{tag}_done");
    let result = abi::int_result_reg(emitter);
    let a0 = abi::int_arg_reg_name(emitter.target, 0);
    let a1 = abi::int_arg_reg_name(emitter.target, 1);
    let a2 = abi::int_arg_reg_name(emitter.target, 2);
    let scratch = abi::temp_int_reg(emitter.target);
    if emitter.target.arch == Arch::AArch64 { emitter.raw(".align 2"); }
    emitter.label_global(symbol);
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, a0, PATH);
    abi::store_at_offset(emitter, a1, LENGTH);
    abi::emit_symbol_address(emitter, result, table);
    abi::store_at_offset(emitter, result, CURSOR);
    abi::emit_load_int_immediate(emitter, result, count as i64);
    abi::store_at_offset(emitter, result, REMAINING);
    emitter.label(&again);
    abi::load_at_offset(emitter, result, REMAINING);
    abi::emit_branch_if_int_result_zero(emitter, &missing);
    abi::load_at_offset(emitter, result, CURSOR);
    abi::emit_load_from_address(emitter, a2, result, 8);
    abi::load_at_offset(emitter, scratch, LENGTH);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {a2}, {scratch}"));               // compare exact path byte lengths before reading bytes
            emitter.instruction(&format!("b.ne {next}"));                       // skip entries with a different length
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {a2}, {scratch}"));               // compare exact path byte lengths before reading bytes
            emitter.instruction(&format!("jne {next}"));                        // skip entries with a different length
        }
    }
    abi::emit_load_from_address(emitter, a1, result, 0);
    abi::load_at_offset(emitter, a0, PATH);
    let compare = emitter.target.extern_symbol("memcmp");
    abi::emit_call_label(emitter, &compare);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("cbnz w0, {next}")),      // test the C int result without depending on upper bits
        Arch::X86_64 => {
            emitter.instruction("test eax, eax");                               // test the C int result without depending on upper bits
            emitter.instruction(&format!("jnz {next}"));                        // continue when the path bytes differ
        }
    }
    abi::load_at_offset(emitter, result, CURSOR);
    abi::emit_load_from_address(emitter, result, result, 16);
    abi::emit_jump(emitter, &done);
    emitter.label(&next);
    abi::load_at_offset(emitter, result, CURSOR);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("add {result}, {result}, #{RECORD}")), // advance one source record
        Arch::X86_64 => emitter.instruction(&format!("add {result}, {RECORD}")), // advance one source record
    }
    abi::store_at_offset(emitter, result, CURSOR);
    abi::load_at_offset(emitter, result, REMAINING);
    match emitter.target.arch {
        Arch::AArch64 => emitter.instruction(&format!("sub {result}, {result}, #1")), // consume one candidate record
        Arch::X86_64 => emitter.instruction(&format!("sub {result}, 1")),       // consume one candidate record
    }
    abi::store_at_offset(emitter, result, REMAINING);
    abi::emit_jump(emitter, &again);
    emitter.label(&missing);
    abi::emit_load_int_immediate(emitter, result, 0);
    emitter.label(&done);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

fn emit_reset(emitter: &mut Emitter, symbol: &str, tag: &str, table: &str, count: usize) {
    let prefix = emitter.target.platform.local_label_prefix();
    let again = format!("{prefix}{tag}_again");
    let done = format!("{prefix}{tag}_done");
    let cursor = abi::int_result_reg(emitter);
    let remaining = abi::int_arg_reg_name(emitter.target, 1);
    let cell = abi::temp_int_reg(emitter.target);
    if emitter.target.arch == Arch::AArch64 { emitter.raw(".align 2"); }
    emitter.label_global(symbol);
    abi::emit_frame_prologue(emitter, 16);
    abi::emit_symbol_address(emitter, cursor, table);
    abi::emit_load_int_immediate(emitter, remaining, count as i64);
    emitter.label(&again);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cbz {remaining}, {done}"));           // stop after every native source cell was reset
            emitter.instruction(&format!("ldr {cell}, [{cursor}, #16]"));       // load the current record's guard address
            emitter.instruction(&format!("str xzr, [{cell}]"));                 // clear actual native inclusion state
            emitter.instruction(&format!("add {cursor}, {cursor}, #{RECORD}")); // advance one source record
            emitter.instruction(&format!("sub {remaining}, {remaining}, #1"));  // consume one source record
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("test {remaining}, {remaining}"));     // test whether any source records remain
            emitter.instruction(&format!("jz {done}"));                         // stop after every native source cell was reset
            emitter.instruction(&format!("mov {cell}, QWORD PTR [{cursor} + 16]")); // load the current record's guard address
            emitter.instruction(&format!("mov QWORD PTR [{cell}], 0"));         // clear actual native inclusion state
            emitter.instruction(&format!("add {cursor}, {RECORD}"));            // advance one source record
            emitter.instruction(&format!("sub {remaining}, 1"));                // consume one source record
        }
    }
    abi::emit_jump(emitter, &again);
    emitter.label(&done);
    abi::emit_frame_restore(emitter, 16);
    abi::emit_return(emitter);
}

/// Emits `__rt_source_include_prime`, which raises the guard of every source the COMPILER
/// already included.
///
/// The autoload pass opened those files at compile time and spliced their declarations into the
/// program, which is the same observable outcome PHP reaches by running the autoloader: the
/// symbols exist and the file counts as included. Without this, an interpreted `include_once`
/// of such a path found its guard clear, re-ran the file, and died redeclaring a class the
/// binary already carries.
///
/// Only raises flags — never clears one — so it is safe to repeat at every entry that installs
/// the include state.
fn emit_prime(emitter: &mut Emitter, table: &str, count: usize) {
    let prefix = emitter.target.platform.local_label_prefix();
    let again = format!("{prefix}source_prime_again");
    let next = format!("{prefix}source_prime_next");
    let done = format!("{prefix}source_prime_done");
    let cursor = abi::int_result_reg(emitter);
    let remaining = abi::int_arg_reg_name(emitter.target, 1);
    let cell = abi::temp_int_reg(emitter.target);
    let flag = abi::int_arg_reg_name(emitter.target, 2);
    if emitter.target.arch == Arch::AArch64 { emitter.raw(".align 2"); }
    emitter.label_global(PRIME);
    abi::emit_frame_prologue(emitter, 16);
    abi::emit_symbol_address(emitter, cursor, table);
    abi::emit_load_int_immediate(emitter, remaining, count as i64);
    emitter.label(&again);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cbz {remaining}, {done}"));           // stop after every source record was inspected
            emitter.instruction(&format!("ldr {flag}, [{cursor}, #24]"));       // load the compiler-included flag
            emitter.instruction(&format!("cbz {flag}, {next}"));                // leave a runtime-only source untouched
            emitter.instruction(&format!("ldr {cell}, [{cursor}, #16]"));       // load the current record's guard address
            emitter.instruction(&format!("str {flag}, [{cell}]"));              // record the compile-time inclusion
            emitter.label(&next);
            emitter.instruction(&format!("add {cursor}, {cursor}, #{RECORD}")); // advance one source record
            emitter.instruction(&format!("sub {remaining}, {remaining}, #1"));  // consume one source record
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("test {remaining}, {remaining}"));     // test whether any source records remain
            emitter.instruction(&format!("jz {done}"));                         // stop after every source record was inspected
            emitter.instruction(&format!("mov {flag}, QWORD PTR [{cursor} + 24]")); // load the compiler-included flag
            emitter.instruction(&format!("test {flag}, {flag}"));               // test the compiler-included flag
            emitter.instruction(&format!("jz {next}"));                         // leave a runtime-only source untouched
            emitter.instruction(&format!("mov {cell}, QWORD PTR [{cursor} + 16]")); // load the current record's guard address
            emitter.instruction(&format!("mov QWORD PTR [{cell}], {flag}"));    // record the compile-time inclusion
            emitter.label(&next);
            emitter.instruction(&format!("add {cursor}, {RECORD}"));            // advance one source record
            emitter.instruction(&format!("sub {remaining}, 1"));                // consume one source record
        }
    }
    abi::emit_jump(emitter, &again);
    emitter.label(&done);
    abi::emit_frame_restore(emitter, 16);
    abi::emit_return(emitter);
}

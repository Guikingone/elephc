//! Purpose:
//! Implements fail-closed strict PHP equality for exact WASM value shapes and
//! the length-delimited byte-string helper used by `===` and `!==`.
//!
//! Called from:
//! - `crate::codegen_wasm::inst::lower_instruction()` for strict comparisons.
//! - `crate::codegen_wasm::plan::plan_module()` to register the string runtime.
//! - `crate::codegen_wasm::capability` to share the admitted value families.
//!
//! Key details:
//! - Type identity is based on exact PHP/EIR metadata, never `codegen_repr()`;
//!   resources must not collapse into integers for strict comparison.
//! - Strings are compared by length and raw bytes, including embedded NUL and
//!   invalid UTF-8; operands remain borrowed.
//! - Mixed, unions, tagged scalars, containers, callables, resources, pointers,
//!   and packed values remain outside this deliberately narrow batch.

use super::context::{FnCtx, Result};
use super::inst::{operand, store_result};
use super::wat::WatModule;
use super::WasmError;
use crate::ir::{Instruction, IrHeapKind, IrType, Op, Ownership};
use crate::types::PhpType;

/// Exact value families whose PHP strict identity is implemented by WASM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StrictValueKind {
    Int,
    Bool,
    Null,
    Float,
    Str,
    Object,
}

/// Classifies one exact EIR/PHP/ownership shape for strict comparison.
pub(super) fn classify_strict_value(
    ir_type: IrType,
    php_type: &PhpType,
    ownership: Ownership,
) -> Option<StrictValueKind> {
    match (ir_type, php_type, ownership) {
        (IrType::I64, PhpType::Int, Ownership::NonHeap) => Some(StrictValueKind::Int),
        (
            IrType::I64,
            PhpType::Bool | PhpType::False,
            Ownership::NonHeap,
        ) => Some(StrictValueKind::Bool),
        (IrType::I64, PhpType::Void, Ownership::NonHeap) => Some(StrictValueKind::Null),
        (IrType::F64, PhpType::Float, Ownership::NonHeap) => Some(StrictValueKind::Float),
        (
            IrType::Str,
            PhpType::Str,
            Ownership::Owned
            | Ownership::Borrowed
            | Ownership::MaybeOwned
            | Ownership::Persistent,
        ) => Some(StrictValueKind::Str),
        (
            IrType::Heap(IrHeapKind::Object),
            PhpType::Object(_),
            Ownership::Owned
            | Ownership::Borrowed
            | Ownership::MaybeOwned
            | Ownership::Persistent,
        ) => Some(StrictValueKind::Object),
        _ => None,
    }
}

/// Lowers exact strict equality or inequality without PHP coercion.
pub(super) fn lower_strict_compare(ctx: &mut FnCtx, inst: &Instruction) -> Result<()> {
    let lhs = operand(inst, 0)?;
    let rhs = operand(inst, 1)?;
    let lhs_value = ctx
        .function
        .value(lhs)
        .ok_or_else(|| WasmError::Unsupported(format!("missing strict lhs {:?}", lhs)))?;
    let rhs_value = ctx
        .function
        .value(rhs)
        .ok_or_else(|| WasmError::Unsupported(format!("missing strict rhs {:?}", rhs)))?;
    let lhs_kind = classify_strict_value(
        lhs_value.ir_type,
        &lhs_value.php_type,
        lhs_value.ownership,
    )
    .ok_or_else(|| {
        WasmError::Unsupported(format!(
            "strict lhs shape {:?}/{:?}/{:?}",
            lhs_value.ir_type, lhs_value.php_type, lhs_value.ownership
        ))
    })?;
    let rhs_kind = classify_strict_value(
        rhs_value.ir_type,
        &rhs_value.php_type,
        rhs_value.ownership,
    )
    .ok_or_else(|| {
        WasmError::Unsupported(format!(
            "strict rhs shape {:?}/{:?}/{:?}",
            rhs_value.ir_type, rhs_value.php_type, rhs_value.ownership
        ))
    })?;
    let negated = inst.op == Op::StrictNotEq;

    if lhs_kind != rhs_kind {
        ctx.fb.ins(
            if negated {
                "i64.const 1"
            } else {
                "i64.const 0"
            },
            "different exact PHP types compare strictly unequal",
        );
        return store_result(ctx, inst);
    }

    match lhs_kind {
        StrictValueKind::Int | StrictValueKind::Bool | StrictValueKind::Null => {
            ctx.emit_load_value(lhs)?;
            ctx.emit_load_value(rhs)?;
            ctx.fb.ins(
                if negated { "i64.ne" } else { "i64.eq" },
                "strict integer-backed equality",
            );
            finish_i32_boolean(ctx, inst, false)
        }
        StrictValueKind::Float => {
            ctx.emit_load_value(lhs)?;
            ctx.emit_load_value(rhs)?;
            ctx.fb.ins(
                if negated { "f64.ne" } else { "f64.eq" },
                "strict PHP float equality",
            );
            finish_i32_boolean(ctx, inst, false)
        }
        StrictValueKind::Str => {
            ctx.emit_load_value(lhs)?;
            ctx.emit_load_value(rhs)?;
            ctx.fb.ins(
                "call $__rt_strict_str_eq",
                "compare strict strings by length and bytes",
            );
            finish_i32_boolean(ctx, inst, negated)
        }
        StrictValueKind::Object => {
            ctx.emit_load_value(lhs)?;
            ctx.emit_load_value(rhs)?;
            ctx.fb.ins(
                if negated { "i32.ne" } else { "i32.eq" },
                "compare strict object identity",
            );
            finish_i32_boolean(ctx, inst, false)
        }
    }
}

/// Converts an i32 comparison result into the EIR i64 boolean representation.
fn finish_i32_boolean(ctx: &mut FnCtx, inst: &Instruction, negate: bool) -> Result<()> {
    if negate {
        ctx.fb.ins("i32.eqz", "invert strict string equality");
    }
    ctx.fb.ins("i64.extend_i32_u", "strict bool i32 -> i64");
    store_result(ctx, inst)
}

const RT_STRICT_STR_EQ: &str = r#"(func $__rt_strict_str_eq
  (param $ap i32) (param $al i64) (param $bp i32) (param $bl i64) (result i32)
  (local $i i64)
  (if (i64.ne (local.get $al) (local.get $bl))
    (then (return (i32.const 0))))
  (loop $scan
    (if (i64.ge_u (local.get $i) (local.get $al))
      (then (return (i32.const 1))))
    (if
      (i32.ne
        (i32.load8_u
          (i32.add (local.get $ap) (i32.wrap_i64 (local.get $i))))
        (i32.load8_u
          (i32.add (local.get $bp) (i32.wrap_i64 (local.get $i)))))
      (then (return (i32.const 0))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $scan))
  (i32.const 0))
"#;

/// Registers the borrowed, length-delimited strict string comparison runtime.
pub(super) fn emit_strict_runtime(module: &mut WatModule) {
    module.add_raw_func(RT_STRICT_STR_EQ);
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! WAT validation and Wasmer regressions for strict binary string equality.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Raw data segments exercise empty, prefix, embedded-NUL, and invalid
    //!   UTF-8 bytes without passing through Rust or PHP string decoding.

    use super::*;
    use crate::codegen::Emit;
    use crate::codegen_wasm::wat::DataSegment;
    use crate::ir::{Builder, Function, Module, Terminator};
    use crate::codegen_support::platform::Target;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

    /// Returns whether the Wasmer CLI is available for runtime execution.
    fn wasmer_available() -> bool {
        std::process::Command::new("wasmer")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Builds, validates, and invokes the strict-string runtime driver.
    fn run_strict_string_driver() -> Option<String> {
        let mut module = WatModule::new();
        module.set_memory(1, Some("memory"));
        emit_strict_runtime(&mut module);
        for (offset, bytes) in [
            (32, vec![b'a', 0, b'b', 0xff]),
            (64, vec![b'a', 0, b'b', 0xff]),
            (96, vec![b'a', 0, b'b']),
            (128, vec![b'a', 0, b'c', 0xff]),
        ] {
            module.add_data(DataSegment { offset, bytes });
        }
        module.add_raw_func(
            r#"(func $t (export "t") (result i32)
  (call $__rt_strict_str_eq (i32.const 0) (i64.const 0) (i32.const 0) (i64.const 0))
  (i32.const 1000)
  i32.mul
  (call $__rt_strict_str_eq (i32.const 32) (i64.const 4) (i32.const 64) (i64.const 4))
  (i32.const 100)
  i32.mul
  i32.add
  (call $__rt_strict_str_eq (i32.const 32) (i64.const 4) (i32.const 96) (i64.const 3))
  i32.eqz
  (i32.const 10)
  i32.mul
  i32.add
  (call $__rt_strict_str_eq (i32.const 32) (i64.const 4) (i32.const 128) (i64.const 4))
  i32.eqz
  i32.add)
"#,
        );
        let wat = module.render();
        let bytes =
            ::wat::parse_str(&wat).unwrap_or_else(|error| panic!("WAT failed: {error}\n{wat}"));
        wasmparser::validate(&bytes)
            .unwrap_or_else(|error| panic!("WASM validation failed: {error}\n{wat}"));
        if !wasmer_available() {
            return None;
        }

        let sequence = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "elephc_wasm_strict_{}_{}",
            std::process::id(),
            sequence
        ));
        std::fs::create_dir_all(&dir).expect("strict temp directory");
        let path = dir.join("strict.wasm");
        std::fs::write(&path, bytes).expect("strict wasm artifact");
        let output = std::process::Command::new("wasmer")
            .arg("run")
            .arg("--invoke")
            .arg("t")
            .arg(&path)
            .output()
            .expect("invoke strict wasm driver");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            output.status.success(),
            "strict driver failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Verifies both strict opcodes pass capability planning, assemble, validate,
    /// and execute with the EIR i64 boolean result convention.
    #[test]
    fn strict_scalar_equality_opcodes_lower_and_run() {
        let mut module = Module::new(Target::wasm());
        let delimiter = module.data.intern_string(",");
        let mut function = Function::new("main".to_string(), IrType::Void, PhpType::Void);
        function.flags.is_main = true;
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", Vec::new());
            builder.set_entry(entry);
            builder.position_at_end(entry);

            for (op, lhs, rhs) in [
                (Op::StrictEq, 7, 7),
                (Op::StrictNotEq, 7, 8),
                (Op::StrictEq, 7, 8),
                (Op::StrictNotEq, 7, 7),
            ] {
                let lhs = builder.emit_const_i64(lhs);
                let rhs = builder.emit_const_i64(rhs);
                let result = builder
                    .emit(
                        op,
                        vec![lhs, rhs],
                        None,
                        IrType::I64,
                        PhpType::Bool,
                        Ownership::NonHeap,
                    )
                    .expect("strict scalar result");
                let _ = builder.emit(
                    Op::EchoValue,
                    vec![result],
                    None,
                    IrType::Void,
                    PhpType::Void,
                    Ownership::NonHeap,
                );
                let delimiter = builder.emit_const_str(delimiter);
                let _ = builder.emit(
                    Op::EchoValue,
                    vec![delimiter],
                    None,
                    IrType::Void,
                    PhpType::Void,
                    Ownership::NonHeap,
                );
            }
            builder.terminate(Terminator::Return { value: None });
        }
        module.add_function(function);

        let wat = crate::codegen_wasm::generate(&module, Emit::Executable)
            .expect("strict scalar module");
        let bytes =
            ::wat::parse_str(&wat).unwrap_or_else(|error| panic!("WAT failed: {error}\n{wat}"));
        wasmparser::validate(&bytes)
            .unwrap_or_else(|error| panic!("WASM validation failed: {error}\n{wat}"));
        if !wasmer_available() {
            return;
        }

        let sequence = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "elephc_wasm_strict_eir_{}_{}",
            std::process::id(),
            sequence
        ));
        std::fs::create_dir_all(&dir).expect("strict EIR temp directory");
        let path = dir.join("strict-eir.wasm");
        std::fs::write(&path, bytes).expect("strict EIR wasm artifact");
        let output = std::process::Command::new("wasmer")
            .arg("run")
            .arg(&path)
            .output()
            .expect("run strict EIR module");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            output.status.success(),
            "strict EIR module failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "1,1,,,");
        assert!(output.stderr.is_empty());
    }

    /// Verifies strict strings compare by exact length and raw byte content.
    #[test]
    fn strict_binary_string_equality_is_length_delimited() {
        if let Some(output) = run_strict_string_driver() {
            assert_eq!(output, "1111");
        }
    }
}

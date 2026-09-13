//! Purpose:
//! Lowers the typed PHP 8.5 `clone()` runtime operation.
//!
//! Called from:
//! - `crate::codegen::lower_inst::runtime_functions::group_02` for
//!   `RuntimeFnId::CloneWith`.
//!
//! Key details:
//! - Runtime-class cloning and post-hook property initialization require one shared,
//!   target-aware helper. The explicit diagnostic remains until that helper lands.

use crate::codegen::context::FunctionContext;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::Instruction;

/// Rejects the not-yet-connected runtime helper without panicking the compiler.
pub(crate) fn lower_clone_with(
    _ctx: &mut FunctionContext<'_>,
    _inst: &Instruction,
) -> Result<()> {
    Err(CodegenIrError::unsupported(
        "clone() runtime-class lowering is not connected",
    ))
}

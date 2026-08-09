//! Purpose:
//! Emits the process-wide user-stack budget guard used by generated functions.
//! It fails closed before native call-stack exhaustion on every supported target.
//!
//! Called from:
//! - `crate::codegen::frame` at user-function entry and normal return paths.
//!
//! Key details:
//! - The counter tracks bytes reserved by active direct-callable frames and the fatal path must not return.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::data::STACK_OVERFLOW_MSG;

/// The highest generated PHP frame allocation accepted before terminating.
// The guard counts exact `FunctionContext::frame_size` allocations, not calls:
// a single spill-heavy frame must not bypass the same native-stack budget that
// bounds many small frames. This stays below the usual 8 MiB process stack once
// return addresses, frame pointers, and host/runtime frames are included.
const MAX_RUNTIME_STACK_BYTES: usize = 4 * 1024 * 1024;

/// Emits stack-budget enter and leave helpers accepting one ABI frame-byte argument.
pub(crate) fn emit_recursion_guard(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_recursion_guard_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: recursion_stack_budget ---");
    emitter.label_global("__rt_recursion_stack_bytes_enter");
    abi::emit_symbol_address(emitter, "x9", "_runtime_recursion_stack_bytes");
    emitter.instruction("ldr x10, [x9]");                                       // load active generated-frame bytes
    emitter.instruction("add x10, x10, x0");                                    // account for this exact frame allocation
    emitter.instruction("str x10, [x9]");                                       // publish bytes before checking them
    emitter.instruction(&format!("cmp x10, #{}", MAX_RUNTIME_STACK_BYTES));     // enforce a bounded native stack
    emitter.instruction("b.ls __rt_recursion_stack_bytes_ok");                  // current frame is within the budget
    emitter.instruction("mov x0, #2");                                          // stderr file descriptor
    abi::emit_symbol_address(emitter, "x1", "_runtime_recursion_depth_msg");
    emitter.instruction(&format!("mov x2, #{}", STACK_OVERFLOW_MSG.len()));     // write complete fatal diagnostic
    emitter.syscall(4);                                                          // write(2, message, length)
    emitter.instruction("mov x0, #1");                                          // non-zero fatal exit status
    emitter.syscall(1);                                                          // exit without returning to the overflowing caller
    emitter.label("__rt_recursion_stack_bytes_ok");
    emitter.instruction("ret");                                                 // caller may execute its function body

    emitter.label_global("__rt_recursion_stack_bytes_leave");
    abi::emit_symbol_address(emitter, "x9", "_runtime_recursion_stack_bytes");
    emitter.instruction("ldr x10, [x9]");                                       // load active generated-frame bytes
    emitter.instruction("sub x10, x10, x0");                                    // release this exact frame allocation
    emitter.instruction("str x10, [x9]");                                       // publish bytes for sibling calls
    emitter.instruction("ret");                                                 // resume the ordinary function epilogue
}

/// Emits the Linux x86_64 implementation of the user-stack budget guard.
fn emit_recursion_guard_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: recursion_stack_budget ---");
    emitter.label_global("__rt_recursion_stack_bytes_enter");
    abi::emit_load_symbol_to_reg(emitter, "rax", "_runtime_recursion_stack_bytes", 0); // load active generated-frame bytes
    emitter.instruction("add rax, rdi");                                        // account for this exact frame allocation
    abi::emit_store_reg_to_symbol(emitter, "rax", "_runtime_recursion_stack_bytes", 0); // publish bytes before checking them
    emitter.instruction(&format!("cmp rax, {}", MAX_RUNTIME_STACK_BYTES));      // enforce a bounded native stack
    emitter.instruction("jbe __rt_recursion_stack_bytes_ok_x");                 // current frame is within the budget
    emitter.instruction("mov edi, 2");                                          // stderr file descriptor
    abi::emit_symbol_address(emitter, "rsi", "_runtime_recursion_depth_msg");
    emitter.instruction(&format!("mov edx, {}", STACK_OVERFLOW_MSG.len()));     // write complete fatal diagnostic
    emitter.instruction("mov eax, 1");                                          // Linux write syscall number
    emitter.instruction("syscall");                                             // write(2, message, length)
    emitter.instruction("mov edi, 1");                                          // non-zero fatal exit status
    emitter.instruction("mov eax, 60");                                         // Linux exit syscall number
    emitter.instruction("syscall");                                             // exit without returning to the overflowing caller
    emitter.label("__rt_recursion_stack_bytes_ok_x");
    emitter.instruction("ret");                                                 // caller may execute its function body

    emitter.label_global("__rt_recursion_stack_bytes_leave");
    abi::emit_load_symbol_to_reg(emitter, "rax", "_runtime_recursion_stack_bytes", 0); // load active generated-frame bytes
    emitter.instruction("sub rax, rdi");                                        // release this exact frame allocation
    abi::emit_store_reg_to_symbol(emitter, "rax", "_runtime_recursion_stack_bytes", 0); // publish bytes for sibling calls
    emitter.instruction("ret");                                                 // resume the ordinary function epilogue
}

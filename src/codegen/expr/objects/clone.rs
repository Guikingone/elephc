use crate::codegen::abi;
use crate::codegen::context::Context;
use crate::codegen::data_section::DataSection;
use crate::codegen::emit::Emitter;
use crate::codegen::platform::Arch;
use crate::names::method_symbol;
use crate::parser::ast::Expr;
use crate::types::PhpType;

use super::super::{
    emit_expr, restore_concat_offset_after_nested_call, save_concat_offset_before_nested_call,
};

const X86_64_HEAP_MAGIC_HI32: u64 = 0x454C5048;

/// Emit ARM64/x86_64 code for `clone <expr>`.
/// Allocates a new object of the same class, copies the class id and every
/// property slot, then increments refcounts on each refcounted slot so that
/// strings, arrays and child objects survive after the source is dropped.
pub(super) fn emit_clone(
    inner: &Expr,
    emitter: &mut Emitter,
    ctx: &mut Context,
    data: &mut DataSection,
) -> PhpType {
    let inner_ty = emit_expr(inner, emitter, ctx, data);
    let class_name = match inner_ty {
        PhpType::Object(name) => name,
        _ => {
            emitter.comment("WARNING: clone applied to non-object — emitting no-op");
            return PhpType::Void;
        }
    };

    let class_info = match ctx.classes.get(&class_name).cloned() {
        Some(c) => c,
        None => {
            emitter.comment(&format!("WARNING: clone target {} has no class info", class_name));
            return PhpType::Object(class_name);
        }
    };
    let num_props = class_info.properties.len();
    let obj_size = 8 + num_props * 16;

    emitter.comment(&format!("clone {}", class_name));

    abi::emit_push_reg(emitter, abi::int_result_reg(emitter));                  // save source object pointer

    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("mov x0, #{}", obj_size));             // size of cloned object
            emitter.instruction("bl __rt_heap_alloc");                          // allocate new object -> x0 = dest pointer
            emitter.instruction("mov x9, #4");                                  // heap kind 4 = object instance
            emitter.instruction("str x9, [x0, #-8]");                           // stamp the cloned object header
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov rax, {}", obj_size));             // size of cloned object
            abi::emit_call_label(emitter, "__rt_heap_alloc");                   // allocate new object -> rax = dest pointer
            emitter.instruction(&format!(
                "mov r10, 0x{:x}",
                (X86_64_HEAP_MAGIC_HI32 << 32) | 4
            ));                                                                 // materialize the x86_64 object header word
            emitter.instruction("mov QWORD PTR [rax - 8], r10");                // stamp the cloned object header
        }
    }

    abi::emit_push_reg(emitter, abi::int_result_reg(emitter));                  // save dest pointer below source on the stack

    // Stack layout now (top to bottom):  [dest][source]
    // We peek source through index 8 and dest at top.

    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x10, [sp, #16]");                          // reload source pointer from saved slot
            emitter.instruction("ldr x11, [sp]");                               // reload dest pointer from saved slot
            emitter.instruction("ldr x12, [x10]");                              // load source class id
            emitter.instruction("str x12, [x11]");                              // copy class id into the cloned object
        }
        Arch::X86_64 => {
            emitter.instruction("mov r10, QWORD PTR [rsp + 16]");               // reload source pointer from the saved stack slot
            emitter.instruction("mov r11, QWORD PTR [rsp]");                    // reload dest pointer from the saved stack slot
            emitter.instruction("mov r12, QWORD PTR [r10]");                    // load source class id
            emitter.instruction("mov QWORD PTR [r11], r12");                    // copy class id into the cloned object
        }
    }

    for i in 0..num_props {
        let prop_ty = class_info.properties[i].1.clone();
        let value_offset = 8 + i * 16;
        let tag_offset = value_offset + 8;
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction("ldr x10, [sp, #16]");                      // reload source pointer
                emitter.instruction("ldr x11, [sp]");                           // reload dest pointer
                if matches!(prop_ty, PhpType::Str) {
                    emitter.instruction(&format!("ldr x1, [x10, #{}]", value_offset)); // load source string pointer
                    emitter.instruction(&format!("ldr x2, [x10, #{}]", tag_offset));   // load source string length
                    emitter.instruction("bl __rt_str_persist");                 // copy the string into a fresh heap allocation owned by the clone
                    emitter.instruction("ldr x11, [sp]");                       // reload dest pointer (str_persist clobbers x11)
                    emitter.instruction(&format!("str x1, [x11, #{}]", value_offset)); // store cloned string pointer
                    emitter.instruction(&format!("str x2, [x11, #{}]", tag_offset));   // store cloned string length
                } else {
                    emitter.instruction(&format!("ldr x12, [x10, #{}]", value_offset)); // copy property value word
                    emitter.instruction(&format!("str x12, [x11, #{}]", value_offset));
                    emitter.instruction(&format!("ldr x13, [x10, #{}]", tag_offset)); // copy property tag/length word
                    emitter.instruction(&format!("str x13, [x11, #{}]", tag_offset));
                    if prop_ty.is_refcounted() {
                        emitter.instruction("mov x0, x12");                     // pass the shared payload pointer to the refcount helper
                        emitter.instruction("bl __rt_incref");                  // retain the shared payload for the cloned object
                    }
                }
            }
            Arch::X86_64 => {
                emitter.instruction("mov r10, QWORD PTR [rsp + 16]");           // reload source pointer
                emitter.instruction("mov r11, QWORD PTR [rsp]");                // reload dest pointer
                if matches!(prop_ty, PhpType::Str) {
                    emitter.instruction(&format!("mov rax, QWORD PTR [r10 + {}]", value_offset)); // load source string pointer
                    emitter.instruction(&format!("mov rdx, QWORD PTR [r10 + {}]", tag_offset));   // load source string length
                    abi::emit_call_label(emitter, "__rt_str_persist");          // copy the string into a fresh heap allocation owned by the clone
                    emitter.instruction("mov r11, QWORD PTR [rsp]");            // reload dest pointer (str_persist clobbers r11)
                    emitter.instruction(&format!("mov QWORD PTR [r11 + {}], rax", value_offset)); // store cloned string pointer
                    emitter.instruction(&format!("mov QWORD PTR [r11 + {}], rdx", tag_offset));   // store cloned string length
                } else {
                    emitter.instruction(&format!("mov r12, QWORD PTR [r10 + {}]", value_offset)); // copy property value word
                    emitter.instruction(&format!("mov QWORD PTR [r11 + {}], r12", value_offset));
                    emitter.instruction(&format!("mov r13, QWORD PTR [r10 + {}]", tag_offset)); // copy property tag/length word
                    emitter.instruction(&format!("mov QWORD PTR [r11 + {}], r13", tag_offset));
                    if prop_ty.is_refcounted() {
                        emitter.instruction("mov rax, r12");                    // pass the shared payload pointer to the refcount helper
                        abi::emit_call_label(emitter, "__rt_incref");           // retain the shared payload for the cloned object
                    }
                }
            }
        }
    }

    // Invoke __clone on the cloned object if the class (or an ancestor)
    // declares it. Matches PHP semantics: the magic method runs on the new
    // instance only and lets user code finish a deep copy if needed.
    if class_info.methods.contains_key("__clone") {
        let impl_class = class_info
            .method_impl_classes
            .get("__clone")
            .map(String::as_str)
            .unwrap_or(class_name.as_str())
            .to_string();
        emitter.comment(&format!("call {}::__clone on the cloned object", impl_class));
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction("ldr x0, [sp]");                            // load dest pointer (top of stack) into the AArch64 first-arg register as $this
            }
            Arch::X86_64 => {
                emitter.instruction("mov rdi, QWORD PTR [rsp]");                // load dest pointer into the SysV first-arg register as $this
            }
        }
        save_concat_offset_before_nested_call(emitter, ctx);
        abi::emit_call_label(emitter, &method_symbol(&impl_class, "__clone"));  // invoke the resolved __clone implementation for the cloned object
        restore_concat_offset_after_nested_call(emitter, ctx, &PhpType::Void);
    }

    abi::emit_pop_reg(emitter, abi::int_result_reg(emitter));                   // pop dest pointer into the result register
    let scratch = abi::symbol_scratch_reg(emitter);
    abi::emit_pop_reg(emitter, scratch);                                        // discard the saved source pointer

    PhpType::Object(class_name)
}

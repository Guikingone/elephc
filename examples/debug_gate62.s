    ; @fn name=_class_propinit_2 symbol=_class_propinit_2 synthetic=1
.align 2

.globl _class_propinit_2
_class_propinit_2:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_2_stack_ok_1
    b __rt_stack_overflow
_eir__class_propinit_2_stack_ok_1:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_2_entry_0:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_2_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_2
    ; @fn name=_class_propinit_3 symbol=_class_propinit_3 synthetic=1
.align 2

.globl _class_propinit_3
_class_propinit_3:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_3_stack_ok_3
    b __rt_stack_overflow
_eir__class_propinit_3_stack_ok_3:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_3_entry_2:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_3_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_3
    ; @fn name=_class_propinit_4 symbol=_class_propinit_4 synthetic=1
.align 2

.globl _class_propinit_4
_class_propinit_4:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_4_stack_ok_5
    b __rt_stack_overflow
_eir__class_propinit_4_stack_ok_5:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_4_entry_4:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_4_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_4
    ; @fn name=_class_propinit_5 symbol=_class_propinit_5 synthetic=1
.align 2

.globl _class_propinit_5
_class_propinit_5:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_5_stack_ok_7
    b __rt_stack_overflow
_eir__class_propinit_5_stack_ok_7:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_5_entry_6:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_5_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_5
    ; @fn name=_class_propinit_8 symbol=_class_propinit_8 synthetic=1
.align 2

.globl _class_propinit_8
_class_propinit_8:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_8_stack_ok_9
    b __rt_stack_overflow
_eir__class_propinit_8_stack_ok_9:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_8_entry_8:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_8_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_8
    ; @fn name=_class_propinit_9 symbol=_class_propinit_9 synthetic=1
.align 2

.globl _class_propinit_9
_class_propinit_9:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_9_stack_ok_11
    b __rt_stack_overflow
_eir__class_propinit_9_stack_ok_11:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_9_entry_10:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_9_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_9
    ; @fn name=_class_propinit_10 symbol=_class_propinit_10 synthetic=1
.align 2

.globl _class_propinit_10
_class_propinit_10:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_10_stack_ok_13
    b __rt_stack_overflow
_eir__class_propinit_10_stack_ok_13:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_10_entry_12:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_10_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_10
    ; @fn name=_class_propinit_11 symbol=_class_propinit_11 synthetic=1
.align 2

.globl _class_propinit_11
_class_propinit_11:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_11_stack_ok_15
    b __rt_stack_overflow
_eir__class_propinit_11_stack_ok_15:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_11_entry_14:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_11_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_11
    ; @fn name=_class_propinit_12 symbol=_class_propinit_12 synthetic=1
.align 2

.globl _class_propinit_12
_class_propinit_12:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_12_stack_ok_17
    b __rt_stack_overflow
_eir__class_propinit_12_stack_ok_17:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_12_entry_16:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_12_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_12
    ; @fn name=_class_propinit_15 symbol=_class_propinit_15 synthetic=1
.align 2

.globl _class_propinit_15
_class_propinit_15:
    ; prologue
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_15_stack_ok_19
    b __rt_stack_overflow
_eir__class_propinit_15_stack_ok_19:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-16]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; @block name=entry
_eir__class_propinit_15_entry_18:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_15_epilogue:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_15
    ; @fn name=_class_propinit_16 symbol=_class_propinit_16 synthetic=1
.align 2

.globl _class_propinit_16
_class_propinit_16:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_16_stack_ok_21
    b __rt_stack_overflow
_eir__class_propinit_16_stack_ok_21:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_16_entry_20:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_16_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_16
    ; @fn name=_class_propinit_17 symbol=_class_propinit_17 synthetic=1
.align 2

.globl _class_propinit_17
_class_propinit_17:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_17_stack_ok_23
    b __rt_stack_overflow
_eir__class_propinit_17_stack_ok_23:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_17_entry_22:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_17_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_17
    ; @fn name=_class_propinit_18 symbol=_class_propinit_18 synthetic=1
.align 2

.globl _class_propinit_18
_class_propinit_18:
    ; prologue
    sub sp, sp, #272
    stp x29, x30, [sp, #256]
    add x29, sp, #256
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_18_stack_ok_25
    b __rt_stack_overflow
_eir__class_propinit_18_stack_ok_25:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #256
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    stur d8, [x29, #-240]
    stur x21, [x29, #-248]
    ; param $this from x0
    stur x0, [x29, #-232]
    ; @block name=entry
_eir__class_propinit_18_entry_24:
    ldur x0, [x29, #-232]
    stur x0, [x29, #-16]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-40]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-40]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-64]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-64]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-88]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-88]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-112]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-112]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #72]
    str xzr, [x9, #80]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-136]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-136]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #88]
    str xzr, [x9, #96]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-160]
    adrp x9, _float_1@PAGE
    add x9, x9, _float_1@PAGEOFF
    ldr d0, [x9]
    fmov d8, d0
    ldur x9, [x29, #-160]
    str x9, [sp, #-16]!
    fmov d0, d8
    ldr x9, [sp], #16
    str d0, [x9, #104]
    str xzr, [x9, #112]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-184]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-184]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #120]
    str xzr, [x9, #128]
    ldur x0, [x29, #-232]
    stur x0, [x29, #-208]
    mov x0, #0
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    stur x0, [x29, #-224]
    ldur x9, [x29, #-208]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-224]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #136]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #136]
    str xzr, [x9, #144]
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_18_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_18
    ; @fn name=_class_propinit_19 symbol=_class_propinit_19 synthetic=1
.align 2

.globl _class_propinit_19
_class_propinit_19:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_19_stack_ok_27
    b __rt_stack_overflow
_eir__class_propinit_19_stack_ok_27:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_19_entry_26:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_19_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_19
    ; @fn name=_class_propinit_20 symbol=_class_propinit_20 synthetic=1
.align 2

.globl _class_propinit_20
_class_propinit_20:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_20_stack_ok_29
    b __rt_stack_overflow
_eir__class_propinit_20_stack_ok_29:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_20_entry_28:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_20_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_20
    ; @fn name=_class_propinit_21 symbol=_class_propinit_21 synthetic=1
.align 2

.globl _class_propinit_21
_class_propinit_21:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_21_stack_ok_31
    b __rt_stack_overflow
_eir__class_propinit_21_stack_ok_31:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_21_entry_30:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_21_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_21
    ; @fn name=_class_propinit_22 symbol=_class_propinit_22 synthetic=1
.align 2

.globl _class_propinit_22
_class_propinit_22:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_22_stack_ok_33
    b __rt_stack_overflow
_eir__class_propinit_22_stack_ok_33:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_22_entry_32:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_22_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_22
    ; @fn name=_class_propinit_23 symbol=_class_propinit_23 synthetic=1
.align 2

.globl _class_propinit_23
_class_propinit_23:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_23_stack_ok_35
    b __rt_stack_overflow
_eir__class_propinit_23_stack_ok_35:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_23_entry_34:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_23_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_23
    ; @fn name=_class_propinit_24 symbol=_class_propinit_24 synthetic=1
.align 2

.globl _class_propinit_24
_class_propinit_24:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_24_stack_ok_37
    b __rt_stack_overflow
_eir__class_propinit_24_stack_ok_37:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_24_entry_36:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_24_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_24
    ; @fn name=_class_propinit_25 symbol=_class_propinit_25 synthetic=1
.align 2

.globl _class_propinit_25
_class_propinit_25:
    ; prologue
    sub sp, sp, #432
    stp x29, x30, [sp, #416]
    add x29, sp, #416
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_25_stack_ok_39
    b __rt_stack_overflow
_eir__class_propinit_25_stack_ok_39:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #408
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #400
    str x21, [x9]
    ; param $this from x0
    sub x9, x29, #392
    str x0, [x9]
    ; @block name=entry
_eir__class_propinit_25_entry_38:
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-16]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-40]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-40]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-64]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-64]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-88]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-88]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-112]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-112]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #72]
    str xzr, [x9, #80]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-136]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-136]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #88]
    str xzr, [x9, #96]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-160]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-160]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #104]
    str xzr, [x9, #112]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-184]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-184]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #120]
    str xzr, [x9, #128]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-208]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-208]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #136]
    str xzr, [x9, #144]
    sub x9, x29, #392
    ldr x0, [x9]
    stur x0, [x29, #-232]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-232]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #152]
    str xzr, [x9, #160]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #256
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #256
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #168]
    str xzr, [x9, #176]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #280
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #280
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #184]
    str xzr, [x9, #192]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #304
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #304
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #200]
    str xzr, [x9, #208]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #328
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #328
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #216]
    str xzr, [x9, #224]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #352
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #352
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #232]
    str xzr, [x9, #240]
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #376
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    sub x9, x29, #376
    ldr x9, [x9]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #248]
    str xzr, [x9, #256]
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_25_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_25
    ; @fn name=_class_propinit_26 symbol=_class_propinit_26 synthetic=1
.align 2

.globl _class_propinit_26
_class_propinit_26:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_26_stack_ok_41
    b __rt_stack_overflow
_eir__class_propinit_26_stack_ok_41:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_26_entry_40:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_26_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_26
    ; @fn name=_class_propinit_27 symbol=_class_propinit_27 synthetic=1
.align 2

.globl _class_propinit_27
_class_propinit_27:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_27_stack_ok_43
    b __rt_stack_overflow
_eir__class_propinit_27_stack_ok_43:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_27_entry_42:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_27_epilogue:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_27
    ; @fn name=_class_propinit_28 symbol=_class_propinit_28 synthetic=1
.align 2

.globl _class_propinit_28
_class_propinit_28:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_28_stack_ok_45
    b __rt_stack_overflow
_eir__class_propinit_28_stack_ok_45:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_28_entry_44:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-16]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-48]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ldur x9, [x29, #-48]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #24]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #24]
    str x2, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_28_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_28
    ; @fn name=_class_propinit_29 symbol=_class_propinit_29 synthetic=1
.align 2

.globl _class_propinit_29
_class_propinit_29:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_29_stack_ok_47
    b __rt_stack_overflow
_eir__class_propinit_29_stack_ok_47:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_29_entry_46:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-16]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-48]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ldur x9, [x29, #-48]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #24]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #24]
    str x2, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_29_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_29
    ; @fn name=_class_propinit_30 symbol=_class_propinit_30 synthetic=1
.align 2

.globl _class_propinit_30
_class_propinit_30:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_30_stack_ok_49
    b __rt_stack_overflow
_eir__class_propinit_30_stack_ok_49:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_30_entry_48:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_30_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_30
    ; @fn name=_class_propinit_33 symbol=_class_propinit_33 synthetic=1
.align 2

.globl _class_propinit_33
_class_propinit_33:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_33_stack_ok_51
    b __rt_stack_overflow
_eir__class_propinit_33_stack_ok_51:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_33_entry_50:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_33_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_33
    ; @fn name=_class_propinit_34 symbol=_class_propinit_34 synthetic=1
.align 2

.globl _class_propinit_34
_class_propinit_34:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_34_stack_ok_53
    b __rt_stack_overflow
_eir__class_propinit_34_stack_ok_53:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_34_entry_52:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_34_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_34
    ; @fn name=_class_propinit_37 symbol=_class_propinit_37 synthetic=1
.align 2

.globl _class_propinit_37
_class_propinit_37:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_37_stack_ok_55
    b __rt_stack_overflow
_eir__class_propinit_37_stack_ok_55:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_37_entry_54:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_37_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_37
    ; @fn name=_class_propinit_43 symbol=_class_propinit_43 synthetic=1
.align 2

.globl _class_propinit_43
_class_propinit_43:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_43_stack_ok_57
    b __rt_stack_overflow
_eir__class_propinit_43_stack_ok_57:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_43_entry_56:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_43_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_43
    ; @fn name=_class_propinit_44 symbol=_class_propinit_44 synthetic=1
.align 2

.globl _class_propinit_44
_class_propinit_44:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_44_stack_ok_59
    b __rt_stack_overflow
_eir__class_propinit_44_stack_ok_59:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_44_entry_58:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_44_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_44
    ; @fn name=_class_propinit_45 symbol=_class_propinit_45 synthetic=1
.align 2

.globl _class_propinit_45
_class_propinit_45:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_45_stack_ok_61
    b __rt_stack_overflow
_eir__class_propinit_45_stack_ok_61:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_45_entry_60:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_45_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_45
    ; @fn name=_class_propinit_46 symbol=_class_propinit_46 synthetic=1
.align 2

.globl _class_propinit_46
_class_propinit_46:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_46_stack_ok_63
    b __rt_stack_overflow
_eir__class_propinit_46_stack_ok_63:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_46_entry_62:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_46_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_46
    ; @fn name=_class_propinit_50 symbol=_class_propinit_50 synthetic=1
.align 2

.globl _class_propinit_50
_class_propinit_50:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_50_stack_ok_65
    b __rt_stack_overflow
_eir__class_propinit_50_stack_ok_65:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_50_entry_64:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_50_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_50
    ; @fn name=_class_propinit_51 symbol=_class_propinit_51 synthetic=1
.align 2

.globl _class_propinit_51
_class_propinit_51:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_51_stack_ok_67
    b __rt_stack_overflow
_eir__class_propinit_51_stack_ok_67:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_51_entry_66:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_51_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_51
    ; @fn name=_class_propinit_52 symbol=_class_propinit_52 synthetic=1
.align 2

.globl _class_propinit_52
_class_propinit_52:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_52_stack_ok_69
    b __rt_stack_overflow
_eir__class_propinit_52_stack_ok_69:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_52_entry_68:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_52_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_52
    ; @fn name=_class_propinit_58 symbol=_class_propinit_58 synthetic=1
.align 2

.globl _class_propinit_58
_class_propinit_58:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_58_stack_ok_71
    b __rt_stack_overflow
_eir__class_propinit_58_stack_ok_71:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_58_entry_70:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_58_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_58
    ; @fn name=_class_propinit_61 symbol=_class_propinit_61 synthetic=1
.align 2

.globl _class_propinit_61
_class_propinit_61:
    ; prologue
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_61_stack_ok_73
    b __rt_stack_overflow
_eir__class_propinit_61_stack_ok_73:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-16]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; @block name=entry
_eir__class_propinit_61_entry_72:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_61_epilogue:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_61
    ; @fn name=_class_propinit_64 symbol=_class_propinit_64 synthetic=1
.align 2

.globl _class_propinit_64
_class_propinit_64:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_64_stack_ok_75
    b __rt_stack_overflow
_eir__class_propinit_64_stack_ok_75:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_64_entry_74:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-32]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-32]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #88]
    str xzr, [x9, #96]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_64_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_64
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_65_stack_ok_77
    b __rt_stack_overflow
_eir__class_propinit_65_stack_ok_77:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_65_entry_76:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-32]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-32]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #88]
    str xzr, [x9, #96]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_65_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_65
    ; @fn name=_class_propinit_72 symbol=_class_propinit_72 synthetic=1
.align 2

.globl _class_propinit_72
_class_propinit_72:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_72_stack_ok_79
    b __rt_stack_overflow
_eir__class_propinit_72_stack_ok_79:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_72_entry_78:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_72_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_72
    ; @fn name=_class_propinit_92 symbol=_class_propinit_92 synthetic=1
.align 2

.globl _class_propinit_92
_class_propinit_92:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_92_stack_ok_81
    b __rt_stack_overflow
_eir__class_propinit_92_stack_ok_81:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_92_entry_80:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_92_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_92
    ; @fn name=_class_propinit_93 symbol=_class_propinit_93 synthetic=1
.align 2

.globl _class_propinit_93
_class_propinit_93:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_93_stack_ok_83
    b __rt_stack_overflow
_eir__class_propinit_93_stack_ok_83:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_93_entry_82:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_93_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_93
    ; @fn name=_class_propinit_94 symbol=_class_propinit_94 synthetic=1
.align 2

.globl _class_propinit_94
_class_propinit_94:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_94_stack_ok_85
    b __rt_stack_overflow
_eir__class_propinit_94_stack_ok_85:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_94_entry_84:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_94_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_94
    ; @fn name=_class_propinit_95 symbol=_class_propinit_95 synthetic=1
.align 2

.globl _class_propinit_95
_class_propinit_95:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_95_stack_ok_87
    b __rt_stack_overflow
_eir__class_propinit_95_stack_ok_87:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_95_entry_86:
    ldur x0, [x29, #-104]
    stur x0, [x29, #-24]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-96]
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-96]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_95_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_95
    ; @fn name=_class_propinit_97 symbol=_class_propinit_97 synthetic=1
.align 2

.globl _class_propinit_97
_class_propinit_97:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_97_stack_ok_89
    b __rt_stack_overflow
_eir__class_propinit_97_stack_ok_89:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_97_entry_88:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #88]
    str xzr, [x9, #96]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_97_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_97
    ; @fn name=_class_propinit_98 symbol=_class_propinit_98 synthetic=1
.align 2

.globl _class_propinit_98
_class_propinit_98:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    ; call-stack overflow guard
    adrp x9, _stack_limit@PAGE
    ldr x9, [x9, _stack_limit@PAGEOFF]
    cmp sp, x9
    b.hs _eir__class_propinit_98_stack_ok_91
    b __rt_stack_overflow
_eir__class_propinit_98_stack_ok_91:
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_98_entry_90:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-32]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-32]
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
_class_propinit_98_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @endfn name=_class_propinit_98
.align 2

    ; --- lazy materializer: backed enum PropertyHookType ---
.globl _enum_init_all_PropertyHookType_Get
_enum_init_all_PropertyHookType_Get:
    ; prologue
    sub sp, sp, #16
    stp x29, x30, [sp, #0]
    mov x29, sp
    stp x0, x1, [sp, #-144]!
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    ; skip the whole body once the cases already exist
    adrp x9, _enum_case_PropertyHookType_Get@PAGE
    add x9, x9, _enum_case_PropertyHookType_Get@PAGEOFF
    ldr x0, [x9]
    cbz x0, 1f
    b _enum_init_all_PropertyHookType_Get_done
1:
    ; materialize enum singleton PropertyHookType::Get
    mov x0, #40
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    bl __rt_object_handle_acquire
    mov x10, #101
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #32]
    adrp x10, _str_3@PAGE
    add x10, x10, _str_3@PAGEOFF
    str x10, [x0, #8]
    mov x10, #3
    str x10, [x0, #16]
    adrp x10, _str_4@PAGE
    add x10, x10, _str_4@PAGEOFF
    str x10, [x0, #24]
    mov x10, #3
    str x10, [x0, #32]
    adrp x9, _enum_case_PropertyHookType_Get@PAGE
    add x9, x9, _enum_case_PropertyHookType_Get@PAGEOFF
    str x0, [x9]
    ; materialize enum singleton PropertyHookType::Set
    mov x0, #40
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    bl __rt_object_handle_acquire
    mov x10, #101
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #32]
    adrp x10, _str_5@PAGE
    add x10, x10, _str_5@PAGEOFF
    str x10, [x0, #8]
    mov x10, #3
    str x10, [x0, #16]
    adrp x10, _str_6@PAGE
    add x10, x10, _str_6@PAGEOFF
    str x10, [x0, #24]
    mov x10, #3
    str x10, [x0, #32]
    adrp x9, _enum_case_PropertyHookType_Set@PAGE
    add x9, x9, _enum_case_PropertyHookType_Set@PAGEOFF
    str x0, [x9]
_enum_init_all_PropertyHookType_Get_done:
    ldp x16, x17, [sp, #128]
    ldp x14, x15, [sp, #112]
    ldp x12, x13, [sp, #96]
    ldp x10, x11, [sp, #80]
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp], #144
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
.align 2

    ; --- lazy materializer: enum case SortDirection::Ascending ---
.globl _enum_init_SortDirection_Ascending
_enum_init_SortDirection_Ascending:
    ; prologue
    sub sp, sp, #16
    stp x29, x30, [sp, #0]
    mov x29, sp
    stp x0, x1, [sp, #-144]!
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    ; skip the whole body once the cases already exist
    adrp x9, _enum_case_SortDirection_Ascending@PAGE
    add x9, x9, _enum_case_SortDirection_Ascending@PAGEOFF
    ldr x0, [x9]
    cbz x0, 1f
    b _enum_init_SortDirection_Ascending_done
1:
    ; materialize enum singleton SortDirection::Ascending
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    bl __rt_object_handle_acquire
    mov x10, #100
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    adrp x10, _str_7@PAGE
    add x10, x10, _str_7@PAGEOFF
    str x10, [x0, #8]
    mov x10, #9
    str x10, [x0, #16]
    adrp x9, _enum_case_SortDirection_Ascending@PAGE
    add x9, x9, _enum_case_SortDirection_Ascending@PAGEOFF
    str x0, [x9]
_enum_init_SortDirection_Ascending_done:
    ldp x16, x17, [sp, #128]
    ldp x14, x15, [sp, #112]
    ldp x12, x13, [sp, #96]
    ldp x10, x11, [sp, #80]
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp], #144
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
.align 2

    ; --- lazy materializer: enum case SortDirection::Descending ---
.globl _enum_init_SortDirection_Descending
_enum_init_SortDirection_Descending:
    ; prologue
    sub sp, sp, #16
    stp x29, x30, [sp, #0]
    mov x29, sp
    stp x0, x1, [sp, #-144]!
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    ; skip the whole body once the cases already exist
    adrp x9, _enum_case_SortDirection_Descending@PAGE
    add x9, x9, _enum_case_SortDirection_Descending@PAGEOFF
    ldr x0, [x9]
    cbz x0, 1f
    b _enum_init_SortDirection_Descending_done
1:
    ; materialize enum singleton SortDirection::Descending
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    bl __rt_object_handle_acquire
    mov x10, #100
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    adrp x10, _str_8@PAGE
    add x10, x10, _str_8@PAGEOFF
    str x10, [x0, #8]
    mov x10, #10
    str x10, [x0, #16]
    adrp x9, _enum_case_SortDirection_Descending@PAGE
    add x9, x9, _enum_case_SortDirection_Descending@PAGEOFF
    str x0, [x9]
_enum_init_SortDirection_Descending_done:
    ldp x16, x17, [sp, #128]
    ldp x14, x15, [sp, #112]
    ldp x12, x13, [sp, #96]
    ldp x10, x11, [sp, #80]
    ldp x8, x9, [sp, #64]
    ldp x6, x7, [sp, #48]
    ldp x4, x5, [sp, #32]
    ldp x2, x3, [sp, #16]
    ldp x0, x1, [sp], #144
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    ret
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #192
    stp x29, x30, [sp, #176]
    add x29, sp, #176
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-176]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    ; publish the call-stack overflow floor for this process
    bl __rt_stack_limit_init
    stur xzr, [x29, #-168]
    ; @block name=entry
_eir_main_entry_92:
    ; @src line=2 col=1 end=2:8 op=concat_reset
    ldur x10, [x29, #-176]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=2 col=21 end=2:33 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #10
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=2 col=35 end=2:36 op=array_new
    mov x0, #4
    mov x1, #16
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #1
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-24]
    ; @src line=2 col=36 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=2 col=36 op=array_push
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-24]
    ; @src line=2 col=43 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-56]
    stur x2, [x29, #-48]
    ; @src line=2 col=43 op=array_push
    ldur x1, [x29, #-56]
    ldur x2, [x29, #-48]
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-24]
    ; @src line=2 col=54 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=2 col=54 op=array_push
    ldur x1, [x29, #-72]
    ldur x2, [x29, #-64]
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-24]
    ; @src line=2 col=63 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-88]
    stur x2, [x29, #-80]
    ; @src line=2 col=63 op=array_push
    ldur x1, [x29, #-88]
    ldur x2, [x29, #-80]
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-24]
    ; @src line=2 col=11 end=2:72 op=runtime_call
    b _eir_main_preg_grep_after_predicate_94
_eir_main_preg_grep_predicate_93:
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    str x1, [sp]
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #16]
    mov x3, x1
    mov x4, x2
    ldr x9, [sp]
    ldp x1, x2, [x9]
    bl __rt_preg_match
    ldr x9, [sp]
    ldr x9, [x9, #16]
    eor x0, x0, x9
    str x0, [sp, #32]
    ldr x0, [sp, #16]
    bl __rt_heap_free
    ldr x0, [sp, #32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
_eir_main_preg_grep_after_predicate_94:
    sub sp, sp, #32
    mov x0, #0
    and x0, x0, #1
    str x0, [sp, #16]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    stp x1, x2, [sp]
    adrp x0, _eir_main_preg_grep_predicate_93@PAGE
    add x0, x0, _eir_main_preg_grep_predicate_93@PAGEOFF
    ldur x1, [x29, #-24]
    mov x2, sp
    mov x3, #0
    bl __rt_array_filter_mixed_raw
    str x0, [sp]
    bl __rt_mixed_unbox
    str x1, [sp, #8]
    mov x0, x1
    bl __rt_incref
    ldr x0, [sp]
    bl __rt_decref_mixed
    ldr x0, [sp, #8]
    add sp, sp, #32
    stur x0, [x29, #-96]
    ; @src line=2 col=11 end=2:72 op=release
    ldur x0, [x29, #-24]
    bl __rt_decref_array
    ; @src line=2 col=1 end=2:8 op=acquire
    ldur x0, [x29, #-96]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-104]
    ; @src line=2 col=1 end=2:8 op=store_local
    ldur x0, [x29, #-104]
    stur x0, [x29, #-168]
    ; @src line=2 col=1 end=2:8 op=release
    ldur x0, [x29, #-96]
    bl __rt_decref_hash
    ; @src line=3 col=1 end=3:5 op=concat_reset
    ldur x10, [x29, #-176]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=3 col=14 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=3 col=32 end=3:39 op=load_local
    ldur x0, [x29, #-168]
    stur x0, [x29, #-128]
    ; @src line=3 col=19 end=3:40 op=runtime_call
    ldur x0, [x29, #-128]
    str x0, [sp, #-16]!
    ldr x0, [x0]
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    str x0, [sp, #-16]!
    str xzr, [sp, #-16]!
_eir_main_avals_assoc_loop_95:
    ldr x0, [sp, #32]
    ldr x1, [sp]
    bl __rt_hash_iter_next
    cmn x0, #1
    b.eq _eir_main_avals_assoc_end_96
    str x0, [sp]
    cmp x5, #7
    b.eq _eir_main_avals_assoc_reuse_mixed_97
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_avals_assoc_store_mixed_98
_eir_main_avals_assoc_reuse_mixed_97:
    mov x0, x3
    bl __rt_incref
_eir_main_avals_assoc_store_mixed_98:
    ldr x9, [sp, #16]
    ldr x10, [x9]
    add x11, x9, #24
    str x0, [x11, x10, lsl #3]
    add x10, x10, #1
    str x10, [x9]
    b _eir_main_avals_assoc_loop_95
_eir_main_avals_assoc_end_96:
    add sp, sp, #16
    ldr x0, [sp], #16
    add sp, sp, #16
    stur x0, [x29, #-136]
    ; @src line=3 col=19 end=3:40 op=nop
    ; @src line=3 col=6 end=3:41 op=runtime_call
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    stp x1, x2, [sp, #-16]!
    ldur x0, [x29, #-136]
    mov x3, x0
    ldp x1, x2, [sp], #16
    bl __rt_implode
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=3 col=6 end=3:41 op=release
    ldur x0, [x29, #-136]
    bl __rt_decref_array
    ; @src line=3 col=1 end=3:5 op=echo_value
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=3 col=1 end=3:5 op=release

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $values
    ldur x0, [x29, #-168]
    cbz x0, _eir_main_main_refcounted_cleanup_done_99
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_99:
    mov x9, x29
    add sp, x9, #16
    ldp x29, x30, [x9]
    bl __rt_ob_flush_all
    mov x0, #0
    mov x16, #1
    svc #0x80
    ; @endfn name=main
_static_Fiber_getcurrent:
    b __rt_fiber_get_current
_method_Fiber_getreturn:
    b __rt_fiber_get_return
_method_Fiber_isrunning:
    b __rt_fiber_state_eq
_method_Fiber_isstarted:
    b __rt_fiber_state_eq
_method_Fiber_issuspended:
    b __rt_fiber_state_eq
_method_Fiber_isterminated:
    b __rt_fiber_state_eq
_method_Fiber_resume:
    b __rt_fiber_resume
_method_Fiber_start:
    b __rt_fiber_start
_static_Fiber_suspend:
    b __rt_fiber_suspend
_method_Fiber_throw:
    b __rt_fiber_throw
_method_Generator_current:
    b __rt_gen_current
_method_Generator_getreturn:
    b __rt_gen_get_return
_method_Generator_key:
    b __rt_gen_key
_method_Generator_next:
    b __rt_gen_next
_method_Generator_rewind:
    b __rt_gen_rewind
_method_Generator_send:
    b __rt_gen_send
_method_Generator_throw:
    b __rt_gen_throw
_method_Generator_valid:
    b __rt_gen_valid
_method___SplDoublyLinkedList____u__u_serialize:
    b __rt_spl_dll_serialize_array
_method_SplDoublyLinkedList_add:
    b __rt_spl_dll_insert
_method_SplDoublyLinkedList_bottom:
    b __rt_spl_dll_bottom
_method_SplDoublyLinkedList_count:
    b __rt_spl_dll_count
_method_SplDoublyLinkedList_current:
    b __rt_spl_dll_current
_method_SplDoublyLinkedList_getiteratormode:
    b __rt_spl_dll_get_iterator_mode
_method_SplDoublyLinkedList_isempty:
    b __rt_spl_dll_is_empty
_method_SplDoublyLinkedList_key:
    b __rt_spl_dll_key
_method_SplDoublyLinkedList_next:
    b __rt_spl_dll_next
_method_SplDoublyLinkedList_offsetexists:
    b __rt_spl_dll_offset_exists
_method_SplDoublyLinkedList_offsetget:
    b __rt_spl_dll_offset_get
_method_SplDoublyLinkedList_offsetset:
    b __rt_spl_dll_offset_set
_method_SplDoublyLinkedList_offsetunset:
    b __rt_spl_dll_offset_unset
_method_SplDoublyLinkedList_pop:
    b __rt_spl_dll_pop
_method_SplDoublyLinkedList_prev:
    b __rt_spl_dll_prev
_method_SplDoublyLinkedList_push:
    b __rt_spl_dll_push
_method_SplDoublyLinkedList_rewind:
    b __rt_spl_dll_rewind
_method_SplDoublyLinkedList_serialize:
    b __rt_spl_dll_serialize
_method_SplDoublyLinkedList_setiteratormode:
    b __rt_spl_dll_set_iterator_mode
_method_SplDoublyLinkedList_shift:
    b __rt_spl_dll_shift
_method_SplDoublyLinkedList_top:
    b __rt_spl_dll_top
_method_SplDoublyLinkedList_unserialize:
    b __rt_spl_dll_unserialize
_method_SplDoublyLinkedList_unshift:
    b __rt_spl_dll_unshift
_method_SplDoublyLinkedList_valid:
    b __rt_spl_dll_valid
_method___SplFixedArray____u__u_construct:
    b __rt_spl_fixed_set_size
_method___SplFixedArray____u__u_unserialize:
    b __rt_spl_fixed_unserialize
_method_SplFixedArray_count:
    b __rt_spl_fixed_count
_static_SplFixedArray_fromarray:
    b __rt_spl_fixed_from_array
_method_SplFixedArray_getsize:
    b __rt_spl_fixed_count
_method_SplFixedArray_jsonserialize:
    b __rt_spl_fixed_to_array
_method_SplFixedArray_offsetexists:
    b __rt_spl_fixed_offset_exists
_method_SplFixedArray_offsetget:
    b __rt_spl_fixed_offset_get
_method_SplFixedArray_offsetset:
    b __rt_spl_fixed_offset_set
_method_SplFixedArray_offsetunset:
    b __rt_spl_fixed_offset_unset
_method_SplFixedArray_setsize:
    b __rt_spl_fixed_set_size
_method_SplFixedArray_toarray:
    b __rt_spl_fixed_to_array
_method_SplQueue_dequeue:
    b __rt_spl_dll_shift
_method_SplQueue_enqueue:
    b __rt_spl_dll_push

.data
.globl _str_0
_str_0:
    .ascii ""
.globl _str_2
_str_2:
    .ascii "UTC"
.globl _str_3
_str_3:
    .ascii "get"
.globl _str_4
_str_4:
    .ascii "Get"
.globl _str_5
_str_5:
    .ascii "set"
.globl _str_6
_str_6:
    .ascii "Set"
.globl _str_7
_str_7:
    .ascii "Ascending"
.globl _str_8
_str_8:
    .ascii "Descending"
.globl _str_9
_str_9:
    .ascii "/^[a-z]+$/"
.globl _str_10
_str_10:
    .ascii "123"
.globl _str_11
_str_11:
    .ascii "charlie"
.globl _str_12
_str_12:
    .ascii "alpha"
.globl _str_13
_str_13:
    .ascii "bravo"
.globl _str_14
_str_14:
    .ascii ","
.p2align 3
.globl _float_1
_float_1:
    .quad 0x0000000000000000

.comm _enum_case_PropertyHookType_Get, 8, 3
.comm _enum_case_PropertyHookType_Set, 8, 3
.comm _enum_case_SortDirection_Ascending, 8, 3
.comm _enum_case_SortDirection_Descending, 8, 3
.data
.p2align 3
.globl _callable_user_fn_name_0
_callable_user_fn_name_0:
    .ascii "main"
.p2align 3
.globl _callable_user_function_count
_callable_user_function_count:
    .quad 1
.globl _callable_user_function_table
_callable_user_function_table:
    .quad _callable_user_fn_name_0
    .quad 4
    .quad 0
.p2align 3
.globl _instanceof_target_count
_instanceof_target_count:
    .quad 38
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_2
    .quad 5
    .quad 2
    .quad 0
    .quad _instanceof_name_class_abs_2
    .quad 6
    .quad 2
    .quad 0
    .quad _instanceof_name_class_3
    .quad 9
    .quad 3
    .quad 0
    .quad _instanceof_name_class_abs_3
    .quad 10
    .quad 3
    .quad 0
    .quad _instanceof_name_class_4
    .quad 18
    .quad 4
    .quad 0
    .quad _instanceof_name_class_abs_4
    .quad 19
    .quad 4
    .quad 0
    .quad _instanceof_name_class_5
    .quad 15
    .quad 5
    .quad 0
    .quad _instanceof_name_class_abs_5
    .quad 16
    .quad 5
    .quad 0
    .quad _instanceof_name_class_8
    .quad 14
    .quad 8
    .quad 0
    .quad _instanceof_name_class_abs_8
    .quad 15
    .quad 8
    .quad 0
    .quad _instanceof_name_class_9
    .quad 9
    .quad 9
    .quad 0
    .quad _instanceof_name_class_abs_9
    .quad 10
    .quad 9
    .quad 0
    .quad _instanceof_name_class_10
    .quad 14
    .quad 10
    .quad 0
    .quad _instanceof_name_class_abs_10
    .quad 15
    .quad 10
    .quad 0
    .quad _instanceof_name_class_33
    .quad 19
    .quad 33
    .quad 0
    .quad _instanceof_name_class_abs_33
    .quad 20
    .quad 33
    .quad 0
    .quad _instanceof_name_class_43
    .quad 24
    .quad 43
    .quad 0
    .quad _instanceof_name_class_abs_43
    .quad 25
    .quad 43
    .quad 0
    .quad _instanceof_name_class_44
    .quad 16
    .quad 44
    .quad 0
    .quad _instanceof_name_class_abs_44
    .quad 17
    .quad 44
    .quad 0
    .quad _instanceof_name_class_45
    .quad 13
    .quad 45
    .quad 0
    .quad _instanceof_name_class_abs_45
    .quad 14
    .quad 45
    .quad 0
    .quad _instanceof_name_class_50
    .quad 20
    .quad 50
    .quad 0
    .quad _instanceof_name_class_abs_50
    .quad 21
    .quad 50
    .quad 0
    .quad _instanceof_name_class_51
    .quad 19
    .quad 51
    .quad 0
    .quad _instanceof_name_class_abs_51
    .quad 20
    .quad 51
    .quad 0
    .quad _instanceof_name_class_72
    .quad 19
    .quad 72
    .quad 0
    .quad _instanceof_name_class_abs_72
    .quad 20
    .quad 72
    .quad 0
    .quad _instanceof_name_class_94
    .quad 19
    .quad 94
    .quad 0
    .quad _instanceof_name_class_abs_94
    .quad 20
    .quad 94
    .quad 0
    .quad _instanceof_name_class_95
    .quad 10
    .quad 95
    .quad 0
    .quad _instanceof_name_class_abs_95
    .quad 11
    .quad 95
    .quad 0
    .quad _instanceof_name_class_99
    .quad 8
    .quad 99
    .quad 0
    .quad _instanceof_name_class_abs_99
    .quad 9
    .quad 99
    .quad 0
    .quad _instanceof_name_interface_13
    .quad 10
    .quad 13
    .quad 1
    .quad _instanceof_name_interface_abs_13
    .quad 11
    .quad 13
    .quad 1
    .quad _instanceof_name_interface_14
    .quad 9
    .quad 14
    .quad 1
    .quad _instanceof_name_interface_abs_14
    .quad 10
    .quad 14
    .quad 1
.globl _instanceof_name_class_2
_instanceof_name_class_2:
    .ascii "Error"
.globl _instanceof_name_class_abs_2
_instanceof_name_class_abs_2:
    .ascii "\\Error"
.globl _instanceof_name_class_3
_instanceof_name_class_3:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_3
_instanceof_name_class_abs_3:
    .ascii "\\TypeError"
.globl _instanceof_name_class_4
_instanceof_name_class_4:
    .ascii "ArgumentCountError"
.globl _instanceof_name_class_abs_4
_instanceof_name_class_abs_4:
    .ascii "\\ArgumentCountError"
.globl _instanceof_name_class_5
_instanceof_name_class_5:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_5
_instanceof_name_class_abs_5:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_8
_instanceof_name_class_8:
    .ascii "AssertionError"
.globl _instanceof_name_class_abs_8
_instanceof_name_class_abs_8:
    .ascii "\\AssertionError"
.globl _instanceof_name_class_9
_instanceof_name_class_9:
    .ascii "Exception"
.globl _instanceof_name_class_abs_9
_instanceof_name_class_abs_9:
    .ascii "\\Exception"
.globl _instanceof_name_class_10
_instanceof_name_class_10:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_10
_instanceof_name_class_abs_10:
    .ascii "\\LogicException"
.globl _instanceof_name_class_33
_instanceof_name_class_33:
    .ascii "DivisionByZeroError"
.globl _instanceof_name_class_abs_33
_instanceof_name_class_abs_33:
    .ascii "\\DivisionByZeroError"
.globl _instanceof_name_class_43
_instanceof_name_class_43:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_43
_instanceof_name_class_abs_43:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_44
_instanceof_name_class_44:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_44
_instanceof_name_class_abs_44:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_45
_instanceof_name_class_45:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_45
_instanceof_name_class_abs_45:
    .ascii "\\JsonException"
.globl _instanceof_name_class_50
_instanceof_name_class_50:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_50
_instanceof_name_class_abs_50:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_51
_instanceof_name_class_51:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_51
_instanceof_name_class_abs_51:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_72
_instanceof_name_class_72:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_72
_instanceof_name_class_abs_72:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_94
_instanceof_name_class_94:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_94
_instanceof_name_class_abs_94:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_95
_instanceof_name_class_95:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_95
_instanceof_name_class_abs_95:
    .ascii "\\ValueError"
.globl _instanceof_name_class_99
_instanceof_name_class_99:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_99
_instanceof_name_class_abs_99:
    .ascii "\\stdClass"
.globl _instanceof_name_interface_13
_instanceof_name_interface_13:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_13
_instanceof_name_interface_abs_13:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_14
_instanceof_name_interface_14:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_14
_instanceof_name_interface_abs_14:
    .ascii "\\Throwable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 100
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_2
    .quad 5
    .quad _class_name_3
    .quad 9
    .quad _class_name_4
    .quad 18
    .quad _class_name_5
    .quad 15
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_8
    .quad 14
    .quad _class_name_9
    .quad 9
    .quad _class_name_10
    .quad 14
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_33
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_43
    .quad 24
    .quad _class_name_44
    .quad 16
    .quad _class_name_45
    .quad 13
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_50
    .quad 20
    .quad _class_name_51
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_72
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_94
    .quad 19
    .quad _class_name_95
    .quad 10
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_99
    .quad 8
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_2
_class_name_2:
    .ascii "Error"
.globl _class_name_3
_class_name_3:
    .ascii "TypeError"
.globl _class_name_4
_class_name_4:
    .ascii "ArgumentCountError"
.globl _class_name_5
_class_name_5:
    .ascii "ArithmeticError"
.globl _class_name_8
_class_name_8:
    .ascii "AssertionError"
.globl _class_name_9
_class_name_9:
    .ascii "Exception"
.globl _class_name_10
_class_name_10:
    .ascii "LogicException"
.globl _class_name_33
_class_name_33:
    .ascii "DivisionByZeroError"
.globl _class_name_43
_class_name_43:
    .ascii "InvalidArgumentException"
.globl _class_name_44
_class_name_44:
    .ascii "RuntimeException"
.globl _class_name_45
_class_name_45:
    .ascii "JsonException"
.globl _class_name_50
_class_name_50:
    .ascii "OutOfBoundsException"
.globl _class_name_51
_class_name_51:
    .ascii "OutOfRangeException"
.globl _class_name_72
_class_name_72:
    .ascii "ReflectionException"
.globl _class_name_94
_class_name_94:
    .ascii "UnhandledMatchError"
.globl _class_name_95
_class_name_95:
    .ascii "ValueError"
.globl _class_name_99
_class_name_99:
    .ascii "stdClass"
    .p2align 3
.globl _interface_name_0
_interface_name_0:
    .ascii "ArrayAccess"
.globl _interface_name_1
_interface_name_1:
    .ascii "Countable"
.globl _interface_name_2
_interface_name_2:
    .ascii "DateTimeInterface"
.globl _interface_name_3
_interface_name_3:
    .ascii "Iterator"
.globl _interface_name_4
_interface_name_4:
    .ascii "IteratorAggregate"
.globl _interface_name_5
_interface_name_5:
    .ascii "JsonSerializable"
.globl _interface_name_6
_interface_name_6:
    .ascii "OuterIterator"
.globl _interface_name_7
_interface_name_7:
    .ascii "RecursiveIterator"
.globl _interface_name_8
_interface_name_8:
    .ascii "Reflector"
.globl _interface_name_9
_interface_name_9:
    .ascii "SeekableIterator"
.globl _interface_name_10
_interface_name_10:
    .ascii "SplObserver"
.globl _interface_name_11
_interface_name_11:
    .ascii "SplSubject"
.globl _interface_name_12
_interface_name_12:
    .ascii "Stringable"
.globl _interface_name_13
_interface_name_13:
    .ascii "Throwable"
.globl _interface_name_14
_interface_name_14:
    .ascii "Traversable"
.p2align 3
.globl _interface_names_count
_interface_names_count:
    .quad 15
.globl _interface_names
_interface_names:
    .quad _interface_name_0
    .quad 11
    .quad _interface_name_1
    .quad 9
    .quad _interface_name_2
    .quad 17
    .quad _interface_name_3
    .quad 8
    .quad _interface_name_4
    .quad 17
    .quad _interface_name_5
    .quad 16
    .quad _interface_name_6
    .quad 13
    .quad _interface_name_7
    .quad 17
    .quad _interface_name_8
    .quad 9
    .quad _interface_name_9
    .quad 16
    .quad _interface_name_10
    .quad 11
    .quad _interface_name_11
    .quad 10
    .quad _interface_name_12
    .quad 10
    .quad _interface_name_13
    .quad 9
    .quad _interface_name_14
    .quad 11
.p2align 3
.globl _trait_names_count
_trait_names_count:
    .quad 0
.globl _trait_names
_trait_names:
.globl _enum_name_0
_enum_name_0:
    .ascii "PropertyHookType"
.globl _enum_name_1
_enum_name_1:
    .ascii "SortDirection"
.p2align 3
.globl _enum_names_count
_enum_names_count:
    .quad 2
.globl _enum_names
_enum_names:
    .quad _enum_name_0
    .quad 16
    .quad _enum_name_1
    .quad 13
.globl _fiber_class_id
_fiber_class_id:
    .quad 36
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 37
.globl _generator_class_id
_generator_class_id:
    .quad 39
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 81
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 90
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 89
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 83
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 2
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 10
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 44
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 51
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 50
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 43
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 3
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 95
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 5
.globl _spl_division_by_zero_error_class_id
_spl_division_by_zero_error_class_id:
    .quad 33
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_13
    .quad _interface_methods_14
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_2
    .quad _class_interfaces_3
    .quad _class_interfaces_4
    .quad _class_interfaces_5
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_8
    .quad _class_interfaces_9
    .quad _class_interfaces_10
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_33
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_43
    .quad _class_interfaces_44
    .quad _class_interfaces_45
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_50
    .quad _class_interfaces_51
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_72
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_94
    .quad _class_interfaces_95
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_99
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_2
    .quad _class_json_desc_3
    .quad _class_json_desc_4
    .quad _class_json_desc_5
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_8
    .quad _class_json_desc_9
    .quad _class_json_desc_10
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_33
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_43
    .quad _class_json_desc_44
    .quad _class_json_desc_45
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_50
    .quad _class_json_desc_51
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_72
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_94
    .quad _class_json_desc_95
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_99
.globl _class_vd_desc_ptrs
_class_vd_desc_ptrs:
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_2
    .quad _class_vd_desc_3
    .quad _class_vd_desc_4
    .quad _class_vd_desc_5
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_8
    .quad _class_vd_desc_9
    .quad _class_vd_desc_10
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_33
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_43
    .quad _class_vd_desc_44
    .quad _class_vd_desc_45
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_50
    .quad _class_vd_desc_51
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_72
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_94
    .quad _class_vd_desc_95
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_missing
    .quad _class_vd_desc_99
.globl _class_prop_desc_ptrs
_class_prop_desc_ptrs:
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_2
    .quad _class_prop_desc_3
    .quad _class_prop_desc_4
    .quad _class_prop_desc_5
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_8
    .quad _class_prop_desc_9
    .quad _class_prop_desc_10
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_33
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_43
    .quad _class_prop_desc_44
    .quad _class_prop_desc_45
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_50
    .quad _class_prop_desc_51
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_72
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_94
    .quad _class_prop_desc_95
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_missing
    .quad _class_prop_desc_99
.globl _class_enum_kinds
_class_enum_kinds:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_enum_name_offsets
_class_enum_name_offsets:
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 45
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
    .quad -1
    .quad -1
    .quad 2
    .quad 3
    .quad 2
    .quad -1
    .quad -1
    .quad 2
    .quad -1
    .quad 9
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 5
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 10
    .quad 9
    .quad 44
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 44
    .quad 10
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 9
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 2
    .quad 2
    .quad -1
    .quad -1
    .quad -1
    .quad -1
.globl _class_object_payload_sizes
_class_object_payload_sizes:
    .quad 0
    .quad 0
    .quad 56
    .quad 56
    .quad 56
    .quad 56
    .quad 0
    .quad 0
    .quad 56
    .quad 56
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 56
    .quad 56
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 56
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 56
    .quad 56
    .quad 0
    .quad 0
    .quad 0
    .quad 16
.globl _class_object_dynamic_prop_flags
_class_object_dynamic_prop_flags:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 1
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 100
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_2
    .quad _class_gc_desc_3
    .quad _class_gc_desc_4
    .quad _class_gc_desc_5
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_8
    .quad _class_gc_desc_9
    .quad _class_gc_desc_10
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_33
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_43
    .quad _class_gc_desc_44
    .quad _class_gc_desc_45
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_50
    .quad _class_gc_desc_51
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_72
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_94
    .quad _class_gc_desc_95
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_99
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_2
    .quad _class_vtable_3
    .quad _class_vtable_4
    .quad _class_vtable_5
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_8
    .quad _class_vtable_9
    .quad _class_vtable_10
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_33
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_43
    .quad _class_vtable_44
    .quad _class_vtable_45
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_50
    .quad _class_vtable_51
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_72
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_94
    .quad _class_vtable_95
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_99
.globl _class_destruct_count
_class_destruct_count:
    .quad 100
.globl _class_destruct_ptrs
_class_destruct_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_serialize_ptrs
_class_serialize_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_unserialize_ptrs
_class_unserialize_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_sleep_ptrs
_class_sleep_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_wakeup_ptrs
_class_wakeup_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad 0
    .quad 0
    .quad _class_propinit_2
    .quad _class_propinit_3
    .quad _class_propinit_4
    .quad _class_propinit_5
    .quad 0
    .quad 0
    .quad _class_propinit_8
    .quad _class_propinit_9
    .quad _class_propinit_10
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_33
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_43
    .quad _class_propinit_44
    .quad _class_propinit_45
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_50
    .quad _class_propinit_51
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_94
    .quad _class_propinit_95
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_2
    .quad _class_serprop_3
    .quad _class_serprop_4
    .quad _class_serprop_5
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_8
    .quad _class_serprop_9
    .quad _class_serprop_10
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_33
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_43
    .quad _class_serprop_44
    .quad _class_serprop_45
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_50
    .quad _class_serprop_51
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_72
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_94
    .quad _class_serprop_95
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_99
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_2
    .quad _class_static_vtable_3
    .quad _class_static_vtable_4
    .quad _class_static_vtable_5
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_8
    .quad _class_static_vtable_9
    .quad _class_static_vtable_10
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_33
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_43
    .quad _class_static_vtable_44
    .quad _class_static_vtable_45
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_50
    .quad _class_static_vtable_51
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_72
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_94
    .quad _class_static_vtable_95
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_99
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_2
    .quad _class_callable_methods_3
    .quad _class_callable_methods_4
    .quad _class_callable_methods_5
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_8
    .quad _class_callable_methods_9
    .quad _class_callable_methods_10
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_33
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_43
    .quad _class_callable_methods_44
    .quad _class_callable_methods_45
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_50
    .quad _class_callable_methods_51
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_72
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_94
    .quad _class_callable_methods_95
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_99
.p2align 3
.globl _user_wrapper_vtable_ptrs
_user_wrapper_vtable_ptrs:
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
    .quad _user_wrapper_vtable_missing
.p2align 3
.globl _user_filter_vtable_ptrs
_user_filter_vtable_ptrs:
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
    .quad _user_filter_vtable_missing
.globl _class_interfaces_missing
_class_interfaces_missing:
    .quad 0
.globl _class_gc_desc_missing
_class_gc_desc_missing:
    .byte 0
    .p2align 3
.globl _class_serprop_missing
_class_serprop_missing:
    .quad 0
    .p2align 3
.globl _class_json_desc_missing
_class_json_desc_missing:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_vd_desc_missing
_class_vd_desc_missing:
    .quad 0
    .p2align 3
.globl _class_prop_desc_missing
_class_prop_desc_missing:
    .quad 0
    .p2align 3
.globl _class_vtable_missing
_class_vtable_missing:
    .quad 0
    .p2align 3
.globl _class_static_vtable_missing
_class_static_vtable_missing:
    .quad 0
    .p2align 3
.globl _class_callable_methods_missing
_class_callable_methods_missing:
    .quad 0
    .p2align 3
.globl _user_wrapper_vtable_missing
_user_wrapper_vtable_missing:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _user_filter_vtable_missing
_user_filter_vtable_missing:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.p2align 3
.p2align 3
.globl _class_callable_static_method_count
_class_callable_static_method_count:
    .quad 0
.globl _class_callable_static_method_table
_class_callable_static_method_table:
.p2align 3
.globl _script_source_file
_script_source_file:
    .ascii "/Users/guillaumeloulier/PhpstormProjects/oss/elephc/.claude/worktrees/lazy-petting-popcorn/examples/debug_gate62.php"
.p2align 3
.globl _script_source_file_len
_script_source_file_len:
    .quad 116
.p2align 3
.globl _class_by_name_str_2
_class_by_name_str_2:
    .ascii "Error"
.globl _class_by_name_str_3
_class_by_name_str_3:
    .ascii "TypeError"
.globl _class_by_name_str_4
_class_by_name_str_4:
    .ascii "ArgumentCountError"
.globl _class_by_name_str_5
_class_by_name_str_5:
    .ascii "ArithmeticError"
.globl _class_by_name_str_8
_class_by_name_str_8:
    .ascii "AssertionError"
.globl _class_by_name_str_9
_class_by_name_str_9:
    .ascii "Exception"
.globl _class_by_name_str_10
_class_by_name_str_10:
    .ascii "LogicException"
.globl _class_by_name_str_33
_class_by_name_str_33:
    .ascii "DivisionByZeroError"
.globl _class_by_name_str_43
_class_by_name_str_43:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_44
_class_by_name_str_44:
    .ascii "RuntimeException"
.globl _class_by_name_str_45
_class_by_name_str_45:
    .ascii "JsonException"
.globl _class_by_name_str_50
_class_by_name_str_50:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_51
_class_by_name_str_51:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_72
_class_by_name_str_72:
    .ascii "ReflectionException"
.globl _class_by_name_str_94
_class_by_name_str_94:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_95
_class_by_name_str_95:
    .ascii "ValueError"
.globl _class_by_name_str_99
_class_by_name_str_99:
    .ascii "stdClass"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 17
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_2
    .quad 5
    .quad 2
    .quad 56
    .quad _class_by_name_str_3
    .quad 9
    .quad 3
    .quad 56
    .quad _class_by_name_str_4
    .quad 18
    .quad 4
    .quad 56
    .quad _class_by_name_str_5
    .quad 15
    .quad 5
    .quad 56
    .quad _class_by_name_str_8
    .quad 14
    .quad 8
    .quad 56
    .quad _class_by_name_str_9
    .quad 9
    .quad 9
    .quad 56
    .quad _class_by_name_str_10
    .quad 14
    .quad 10
    .quad 56
    .quad _class_by_name_str_33
    .quad 19
    .quad 33
    .quad 56
    .quad _class_by_name_str_43
    .quad 24
    .quad 43
    .quad 56
    .quad _class_by_name_str_44
    .quad 16
    .quad 44
    .quad 56
    .quad _class_by_name_str_45
    .quad 13
    .quad 45
    .quad 56
    .quad _class_by_name_str_50
    .quad 20
    .quad 50
    .quad 56
    .quad _class_by_name_str_51
    .quad 19
    .quad 51
    .quad 56
    .quad _class_by_name_str_72
    .quad 19
    .quad 72
    .quad 56
    .quad _class_by_name_str_94
    .quad 19
    .quad 94
    .quad 56
    .quad _class_by_name_str_95
    .quad 10
    .quad 95
    .quad 56
    .quad _class_by_name_str_99
    .quad 8
    .quad 99
    .quad 16
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 100
.globl _class_attribute_ptrs
_class_attribute_ptrs:
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_13
_interface_methods_13:
    .quad 1
    .quad 0
.globl _interface_methods_14
_interface_methods_14:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _class_interfaces_2
_class_interfaces_2:
    .quad 2
    .quad 14
    .quad _class_interface_impl_2_14
    .quad 13
    .quad _class_interface_impl_2_13
.globl _class_interface_impl_2_14
_class_interface_impl_2_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_2_13
_class_interface_impl_2_13:
    .quad 0
.globl _class_json_pname_2_0
_class_json_pname_2_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_2
_class_json_desc_2:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_2_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_2
_class_gc_desc_2:
    .byte 1, 0, 7
.globl _class_serpname_2_0
_class_serpname_2_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_2_1
_class_serpname_2_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_2_2
_class_serpname_2_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_2
_class_serprop_2:
    .quad 3
    .quad _class_serpname_2_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_2_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_2_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_2_0
_class_vd_pkey_2_0:
    .ascii "\"message\""
.globl _class_vd_ptype_2_0
_class_vd_ptype_2_0:
    .ascii "string"
.globl _class_vd_pkey_2_1
_class_vd_pkey_2_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_2_1
_class_vd_ptype_2_1:
    .ascii "int"
.globl _class_vd_pkey_2_2
_class_vd_pkey_2_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_2_2
_class_vd_ptype_2_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_2
_class_vd_desc_2:
    .quad 3
    .quad _class_vd_pkey_2_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_2_0
    .quad 6
    .quad _class_vd_pkey_2_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_2_1
    .quad 3
    .quad _class_vd_pkey_2_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_2_2
    .quad 5
.globl _class_prop_pkey_2_0
_class_prop_pkey_2_0:
    .ascii "message"
.globl _class_prop_nkey_2_0
_class_prop_nkey_2_0:
    .ascii "message"
.globl _class_prop_pkey_2_1
_class_prop_pkey_2_1:
    .ascii "code:protected"
.globl _class_prop_nkey_2_1
_class_prop_nkey_2_1:
    .ascii "code"
.globl _class_prop_pkey_2_2
_class_prop_pkey_2_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_2_2
_class_prop_nkey_2_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_2
_class_prop_desc_2:
    .quad 3
    .quad _class_prop_pkey_2_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_2_0
    .quad 7
    .quad _class_prop_pkey_2_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_2_1
    .quad 4
    .quad _class_prop_pkey_2_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_2_2
    .quad 8
    .p2align 3
.globl _class_vtable_2
_class_vtable_2:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_2
_class_static_vtable_2:
    .quad 0
.globl _class_callable_method_name_2__u__u_construct
_class_callable_method_name_2__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_2__u__u_tostring
_class_callable_method_name_2__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_2_getcode
_class_callable_method_name_2_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_2_getfile
_class_callable_method_name_2_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_2_getline
_class_callable_method_name_2_getline:
    .ascii "getline"
.globl _class_callable_method_name_2_getmessage
_class_callable_method_name_2_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_2_getprevious
_class_callable_method_name_2_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_2_gettrace
_class_callable_method_name_2_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_2_gettraceasstring
_class_callable_method_name_2_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_2
_class_callable_methods_2:
    .quad 9
    .quad _class_callable_method_name_2__u__u_construct
    .quad 11
    .quad _class_callable_method_name_2__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_2_getcode
    .quad 7
    .quad _class_callable_method_name_2_getfile
    .quad 7
    .quad _class_callable_method_name_2_getline
    .quad 7
    .quad _class_callable_method_name_2_getmessage
    .quad 10
    .quad _class_callable_method_name_2_getprevious
    .quad 11
    .quad _class_callable_method_name_2_gettrace
    .quad 8
    .quad _class_callable_method_name_2_gettraceasstring
    .quad 16
.globl _class_interfaces_3
_class_interfaces_3:
    .quad 2
    .quad 14
    .quad _class_interface_impl_3_14
    .quad 13
    .quad _class_interface_impl_3_13
.globl _class_interface_impl_3_14
_class_interface_impl_3_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_3_13
_class_interface_impl_3_13:
    .quad 0
.globl _class_json_pname_3_0
_class_json_pname_3_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_3
_class_json_desc_3:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_3_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_3
_class_gc_desc_3:
    .byte 1, 0, 7
.globl _class_serpname_3_0
_class_serpname_3_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_3_1
_class_serpname_3_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_3_2
_class_serpname_3_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_3
_class_serprop_3:
    .quad 3
    .quad _class_serpname_3_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_3_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_3_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_3_0
_class_vd_pkey_3_0:
    .ascii "\"message\""
.globl _class_vd_ptype_3_0
_class_vd_ptype_3_0:
    .ascii "string"
.globl _class_vd_pkey_3_1
_class_vd_pkey_3_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_3_1
_class_vd_ptype_3_1:
    .ascii "int"
.globl _class_vd_pkey_3_2
_class_vd_pkey_3_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_3_2
_class_vd_ptype_3_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_3
_class_vd_desc_3:
    .quad 3
    .quad _class_vd_pkey_3_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_3_0
    .quad 6
    .quad _class_vd_pkey_3_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_3_1
    .quad 3
    .quad _class_vd_pkey_3_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_3_2
    .quad 5
.globl _class_prop_pkey_3_0
_class_prop_pkey_3_0:
    .ascii "message"
.globl _class_prop_nkey_3_0
_class_prop_nkey_3_0:
    .ascii "message"
.globl _class_prop_pkey_3_1
_class_prop_pkey_3_1:
    .ascii "code:protected"
.globl _class_prop_nkey_3_1
_class_prop_nkey_3_1:
    .ascii "code"
.globl _class_prop_pkey_3_2
_class_prop_pkey_3_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_3_2
_class_prop_nkey_3_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_3
_class_prop_desc_3:
    .quad 3
    .quad _class_prop_pkey_3_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_3_0
    .quad 7
    .quad _class_prop_pkey_3_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_3_1
    .quad 4
    .quad _class_prop_pkey_3_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_3_2
    .quad 8
    .p2align 3
.globl _class_vtable_3
_class_vtable_3:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_3
_class_static_vtable_3:
    .quad 0
.globl _class_callable_method_name_3__u__u_construct
_class_callable_method_name_3__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_3__u__u_tostring
_class_callable_method_name_3__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_3_getcode
_class_callable_method_name_3_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_3_getfile
_class_callable_method_name_3_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_3_getline
_class_callable_method_name_3_getline:
    .ascii "getline"
.globl _class_callable_method_name_3_getmessage
_class_callable_method_name_3_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_3_getprevious
_class_callable_method_name_3_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_3_gettrace
_class_callable_method_name_3_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_3_gettraceasstring
_class_callable_method_name_3_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_3
_class_callable_methods_3:
    .quad 9
    .quad _class_callable_method_name_3__u__u_construct
    .quad 11
    .quad _class_callable_method_name_3__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_3_getcode
    .quad 7
    .quad _class_callable_method_name_3_getfile
    .quad 7
    .quad _class_callable_method_name_3_getline
    .quad 7
    .quad _class_callable_method_name_3_getmessage
    .quad 10
    .quad _class_callable_method_name_3_getprevious
    .quad 11
    .quad _class_callable_method_name_3_gettrace
    .quad 8
    .quad _class_callable_method_name_3_gettraceasstring
    .quad 16
.globl _class_interfaces_4
_class_interfaces_4:
    .quad 2
    .quad 14
    .quad _class_interface_impl_4_14
    .quad 13
    .quad _class_interface_impl_4_13
.globl _class_interface_impl_4_14
_class_interface_impl_4_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_4_13
_class_interface_impl_4_13:
    .quad 0
.globl _class_json_pname_4_0
_class_json_pname_4_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_4
_class_json_desc_4:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_4_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_4
_class_gc_desc_4:
    .byte 1, 0, 7
.globl _class_serpname_4_0
_class_serpname_4_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_4_1
_class_serpname_4_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_4_2
_class_serpname_4_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_4
_class_serprop_4:
    .quad 3
    .quad _class_serpname_4_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_4_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_4_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_4_0
_class_vd_pkey_4_0:
    .ascii "\"message\""
.globl _class_vd_ptype_4_0
_class_vd_ptype_4_0:
    .ascii "string"
.globl _class_vd_pkey_4_1
_class_vd_pkey_4_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_4_1
_class_vd_ptype_4_1:
    .ascii "int"
.globl _class_vd_pkey_4_2
_class_vd_pkey_4_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_4_2
_class_vd_ptype_4_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_4
_class_vd_desc_4:
    .quad 3
    .quad _class_vd_pkey_4_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_4_0
    .quad 6
    .quad _class_vd_pkey_4_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_4_1
    .quad 3
    .quad _class_vd_pkey_4_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_4_2
    .quad 5
.globl _class_prop_pkey_4_0
_class_prop_pkey_4_0:
    .ascii "message"
.globl _class_prop_nkey_4_0
_class_prop_nkey_4_0:
    .ascii "message"
.globl _class_prop_pkey_4_1
_class_prop_pkey_4_1:
    .ascii "code:protected"
.globl _class_prop_nkey_4_1
_class_prop_nkey_4_1:
    .ascii "code"
.globl _class_prop_pkey_4_2
_class_prop_pkey_4_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_4_2
_class_prop_nkey_4_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_4
_class_prop_desc_4:
    .quad 3
    .quad _class_prop_pkey_4_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_4_0
    .quad 7
    .quad _class_prop_pkey_4_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_4_1
    .quad 4
    .quad _class_prop_pkey_4_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_4_2
    .quad 8
    .p2align 3
.globl _class_vtable_4
_class_vtable_4:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_4
_class_static_vtable_4:
    .quad 0
.globl _class_callable_method_name_4__u__u_construct
_class_callable_method_name_4__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_4__u__u_tostring
_class_callable_method_name_4__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_4_getcode
_class_callable_method_name_4_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_4_getfile
_class_callable_method_name_4_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_4_getline
_class_callable_method_name_4_getline:
    .ascii "getline"
.globl _class_callable_method_name_4_getmessage
_class_callable_method_name_4_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_4_getprevious
_class_callable_method_name_4_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_4_gettrace
_class_callable_method_name_4_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_4_gettraceasstring
_class_callable_method_name_4_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_4
_class_callable_methods_4:
    .quad 9
    .quad _class_callable_method_name_4__u__u_construct
    .quad 11
    .quad _class_callable_method_name_4__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_4_getcode
    .quad 7
    .quad _class_callable_method_name_4_getfile
    .quad 7
    .quad _class_callable_method_name_4_getline
    .quad 7
    .quad _class_callable_method_name_4_getmessage
    .quad 10
    .quad _class_callable_method_name_4_getprevious
    .quad 11
    .quad _class_callable_method_name_4_gettrace
    .quad 8
    .quad _class_callable_method_name_4_gettraceasstring
    .quad 16
.globl _class_interfaces_5
_class_interfaces_5:
    .quad 2
    .quad 14
    .quad _class_interface_impl_5_14
    .quad 13
    .quad _class_interface_impl_5_13
.globl _class_interface_impl_5_14
_class_interface_impl_5_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_5_13
_class_interface_impl_5_13:
    .quad 0
.globl _class_json_pname_5_0
_class_json_pname_5_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_5
_class_json_desc_5:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_5_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_5
_class_gc_desc_5:
    .byte 1, 0, 7
.globl _class_serpname_5_0
_class_serpname_5_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_5_1
_class_serpname_5_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_5_2
_class_serpname_5_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_5
_class_serprop_5:
    .quad 3
    .quad _class_serpname_5_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_5_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_5_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_5_0
_class_vd_pkey_5_0:
    .ascii "\"message\""
.globl _class_vd_ptype_5_0
_class_vd_ptype_5_0:
    .ascii "string"
.globl _class_vd_pkey_5_1
_class_vd_pkey_5_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_5_1
_class_vd_ptype_5_1:
    .ascii "int"
.globl _class_vd_pkey_5_2
_class_vd_pkey_5_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_5_2
_class_vd_ptype_5_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_5
_class_vd_desc_5:
    .quad 3
    .quad _class_vd_pkey_5_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_5_0
    .quad 6
    .quad _class_vd_pkey_5_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_5_1
    .quad 3
    .quad _class_vd_pkey_5_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_5_2
    .quad 5
.globl _class_prop_pkey_5_0
_class_prop_pkey_5_0:
    .ascii "message"
.globl _class_prop_nkey_5_0
_class_prop_nkey_5_0:
    .ascii "message"
.globl _class_prop_pkey_5_1
_class_prop_pkey_5_1:
    .ascii "code:protected"
.globl _class_prop_nkey_5_1
_class_prop_nkey_5_1:
    .ascii "code"
.globl _class_prop_pkey_5_2
_class_prop_pkey_5_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_5_2
_class_prop_nkey_5_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_5
_class_prop_desc_5:
    .quad 3
    .quad _class_prop_pkey_5_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_5_0
    .quad 7
    .quad _class_prop_pkey_5_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_5_1
    .quad 4
    .quad _class_prop_pkey_5_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_5_2
    .quad 8
    .p2align 3
.globl _class_vtable_5
_class_vtable_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_5
_class_static_vtable_5:
    .quad 0
.globl _class_callable_method_name_5__u__u_construct
_class_callable_method_name_5__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_5__u__u_tostring
_class_callable_method_name_5__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_5_getcode
_class_callable_method_name_5_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_5_getfile
_class_callable_method_name_5_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_5_getline
_class_callable_method_name_5_getline:
    .ascii "getline"
.globl _class_callable_method_name_5_getmessage
_class_callable_method_name_5_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_5_getprevious
_class_callable_method_name_5_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_5_gettrace
_class_callable_method_name_5_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_5_gettraceasstring
_class_callable_method_name_5_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_5
_class_callable_methods_5:
    .quad 9
    .quad _class_callable_method_name_5__u__u_construct
    .quad 11
    .quad _class_callable_method_name_5__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_5_getcode
    .quad 7
    .quad _class_callable_method_name_5_getfile
    .quad 7
    .quad _class_callable_method_name_5_getline
    .quad 7
    .quad _class_callable_method_name_5_getmessage
    .quad 10
    .quad _class_callable_method_name_5_getprevious
    .quad 11
    .quad _class_callable_method_name_5_gettrace
    .quad 8
    .quad _class_callable_method_name_5_gettraceasstring
    .quad 16
.globl _class_interfaces_8
_class_interfaces_8:
    .quad 2
    .quad 14
    .quad _class_interface_impl_8_14
    .quad 13
    .quad _class_interface_impl_8_13
.globl _class_interface_impl_8_14
_class_interface_impl_8_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_8_13
_class_interface_impl_8_13:
    .quad 0
.globl _class_json_pname_8_0
_class_json_pname_8_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_8
_class_json_desc_8:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_8_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_8
_class_gc_desc_8:
    .byte 1, 0, 7
.globl _class_serpname_8_0
_class_serpname_8_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_8_1
_class_serpname_8_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_8_2
_class_serpname_8_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_8
_class_serprop_8:
    .quad 3
    .quad _class_serpname_8_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_8_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_8_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_8_0
_class_vd_pkey_8_0:
    .ascii "\"message\""
.globl _class_vd_ptype_8_0
_class_vd_ptype_8_0:
    .ascii "string"
.globl _class_vd_pkey_8_1
_class_vd_pkey_8_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_8_1
_class_vd_ptype_8_1:
    .ascii "int"
.globl _class_vd_pkey_8_2
_class_vd_pkey_8_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_8_2
_class_vd_ptype_8_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_8
_class_vd_desc_8:
    .quad 3
    .quad _class_vd_pkey_8_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_8_0
    .quad 6
    .quad _class_vd_pkey_8_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_8_1
    .quad 3
    .quad _class_vd_pkey_8_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_8_2
    .quad 5
.globl _class_prop_pkey_8_0
_class_prop_pkey_8_0:
    .ascii "message"
.globl _class_prop_nkey_8_0
_class_prop_nkey_8_0:
    .ascii "message"
.globl _class_prop_pkey_8_1
_class_prop_pkey_8_1:
    .ascii "code:protected"
.globl _class_prop_nkey_8_1
_class_prop_nkey_8_1:
    .ascii "code"
.globl _class_prop_pkey_8_2
_class_prop_pkey_8_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_8_2
_class_prop_nkey_8_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_8
_class_prop_desc_8:
    .quad 3
    .quad _class_prop_pkey_8_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_8_0
    .quad 7
    .quad _class_prop_pkey_8_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_8_1
    .quad 4
    .quad _class_prop_pkey_8_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_8_2
    .quad 8
    .p2align 3
.globl _class_vtable_8
_class_vtable_8:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_8
_class_static_vtable_8:
    .quad 0
.globl _class_callable_method_name_8__u__u_construct
_class_callable_method_name_8__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_8__u__u_tostring
_class_callable_method_name_8__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_8_getcode
_class_callable_method_name_8_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_8_getfile
_class_callable_method_name_8_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_8_getline
_class_callable_method_name_8_getline:
    .ascii "getline"
.globl _class_callable_method_name_8_getmessage
_class_callable_method_name_8_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_8_getprevious
_class_callable_method_name_8_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_8_gettrace
_class_callable_method_name_8_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_8_gettraceasstring
_class_callable_method_name_8_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_8
_class_callable_methods_8:
    .quad 9
    .quad _class_callable_method_name_8__u__u_construct
    .quad 11
    .quad _class_callable_method_name_8__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_8_getcode
    .quad 7
    .quad _class_callable_method_name_8_getfile
    .quad 7
    .quad _class_callable_method_name_8_getline
    .quad 7
    .quad _class_callable_method_name_8_getmessage
    .quad 10
    .quad _class_callable_method_name_8_getprevious
    .quad 11
    .quad _class_callable_method_name_8_gettrace
    .quad 8
    .quad _class_callable_method_name_8_gettraceasstring
    .quad 16
.globl _class_interfaces_9
_class_interfaces_9:
    .quad 2
    .quad 14
    .quad _class_interface_impl_9_14
    .quad 13
    .quad _class_interface_impl_9_13
.globl _class_interface_impl_9_14
_class_interface_impl_9_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_9_13
_class_interface_impl_9_13:
    .quad 0
.globl _class_json_pname_9_0
_class_json_pname_9_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_9
_class_json_desc_9:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_9_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_9
_class_gc_desc_9:
    .byte 1, 0, 7
.globl _class_serpname_9_0
_class_serpname_9_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_9_1
_class_serpname_9_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_9_2
_class_serpname_9_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_9
_class_serprop_9:
    .quad 3
    .quad _class_serpname_9_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_9_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_9_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_9_0
_class_vd_pkey_9_0:
    .ascii "\"message\""
.globl _class_vd_ptype_9_0
_class_vd_ptype_9_0:
    .ascii "string"
.globl _class_vd_pkey_9_1
_class_vd_pkey_9_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_9_1
_class_vd_ptype_9_1:
    .ascii "int"
.globl _class_vd_pkey_9_2
_class_vd_pkey_9_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_9_2
_class_vd_ptype_9_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_9
_class_vd_desc_9:
    .quad 3
    .quad _class_vd_pkey_9_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_9_0
    .quad 6
    .quad _class_vd_pkey_9_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_9_1
    .quad 3
    .quad _class_vd_pkey_9_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_9_2
    .quad 5
.globl _class_prop_pkey_9_0
_class_prop_pkey_9_0:
    .ascii "message"
.globl _class_prop_nkey_9_0
_class_prop_nkey_9_0:
    .ascii "message"
.globl _class_prop_pkey_9_1
_class_prop_pkey_9_1:
    .ascii "code:protected"
.globl _class_prop_nkey_9_1
_class_prop_nkey_9_1:
    .ascii "code"
.globl _class_prop_pkey_9_2
_class_prop_pkey_9_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_9_2
_class_prop_nkey_9_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_9
_class_prop_desc_9:
    .quad 3
    .quad _class_prop_pkey_9_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_9_0
    .quad 7
    .quad _class_prop_pkey_9_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_9_1
    .quad 4
    .quad _class_prop_pkey_9_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_9_2
    .quad 8
    .p2align 3
.globl _class_vtable_9
_class_vtable_9:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_9
_class_static_vtable_9:
    .quad 0
.globl _class_callable_method_name_9__u__u_construct
_class_callable_method_name_9__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_9__u__u_tostring
_class_callable_method_name_9__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_9_getcode
_class_callable_method_name_9_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_9_getfile
_class_callable_method_name_9_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_9_getline
_class_callable_method_name_9_getline:
    .ascii "getline"
.globl _class_callable_method_name_9_getmessage
_class_callable_method_name_9_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_9_getprevious
_class_callable_method_name_9_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_9_gettrace
_class_callable_method_name_9_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_9_gettraceasstring
_class_callable_method_name_9_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_9
_class_callable_methods_9:
    .quad 9
    .quad _class_callable_method_name_9__u__u_construct
    .quad 11
    .quad _class_callable_method_name_9__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_9_getcode
    .quad 7
    .quad _class_callable_method_name_9_getfile
    .quad 7
    .quad _class_callable_method_name_9_getline
    .quad 7
    .quad _class_callable_method_name_9_getmessage
    .quad 10
    .quad _class_callable_method_name_9_getprevious
    .quad 11
    .quad _class_callable_method_name_9_gettrace
    .quad 8
    .quad _class_callable_method_name_9_gettraceasstring
    .quad 16
.globl _class_interfaces_10
_class_interfaces_10:
    .quad 2
    .quad 14
    .quad _class_interface_impl_10_14
    .quad 13
    .quad _class_interface_impl_10_13
.globl _class_interface_impl_10_14
_class_interface_impl_10_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_10_13
_class_interface_impl_10_13:
    .quad 0
.globl _class_json_pname_10_0
_class_json_pname_10_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_10
_class_json_desc_10:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_10_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_10
_class_gc_desc_10:
    .byte 1, 0, 7
.globl _class_serpname_10_0
_class_serpname_10_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_10_1
_class_serpname_10_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_10_2
_class_serpname_10_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_10
_class_serprop_10:
    .quad 3
    .quad _class_serpname_10_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_10_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_10_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_10_0
_class_vd_pkey_10_0:
    .ascii "\"message\""
.globl _class_vd_ptype_10_0
_class_vd_ptype_10_0:
    .ascii "string"
.globl _class_vd_pkey_10_1
_class_vd_pkey_10_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_10_1
_class_vd_ptype_10_1:
    .ascii "int"
.globl _class_vd_pkey_10_2
_class_vd_pkey_10_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_10_2
_class_vd_ptype_10_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_10
_class_vd_desc_10:
    .quad 3
    .quad _class_vd_pkey_10_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_10_0
    .quad 6
    .quad _class_vd_pkey_10_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_10_1
    .quad 3
    .quad _class_vd_pkey_10_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_10_2
    .quad 5
.globl _class_prop_pkey_10_0
_class_prop_pkey_10_0:
    .ascii "message"
.globl _class_prop_nkey_10_0
_class_prop_nkey_10_0:
    .ascii "message"
.globl _class_prop_pkey_10_1
_class_prop_pkey_10_1:
    .ascii "code:protected"
.globl _class_prop_nkey_10_1
_class_prop_nkey_10_1:
    .ascii "code"
.globl _class_prop_pkey_10_2
_class_prop_pkey_10_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_10_2
_class_prop_nkey_10_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_10
_class_prop_desc_10:
    .quad 3
    .quad _class_prop_pkey_10_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_10_0
    .quad 7
    .quad _class_prop_pkey_10_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_10_1
    .quad 4
    .quad _class_prop_pkey_10_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_10_2
    .quad 8
    .p2align 3
.globl _class_vtable_10
_class_vtable_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_10
_class_static_vtable_10:
    .quad 0
.globl _class_callable_method_name_10__u__u_construct
_class_callable_method_name_10__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_10__u__u_tostring
_class_callable_method_name_10__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_10_getcode
_class_callable_method_name_10_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_10_getfile
_class_callable_method_name_10_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_10_getline
_class_callable_method_name_10_getline:
    .ascii "getline"
.globl _class_callable_method_name_10_getmessage
_class_callable_method_name_10_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_10_getprevious
_class_callable_method_name_10_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_10_gettrace
_class_callable_method_name_10_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_10_gettraceasstring
_class_callable_method_name_10_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_10
_class_callable_methods_10:
    .quad 9
    .quad _class_callable_method_name_10__u__u_construct
    .quad 11
    .quad _class_callable_method_name_10__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_10_getcode
    .quad 7
    .quad _class_callable_method_name_10_getfile
    .quad 7
    .quad _class_callable_method_name_10_getline
    .quad 7
    .quad _class_callable_method_name_10_getmessage
    .quad 10
    .quad _class_callable_method_name_10_getprevious
    .quad 11
    .quad _class_callable_method_name_10_gettrace
    .quad 8
    .quad _class_callable_method_name_10_gettraceasstring
    .quad 16
.globl _class_interfaces_33
_class_interfaces_33:
    .quad 2
    .quad 14
    .quad _class_interface_impl_33_14
    .quad 13
    .quad _class_interface_impl_33_13
.globl _class_interface_impl_33_14
_class_interface_impl_33_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_33_13
_class_interface_impl_33_13:
    .quad 0
.globl _class_json_pname_33_0
_class_json_pname_33_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_33
_class_json_desc_33:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_33_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_33
_class_gc_desc_33:
    .byte 1, 0, 7
.globl _class_serpname_33_0
_class_serpname_33_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_33_1
_class_serpname_33_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_33_2
_class_serpname_33_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_33
_class_serprop_33:
    .quad 3
    .quad _class_serpname_33_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_33_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_33_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_33_0
_class_vd_pkey_33_0:
    .ascii "\"message\""
.globl _class_vd_ptype_33_0
_class_vd_ptype_33_0:
    .ascii "string"
.globl _class_vd_pkey_33_1
_class_vd_pkey_33_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_33_1
_class_vd_ptype_33_1:
    .ascii "int"
.globl _class_vd_pkey_33_2
_class_vd_pkey_33_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_33_2
_class_vd_ptype_33_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_33
_class_vd_desc_33:
    .quad 3
    .quad _class_vd_pkey_33_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_33_0
    .quad 6
    .quad _class_vd_pkey_33_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_33_1
    .quad 3
    .quad _class_vd_pkey_33_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_33_2
    .quad 5
.globl _class_prop_pkey_33_0
_class_prop_pkey_33_0:
    .ascii "message"
.globl _class_prop_nkey_33_0
_class_prop_nkey_33_0:
    .ascii "message"
.globl _class_prop_pkey_33_1
_class_prop_pkey_33_1:
    .ascii "code:protected"
.globl _class_prop_nkey_33_1
_class_prop_nkey_33_1:
    .ascii "code"
.globl _class_prop_pkey_33_2
_class_prop_pkey_33_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_33_2
_class_prop_nkey_33_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_33
_class_prop_desc_33:
    .quad 3
    .quad _class_prop_pkey_33_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_33_0
    .quad 7
    .quad _class_prop_pkey_33_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_33_1
    .quad 4
    .quad _class_prop_pkey_33_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_33_2
    .quad 8
    .p2align 3
.globl _class_vtable_33
_class_vtable_33:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_33
_class_static_vtable_33:
    .quad 0
.globl _class_callable_method_name_33__u__u_construct
_class_callable_method_name_33__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_33__u__u_tostring
_class_callable_method_name_33__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_33_getcode
_class_callable_method_name_33_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_33_getfile
_class_callable_method_name_33_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_33_getline
_class_callable_method_name_33_getline:
    .ascii "getline"
.globl _class_callable_method_name_33_getmessage
_class_callable_method_name_33_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_33_getprevious
_class_callable_method_name_33_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_33_gettrace
_class_callable_method_name_33_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_33_gettraceasstring
_class_callable_method_name_33_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_33
_class_callable_methods_33:
    .quad 9
    .quad _class_callable_method_name_33__u__u_construct
    .quad 11
    .quad _class_callable_method_name_33__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_33_getcode
    .quad 7
    .quad _class_callable_method_name_33_getfile
    .quad 7
    .quad _class_callable_method_name_33_getline
    .quad 7
    .quad _class_callable_method_name_33_getmessage
    .quad 10
    .quad _class_callable_method_name_33_getprevious
    .quad 11
    .quad _class_callable_method_name_33_gettrace
    .quad 8
    .quad _class_callable_method_name_33_gettraceasstring
    .quad 16
.globl _class_interfaces_43
_class_interfaces_43:
    .quad 2
    .quad 14
    .quad _class_interface_impl_43_14
    .quad 13
    .quad _class_interface_impl_43_13
.globl _class_interface_impl_43_14
_class_interface_impl_43_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_43_13
_class_interface_impl_43_13:
    .quad 0
.globl _class_json_pname_43_0
_class_json_pname_43_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_43
_class_json_desc_43:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_43_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_43
_class_gc_desc_43:
    .byte 1, 0, 7
.globl _class_serpname_43_0
_class_serpname_43_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_43_1
_class_serpname_43_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_43_2
_class_serpname_43_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_43
_class_serprop_43:
    .quad 3
    .quad _class_serpname_43_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_43_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_43_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_43_0
_class_vd_pkey_43_0:
    .ascii "\"message\""
.globl _class_vd_ptype_43_0
_class_vd_ptype_43_0:
    .ascii "string"
.globl _class_vd_pkey_43_1
_class_vd_pkey_43_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_43_1
_class_vd_ptype_43_1:
    .ascii "int"
.globl _class_vd_pkey_43_2
_class_vd_pkey_43_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_43_2
_class_vd_ptype_43_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_43
_class_vd_desc_43:
    .quad 3
    .quad _class_vd_pkey_43_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_43_0
    .quad 6
    .quad _class_vd_pkey_43_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_43_1
    .quad 3
    .quad _class_vd_pkey_43_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_43_2
    .quad 5
.globl _class_prop_pkey_43_0
_class_prop_pkey_43_0:
    .ascii "message"
.globl _class_prop_nkey_43_0
_class_prop_nkey_43_0:
    .ascii "message"
.globl _class_prop_pkey_43_1
_class_prop_pkey_43_1:
    .ascii "code:protected"
.globl _class_prop_nkey_43_1
_class_prop_nkey_43_1:
    .ascii "code"
.globl _class_prop_pkey_43_2
_class_prop_pkey_43_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_43_2
_class_prop_nkey_43_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_43
_class_prop_desc_43:
    .quad 3
    .quad _class_prop_pkey_43_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_43_0
    .quad 7
    .quad _class_prop_pkey_43_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_43_1
    .quad 4
    .quad _class_prop_pkey_43_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_43_2
    .quad 8
    .p2align 3
.globl _class_vtable_43
_class_vtable_43:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_43
_class_static_vtable_43:
    .quad 0
.globl _class_callable_method_name_43__u__u_construct
_class_callable_method_name_43__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_43__u__u_tostring
_class_callable_method_name_43__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_43_getcode
_class_callable_method_name_43_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_43_getfile
_class_callable_method_name_43_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_43_getline
_class_callable_method_name_43_getline:
    .ascii "getline"
.globl _class_callable_method_name_43_getmessage
_class_callable_method_name_43_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_43_getprevious
_class_callable_method_name_43_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_43_gettrace
_class_callable_method_name_43_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_43_gettraceasstring
_class_callable_method_name_43_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_43
_class_callable_methods_43:
    .quad 9
    .quad _class_callable_method_name_43__u__u_construct
    .quad 11
    .quad _class_callable_method_name_43__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_43_getcode
    .quad 7
    .quad _class_callable_method_name_43_getfile
    .quad 7
    .quad _class_callable_method_name_43_getline
    .quad 7
    .quad _class_callable_method_name_43_getmessage
    .quad 10
    .quad _class_callable_method_name_43_getprevious
    .quad 11
    .quad _class_callable_method_name_43_gettrace
    .quad 8
    .quad _class_callable_method_name_43_gettraceasstring
    .quad 16
.globl _class_interfaces_44
_class_interfaces_44:
    .quad 2
    .quad 14
    .quad _class_interface_impl_44_14
    .quad 13
    .quad _class_interface_impl_44_13
.globl _class_interface_impl_44_14
_class_interface_impl_44_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_44_13
_class_interface_impl_44_13:
    .quad 0
.globl _class_json_pname_44_0
_class_json_pname_44_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_44
_class_json_desc_44:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_44_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_44
_class_gc_desc_44:
    .byte 1, 0, 7
.globl _class_serpname_44_0
_class_serpname_44_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_44_1
_class_serpname_44_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_44_2
_class_serpname_44_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_44
_class_serprop_44:
    .quad 3
    .quad _class_serpname_44_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_44_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_44_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_44_0
_class_vd_pkey_44_0:
    .ascii "\"message\""
.globl _class_vd_ptype_44_0
_class_vd_ptype_44_0:
    .ascii "string"
.globl _class_vd_pkey_44_1
_class_vd_pkey_44_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_44_1
_class_vd_ptype_44_1:
    .ascii "int"
.globl _class_vd_pkey_44_2
_class_vd_pkey_44_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_44_2
_class_vd_ptype_44_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_44
_class_vd_desc_44:
    .quad 3
    .quad _class_vd_pkey_44_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_44_0
    .quad 6
    .quad _class_vd_pkey_44_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_44_1
    .quad 3
    .quad _class_vd_pkey_44_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_44_2
    .quad 5
.globl _class_prop_pkey_44_0
_class_prop_pkey_44_0:
    .ascii "message"
.globl _class_prop_nkey_44_0
_class_prop_nkey_44_0:
    .ascii "message"
.globl _class_prop_pkey_44_1
_class_prop_pkey_44_1:
    .ascii "code:protected"
.globl _class_prop_nkey_44_1
_class_prop_nkey_44_1:
    .ascii "code"
.globl _class_prop_pkey_44_2
_class_prop_pkey_44_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_44_2
_class_prop_nkey_44_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_44
_class_prop_desc_44:
    .quad 3
    .quad _class_prop_pkey_44_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_44_0
    .quad 7
    .quad _class_prop_pkey_44_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_44_1
    .quad 4
    .quad _class_prop_pkey_44_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_44_2
    .quad 8
    .p2align 3
.globl _class_vtable_44
_class_vtable_44:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_44
_class_static_vtable_44:
    .quad 0
.globl _class_callable_method_name_44__u__u_construct
_class_callable_method_name_44__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_44__u__u_tostring
_class_callable_method_name_44__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_44_getcode
_class_callable_method_name_44_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_44_getfile
_class_callable_method_name_44_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_44_getline
_class_callable_method_name_44_getline:
    .ascii "getline"
.globl _class_callable_method_name_44_getmessage
_class_callable_method_name_44_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_44_getprevious
_class_callable_method_name_44_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_44_gettrace
_class_callable_method_name_44_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_44_gettraceasstring
_class_callable_method_name_44_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_44
_class_callable_methods_44:
    .quad 9
    .quad _class_callable_method_name_44__u__u_construct
    .quad 11
    .quad _class_callable_method_name_44__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_44_getcode
    .quad 7
    .quad _class_callable_method_name_44_getfile
    .quad 7
    .quad _class_callable_method_name_44_getline
    .quad 7
    .quad _class_callable_method_name_44_getmessage
    .quad 10
    .quad _class_callable_method_name_44_getprevious
    .quad 11
    .quad _class_callable_method_name_44_gettrace
    .quad 8
    .quad _class_callable_method_name_44_gettraceasstring
    .quad 16
.globl _class_interfaces_45
_class_interfaces_45:
    .quad 2
    .quad 14
    .quad _class_interface_impl_45_14
    .quad 13
    .quad _class_interface_impl_45_13
.globl _class_interface_impl_45_14
_class_interface_impl_45_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_45_13
_class_interface_impl_45_13:
    .quad 0
.globl _class_json_pname_45_0
_class_json_pname_45_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_45
_class_json_desc_45:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_45_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_45
_class_gc_desc_45:
    .byte 1, 0, 7
.globl _class_serpname_45_0
_class_serpname_45_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_45_1
_class_serpname_45_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_45_2
_class_serpname_45_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_45
_class_serprop_45:
    .quad 3
    .quad _class_serpname_45_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_45_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_45_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_45_0
_class_vd_pkey_45_0:
    .ascii "\"message\""
.globl _class_vd_ptype_45_0
_class_vd_ptype_45_0:
    .ascii "string"
.globl _class_vd_pkey_45_1
_class_vd_pkey_45_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_45_1
_class_vd_ptype_45_1:
    .ascii "int"
.globl _class_vd_pkey_45_2
_class_vd_pkey_45_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_45_2
_class_vd_ptype_45_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_45
_class_vd_desc_45:
    .quad 3
    .quad _class_vd_pkey_45_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_45_0
    .quad 6
    .quad _class_vd_pkey_45_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_45_1
    .quad 3
    .quad _class_vd_pkey_45_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_45_2
    .quad 5
.globl _class_prop_pkey_45_0
_class_prop_pkey_45_0:
    .ascii "message"
.globl _class_prop_nkey_45_0
_class_prop_nkey_45_0:
    .ascii "message"
.globl _class_prop_pkey_45_1
_class_prop_pkey_45_1:
    .ascii "code:protected"
.globl _class_prop_nkey_45_1
_class_prop_nkey_45_1:
    .ascii "code"
.globl _class_prop_pkey_45_2
_class_prop_pkey_45_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_45_2
_class_prop_nkey_45_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_45
_class_prop_desc_45:
    .quad 3
    .quad _class_prop_pkey_45_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_45_0
    .quad 7
    .quad _class_prop_pkey_45_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_45_1
    .quad 4
    .quad _class_prop_pkey_45_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_45_2
    .quad 8
    .p2align 3
.globl _class_vtable_45
_class_vtable_45:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_45
_class_static_vtable_45:
    .quad 0
.globl _class_callable_method_name_45__u__u_construct
_class_callable_method_name_45__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_45__u__u_tostring
_class_callable_method_name_45__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_45_getcode
_class_callable_method_name_45_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_45_getfile
_class_callable_method_name_45_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_45_getline
_class_callable_method_name_45_getline:
    .ascii "getline"
.globl _class_callable_method_name_45_getmessage
_class_callable_method_name_45_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_45_getprevious
_class_callable_method_name_45_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_45_gettrace
_class_callable_method_name_45_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_45_gettraceasstring
_class_callable_method_name_45_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_45
_class_callable_methods_45:
    .quad 9
    .quad _class_callable_method_name_45__u__u_construct
    .quad 11
    .quad _class_callable_method_name_45__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_45_getcode
    .quad 7
    .quad _class_callable_method_name_45_getfile
    .quad 7
    .quad _class_callable_method_name_45_getline
    .quad 7
    .quad _class_callable_method_name_45_getmessage
    .quad 10
    .quad _class_callable_method_name_45_getprevious
    .quad 11
    .quad _class_callable_method_name_45_gettrace
    .quad 8
    .quad _class_callable_method_name_45_gettraceasstring
    .quad 16
.globl _class_interfaces_50
_class_interfaces_50:
    .quad 2
    .quad 14
    .quad _class_interface_impl_50_14
    .quad 13
    .quad _class_interface_impl_50_13
.globl _class_interface_impl_50_14
_class_interface_impl_50_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_50_13
_class_interface_impl_50_13:
    .quad 0
.globl _class_json_pname_50_0
_class_json_pname_50_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_50
_class_json_desc_50:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_50_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_50
_class_gc_desc_50:
    .byte 1, 0, 7
.globl _class_serpname_50_0
_class_serpname_50_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_50_1
_class_serpname_50_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_50_2
_class_serpname_50_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_50
_class_serprop_50:
    .quad 3
    .quad _class_serpname_50_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_50_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_50_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_50_0
_class_vd_pkey_50_0:
    .ascii "\"message\""
.globl _class_vd_ptype_50_0
_class_vd_ptype_50_0:
    .ascii "string"
.globl _class_vd_pkey_50_1
_class_vd_pkey_50_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_50_1
_class_vd_ptype_50_1:
    .ascii "int"
.globl _class_vd_pkey_50_2
_class_vd_pkey_50_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_50_2
_class_vd_ptype_50_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_50
_class_vd_desc_50:
    .quad 3
    .quad _class_vd_pkey_50_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_50_0
    .quad 6
    .quad _class_vd_pkey_50_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_50_1
    .quad 3
    .quad _class_vd_pkey_50_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_50_2
    .quad 5
.globl _class_prop_pkey_50_0
_class_prop_pkey_50_0:
    .ascii "message"
.globl _class_prop_nkey_50_0
_class_prop_nkey_50_0:
    .ascii "message"
.globl _class_prop_pkey_50_1
_class_prop_pkey_50_1:
    .ascii "code:protected"
.globl _class_prop_nkey_50_1
_class_prop_nkey_50_1:
    .ascii "code"
.globl _class_prop_pkey_50_2
_class_prop_pkey_50_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_50_2
_class_prop_nkey_50_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_50
_class_prop_desc_50:
    .quad 3
    .quad _class_prop_pkey_50_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_50_0
    .quad 7
    .quad _class_prop_pkey_50_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_50_1
    .quad 4
    .quad _class_prop_pkey_50_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_50_2
    .quad 8
    .p2align 3
.globl _class_vtable_50
_class_vtable_50:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_50
_class_static_vtable_50:
    .quad 0
.globl _class_callable_method_name_50__u__u_construct
_class_callable_method_name_50__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_50__u__u_tostring
_class_callable_method_name_50__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_50_getcode
_class_callable_method_name_50_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_50_getfile
_class_callable_method_name_50_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_50_getline
_class_callable_method_name_50_getline:
    .ascii "getline"
.globl _class_callable_method_name_50_getmessage
_class_callable_method_name_50_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_50_getprevious
_class_callable_method_name_50_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_50_gettrace
_class_callable_method_name_50_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_50_gettraceasstring
_class_callable_method_name_50_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_50
_class_callable_methods_50:
    .quad 9
    .quad _class_callable_method_name_50__u__u_construct
    .quad 11
    .quad _class_callable_method_name_50__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_50_getcode
    .quad 7
    .quad _class_callable_method_name_50_getfile
    .quad 7
    .quad _class_callable_method_name_50_getline
    .quad 7
    .quad _class_callable_method_name_50_getmessage
    .quad 10
    .quad _class_callable_method_name_50_getprevious
    .quad 11
    .quad _class_callable_method_name_50_gettrace
    .quad 8
    .quad _class_callable_method_name_50_gettraceasstring
    .quad 16
.globl _class_interfaces_51
_class_interfaces_51:
    .quad 2
    .quad 14
    .quad _class_interface_impl_51_14
    .quad 13
    .quad _class_interface_impl_51_13
.globl _class_interface_impl_51_14
_class_interface_impl_51_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_51_13
_class_interface_impl_51_13:
    .quad 0
.globl _class_json_pname_51_0
_class_json_pname_51_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_51
_class_json_desc_51:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_51_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_51
_class_gc_desc_51:
    .byte 1, 0, 7
.globl _class_serpname_51_0
_class_serpname_51_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_51_1
_class_serpname_51_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_51_2
_class_serpname_51_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_51
_class_serprop_51:
    .quad 3
    .quad _class_serpname_51_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_51_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_51_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_51_0
_class_vd_pkey_51_0:
    .ascii "\"message\""
.globl _class_vd_ptype_51_0
_class_vd_ptype_51_0:
    .ascii "string"
.globl _class_vd_pkey_51_1
_class_vd_pkey_51_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_51_1
_class_vd_ptype_51_1:
    .ascii "int"
.globl _class_vd_pkey_51_2
_class_vd_pkey_51_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_51_2
_class_vd_ptype_51_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_51
_class_vd_desc_51:
    .quad 3
    .quad _class_vd_pkey_51_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_51_0
    .quad 6
    .quad _class_vd_pkey_51_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_51_1
    .quad 3
    .quad _class_vd_pkey_51_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_51_2
    .quad 5
.globl _class_prop_pkey_51_0
_class_prop_pkey_51_0:
    .ascii "message"
.globl _class_prop_nkey_51_0
_class_prop_nkey_51_0:
    .ascii "message"
.globl _class_prop_pkey_51_1
_class_prop_pkey_51_1:
    .ascii "code:protected"
.globl _class_prop_nkey_51_1
_class_prop_nkey_51_1:
    .ascii "code"
.globl _class_prop_pkey_51_2
_class_prop_pkey_51_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_51_2
_class_prop_nkey_51_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_51
_class_prop_desc_51:
    .quad 3
    .quad _class_prop_pkey_51_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_51_0
    .quad 7
    .quad _class_prop_pkey_51_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_51_1
    .quad 4
    .quad _class_prop_pkey_51_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_51_2
    .quad 8
    .p2align 3
.globl _class_vtable_51
_class_vtable_51:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_51
_class_static_vtable_51:
    .quad 0
.globl _class_callable_method_name_51__u__u_construct
_class_callable_method_name_51__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_51__u__u_tostring
_class_callable_method_name_51__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_51_getcode
_class_callable_method_name_51_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_51_getfile
_class_callable_method_name_51_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_51_getline
_class_callable_method_name_51_getline:
    .ascii "getline"
.globl _class_callable_method_name_51_getmessage
_class_callable_method_name_51_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_51_getprevious
_class_callable_method_name_51_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_51_gettrace
_class_callable_method_name_51_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_51_gettraceasstring
_class_callable_method_name_51_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_51
_class_callable_methods_51:
    .quad 9
    .quad _class_callable_method_name_51__u__u_construct
    .quad 11
    .quad _class_callable_method_name_51__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_51_getcode
    .quad 7
    .quad _class_callable_method_name_51_getfile
    .quad 7
    .quad _class_callable_method_name_51_getline
    .quad 7
    .quad _class_callable_method_name_51_getmessage
    .quad 10
    .quad _class_callable_method_name_51_getprevious
    .quad 11
    .quad _class_callable_method_name_51_gettrace
    .quad 8
    .quad _class_callable_method_name_51_gettraceasstring
    .quad 16
.globl _class_interfaces_72
_class_interfaces_72:
    .quad 2
    .quad 14
    .quad _class_interface_impl_72_14
    .quad 13
    .quad _class_interface_impl_72_13
.globl _class_interface_impl_72_14
_class_interface_impl_72_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_72_13
_class_interface_impl_72_13:
    .quad 0
.globl _class_json_pname_72_0
_class_json_pname_72_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_72
_class_json_desc_72:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_72_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_72
_class_gc_desc_72:
    .byte 1, 0, 7
.globl _class_serpname_72_0
_class_serpname_72_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_72_1
_class_serpname_72_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_72_2
_class_serpname_72_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_72
_class_serprop_72:
    .quad 3
    .quad _class_serpname_72_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_72_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_72_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_72_0
_class_vd_pkey_72_0:
    .ascii "\"message\""
.globl _class_vd_ptype_72_0
_class_vd_ptype_72_0:
    .ascii "string"
.globl _class_vd_pkey_72_1
_class_vd_pkey_72_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_72_1
_class_vd_ptype_72_1:
    .ascii "int"
.globl _class_vd_pkey_72_2
_class_vd_pkey_72_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_72_2
_class_vd_ptype_72_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_72
_class_vd_desc_72:
    .quad 3
    .quad _class_vd_pkey_72_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_72_0
    .quad 6
    .quad _class_vd_pkey_72_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_72_1
    .quad 3
    .quad _class_vd_pkey_72_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_72_2
    .quad 5
.globl _class_prop_pkey_72_0
_class_prop_pkey_72_0:
    .ascii "message"
.globl _class_prop_nkey_72_0
_class_prop_nkey_72_0:
    .ascii "message"
.globl _class_prop_pkey_72_1
_class_prop_pkey_72_1:
    .ascii "code:protected"
.globl _class_prop_nkey_72_1
_class_prop_nkey_72_1:
    .ascii "code"
.globl _class_prop_pkey_72_2
_class_prop_pkey_72_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_72_2
_class_prop_nkey_72_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_72
_class_prop_desc_72:
    .quad 3
    .quad _class_prop_pkey_72_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_72_0
    .quad 7
    .quad _class_prop_pkey_72_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_72_1
    .quad 4
    .quad _class_prop_pkey_72_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_72_2
    .quad 8
    .p2align 3
.globl _class_vtable_72
_class_vtable_72:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_72
_class_static_vtable_72:
    .quad 0
.globl _class_callable_method_name_72__u__u_construct
_class_callable_method_name_72__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_72__u__u_tostring
_class_callable_method_name_72__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_72_getcode
_class_callable_method_name_72_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_72_getfile
_class_callable_method_name_72_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_72_getline
_class_callable_method_name_72_getline:
    .ascii "getline"
.globl _class_callable_method_name_72_getmessage
_class_callable_method_name_72_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_72_getprevious
_class_callable_method_name_72_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_72_gettrace
_class_callable_method_name_72_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_72_gettraceasstring
_class_callable_method_name_72_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_72
_class_callable_methods_72:
    .quad 9
    .quad _class_callable_method_name_72__u__u_construct
    .quad 11
    .quad _class_callable_method_name_72__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_72_getcode
    .quad 7
    .quad _class_callable_method_name_72_getfile
    .quad 7
    .quad _class_callable_method_name_72_getline
    .quad 7
    .quad _class_callable_method_name_72_getmessage
    .quad 10
    .quad _class_callable_method_name_72_getprevious
    .quad 11
    .quad _class_callable_method_name_72_gettrace
    .quad 8
    .quad _class_callable_method_name_72_gettraceasstring
    .quad 16
.globl _class_interfaces_94
_class_interfaces_94:
    .quad 2
    .quad 14
    .quad _class_interface_impl_94_14
    .quad 13
    .quad _class_interface_impl_94_13
.globl _class_interface_impl_94_14
_class_interface_impl_94_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_94_13
_class_interface_impl_94_13:
    .quad 0
.globl _class_json_pname_94_0
_class_json_pname_94_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_94
_class_json_desc_94:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_94_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_94
_class_gc_desc_94:
    .byte 1, 0, 7
.globl _class_serpname_94_0
_class_serpname_94_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_94_1
_class_serpname_94_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_94_2
_class_serpname_94_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_94
_class_serprop_94:
    .quad 3
    .quad _class_serpname_94_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_94_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_94_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_94_0
_class_vd_pkey_94_0:
    .ascii "\"message\""
.globl _class_vd_ptype_94_0
_class_vd_ptype_94_0:
    .ascii "string"
.globl _class_vd_pkey_94_1
_class_vd_pkey_94_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_94_1
_class_vd_ptype_94_1:
    .ascii "int"
.globl _class_vd_pkey_94_2
_class_vd_pkey_94_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_94_2
_class_vd_ptype_94_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_94
_class_vd_desc_94:
    .quad 3
    .quad _class_vd_pkey_94_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_94_0
    .quad 6
    .quad _class_vd_pkey_94_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_94_1
    .quad 3
    .quad _class_vd_pkey_94_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_94_2
    .quad 5
.globl _class_prop_pkey_94_0
_class_prop_pkey_94_0:
    .ascii "message"
.globl _class_prop_nkey_94_0
_class_prop_nkey_94_0:
    .ascii "message"
.globl _class_prop_pkey_94_1
_class_prop_pkey_94_1:
    .ascii "code:protected"
.globl _class_prop_nkey_94_1
_class_prop_nkey_94_1:
    .ascii "code"
.globl _class_prop_pkey_94_2
_class_prop_pkey_94_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_94_2
_class_prop_nkey_94_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_94
_class_prop_desc_94:
    .quad 3
    .quad _class_prop_pkey_94_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_94_0
    .quad 7
    .quad _class_prop_pkey_94_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_94_1
    .quad 4
    .quad _class_prop_pkey_94_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_94_2
    .quad 8
    .p2align 3
.globl _class_vtable_94
_class_vtable_94:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_94
_class_static_vtable_94:
    .quad 0
.globl _class_callable_method_name_94__u__u_construct
_class_callable_method_name_94__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_94__u__u_tostring
_class_callable_method_name_94__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_94_getcode
_class_callable_method_name_94_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_94_getfile
_class_callable_method_name_94_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_94_getline
_class_callable_method_name_94_getline:
    .ascii "getline"
.globl _class_callable_method_name_94_getmessage
_class_callable_method_name_94_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_94_getprevious
_class_callable_method_name_94_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_94_gettrace
_class_callable_method_name_94_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_94_gettraceasstring
_class_callable_method_name_94_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_94
_class_callable_methods_94:
    .quad 9
    .quad _class_callable_method_name_94__u__u_construct
    .quad 11
    .quad _class_callable_method_name_94__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_94_getcode
    .quad 7
    .quad _class_callable_method_name_94_getfile
    .quad 7
    .quad _class_callable_method_name_94_getline
    .quad 7
    .quad _class_callable_method_name_94_getmessage
    .quad 10
    .quad _class_callable_method_name_94_getprevious
    .quad 11
    .quad _class_callable_method_name_94_gettrace
    .quad 8
    .quad _class_callable_method_name_94_gettraceasstring
    .quad 16
.globl _class_interfaces_95
_class_interfaces_95:
    .quad 2
    .quad 14
    .quad _class_interface_impl_95_14
    .quad 13
    .quad _class_interface_impl_95_13
.globl _class_interface_impl_95_14
_class_interface_impl_95_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_95_13
_class_interface_impl_95_13:
    .quad 0
.globl _class_json_pname_95_0
_class_json_pname_95_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_95
_class_json_desc_95:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_95_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_95
_class_gc_desc_95:
    .byte 1, 0, 7
.globl _class_serpname_95_0
_class_serpname_95_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_95_1
_class_serpname_95_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_95_2
_class_serpname_95_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
    .p2align 3
.globl _class_serprop_95
_class_serprop_95:
    .quad 3
    .quad _class_serpname_95_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_95_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_95_2
    .quad 11
    .quad 40
    .quad 7
.globl _class_vd_pkey_95_0
_class_vd_pkey_95_0:
    .ascii "\"message\""
.globl _class_vd_ptype_95_0
_class_vd_ptype_95_0:
    .ascii "string"
.globl _class_vd_pkey_95_1
_class_vd_pkey_95_1:
    .ascii "\"code\":protected"
.globl _class_vd_ptype_95_1
_class_vd_ptype_95_1:
    .ascii "int"
.globl _class_vd_pkey_95_2
_class_vd_pkey_95_2:
    .ascii "\"previous\":protected"
.globl _class_vd_ptype_95_2
_class_vd_ptype_95_2:
    .ascii "mixed"
    .p2align 3
.globl _class_vd_desc_95
_class_vd_desc_95:
    .quad 3
    .quad _class_vd_pkey_95_0
    .quad 9
    .quad 8
    .quad 1
    .quad _class_vd_ptype_95_0
    .quad 6
    .quad _class_vd_pkey_95_1
    .quad 16
    .quad 24
    .quad 0
    .quad _class_vd_ptype_95_1
    .quad 3
    .quad _class_vd_pkey_95_2
    .quad 20
    .quad 40
    .quad 7
    .quad _class_vd_ptype_95_2
    .quad 5
.globl _class_prop_pkey_95_0
_class_prop_pkey_95_0:
    .ascii "message"
.globl _class_prop_nkey_95_0
_class_prop_nkey_95_0:
    .ascii "message"
.globl _class_prop_pkey_95_1
_class_prop_pkey_95_1:
    .ascii "code:protected"
.globl _class_prop_nkey_95_1
_class_prop_nkey_95_1:
    .ascii "code"
.globl _class_prop_pkey_95_2
_class_prop_pkey_95_2:
    .ascii "previous:protected"
.globl _class_prop_nkey_95_2
_class_prop_nkey_95_2:
    .ascii "previous"
    .p2align 3
.globl _class_prop_desc_95
_class_prop_desc_95:
    .quad 3
    .quad _class_prop_pkey_95_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_prop_nkey_95_0
    .quad 7
    .quad _class_prop_pkey_95_1
    .quad 14
    .quad 24
    .quad 0
    .quad _class_prop_nkey_95_1
    .quad 4
    .quad _class_prop_pkey_95_2
    .quad 18
    .quad 40
    .quad 7
    .quad _class_prop_nkey_95_2
    .quad 8
    .p2align 3
.globl _class_vtable_95
_class_vtable_95:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_95
_class_static_vtable_95:
    .quad 0
.globl _class_callable_method_name_95__u__u_construct
_class_callable_method_name_95__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_95__u__u_tostring
_class_callable_method_name_95__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_95_getcode
_class_callable_method_name_95_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_95_getfile
_class_callable_method_name_95_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_95_getline
_class_callable_method_name_95_getline:
    .ascii "getline"
.globl _class_callable_method_name_95_getmessage
_class_callable_method_name_95_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_95_getprevious
_class_callable_method_name_95_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_95_gettrace
_class_callable_method_name_95_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_95_gettraceasstring
_class_callable_method_name_95_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_95
_class_callable_methods_95:
    .quad 9
    .quad _class_callable_method_name_95__u__u_construct
    .quad 11
    .quad _class_callable_method_name_95__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_95_getcode
    .quad 7
    .quad _class_callable_method_name_95_getfile
    .quad 7
    .quad _class_callable_method_name_95_getline
    .quad 7
    .quad _class_callable_method_name_95_getmessage
    .quad 10
    .quad _class_callable_method_name_95_getprevious
    .quad 11
    .quad _class_callable_method_name_95_gettrace
    .quad 8
    .quad _class_callable_method_name_95_gettraceasstring
    .quad 16
.globl _class_interfaces_99
_class_interfaces_99:
    .quad 0
    .p2align 3
.globl _class_json_desc_99
_class_json_desc_99:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_99
_class_gc_desc_99:
    .byte 0
    .p2align 3
.globl _class_serprop_99
_class_serprop_99:
    .quad 0
    .p2align 3
.globl _class_vd_desc_99
_class_vd_desc_99:
    .quad 0
    .p2align 3
.globl _class_prop_desc_99
_class_prop_desc_99:
    .quad 0
    .p2align 3
.globl _class_vtable_99
_class_vtable_99:
    .quad 0
    .p2align 3
.globl _class_static_vtable_99
_class_static_vtable_99:
    .quad 0
.p2align 3
.globl _class_callable_methods_99
_class_callable_methods_99:
    .quad 0
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 99

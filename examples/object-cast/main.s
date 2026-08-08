    ; @fn name=_class_propinit_0 symbol=_class_propinit_0 synthetic=1
.align 2

.globl _class_propinit_0
_class_propinit_0:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_0_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_0_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_0
    ; @fn name=_class_propinit_1 symbol=_class_propinit_1 synthetic=1
.align 2

.globl _class_propinit_1
_class_propinit_1:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_1_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_1_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_1
    ; @fn name=_class_propinit_2 symbol=_class_propinit_2 synthetic=1
.align 2

.globl _class_propinit_2
_class_propinit_2:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_2_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_2_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_2
    ; @fn name=_class_propinit_3 symbol=_class_propinit_3 synthetic=1
.align 2

.globl _class_propinit_3
_class_propinit_3:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_3_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_3_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_3
    ; @fn name=_class_propinit_4 symbol=_class_propinit_4 synthetic=1
.align 2

.globl _class_propinit_4
_class_propinit_4:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_4_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_4_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_4
    ; @fn name=_class_propinit_5 symbol=_class_propinit_5 synthetic=1
.align 2

.globl _class_propinit_5
_class_propinit_5:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_5_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_5_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_5
    ; @fn name=_class_propinit_19 symbol=_class_propinit_19 synthetic=1
.align 2

.globl _class_propinit_19
_class_propinit_19:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_19_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_19_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_19
    ; @fn name=_class_propinit_20 symbol=_class_propinit_20 synthetic=1
.align 2

.globl _class_propinit_20
_class_propinit_20:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_20_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_20_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_20
    ; @fn name=_class_propinit_25 symbol=_class_propinit_25 synthetic=1
.align 2

.globl _class_propinit_25
_class_propinit_25:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_25_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_25_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_25
    ; @fn name=_class_propinit_26 symbol=_class_propinit_26 synthetic=1
.align 2

.globl _class_propinit_26
_class_propinit_26:
    ; prologue
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-16]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; @block name=entry
_eir__class_propinit_26_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_26_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_26
    ; @fn name=_class_propinit_28 symbol=_class_propinit_28 synthetic=1
.align 2

.globl _class_propinit_28
_class_propinit_28:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_28_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_28_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_28
    ; @fn name=_class_propinit_31 symbol=_class_propinit_31 synthetic=1
.align 2

.globl _class_propinit_31
_class_propinit_31:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_31_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_31_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_31
    ; @fn name=_class_propinit_35 symbol=_class_propinit_35 synthetic=1
.align 2

.globl _class_propinit_35
_class_propinit_35:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_35_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_35_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_35
    ; @fn name=_class_propinit_39 symbol=_class_propinit_39 synthetic=1
.align 2

.globl _class_propinit_39
_class_propinit_39:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_39_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_39_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_39
    ; @fn name=_class_propinit_40 symbol=_class_propinit_40 synthetic=1
.align 2

.globl _class_propinit_40
_class_propinit_40:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_40_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_40_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_40
    ; @fn name=_class_propinit_41 symbol=_class_propinit_41 synthetic=1
.align 2

.globl _class_propinit_41
_class_propinit_41:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_41_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_41_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_41
    ; @fn name=_class_propinit_42 symbol=_class_propinit_42 synthetic=1
.align 2

.globl _class_propinit_42
_class_propinit_42:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_42_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_42_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_42
    ; @fn name=_class_propinit_43 symbol=_class_propinit_43 synthetic=1
.align 2

.globl _class_propinit_43
_class_propinit_43:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_43_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_43_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_43
    ; @fn name=_class_propinit_44 symbol=_class_propinit_44 synthetic=1
.align 2

.globl _class_propinit_44
_class_propinit_44:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_44_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_44_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_44
    ; @fn name=_class_propinit_45 symbol=_class_propinit_45 synthetic=1
.align 2

.globl _class_propinit_45
_class_propinit_45:
    ; prologue
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-16]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; @block name=entry
_eir__class_propinit_45_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_45_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_45
    ; @fn name=_class_propinit_47 symbol=_class_propinit_47 synthetic=1
.align 2

.globl _class_propinit_47
_class_propinit_47:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_47_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_47_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_47
    ; @fn name=_class_propinit_48 symbol=_class_propinit_48 synthetic=1
.align 2

.globl _class_propinit_48
_class_propinit_48:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_48_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_48_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_48
    ; @fn name=_class_propinit_51 symbol=_class_propinit_51 synthetic=1
.align 2

.globl _class_propinit_51
_class_propinit_51:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_51_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_51_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_51
    ; @fn name=_class_propinit_55 symbol=_class_propinit_55 synthetic=1
.align 2

.globl _class_propinit_55
_class_propinit_55:
    ; prologue
    sub sp, sp, #96
    stp x29, x30, [sp, #80]
    add x29, sp, #80
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-72]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_55_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-56]
    stur x0, [x29, #-40]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-40]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
_class_propinit_55_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_55
    ; @fn name=_class_propinit_56 symbol=_class_propinit_56 synthetic=1
.align 2

.globl _class_propinit_56
_class_propinit_56:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_56_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_56_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_56
    ; @fn name=_class_propinit_57 symbol=_class_propinit_57 synthetic=1
.align 2

.globl _class_propinit_57
_class_propinit_57:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_57_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_57_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_57
    ; @fn name=_class_propinit_59 symbol=_class_propinit_59 synthetic=1
.align 2

.globl _class_propinit_59
_class_propinit_59:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_59_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_59_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_59
    ; @fn name=_class_propinit_61 symbol=_class_propinit_61 synthetic=1
.align 2

.globl _class_propinit_61
_class_propinit_61:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_61_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_61_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_64_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
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
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_64_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_64
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_65_entry_0:
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
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
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
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_class_propinit_65_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_65
    ; @fn name=_class_propinit_66 symbol=_class_propinit_66 synthetic=1
.align 2

.globl _class_propinit_66
_class_propinit_66:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_66_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_66_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_66
    ; @fn name=_class_propinit_67 symbol=_class_propinit_67 synthetic=1
.align 2

.globl _class_propinit_67
_class_propinit_67:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_67_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_67_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_67
    ; @fn name=_class_propinit_75 symbol=_class_propinit_75 synthetic=1
.align 2

.globl _class_propinit_75
_class_propinit_75:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_75_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_75_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_75
    ; @fn name=_class_propinit_77 symbol=_class_propinit_77 synthetic=1
.align 2

.globl _class_propinit_77
_class_propinit_77:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_77_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_77_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_77
    ; @fn name=_class_propinit_79 symbol=_class_propinit_79 synthetic=1
.align 2

.globl _class_propinit_79
_class_propinit_79:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_79_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_79_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_79
    ; @fn name=_class_propinit_81 symbol=_class_propinit_81 synthetic=1
.align 2

.globl _class_propinit_81
_class_propinit_81:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_81_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_81_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_81
    ; @fn name=_class_propinit_82 symbol=_class_propinit_82 synthetic=1
.align 2

.globl _class_propinit_82
_class_propinit_82:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_82_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_82_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_82
    ; @fn name=_class_propinit_84 symbol=_class_propinit_84 synthetic=1
.align 2

.globl _class_propinit_84
_class_propinit_84:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_84_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_84_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_84
    ; @fn name=_class_propinit_85 symbol=_class_propinit_85 synthetic=1
.align 2

.globl _class_propinit_85
_class_propinit_85:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_85_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_85_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_85
    ; @fn name=_class_propinit_87 symbol=_class_propinit_87 synthetic=1
.align 2

.globl _class_propinit_87
_class_propinit_87:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_87_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #88]
    str xzr, [x9, #96]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_87_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_87
    ; @fn name=_class_propinit_90 symbol=_class_propinit_90 synthetic=1
.align 2

.globl _class_propinit_90
_class_propinit_90:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_90_entry_0:
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
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
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
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_class_propinit_90_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_90
    ; @fn name=_class_propinit_91 symbol=_class_propinit_91 synthetic=1
.align 2

.globl _class_propinit_91
_class_propinit_91:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_91_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_91_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_91
    ; @fn name=_class_propinit_96 symbol=_class_propinit_96 synthetic=1
.align 2

.globl _class_propinit_96
_class_propinit_96:
    ; prologue
    sub sp, sp, #272
    stp x29, x30, [sp, #256]
    add x29, sp, #256
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
_eir__class_propinit_96_entry_0:
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
    adrp x9, _float_2@PAGE
    add x9, x9, _float_2@PAGEOFF
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
    ldur x9, [x29, #-208]
    str x9, [sp, #-16]!
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
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
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
_class_propinit_96_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_96
    ; @fn name=_class_propinit_98 symbol=_class_propinit_98 synthetic=1
.align 2

.globl _class_propinit_98
_class_propinit_98:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_98_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #88]
    str xzr, [x9, #96]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_98_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_98
    ; @fn name=_class_propinit_99 symbol=_class_propinit_99 synthetic=1
.align 2

.globl _class_propinit_99
_class_propinit_99:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_99_entry_0:
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
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_99_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_99
    ; @fn name=_class_propinit_100 symbol=_class_propinit_100 synthetic=1
.align 2

.globl _class_propinit_100
_class_propinit_100:
    ; prologue
    sub sp, sp, #432
    stp x29, x30, [sp, #416]
    add x29, sp, #416
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
_eir__class_propinit_100_entry_0:
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
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
_class_propinit_100_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_100
    ; @fn name=_class_propinit_102 symbol=_class_propinit_102 synthetic=1
.align 2

.globl _class_propinit_102
_class_propinit_102:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_102_entry_0:
    ldur x0, [x29, #-120]
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
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_102_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_102
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #704
    mov x9, sp
    add x9, x9, #688
    stp x29, x30, [x9]
    add x29, sp, #688
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #688
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #680
    str x21, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #616
    str xzr, [x9]
    sub x9, x29, #624
    str xzr, [x9]
    sub x9, x29, #632
    str xzr, [x9]
    sub x9, x29, #640
    str xzr, [x9]
    sub x9, x29, #656
    str xzr, [x9]
    sub x9, x29, #648
    str xzr, [x9]
    sub x9, x29, #664
    str xzr, [x9]
    sub x9, x29, #672
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=5 col=1 end=5:8 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=5 col=19 end=5:20 op=hash_new
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    stur x0, [x29, #-8]
    ; @src line=5 col=20 op=const_str
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #4
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ; @src line=5 col=30 op=const_str
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #9
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=5 col=19 end=5:20 op=hash_set
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    ldur x0, [x29, #-8]
    mov x5, #1
    bl __rt_hash_set
    stur x0, [x29, #-8]
    ; @src line=5 col=43 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #4
    stur x1, [x29, #-56]
    stur x2, [x29, #-48]
    ; @src line=5 col=53 end=5:57 op=const_i64
    mov x0, #8080
    mov x21, x0
    ; @src line=5 col=19 end=5:20 op=hash_set
    ldur x1, [x29, #-56]
    ldur x2, [x29, #-48]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    ldur x0, [x29, #-8]
    mov x5, #0
    bl __rt_hash_set
    stur x0, [x29, #-8]
    ; @src line=5 col=11 end=5:12 op=mixed_box
    ldur x0, [x29, #-8]
    mov x1, x0
    mov x2, xzr
    mov x0, #5
    bl __rt_mixed_from_value
    stur x0, [x29, #-72]
    ; @src line=5 col=11 end=5:12 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_hash
    ; @src line=5 col=11 end=5:12 op=object_cast
    ldur x0, [x29, #-72]
    bl __rt_object_from_mixed
    stur x0, [x29, #-80]
    ; @src line=5 col=11 end=5:12 op=release
    ldur x0, [x29, #-72]
    bl __rt_decref_mixed
    ; @src line=5 col=11 end=5:12 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_hash
    ; @src line=5 col=1 end=5:8 op=acquire
    ldur x0, [x29, #-80]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-88]
    ; @src line=5 col=1 end=5:8 op=store_local
    ldur x0, [x29, #-88]
    sub x9, x29, #616
    str x0, [x9]
    ; @src line=5 col=1 end=5:8 op=release
    ldur x0, [x29, #-80]
    bl __rt_decref_object
    ; @src line=6 col=1 end=6:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=1 end=6:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=6 end=6:13 op=load_local
    sub x9, x29, #616
    ldr x0, [x9]
    stur x0, [x29, #-96]
    ; @src line=6 col=13 end=6:15 op=prop_get
    ldur x9, [x29, #-96]
    cbz x9, _eir_main_prop_get_null_receiver_0
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_0
    ldur x0, [x29, #-96]
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #4
    bl __rt_stdclass_get
    stur x0, [x29, #-104]
    b _eir_main_prop_get_done_1
_eir_main_prop_get_null_receiver_0:
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #49
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    stur x0, [x29, #-104]
_eir_main_prop_get_done_1:
    ; @src line=6 col=13 end=6:15 op=nop
    ; @src line=6 col=1 end=6:5 op=echo_value
    ldur x0, [x29, #-104]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_2
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_object_2:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_5
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_6
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_7
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_8
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_9
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_10
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_11
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_12
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_13
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_14
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_15
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_16
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_17
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_18
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_19
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_20
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_21
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_22
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_23
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_24
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_25
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_26
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_27
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_28
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_29
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_30
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_31
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_32
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_33
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_34
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_35
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_36
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_37
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_38
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_39
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_40
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_41
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_42
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_43
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_44
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_45
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_46
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_47
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_48
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_49
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_50
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_51
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_52
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_53
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_54
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_55
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_56
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_57
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_58
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_59
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_60
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_61
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_62
    b _eir_main_mixed_string_no_match_3
_eir_main_mixed_string_Exception_5:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_RuntimeException_6:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_OutOfBoundsException_7:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_Error_8:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateException_9:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateInvalidTimeZoneException_10:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionClass_11:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionFunctionAbstract_12:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionMethod_13:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionProperty_14:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionObject_15:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionClassConstant_16:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_LogicException_17:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DomainException_18:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_SplFileInfo_19:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DirectoryIterator_20:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_FilesystemIterator_21:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_RecursiveDirectoryIterator_22:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_FiberError_23:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_BadFunctionCallException_24:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionUnionType_25:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionEnumUnitCase_26:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateMalformedPeriodStringException_27:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_TypeError_28:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_SplFileObject_29:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_SplTempFileObject_30:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_OverflowException_31:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_JsonException_32:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateError_33:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateRangeError_34:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateInvalidOperationException_35:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateUnknownException_36:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateMalformedStringException_37:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_CachingIterator_38:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_RecursiveCachingIterator_39:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ValueError_40:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionFunction_41:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_UnhandledMatchError_42:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionNamedType_43:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionEnumBackedCase_44:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_OutOfRangeException_45:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_UnderflowException_46:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionEnum_47:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_PharFileInfo_48:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_PharData_49:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionParameter_50:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionIntersectionType_51:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_BadMethodCallException_52:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_GlobIterator_53:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateMalformedIntervalStringException_54:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ArithmeticError_55:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_DateObjectError_56:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_LengthException_57:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_Phar_58:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_ReflectionException_59:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_RangeException_60:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_UnexpectedValueException_61:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_InvalidArgumentException_62:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_4
_eir_main_mixed_string_no_match_3:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_4:
    ; @src line=6 col=1 end=6:5 op=release
    ldur x0, [x29, #-104]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=21 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=6 col=1 end=6:5 op=echo_value
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=6 col=1 end=6:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=26 end=6:33 op=load_local
    sub x9, x29, #616
    ldr x0, [x9]
    stur x0, [x29, #-128]
    ; @src line=6 col=33 end=6:35 op=prop_get
    ldur x9, [x29, #-128]
    cbz x9, _eir_main_prop_get_null_receiver_63
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_63
    ldur x0, [x29, #-128]
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #4
    bl __rt_stdclass_get
    stur x0, [x29, #-136]
    b _eir_main_prop_get_done_64
_eir_main_prop_get_null_receiver_63:
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #49
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    stur x0, [x29, #-136]
_eir_main_prop_get_done_64:
    ; @src line=6 col=33 end=6:35 op=nop
    ; @src line=6 col=1 end=6:5 op=echo_value
    ldur x0, [x29, #-136]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_65
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_object_65:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_68
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_69
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_70
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_71
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_72
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_73
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_74
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_75
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_76
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_77
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_78
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_79
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_80
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_81
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_82
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_83
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_84
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_85
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_86
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_87
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_88
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_89
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_90
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_91
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_92
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_93
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_94
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_95
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_96
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_97
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_98
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_99
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_100
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_101
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_102
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_103
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_104
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_105
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_106
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_107
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_108
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_109
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_110
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_111
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_112
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_113
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_114
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_115
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_116
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_117
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_118
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_119
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_120
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_121
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_122
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_123
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_124
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_125
    b _eir_main_mixed_string_no_match_66
_eir_main_mixed_string_Exception_68:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_RuntimeException_69:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_OutOfBoundsException_70:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_Error_71:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateException_72:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateInvalidTimeZoneException_73:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionClass_74:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionFunctionAbstract_75:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionMethod_76:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionProperty_77:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionObject_78:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionClassConstant_79:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_LogicException_80:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DomainException_81:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_SplFileInfo_82:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DirectoryIterator_83:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_FilesystemIterator_84:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_RecursiveDirectoryIterator_85:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_FiberError_86:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_BadFunctionCallException_87:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionUnionType_88:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionEnumUnitCase_89:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateMalformedPeriodStringException_90:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_TypeError_91:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_SplFileObject_92:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_SplTempFileObject_93:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_OverflowException_94:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_JsonException_95:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateError_96:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateRangeError_97:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateInvalidOperationException_98:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateUnknownException_99:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateMalformedStringException_100:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_CachingIterator_101:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_RecursiveCachingIterator_102:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ValueError_103:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionFunction_104:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_UnhandledMatchError_105:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionNamedType_106:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionEnumBackedCase_107:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_OutOfRangeException_108:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_UnderflowException_109:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionEnum_110:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_PharFileInfo_111:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_PharData_112:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionParameter_113:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionIntersectionType_114:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_BadMethodCallException_115:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_GlobIterator_116:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateMalformedIntervalStringException_117:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ArithmeticError_118:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_DateObjectError_119:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_LengthException_120:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_Phar_121:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_ReflectionException_122:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_RangeException_123:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_UnexpectedValueException_124:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_InvalidArgumentException_125:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_67
_eir_main_mixed_string_no_match_66:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_67:
    ; @src line=6 col=1 end=6:5 op=release
    ldur x0, [x29, #-136]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=41 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=6 col=1 end=6:5 op=echo_value
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=9 col=1 end=9:6 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=17 end=9:18 op=array_new
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
    stur x0, [x29, #-160]
    ; @src line=9 col=18 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=9 col=18 op=array_push
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    ldur x9, [x29, #-160]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-160]
    ; @src line=9 col=23 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ; @src line=9 col=23 op=array_push
    ldur x1, [x29, #-192]
    ldur x2, [x29, #-184]
    ldur x9, [x29, #-160]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-160]
    ; @src line=9 col=9 end=9:10 op=mixed_box
    ldur x0, [x29, #-160]
    mov x1, x0
    mov x2, xzr
    mov x0, #4
    bl __rt_mixed_from_value
    stur x0, [x29, #-200]
    ; @src line=9 col=9 end=9:10 op=release
    ldur x0, [x29, #-160]
    bl __rt_decref_any
    ; @src line=9 col=9 end=9:10 op=object_cast
    ldur x0, [x29, #-200]
    bl __rt_object_from_mixed
    stur x0, [x29, #-208]
    ; @src line=9 col=9 end=9:10 op=release
    ldur x0, [x29, #-200]
    bl __rt_decref_mixed
    ; @src line=9 col=9 end=9:10 op=release
    ldur x0, [x29, #-160]
    bl __rt_decref_any
    ; @src line=9 col=1 end=9:6 op=acquire
    ldur x0, [x29, #-208]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-216]
    ; @src line=9 col=1 end=9:6 op=store_local
    ldur x0, [x29, #-216]
    sub x9, x29, #624
    str x0, [x9]
    ; @src line=9 col=1 end=9:6 op=release
    ldur x0, [x29, #-208]
    bl __rt_decref_object
    ; @src line=10 col=1 end=10:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=1 end=10:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=6 end=10:11 op=load_local
    sub x9, x29, #624
    ldr x0, [x9]
    stur x0, [x29, #-224]
    ; @src line=10 col=14 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-240]
    stur x2, [x29, #-232]
    ; @src line=10 col=11 end=10:13 op=dynamic_prop_get
    ldur x9, [x29, #-224]
    cbz x9, _eir_main_dynamic_prop_get_null_receiver_126
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_dynamic_prop_get_null_receiver_126
    ldur x0, [x29, #-224]
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    bl __rt_stdclass_get
    stur x0, [x29, #-248]
    b _eir_main_dynamic_prop_get_done_127
_eir_main_dynamic_prop_get_null_receiver_126:
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #35
    bl __rt_diag_warning
    ldur x1, [x29, #-240]
    ldur x2, [x29, #-232]
    bl __rt_diag_warning
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #10
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    stur x0, [x29, #-248]
_eir_main_dynamic_prop_get_done_127:
    ; @src line=10 col=11 end=10:13 op=nop
    ; @src line=10 col=1 end=10:5 op=echo_value
    ldur x0, [x29, #-248]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_128
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_object_128:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_131
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_132
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_133
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_134
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_135
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_136
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_137
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_138
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_139
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_140
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_141
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_142
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_143
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_144
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_145
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_146
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_147
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_148
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_149
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_150
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_151
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_152
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_153
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_154
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_155
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_156
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_157
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_158
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_159
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_160
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_161
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_162
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_163
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_164
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_165
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_166
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_167
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_168
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_169
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_170
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_171
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_172
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_173
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_174
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_175
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_176
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_177
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_178
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_179
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_180
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_181
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_182
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_183
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_184
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_185
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_186
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_187
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_188
    b _eir_main_mixed_string_no_match_129
_eir_main_mixed_string_Exception_131:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_RuntimeException_132:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_OutOfBoundsException_133:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_Error_134:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateException_135:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateInvalidTimeZoneException_136:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionClass_137:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionFunctionAbstract_138:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionMethod_139:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionProperty_140:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionObject_141:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionClassConstant_142:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_LogicException_143:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DomainException_144:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_SplFileInfo_145:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DirectoryIterator_146:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_FilesystemIterator_147:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_RecursiveDirectoryIterator_148:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_FiberError_149:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_BadFunctionCallException_150:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionUnionType_151:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionEnumUnitCase_152:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateMalformedPeriodStringException_153:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_TypeError_154:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_SplFileObject_155:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_SplTempFileObject_156:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_OverflowException_157:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_JsonException_158:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateError_159:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateRangeError_160:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateInvalidOperationException_161:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateUnknownException_162:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateMalformedStringException_163:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_CachingIterator_164:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_RecursiveCachingIterator_165:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ValueError_166:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionFunction_167:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_UnhandledMatchError_168:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionNamedType_169:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionEnumBackedCase_170:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_OutOfRangeException_171:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_UnderflowException_172:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionEnum_173:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_PharFileInfo_174:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_PharData_175:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionParameter_176:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionIntersectionType_177:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_BadMethodCallException_178:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_GlobIterator_179:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateMalformedIntervalStringException_180:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ArithmeticError_181:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_DateObjectError_182:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_LengthException_183:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_Phar_184:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_ReflectionException_185:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_RangeException_186:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_UnexpectedValueException_187:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_InvalidArgumentException_188:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_130
_eir_main_mixed_string_no_match_129:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_130:
    ; @src line=10 col=1 end=10:5 op=release
    ldur x0, [x29, #-248]
    bl __rt_decref_mixed
    ; @src line=10 col=1 end=10:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=20 end=10:25 op=load_local
    sub x9, x29, #624
    ldr x0, [x9]
    sub x9, x29, #256
    str x0, [x9]
    ; @src line=10 col=28 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #1
    sub x9, x29, #272
    str x1, [x9]
    sub x9, x29, #264
    str x2, [x9]
    ; @src line=10 col=25 end=10:27 op=dynamic_prop_get
    sub x9, x29, #256
    ldr x9, [x9]
    cbz x9, _eir_main_dynamic_prop_get_null_receiver_189
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_dynamic_prop_get_null_receiver_189
    sub x9, x29, #256
    ldr x0, [x9]
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #1
    bl __rt_stdclass_get
    sub x9, x29, #280
    str x0, [x9]
    b _eir_main_dynamic_prop_get_done_190
_eir_main_dynamic_prop_get_null_receiver_189:
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #35
    bl __rt_diag_warning
    sub x9, x29, #272
    ldr x1, [x9]
    sub x9, x29, #264
    ldr x2, [x9]
    bl __rt_diag_warning
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #10
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    sub x9, x29, #280
    str x0, [x9]
_eir_main_dynamic_prop_get_done_190:
    ; @src line=10 col=25 end=10:27 op=nop
    ; @src line=10 col=1 end=10:5 op=echo_value
    sub x9, x29, #280
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_191
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_object_191:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_194
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_195
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_196
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_197
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_198
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_199
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_200
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_201
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_202
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_203
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_204
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_205
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_206
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_207
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_208
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_209
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_210
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_211
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_212
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_213
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_214
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_215
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_216
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_217
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_218
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_219
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_220
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_221
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_222
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_223
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_224
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_225
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_226
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_227
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_228
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_229
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_230
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_231
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_232
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_233
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_234
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_235
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_236
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_237
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_238
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_239
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_240
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_241
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_242
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_243
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_244
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_245
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_246
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_247
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_248
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_249
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_250
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_251
    b _eir_main_mixed_string_no_match_192
_eir_main_mixed_string_Exception_194:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_RuntimeException_195:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_OutOfBoundsException_196:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_Error_197:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateException_198:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateInvalidTimeZoneException_199:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionClass_200:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionFunctionAbstract_201:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionMethod_202:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionProperty_203:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionObject_204:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionClassConstant_205:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_LogicException_206:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DomainException_207:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_SplFileInfo_208:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DirectoryIterator_209:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_FilesystemIterator_210:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_RecursiveDirectoryIterator_211:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_FiberError_212:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_BadFunctionCallException_213:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionUnionType_214:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionEnumUnitCase_215:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateMalformedPeriodStringException_216:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_TypeError_217:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_SplFileObject_218:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_SplTempFileObject_219:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_OverflowException_220:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_JsonException_221:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateError_222:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateRangeError_223:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateInvalidOperationException_224:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateUnknownException_225:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateMalformedStringException_226:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_CachingIterator_227:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_RecursiveCachingIterator_228:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ValueError_229:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionFunction_230:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_UnhandledMatchError_231:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionNamedType_232:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionEnumBackedCase_233:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_OutOfRangeException_234:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_UnderflowException_235:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionEnum_236:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_PharFileInfo_237:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_PharData_238:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionParameter_239:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionIntersectionType_240:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_BadMethodCallException_241:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_GlobIterator_242:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateMalformedIntervalStringException_243:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ArithmeticError_244:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_DateObjectError_245:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_LengthException_246:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_Phar_247:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_ReflectionException_248:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_RangeException_249:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_UnexpectedValueException_250:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_InvalidArgumentException_251:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_193
_eir_main_mixed_string_no_match_192:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_193:
    ; @src line=10 col=1 end=10:5 op=release
    sub x9, x29, #280
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=10 col=1 end=10:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=34 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #1
    sub x9, x29, #296
    str x1, [x9]
    sub x9, x29, #288
    str x2, [x9]
    ; @src line=10 col=1 end=10:5 op=echo_value
    sub x9, x29, #296
    ldr x1, [x9]
    sub x9, x29, #288
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=13 col=1 end=13:9 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=20 end=13:22 op=const_i64
    mov x0, #42
    mov x21, x0
    ; @src line=13 col=12 end=13:13 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    sub x9, x29, #312
    str x0, [x9]
    ; @src line=13 col=12 end=13:13 op=object_cast
    sub x9, x29, #312
    ldr x0, [x9]
    bl __rt_object_from_mixed
    sub x9, x29, #320
    str x0, [x9]
    ; @src line=13 col=12 end=13:13 op=release
    sub x9, x29, #312
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=1 end=13:9 op=acquire
    sub x9, x29, #320
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #328
    str x0, [x9]
    ; @src line=13 col=1 end=13:9 op=store_local
    sub x9, x29, #328
    ldr x0, [x9]
    sub x9, x29, #632
    str x0, [x9]
    ; @src line=13 col=1 end=13:9 op=release
    sub x9, x29, #320
    ldr x0, [x9]
    bl __rt_decref_object
    ; @src line=14 col=1 end=14:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=1 end=14:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=6 end=14:14 op=load_local
    sub x9, x29, #632
    ldr x0, [x9]
    sub x9, x29, #336
    str x0, [x9]
    ; @src line=14 col=14 end=14:16 op=prop_get
    sub x9, x29, #336
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_252
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_252
    sub x9, x29, #336
    ldr x0, [x9]
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #6
    bl __rt_stdclass_get
    sub x9, x29, #344
    str x0, [x9]
    b _eir_main_prop_get_done_253
_eir_main_prop_get_null_receiver_252:
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #51
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    sub x9, x29, #344
    str x0, [x9]
_eir_main_prop_get_done_253:
    ; @src line=14 col=14 end=14:16 op=nop
    ; @src line=14 col=1 end=14:5 op=echo_value
    sub x9, x29, #344
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_254
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_object_254:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_257
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_258
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_259
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_260
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_261
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_262
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_263
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_264
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_265
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_266
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_267
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_268
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_269
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_270
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_271
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_272
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_273
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_274
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_275
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_276
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_277
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_278
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_279
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_280
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_281
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_282
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_283
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_284
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_285
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_286
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_287
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_288
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_289
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_290
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_291
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_292
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_293
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_294
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_295
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_296
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_297
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_298
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_299
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_300
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_301
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_302
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_303
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_304
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_305
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_306
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_307
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_308
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_309
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_310
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_311
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_312
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_313
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_314
    b _eir_main_mixed_string_no_match_255
_eir_main_mixed_string_Exception_257:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_RuntimeException_258:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_OutOfBoundsException_259:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_Error_260:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateException_261:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateInvalidTimeZoneException_262:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionClass_263:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionFunctionAbstract_264:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionMethod_265:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionProperty_266:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionObject_267:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionClassConstant_268:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_LogicException_269:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DomainException_270:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_SplFileInfo_271:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DirectoryIterator_272:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_FilesystemIterator_273:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_RecursiveDirectoryIterator_274:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_FiberError_275:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_BadFunctionCallException_276:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionUnionType_277:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionEnumUnitCase_278:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateMalformedPeriodStringException_279:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_TypeError_280:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_SplFileObject_281:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_SplTempFileObject_282:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_OverflowException_283:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_JsonException_284:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateError_285:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateRangeError_286:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateInvalidOperationException_287:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateUnknownException_288:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateMalformedStringException_289:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_CachingIterator_290:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_RecursiveCachingIterator_291:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ValueError_292:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionFunction_293:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_UnhandledMatchError_294:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionNamedType_295:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionEnumBackedCase_296:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_OutOfRangeException_297:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_UnderflowException_298:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionEnum_299:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_PharFileInfo_300:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_PharData_301:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionParameter_302:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionIntersectionType_303:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_BadMethodCallException_304:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_GlobIterator_305:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateMalformedIntervalStringException_306:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ArithmeticError_307:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_DateObjectError_308:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_LengthException_309:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_Phar_310:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_ReflectionException_311:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_RangeException_312:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_UnexpectedValueException_313:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_InvalidArgumentException_314:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_256
_eir_main_mixed_string_no_match_255:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_256:
    ; @src line=14 col=1 end=14:5 op=release
    sub x9, x29, #344
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=14 col=1 end=14:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=24 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #1
    sub x9, x29, #360
    str x1, [x9]
    sub x9, x29, #352
    str x2, [x9]
    ; @src line=14 col=1 end=14:5 op=echo_value
    sub x9, x29, #360
    ldr x1, [x9]
    sub x9, x29, #352
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=17 col=1 end=17:7 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=18 end=17:22 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=17 col=10 end=17:11 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #376
    str x0, [x9]
    ; @src line=17 col=10 end=17:11 op=object_cast
    sub x9, x29, #376
    ldr x0, [x9]
    bl __rt_object_from_mixed
    sub x9, x29, #384
    str x0, [x9]
    ; @src line=17 col=10 end=17:11 op=release
    sub x9, x29, #376
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=17 col=1 end=17:7 op=acquire
    sub x9, x29, #384
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #392
    str x0, [x9]
    ; @src line=17 col=1 end=17:7 op=store_local
    sub x9, x29, #392
    ldr x0, [x9]
    sub x9, x29, #640
    str x0, [x9]
    ; @src line=17 col=1 end=17:7 op=release
    sub x9, x29, #384
    ldr x0, [x9]
    bl __rt_decref_object
    ; @src line=18 col=1 end=18:7 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=1 end=18:7 op=load_local
    sub x9, x29, #640
    ldr x0, [x9]
    sub x9, x29, #400
    str x0, [x9]
    ; @src line=18 col=17 end=18:21 op=const_bool
    mov x0, #1
    mov x21, x0
    ; @src line=18 col=1 end=18:7 op=prop_set
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #400
    ldr x0, [x9]
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #5
    ldr x3, [sp], #16
    bl __rt_stdclass_set
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=6 end=19:12 op=load_local
    sub x9, x29, #640
    ldr x0, [x9]
    sub x9, x29, #416
    str x0, [x9]
    ; @src line=19 col=12 end=19:14 op=prop_get
    sub x9, x29, #416
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_315
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_315
    sub x9, x29, #416
    ldr x0, [x9]
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #5
    bl __rt_stdclass_get
    sub x9, x29, #424
    str x0, [x9]
    b _eir_main_prop_get_done_316
_eir_main_prop_get_null_receiver_315:
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    sub x9, x29, #424
    str x0, [x9]
_eir_main_prop_get_done_316:
    ; @src line=19 col=12 end=19:14 op=nop
    ; @src line=19 col=12 end=19:14 op=is_truthy
    sub x9, x29, #424
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    ; @src line=19 col=12 end=19:14 op=release
    sub x9, x29, #424
    ldr x0, [x9]
    bl __rt_decref_mixed
    mov x0, x21
    cbnz x0, _eir_main_ternary_then_1
    b _eir_main_ternary_else_2
    ; @block name=ternary.then
_eir_main_ternary_then_1:
    ; @src line=19 col=22 op=const_str
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #6
    sub x9, x29, #448
    str x1, [x9]
    sub x9, x29, #440
    str x2, [x9]
    ; @src line=19 col=20 end=19:21 op=acquire
    sub x9, x29, #448
    ldr x1, [x9]
    sub x9, x29, #440
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #464
    str x1, [x9]
    sub x9, x29, #456
    str x2, [x9]
    ; @src line=19 col=20 end=19:21 op=store_local
    sub x9, x29, #464
    ldr x1, [x9]
    sub x9, x29, #456
    ldr x2, [x9]
    sub x9, x29, #656
    str x1, [x9]
    sub x9, x29, #648
    str x2, [x9]
    b _eir_main_ternary_merge_3
    ; @block name=ternary.else
_eir_main_ternary_else_2:
    ; @src line=19 col=34 op=const_str
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #3
    sub x9, x29, #480
    str x1, [x9]
    sub x9, x29, #472
    str x2, [x9]
    ; @src line=19 col=20 end=19:21 op=acquire
    sub x9, x29, #480
    ldr x1, [x9]
    sub x9, x29, #472
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #496
    str x1, [x9]
    sub x9, x29, #488
    str x2, [x9]
    ; @src line=19 col=20 end=19:21 op=store_local
    sub x9, x29, #496
    ldr x1, [x9]
    sub x9, x29, #488
    ldr x2, [x9]
    sub x9, x29, #656
    str x1, [x9]
    sub x9, x29, #648
    str x2, [x9]
    b _eir_main_ternary_merge_3
    ; @block name=ternary.merge
_eir_main_ternary_merge_3:
    ; @src line=19 col=20 end=19:21 op=load_local
    sub x9, x29, #656
    ldr x1, [x9]
    sub x9, x29, #648
    ldr x2, [x9]
    sub x9, x29, #512
    str x1, [x9]
    sub x9, x29, #504
    str x2, [x9]
    ; @src line=19 col=20 end=19:21 op=unset_local
    sub x9, x29, #656
    str xzr, [x9]
    sub x9, x29, #648
    str xzr, [x9]
    ; @src line=19 col=1 end=19:5 op=echo_value
    sub x9, x29, #512
    ldr x1, [x9]
    sub x9, x29, #504
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=1 end=19:5 op=release
    sub x9, x29, #512
    ldr x1, [x9]
    sub x9, x29, #504
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=23 col=1 end=23:10 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=23 col=13 end=23:27 op=object_new
    bl __rt_stdclass_new
    sub x9, x29, #520
    str x0, [x9]
    ; @src line=23 col=1 end=23:10 op=acquire
    sub x9, x29, #520
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #528
    str x0, [x9]
    ; @src line=23 col=1 end=23:10 op=store_local
    sub x9, x29, #528
    ldr x0, [x9]
    sub x9, x29, #664
    str x0, [x9]
    ; @src line=23 col=1 end=23:10 op=release
    sub x9, x29, #520
    ldr x0, [x9]
    bl __rt_decref_object
    ; @src line=24 col=1 end=24:10 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=24 col=1 end=24:10 op=load_local
    sub x9, x29, #664
    ldr x0, [x9]
    sub x9, x29, #536
    str x0, [x9]
    ; @src line=24 col=20 end=24:21 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=24 col=1 end=24:10 op=prop_set
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #536
    ldr x0, [x9]
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #5
    ldr x3, [sp], #16
    bl __rt_stdclass_set
    ; @src line=25 col=1 end=25:7 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=18 end=25:27 op=load_local
    sub x9, x29, #664
    ldr x0, [x9]
    sub x9, x29, #552
    str x0, [x9]
    ; @src line=25 col=1 end=25:7 op=acquire
    sub x9, x29, #552
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #560
    str x0, [x9]
    ; @src line=25 col=1 end=25:7 op=store_local
    sub x9, x29, #560
    ldr x0, [x9]
    sub x9, x29, #672
    str x0, [x9]
    ; @src line=25 col=1 end=25:7 op=nop
    ; @src line=26 col=1 end=26:7 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=1 end=26:7 op=load_local
    sub x9, x29, #672
    ldr x0, [x9]
    sub x9, x29, #568
    str x0, [x9]
    ; @src line=26 col=17 end=26:18 op=const_i64
    mov x0, #2
    mov x21, x0
    ; @src line=26 col=1 end=26:7 op=prop_set
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #568
    ldr x0, [x9]
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #5
    ldr x3, [sp], #16
    bl __rt_stdclass_set
    ; @src line=27 col=1 end=27:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=1 end=27:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=6 end=27:15 op=load_local
    sub x9, x29, #664
    ldr x0, [x9]
    sub x9, x29, #584
    str x0, [x9]
    ; @src line=27 col=15 end=27:17 op=prop_get
    sub x9, x29, #584
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_317
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_317
    sub x9, x29, #584
    ldr x0, [x9]
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #5
    bl __rt_stdclass_get
    sub x9, x29, #592
    str x0, [x9]
    b _eir_main_prop_get_done_318
_eir_main_prop_get_null_receiver_317:
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
    sub x9, x29, #592
    str x0, [x9]
_eir_main_prop_get_done_318:
    ; @src line=27 col=15 end=27:17 op=nop
    ; @src line=27 col=1 end=27:5 op=echo_value
    sub x9, x29, #592
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_319
    ldr x0, [sp], #16
    bl __rt_mixed_write_stdout
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_object_319:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #0
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_322
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_323
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_324
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_325
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_326
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_327
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_328
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_329
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_330
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_331
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_332
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_333
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_334
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_335
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_336
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_337
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_338
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_339
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_340
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_341
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_342
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_343
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_344
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_345
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_346
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_347
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_348
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_349
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_350
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_351
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_352
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_353
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_354
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_355
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_356
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_357
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_358
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_359
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_360
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_361
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_362
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_363
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_364
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_365
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_366
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_367
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_368
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_369
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_370
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_371
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_372
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_373
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_374
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_375
    mov x10, #84
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_376
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_377
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_378
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_379
    b _eir_main_mixed_string_no_match_320
_eir_main_mixed_string_Exception_322:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_RuntimeException_323:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_OutOfBoundsException_324:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_Error_325:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateException_326:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateInvalidTimeZoneException_327:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionClass_328:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionFunctionAbstract_329:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionMethod_330:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionProperty_331:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #152]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionObject_332:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionClassConstant_333:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #88]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_LogicException_334:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DomainException_335:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_SplFileInfo_336:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DirectoryIterator_337:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_FilesystemIterator_338:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_RecursiveDirectoryIterator_339:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_FiberError_340:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_BadFunctionCallException_341:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionUnionType_342:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionEnumUnitCase_343:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #96]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateMalformedPeriodStringException_344:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_TypeError_345:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_SplFileObject_346:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_SplTempFileObject_347:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_OverflowException_348:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_JsonException_349:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateError_350:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateRangeError_351:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateInvalidOperationException_352:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateUnknownException_353:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateMalformedStringException_354:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_CachingIterator_355:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_RecursiveCachingIterator_356:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ValueError_357:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionFunction_358:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_UnhandledMatchError_359:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionNamedType_360:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionEnumBackedCase_361:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #104]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_OutOfRangeException_362:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_UnderflowException_363:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionEnum_364:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #16]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_PharFileInfo_365:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_PharData_366:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionParameter_367:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #112]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionIntersectionType_368:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_BadMethodCallException_369:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_GlobIterator_370:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateMalformedIntervalStringException_371:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ArithmeticError_372:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_DateObjectError_373:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_LengthException_374:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_Phar_375:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #8]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_ReflectionException_376:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_RangeException_377:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_UnexpectedValueException_378:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_InvalidArgumentException_379:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_mixed_string_done_321
_eir_main_mixed_string_no_match_320:
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_321:
    ; @src line=27 col=1 end=27:5 op=release
    sub x9, x29, #592
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=27 col=1 end=27:5 op=concat_reset
    sub x9, x29, #688
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=24 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #1
    sub x9, x29, #608
    str x1, [x9]
    sub x9, x29, #600
    str x2, [x9]
    ; @src line=27 col=1 end=27:5 op=echo_value
    sub x9, x29, #608
    ldr x1, [x9]
    sub x9, x29, #600
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $config
    sub x9, x29, #616
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_380
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_380:
    ; epilogue cleanup $pair
    sub x9, x29, #624
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_381
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_381:
    ; epilogue cleanup $wrapped
    sub x9, x29, #632
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_382
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_382:
    ; epilogue cleanup $empty
    sub x9, x29, #640
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_383
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_383:
    ; epilogue cleanup $__eir_tmp0
    sub x9, x29, #656
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $original
    sub x9, x29, #664
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_384
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_384:
    ; epilogue cleanup $alias
    sub x9, x29, #672
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_385
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_385:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #680
    ldr x21, [x9]
    mov x9, sp
    add x9, x9, #688
    ldp x29, x30, [x9]
    add sp, sp, #704
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_386
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_386:
    bl __rt_ob_flush_all
    mov x0, #0
    mov x16, #1
    svc #0x80
    ; @endfn name=main
_method_Error__u__u_construct:
    b __rt_exception_construct
_method_Exception__u__u_construct:
    b __rt_exception_construct
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
_method_SplDoublyLinkedList__u__u_serialize:
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
_method_SplFixedArray__u__u_construct:
    b __rt_spl_fixed_set_size
_method_SplFixedArray__u__u_unserialize:
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
.globl _str_1
_str_1:
    .ascii "UTC"
.globl _str_3
_str_3:
    .ascii "host"
.globl _str_4
_str_4:
    .ascii "localhost"
.globl _str_5
_str_5:
    .ascii "port"
.globl _str_6
_str_6:
    .ascii "Warning: Attempt to read property \"host\" on null\n"
.globl _str_7
_str_7:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_8
_str_8:
    .ascii ":"
.globl _str_9
_str_9:
    .ascii "Warning: Attempt to read property \"port\" on null\n"
.globl _str_10
_str_10:
    .ascii "\n"
.globl _str_11
_str_11:
    .ascii "a"
.globl _str_12
_str_12:
    .ascii "b"
.globl _str_13
_str_13:
    .ascii "0"
.globl _str_14
_str_14:
    .ascii "Warning: Attempt to read property \""
.globl _str_15
_str_15:
    .ascii "\" on null\n"
.globl _str_16
_str_16:
    .ascii "1"
.globl _str_17
_str_17:
    .ascii "scalar"
.globl _str_18
_str_18:
    .ascii "Warning: Attempt to read property \"scalar\" on null\n"
.globl _str_19
_str_19:
    .ascii "ready"
.globl _str_20
_str_20:
    .ascii "Warning: Attempt to read property \"ready\" on null\n"
.globl _str_21
_str_21:
    .ascii "ready\n"
.globl _str_22
_str_22:
    .ascii "no\n"
.globl _str_23
_str_23:
    .ascii "count"
.globl _str_24
_str_24:
    .ascii "Warning: Attempt to read property \"count\" on null\n"
.p2align 3
.globl _float_2
_float_2:
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
    .quad 32
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_0
    .quad 9
    .quad 0
    .quad 0
    .quad _instanceof_name_class_abs_0
    .quad 10
    .quad 0
    .quad 0
    .quad _instanceof_name_class_1
    .quad 16
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 17
    .quad 1
    .quad 0
    .quad _instanceof_name_class_2
    .quad 20
    .quad 2
    .quad 0
    .quad _instanceof_name_class_abs_2
    .quad 21
    .quad 2
    .quad 0
    .quad _instanceof_name_class_3
    .quad 5
    .quad 3
    .quad 0
    .quad _instanceof_name_class_abs_3
    .quad 6
    .quad 3
    .quad 0
    .quad _instanceof_name_class_19
    .quad 14
    .quad 19
    .quad 0
    .quad _instanceof_name_class_abs_19
    .quad 15
    .quad 19
    .quad 0
    .quad _instanceof_name_class_35
    .quad 9
    .quad 35
    .quad 0
    .quad _instanceof_name_class_abs_35
    .quad 10
    .quad 35
    .quad 0
    .quad _instanceof_name_class_42
    .quad 13
    .quad 42
    .quad 0
    .quad _instanceof_name_class_abs_42
    .quad 14
    .quad 42
    .quad 0
    .quad _instanceof_name_class_56
    .quad 10
    .quad 56
    .quad 0
    .quad _instanceof_name_class_abs_56
    .quad 11
    .quad 56
    .quad 0
    .quad _instanceof_name_class_61
    .quad 19
    .quad 61
    .quad 0
    .quad _instanceof_name_class_abs_61
    .quad 20
    .quad 61
    .quad 0
    .quad _instanceof_name_class_66
    .quad 19
    .quad 66
    .quad 0
    .quad _instanceof_name_class_abs_66
    .quad 20
    .quad 66
    .quad 0
    .quad _instanceof_name_class_79
    .quad 15
    .quad 79
    .quad 0
    .quad _instanceof_name_class_abs_79
    .quad 16
    .quad 79
    .quad 0
    .quad _instanceof_name_class_84
    .quad 19
    .quad 84
    .quad 0
    .quad _instanceof_name_class_abs_84
    .quad 20
    .quad 84
    .quad 0
    .quad _instanceof_name_class_101
    .quad 8
    .quad 101
    .quad 0
    .quad _instanceof_name_class_abs_101
    .quad 9
    .quad 101
    .quad 0
    .quad _instanceof_name_class_102
    .quad 24
    .quad 102
    .quad 0
    .quad _instanceof_name_class_abs_102
    .quad 25
    .quad 102
    .quad 0
    .quad _instanceof_name_interface_0
    .quad 10
    .quad 0
    .quad 1
    .quad _instanceof_name_interface_abs_0
    .quad 11
    .quad 0
    .quad 1
    .quad _instanceof_name_interface_16
    .quad 9
    .quad 16
    .quad 1
    .quad _instanceof_name_interface_abs_16
    .quad 10
    .quad 16
    .quad 1
.globl _instanceof_name_class_0
_instanceof_name_class_0:
    .ascii "Exception"
.globl _instanceof_name_class_abs_0
_instanceof_name_class_abs_0:
    .ascii "\\Exception"
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_2
_instanceof_name_class_2:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_2
_instanceof_name_class_abs_2:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_3
_instanceof_name_class_3:
    .ascii "Error"
.globl _instanceof_name_class_abs_3
_instanceof_name_class_abs_3:
    .ascii "\\Error"
.globl _instanceof_name_class_19
_instanceof_name_class_19:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_19
_instanceof_name_class_abs_19:
    .ascii "\\LogicException"
.globl _instanceof_name_class_35
_instanceof_name_class_35:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_35
_instanceof_name_class_abs_35:
    .ascii "\\TypeError"
.globl _instanceof_name_class_42
_instanceof_name_class_42:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_42
_instanceof_name_class_abs_42:
    .ascii "\\JsonException"
.globl _instanceof_name_class_56
_instanceof_name_class_56:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_56
_instanceof_name_class_abs_56:
    .ascii "\\ValueError"
.globl _instanceof_name_class_61
_instanceof_name_class_61:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_61
_instanceof_name_class_abs_61:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_66
_instanceof_name_class_66:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_66
_instanceof_name_class_abs_66:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_79
_instanceof_name_class_79:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_79
_instanceof_name_class_abs_79:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_84
_instanceof_name_class_84:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_84
_instanceof_name_class_abs_84:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_101
_instanceof_name_class_101:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_101
_instanceof_name_class_abs_101:
    .ascii "\\stdClass"
.globl _instanceof_name_class_102
_instanceof_name_class_102:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_102
_instanceof_name_class_abs_102:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_interface_0
_instanceof_name_interface_0:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_0
_instanceof_name_interface_abs_0:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_16
_instanceof_name_interface_16:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_16
_instanceof_name_interface_abs_16:
    .ascii "\\Throwable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 103
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_0
    .quad 9
    .quad _class_name_1
    .quad 16
    .quad _class_name_2
    .quad 20
    .quad _class_name_3
    .quad 5
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
    .quad _class_name_19
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
    .quad _class_name_35
    .quad 9
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
    .quad _class_name_42
    .quad 13
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
    .quad _class_name_56
    .quad 10
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_61
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_66
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
    .quad _class_name_79
    .quad 15
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_84
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
    .quad _class_name_101
    .quad 8
    .quad _class_name_102
    .quad 24
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_0
_class_name_0:
    .ascii "Exception"
.globl _class_name_1
_class_name_1:
    .ascii "RuntimeException"
.globl _class_name_2
_class_name_2:
    .ascii "OutOfBoundsException"
.globl _class_name_3
_class_name_3:
    .ascii "Error"
.globl _class_name_19
_class_name_19:
    .ascii "LogicException"
.globl _class_name_35
_class_name_35:
    .ascii "TypeError"
.globl _class_name_42
_class_name_42:
    .ascii "JsonException"
.globl _class_name_56
_class_name_56:
    .ascii "ValueError"
.globl _class_name_61
_class_name_61:
    .ascii "UnhandledMatchError"
.globl _class_name_66
_class_name_66:
    .ascii "OutOfRangeException"
.globl _class_name_79
_class_name_79:
    .ascii "ArithmeticError"
.globl _class_name_84
_class_name_84:
    .ascii "ReflectionException"
.globl _class_name_101
_class_name_101:
    .ascii "stdClass"
.globl _class_name_102
_class_name_102:
    .ascii "InvalidArgumentException"
    .p2align 3
.globl _interface_name_0
_interface_name_0:
    .ascii "ArrayAccess"
.globl _interface_name_1
_interface_name_1:
    .ascii "BackedEnum"
.globl _interface_name_2
_interface_name_2:
    .ascii "Countable"
.globl _interface_name_3
_interface_name_3:
    .ascii "DateTimeInterface"
.globl _interface_name_4
_interface_name_4:
    .ascii "Iterator"
.globl _interface_name_5
_interface_name_5:
    .ascii "IteratorAggregate"
.globl _interface_name_6
_interface_name_6:
    .ascii "JsonSerializable"
.globl _interface_name_7
_interface_name_7:
    .ascii "OuterIterator"
.globl _interface_name_8
_interface_name_8:
    .ascii "RecursiveIterator"
.globl _interface_name_9
_interface_name_9:
    .ascii "Reflector"
.globl _interface_name_10
_interface_name_10:
    .ascii "SeekableIterator"
.globl _interface_name_11
_interface_name_11:
    .ascii "SplObserver"
.globl _interface_name_12
_interface_name_12:
    .ascii "SplSubject"
.globl _interface_name_13
_interface_name_13:
    .ascii "Stringable"
.globl _interface_name_14
_interface_name_14:
    .ascii "Throwable"
.globl _interface_name_15
_interface_name_15:
    .ascii "Traversable"
.globl _interface_name_16
_interface_name_16:
    .ascii "UnitEnum"
.p2align 3
.globl _interface_names_count
_interface_names_count:
    .quad 17
.globl _interface_names
_interface_names:
    .quad _interface_name_0
    .quad 11
    .quad _interface_name_1
    .quad 10
    .quad _interface_name_2
    .quad 9
    .quad _interface_name_3
    .quad 17
    .quad _interface_name_4
    .quad 8
    .quad _interface_name_5
    .quad 17
    .quad _interface_name_6
    .quad 16
    .quad _interface_name_7
    .quad 13
    .quad _interface_name_8
    .quad 17
    .quad _interface_name_9
    .quad 9
    .quad _interface_name_10
    .quad 16
    .quad _interface_name_11
    .quad 11
    .quad _interface_name_12
    .quad 10
    .quad _interface_name_13
    .quad 10
    .quad _interface_name_14
    .quad 9
    .quad _interface_name_15
    .quad 11
    .quad _interface_name_16
    .quad 8
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
    .quad 93
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 25
.globl _generator_class_id
_generator_class_id:
    .quad 89
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 11
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 97
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 68
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 94
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 3
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 19
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 1
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 66
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 2
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 102
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 35
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 56
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 84
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 79
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_0
    .quad _interface_methods_16
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_0
    .quad _class_interfaces_1
    .quad _class_interfaces_2
    .quad _class_interfaces_3
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
    .quad _class_interfaces_19
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
    .quad _class_interfaces_35
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_42
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
    .quad _class_interfaces_56
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_61
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_66
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
    .quad _class_interfaces_79
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_84
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
    .quad _class_interfaces_101
    .quad _class_interfaces_102
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_0
    .quad _class_json_desc_1
    .quad _class_json_desc_2
    .quad _class_json_desc_3
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
    .quad _class_json_desc_19
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
    .quad _class_json_desc_35
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_42
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
    .quad _class_json_desc_56
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_61
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_66
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
    .quad _class_json_desc_79
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_84
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
    .quad _class_json_desc_101
    .quad _class_json_desc_102
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 42
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
    .quad 0
    .quad 1
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
    .quad 0
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
    .quad 3
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 1
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
    .quad 3
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 3
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 19
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
    .quad 3
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 0
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
    .quad 19
.globl _class_object_payload_sizes
_class_object_payload_sizes:
    .quad 72
    .quad 72
    .quad 72
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 16
    .quad 72
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
    .quad 0
    .quad 0
    .quad 1
    .quad 0
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 103
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_0
    .quad _class_gc_desc_1
    .quad _class_gc_desc_2
    .quad _class_gc_desc_3
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
    .quad _class_gc_desc_19
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
    .quad _class_gc_desc_35
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_42
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
    .quad _class_gc_desc_56
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_61
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_66
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
    .quad _class_gc_desc_79
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_84
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
    .quad _class_gc_desc_101
    .quad _class_gc_desc_102
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_0
    .quad _class_vtable_1
    .quad _class_vtable_2
    .quad _class_vtable_3
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
    .quad _class_vtable_19
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
    .quad _class_vtable_35
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_42
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
    .quad _class_vtable_56
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_61
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_66
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
    .quad _class_vtable_79
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_84
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
    .quad _class_vtable_101
    .quad _class_vtable_102
.globl _class_destruct_count
_class_destruct_count:
    .quad 103
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
    .quad 0
    .quad 0
    .quad 0
.globl _class_clone_count
_class_clone_count:
    .quad 103
.globl _class_clone_ptrs
_class_clone_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 0
    .quad 0
    .quad 0
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad _class_propinit_0
    .quad _class_propinit_1
    .quad _class_propinit_2
    .quad _class_propinit_3
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_19
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_35
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_42
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_56
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_61
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_66
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_79
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_84
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_102
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_0
    .quad _class_serprop_1
    .quad _class_serprop_2
    .quad _class_serprop_3
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
    .quad _class_serprop_19
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
    .quad _class_serprop_35
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_42
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
    .quad _class_serprop_56
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_61
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_66
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
    .quad _class_serprop_79
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_84
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
    .quad _class_serprop_101
    .quad _class_serprop_102
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_0
    .quad _class_static_vtable_1
    .quad _class_static_vtable_2
    .quad _class_static_vtable_3
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
    .quad _class_static_vtable_19
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
    .quad _class_static_vtable_35
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_42
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
    .quad _class_static_vtable_56
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_61
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_66
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
    .quad _class_static_vtable_79
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_84
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
    .quad _class_static_vtable_101
    .quad _class_static_vtable_102
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_0
    .quad _class_callable_methods_1
    .quad _class_callable_methods_2
    .quad _class_callable_methods_3
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
    .quad _class_callable_methods_19
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
    .quad _class_callable_methods_35
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_42
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
    .quad _class_callable_methods_56
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_61
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_66
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
    .quad _class_callable_methods_79
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_84
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
    .quad _class_callable_methods_101
    .quad _class_callable_methods_102
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
.globl _class_by_name_str_0
_class_by_name_str_0:
    .ascii "Exception"
.globl _class_by_name_str_1
_class_by_name_str_1:
    .ascii "RuntimeException"
.globl _class_by_name_str_2
_class_by_name_str_2:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_3
_class_by_name_str_3:
    .ascii "Error"
.globl _class_by_name_str_19
_class_by_name_str_19:
    .ascii "LogicException"
.globl _class_by_name_str_35
_class_by_name_str_35:
    .ascii "TypeError"
.globl _class_by_name_str_42
_class_by_name_str_42:
    .ascii "JsonException"
.globl _class_by_name_str_56
_class_by_name_str_56:
    .ascii "ValueError"
.globl _class_by_name_str_61
_class_by_name_str_61:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_66
_class_by_name_str_66:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_79
_class_by_name_str_79:
    .ascii "ArithmeticError"
.globl _class_by_name_str_84
_class_by_name_str_84:
    .ascii "ReflectionException"
.globl _class_by_name_str_101
_class_by_name_str_101:
    .ascii "stdClass"
.globl _class_by_name_str_102
_class_by_name_str_102:
    .ascii "InvalidArgumentException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 14
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_0
    .quad 9
    .quad 0
    .quad 72
    .quad _class_by_name_str_1
    .quad 16
    .quad 1
    .quad 72
    .quad _class_by_name_str_2
    .quad 20
    .quad 2
    .quad 72
    .quad _class_by_name_str_3
    .quad 5
    .quad 3
    .quad 72
    .quad _class_by_name_str_19
    .quad 14
    .quad 19
    .quad 72
    .quad _class_by_name_str_35
    .quad 9
    .quad 35
    .quad 72
    .quad _class_by_name_str_42
    .quad 13
    .quad 42
    .quad 72
    .quad _class_by_name_str_56
    .quad 10
    .quad 56
    .quad 72
    .quad _class_by_name_str_61
    .quad 19
    .quad 61
    .quad 72
    .quad _class_by_name_str_66
    .quad 19
    .quad 66
    .quad 72
    .quad _class_by_name_str_79
    .quad 15
    .quad 79
    .quad 72
    .quad _class_by_name_str_84
    .quad 19
    .quad 84
    .quad 72
    .quad _class_by_name_str_101
    .quad 8
    .quad 101
    .quad 16
    .quad _class_by_name_str_102
    .quad 24
    .quad 102
    .quad 72
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 103
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
    .quad _class_attributes_missing
    .quad _class_attributes_missing
    .quad _class_attributes_missing
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_0
_interface_methods_0:
    .quad 1
    .quad 0
.globl _interface_methods_16
_interface_methods_16:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _class_interfaces_0
_class_interfaces_0:
    .quad 2
    .quad 16
    .quad _class_interface_impl_0_16
    .quad 0
    .quad _class_interface_impl_0_0
.globl _class_interface_impl_0_16
_class_interface_impl_0_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_0_0
_class_interface_impl_0_0:
    .quad 0
.globl _class_json_pname_0_0
_class_json_pname_0_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_0
_class_json_desc_0:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_0_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_0
_class_gc_desc_0:
    .byte 1, 0, 7, 4
.globl _class_serpname_0_0
_class_serpname_0_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_0_1
_class_serpname_0_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_0_2
_class_serpname_0_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_0_3
_class_serpname_0_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_0
_class_serprop_0:
    .quad 4
    .quad _class_serpname_0_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_0_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_0_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_0_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_0
_class_vtable_0:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_0
_class_static_vtable_0:
    .quad 0
.globl _class_callable_method_name_0__u__u_construct
_class_callable_method_name_0__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_0__u__u_tostring
_class_callable_method_name_0__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_0_getcode
_class_callable_method_name_0_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_0_getfile
_class_callable_method_name_0_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_0_getline
_class_callable_method_name_0_getline:
    .ascii "getline"
.globl _class_callable_method_name_0_getmessage
_class_callable_method_name_0_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_0_getprevious
_class_callable_method_name_0_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_0_gettrace
_class_callable_method_name_0_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_0_gettraceasstring
_class_callable_method_name_0_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_0
_class_callable_methods_0:
    .quad 9
    .quad _class_callable_method_name_0__u__u_construct
    .quad 11
    .quad _class_callable_method_name_0__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_0_getcode
    .quad 7
    .quad _class_callable_method_name_0_getfile
    .quad 7
    .quad _class_callable_method_name_0_getline
    .quad 7
    .quad _class_callable_method_name_0_getmessage
    .quad 10
    .quad _class_callable_method_name_0_getprevious
    .quad 11
    .quad _class_callable_method_name_0_gettrace
    .quad 8
    .quad _class_callable_method_name_0_gettraceasstring
    .quad 16
.globl _class_interfaces_1
_class_interfaces_1:
    .quad 2
    .quad 16
    .quad _class_interface_impl_1_16
    .quad 0
    .quad _class_interface_impl_1_0
.globl _class_interface_impl_1_16
_class_interface_impl_1_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_0
_class_interface_impl_1_0:
    .quad 0
.globl _class_json_pname_1_0
_class_json_pname_1_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_1
_class_json_desc_1:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_1_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_1
_class_gc_desc_1:
    .byte 1, 0, 7, 4
.globl _class_serpname_1_0
_class_serpname_1_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_1_1
_class_serpname_1_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_1_2
_class_serpname_1_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_1_3
_class_serpname_1_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_1
_class_serprop_1:
    .quad 4
    .quad _class_serpname_1_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_1_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_1_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_1_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_1
_class_vtable_1:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_1
_class_static_vtable_1:
    .quad 0
.globl _class_callable_method_name_1__u__u_construct
_class_callable_method_name_1__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_1__u__u_tostring
_class_callable_method_name_1__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_1_getcode
_class_callable_method_name_1_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_1_getfile
_class_callable_method_name_1_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_1_getline
_class_callable_method_name_1_getline:
    .ascii "getline"
.globl _class_callable_method_name_1_getmessage
_class_callable_method_name_1_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_1_getprevious
_class_callable_method_name_1_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_1_gettrace
_class_callable_method_name_1_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_1_gettraceasstring
_class_callable_method_name_1_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_1
_class_callable_methods_1:
    .quad 9
    .quad _class_callable_method_name_1__u__u_construct
    .quad 11
    .quad _class_callable_method_name_1__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_1_getcode
    .quad 7
    .quad _class_callable_method_name_1_getfile
    .quad 7
    .quad _class_callable_method_name_1_getline
    .quad 7
    .quad _class_callable_method_name_1_getmessage
    .quad 10
    .quad _class_callable_method_name_1_getprevious
    .quad 11
    .quad _class_callable_method_name_1_gettrace
    .quad 8
    .quad _class_callable_method_name_1_gettraceasstring
    .quad 16
.globl _class_interfaces_2
_class_interfaces_2:
    .quad 2
    .quad 16
    .quad _class_interface_impl_2_16
    .quad 0
    .quad _class_interface_impl_2_0
.globl _class_interface_impl_2_16
_class_interface_impl_2_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_2_0
_class_interface_impl_2_0:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_2_0
_class_serpname_2_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_2_1
_class_serpname_2_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_2_2
_class_serpname_2_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_2_3
_class_serpname_2_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_2
_class_serprop_2:
    .quad 4
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
    .quad _class_serpname_2_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_2
_class_vtable_2:
    .quad _method_Exception__u__u_construct
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
    .quad 16
    .quad _class_interface_impl_3_16
    .quad 0
    .quad _class_interface_impl_3_0
.globl _class_interface_impl_3_16
_class_interface_impl_3_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_3_0
_class_interface_impl_3_0:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_3_0
_class_serpname_3_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_3_1
_class_serpname_3_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_3_2
_class_serpname_3_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_3_3
_class_serpname_3_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_3
_class_serprop_3:
    .quad 4
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
    .quad _class_serpname_3_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_3
_class_vtable_3:
    .quad _method_Error__u__u_construct
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
.globl _class_interfaces_19
_class_interfaces_19:
    .quad 2
    .quad 16
    .quad _class_interface_impl_19_16
    .quad 0
    .quad _class_interface_impl_19_0
.globl _class_interface_impl_19_16
_class_interface_impl_19_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_19_0
_class_interface_impl_19_0:
    .quad 0
.globl _class_json_pname_19_0
_class_json_pname_19_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_19
_class_json_desc_19:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_19_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_19
_class_gc_desc_19:
    .byte 1, 0, 7, 4
.globl _class_serpname_19_0
_class_serpname_19_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_19_1
_class_serpname_19_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_19_2
_class_serpname_19_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_19_3
_class_serpname_19_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_19
_class_serprop_19:
    .quad 4
    .quad _class_serpname_19_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_19_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_19_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_19_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_19
_class_vtable_19:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_19
_class_static_vtable_19:
    .quad 0
.globl _class_callable_method_name_19__u__u_construct
_class_callable_method_name_19__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_19__u__u_tostring
_class_callable_method_name_19__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_19_getcode
_class_callable_method_name_19_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_19_getfile
_class_callable_method_name_19_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_19_getline
_class_callable_method_name_19_getline:
    .ascii "getline"
.globl _class_callable_method_name_19_getmessage
_class_callable_method_name_19_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_19_getprevious
_class_callable_method_name_19_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_19_gettrace
_class_callable_method_name_19_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_19_gettraceasstring
_class_callable_method_name_19_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_19
_class_callable_methods_19:
    .quad 9
    .quad _class_callable_method_name_19__u__u_construct
    .quad 11
    .quad _class_callable_method_name_19__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_19_getcode
    .quad 7
    .quad _class_callable_method_name_19_getfile
    .quad 7
    .quad _class_callable_method_name_19_getline
    .quad 7
    .quad _class_callable_method_name_19_getmessage
    .quad 10
    .quad _class_callable_method_name_19_getprevious
    .quad 11
    .quad _class_callable_method_name_19_gettrace
    .quad 8
    .quad _class_callable_method_name_19_gettraceasstring
    .quad 16
.globl _class_interfaces_35
_class_interfaces_35:
    .quad 2
    .quad 16
    .quad _class_interface_impl_35_16
    .quad 0
    .quad _class_interface_impl_35_0
.globl _class_interface_impl_35_16
_class_interface_impl_35_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_35_0
_class_interface_impl_35_0:
    .quad 0
.globl _class_json_pname_35_0
_class_json_pname_35_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_35
_class_json_desc_35:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_35_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_35
_class_gc_desc_35:
    .byte 1, 0, 7, 4
.globl _class_serpname_35_0
_class_serpname_35_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_35_1
_class_serpname_35_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_35_2
_class_serpname_35_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_35_3
_class_serpname_35_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_35
_class_serprop_35:
    .quad 4
    .quad _class_serpname_35_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_35_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_35_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_35_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_35
_class_vtable_35:
    .quad _method_Error__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_35
_class_static_vtable_35:
    .quad 0
.globl _class_callable_method_name_35__u__u_construct
_class_callable_method_name_35__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_35__u__u_tostring
_class_callable_method_name_35__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_35_getcode
_class_callable_method_name_35_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_35_getfile
_class_callable_method_name_35_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_35_getline
_class_callable_method_name_35_getline:
    .ascii "getline"
.globl _class_callable_method_name_35_getmessage
_class_callable_method_name_35_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_35_getprevious
_class_callable_method_name_35_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_35_gettrace
_class_callable_method_name_35_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_35_gettraceasstring
_class_callable_method_name_35_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_35
_class_callable_methods_35:
    .quad 9
    .quad _class_callable_method_name_35__u__u_construct
    .quad 11
    .quad _class_callable_method_name_35__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_35_getcode
    .quad 7
    .quad _class_callable_method_name_35_getfile
    .quad 7
    .quad _class_callable_method_name_35_getline
    .quad 7
    .quad _class_callable_method_name_35_getmessage
    .quad 10
    .quad _class_callable_method_name_35_getprevious
    .quad 11
    .quad _class_callable_method_name_35_gettrace
    .quad 8
    .quad _class_callable_method_name_35_gettraceasstring
    .quad 16
.globl _class_interfaces_42
_class_interfaces_42:
    .quad 2
    .quad 16
    .quad _class_interface_impl_42_16
    .quad 0
    .quad _class_interface_impl_42_0
.globl _class_interface_impl_42_16
_class_interface_impl_42_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_42_0
_class_interface_impl_42_0:
    .quad 0
.globl _class_json_pname_42_0
_class_json_pname_42_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_42
_class_json_desc_42:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_42_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_42
_class_gc_desc_42:
    .byte 1, 0, 7, 4
.globl _class_serpname_42_0
_class_serpname_42_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_42_1
_class_serpname_42_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_42_2
_class_serpname_42_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_42_3
_class_serpname_42_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_42
_class_serprop_42:
    .quad 4
    .quad _class_serpname_42_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_42_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_42_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_42_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_42
_class_vtable_42:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_42
_class_static_vtable_42:
    .quad 0
.globl _class_callable_method_name_42__u__u_construct
_class_callable_method_name_42__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_42__u__u_tostring
_class_callable_method_name_42__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_42_getcode
_class_callable_method_name_42_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_42_getfile
_class_callable_method_name_42_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_42_getline
_class_callable_method_name_42_getline:
    .ascii "getline"
.globl _class_callable_method_name_42_getmessage
_class_callable_method_name_42_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_42_getprevious
_class_callable_method_name_42_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_42_gettrace
_class_callable_method_name_42_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_42_gettraceasstring
_class_callable_method_name_42_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_42
_class_callable_methods_42:
    .quad 9
    .quad _class_callable_method_name_42__u__u_construct
    .quad 11
    .quad _class_callable_method_name_42__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_42_getcode
    .quad 7
    .quad _class_callable_method_name_42_getfile
    .quad 7
    .quad _class_callable_method_name_42_getline
    .quad 7
    .quad _class_callable_method_name_42_getmessage
    .quad 10
    .quad _class_callable_method_name_42_getprevious
    .quad 11
    .quad _class_callable_method_name_42_gettrace
    .quad 8
    .quad _class_callable_method_name_42_gettraceasstring
    .quad 16
.globl _class_interfaces_56
_class_interfaces_56:
    .quad 2
    .quad 16
    .quad _class_interface_impl_56_16
    .quad 0
    .quad _class_interface_impl_56_0
.globl _class_interface_impl_56_16
_class_interface_impl_56_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_56_0
_class_interface_impl_56_0:
    .quad 0
.globl _class_json_pname_56_0
_class_json_pname_56_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_56
_class_json_desc_56:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_56_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_56
_class_gc_desc_56:
    .byte 1, 0, 7, 4
.globl _class_serpname_56_0
_class_serpname_56_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_56_1
_class_serpname_56_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_56_2
_class_serpname_56_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_56_3
_class_serpname_56_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_56
_class_serprop_56:
    .quad 4
    .quad _class_serpname_56_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_56_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_56_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_56_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_56
_class_vtable_56:
    .quad _method_Error__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_56
_class_static_vtable_56:
    .quad 0
.globl _class_callable_method_name_56__u__u_construct
_class_callable_method_name_56__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_56__u__u_tostring
_class_callable_method_name_56__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_56_getcode
_class_callable_method_name_56_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_56_getfile
_class_callable_method_name_56_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_56_getline
_class_callable_method_name_56_getline:
    .ascii "getline"
.globl _class_callable_method_name_56_getmessage
_class_callable_method_name_56_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_56_getprevious
_class_callable_method_name_56_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_56_gettrace
_class_callable_method_name_56_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_56_gettraceasstring
_class_callable_method_name_56_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_56
_class_callable_methods_56:
    .quad 9
    .quad _class_callable_method_name_56__u__u_construct
    .quad 11
    .quad _class_callable_method_name_56__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_56_getcode
    .quad 7
    .quad _class_callable_method_name_56_getfile
    .quad 7
    .quad _class_callable_method_name_56_getline
    .quad 7
    .quad _class_callable_method_name_56_getmessage
    .quad 10
    .quad _class_callable_method_name_56_getprevious
    .quad 11
    .quad _class_callable_method_name_56_gettrace
    .quad 8
    .quad _class_callable_method_name_56_gettraceasstring
    .quad 16
.globl _class_interfaces_61
_class_interfaces_61:
    .quad 2
    .quad 16
    .quad _class_interface_impl_61_16
    .quad 0
    .quad _class_interface_impl_61_0
.globl _class_interface_impl_61_16
_class_interface_impl_61_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_61_0
_class_interface_impl_61_0:
    .quad 0
.globl _class_json_pname_61_0
_class_json_pname_61_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_61
_class_json_desc_61:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_61_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_61
_class_gc_desc_61:
    .byte 1, 0, 7, 4
.globl _class_serpname_61_0
_class_serpname_61_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_61_1
_class_serpname_61_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_61_2
_class_serpname_61_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_61_3
_class_serpname_61_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_61
_class_serprop_61:
    .quad 4
    .quad _class_serpname_61_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_61_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_61_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_61_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_61
_class_vtable_61:
    .quad _method_Error__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_61
_class_static_vtable_61:
    .quad 0
.globl _class_callable_method_name_61__u__u_construct
_class_callable_method_name_61__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_61__u__u_tostring
_class_callable_method_name_61__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_61_getcode
_class_callable_method_name_61_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_61_getfile
_class_callable_method_name_61_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_61_getline
_class_callable_method_name_61_getline:
    .ascii "getline"
.globl _class_callable_method_name_61_getmessage
_class_callable_method_name_61_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_61_getprevious
_class_callable_method_name_61_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_61_gettrace
_class_callable_method_name_61_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_61_gettraceasstring
_class_callable_method_name_61_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_61
_class_callable_methods_61:
    .quad 9
    .quad _class_callable_method_name_61__u__u_construct
    .quad 11
    .quad _class_callable_method_name_61__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_61_getcode
    .quad 7
    .quad _class_callable_method_name_61_getfile
    .quad 7
    .quad _class_callable_method_name_61_getline
    .quad 7
    .quad _class_callable_method_name_61_getmessage
    .quad 10
    .quad _class_callable_method_name_61_getprevious
    .quad 11
    .quad _class_callable_method_name_61_gettrace
    .quad 8
    .quad _class_callable_method_name_61_gettraceasstring
    .quad 16
.globl _class_interfaces_66
_class_interfaces_66:
    .quad 2
    .quad 16
    .quad _class_interface_impl_66_16
    .quad 0
    .quad _class_interface_impl_66_0
.globl _class_interface_impl_66_16
_class_interface_impl_66_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_66_0
_class_interface_impl_66_0:
    .quad 0
.globl _class_json_pname_66_0
_class_json_pname_66_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_66
_class_json_desc_66:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_66_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_66
_class_gc_desc_66:
    .byte 1, 0, 7, 4
.globl _class_serpname_66_0
_class_serpname_66_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_66_1
_class_serpname_66_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_66_2
_class_serpname_66_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_66_3
_class_serpname_66_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_66
_class_serprop_66:
    .quad 4
    .quad _class_serpname_66_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_66_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_66_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_66_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_66
_class_vtable_66:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_66
_class_static_vtable_66:
    .quad 0
.globl _class_callable_method_name_66__u__u_construct
_class_callable_method_name_66__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_66__u__u_tostring
_class_callable_method_name_66__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_66_getcode
_class_callable_method_name_66_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_66_getfile
_class_callable_method_name_66_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_66_getline
_class_callable_method_name_66_getline:
    .ascii "getline"
.globl _class_callable_method_name_66_getmessage
_class_callable_method_name_66_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_66_getprevious
_class_callable_method_name_66_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_66_gettrace
_class_callable_method_name_66_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_66_gettraceasstring
_class_callable_method_name_66_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_66
_class_callable_methods_66:
    .quad 9
    .quad _class_callable_method_name_66__u__u_construct
    .quad 11
    .quad _class_callable_method_name_66__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_66_getcode
    .quad 7
    .quad _class_callable_method_name_66_getfile
    .quad 7
    .quad _class_callable_method_name_66_getline
    .quad 7
    .quad _class_callable_method_name_66_getmessage
    .quad 10
    .quad _class_callable_method_name_66_getprevious
    .quad 11
    .quad _class_callable_method_name_66_gettrace
    .quad 8
    .quad _class_callable_method_name_66_gettraceasstring
    .quad 16
.globl _class_interfaces_79
_class_interfaces_79:
    .quad 2
    .quad 16
    .quad _class_interface_impl_79_16
    .quad 0
    .quad _class_interface_impl_79_0
.globl _class_interface_impl_79_16
_class_interface_impl_79_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_79_0
_class_interface_impl_79_0:
    .quad 0
.globl _class_json_pname_79_0
_class_json_pname_79_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_79
_class_json_desc_79:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_79_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_79
_class_gc_desc_79:
    .byte 1, 0, 7, 4
.globl _class_serpname_79_0
_class_serpname_79_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_79_1
_class_serpname_79_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_79_2
_class_serpname_79_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_79_3
_class_serpname_79_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_79
_class_serprop_79:
    .quad 4
    .quad _class_serpname_79_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_79_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_79_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_79_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_79
_class_vtable_79:
    .quad _method_Error__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_79
_class_static_vtable_79:
    .quad 0
.globl _class_callable_method_name_79__u__u_construct
_class_callable_method_name_79__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_79__u__u_tostring
_class_callable_method_name_79__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_79_getcode
_class_callable_method_name_79_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_79_getfile
_class_callable_method_name_79_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_79_getline
_class_callable_method_name_79_getline:
    .ascii "getline"
.globl _class_callable_method_name_79_getmessage
_class_callable_method_name_79_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_79_getprevious
_class_callable_method_name_79_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_79_gettrace
_class_callable_method_name_79_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_79_gettraceasstring
_class_callable_method_name_79_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_79
_class_callable_methods_79:
    .quad 9
    .quad _class_callable_method_name_79__u__u_construct
    .quad 11
    .quad _class_callable_method_name_79__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_79_getcode
    .quad 7
    .quad _class_callable_method_name_79_getfile
    .quad 7
    .quad _class_callable_method_name_79_getline
    .quad 7
    .quad _class_callable_method_name_79_getmessage
    .quad 10
    .quad _class_callable_method_name_79_getprevious
    .quad 11
    .quad _class_callable_method_name_79_gettrace
    .quad 8
    .quad _class_callable_method_name_79_gettraceasstring
    .quad 16
.globl _class_interfaces_84
_class_interfaces_84:
    .quad 2
    .quad 16
    .quad _class_interface_impl_84_16
    .quad 0
    .quad _class_interface_impl_84_0
.globl _class_interface_impl_84_16
_class_interface_impl_84_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_84_0
_class_interface_impl_84_0:
    .quad 0
.globl _class_json_pname_84_0
_class_json_pname_84_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_84
_class_json_desc_84:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_84_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_84
_class_gc_desc_84:
    .byte 1, 0, 7, 4
.globl _class_serpname_84_0
_class_serpname_84_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_84_1
_class_serpname_84_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_84_2
_class_serpname_84_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_84_3
_class_serpname_84_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_84
_class_serprop_84:
    .quad 4
    .quad _class_serpname_84_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_84_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_84_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_84_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_84
_class_vtable_84:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_84
_class_static_vtable_84:
    .quad 0
.globl _class_callable_method_name_84__u__u_construct
_class_callable_method_name_84__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_84__u__u_tostring
_class_callable_method_name_84__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_84_getcode
_class_callable_method_name_84_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_84_getfile
_class_callable_method_name_84_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_84_getline
_class_callable_method_name_84_getline:
    .ascii "getline"
.globl _class_callable_method_name_84_getmessage
_class_callable_method_name_84_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_84_getprevious
_class_callable_method_name_84_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_84_gettrace
_class_callable_method_name_84_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_84_gettraceasstring
_class_callable_method_name_84_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_84
_class_callable_methods_84:
    .quad 9
    .quad _class_callable_method_name_84__u__u_construct
    .quad 11
    .quad _class_callable_method_name_84__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_84_getcode
    .quad 7
    .quad _class_callable_method_name_84_getfile
    .quad 7
    .quad _class_callable_method_name_84_getline
    .quad 7
    .quad _class_callable_method_name_84_getmessage
    .quad 10
    .quad _class_callable_method_name_84_getprevious
    .quad 11
    .quad _class_callable_method_name_84_gettrace
    .quad 8
    .quad _class_callable_method_name_84_gettraceasstring
    .quad 16
.globl _class_interfaces_101
_class_interfaces_101:
    .quad 0
    .p2align 3
.globl _class_json_desc_101
_class_json_desc_101:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_101
_class_gc_desc_101:
    .byte 0
    .p2align 3
.globl _class_serprop_101
_class_serprop_101:
    .quad 0
    .p2align 3
.globl _class_vtable_101
_class_vtable_101:
    .quad 0
    .p2align 3
.globl _class_static_vtable_101
_class_static_vtable_101:
    .quad 0
.p2align 3
.globl _class_callable_methods_101
_class_callable_methods_101:
    .quad 0
.globl _class_interfaces_102
_class_interfaces_102:
    .quad 2
    .quad 16
    .quad _class_interface_impl_102_16
    .quad 0
    .quad _class_interface_impl_102_0
.globl _class_interface_impl_102_16
_class_interface_impl_102_16:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_102_0
_class_interface_impl_102_0:
    .quad 0
.globl _class_json_pname_102_0
_class_json_pname_102_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_102
_class_json_desc_102:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_102_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_102
_class_gc_desc_102:
    .byte 1, 0, 7, 4
.globl _class_serpname_102_0
_class_serpname_102_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_102_1
_class_serpname_102_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_102_2
_class_serpname_102_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_102_3
_class_serpname_102_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_102
_class_serprop_102:
    .quad 4
    .quad _class_serpname_102_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_102_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_102_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_102_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_102
_class_vtable_102:
    .quad _method_Exception__u__u_construct
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_102
_class_static_vtable_102:
    .quad 0
.globl _class_callable_method_name_102__u__u_construct
_class_callable_method_name_102__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_102__u__u_tostring
_class_callable_method_name_102__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_102_getcode
_class_callable_method_name_102_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_102_getfile
_class_callable_method_name_102_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_102_getline
_class_callable_method_name_102_getline:
    .ascii "getline"
.globl _class_callable_method_name_102_getmessage
_class_callable_method_name_102_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_102_getprevious
_class_callable_method_name_102_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_102_gettrace
_class_callable_method_name_102_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_102_gettraceasstring
_class_callable_method_name_102_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_102
_class_callable_methods_102:
    .quad 9
    .quad _class_callable_method_name_102__u__u_construct
    .quad 11
    .quad _class_callable_method_name_102__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_102_getcode
    .quad 7
    .quad _class_callable_method_name_102_getfile
    .quad 7
    .quad _class_callable_method_name_102_getline
    .quad 7
    .quad _class_callable_method_name_102_getmessage
    .quad 10
    .quad _class_callable_method_name_102_getprevious
    .quad 11
    .quad _class_callable_method_name_102_gettrace
    .quad 8
    .quad _class_callable_method_name_102_gettraceasstring
    .quad 16
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 101

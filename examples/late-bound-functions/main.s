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
    ; @fn name=_class_propinit_6 symbol=_class_propinit_6 synthetic=1
.align 2

.globl _class_propinit_6
_class_propinit_6:
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
_eir__class_propinit_6_entry_0:
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
_class_propinit_6_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_6
    ; @fn name=_class_propinit_7 symbol=_class_propinit_7 synthetic=1
.align 2

.globl _class_propinit_7
_class_propinit_7:
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
_eir__class_propinit_7_entry_0:
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
_class_propinit_7_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_7
    ; @fn name=_class_propinit_8 symbol=_class_propinit_8 synthetic=1
.align 2

.globl _class_propinit_8
_class_propinit_8:
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
_eir__class_propinit_8_entry_0:
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
_class_propinit_8_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_8
    ; @fn name=_class_propinit_9 symbol=_class_propinit_9 synthetic=1
.align 2

.globl _class_propinit_9
_class_propinit_9:
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
_eir__class_propinit_9_entry_0:
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
_class_propinit_9_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_9
    ; @fn name=_class_propinit_11 symbol=_class_propinit_11 synthetic=1
.align 2

.globl _class_propinit_11
_class_propinit_11:
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
_eir__class_propinit_11_entry_0:
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
_class_propinit_11_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_11
    ; @fn name=_class_propinit_16 symbol=_class_propinit_16 synthetic=1
.align 2

.globl _class_propinit_16
_class_propinit_16:
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
_eir__class_propinit_16_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_16_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_16
    ; @fn name=_class_propinit_17 symbol=_class_propinit_17 synthetic=1
.align 2

.globl _class_propinit_17
_class_propinit_17:
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
_eir__class_propinit_17_entry_0:
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
_class_propinit_17_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_17
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
    ; @fn name=_class_propinit_21 symbol=_class_propinit_21 synthetic=1
.align 2

.globl _class_propinit_21
_class_propinit_21:
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
_eir__class_propinit_21_entry_0:
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
_class_propinit_21_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_21
    ; @fn name=_class_propinit_30 symbol=_class_propinit_30 synthetic=1
.align 2

.globl _class_propinit_30
_class_propinit_30:
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
_eir__class_propinit_30_entry_0:
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
_class_propinit_30_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_30
    ; @fn name=_class_propinit_34 symbol=_class_propinit_34 synthetic=1
.align 2

.globl _class_propinit_34
_class_propinit_34:
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
_eir__class_propinit_34_entry_0:
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
_class_propinit_34_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_34
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
    ; @fn name=_class_propinit_36 symbol=_class_propinit_36 synthetic=1
.align 2

.globl _class_propinit_36
_class_propinit_36:
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
_eir__class_propinit_36_entry_0:
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
_class_propinit_36_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_36
    ; @fn name=_class_propinit_38 symbol=_class_propinit_38 synthetic=1
.align 2

.globl _class_propinit_38
_class_propinit_38:
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
_eir__class_propinit_38_entry_0:
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
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_38_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_38
    ; @fn name=_class_propinit_40 symbol=_class_propinit_40 synthetic=1
.align 2

.globl _class_propinit_40
_class_propinit_40:
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
_eir__class_propinit_40_entry_0:
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
_class_propinit_40_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_40
    ; @fn name=_class_propinit_46 symbol=_class_propinit_46 synthetic=1
.align 2

.globl _class_propinit_46
_class_propinit_46:
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
_eir__class_propinit_46_entry_0:
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
_class_propinit_46_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_46
    ; @fn name=_class_propinit_47 symbol=_class_propinit_47 synthetic=1
.align 2

.globl _class_propinit_47
_class_propinit_47:
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
_eir__class_propinit_47_entry_0:
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
_class_propinit_47_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_47
    ; @fn name=_class_propinit_48 symbol=_class_propinit_48 synthetic=1
.align 2

.globl _class_propinit_48
_class_propinit_48:
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
_eir__class_propinit_48_entry_0:
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
_class_propinit_48_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_48
    ; @fn name=_class_propinit_50 symbol=_class_propinit_50 synthetic=1
.align 2

.globl _class_propinit_50
_class_propinit_50:
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
_eir__class_propinit_50_entry_0:
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
_class_propinit_50_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_50
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
    ; @fn name=_class_propinit_54 symbol=_class_propinit_54 synthetic=1
.align 2

.globl _class_propinit_54
_class_propinit_54:
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
_eir__class_propinit_54_entry_0:
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
_class_propinit_54_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_54
    ; @fn name=_class_propinit_64 symbol=_class_propinit_64 synthetic=1
.align 2

.globl _class_propinit_64
_class_propinit_64:
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
_eir__class_propinit_64_entry_0:
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
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_class_propinit_64_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_64
    ; @fn name=_class_propinit_67 symbol=_class_propinit_67 synthetic=1
.align 2

.globl _class_propinit_67
_class_propinit_67:
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
_eir__class_propinit_67_entry_0:
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
_class_propinit_67_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_67
    ; @fn name=_class_propinit_68 symbol=_class_propinit_68 synthetic=1
.align 2

.globl _class_propinit_68
_class_propinit_68:
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
_eir__class_propinit_68_entry_0:
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
_class_propinit_68_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_68
    ; @fn name=_class_propinit_69 symbol=_class_propinit_69 synthetic=1
.align 2

.globl _class_propinit_69
_class_propinit_69:
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
_eir__class_propinit_69_entry_0:
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
_class_propinit_69_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_69
    ; @fn name=_class_propinit_72 symbol=_class_propinit_72 synthetic=1
.align 2

.globl _class_propinit_72
_class_propinit_72:
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
_eir__class_propinit_72_entry_0:
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
_class_propinit_72_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_72
    ; @fn name=_class_propinit_80 symbol=_class_propinit_80 synthetic=1
.align 2

.globl _class_propinit_80
_class_propinit_80:
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
_eir__class_propinit_80_entry_0:
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
_class_propinit_80_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_80
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
_eir__class_propinit_82_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_82_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_82
    ; @fn name=_class_propinit_83 symbol=_class_propinit_83 synthetic=1
.align 2

.globl _class_propinit_83
_class_propinit_83:
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
_eir__class_propinit_83_entry_0:
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
_class_propinit_83_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_83
    ; @fn name=_class_propinit_86 symbol=_class_propinit_86 synthetic=1
.align 2

.globl _class_propinit_86
_class_propinit_86:
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
_eir__class_propinit_86_entry_0:
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
_class_propinit_86_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_86
    ; @fn name=_class_propinit_88 symbol=_class_propinit_88 synthetic=1
.align 2

.globl _class_propinit_88
_class_propinit_88:
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
_eir__class_propinit_88_entry_0:
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
_class_propinit_88_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_88
    ; @fn name=_class_propinit_90 symbol=_class_propinit_90 synthetic=1
.align 2

.globl _class_propinit_90
_class_propinit_90:
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
_eir__class_propinit_90_entry_0:
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
_class_propinit_90_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
    ; @fn name=_class_propinit_92 symbol=_class_propinit_92 synthetic=1
.align 2

.globl _class_propinit_92
_class_propinit_92:
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
_eir__class_propinit_92_entry_0:
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
_class_propinit_92_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_92
    ; @fn name=_class_propinit_94 symbol=_class_propinit_94 synthetic=1
.align 2

.globl _class_propinit_94
_class_propinit_94:
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
_eir__class_propinit_94_entry_0:
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
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_class_propinit_94_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_94
    ; @fn name=_class_propinit_96 symbol=_class_propinit_96 synthetic=1
.align 2

.globl _class_propinit_96
_class_propinit_96:
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
_eir__class_propinit_96_entry_0:
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
_class_propinit_96_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_96
    ; @fn name=_class_propinit_97 symbol=_class_propinit_97 synthetic=1
.align 2

.globl _class_propinit_97
_class_propinit_97:
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
_eir__class_propinit_97_entry_0:
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
_class_propinit_97_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_97
    ; @fn name=_class_propinit_98 symbol=_class_propinit_98 synthetic=1
.align 2

.globl _class_propinit_98
_class_propinit_98:
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
_eir__class_propinit_98_entry_0:
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
_class_propinit_98_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_98
    ; @fn name=_class_propinit_99 symbol=_class_propinit_99 synthetic=1
.align 2

.globl _class_propinit_99
_class_propinit_99:
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
_eir__class_propinit_99_entry_0:
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
_class_propinit_99_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_99
    ; @fn name=_class_propinit_100 symbol=_class_propinit_100 synthetic=1
.align 2

.globl _class_propinit_100
_class_propinit_100:
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
_eir__class_propinit_100_entry_0:
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
_class_propinit_100_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_100
    ; @fn name=ApcuBackedCache::isSupported symbol=_static_ApcuBackedCache_issupported
.align 2

.globl _static_ApcuBackedCache_issupported
_static_ApcuBackedCache_issupported:
    ; prologue
    sub sp, sp, #128
    stp x29, x30, [sp, #112]
    add x29, sp, #112
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-112]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-104]
    ; param $__elephc_called_class_id from x0
    stur x0, [x29, #-88]
    stur xzr, [x29, #-96]
    ; @block name=entry
_eir_ApcuBackedCache__isSupported_entry_0:
    ; @src line=21 col=9 end=21:15 op=concat_reset
    ldur x10, [x29, #-112]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=16 end=21:20 op=load_static_property
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    ldr x10, [x9, #8]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ApcuBackedCache__isSupported_static_prop_initialized_0
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_ApcuBackedCache__isSupported_typed_static_property_throw_1
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #110
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ApcuBackedCache__isSupported_typed_static_property_throw_1:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_4@PAGE
    add x9, x9, _str_4@PAGEOFF
    str x9, [x0, #8]
    mov x9, #96
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_ApcuBackedCache__isSupported_static_prop_initialized_0:
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-8]
    ; @src line=21 col=37 end=21:65 op=is_null
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #8
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_ApcuBackedCache__isSupported_coalesce_assign_default_1
    b _eir_ApcuBackedCache__isSupported_coalesce_assign_value_2
    ; @block name=coalesce_assign.default
_eir_ApcuBackedCache__isSupported_coalesce_assign_default_1:
    ; @src line=21 col=41 end=21:65 op=const_bool
    mov x0, #0
    mov x21, x0
    ; @src line=21 col=37 end=21:65 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    stur x0, [x29, #-32]
    ; @src line=21 col=37 end=21:65 op=acquire
    ldur x0, [x29, #-32]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-40]
    ; @src line=21 col=37 end=21:65 op=store_local
    ldur x0, [x29, #-40]
    stur x0, [x29, #-96]
    ; @src line=21 col=37 end=21:65 op=release
    ldur x0, [x29, #-32]
    bl __rt_decref_mixed
    ; @src line=21 col=37 end=21:65 op=concat_reset
    ldur x10, [x29, #-112]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=37 end=21:65 op=load_local
    ldur x0, [x29, #-96]
    stur x0, [x29, #-48]
    ; @src line=21 col=37 end=21:65 op=store_static_property
    ldur x0, [x29, #-48]
    str x0, [sp, #-16]!
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    ldr x0, [x9]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    str xzr, [x9, #8]
    b _eir_ApcuBackedCache__isSupported_coalesce_assign_merge_3
    ; @block name=coalesce_assign.value
_eir_ApcuBackedCache__isSupported_coalesce_assign_value_2:
    ; @src line=21 col=37 end=21:65 op=acquire
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-56]
    ; @src line=21 col=37 end=21:65 op=load_local
    ldur x0, [x29, #-96]
    stur x0, [x29, #-64]
    ; @src line=21 col=37 end=21:65 op=release
    ldur x0, [x29, #-64]
    bl __rt_decref_mixed
    ; @src line=21 col=37 end=21:65 op=store_local
    ldur x0, [x29, #-56]
    stur x0, [x29, #-96]
    b _eir_ApcuBackedCache__isSupported_coalesce_assign_merge_3
    ; @block name=coalesce_assign.merge
_eir_ApcuBackedCache__isSupported_coalesce_assign_merge_3:
    ; @src line=21 col=37 end=21:65 op=load_local
    ldur x0, [x29, #-96]
    stur x0, [x29, #-72]
    ; @src line=21 col=9 end=21:15 op=cast
    ldur x0, [x29, #-72]
    bl __rt_mixed_cast_int
    mov x21, x0
    ; @src line=21 col=9 end=21:15 op=release
    ldur x0, [x29, #-72]
    bl __rt_decref_mixed
    mov x0, x21
    str x0, [sp, #-16]!
    ; epilogue cleanup $__elephc_assign_expr_21_37_0
    ldur x0, [x29, #-96]
    cbz x0, _eir_ApcuBackedCache__isSupported_main_refcounted_cleanup_done_2
    bl __rt_decref_mixed
_eir_ApcuBackedCache__isSupported_main_refcounted_cleanup_done_2:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-104]
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
_static_ApcuBackedCache_issupported_epilogue:
    str x0, [sp, #-16]!
    ; epilogue cleanup $__elephc_assign_expr_21_37_0
    ldur x0, [x29, #-96]
    cbz x0, _eir_ApcuBackedCache__isSupported_main_refcounted_cleanup_done_3
    bl __rt_decref_mixed
_eir_ApcuBackedCache__isSupported_main_refcounted_cleanup_done_3:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-104]
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
    ; @endfn name=ApcuBackedCache::isSupported
    ; @fn name=ApcuBackedCache::fetch symbol=_static_ApcuBackedCache_fetch
.align 2

.globl _static_ApcuBackedCache_fetch
_static_ApcuBackedCache_fetch:
    ; prologue
    sub sp, sp, #256
    stp x29, x30, [sp, #240]
    add x29, sp, #240
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-240]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-224]
    stur x22, [x29, #-232]
    ; param $__elephc_called_class_id from x0
    stur x0, [x29, #-200]
    ; param $key from x1,x2
    stur x1, [x29, #-216]
    stur x2, [x29, #-208]
    ; @block name=entry
_eir_ApcuBackedCache__fetch_entry_0:
    ; @src line=26 col=9 end=26:11 op=concat_reset
    ldur x10, [x29, #-240]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=13 end=26:32 op=static_method_call
    ldur x0, [x29, #-200]
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    bl _static_ApcuBackedCache_issupported
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_ApcuBackedCache__fetch_if_then_2
    b _eir_ApcuBackedCache__fetch_if_merge_1
    ; @block name=if.merge
_eir_ApcuBackedCache__fetch_if_merge_1:
    ; @src line=33 col=9 end=33:15 op=concat_reset
    ldur x10, [x29, #-240]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=33 col=16 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #9
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=33 col=16 op=load_local
    ldur x1, [x29, #-216]
    ldur x2, [x29, #-208]
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=33 col=16 op=str_concat
    ldur x1, [x29, #-136]
    ldur x2, [x29, #-128]
    ldur x3, [x29, #-152]
    ldur x4, [x29, #-144]
    bl __rt_concat
    stur x1, [x29, #-168]
    stur x2, [x29, #-160]
    ; @src line=33 col=9 end=33:15 op=str_persist
    ldur x1, [x29, #-168]
    ldur x2, [x29, #-160]
    bl __rt_str_persist
    stur x1, [x29, #-184]
    stur x2, [x29, #-176]
    ldur x1, [x29, #-184]
    ldur x2, [x29, #-176]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-224]
    ldur x22, [x29, #-232]
    ldp x29, x30, [sp, #240]
    add sp, sp, #256
    ret
    ; @block name=if.then
_eir_ApcuBackedCache__fetch_if_then_2:
    ; @src line=29 col=13 end=29:24 op=concat_reset
    ldur x10, [x29, #-240]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=29 col=13 end=29:30 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #40
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    mov x0, #0
    mov x21, x0
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x22, x0
    ; @src line=29 col=13 end=29:30 op=object_new
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    mov x9, #4
    str x9, [x0]
    str xzr, [x0, #40]
    str x0, [sp, #-16]!
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    ldr x9, [sp]
    str x1, [x9, #8]
    str x2, [x9, #16]
    mov x1, x21
    ldr x9, [sp]
    str x1, [x9, #24]
    mov x0, x22
    cbz x0, _eir_ApcuBackedCache__fetch_throwable_previous_null_1
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_ApcuBackedCache__fetch_throwable_previous_null_1
    bl __rt_incref
    b _eir_ApcuBackedCache__fetch_throwable_previous_store_0
_eir_ApcuBackedCache__fetch_throwable_previous_null_1:
    mov x0, xzr
_eir_ApcuBackedCache__fetch_throwable_previous_store_0:
    ldr x9, [sp]
    str x0, [x9, #40]
    ldr x0, [sp], #16
    stur x0, [x29, #-48]
    ; @src line=29 col=13 end=29:30 op=throw_exception
    ldur x0, [x29, #-48]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    bl __rt_throw_current
    ; @src line=29 col=13 end=29:30 op=nop
    ; @src line=30 col=13 end=30:19 op=concat_reset
    ldur x10, [x29, #-240]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=20 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=30 col=20 op=load_local
    ldur x1, [x29, #-216]
    ldur x2, [x29, #-208]
    stur x1, [x29, #-88]
    stur x2, [x29, #-80]
    ; @src line=30 col=20 op=str_concat
    ldur x1, [x29, #-72]
    ldur x2, [x29, #-64]
    ldur x3, [x29, #-88]
    ldur x4, [x29, #-80]
    bl __rt_concat
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=30 col=13 end=30:19 op=str_persist
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    bl __rt_str_persist
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-224]
    ldur x22, [x29, #-232]
    ldp x29, x30, [sp, #240]
    add sp, sp, #256
    ret
    ; @block name=if.else
_eir_ApcuBackedCache__fetch_if_else_3:
    ; @src line=26 col=9 end=26:11 op=nop
    udf #0
_static_ApcuBackedCache_fetch_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-224]
    ldur x22, [x29, #-232]
    ldp x29, x30, [sp, #240]
    add sp, sp, #256
    ret
    ; @endfn name=ApcuBackedCache::fetch
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #496
    stp x29, x30, [sp, #480]
    add x29, sp, #480
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #480
    str x10, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    stur xzr, [x29, #-248]
    ; initialize static property ApcuBackedCache::$apcuSupported
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop_ApcuBackedCache_apcuSupported@PAGE
    add x9, x9, _static_prop_ApcuBackedCache_apcuSupported@PAGEOFF
    str xzr, [x9, #8]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=12 col=1 end=12:6 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=1 end=12:6 op=nop
    ; @src line=37 col=1 end=37:5 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=37 col=1 end=37:5 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=37 col=29 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=37 col=6 end=37:39 op=static_method_call
    mov x0, #85
    str x0, [sp, #-16]!
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    ldr x2, [sp, #8]
    add sp, sp, #32
    bl _static_ApcuBackedCache_fetch
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=37 col=1 end=37:5 op=echo_value
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=37 col=1 end=37:5 op=release
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=37 col=1 end=37:5 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=37 col=41 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=37 col=1 end=37:5 op=echo_value
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=42 col=1 end=42:4 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=42 col=1 end=42:4 op=try_push_handler
    ; push EIR exception handler
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #472
    str x10, [x9]
    mov x10, #0
    sub x9, x29, #464
    str x10, [x9]
    adrp x9, _rt_diag_suppression@PAGE
    add x9, x9, _rt_diag_suppression@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #456
    str x10, [x9]
    sub x10, x29, #472
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    str x10, [x9]
    sub x0, x29, #448
    bl _setjmp
    cbnz x0, _eir_main_try_catch_dispatch_1
    ; @src line=43 col=5 end=43:15 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=43 col=5 end=43:36 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #39
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    mov x0, #0
    stur x0, [x29, #-72]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    stur x0, [x29, #-80]
    ; @src line=43 col=5 end=43:36 op=object_new
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    mov x9, #4
    str x9, [x0]
    str xzr, [x0, #40]
    str x0, [sp, #-16]!
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]
    bl __rt_str_persist
    ldr x9, [sp]
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x1, [x29, #-72]
    ldr x9, [sp]
    str x1, [x9, #24]
    ldur x0, [x29, #-80]
    cbz x0, _eir_main_throwable_previous_null_1
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_throwable_previous_null_1
    bl __rt_incref
    b _eir_main_throwable_previous_store_0
_eir_main_throwable_previous_null_1:
    mov x0, xzr
_eir_main_throwable_previous_store_0:
    ldr x9, [sp]
    str x0, [x9, #40]
    ldr x0, [sp], #16
    stur x0, [x29, #-88]
    ; @src line=43 col=5 end=43:36 op=throw_exception
    ldur x0, [x29, #-88]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    bl __rt_throw_current
    ; @src line=43 col=5 end=43:36 op=nop
    ; @src line=44 col=5 end=44:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=44 col=10 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #12
    stur x1, [x29, #-112]
    stur x2, [x29, #-104]
    ; @src line=44 col=5 end=44:9 op=echo_value
    ldur x1, [x29, #-112]
    ldur x2, [x29, #-104]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=42 col=1 end=42:4 op=try_pop_handler
    ; pop EIR exception handler
    sub x9, x29, #472
    ldr x10, [x9]
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    str x10, [x9]
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _rt_diag_suppression@PAGE
    add x9, x9, _rt_diag_suppression@PAGEOFF
    str x10, [x9]
    b _eir_main_try_after_2
    ; @block name=try.catch_dispatch
_eir_main_try_catch_dispatch_1:
    ; @src line=42 col=1 end=42:4 op=try_pop_handler
    ; pop EIR exception handler
    sub x9, x29, #472
    ldr x10, [x9]
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    str x10, [x9]
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _rt_diag_suppression@PAGE
    add x9, x9, _rt_diag_suppression@PAGEOFF
    str x10, [x9]
    ; @src line=42 col=1 end=42:4 op=catch_current
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-120]
    ; @src line=42 col=1 end=42:4 op=instance_of
    ldur x0, [x29, #-120]
    mov x1, #4
    mov x2, #0
    bl __rt_exception_matches
    stur x0, [x29, #-128]
    ldur x0, [x29, #-128]
    cbnz x0, _eir_main_try_catch_body_3
    b _eir_main_try_catch_next_4
    ; @block name=try.after
_eir_main_try_after_2:
    ; @src line=49 col=1 end=49:5 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=6 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-240]
    stur x2, [x29, #-232]
    ; @src line=49 col=1 end=49:5 op=echo_value
    ldur x1, [x29, #-240]
    ldur x2, [x29, #-232]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $e
    ldur x0, [x29, #-248]
    cbz x0, _eir_main_main_refcounted_cleanup_done_2
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_2:
    ldp x29, x30, [sp, #480]
    add sp, sp, #496
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_3
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_3:
    bl __rt_ob_flush_all
    mov x0, #0
    mov x16, #1
    svc #0x80
    ; @block name=try.catch_body
_eir_main_try_catch_body_3:
    ; @src line=42 col=1 end=42:4 op=catch_bind
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-136]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str xzr, [x9]
    ; @src line=42 col=1 end=42:4 op=store_local
    ldur x0, [x29, #-136]
    stur x0, [x29, #-248]
    ; @src line=46 col=5 end=46:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=46 col=5 end=46:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=46 col=20 end=46:22 op=load_local
    ldur x0, [x29, #-248]
    stur x0, [x29, #-144]
    ; @src line=46 col=10 end=46:23 op=runtime_call
    ldur x0, [x29, #-144]
    cbz x0, _eir_main_get_class_empty_4
    ldr x9, [x0]
    adrp x10, _class_name_count@PAGE
    add x10, x10, _class_name_count@PAGEOFF
    ldr x10, [x10]
    cmp x9, x10
    b.hs _eir_main_get_class_empty_4
    adrp x11, _class_name_entries@PAGE
    add x11, x11, _class_name_entries@PAGEOFF
    lsl x12, x9, #4
    add x11, x11, x12
    ldr x1, [x11]
    ldr x2, [x11, #8]
    b _eir_main_get_class_done_5
_eir_main_get_class_empty_4:
    adrp x1, _class_name_missing@PAGE
    add x1, x1, _class_name_missing@PAGEOFF
    mov x2, #0
_eir_main_get_class_done_5:
    stur x1, [x29, #-160]
    stur x2, [x29, #-152]
    ; @src line=46 col=10 end=46:23 op=nop
    ; @src line=46 col=5 end=46:9 op=echo_value
    ldur x1, [x29, #-160]
    ldur x2, [x29, #-152]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=46 col=5 end=46:9 op=release
    ; @src line=46 col=5 end=46:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=46 col=25 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=46 col=5 end=46:9 op=echo_value
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=46 col=5 end=46:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=46 col=31 end=46:33 op=load_local
    ldur x0, [x29, #-248]
    stur x0, [x29, #-184]
    ; @src line=46 col=33 end=46:47 op=method_call
    ldur x9, [x29, #-184]
    cbz x9, _eir_main_static_method_receiver_null_6
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_static_method_receiver_null_6
    b _eir_main_static_method_receiver_checked_7
_eir_main_static_method_receiver_null_6:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_8
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #76
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_static_exception_throw_8:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_15@PAGE
    add x9, x9, _str_15@PAGEOFF
    str x9, [x0, #8]
    mov x9, #46
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_static_method_receiver_checked_7:
    ldur x9, [x29, #-184]
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    bl __rt_str_persist
    stur x1, [x29, #-200]
    stur x2, [x29, #-192]
    ; @src line=46 col=33 end=46:47 op=nop
    ; @src line=46 col=5 end=46:9 op=echo_value
    ldur x1, [x29, #-200]
    ldur x2, [x29, #-192]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=46 col=5 end=46:9 op=release
    ldur x1, [x29, #-200]
    ldur x2, [x29, #-192]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=46 col=5 end=46:9 op=concat_reset
    sub x9, x29, #480
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=46 col=49 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-216]
    stur x2, [x29, #-208]
    ; @src line=46 col=5 end=46:9 op=echo_value
    ldur x1, [x29, #-216]
    ldur x2, [x29, #-208]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    b _eir_main_try_after_2
    ; @block name=try.catch_next
_eir_main_try_catch_next_4:
    ; @src line=42 col=1 end=42:4 op=catch_current
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-224]
    ldur x0, [x29, #-224]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    bl __rt_throw_current
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
.globl _str_2
_str_2:
    .ascii "UTC"
.globl _str_3
_str_3:
    .ascii "Fatal error: Typed static property ApcuBackedCache::$apcuSupported must not be accessed before initialization\n"
.globl _str_4
_str_4:
    .ascii "Typed static property ApcuBackedCache::$apcuSupported must not be accessed before initialization"
.globl _str_5
_str_5:
    .ascii "fallback:"
.globl _str_6
_str_6:
    .ascii "Call to undefined function apcu_exists()"
.globl _str_7
_str_7:
    .ascii "apcu:"
.globl _str_8
_str_8:
    .ascii "user:42"
.globl _str_9
_str_9:
    .ascii "\n"
.globl _str_10
_str_10:
    .ascii "Call to undefined function apcu_store()"
.globl _str_11
_str_11:
    .ascii "unreachable\n"
.globl _str_12
_str_12:
    .ascii "done\n"
.globl _str_13
_str_13:
    .ascii ": "
.globl _str_14
_str_14:
    .ascii "Fatal error: Uncaught Error: Call to a member function getMessage() on null\n"
.globl _str_15
_str_15:
    .ascii "Call to a member function getMessage() on null"
.p2align 3
.globl _float_1
_float_1:
    .quad 0x0000000000000000

.comm _static_prop_ApcuBackedCache_apcuSupported, 16, 3
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
    .quad 34
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_1
    .quad 9
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 10
    .quad 1
    .quad 0
    .quad _instanceof_name_class_2
    .quad 16
    .quad 2
    .quad 0
    .quad _instanceof_name_class_abs_2
    .quad 17
    .quad 2
    .quad 0
    .quad _instanceof_name_class_3
    .quad 13
    .quad 3
    .quad 0
    .quad _instanceof_name_class_abs_3
    .quad 14
    .quad 3
    .quad 0
    .quad _instanceof_name_class_4
    .quad 5
    .quad 4
    .quad 0
    .quad _instanceof_name_class_abs_4
    .quad 6
    .quad 4
    .quad 0
    .quad _instanceof_name_class_17
    .quad 15
    .quad 17
    .quad 0
    .quad _instanceof_name_class_abs_17
    .quad 16
    .quad 17
    .quad 0
    .quad _instanceof_name_class_20
    .quad 20
    .quad 20
    .quad 0
    .quad _instanceof_name_class_abs_20
    .quad 21
    .quad 20
    .quad 0
    .quad _instanceof_name_class_34
    .quad 9
    .quad 34
    .quad 0
    .quad _instanceof_name_class_abs_34
    .quad 10
    .quad 34
    .quad 0
    .quad _instanceof_name_class_35
    .quad 14
    .quad 35
    .quad 0
    .quad _instanceof_name_class_abs_35
    .quad 15
    .quad 35
    .quad 0
    .quad _instanceof_name_class_43
    .quad 8
    .quad 43
    .quad 0
    .quad _instanceof_name_class_abs_43
    .quad 9
    .quad 43
    .quad 0
    .quad _instanceof_name_class_50
    .quad 19
    .quad 50
    .quad 0
    .quad _instanceof_name_class_abs_50
    .quad 20
    .quad 50
    .quad 0
    .quad _instanceof_name_class_80
    .quad 19
    .quad 80
    .quad 0
    .quad _instanceof_name_class_abs_80
    .quad 20
    .quad 80
    .quad 0
    .quad _instanceof_name_class_81
    .quad 24
    .quad 81
    .quad 0
    .quad _instanceof_name_class_abs_81
    .quad 25
    .quad 81
    .quad 0
    .quad _instanceof_name_class_85
    .quad 15
    .quad 85
    .quad 0
    .quad _instanceof_name_class_abs_85
    .quad 16
    .quad 85
    .quad 0
    .quad _instanceof_name_class_86
    .quad 10
    .quad 86
    .quad 0
    .quad _instanceof_name_class_abs_86
    .quad 11
    .quad 86
    .quad 0
    .quad _instanceof_name_class_90
    .quad 19
    .quad 90
    .quad 0
    .quad _instanceof_name_class_abs_90
    .quad 20
    .quad 90
    .quad 0
    .quad _instanceof_name_interface_1
    .quad 9
    .quad 1
    .quad 1
    .quad _instanceof_name_interface_abs_1
    .quad 10
    .quad 1
    .quad 1
    .quad _instanceof_name_interface_5
    .quad 10
    .quad 5
    .quad 1
    .quad _instanceof_name_interface_abs_5
    .quad 11
    .quad 5
    .quad 1
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "Exception"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\Exception"
.globl _instanceof_name_class_2
_instanceof_name_class_2:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_2
_instanceof_name_class_abs_2:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_3
_instanceof_name_class_3:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_3
_instanceof_name_class_abs_3:
    .ascii "\\JsonException"
.globl _instanceof_name_class_4
_instanceof_name_class_4:
    .ascii "Error"
.globl _instanceof_name_class_abs_4
_instanceof_name_class_abs_4:
    .ascii "\\Error"
.globl _instanceof_name_class_17
_instanceof_name_class_17:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_17
_instanceof_name_class_abs_17:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_20
_instanceof_name_class_20:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_20
_instanceof_name_class_abs_20:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_34
_instanceof_name_class_34:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_34
_instanceof_name_class_abs_34:
    .ascii "\\TypeError"
.globl _instanceof_name_class_35
_instanceof_name_class_35:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_35
_instanceof_name_class_abs_35:
    .ascii "\\LogicException"
.globl _instanceof_name_class_43
_instanceof_name_class_43:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_43
_instanceof_name_class_abs_43:
    .ascii "\\stdClass"
.globl _instanceof_name_class_50
_instanceof_name_class_50:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_50
_instanceof_name_class_abs_50:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_80
_instanceof_name_class_80:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_80
_instanceof_name_class_abs_80:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_81
_instanceof_name_class_81:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_81
_instanceof_name_class_abs_81:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_85
_instanceof_name_class_85:
    .ascii "ApcuBackedCache"
.globl _instanceof_name_class_abs_85
_instanceof_name_class_abs_85:
    .ascii "\\ApcuBackedCache"
.globl _instanceof_name_class_86
_instanceof_name_class_86:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_86
_instanceof_name_class_abs_86:
    .ascii "\\ValueError"
.globl _instanceof_name_class_90
_instanceof_name_class_90:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_90
_instanceof_name_class_abs_90:
    .ascii "\\ReflectionException"
.globl _instanceof_name_interface_1
_instanceof_name_interface_1:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_1
_instanceof_name_interface_abs_1:
    .ascii "\\Throwable"
.globl _instanceof_name_interface_5
_instanceof_name_interface_5:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_5
_instanceof_name_interface_abs_5:
    .ascii "\\Stringable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 91
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_missing
    .quad 0
    .quad _class_name_1
    .quad 9
    .quad _class_name_2
    .quad 16
    .quad _class_name_3
    .quad 13
    .quad _class_name_4
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
    .quad _class_name_17
    .quad 15
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_20
    .quad 20
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
    .quad _class_name_34
    .quad 9
    .quad _class_name_35
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
    .quad _class_name_43
    .quad 8
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
    .quad _class_name_50
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
    .quad _class_name_80
    .quad 19
    .quad _class_name_81
    .quad 24
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_85
    .quad 15
    .quad _class_name_86
    .quad 10
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_90
    .quad 19
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_1
_class_name_1:
    .ascii "Exception"
.globl _class_name_2
_class_name_2:
    .ascii "RuntimeException"
.globl _class_name_3
_class_name_3:
    .ascii "JsonException"
.globl _class_name_4
_class_name_4:
    .ascii "Error"
.globl _class_name_17
_class_name_17:
    .ascii "ArithmeticError"
.globl _class_name_20
_class_name_20:
    .ascii "OutOfBoundsException"
.globl _class_name_34
_class_name_34:
    .ascii "TypeError"
.globl _class_name_35
_class_name_35:
    .ascii "LogicException"
.globl _class_name_43
_class_name_43:
    .ascii "stdClass"
.globl _class_name_50
_class_name_50:
    .ascii "UnhandledMatchError"
.globl _class_name_80
_class_name_80:
    .ascii "OutOfRangeException"
.globl _class_name_81
_class_name_81:
    .ascii "InvalidArgumentException"
.globl _class_name_85
_class_name_85:
    .ascii "ApcuBackedCache"
.globl _class_name_86
_class_name_86:
    .ascii "ValueError"
.globl _class_name_90
_class_name_90:
    .ascii "ReflectionException"
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
    .quad 75
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 11
.globl _generator_class_id
_generator_class_id:
    .quad 10
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 65
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 84
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 66
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 55
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 4
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 35
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 2
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 80
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 20
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 81
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 34
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 86
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 90
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 17
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_1
    .quad _interface_methods_5
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_1
    .quad _class_interfaces_2
    .quad _class_interfaces_3
    .quad _class_interfaces_4
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
    .quad _class_interfaces_17
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_20
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
    .quad _class_interfaces_34
    .quad _class_interfaces_35
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_43
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_50
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_80
    .quad _class_interfaces_81
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_85
    .quad _class_interfaces_86
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_90
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_missing
    .quad _class_json_desc_1
    .quad _class_json_desc_2
    .quad _class_json_desc_3
    .quad _class_json_desc_4
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
    .quad _class_json_desc_17
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_20
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
    .quad _class_json_desc_34
    .quad _class_json_desc_35
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_43
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_50
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_80
    .quad _class_json_desc_81
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_85
    .quad _class_json_desc_86
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_90
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 3
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
    .quad -1
    .quad 1
    .quad 2
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
    .quad 4
    .quad -1
    .quad -1
    .quad 2
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
    .quad 4
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
    .quad 4
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
    .quad 35
    .quad 35
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 4
    .quad -1
    .quad -1
    .quad -1
    .quad 1
.globl _class_object_payload_sizes
_class_object_payload_sizes:
    .quad 0
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
    .quad 72
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
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 16
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 8
    .quad 72
    .quad 0
    .quad 0
    .quad 0
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
    .quad 1
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 91
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_1
    .quad _class_gc_desc_2
    .quad _class_gc_desc_3
    .quad _class_gc_desc_4
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
    .quad _class_gc_desc_17
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_20
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
    .quad _class_gc_desc_34
    .quad _class_gc_desc_35
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_43
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_50
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_80
    .quad _class_gc_desc_81
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_85
    .quad _class_gc_desc_86
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_90
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_missing
    .quad _class_vtable_1
    .quad _class_vtable_2
    .quad _class_vtable_3
    .quad _class_vtable_4
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
    .quad _class_vtable_17
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_20
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
    .quad _class_vtable_34
    .quad _class_vtable_35
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_43
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_50
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
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_80
    .quad _class_vtable_81
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_85
    .quad _class_vtable_86
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_90
.globl _class_destruct_count
_class_destruct_count:
    .quad 91
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
.globl _class_clone_count
_class_clone_count:
    .quad 91
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
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad 0
    .quad _class_propinit_1
    .quad _class_propinit_2
    .quad _class_propinit_3
    .quad _class_propinit_4
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_17
    .quad 0
    .quad 0
    .quad _class_propinit_20
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_34
    .quad _class_propinit_35
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_50
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_80
    .quad _class_propinit_81
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_86
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_90
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_missing
    .quad _class_serprop_1
    .quad _class_serprop_2
    .quad _class_serprop_3
    .quad _class_serprop_4
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
    .quad _class_serprop_17
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_20
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
    .quad _class_serprop_34
    .quad _class_serprop_35
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_43
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_50
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
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_80
    .quad _class_serprop_81
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_85
    .quad _class_serprop_86
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_90
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_1
    .quad _class_static_vtable_2
    .quad _class_static_vtable_3
    .quad _class_static_vtable_4
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
    .quad _class_static_vtable_17
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_20
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
    .quad _class_static_vtable_34
    .quad _class_static_vtable_35
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_43
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_50
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_80
    .quad _class_static_vtable_81
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_85
    .quad _class_static_vtable_86
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_90
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_1
    .quad _class_callable_methods_2
    .quad _class_callable_methods_3
    .quad _class_callable_methods_4
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
    .quad _class_callable_methods_17
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_20
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
    .quad _class_callable_methods_34
    .quad _class_callable_methods_35
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_43
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_50
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_80
    .quad _class_callable_methods_81
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_85
    .quad _class_callable_methods_86
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_90
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
.globl _class_callable_static_class_name_85
_class_callable_static_class_name_85:
    .ascii "ApcuBackedCache"
.globl _class_callable_static_method_name_85_fetch
_class_callable_static_method_name_85_fetch:
    .ascii "fetch"
.p2align 3
.globl _class_callable_static_method_count
_class_callable_static_method_count:
    .quad 1
.globl _class_callable_static_method_table
_class_callable_static_method_table:
    .quad _class_callable_static_class_name_85
    .quad 15
    .quad _class_callable_static_method_name_85_fetch
    .quad 5
.p2align 3
.globl _class_by_name_str_1
_class_by_name_str_1:
    .ascii "Exception"
.globl _class_by_name_str_2
_class_by_name_str_2:
    .ascii "RuntimeException"
.globl _class_by_name_str_3
_class_by_name_str_3:
    .ascii "JsonException"
.globl _class_by_name_str_4
_class_by_name_str_4:
    .ascii "Error"
.globl _class_by_name_str_17
_class_by_name_str_17:
    .ascii "ArithmeticError"
.globl _class_by_name_str_20
_class_by_name_str_20:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_34
_class_by_name_str_34:
    .ascii "TypeError"
.globl _class_by_name_str_35
_class_by_name_str_35:
    .ascii "LogicException"
.globl _class_by_name_str_43
_class_by_name_str_43:
    .ascii "stdClass"
.globl _class_by_name_str_50
_class_by_name_str_50:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_80
_class_by_name_str_80:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_81
_class_by_name_str_81:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_85
_class_by_name_str_85:
    .ascii "ApcuBackedCache"
.globl _class_by_name_str_86
_class_by_name_str_86:
    .ascii "ValueError"
.globl _class_by_name_str_90
_class_by_name_str_90:
    .ascii "ReflectionException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 15
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_1
    .quad 9
    .quad 1
    .quad 72
    .quad _class_by_name_str_2
    .quad 16
    .quad 2
    .quad 72
    .quad _class_by_name_str_3
    .quad 13
    .quad 3
    .quad 72
    .quad _class_by_name_str_4
    .quad 5
    .quad 4
    .quad 72
    .quad _class_by_name_str_17
    .quad 15
    .quad 17
    .quad 72
    .quad _class_by_name_str_20
    .quad 20
    .quad 20
    .quad 72
    .quad _class_by_name_str_34
    .quad 9
    .quad 34
    .quad 72
    .quad _class_by_name_str_35
    .quad 14
    .quad 35
    .quad 72
    .quad _class_by_name_str_43
    .quad 8
    .quad 43
    .quad 16
    .quad _class_by_name_str_50
    .quad 19
    .quad 50
    .quad 72
    .quad _class_by_name_str_80
    .quad 19
    .quad 80
    .quad 72
    .quad _class_by_name_str_81
    .quad 24
    .quad 81
    .quad 72
    .quad _class_by_name_str_85
    .quad 15
    .quad 85
    .quad 8
    .quad _class_by_name_str_86
    .quad 10
    .quad 86
    .quad 72
    .quad _class_by_name_str_90
    .quad 19
    .quad 90
    .quad 72
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 91
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
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_1
_interface_methods_1:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _interface_methods_5
_interface_methods_5:
    .quad 1
    .quad 0
.globl _class_interfaces_1
_class_interfaces_1:
    .quad 2
    .quad 1
    .quad _class_interface_impl_1_1
    .quad 5
    .quad _class_interface_impl_1_5
.globl _class_interface_impl_1_1
_class_interface_impl_1_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_5
_class_interface_impl_1_5:
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
    .quad 1
    .quad _class_interface_impl_2_1
    .quad 5
    .quad _class_interface_impl_2_5
.globl _class_interface_impl_2_1
_class_interface_impl_2_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_2_5
_class_interface_impl_2_5:
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
    .quad 1
    .quad _class_interface_impl_3_1
    .quad 5
    .quad _class_interface_impl_3_5
.globl _class_interface_impl_3_1
_class_interface_impl_3_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_3_5
_class_interface_impl_3_5:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_3
_class_vtable_3:
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
    .quad 1
    .quad _class_interface_impl_4_1
    .quad 5
    .quad _class_interface_impl_4_5
.globl _class_interface_impl_4_1
_class_interface_impl_4_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_4_5
_class_interface_impl_4_5:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_4_0
_class_serpname_4_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_4_1
_class_serpname_4_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_4_2
_class_serpname_4_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_4_3
_class_serpname_4_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_4
_class_serprop_4:
    .quad 4
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
    .quad _class_serpname_4_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_4
_class_vtable_4:
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
.globl _class_interfaces_17
_class_interfaces_17:
    .quad 2
    .quad 1
    .quad _class_interface_impl_17_1
    .quad 5
    .quad _class_interface_impl_17_5
.globl _class_interface_impl_17_1
_class_interface_impl_17_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_17_5
_class_interface_impl_17_5:
    .quad 0
.globl _class_json_pname_17_0
_class_json_pname_17_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_17
_class_json_desc_17:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_17_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_17
_class_gc_desc_17:
    .byte 1, 0, 7, 4
.globl _class_serpname_17_0
_class_serpname_17_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_17_1
_class_serpname_17_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_17_2
_class_serpname_17_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_17_3
_class_serpname_17_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_17
_class_serprop_17:
    .quad 4
    .quad _class_serpname_17_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_17_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_17_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_17_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_17
_class_vtable_17:
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
.globl _class_static_vtable_17
_class_static_vtable_17:
    .quad 0
.globl _class_callable_method_name_17__u__u_construct
_class_callable_method_name_17__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_17__u__u_tostring
_class_callable_method_name_17__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_17_getcode
_class_callable_method_name_17_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_17_getfile
_class_callable_method_name_17_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_17_getline
_class_callable_method_name_17_getline:
    .ascii "getline"
.globl _class_callable_method_name_17_getmessage
_class_callable_method_name_17_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_17_getprevious
_class_callable_method_name_17_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_17_gettrace
_class_callable_method_name_17_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_17_gettraceasstring
_class_callable_method_name_17_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_17
_class_callable_methods_17:
    .quad 9
    .quad _class_callable_method_name_17__u__u_construct
    .quad 11
    .quad _class_callable_method_name_17__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_17_getcode
    .quad 7
    .quad _class_callable_method_name_17_getfile
    .quad 7
    .quad _class_callable_method_name_17_getline
    .quad 7
    .quad _class_callable_method_name_17_getmessage
    .quad 10
    .quad _class_callable_method_name_17_getprevious
    .quad 11
    .quad _class_callable_method_name_17_gettrace
    .quad 8
    .quad _class_callable_method_name_17_gettraceasstring
    .quad 16
.globl _class_interfaces_20
_class_interfaces_20:
    .quad 2
    .quad 1
    .quad _class_interface_impl_20_1
    .quad 5
    .quad _class_interface_impl_20_5
.globl _class_interface_impl_20_1
_class_interface_impl_20_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_20_5
_class_interface_impl_20_5:
    .quad 0
.globl _class_json_pname_20_0
_class_json_pname_20_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_20
_class_json_desc_20:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_20_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_20
_class_gc_desc_20:
    .byte 1, 0, 7, 4
.globl _class_serpname_20_0
_class_serpname_20_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_20_1
_class_serpname_20_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_20_2
_class_serpname_20_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_20_3
_class_serpname_20_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_20
_class_serprop_20:
    .quad 4
    .quad _class_serpname_20_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_20_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_20_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_20_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_20
_class_vtable_20:
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
.globl _class_static_vtable_20
_class_static_vtable_20:
    .quad 0
.globl _class_callable_method_name_20__u__u_construct
_class_callable_method_name_20__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_20__u__u_tostring
_class_callable_method_name_20__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_20_getcode
_class_callable_method_name_20_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_20_getfile
_class_callable_method_name_20_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_20_getline
_class_callable_method_name_20_getline:
    .ascii "getline"
.globl _class_callable_method_name_20_getmessage
_class_callable_method_name_20_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_20_getprevious
_class_callable_method_name_20_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_20_gettrace
_class_callable_method_name_20_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_20_gettraceasstring
_class_callable_method_name_20_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_20
_class_callable_methods_20:
    .quad 9
    .quad _class_callable_method_name_20__u__u_construct
    .quad 11
    .quad _class_callable_method_name_20__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_20_getcode
    .quad 7
    .quad _class_callable_method_name_20_getfile
    .quad 7
    .quad _class_callable_method_name_20_getline
    .quad 7
    .quad _class_callable_method_name_20_getmessage
    .quad 10
    .quad _class_callable_method_name_20_getprevious
    .quad 11
    .quad _class_callable_method_name_20_gettrace
    .quad 8
    .quad _class_callable_method_name_20_gettraceasstring
    .quad 16
.globl _class_interfaces_34
_class_interfaces_34:
    .quad 2
    .quad 1
    .quad _class_interface_impl_34_1
    .quad 5
    .quad _class_interface_impl_34_5
.globl _class_interface_impl_34_1
_class_interface_impl_34_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_34_5
_class_interface_impl_34_5:
    .quad 0
.globl _class_json_pname_34_0
_class_json_pname_34_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_34
_class_json_desc_34:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_34_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_34
_class_gc_desc_34:
    .byte 1, 0, 7, 4
.globl _class_serpname_34_0
_class_serpname_34_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_34_1
_class_serpname_34_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_34_2
_class_serpname_34_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_34_3
_class_serpname_34_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_34
_class_serprop_34:
    .quad 4
    .quad _class_serpname_34_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_34_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_34_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_34_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_34
_class_vtable_34:
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
.globl _class_static_vtable_34
_class_static_vtable_34:
    .quad 0
.globl _class_callable_method_name_34__u__u_construct
_class_callable_method_name_34__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_34__u__u_tostring
_class_callable_method_name_34__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_34_getcode
_class_callable_method_name_34_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_34_getfile
_class_callable_method_name_34_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_34_getline
_class_callable_method_name_34_getline:
    .ascii "getline"
.globl _class_callable_method_name_34_getmessage
_class_callable_method_name_34_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_34_getprevious
_class_callable_method_name_34_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_34_gettrace
_class_callable_method_name_34_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_34_gettraceasstring
_class_callable_method_name_34_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_34
_class_callable_methods_34:
    .quad 9
    .quad _class_callable_method_name_34__u__u_construct
    .quad 11
    .quad _class_callable_method_name_34__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_34_getcode
    .quad 7
    .quad _class_callable_method_name_34_getfile
    .quad 7
    .quad _class_callable_method_name_34_getline
    .quad 7
    .quad _class_callable_method_name_34_getmessage
    .quad 10
    .quad _class_callable_method_name_34_getprevious
    .quad 11
    .quad _class_callable_method_name_34_gettrace
    .quad 8
    .quad _class_callable_method_name_34_gettraceasstring
    .quad 16
.globl _class_interfaces_35
_class_interfaces_35:
    .quad 2
    .quad 1
    .quad _class_interface_impl_35_1
    .quad 5
    .quad _class_interface_impl_35_5
.globl _class_interface_impl_35_1
_class_interface_impl_35_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_35_5
_class_interface_impl_35_5:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_35
_class_vtable_35:
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
.globl _class_interfaces_43
_class_interfaces_43:
    .quad 0
    .p2align 3
.globl _class_json_desc_43
_class_json_desc_43:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_43
_class_gc_desc_43:
    .byte 0
    .p2align 3
.globl _class_serprop_43
_class_serprop_43:
    .quad 0
    .p2align 3
.globl _class_vtable_43
_class_vtable_43:
    .quad 0
    .p2align 3
.globl _class_static_vtable_43
_class_static_vtable_43:
    .quad 0
.p2align 3
.globl _class_callable_methods_43
_class_callable_methods_43:
    .quad 0
.globl _class_interfaces_50
_class_interfaces_50:
    .quad 2
    .quad 1
    .quad _class_interface_impl_50_1
    .quad 5
    .quad _class_interface_impl_50_5
.globl _class_interface_impl_50_1
_class_interface_impl_50_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_50_5
_class_interface_impl_50_5:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_50_0
_class_serpname_50_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_50_1
_class_serpname_50_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_50_2
_class_serpname_50_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_50_3
_class_serpname_50_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_50
_class_serprop_50:
    .quad 4
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
    .quad _class_serpname_50_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_50
_class_vtable_50:
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
.globl _class_interfaces_80
_class_interfaces_80:
    .quad 2
    .quad 1
    .quad _class_interface_impl_80_1
    .quad 5
    .quad _class_interface_impl_80_5
.globl _class_interface_impl_80_1
_class_interface_impl_80_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_80_5
_class_interface_impl_80_5:
    .quad 0
.globl _class_json_pname_80_0
_class_json_pname_80_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_80
_class_json_desc_80:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_80_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_80
_class_gc_desc_80:
    .byte 1, 0, 7, 4
.globl _class_serpname_80_0
_class_serpname_80_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_80_1
_class_serpname_80_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_80_2
_class_serpname_80_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_80_3
_class_serpname_80_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_80
_class_serprop_80:
    .quad 4
    .quad _class_serpname_80_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_80_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_80_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_80_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_80
_class_vtable_80:
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
.globl _class_static_vtable_80
_class_static_vtable_80:
    .quad 0
.globl _class_callable_method_name_80__u__u_construct
_class_callable_method_name_80__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_80__u__u_tostring
_class_callable_method_name_80__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_80_getcode
_class_callable_method_name_80_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_80_getfile
_class_callable_method_name_80_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_80_getline
_class_callable_method_name_80_getline:
    .ascii "getline"
.globl _class_callable_method_name_80_getmessage
_class_callable_method_name_80_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_80_getprevious
_class_callable_method_name_80_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_80_gettrace
_class_callable_method_name_80_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_80_gettraceasstring
_class_callable_method_name_80_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_80
_class_callable_methods_80:
    .quad 9
    .quad _class_callable_method_name_80__u__u_construct
    .quad 11
    .quad _class_callable_method_name_80__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_80_getcode
    .quad 7
    .quad _class_callable_method_name_80_getfile
    .quad 7
    .quad _class_callable_method_name_80_getline
    .quad 7
    .quad _class_callable_method_name_80_getmessage
    .quad 10
    .quad _class_callable_method_name_80_getprevious
    .quad 11
    .quad _class_callable_method_name_80_gettrace
    .quad 8
    .quad _class_callable_method_name_80_gettraceasstring
    .quad 16
.globl _class_interfaces_81
_class_interfaces_81:
    .quad 2
    .quad 1
    .quad _class_interface_impl_81_1
    .quad 5
    .quad _class_interface_impl_81_5
.globl _class_interface_impl_81_1
_class_interface_impl_81_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_81_5
_class_interface_impl_81_5:
    .quad 0
.globl _class_json_pname_81_0
_class_json_pname_81_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_81
_class_json_desc_81:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_81_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_81
_class_gc_desc_81:
    .byte 1, 0, 7, 4
.globl _class_serpname_81_0
_class_serpname_81_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_81_1
_class_serpname_81_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_81_2
_class_serpname_81_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_81_3
_class_serpname_81_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_81
_class_serprop_81:
    .quad 4
    .quad _class_serpname_81_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_81_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_81_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_81_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_81
_class_vtable_81:
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
.globl _class_static_vtable_81
_class_static_vtable_81:
    .quad 0
.globl _class_callable_method_name_81__u__u_construct
_class_callable_method_name_81__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_81__u__u_tostring
_class_callable_method_name_81__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_81_getcode
_class_callable_method_name_81_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_81_getfile
_class_callable_method_name_81_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_81_getline
_class_callable_method_name_81_getline:
    .ascii "getline"
.globl _class_callable_method_name_81_getmessage
_class_callable_method_name_81_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_81_getprevious
_class_callable_method_name_81_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_81_gettrace
_class_callable_method_name_81_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_81_gettraceasstring
_class_callable_method_name_81_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_81
_class_callable_methods_81:
    .quad 9
    .quad _class_callable_method_name_81__u__u_construct
    .quad 11
    .quad _class_callable_method_name_81__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_81_getcode
    .quad 7
    .quad _class_callable_method_name_81_getfile
    .quad 7
    .quad _class_callable_method_name_81_getline
    .quad 7
    .quad _class_callable_method_name_81_getmessage
    .quad 10
    .quad _class_callable_method_name_81_getprevious
    .quad 11
    .quad _class_callable_method_name_81_gettrace
    .quad 8
    .quad _class_callable_method_name_81_gettraceasstring
    .quad 16
.globl _class_interfaces_85
_class_interfaces_85:
    .quad 0
    .p2align 3
.globl _class_json_desc_85
_class_json_desc_85:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_85
_class_gc_desc_85:
    .byte 0
    .p2align 3
.globl _class_serprop_85
_class_serprop_85:
    .quad 0
    .p2align 3
.globl _class_vtable_85
_class_vtable_85:
    .quad 0
    .p2align 3
.globl _class_static_vtable_85
_class_static_vtable_85:
    .quad _static_ApcuBackedCache_fetch
.p2align 3
.globl _class_callable_methods_85
_class_callable_methods_85:
    .quad 0
.globl _class_interfaces_86
_class_interfaces_86:
    .quad 2
    .quad 1
    .quad _class_interface_impl_86_1
    .quad 5
    .quad _class_interface_impl_86_5
.globl _class_interface_impl_86_1
_class_interface_impl_86_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_86_5
_class_interface_impl_86_5:
    .quad 0
.globl _class_json_pname_86_0
_class_json_pname_86_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_86
_class_json_desc_86:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_86_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_86
_class_gc_desc_86:
    .byte 1, 0, 7, 4
.globl _class_serpname_86_0
_class_serpname_86_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_86_1
_class_serpname_86_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_86_2
_class_serpname_86_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_86_3
_class_serpname_86_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_86
_class_serprop_86:
    .quad 4
    .quad _class_serpname_86_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_86_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_86_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_86_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_86
_class_vtable_86:
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
.globl _class_static_vtable_86
_class_static_vtable_86:
    .quad 0
.globl _class_callable_method_name_86__u__u_construct
_class_callable_method_name_86__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_86__u__u_tostring
_class_callable_method_name_86__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_86_getcode
_class_callable_method_name_86_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_86_getfile
_class_callable_method_name_86_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_86_getline
_class_callable_method_name_86_getline:
    .ascii "getline"
.globl _class_callable_method_name_86_getmessage
_class_callable_method_name_86_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_86_getprevious
_class_callable_method_name_86_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_86_gettrace
_class_callable_method_name_86_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_86_gettraceasstring
_class_callable_method_name_86_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_86
_class_callable_methods_86:
    .quad 9
    .quad _class_callable_method_name_86__u__u_construct
    .quad 11
    .quad _class_callable_method_name_86__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_86_getcode
    .quad 7
    .quad _class_callable_method_name_86_getfile
    .quad 7
    .quad _class_callable_method_name_86_getline
    .quad 7
    .quad _class_callable_method_name_86_getmessage
    .quad 10
    .quad _class_callable_method_name_86_getprevious
    .quad 11
    .quad _class_callable_method_name_86_gettrace
    .quad 8
    .quad _class_callable_method_name_86_gettraceasstring
    .quad 16
.globl _class_interfaces_90
_class_interfaces_90:
    .quad 2
    .quad 1
    .quad _class_interface_impl_90_1
    .quad 5
    .quad _class_interface_impl_90_5
.globl _class_interface_impl_90_1
_class_interface_impl_90_1:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_90_5
_class_interface_impl_90_5:
    .quad 0
.globl _class_json_pname_90_0
_class_json_pname_90_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_90
_class_json_desc_90:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_90_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_90
_class_gc_desc_90:
    .byte 1, 0, 7, 4
.globl _class_serpname_90_0
_class_serpname_90_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_90_1
_class_serpname_90_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_90_2
_class_serpname_90_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_90_3
_class_serpname_90_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_90
_class_serprop_90:
    .quad 4
    .quad _class_serpname_90_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_90_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_90_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_90_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_90
_class_vtable_90:
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
.globl _class_static_vtable_90
_class_static_vtable_90:
    .quad 0
.globl _class_callable_method_name_90__u__u_construct
_class_callable_method_name_90__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_90__u__u_tostring
_class_callable_method_name_90__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_90_getcode
_class_callable_method_name_90_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_90_getfile
_class_callable_method_name_90_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_90_getline
_class_callable_method_name_90_getline:
    .ascii "getline"
.globl _class_callable_method_name_90_getmessage
_class_callable_method_name_90_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_90_getprevious
_class_callable_method_name_90_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_90_gettrace
_class_callable_method_name_90_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_90_gettraceasstring
_class_callable_method_name_90_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_90
_class_callable_methods_90:
    .quad 9
    .quad _class_callable_method_name_90__u__u_construct
    .quad 11
    .quad _class_callable_method_name_90__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_90_getcode
    .quad 7
    .quad _class_callable_method_name_90_getfile
    .quad 7
    .quad _class_callable_method_name_90_getline
    .quad 7
    .quad _class_callable_method_name_90_getmessage
    .quad 10
    .quad _class_callable_method_name_90_getprevious
    .quad 11
    .quad _class_callable_method_name_90_gettrace
    .quad 8
    .quad _class_callable_method_name_90_gettraceasstring
    .quad 16
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 43

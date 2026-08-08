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
    ; @fn name=_class_propinit_10 symbol=_class_propinit_10 synthetic=1
.align 2

.globl _class_propinit_10
_class_propinit_10:
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
_eir__class_propinit_10_entry_0:
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
_class_propinit_10_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_10
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
    ; @fn name=_class_propinit_12 symbol=_class_propinit_12 synthetic=1
.align 2

.globl _class_propinit_12
_class_propinit_12:
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
_eir__class_propinit_12_entry_0:
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
_class_propinit_12_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_12
    ; @fn name=_class_propinit_15 symbol=_class_propinit_15 synthetic=1
.align 2

.globl _class_propinit_15
_class_propinit_15:
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
_eir__class_propinit_15_entry_0:
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
_class_propinit_15_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_16_entry_0:
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
_class_propinit_16_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_16
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
_eir__class_propinit_21_entry_0:
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
_class_propinit_21_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_21
    ; @fn name=_class_propinit_22 symbol=_class_propinit_22 synthetic=1
.align 2

.globl _class_propinit_22
_class_propinit_22:
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
_eir__class_propinit_22_entry_0:
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
_class_propinit_22_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_22
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
    ; @fn name=_class_propinit_32 symbol=_class_propinit_32 synthetic=1
.align 2

.globl _class_propinit_32
_class_propinit_32:
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
_eir__class_propinit_32_entry_0:
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
_class_propinit_32_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_32
    ; @fn name=_class_propinit_33 symbol=_class_propinit_33 synthetic=1
.align 2

.globl _class_propinit_33
_class_propinit_33:
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
_eir__class_propinit_33_entry_0:
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
_class_propinit_33_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_33
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
    ; @fn name=_class_propinit_37 symbol=_class_propinit_37 synthetic=1
.align 2

.globl _class_propinit_37
_class_propinit_37:
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
_eir__class_propinit_37_entry_0:
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
_class_propinit_37_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_37
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
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_38_entry_0:
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
_class_propinit_38_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_38
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
_class_propinit_47_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_47
    ; @fn name=_class_propinit_49 symbol=_class_propinit_49 synthetic=1
.align 2

.globl _class_propinit_49
_class_propinit_49:
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
_eir__class_propinit_49_entry_0:
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
_class_propinit_49_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_49
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
    ; @fn name=_class_propinit_53 symbol=_class_propinit_53 synthetic=1
.align 2

.globl _class_propinit_53
_class_propinit_53:
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
_eir__class_propinit_53_entry_0:
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
_class_propinit_53_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_53
    ; @fn name=_class_propinit_55 symbol=_class_propinit_55 synthetic=1
.align 2

.globl _class_propinit_55
_class_propinit_55:
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
_eir__class_propinit_55_entry_0:
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
_class_propinit_55_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_55
    ; @fn name=_class_propinit_56 symbol=_class_propinit_56 synthetic=1
.align 2

.globl _class_propinit_56
_class_propinit_56:
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
_eir__class_propinit_56_entry_0:
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
_class_propinit_56_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_56
    ; @fn name=_class_propinit_57 symbol=_class_propinit_57 synthetic=1
.align 2

.globl _class_propinit_57
_class_propinit_57:
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
_eir__class_propinit_57_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_57_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_57
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
    ; @fn name=_class_propinit_63 symbol=_class_propinit_63 synthetic=1
.align 2

.globl _class_propinit_63
_class_propinit_63:
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
_eir__class_propinit_63_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_63_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_63
    ; @fn name=_class_propinit_64 symbol=_class_propinit_64 synthetic=1
.align 2

.globl _class_propinit_64
_class_propinit_64:
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
_eir__class_propinit_64_entry_0:
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
_class_propinit_64_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_64
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
    ; @fn name=_class_propinit_70 symbol=_class_propinit_70 synthetic=1
.align 2

.globl _class_propinit_70
_class_propinit_70:
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
_eir__class_propinit_70_entry_0:
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
_class_propinit_70_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_70
    ; @fn name=_class_propinit_71 symbol=_class_propinit_71 synthetic=1
.align 2

.globl _class_propinit_71
_class_propinit_71:
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
_eir__class_propinit_71_entry_0:
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
_class_propinit_71_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_71
    ; @fn name=_class_propinit_72 symbol=_class_propinit_72 synthetic=1
.align 2

.globl _class_propinit_72
_class_propinit_72:
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
_eir__class_propinit_72_entry_0:
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
_class_propinit_72_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_72
    ; @fn name=_class_propinit_73 symbol=_class_propinit_73 synthetic=1
.align 2

.globl _class_propinit_73
_class_propinit_73:
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
_eir__class_propinit_73_entry_0:
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
_class_propinit_73_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_73
    ; @fn name=_class_propinit_78 symbol=_class_propinit_78 synthetic=1
.align 2

.globl _class_propinit_78
_class_propinit_78:
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
_eir__class_propinit_78_entry_0:
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
_class_propinit_78_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_78
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
    ; @fn name=_class_propinit_88 symbol=_class_propinit_88 synthetic=1
.align 2

.globl _class_propinit_88
_class_propinit_88:
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
_eir__class_propinit_88_entry_0:
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
_class_propinit_88_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_88
    ; @fn name=_class_propinit_89 symbol=_class_propinit_89 synthetic=1
.align 2

.globl _class_propinit_89
_class_propinit_89:
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
_eir__class_propinit_89_entry_0:
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
_class_propinit_89_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_89
    ; @fn name=_class_propinit_92 symbol=_class_propinit_92 synthetic=1
.align 2

.globl _class_propinit_92
_class_propinit_92:
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
_eir__class_propinit_92_entry_0:
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
_class_propinit_92_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_92
    ; @fn name=_class_propinit_97 symbol=_class_propinit_97 synthetic=1
.align 2

.globl _class_propinit_97
_class_propinit_97:
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
_eir__class_propinit_97_entry_0:
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
_class_propinit_97_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_97
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
    ; @fn name=_class_propinit_101 symbol=_class_propinit_101 synthetic=1
.align 2

.globl _class_propinit_101
_class_propinit_101:
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
_eir__class_propinit_101_entry_0:
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
_class_propinit_101_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_101
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
    sub sp, sp, #1552
    mov x9, sp
    add x9, x9, #1536
    stp x29, x30, [x9]
    add x29, sp, #1536
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #1528
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #1496
    str x21, [x9]
    sub x9, x29, #1504
    str x22, [x9]
    sub x9, x29, #1512
    str x23, [x9]
    sub x9, x29, #1520
    str x24, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #1416
    str xzr, [x9]
    sub x9, x29, #1432
    str xzr, [x9]
    sub x9, x29, #1424
    str xzr, [x9]
    sub x9, x29, #1440
    str xzr, [x9]
    sub x9, x29, #1448
    str xzr, [x9]
    sub x9, x29, #1456
    str xzr, [x9]
    sub x9, x29, #1464
    str xzr, [x9]
    sub x9, x29, #1472
    str xzr, [x9]
    sub x9, x29, #1480
    str xzr, [x9]
    sub x9, x29, #1488
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=8 col=1 end=8:8 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=11 end=8:16 op=array_new
    mov x0, #5
    mov x1, #8
    bl __rt_array_new
    stur x0, [x29, #-8]
    ; @src line=8 col=17 end=8:18 op=const_i64
    mov x0, #2
    mov x21, x0
    ; @src line=8 col=17 end=8:18 op=array_push
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=8 col=20 end=8:21 op=const_i64
    mov x0, #3
    mov x21, x0
    ; @src line=8 col=20 end=8:21 op=array_push
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=8 col=23 end=8:24 op=const_i64
    mov x0, #5
    mov x21, x0
    ; @src line=8 col=23 end=8:24 op=array_push
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=8 col=26 end=8:27 op=const_i64
    mov x0, #7
    mov x21, x0
    ; @src line=8 col=26 end=8:27 op=array_push
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=8 col=29 end=8:31 op=const_i64
    mov x0, #11
    mov x21, x0
    ; @src line=8 col=29 end=8:31 op=array_push
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=8 col=1 end=8:8 op=acquire
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-56]
    ; @src line=8 col=1 end=8:8 op=store_local
    ldur x0, [x29, #-56]
    sub x9, x29, #1416
    str x0, [x9]
    ; @src line=8 col=1 end=8:8 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_any
    ; @src line=9 col=1 end=9:5 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=6 op=const_str
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #8
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=9 col=25 end=9:32 op=load_local
    sub x9, x29, #1416
    ldr x0, [x9]
    stur x0, [x29, #-80]
    ; @src line=9 col=19 end=9:33 op=runtime_call
    ldur x0, [x29, #-80]
    cbz x0, _eir_main_count_null_container_0
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir_main_count_null_container_0
    ldr x0, [x0]
    b _eir_main_count_done_1
_eir_main_count_null_container_0:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_2
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #107
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_static_exception_throw_2:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_type_error_class_id@PAGE
    add x9, x9, _spl_type_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_5@PAGE
    add x9, x9, _str_5@PAGEOFF
    str x9, [x0, #8]
    mov x9, #73
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_count_done_1:
    mov x21, x0
    ; @src line=9 col=19 end=9:33 op=nop
    ; @src line=9 col=17 end=9:33 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=9 col=17 end=9:33 op=str_concat
    ldur x1, [x29, #-72]
    ldur x2, [x29, #-64]
    ldur x3, [x29, #-104]
    ldur x4, [x29, #-96]
    bl __rt_concat
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=9 col=17 end=9:33 op=release
    ; @src line=9 col=36 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=9 col=34 end=9:36 op=str_concat
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    ldur x3, [x29, #-136]
    ldur x4, [x29, #-128]
    bl __rt_concat
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=9 col=34 end=9:36 op=release
    ; @src line=9 col=55 end=9:56 op=const_i64
    mov x0, #11
    mov x21, x0
    ; @src line=9 col=46 end=9:56 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=9 col=46 end=9:56 op=str_concat
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]
    ldur x3, [x29, #-176]
    ldur x4, [x29, #-168]
    bl __rt_concat
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ; @src line=9 col=46 end=9:56 op=release
    ; @src line=9 col=46 end=9:56 op=release
    ; @src line=9 col=61 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-208]
    stur x2, [x29, #-200]
    ; @src line=9 col=59 end=9:61 op=str_concat
    ldur x1, [x29, #-192]
    ldur x2, [x29, #-184]
    ldur x3, [x29, #-208]
    ldur x4, [x29, #-200]
    bl __rt_concat
    stur x1, [x29, #-224]
    stur x2, [x29, #-216]
    ; @src line=9 col=59 end=9:61 op=release
    ; @src line=9 col=1 end=9:5 op=echo_value
    ldur x1, [x29, #-224]
    ldur x2, [x29, #-216]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=9 col=1 end=9:5 op=release
    ; @src line=12 col=1 end=12:5 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=8 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #4
    stur x1, [x29, #-240]
    stur x2, [x29, #-232]
    ; @src line=12 col=1 end=12:5 op=acquire
    ldur x1, [x29, #-240]
    ldur x2, [x29, #-232]
    bl __rt_str_persist
    sub x9, x29, #256
    str x1, [x9]
    stur x2, [x29, #-248]
    ; @src line=12 col=1 end=12:5 op=store_local
    sub x9, x29, #256
    ldr x1, [x9]
    ldur x2, [x29, #-248]
    sub x9, x29, #1432
    str x1, [x9]
    sub x9, x29, #1424
    str x2, [x9]
    ; @src line=13 col=1 end=13:8 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=11 end=13:16 op=hash_new
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #264
    str x0, [x9]
    ; @src line=14 col=5 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #5
    sub x9, x29, #280
    str x1, [x9]
    sub x9, x29, #272
    str x2, [x9]
    ; @src line=14 col=18 end=14:23 op=const_bool
    mov x0, #0
    mov x21, x0
    ; @src line=13 col=11 end=13:16 op=hash_set
    sub x9, x29, #280
    ldr x1, [x9]
    sub x9, x29, #272
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #264
    ldr x0, [x9]
    mov x5, #3
    bl __rt_hash_set
    sub x9, x29, #264
    str x0, [x9]
    ; @src line=15 col=5 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #3
    sub x9, x29, #304
    str x1, [x9]
    sub x9, x29, #296
    str x2, [x9]
    ; @src line=15 col=18 end=15:22 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #4
    sub x9, x29, #320
    str x1, [x9]
    sub x9, x29, #312
    str x2, [x9]
    ; @src line=13 col=11 end=13:16 op=hash_set
    sub x9, x29, #304
    ldr x1, [x9]
    sub x9, x29, #296
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #320
    ldr x1, [x9]
    sub x9, x29, #312
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #264
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #264
    str x0, [x9]
    ; @src line=16 col=5 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #6
    sub x9, x29, #336
    str x1, [x9]
    sub x9, x29, #328
    str x2, [x9]
    ; @src line=16 col=18 end=16:22 op=const_i64
    mov x0, #1234
    mov x21, x0
    ; @src line=13 col=11 end=13:16 op=hash_set
    sub x9, x29, #336
    ldr x1, [x9]
    sub x9, x29, #328
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #264
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #264
    str x0, [x9]
    ; @src line=13 col=1 end=13:8 op=acquire
    sub x9, x29, #264
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #352
    str x0, [x9]
    ; @src line=13 col=1 end=13:8 op=store_local
    sub x9, x29, #352
    ldr x0, [x9]
    sub x9, x29, #1440
    str x0, [x9]
    ; @src line=13 col=1 end=13:8 op=release
    sub x9, x29, #264
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=81 end=18:83 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #22
    sub x9, x29, #368
    str x1, [x9]
    sub x9, x29, #360
    str x2, [x9]
    ; @src line=18 col=1 end=18:5 op=echo_value
    sub x9, x29, #368
    ldr x1, [x9]
    sub x9, x29, #360
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=21 col=1 end=21:8 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=11 end=21:16 op=hash_new
    mov x0, #16
    mov x1, #5
    bl __rt_hash_new
    sub x9, x29, #376
    str x0, [x9]
    ; @src line=22 col=5 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #4
    sub x9, x29, #392
    str x1, [x9]
    sub x9, x29, #384
    str x2, [x9]
    ; @src line=22 col=16 end=22:17 op=hash_new
    mov x0, #16
    mov x1, #1
    bl __rt_hash_new
    sub x9, x29, #400
    str x0, [x9]
    ; @src line=22 col=17 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #4
    sub x9, x29, #416
    str x1, [x9]
    sub x9, x29, #408
    str x2, [x9]
    ; @src line=22 col=27 op=const_str
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #1
    sub x9, x29, #432
    str x1, [x9]
    sub x9, x29, #424
    str x2, [x9]
    ; @src line=22 col=16 end=22:17 op=hash_set
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #432
    ldr x1, [x9]
    sub x9, x29, #424
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #400
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #400
    str x0, [x9]
    ; @src line=22 col=32 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #6
    sub x9, x29, #448
    str x1, [x9]
    sub x9, x29, #440
    str x2, [x9]
    ; @src line=22 col=44 op=const_str
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #3
    sub x9, x29, #464
    str x1, [x9]
    sub x9, x29, #456
    str x2, [x9]
    ; @src line=22 col=16 end=22:17 op=hash_set
    sub x9, x29, #448
    ldr x1, [x9]
    sub x9, x29, #440
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #464
    ldr x1, [x9]
    sub x9, x29, #456
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #400
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #400
    str x0, [x9]
    ; @src line=21 col=11 end=21:16 op=hash_set
    sub x9, x29, #392
    ldr x1, [x9]
    sub x9, x29, #384
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #400
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #376
    ldr x0, [x9]
    mov x5, #5
    bl __rt_hash_set
    sub x9, x29, #376
    str x0, [x9]
    ; @src line=23 col=5 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #5
    sub x9, x29, #480
    str x1, [x9]
    sub x9, x29, #472
    str x2, [x9]
    ; @src line=23 col=16 end=23:21 op=hash_new
    mov x0, #16
    mov x1, #1
    bl __rt_hash_new
    sub x9, x29, #488
    str x0, [x9]
    ; @src line=23 col=22 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #4
    sub x9, x29, #504
    str x1, [x9]
    sub x9, x29, #496
    str x2, [x9]
    ; @src line=23 col=32 op=const_str
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #6
    sub x9, x29, #520
    str x1, [x9]
    sub x9, x29, #512
    str x2, [x9]
    ; @src line=23 col=16 end=23:21 op=hash_set
    sub x9, x29, #504
    ldr x1, [x9]
    sub x9, x29, #496
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #520
    ldr x1, [x9]
    sub x9, x29, #512
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #488
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #488
    str x0, [x9]
    ; @src line=23 col=42 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #6
    sub x9, x29, #536
    str x1, [x9]
    sub x9, x29, #528
    str x2, [x9]
    ; @src line=23 col=54 op=const_str
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #3
    sub x9, x29, #552
    str x1, [x9]
    sub x9, x29, #544
    str x2, [x9]
    ; @src line=23 col=16 end=23:21 op=hash_set
    sub x9, x29, #536
    ldr x1, [x9]
    sub x9, x29, #528
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #552
    ldr x1, [x9]
    sub x9, x29, #544
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #488
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #488
    str x0, [x9]
    ; @src line=21 col=11 end=21:16 op=hash_set
    sub x9, x29, #480
    ldr x1, [x9]
    sub x9, x29, #472
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #488
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #376
    ldr x0, [x9]
    mov x5, #5
    bl __rt_hash_set
    sub x9, x29, #376
    str x0, [x9]
    ; @src line=21 col=1 end=21:8 op=acquire
    sub x9, x29, #376
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #560
    str x0, [x9]
    ; @src line=21 col=1 end=21:8 op=store_local
    sub x9, x29, #560
    ldr x0, [x9]
    sub x9, x29, #1448
    str x0, [x9]
    ; @src line=21 col=1 end=21:8 op=release
    sub x9, x29, #376
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=25 col=1 end=25:8 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #1448
    ldr x0, [x9]
    sub x9, x29, #568
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=iter_start
    sub x9, x29, #568
    ldr x0, [x9]
    sub x9, x29, #640
    str x0, [x9]
    mov x0, #0
    sub x9, x29, #632
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=25 col=10 end=25:17 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #656
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #656
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #664
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #664
    ldr x0, [x9]
    sub x9, x29, #1456
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #656
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=25 col=10 end=25:17 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #680
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #680
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #688
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #688
    ldr x0, [x9]
    sub x9, x29, #1464
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #680
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_1
    ; @block name=foreach.next
_eir_main_foreach_next_1:
    ; @src line=25 col=10 end=25:17 op=iter_next
    sub x9, x29, #640
    ldr x0, [x9]
    sub x9, x29, #632
    ldr x1, [x9]
    bl __rt_hash_iter_next
    cmn x0, #1
    sub x9, x29, #632
    str x0, [x9]
    sub x9, x29, #624
    str x1, [x9]
    sub x9, x29, #616
    str x2, [x9]
    sub x9, x29, #608
    str x3, [x9]
    sub x9, x29, #600
    str x4, [x9]
    sub x9, x29, #592
    str x5, [x9]
    sub x9, x29, #584
    str x6, [x9]
    cset x0, ne
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_2
    b _eir_main_foreach_exit_3
    ; @block name=foreach.body
_eir_main_foreach_body_2:
    ; @src line=25 col=10 end=25:17 op=iter_current_key
    sub x9, x29, #624
    ldr x1, [x9]
    sub x9, x29, #616
    ldr x2, [x9]
    cmn x2, #1
    b.ne _eir_main_iter_hash_key_string_3
    mov x0, #0
    mov x2, xzr
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_key_done_4
_eir_main_iter_hash_key_string_3:
    mov x0, #1
    bl __rt_mixed_from_value
_eir_main_iter_hash_key_done_4:
    sub x9, x29, #704
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #704
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #712
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #1456
    ldr x0, [x9]
    sub x9, x29, #720
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #720
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #712
    ldr x0, [x9]
    sub x9, x29, #1456
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #704
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=iter_current_value
    sub x9, x29, #592
    ldr x5, [x9]
    sub x9, x29, #608
    ldr x3, [x9]
    sub x9, x29, #600
    ldr x4, [x9]
    mov x1, x3
    mov x3, x5
    bl __rt_deref_if_reference
    mov x5, x3
    mov x3, x1
    cmp x5, #7
    b.eq _eir_main_iter_hash_value_inspect_box_5
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_value_boxed_6
_eir_main_iter_hash_value_inspect_box_5:
    str x3, [sp, #-16]!
    mov x0, x3
    bl __rt_heap_kind
    cmp x0, #5
    b.eq _eir_main_iter_hash_value_reuse_box_7
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    bl __rt_heap_kind
    mov x9, x0
    cmp x0, #2
    cset x10, hs
    cmp x0, #4
    cset x11, ls
    and x10, x10, x11
    add x9, x9, #2
    mov x0, #8
    cmp x10, #0
    csel x0, x9, x0, ne
    ldr x1, [sp], #16
    mov x2, xzr
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_value_tagged_done_8
_eir_main_iter_hash_value_reuse_box_7:
    ldr x0, [sp], #16
    bl __rt_incref
_eir_main_iter_hash_value_tagged_done_8:
_eir_main_iter_hash_value_boxed_6:
    sub x9, x29, #728
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #728
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #736
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #1464
    ldr x0, [x9]
    sub x9, x29, #744
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #744
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #736
    ldr x0, [x9]
    sub x9, x29, #1464
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #728
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=26 col=5 end=26:9 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=10 end=26:15 op=load_local
    sub x9, x29, #1456
    ldr x0, [x9]
    sub x9, x29, #752
    str x0, [x9]
    ; @src line=26 col=16 end=26:18 op=cast
    sub x9, x29, #752
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_9
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_object_9:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_12
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_13
    mov x10, #6
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_14
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_15
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_16
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_17
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_18
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_19
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_20
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_21
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_22
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_23
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_24
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_25
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_26
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_27
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_28
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_29
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_30
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_31
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_32
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_33
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_34
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_35
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_36
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_37
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_38
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_39
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_40
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_41
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_42
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_43
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_44
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_45
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_46
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_47
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_48
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_49
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_50
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_51
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_52
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_53
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_54
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_55
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_56
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_57
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_58
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_59
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_60
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_61
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_62
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_63
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_64
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_65
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_66
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_67
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_68
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_69
    b _eir_main_mixed_string_no_match_10
_eir_main_mixed_string_CachingIterator_12:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_Exception_13:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_LogicException_14:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_BadFunctionCallException_15:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_SplFileInfo_16:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionNamedType_17:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateException_18:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateInvalidTimeZoneException_19:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_OutOfRangeException_20:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionFunctionAbstract_21:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionFunction_22:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_RuntimeException_23:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_Error_24:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_TypeError_25:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DirectoryIterator_26:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_FilesystemIterator_27:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_GlobIterator_28:
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
    b _eir_main_mixed_string_done_11
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
    b _eir_main_mixed_string_done_11
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateError_31:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateObjectError_32:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_LengthException_33:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateMalformedPeriodStringException_34:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_OverflowException_35:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_PharFileInfo_36:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionClass_37:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionMethod_38:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionProperty_39:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionObject_40:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionEnumUnitCase_41:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_RecursiveCachingIterator_42:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateMalformedIntervalStringException_43:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionParameter_44:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionUnionType_45:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ValueError_46:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_RecursiveDirectoryIterator_47:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_Phar_48:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateInvalidOperationException_49:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_PharData_50:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateRangeError_51:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateUnknownException_52:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_BadMethodCallException_53:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ArithmeticError_54:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_InvalidArgumentException_55:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_UnderflowException_56:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionEnum_57:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_FiberError_58:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_JsonException_59:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DomainException_60:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionException_61:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_RangeException_62:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionIntersectionType_63:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionClassConstant_64:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_UnhandledMatchError_65:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_OutOfBoundsException_66:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_ReflectionEnumBackedCase_67:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_DateMalformedStringException_68:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_UnexpectedValueException_69:
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
    b _eir_main_mixed_string_done_11
_eir_main_mixed_string_no_match_10:
    mov x0, #2
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_11:
    sub x9, x29, #768
    str x1, [x9]
    sub x9, x29, #760
    str x2, [x9]
    ; @src line=26 col=18 op=const_str
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #4
    sub x9, x29, #784
    str x1, [x9]
    sub x9, x29, #776
    str x2, [x9]
    ; @src line=26 col=16 end=26:18 op=str_concat
    sub x9, x29, #768
    ldr x1, [x9]
    sub x9, x29, #760
    ldr x2, [x9]
    sub x9, x29, #784
    ldr x3, [x9]
    sub x9, x29, #776
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #800
    str x1, [x9]
    sub x9, x29, #792
    str x2, [x9]
    ; @src line=26 col=16 end=26:18 op=release
    sub x9, x29, #768
    ldr x1, [x9]
    sub x9, x29, #760
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=27 end=26:33 op=load_local
    sub x9, x29, #1464
    ldr x0, [x9]
    sub x9, x29, #808
    str x0, [x9]
    ; @src line=26 col=34 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #6
    sub x9, x29, #824
    str x1, [x9]
    sub x9, x29, #816
    str x2, [x9]
    ; @src line=26 col=33 end=26:34 op=const_bool
    mov x0, #1
    mov x21, x0
    ; @src line=26 col=33 end=26:34 op=runtime_call
    sub x9, x29, #824
    ldr x1, [x9]
    sub x9, x29, #816
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    mov x3, x21
    sub x9, x29, #808
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #840
    str x0, [x9]
    ; @src line=26 col=25 end=26:34 op=cast
    sub x9, x29, #840
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_70
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_object_70:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_73
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_74
    mov x10, #6
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_75
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_76
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_77
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_78
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_79
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_80
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_81
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_82
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_83
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_84
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_85
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_86
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_87
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_88
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_89
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_90
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_91
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_92
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_93
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_94
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_95
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_96
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_97
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_98
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_99
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_100
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_101
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_102
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_103
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_104
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_105
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_106
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_107
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_108
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_109
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_110
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_111
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_112
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_113
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_114
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_115
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_116
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_117
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_118
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_119
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_120
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_121
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_122
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_123
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_124
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_125
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_126
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_127
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_128
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_129
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_130
    b _eir_main_mixed_string_no_match_71
_eir_main_mixed_string_CachingIterator_73:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_Exception_74:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_LogicException_75:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_BadFunctionCallException_76:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_SplFileInfo_77:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionNamedType_78:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateException_79:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateInvalidTimeZoneException_80:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_OutOfRangeException_81:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionFunctionAbstract_82:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionFunction_83:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_RuntimeException_84:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_Error_85:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_TypeError_86:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DirectoryIterator_87:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_FilesystemIterator_88:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_GlobIterator_89:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_SplFileObject_90:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_SplTempFileObject_91:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateError_92:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateObjectError_93:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_LengthException_94:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateMalformedPeriodStringException_95:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_OverflowException_96:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_PharFileInfo_97:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionClass_98:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionMethod_99:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionProperty_100:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionObject_101:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionEnumUnitCase_102:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_RecursiveCachingIterator_103:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateMalformedIntervalStringException_104:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionParameter_105:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionUnionType_106:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ValueError_107:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_RecursiveDirectoryIterator_108:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_Phar_109:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateInvalidOperationException_110:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_PharData_111:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateRangeError_112:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateUnknownException_113:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_BadMethodCallException_114:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ArithmeticError_115:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_InvalidArgumentException_116:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_UnderflowException_117:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionEnum_118:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_FiberError_119:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_JsonException_120:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DomainException_121:
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
    b _eir_main_mixed_string_done_72
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
    b _eir_main_mixed_string_done_72
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionIntersectionType_124:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionClassConstant_125:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_UnhandledMatchError_126:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_OutOfBoundsException_127:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_ReflectionEnumBackedCase_128:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_DateMalformedStringException_129:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_UnexpectedValueException_130:
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
    b _eir_main_mixed_string_done_72
_eir_main_mixed_string_no_match_71:
    mov x0, #2
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_72:
    sub x9, x29, #856
    str x1, [x9]
    sub x9, x29, #848
    str x2, [x9]
    ; @src line=26 col=25 end=26:34 op=release
    sub x9, x29, #840
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=26 col=25 end=26:34 op=str_concat
    sub x9, x29, #800
    ldr x1, [x9]
    sub x9, x29, #792
    ldr x2, [x9]
    sub x9, x29, #856
    ldr x3, [x9]
    sub x9, x29, #848
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #872
    str x1, [x9]
    sub x9, x29, #864
    str x2, [x9]
    ; @src line=26 col=25 end=26:34 op=release
    ; @src line=26 col=25 end=26:34 op=release
    sub x9, x29, #856
    ldr x1, [x9]
    sub x9, x29, #848
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=46 op=const_str
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #1
    sub x9, x29, #888
    str x1, [x9]
    sub x9, x29, #880
    str x2, [x9]
    ; @src line=26 col=44 end=26:46 op=str_concat
    sub x9, x29, #872
    ldr x1, [x9]
    sub x9, x29, #864
    ldr x2, [x9]
    sub x9, x29, #888
    ldr x3, [x9]
    sub x9, x29, #880
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #904
    str x1, [x9]
    sub x9, x29, #896
    str x2, [x9]
    ; @src line=26 col=44 end=26:46 op=release
    ; @src line=26 col=52 end=26:58 op=load_local
    sub x9, x29, #1464
    ldr x0, [x9]
    sub x9, x29, #912
    str x0, [x9]
    ; @src line=26 col=59 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #4
    sub x9, x29, #928
    str x1, [x9]
    sub x9, x29, #920
    str x2, [x9]
    ; @src line=26 col=58 end=26:59 op=const_bool
    mov x0, #1
    mov x21, x0
    ; @src line=26 col=58 end=26:59 op=runtime_call
    sub x9, x29, #928
    ldr x1, [x9]
    sub x9, x29, #920
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    mov x3, x21
    sub x9, x29, #912
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #944
    str x0, [x9]
    ; @src line=26 col=50 end=26:59 op=cast
    sub x9, x29, #944
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_131
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_object_131:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_134
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_135
    mov x10, #6
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_136
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_137
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_138
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_139
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_140
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_141
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_142
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_143
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_144
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_145
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_146
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_147
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_148
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_149
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_150
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_151
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_152
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_153
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_154
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_155
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_156
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_157
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_158
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_159
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_160
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_161
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_162
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_163
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_164
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_165
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_166
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_167
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_168
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_169
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_170
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_171
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_172
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_173
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_174
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_175
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_176
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_177
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_178
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_179
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_180
    mov x10, #79
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_181
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_182
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_183
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_184
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_185
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_186
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_187
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_188
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_189
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_190
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_191
    b _eir_main_mixed_string_no_match_132
_eir_main_mixed_string_CachingIterator_134:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_Exception_135:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_LogicException_136:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_BadFunctionCallException_137:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_SplFileInfo_138:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionNamedType_139:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateException_140:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateInvalidTimeZoneException_141:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_OutOfRangeException_142:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionFunctionAbstract_143:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionFunction_144:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_RuntimeException_145:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_Error_146:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_TypeError_147:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DirectoryIterator_148:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_FilesystemIterator_149:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_GlobIterator_150:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_SplFileObject_151:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_SplTempFileObject_152:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateError_153:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateObjectError_154:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_LengthException_155:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateMalformedPeriodStringException_156:
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
    b _eir_main_mixed_string_done_133
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_PharFileInfo_158:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionClass_159:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionMethod_160:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionProperty_161:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionObject_162:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionEnumUnitCase_163:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_RecursiveCachingIterator_164:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateMalformedIntervalStringException_165:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionParameter_166:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionUnionType_167:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ValueError_168:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_RecursiveDirectoryIterator_169:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_Phar_170:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateInvalidOperationException_171:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_PharData_172:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateRangeError_173:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateUnknownException_174:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_BadMethodCallException_175:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ArithmeticError_176:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_InvalidArgumentException_177:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_UnderflowException_178:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionEnum_179:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_FiberError_180:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_JsonException_181:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DomainException_182:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionException_183:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_RangeException_184:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionIntersectionType_185:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionClassConstant_186:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_UnhandledMatchError_187:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_OutOfBoundsException_188:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_ReflectionEnumBackedCase_189:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_DateMalformedStringException_190:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_UnexpectedValueException_191:
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
    b _eir_main_mixed_string_done_133
_eir_main_mixed_string_no_match_132:
    mov x0, #2
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_133:
    sub x9, x29, #960
    str x1, [x9]
    sub x9, x29, #952
    str x2, [x9]
    ; @src line=26 col=50 end=26:59 op=release
    sub x9, x29, #944
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=26 col=50 end=26:59 op=str_concat
    sub x9, x29, #904
    ldr x1, [x9]
    sub x9, x29, #896
    ldr x2, [x9]
    sub x9, x29, #960
    ldr x3, [x9]
    sub x9, x29, #952
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #976
    str x1, [x9]
    sub x9, x29, #968
    str x2, [x9]
    ; @src line=26 col=50 end=26:59 op=release
    ; @src line=26 col=50 end=26:59 op=release
    sub x9, x29, #960
    ldr x1, [x9]
    sub x9, x29, #952
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=69 op=const_str
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #1
    sub x9, x29, #992
    str x1, [x9]
    sub x9, x29, #984
    str x2, [x9]
    ; @src line=26 col=67 end=26:69 op=str_concat
    sub x9, x29, #976
    ldr x1, [x9]
    sub x9, x29, #968
    ldr x2, [x9]
    sub x9, x29, #992
    ldr x3, [x9]
    sub x9, x29, #984
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1008
    str x1, [x9]
    sub x9, x29, #1000
    str x2, [x9]
    ; @src line=26 col=67 end=26:69 op=release
    ; @src line=26 col=5 end=26:9 op=echo_value
    sub x9, x29, #1008
    ldr x1, [x9]
    sub x9, x29, #1000
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=26 col=5 end=26:9 op=release
    b _eir_main_foreach_next_1
    ; @block name=foreach.exit
_eir_main_foreach_exit_3:
    ; @src line=25 col=10 end=25:17 op=nop
    ; @src line=30 col=1 end=30:6 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=9 end=30:14 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #1016
    str x0, [x9]
    ; @src line=30 col=15 end=30:16 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=30 col=15 end=30:16 op=array_push
    mov x1, x21
    sub x9, x29, #1016
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1016
    str x0, [x9]
    ; @src line=30 col=18 end=30:19 op=const_i64
    mov x0, #2
    mov x21, x0
    ; @src line=30 col=18 end=30:19 op=array_push
    mov x1, x21
    sub x9, x29, #1016
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1016
    str x0, [x9]
    ; @src line=30 col=1 end=30:6 op=acquire
    sub x9, x29, #1016
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1040
    str x0, [x9]
    ; @src line=30 col=1 end=30:6 op=store_local
    sub x9, x29, #1040
    ldr x0, [x9]
    sub x9, x29, #1472
    str x0, [x9]
    ; @src line=30 col=1 end=30:6 op=release
    sub x9, x29, #1016
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=31 col=1 end=31:10 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=31 col=22 end=31:27 op=load_local
    sub x9, x29, #1472
    ldr x0, [x9]
    sub x9, x29, #1048
    str x0, [x9]
    ; @src line=31 col=29 end=31:30 op=const_i64
    mov x0, #3
    mov x21, x0
    ; @src line=31 col=32 end=31:33 op=const_i64
    mov x0, #4
    mov x22, x0
    ; @src line=31 col=13 end=31:18 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #1072
    str x0, [x9]
    ; @src line=31 col=19 end=31:22 op=array_len
    sub x9, x29, #1048
    ldr x0, [x9]
    cbz x0, _eir_main_array_len_null_192
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir_main_array_len_null_192
    ldr x0, [x0]
    b _eir_main_array_len_done_193
_eir_main_array_len_null_192:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_194
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #86
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_static_exception_throw_194:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_25@PAGE
    add x9, x9, _str_25@PAGEOFF
    str x9, [x0, #8]
    mov x9, #56
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_array_len_done_193:
    mov x23, x0
    ; @src line=31 col=19 end=31:22 op=const_i64
    mov x0, #0
    sub x9, x29, #1088
    str x0, [x9]
    sub x9, x29, #1088
    ldr x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp], #16
    sub x9, x29, #1096
    str x0, [x9]
    b _eir_main_array_spread_next_4
    ; @block name=array.spread.next
_eir_main_array_spread_next_4:
    ; @src line=31 col=19 end=31:22 op=icmp
    sub x9, x29, #1096
    ldr x0, [x9]
    mov x10, x23
    cmp x0, x10
    cset x0, lt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir_main_array_spread_body_5
    b _eir_main_array_spread_exit_6
    ; @block name=array.spread.body
_eir_main_array_spread_body_5:
    ; @src line=31 col=19 end=31:22 op=array_get
    sub x9, x29, #1096
    ldr x0, [x9]
    sub x9, x29, #1048
    ldr x9, [x9]
    cbz x9, _eir_main_array_get_null_recv_196
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_array_get_null_recv_196
    cmp x0, #0
    b.lt _eir_main_array_get_null_195
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_195
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    b _eir_main_array_get_done_198
_eir_main_array_get_null_195:
    bl __rt_warn_undefined_array_key_int
    b _eir_main_array_get_fallback_197
_eir_main_array_get_null_recv_196:
    bl __rt_warn_array_offset_on_null
_eir_main_array_get_fallback_197:
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_198:
    mov x24, x0
    ; @src line=31 col=19 end=31:22 op=array_push
    mov x1, x24
    sub x9, x29, #1072
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1072
    str x0, [x9]
    ; @src line=31 col=19 end=31:22 op=const_i64
    mov x0, #1
    mov x12, x0
    ; @src line=31 col=19 end=31:22 op=iadd
    sub x9, x29, #1096
    ldr x0, [x9]
    mov x10, x12
    add x0, x0, x10
    sub x9, x29, #1128
    str x0, [x9]
    sub x9, x29, #1128
    ldr x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp], #16
    sub x9, x29, #1096
    str x0, [x9]
    b _eir_main_array_spread_next_4
    ; @block name=array.spread.exit
_eir_main_array_spread_exit_6:
    ; @src line=31 col=19 end=31:22 op=nop
    ; @src line=31 col=29 end=31:30 op=array_push
    mov x1, x21
    sub x9, x29, #1072
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1072
    str x0, [x9]
    ; @src line=31 col=32 end=31:33 op=array_push
    mov x1, x22
    sub x9, x29, #1072
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1072
    str x0, [x9]
    ; @src line=31 col=1 end=31:10 op=acquire
    sub x9, x29, #1072
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1136
    str x0, [x9]
    ; @src line=31 col=1 end=31:10 op=store_local
    sub x9, x29, #1136
    ldr x0, [x9]
    sub x9, x29, #1480
    str x0, [x9]
    ; @src line=31 col=1 end=31:10 op=release
    sub x9, x29, #1072
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=32 col=1 end=32:5 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=32 col=6 op=const_str
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #10
    sub x9, x29, #1152
    str x1, [x9]
    sub x9, x29, #1144
    str x2, [x9]
    ; @src line=32 col=29 op=const_str
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #1
    sub x9, x29, #1168
    str x1, [x9]
    sub x9, x29, #1160
    str x2, [x9]
    ; @src line=32 col=34 end=32:43 op=load_local
    sub x9, x29, #1480
    ldr x0, [x9]
    sub x9, x29, #1176
    str x0, [x9]
    ; @src line=32 col=21 end=32:44 op=runtime_call
    sub x9, x29, #1168
    ldr x1, [x9]
    sub x9, x29, #1160
    ldr x2, [x9]
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1176
    ldr x0, [x9]
    mov x3, x0
    ldp x1, x2, [sp], #16
    bl __rt_implode_int
    sub x9, x29, #1192
    str x1, [x9]
    sub x9, x29, #1184
    str x2, [x9]
    ; @src line=32 col=21 end=32:44 op=nop
    ; @src line=32 col=19 end=32:44 op=str_concat
    sub x9, x29, #1152
    ldr x1, [x9]
    sub x9, x29, #1144
    ldr x2, [x9]
    sub x9, x29, #1192
    ldr x3, [x9]
    sub x9, x29, #1184
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1208
    str x1, [x9]
    sub x9, x29, #1200
    str x2, [x9]
    ; @src line=32 col=19 end=32:44 op=release
    ; @src line=32 col=47 op=const_str
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #1
    sub x9, x29, #1224
    str x1, [x9]
    sub x9, x29, #1216
    str x2, [x9]
    ; @src line=32 col=45 end=32:47 op=str_concat
    sub x9, x29, #1208
    ldr x1, [x9]
    sub x9, x29, #1200
    ldr x2, [x9]
    sub x9, x29, #1224
    ldr x3, [x9]
    sub x9, x29, #1216
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1240
    str x1, [x9]
    sub x9, x29, #1232
    str x2, [x9]
    ; @src line=32 col=45 end=32:47 op=release
    ; @src line=32 col=1 end=32:5 op=echo_value
    sub x9, x29, #1240
    ldr x1, [x9]
    sub x9, x29, #1232
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=32 col=1 end=32:5 op=release
    ; @src line=35 col=1 end=35:8 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=35 col=23 end=35:28 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #1248
    str x0, [x9]
    ; @src line=35 col=29 end=35:30 op=const_i64
    mov x0, #1
    mov x23, x0
    ; @src line=35 col=29 end=35:30 op=array_push
    mov x1, x23
    sub x9, x29, #1248
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1248
    str x0, [x9]
    ; @src line=35 col=32 end=35:33 op=const_i64
    mov x0, #2
    mov x23, x0
    ; @src line=35 col=32 end=35:33 op=array_push
    mov x1, x23
    sub x9, x29, #1248
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1248
    str x0, [x9]
    ; @src line=35 col=36 end=35:37 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #1272
    str x0, [x9]
    ; @src line=35 col=37 end=35:38 op=const_i64
    mov x0, #3
    mov x23, x0
    ; @src line=35 col=37 end=35:38 op=array_push
    mov x1, x23
    sub x9, x29, #1272
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1272
    str x0, [x9]
    ; @src line=35 col=40 end=35:41 op=const_i64
    mov x0, #4
    mov x23, x0
    ; @src line=35 col=40 end=35:41 op=array_push
    mov x1, x23
    sub x9, x29, #1272
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #1272
    str x0, [x9]
    ; @src line=35 col=11 end=35:43 op=runtime_call
    sub x9, x29, #1248
    ldr x0, [x9]
    sub x9, x29, #1272
    ldr x1, [x9]
    bl __rt_array_merge
    sub x9, x29, #1296
    str x0, [x9]
    ; @src line=35 col=11 end=35:43 op=release
    sub x9, x29, #1248
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=35 col=11 end=35:43 op=release
    sub x9, x29, #1272
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=35 col=1 end=35:8 op=acquire
    sub x9, x29, #1296
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1304
    str x0, [x9]
    ; @src line=35 col=1 end=35:8 op=store_local
    sub x9, x29, #1304
    ldr x0, [x9]
    sub x9, x29, #1488
    str x0, [x9]
    ; @src line=35 col=1 end=35:8 op=release
    sub x9, x29, #1296
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=36 col=1 end=36:5 op=concat_reset
    sub x9, x29, #1528
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=36 col=6 op=const_str
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #8
    sub x9, x29, #1320
    str x1, [x9]
    sub x9, x29, #1312
    str x2, [x9]
    ; @src line=36 col=27 op=const_str
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #1
    sub x9, x29, #1336
    str x1, [x9]
    sub x9, x29, #1328
    str x2, [x9]
    ; @src line=36 col=32 end=36:39 op=load_local
    sub x9, x29, #1488
    ldr x0, [x9]
    sub x9, x29, #1344
    str x0, [x9]
    ; @src line=36 col=19 end=36:40 op=runtime_call
    sub x9, x29, #1336
    ldr x1, [x9]
    sub x9, x29, #1328
    ldr x2, [x9]
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1344
    ldr x0, [x9]
    mov x3, x0
    ldp x1, x2, [sp], #16
    bl __rt_implode_int
    sub x9, x29, #1360
    str x1, [x9]
    sub x9, x29, #1352
    str x2, [x9]
    ; @src line=36 col=19 end=36:40 op=nop
    ; @src line=36 col=17 end=36:40 op=str_concat
    sub x9, x29, #1320
    ldr x1, [x9]
    sub x9, x29, #1312
    ldr x2, [x9]
    sub x9, x29, #1360
    ldr x3, [x9]
    sub x9, x29, #1352
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1376
    str x1, [x9]
    sub x9, x29, #1368
    str x2, [x9]
    ; @src line=36 col=17 end=36:40 op=release
    ; @src line=36 col=43 op=const_str
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #1
    sub x9, x29, #1392
    str x1, [x9]
    sub x9, x29, #1384
    str x2, [x9]
    ; @src line=36 col=41 end=36:43 op=str_concat
    sub x9, x29, #1376
    ldr x1, [x9]
    sub x9, x29, #1368
    ldr x2, [x9]
    sub x9, x29, #1392
    ldr x3, [x9]
    sub x9, x29, #1384
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1408
    str x1, [x9]
    sub x9, x29, #1400
    str x2, [x9]
    ; @src line=36 col=41 end=36:43 op=release
    ; @src line=36 col=1 end=36:5 op=echo_value
    sub x9, x29, #1408
    ldr x1, [x9]
    sub x9, x29, #1400
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=36 col=1 end=36:5 op=release

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $primes
    sub x9, x29, #1416
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_199
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_199:
    ; epilogue cleanup $env
    sub x9, x29, #1432
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $config
    sub x9, x29, #1440
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_200
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_200:
    ; epilogue cleanup $routes
    sub x9, x29, #1448
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_201
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_201:
    ; epilogue cleanup $name
    sub x9, x29, #1456
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_202
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_202:
    ; epilogue cleanup $route
    sub x9, x29, #1464
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_203
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_203:
    ; epilogue cleanup $base
    sub x9, x29, #1472
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_204
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_204:
    ; epilogue cleanup $extended
    sub x9, x29, #1480
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_205
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_205:
    ; epilogue cleanup $merged
    sub x9, x29, #1488
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_206
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_206:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #1496
    ldr x21, [x9]
    sub x9, x29, #1504
    ldr x22, [x9]
    sub x9, x29, #1512
    ldr x23, [x9]
    sub x9, x29, #1520
    ldr x24, [x9]
    mov x9, sp
    add x9, x9, #1536
    ldp x29, x30, [x9]
    add sp, sp, #1552
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_207
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_207:
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
    .ascii "primes: "
.globl _str_4
_str_4:
    .ascii "Fatal error: Uncaught TypeError: count(): Argument #1 ($value) must be of type Countable|array, null given\n"
.globl _str_5
_str_5:
    .ascii "count(): Argument #1 ($value) must be of type Countable|array, null given"
.globl _str_6
_str_6:
    .ascii " (last "
.globl _str_7
_str_7:
    .ascii ")\n"
.globl _str_8
_str_8:
    .ascii "prod"
.globl _str_9
_str_9:
    .ascii "debug"
.globl _str_10
_str_10:
    .ascii "env"
.globl _str_11
_str_11:
    .ascii "secret"
.globl _str_12
_str_12:
    .ascii "env: prod, debug: off\n"
.globl _str_13
_str_13:
    .ascii "home"
.globl _str_14
_str_14:
    .ascii "path"
.globl _str_15
_str_15:
    .ascii "/"
.globl _str_16
_str_16:
    .ascii "method"
.globl _str_17
_str_17:
    .ascii "GET"
.globl _str_18
_str_18:
    .ascii "about"
.globl _str_19
_str_19:
    .ascii "/about"
.globl _str_20
_str_20:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_21
_str_21:
    .ascii " => "
.globl _str_22
_str_22:
    .ascii " "
.globl _str_23
_str_23:
    .ascii "\n"
.globl _str_24
_str_24:
    .ascii "Fatal error: Uncaught Error: Only arrays and Traversables can be unpacked, null given\n"
.globl _str_25
_str_25:
    .ascii "Only arrays and Traversables can be unpacked, null given"
.globl _str_26
_str_26:
    .ascii "extended: "
.globl _str_27
_str_27:
    .ascii ","
.globl _str_28
_str_28:
    .ascii "merged: "
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
    .quad _instanceof_name_class_5
    .quad 9
    .quad 5
    .quad 0
    .quad _instanceof_name_class_abs_5
    .quad 10
    .quad 5
    .quad 0
    .quad _instanceof_name_class_6
    .quad 14
    .quad 6
    .quad 0
    .quad _instanceof_name_class_abs_6
    .quad 15
    .quad 6
    .quad 0
    .quad _instanceof_name_class_12
    .quad 19
    .quad 12
    .quad 0
    .quad _instanceof_name_class_abs_12
    .quad 20
    .quad 12
    .quad 0
    .quad _instanceof_name_class_20
    .quad 16
    .quad 20
    .quad 0
    .quad _instanceof_name_class_abs_20
    .quad 17
    .quad 20
    .quad 0
    .quad _instanceof_name_class_21
    .quad 5
    .quad 21
    .quad 0
    .quad _instanceof_name_class_abs_21
    .quad 6
    .quad 21
    .quad 0
    .quad _instanceof_name_class_22
    .quad 9
    .quad 22
    .quad 0
    .quad _instanceof_name_class_abs_22
    .quad 10
    .quad 22
    .quad 0
    .quad _instanceof_name_class_29
    .quad 8
    .quad 29
    .quad 0
    .quad _instanceof_name_class_abs_29
    .quad 9
    .quad 29
    .quad 0
    .quad _instanceof_name_class_55
    .quad 10
    .quad 55
    .quad 0
    .quad _instanceof_name_class_abs_55
    .quad 11
    .quad 55
    .quad 0
    .quad _instanceof_name_class_70
    .quad 15
    .quad 70
    .quad 0
    .quad _instanceof_name_class_abs_70
    .quad 16
    .quad 70
    .quad 0
    .quad _instanceof_name_class_72
    .quad 24
    .quad 72
    .quad 0
    .quad _instanceof_name_class_abs_72
    .quad 25
    .quad 72
    .quad 0
    .quad _instanceof_name_class_79
    .quad 13
    .quad 79
    .quad 0
    .quad _instanceof_name_class_abs_79
    .quad 14
    .quad 79
    .quad 0
    .quad _instanceof_name_class_81
    .quad 19
    .quad 81
    .quad 0
    .quad _instanceof_name_class_abs_81
    .quad 20
    .quad 81
    .quad 0
    .quad _instanceof_name_class_97
    .quad 19
    .quad 97
    .quad 0
    .quad _instanceof_name_class_abs_97
    .quad 20
    .quad 97
    .quad 0
    .quad _instanceof_name_class_99
    .quad 20
    .quad 99
    .quad 0
    .quad _instanceof_name_class_abs_99
    .quad 21
    .quad 99
    .quad 0
    .quad _instanceof_name_interface_9
    .quad 10
    .quad 9
    .quad 1
    .quad _instanceof_name_interface_abs_9
    .quad 11
    .quad 9
    .quad 1
    .quad _instanceof_name_interface_12
    .quad 9
    .quad 12
    .quad 1
    .quad _instanceof_name_interface_abs_12
    .quad 10
    .quad 12
    .quad 1
.globl _instanceof_name_class_5
_instanceof_name_class_5:
    .ascii "Exception"
.globl _instanceof_name_class_abs_5
_instanceof_name_class_abs_5:
    .ascii "\\Exception"
.globl _instanceof_name_class_6
_instanceof_name_class_6:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_6
_instanceof_name_class_abs_6:
    .ascii "\\LogicException"
.globl _instanceof_name_class_12
_instanceof_name_class_12:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_12
_instanceof_name_class_abs_12:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_20
_instanceof_name_class_20:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_20
_instanceof_name_class_abs_20:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_21
_instanceof_name_class_21:
    .ascii "Error"
.globl _instanceof_name_class_abs_21
_instanceof_name_class_abs_21:
    .ascii "\\Error"
.globl _instanceof_name_class_22
_instanceof_name_class_22:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_22
_instanceof_name_class_abs_22:
    .ascii "\\TypeError"
.globl _instanceof_name_class_29
_instanceof_name_class_29:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_29
_instanceof_name_class_abs_29:
    .ascii "\\stdClass"
.globl _instanceof_name_class_55
_instanceof_name_class_55:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_55
_instanceof_name_class_abs_55:
    .ascii "\\ValueError"
.globl _instanceof_name_class_70
_instanceof_name_class_70:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_70
_instanceof_name_class_abs_70:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_72
_instanceof_name_class_72:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_72
_instanceof_name_class_abs_72:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_79
_instanceof_name_class_79:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_79
_instanceof_name_class_abs_79:
    .ascii "\\JsonException"
.globl _instanceof_name_class_81
_instanceof_name_class_81:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_81
_instanceof_name_class_abs_81:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_97
_instanceof_name_class_97:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_97
_instanceof_name_class_abs_97:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_99
_instanceof_name_class_99:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_99
_instanceof_name_class_abs_99:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_interface_9
_instanceof_name_interface_9:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_9
_instanceof_name_interface_abs_9:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_12
_instanceof_name_interface_12:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_12
_instanceof_name_interface_abs_12:
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_5
    .quad 9
    .quad _class_name_6
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
    .quad _class_name_12
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
    .quad _class_name_20
    .quad 16
    .quad _class_name_21
    .quad 5
    .quad _class_name_22
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
    .quad _class_name_29
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
    .quad _class_name_55
    .quad 10
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
    .quad _class_name_70
    .quad 15
    .quad _class_name_missing
    .quad 0
    .quad _class_name_72
    .quad 24
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
    .quad 13
    .quad _class_name_missing
    .quad 0
    .quad _class_name_81
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
    .quad _class_name_97
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_99
    .quad 20
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_5
_class_name_5:
    .ascii "Exception"
.globl _class_name_6
_class_name_6:
    .ascii "LogicException"
.globl _class_name_12
_class_name_12:
    .ascii "OutOfRangeException"
.globl _class_name_20
_class_name_20:
    .ascii "RuntimeException"
.globl _class_name_21
_class_name_21:
    .ascii "Error"
.globl _class_name_22
_class_name_22:
    .ascii "TypeError"
.globl _class_name_29
_class_name_29:
    .ascii "stdClass"
.globl _class_name_55
_class_name_55:
    .ascii "ValueError"
.globl _class_name_70
_class_name_70:
    .ascii "ArithmeticError"
.globl _class_name_72
_class_name_72:
    .ascii "InvalidArgumentException"
.globl _class_name_79
_class_name_79:
    .ascii "JsonException"
.globl _class_name_81
_class_name_81:
    .ascii "ReflectionException"
.globl _class_name_97
_class_name_97:
    .ascii "UnhandledMatchError"
.globl _class_name_99
_class_name_99:
    .ascii "OutOfBoundsException"
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
    .quad 95
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 78
.globl _generator_class_id
_generator_class_id:
    .quad 23
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 68
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 82
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 69
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 74
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 21
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 6
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 20
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 12
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 99
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 72
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 22
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 55
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 81
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 70
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_9
    .quad _interface_methods_12
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_5
    .quad _class_interfaces_6
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_12
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_20
    .quad _class_interfaces_21
    .quad _class_interfaces_22
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_29
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
    .quad _class_interfaces_55
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
    .quad _class_interfaces_70
    .quad _class_interfaces_missing
    .quad _class_interfaces_72
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_79
    .quad _class_interfaces_missing
    .quad _class_interfaces_81
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
    .quad _class_interfaces_97
    .quad _class_interfaces_missing
    .quad _class_interfaces_99
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_5
    .quad _class_json_desc_6
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_12
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_20
    .quad _class_json_desc_21
    .quad _class_json_desc_22
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_29
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
    .quad _class_json_desc_55
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
    .quad _class_json_desc_70
    .quad _class_json_desc_missing
    .quad _class_json_desc_72
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_79
    .quad _class_json_desc_missing
    .quad _class_json_desc_81
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
    .quad _class_json_desc_97
    .quad _class_json_desc_missing
    .quad _class_json_desc_99
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 79
.globl _class_parent_ids
_class_parent_ids:
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
    .quad 6
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 5
    .quad -1
    .quad 21
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
    .quad 21
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
    .quad 21
    .quad -1
    .quad 6
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 20
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 21
    .quad -1
    .quad 20
.globl _class_object_payload_sizes
_class_object_payload_sizes:
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
    .quad 72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 72
    .quad 72
    .quad 72
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 72
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 100
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_5
    .quad _class_gc_desc_6
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_12
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_20
    .quad _class_gc_desc_21
    .quad _class_gc_desc_22
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_29
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
    .quad _class_gc_desc_55
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
    .quad _class_gc_desc_70
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_72
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_79
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_81
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
    .quad _class_gc_desc_97
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_99
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_5
    .quad _class_vtable_6
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_12
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_20
    .quad _class_vtable_21
    .quad _class_vtable_22
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_29
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
    .quad _class_vtable_55
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
    .quad _class_vtable_70
    .quad _class_vtable_missing
    .quad _class_vtable_72
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_79
    .quad _class_vtable_missing
    .quad _class_vtable_81
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
    .quad _class_vtable_97
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
.globl _class_clone_count
_class_clone_count:
    .quad 100
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
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_5
    .quad _class_propinit_6
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_12
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_20
    .quad _class_propinit_21
    .quad _class_propinit_22
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_55
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_70
    .quad 0
    .quad _class_propinit_72
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_79
    .quad 0
    .quad _class_propinit_81
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_97
    .quad 0
    .quad _class_propinit_99
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_5
    .quad _class_serprop_6
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_12
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_20
    .quad _class_serprop_21
    .quad _class_serprop_22
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_29
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
    .quad _class_serprop_55
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
    .quad _class_serprop_70
    .quad _class_serprop_missing
    .quad _class_serprop_72
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_79
    .quad _class_serprop_missing
    .quad _class_serprop_81
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
    .quad _class_serprop_97
    .quad _class_serprop_missing
    .quad _class_serprop_99
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_5
    .quad _class_static_vtable_6
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_12
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_20
    .quad _class_static_vtable_21
    .quad _class_static_vtable_22
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_29
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
    .quad _class_static_vtable_55
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
    .quad _class_static_vtable_70
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_72
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_79
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_81
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
    .quad _class_static_vtable_97
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_99
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_5
    .quad _class_callable_methods_6
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_12
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_20
    .quad _class_callable_methods_21
    .quad _class_callable_methods_22
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_29
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
    .quad _class_callable_methods_55
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
    .quad _class_callable_methods_70
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_72
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_79
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_81
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
    .quad _class_callable_methods_97
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
.globl _class_by_name_str_5
_class_by_name_str_5:
    .ascii "Exception"
.globl _class_by_name_str_6
_class_by_name_str_6:
    .ascii "LogicException"
.globl _class_by_name_str_12
_class_by_name_str_12:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_20
_class_by_name_str_20:
    .ascii "RuntimeException"
.globl _class_by_name_str_21
_class_by_name_str_21:
    .ascii "Error"
.globl _class_by_name_str_22
_class_by_name_str_22:
    .ascii "TypeError"
.globl _class_by_name_str_29
_class_by_name_str_29:
    .ascii "stdClass"
.globl _class_by_name_str_55
_class_by_name_str_55:
    .ascii "ValueError"
.globl _class_by_name_str_70
_class_by_name_str_70:
    .ascii "ArithmeticError"
.globl _class_by_name_str_72
_class_by_name_str_72:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_79
_class_by_name_str_79:
    .ascii "JsonException"
.globl _class_by_name_str_81
_class_by_name_str_81:
    .ascii "ReflectionException"
.globl _class_by_name_str_97
_class_by_name_str_97:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_99
_class_by_name_str_99:
    .ascii "OutOfBoundsException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 14
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_5
    .quad 9
    .quad 5
    .quad 72
    .quad _class_by_name_str_6
    .quad 14
    .quad 6
    .quad 72
    .quad _class_by_name_str_12
    .quad 19
    .quad 12
    .quad 72
    .quad _class_by_name_str_20
    .quad 16
    .quad 20
    .quad 72
    .quad _class_by_name_str_21
    .quad 5
    .quad 21
    .quad 72
    .quad _class_by_name_str_22
    .quad 9
    .quad 22
    .quad 72
    .quad _class_by_name_str_29
    .quad 8
    .quad 29
    .quad 16
    .quad _class_by_name_str_55
    .quad 10
    .quad 55
    .quad 72
    .quad _class_by_name_str_70
    .quad 15
    .quad 70
    .quad 72
    .quad _class_by_name_str_72
    .quad 24
    .quad 72
    .quad 72
    .quad _class_by_name_str_79
    .quad 13
    .quad 79
    .quad 72
    .quad _class_by_name_str_81
    .quad 19
    .quad 81
    .quad 72
    .quad _class_by_name_str_97
    .quad 19
    .quad 97
    .quad 72
    .quad _class_by_name_str_99
    .quad 20
    .quad 99
    .quad 72
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
.globl _interface_methods_9
_interface_methods_9:
    .quad 1
    .quad 0
.globl _interface_methods_12
_interface_methods_12:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _class_interfaces_5
_class_interfaces_5:
    .quad 2
    .quad 12
    .quad _class_interface_impl_5_12
    .quad 9
    .quad _class_interface_impl_5_9
.globl _class_interface_impl_5_12
_class_interface_impl_5_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_5_9
_class_interface_impl_5_9:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_5_0
_class_serpname_5_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_5_1
_class_serpname_5_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_5_2
_class_serpname_5_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_5_3
_class_serpname_5_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_5
_class_serprop_5:
    .quad 4
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
    .quad _class_serpname_5_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_5
_class_vtable_5:
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
.globl _class_interfaces_6
_class_interfaces_6:
    .quad 2
    .quad 12
    .quad _class_interface_impl_6_12
    .quad 9
    .quad _class_interface_impl_6_9
.globl _class_interface_impl_6_12
_class_interface_impl_6_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_6_9
_class_interface_impl_6_9:
    .quad 0
.globl _class_json_pname_6_0
_class_json_pname_6_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_6
_class_json_desc_6:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_6_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_6
_class_gc_desc_6:
    .byte 1, 0, 7, 4
.globl _class_serpname_6_0
_class_serpname_6_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_6_1
_class_serpname_6_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_6_2
_class_serpname_6_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_6_3
_class_serpname_6_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_6
_class_serprop_6:
    .quad 4
    .quad _class_serpname_6_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_6_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_6_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_6_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_6
_class_vtable_6:
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
.globl _class_static_vtable_6
_class_static_vtable_6:
    .quad 0
.globl _class_callable_method_name_6__u__u_construct
_class_callable_method_name_6__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_6__u__u_tostring
_class_callable_method_name_6__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_6_getcode
_class_callable_method_name_6_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_6_getfile
_class_callable_method_name_6_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_6_getline
_class_callable_method_name_6_getline:
    .ascii "getline"
.globl _class_callable_method_name_6_getmessage
_class_callable_method_name_6_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_6_getprevious
_class_callable_method_name_6_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_6_gettrace
_class_callable_method_name_6_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_6_gettraceasstring
_class_callable_method_name_6_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_6
_class_callable_methods_6:
    .quad 9
    .quad _class_callable_method_name_6__u__u_construct
    .quad 11
    .quad _class_callable_method_name_6__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_6_getcode
    .quad 7
    .quad _class_callable_method_name_6_getfile
    .quad 7
    .quad _class_callable_method_name_6_getline
    .quad 7
    .quad _class_callable_method_name_6_getmessage
    .quad 10
    .quad _class_callable_method_name_6_getprevious
    .quad 11
    .quad _class_callable_method_name_6_gettrace
    .quad 8
    .quad _class_callable_method_name_6_gettraceasstring
    .quad 16
.globl _class_interfaces_12
_class_interfaces_12:
    .quad 2
    .quad 12
    .quad _class_interface_impl_12_12
    .quad 9
    .quad _class_interface_impl_12_9
.globl _class_interface_impl_12_12
_class_interface_impl_12_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_12_9
_class_interface_impl_12_9:
    .quad 0
.globl _class_json_pname_12_0
_class_json_pname_12_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_12
_class_json_desc_12:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_12_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_12
_class_gc_desc_12:
    .byte 1, 0, 7, 4
.globl _class_serpname_12_0
_class_serpname_12_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_12_1
_class_serpname_12_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_12_2
_class_serpname_12_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_12_3
_class_serpname_12_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_12
_class_serprop_12:
    .quad 4
    .quad _class_serpname_12_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_12_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_12_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_12_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_12
_class_vtable_12:
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
.globl _class_static_vtable_12
_class_static_vtable_12:
    .quad 0
.globl _class_callable_method_name_12__u__u_construct
_class_callable_method_name_12__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_12__u__u_tostring
_class_callable_method_name_12__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_12_getcode
_class_callable_method_name_12_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_12_getfile
_class_callable_method_name_12_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_12_getline
_class_callable_method_name_12_getline:
    .ascii "getline"
.globl _class_callable_method_name_12_getmessage
_class_callable_method_name_12_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_12_getprevious
_class_callable_method_name_12_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_12_gettrace
_class_callable_method_name_12_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_12_gettraceasstring
_class_callable_method_name_12_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_12
_class_callable_methods_12:
    .quad 9
    .quad _class_callable_method_name_12__u__u_construct
    .quad 11
    .quad _class_callable_method_name_12__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_12_getcode
    .quad 7
    .quad _class_callable_method_name_12_getfile
    .quad 7
    .quad _class_callable_method_name_12_getline
    .quad 7
    .quad _class_callable_method_name_12_getmessage
    .quad 10
    .quad _class_callable_method_name_12_getprevious
    .quad 11
    .quad _class_callable_method_name_12_gettrace
    .quad 8
    .quad _class_callable_method_name_12_gettraceasstring
    .quad 16
.globl _class_interfaces_20
_class_interfaces_20:
    .quad 2
    .quad 12
    .quad _class_interface_impl_20_12
    .quad 9
    .quad _class_interface_impl_20_9
.globl _class_interface_impl_20_12
_class_interface_impl_20_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_20_9
_class_interface_impl_20_9:
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
.globl _class_interfaces_21
_class_interfaces_21:
    .quad 2
    .quad 12
    .quad _class_interface_impl_21_12
    .quad 9
    .quad _class_interface_impl_21_9
.globl _class_interface_impl_21_12
_class_interface_impl_21_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_21_9
_class_interface_impl_21_9:
    .quad 0
.globl _class_json_pname_21_0
_class_json_pname_21_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_21
_class_json_desc_21:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_21_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_21
_class_gc_desc_21:
    .byte 1, 0, 7, 4
.globl _class_serpname_21_0
_class_serpname_21_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_21_1
_class_serpname_21_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_21_2
_class_serpname_21_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_21_3
_class_serpname_21_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_21
_class_serprop_21:
    .quad 4
    .quad _class_serpname_21_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_21_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_21_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_21_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_21
_class_vtable_21:
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
.globl _class_static_vtable_21
_class_static_vtable_21:
    .quad 0
.globl _class_callable_method_name_21__u__u_construct
_class_callable_method_name_21__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_21__u__u_tostring
_class_callable_method_name_21__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_21_getcode
_class_callable_method_name_21_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_21_getfile
_class_callable_method_name_21_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_21_getline
_class_callable_method_name_21_getline:
    .ascii "getline"
.globl _class_callable_method_name_21_getmessage
_class_callable_method_name_21_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_21_getprevious
_class_callable_method_name_21_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_21_gettrace
_class_callable_method_name_21_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_21_gettraceasstring
_class_callable_method_name_21_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_21
_class_callable_methods_21:
    .quad 9
    .quad _class_callable_method_name_21__u__u_construct
    .quad 11
    .quad _class_callable_method_name_21__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_21_getcode
    .quad 7
    .quad _class_callable_method_name_21_getfile
    .quad 7
    .quad _class_callable_method_name_21_getline
    .quad 7
    .quad _class_callable_method_name_21_getmessage
    .quad 10
    .quad _class_callable_method_name_21_getprevious
    .quad 11
    .quad _class_callable_method_name_21_gettrace
    .quad 8
    .quad _class_callable_method_name_21_gettraceasstring
    .quad 16
.globl _class_interfaces_22
_class_interfaces_22:
    .quad 2
    .quad 12
    .quad _class_interface_impl_22_12
    .quad 9
    .quad _class_interface_impl_22_9
.globl _class_interface_impl_22_12
_class_interface_impl_22_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_22_9
_class_interface_impl_22_9:
    .quad 0
.globl _class_json_pname_22_0
_class_json_pname_22_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_22
_class_json_desc_22:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_22_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_22
_class_gc_desc_22:
    .byte 1, 0, 7, 4
.globl _class_serpname_22_0
_class_serpname_22_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_22_1
_class_serpname_22_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_22_2
_class_serpname_22_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_22_3
_class_serpname_22_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_22
_class_serprop_22:
    .quad 4
    .quad _class_serpname_22_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_22_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_22_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_22_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_22
_class_vtable_22:
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
.globl _class_static_vtable_22
_class_static_vtable_22:
    .quad 0
.globl _class_callable_method_name_22__u__u_construct
_class_callable_method_name_22__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_22__u__u_tostring
_class_callable_method_name_22__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_22_getcode
_class_callable_method_name_22_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_22_getfile
_class_callable_method_name_22_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_22_getline
_class_callable_method_name_22_getline:
    .ascii "getline"
.globl _class_callable_method_name_22_getmessage
_class_callable_method_name_22_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_22_getprevious
_class_callable_method_name_22_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_22_gettrace
_class_callable_method_name_22_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_22_gettraceasstring
_class_callable_method_name_22_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_22
_class_callable_methods_22:
    .quad 9
    .quad _class_callable_method_name_22__u__u_construct
    .quad 11
    .quad _class_callable_method_name_22__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_22_getcode
    .quad 7
    .quad _class_callable_method_name_22_getfile
    .quad 7
    .quad _class_callable_method_name_22_getline
    .quad 7
    .quad _class_callable_method_name_22_getmessage
    .quad 10
    .quad _class_callable_method_name_22_getprevious
    .quad 11
    .quad _class_callable_method_name_22_gettrace
    .quad 8
    .quad _class_callable_method_name_22_gettraceasstring
    .quad 16
.globl _class_interfaces_29
_class_interfaces_29:
    .quad 0
    .p2align 3
.globl _class_json_desc_29
_class_json_desc_29:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_29
_class_gc_desc_29:
    .byte 0
    .p2align 3
.globl _class_serprop_29
_class_serprop_29:
    .quad 0
    .p2align 3
.globl _class_vtable_29
_class_vtable_29:
    .quad 0
    .p2align 3
.globl _class_static_vtable_29
_class_static_vtable_29:
    .quad 0
.p2align 3
.globl _class_callable_methods_29
_class_callable_methods_29:
    .quad 0
.globl _class_interfaces_55
_class_interfaces_55:
    .quad 2
    .quad 12
    .quad _class_interface_impl_55_12
    .quad 9
    .quad _class_interface_impl_55_9
.globl _class_interface_impl_55_12
_class_interface_impl_55_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_55_9
_class_interface_impl_55_9:
    .quad 0
.globl _class_json_pname_55_0
_class_json_pname_55_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_55
_class_json_desc_55:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_55_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_55
_class_gc_desc_55:
    .byte 1, 0, 7, 4
.globl _class_serpname_55_0
_class_serpname_55_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_55_1
_class_serpname_55_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_55_2
_class_serpname_55_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_55_3
_class_serpname_55_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_55
_class_serprop_55:
    .quad 4
    .quad _class_serpname_55_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_55_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_55_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_55_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_55
_class_vtable_55:
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
.globl _class_static_vtable_55
_class_static_vtable_55:
    .quad 0
.globl _class_callable_method_name_55__u__u_construct
_class_callable_method_name_55__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_55__u__u_tostring
_class_callable_method_name_55__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_55_getcode
_class_callable_method_name_55_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_55_getfile
_class_callable_method_name_55_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_55_getline
_class_callable_method_name_55_getline:
    .ascii "getline"
.globl _class_callable_method_name_55_getmessage
_class_callable_method_name_55_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_55_getprevious
_class_callable_method_name_55_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_55_gettrace
_class_callable_method_name_55_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_55_gettraceasstring
_class_callable_method_name_55_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_55
_class_callable_methods_55:
    .quad 9
    .quad _class_callable_method_name_55__u__u_construct
    .quad 11
    .quad _class_callable_method_name_55__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_55_getcode
    .quad 7
    .quad _class_callable_method_name_55_getfile
    .quad 7
    .quad _class_callable_method_name_55_getline
    .quad 7
    .quad _class_callable_method_name_55_getmessage
    .quad 10
    .quad _class_callable_method_name_55_getprevious
    .quad 11
    .quad _class_callable_method_name_55_gettrace
    .quad 8
    .quad _class_callable_method_name_55_gettraceasstring
    .quad 16
.globl _class_interfaces_70
_class_interfaces_70:
    .quad 2
    .quad 12
    .quad _class_interface_impl_70_12
    .quad 9
    .quad _class_interface_impl_70_9
.globl _class_interface_impl_70_12
_class_interface_impl_70_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_70_9
_class_interface_impl_70_9:
    .quad 0
.globl _class_json_pname_70_0
_class_json_pname_70_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_70
_class_json_desc_70:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_70_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_70
_class_gc_desc_70:
    .byte 1, 0, 7, 4
.globl _class_serpname_70_0
_class_serpname_70_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_70_1
_class_serpname_70_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_70_2
_class_serpname_70_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_70_3
_class_serpname_70_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_70
_class_serprop_70:
    .quad 4
    .quad _class_serpname_70_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_70_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_70_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_70_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_70
_class_vtable_70:
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
.globl _class_static_vtable_70
_class_static_vtable_70:
    .quad 0
.globl _class_callable_method_name_70__u__u_construct
_class_callable_method_name_70__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_70__u__u_tostring
_class_callable_method_name_70__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_70_getcode
_class_callable_method_name_70_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_70_getfile
_class_callable_method_name_70_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_70_getline
_class_callable_method_name_70_getline:
    .ascii "getline"
.globl _class_callable_method_name_70_getmessage
_class_callable_method_name_70_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_70_getprevious
_class_callable_method_name_70_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_70_gettrace
_class_callable_method_name_70_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_70_gettraceasstring
_class_callable_method_name_70_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_70
_class_callable_methods_70:
    .quad 9
    .quad _class_callable_method_name_70__u__u_construct
    .quad 11
    .quad _class_callable_method_name_70__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_70_getcode
    .quad 7
    .quad _class_callable_method_name_70_getfile
    .quad 7
    .quad _class_callable_method_name_70_getline
    .quad 7
    .quad _class_callable_method_name_70_getmessage
    .quad 10
    .quad _class_callable_method_name_70_getprevious
    .quad 11
    .quad _class_callable_method_name_70_gettrace
    .quad 8
    .quad _class_callable_method_name_70_gettraceasstring
    .quad 16
.globl _class_interfaces_72
_class_interfaces_72:
    .quad 2
    .quad 12
    .quad _class_interface_impl_72_12
    .quad 9
    .quad _class_interface_impl_72_9
.globl _class_interface_impl_72_12
_class_interface_impl_72_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_72_9
_class_interface_impl_72_9:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_72_0
_class_serpname_72_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_72_1
_class_serpname_72_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_72_2
_class_serpname_72_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_72_3
_class_serpname_72_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_72
_class_serprop_72:
    .quad 4
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
    .quad _class_serpname_72_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_72
_class_vtable_72:
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
.globl _class_interfaces_79
_class_interfaces_79:
    .quad 2
    .quad 12
    .quad _class_interface_impl_79_12
    .quad 9
    .quad _class_interface_impl_79_9
.globl _class_interface_impl_79_12
_class_interface_impl_79_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_79_9
_class_interface_impl_79_9:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_79
_class_vtable_79:
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
.globl _class_interfaces_81
_class_interfaces_81:
    .quad 2
    .quad 12
    .quad _class_interface_impl_81_12
    .quad 9
    .quad _class_interface_impl_81_9
.globl _class_interface_impl_81_12
_class_interface_impl_81_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_81_9
_class_interface_impl_81_9:
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
.globl _class_interfaces_97
_class_interfaces_97:
    .quad 2
    .quad 12
    .quad _class_interface_impl_97_12
    .quad 9
    .quad _class_interface_impl_97_9
.globl _class_interface_impl_97_12
_class_interface_impl_97_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_97_9
_class_interface_impl_97_9:
    .quad 0
.globl _class_json_pname_97_0
_class_json_pname_97_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_97
_class_json_desc_97:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_97_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_97
_class_gc_desc_97:
    .byte 1, 0, 7, 4
.globl _class_serpname_97_0
_class_serpname_97_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_97_1
_class_serpname_97_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_97_2
_class_serpname_97_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_97_3
_class_serpname_97_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_97
_class_serprop_97:
    .quad 4
    .quad _class_serpname_97_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_97_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_97_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_97_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_97
_class_vtable_97:
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
.globl _class_static_vtable_97
_class_static_vtable_97:
    .quad 0
.globl _class_callable_method_name_97__u__u_construct
_class_callable_method_name_97__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_97__u__u_tostring
_class_callable_method_name_97__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_97_getcode
_class_callable_method_name_97_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_97_getfile
_class_callable_method_name_97_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_97_getline
_class_callable_method_name_97_getline:
    .ascii "getline"
.globl _class_callable_method_name_97_getmessage
_class_callable_method_name_97_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_97_getprevious
_class_callable_method_name_97_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_97_gettrace
_class_callable_method_name_97_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_97_gettraceasstring
_class_callable_method_name_97_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_97
_class_callable_methods_97:
    .quad 9
    .quad _class_callable_method_name_97__u__u_construct
    .quad 11
    .quad _class_callable_method_name_97__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_97_getcode
    .quad 7
    .quad _class_callable_method_name_97_getfile
    .quad 7
    .quad _class_callable_method_name_97_getline
    .quad 7
    .quad _class_callable_method_name_97_getmessage
    .quad 10
    .quad _class_callable_method_name_97_getprevious
    .quad 11
    .quad _class_callable_method_name_97_gettrace
    .quad 8
    .quad _class_callable_method_name_97_gettraceasstring
    .quad 16
.globl _class_interfaces_99
_class_interfaces_99:
    .quad 2
    .quad 12
    .quad _class_interface_impl_99_12
    .quad 9
    .quad _class_interface_impl_99_9
.globl _class_interface_impl_99_12
_class_interface_impl_99_12:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_99_9
_class_interface_impl_99_9:
    .quad 0
.globl _class_json_pname_99_0
_class_json_pname_99_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_99
_class_json_desc_99:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_99_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_99
_class_gc_desc_99:
    .byte 1, 0, 7, 4
.globl _class_serpname_99_0
_class_serpname_99_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_99_1
_class_serpname_99_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_99_2
_class_serpname_99_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_99_3
_class_serpname_99_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_99
_class_serprop_99:
    .quad 4
    .quad _class_serpname_99_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_99_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_99_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_99_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_99
_class_vtable_99:
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
.globl _class_static_vtable_99
_class_static_vtable_99:
    .quad 0
.globl _class_callable_method_name_99__u__u_construct
_class_callable_method_name_99__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_99__u__u_tostring
_class_callable_method_name_99__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_99_getcode
_class_callable_method_name_99_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_99_getfile
_class_callable_method_name_99_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_99_getline
_class_callable_method_name_99_getline:
    .ascii "getline"
.globl _class_callable_method_name_99_getmessage
_class_callable_method_name_99_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_99_getprevious
_class_callable_method_name_99_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_99_gettrace
_class_callable_method_name_99_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_99_gettraceasstring
_class_callable_method_name_99_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_99
_class_callable_methods_99:
    .quad 9
    .quad _class_callable_method_name_99__u__u_construct
    .quad 11
    .quad _class_callable_method_name_99__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_99_getcode
    .quad 7
    .quad _class_callable_method_name_99_getfile
    .quad 7
    .quad _class_callable_method_name_99_getline
    .quad 7
    .quad _class_callable_method_name_99_getmessage
    .quad 10
    .quad _class_callable_method_name_99_getprevious
    .quad 11
    .quad _class_callable_method_name_99_gettrace
    .quad 8
    .quad _class_callable_method_name_99_gettraceasstring
    .quad 16
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 29

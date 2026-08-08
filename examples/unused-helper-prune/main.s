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
_eir__class_propinit_2_entry_0:
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
_class_propinit_2_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_2
    ; @fn name=_class_propinit_3 symbol=_class_propinit_3 synthetic=1
.align 2

.globl _class_propinit_3
_class_propinit_3:
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
_eir__class_propinit_3_entry_0:
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
_class_propinit_3_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    ; @fn name=_class_propinit_14 symbol=_class_propinit_14 synthetic=1
.align 2

.globl _class_propinit_14
_class_propinit_14:
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
_eir__class_propinit_14_entry_0:
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
_class_propinit_14_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_14
    ; @fn name=_class_propinit_15 symbol=_class_propinit_15 synthetic=1
.align 2

.globl _class_propinit_15
_class_propinit_15:
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
_eir__class_propinit_15_entry_0:
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
_class_propinit_15_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_15
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
    ; @fn name=_class_propinit_18 symbol=_class_propinit_18 synthetic=1
.align 2

.globl _class_propinit_18
_class_propinit_18:
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
_eir__class_propinit_18_entry_0:
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
_class_propinit_18_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_19_entry_0:
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
_class_propinit_19_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
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
    ; @fn name=_class_propinit_28 symbol=_class_propinit_28 synthetic=1
.align 2

.globl _class_propinit_28
_class_propinit_28:
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
_eir__class_propinit_28_entry_0:
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
_class_propinit_28_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_28
    ; @fn name=_class_propinit_29 symbol=_class_propinit_29 synthetic=1
.align 2

.globl _class_propinit_29
_class_propinit_29:
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
_eir__class_propinit_29_entry_0:
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
_class_propinit_29_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_29
    ; @fn name=_class_propinit_30 symbol=_class_propinit_30 synthetic=1
.align 2

.globl _class_propinit_30
_class_propinit_30:
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
_eir__class_propinit_30_entry_0:
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
_class_propinit_30_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_30
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
    ; @fn name=_class_propinit_33 symbol=_class_propinit_33 synthetic=1
.align 2

.globl _class_propinit_33
_class_propinit_33:
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
_eir__class_propinit_33_entry_0:
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
_class_propinit_33_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
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
_eir__class_propinit_39_entry_0:
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
_class_propinit_39_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_39
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
    ; @fn name=_class_propinit_44 symbol=_class_propinit_44 synthetic=1
.align 2

.globl _class_propinit_44
_class_propinit_44:
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
_eir__class_propinit_44_entry_0:
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
_class_propinit_44_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_44
    ; @fn name=_class_propinit_45 symbol=_class_propinit_45 synthetic=1
.align 2

.globl _class_propinit_45
_class_propinit_45:
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
_eir__class_propinit_45_entry_0:
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
_class_propinit_45_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_45
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
_eir__class_propinit_51_entry_0:
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
_class_propinit_51_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_51
    ; @fn name=_class_propinit_52 symbol=_class_propinit_52 synthetic=1
.align 2

.globl _class_propinit_52
_class_propinit_52:
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
_eir__class_propinit_52_entry_0:
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
_class_propinit_52_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_52
    ; @fn name=_class_propinit_53 symbol=_class_propinit_53 synthetic=1
.align 2

.globl _class_propinit_53
_class_propinit_53:
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
_eir__class_propinit_53_entry_0:
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
_class_propinit_53_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_53
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
    ; @fn name=_class_propinit_57 symbol=_class_propinit_57 synthetic=1
.align 2

.globl _class_propinit_57
_class_propinit_57:
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
_eir__class_propinit_57_entry_0:
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
_class_propinit_57_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_57
    ; @fn name=_class_propinit_58 symbol=_class_propinit_58 synthetic=1
.align 2

.globl _class_propinit_58
_class_propinit_58:
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
_eir__class_propinit_58_entry_0:
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
_class_propinit_58_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_58
    ; @fn name=_class_propinit_60 symbol=_class_propinit_60 synthetic=1
.align 2

.globl _class_propinit_60
_class_propinit_60:
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
_eir__class_propinit_60_entry_0:
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
_class_propinit_60_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_60
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
_eir__class_propinit_69_entry_0:
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
_class_propinit_69_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_69
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
_eir__class_propinit_71_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_71_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_71
    ; @fn name=_class_propinit_76 symbol=_class_propinit_76 synthetic=1
.align 2

.globl _class_propinit_76
_class_propinit_76:
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
_eir__class_propinit_76_entry_0:
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
_class_propinit_76_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_76
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
    ; @fn name=_class_propinit_80 symbol=_class_propinit_80 synthetic=1
.align 2

.globl _class_propinit_80
_class_propinit_80:
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
_eir__class_propinit_80_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_80_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_80
    ; @fn name=_class_propinit_85 symbol=_class_propinit_85 synthetic=1
.align 2

.globl _class_propinit_85
_class_propinit_85:
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
_eir__class_propinit_85_entry_0:
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
_class_propinit_85_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_85
    ; @fn name=_class_propinit_89 symbol=_class_propinit_89 synthetic=1
.align 2

.globl _class_propinit_89
_class_propinit_89:
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
_eir__class_propinit_89_entry_0:
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
_class_propinit_89_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_89
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
    ; @fn name=App\Greeter::hello symbol=_method_App_N_Greeter_hello
.align 2

.globl _method_App_N_Greeter_hello
_method_App_N_Greeter_hello:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; param $name from x1,x2
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @block name=entry
_eir_App_Greeter__hello_entry_0:
    ; @src line=10 col=9 end=10:15 op=concat_reset
    ldur x10, [x29, #-128]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=16 op=const_str
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=10 col=16 op=load_local
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=10 col=16 op=str_concat
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    bl __rt_concat
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=10 col=16 op=const_str
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=10 col=16 op=str_concat
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldur x3, [x29, #-64]
    ldur x4, [x29, #-56]
    bl __rt_concat
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ; @src line=10 col=16 op=release
    ; @src line=10 col=9 end=10:15 op=str_persist
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    bl __rt_str_persist
    stur x1, [x29, #-96]
    stur x2, [x29, #-88]
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_method_App_N_Greeter_hello_epilogue:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=App\Greeter::hello
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-128]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    stur xzr, [x29, #-120]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=6 col=1 end=6:6 op=concat_reset
    ldur x10, [x29, #-128]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=1 end=6:6 op=nop
    ; @src line=8 col=1 end=8:3 op=concat_reset
    ldur x10, [x29, #-128]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=6 end=8:19 op=object_new
    mov x0, #8
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    mov x10, #47
    str x10, [x0]
    stur x0, [x29, #-8]
    ; @src line=8 col=1 end=8:3 op=acquire
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-16]
    ; @src line=8 col=1 end=8:3 op=store_local
    ldur x0, [x29, #-16]
    stur x0, [x29, #-120]
    ; @src line=8 col=1 end=8:3 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_object
    ; @src line=9 col=1 end=9:5 op=concat_reset
    ldur x10, [x29, #-128]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=6 end=9:8 op=load_local
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    ; @src line=9 col=16 end=9:23 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=9 col=8 end=9:24 op=method_call
    ldur x9, [x29, #-24]
    cbz x9, _eir_main_static_method_receiver_null_0
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_static_method_receiver_null_0
    b _eir_main_static_method_receiver_checked_1
_eir_main_static_method_receiver_null_0:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_2
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #71
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
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_7@PAGE
    add x9, x9, _str_7@PAGEOFF
    str x9, [x0, #8]
    mov x9, #41
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_static_method_receiver_checked_1:
    ldur x0, [x29, #-24]
    str x0, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    ldr x2, [sp, #8]
    add sp, sp, #32
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    stur x1, [x29, #-56]
    stur x2, [x29, #-48]
    ; @src line=9 col=8 end=9:24 op=nop
    ; @src line=9 col=27 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=9 col=25 end=9:27 op=str_concat
    ldur x1, [x29, #-56]
    ldur x2, [x29, #-48]
    ldur x3, [x29, #-72]
    ldur x4, [x29, #-64]
    bl __rt_concat
    stur x1, [x29, #-88]
    stur x2, [x29, #-80]
    ; @src line=9 col=25 end=9:27 op=release
    ldur x1, [x29, #-56]
    ldur x2, [x29, #-48]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=9 col=1 end=9:5 op=echo_value
    ldur x1, [x29, #-88]
    ldur x2, [x29, #-80]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=9 col=1 end=9:5 op=release
    ; @src line=10 col=1 end=10:5 op=concat_reset
    ldur x10, [x29, #-128]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=6 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=10 col=1 end=10:5 op=echo_value
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $g
    ldur x0, [x29, #-120]
    cbz x0, _eir_main_main_refcounted_cleanup_done_3
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_3:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_4
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_4:
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
    .ascii "Hello, "
.globl _str_4
_str_4:
    .ascii "!"
.globl _str_5
_str_5:
    .ascii "world"
.globl _str_6
_str_6:
    .ascii "Fatal error: Uncaught Error: Call to a member function hello() on null\n"
.globl _str_7
_str_7:
    .ascii "Call to a member function hello() on null"
.globl _str_8
_str_8:
    .ascii "\n"
.globl _str_9
_str_9:
    .ascii "done\n"
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
    .quad 34
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
    .quad 14
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 15
    .quad 1
    .quad 0
    .quad _instanceof_name_class_4
    .quad 16
    .quad 4
    .quad 0
    .quad _instanceof_name_class_abs_4
    .quad 17
    .quad 4
    .quad 0
    .quad _instanceof_name_class_6
    .quad 5
    .quad 6
    .quad 0
    .quad _instanceof_name_class_abs_6
    .quad 6
    .quad 6
    .quad 0
    .quad _instanceof_name_class_21
    .quad 19
    .quad 21
    .quad 0
    .quad _instanceof_name_class_abs_21
    .quad 20
    .quad 21
    .quad 0
    .quad _instanceof_name_class_46
    .quad 9
    .quad 46
    .quad 0
    .quad _instanceof_name_class_abs_46
    .quad 10
    .quad 46
    .quad 0
    .quad _instanceof_name_class_47
    .quad 11
    .quad 47
    .quad 0
    .quad _instanceof_name_class_abs_47
    .quad 12
    .quad 47
    .quad 0
    .quad _instanceof_name_class_54
    .quad 24
    .quad 54
    .quad 0
    .quad _instanceof_name_class_abs_54
    .quad 25
    .quad 54
    .quad 0
    .quad _instanceof_name_class_57
    .quad 13
    .quad 57
    .quad 0
    .quad _instanceof_name_class_abs_57
    .quad 14
    .quad 57
    .quad 0
    .quad _instanceof_name_class_58
    .quad 19
    .quad 58
    .quad 0
    .quad _instanceof_name_class_abs_58
    .quad 20
    .quad 58
    .quad 0
    .quad _instanceof_name_class_76
    .quad 20
    .quad 76
    .quad 0
    .quad _instanceof_name_class_abs_76
    .quad 21
    .quad 76
    .quad 0
    .quad _instanceof_name_class_77
    .quad 15
    .quad 77
    .quad 0
    .quad _instanceof_name_class_abs_77
    .quad 16
    .quad 77
    .quad 0
    .quad _instanceof_name_class_89
    .quad 19
    .quad 89
    .quad 0
    .quad _instanceof_name_class_abs_89
    .quad 20
    .quad 89
    .quad 0
    .quad _instanceof_name_class_91
    .quad 10
    .quad 91
    .quad 0
    .quad _instanceof_name_class_abs_91
    .quad 11
    .quad 91
    .quad 0
    .quad _instanceof_name_class_96
    .quad 8
    .quad 96
    .quad 0
    .quad _instanceof_name_class_abs_96
    .quad 9
    .quad 96
    .quad 0
    .quad _instanceof_name_interface_10
    .quad 10
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_abs_10
    .quad 11
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_14
    .quad 9
    .quad 14
    .quad 1
    .quad _instanceof_name_interface_abs_14
    .quad 10
    .quad 14
    .quad 1
.globl _instanceof_name_class_0
_instanceof_name_class_0:
    .ascii "Exception"
.globl _instanceof_name_class_abs_0
_instanceof_name_class_abs_0:
    .ascii "\\Exception"
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\LogicException"
.globl _instanceof_name_class_4
_instanceof_name_class_4:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_4
_instanceof_name_class_abs_4:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_6
_instanceof_name_class_6:
    .ascii "Error"
.globl _instanceof_name_class_abs_6
_instanceof_name_class_abs_6:
    .ascii "\\Error"
.globl _instanceof_name_class_21
_instanceof_name_class_21:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_21
_instanceof_name_class_abs_21:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_46
_instanceof_name_class_46:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_46
_instanceof_name_class_abs_46:
    .ascii "\\TypeError"
.globl _instanceof_name_class_47
_instanceof_name_class_47:
    .ascii "App\\Greeter"
.globl _instanceof_name_class_abs_47
_instanceof_name_class_abs_47:
    .ascii "\\App\\Greeter"
.globl _instanceof_name_class_54
_instanceof_name_class_54:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_54
_instanceof_name_class_abs_54:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_57
_instanceof_name_class_57:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_57
_instanceof_name_class_abs_57:
    .ascii "\\JsonException"
.globl _instanceof_name_class_58
_instanceof_name_class_58:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_58
_instanceof_name_class_abs_58:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_76
_instanceof_name_class_76:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_76
_instanceof_name_class_abs_76:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_77
_instanceof_name_class_77:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_77
_instanceof_name_class_abs_77:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_89
_instanceof_name_class_89:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_89
_instanceof_name_class_abs_89:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_91
_instanceof_name_class_91:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_91
_instanceof_name_class_abs_91:
    .ascii "\\ValueError"
.globl _instanceof_name_class_96
_instanceof_name_class_96:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_96
_instanceof_name_class_abs_96:
    .ascii "\\stdClass"
.globl _instanceof_name_interface_10
_instanceof_name_interface_10:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_10
_instanceof_name_interface_abs_10:
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
    .quad 97
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_0
    .quad 9
    .quad _class_name_1
    .quad 14
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_4
    .quad 16
    .quad _class_name_missing
    .quad 0
    .quad _class_name_6
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
    .quad _class_name_21
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
    .quad _class_name_46
    .quad 9
    .quad _class_name_47
    .quad 11
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
    .quad _class_name_54
    .quad 24
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_57
    .quad 13
    .quad _class_name_58
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
    .quad _class_name_76
    .quad 20
    .quad _class_name_77
    .quad 15
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
    .quad _class_name_89
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_91
    .quad 10
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_96
    .quad 8
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_0
_class_name_0:
    .ascii "Exception"
.globl _class_name_1
_class_name_1:
    .ascii "LogicException"
.globl _class_name_4
_class_name_4:
    .ascii "RuntimeException"
.globl _class_name_6
_class_name_6:
    .ascii "Error"
.globl _class_name_21
_class_name_21:
    .ascii "UnhandledMatchError"
.globl _class_name_46
_class_name_46:
    .ascii "TypeError"
.globl _class_name_47
_class_name_47:
    .ascii "App\\Greeter"
.globl _class_name_54
_class_name_54:
    .ascii "InvalidArgumentException"
.globl _class_name_57
_class_name_57:
    .ascii "JsonException"
.globl _class_name_58
_class_name_58:
    .ascii "ReflectionException"
.globl _class_name_76
_class_name_76:
    .ascii "OutOfBoundsException"
.globl _class_name_77
_class_name_77:
    .ascii "ArithmeticError"
.globl _class_name_89
_class_name_89:
    .ascii "OutOfRangeException"
.globl _class_name_91
_class_name_91:
    .ascii "ValueError"
.globl _class_name_96
_class_name_96:
    .ascii "stdClass"
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
    .quad 60
.globl _generator_class_id
_generator_class_id:
    .quad 72
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 42
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 43
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 73
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 92
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 6
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 1
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 4
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 89
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 76
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 54
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 46
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 91
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 58
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 77
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_10
    .quad _interface_methods_14
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_0
    .quad _class_interfaces_1
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_4
    .quad _class_interfaces_missing
    .quad _class_interfaces_6
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
    .quad _class_interfaces_21
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
    .quad _class_interfaces_46
    .quad _class_interfaces_47
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_54
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_57
    .quad _class_interfaces_58
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
    .quad _class_interfaces_76
    .quad _class_interfaces_77
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
    .quad _class_interfaces_89
    .quad _class_interfaces_missing
    .quad _class_interfaces_91
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_96
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_0
    .quad _class_json_desc_1
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_4
    .quad _class_json_desc_missing
    .quad _class_json_desc_6
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
    .quad _class_json_desc_21
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
    .quad _class_json_desc_46
    .quad _class_json_desc_47
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_54
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_57
    .quad _class_json_desc_58
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
    .quad _class_json_desc_76
    .quad _class_json_desc_77
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
    .quad _class_json_desc_89
    .quad _class_json_desc_missing
    .quad _class_json_desc_91
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_96
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 57
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
    .quad 0
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
    .quad 6
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
    .quad 6
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 1
    .quad -1
    .quad -1
    .quad 4
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
    .quad 4
    .quad 6
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
    .quad 1
    .quad -1
    .quad 6
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
.globl _class_object_payload_sizes
_class_object_payload_sizes:
    .quad 72
    .quad 72
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
    .quad 72
    .quad 8
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
    .quad 72
    .quad 0
    .quad 72
    .quad 0
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
    .quad 1
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 97
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_0
    .quad _class_gc_desc_1
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_4
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_6
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
    .quad _class_gc_desc_21
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
    .quad _class_gc_desc_46
    .quad _class_gc_desc_47
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_54
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_57
    .quad _class_gc_desc_58
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
    .quad _class_gc_desc_76
    .quad _class_gc_desc_77
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
    .quad _class_gc_desc_89
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_91
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_96
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_0
    .quad _class_vtable_1
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_4
    .quad _class_vtable_missing
    .quad _class_vtable_6
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
    .quad _class_vtable_21
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
    .quad _class_vtable_46
    .quad _class_vtable_47
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_54
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_57
    .quad _class_vtable_58
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
    .quad _class_vtable_76
    .quad _class_vtable_77
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
    .quad _class_vtable_89
    .quad _class_vtable_missing
    .quad _class_vtable_91
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_96
.globl _class_destruct_count
_class_destruct_count:
    .quad 97
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
.globl _class_clone_count
_class_clone_count:
    .quad 97
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
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad _class_propinit_0
    .quad _class_propinit_1
    .quad 0
    .quad 0
    .quad _class_propinit_4
    .quad 0
    .quad _class_propinit_6
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_21
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_46
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_54
    .quad 0
    .quad 0
    .quad _class_propinit_57
    .quad _class_propinit_58
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_76
    .quad _class_propinit_77
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_89
    .quad 0
    .quad _class_propinit_91
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_0
    .quad _class_serprop_1
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_4
    .quad _class_serprop_missing
    .quad _class_serprop_6
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
    .quad _class_serprop_21
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
    .quad _class_serprop_46
    .quad _class_serprop_47
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_54
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_57
    .quad _class_serprop_58
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
    .quad _class_serprop_76
    .quad _class_serprop_77
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
    .quad _class_serprop_89
    .quad _class_serprop_missing
    .quad _class_serprop_91
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_96
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_0
    .quad _class_static_vtable_1
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_4
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_6
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
    .quad _class_static_vtable_21
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
    .quad _class_static_vtable_46
    .quad _class_static_vtable_47
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_54
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_57
    .quad _class_static_vtable_58
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
    .quad _class_static_vtable_76
    .quad _class_static_vtable_77
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
    .quad _class_static_vtable_89
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_91
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_96
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_0
    .quad _class_callable_methods_1
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_4
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_6
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
    .quad _class_callable_methods_21
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
    .quad _class_callable_methods_46
    .quad _class_callable_methods_47
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_54
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_57
    .quad _class_callable_methods_58
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
    .quad _class_callable_methods_76
    .quad _class_callable_methods_77
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
    .quad _class_callable_methods_89
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_91
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_96
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
    .ascii "LogicException"
.globl _class_by_name_str_4
_class_by_name_str_4:
    .ascii "RuntimeException"
.globl _class_by_name_str_6
_class_by_name_str_6:
    .ascii "Error"
.globl _class_by_name_str_21
_class_by_name_str_21:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_46
_class_by_name_str_46:
    .ascii "TypeError"
.globl _class_by_name_str_47
_class_by_name_str_47:
    .ascii "App\\Greeter"
.globl _class_by_name_str_54
_class_by_name_str_54:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_57
_class_by_name_str_57:
    .ascii "JsonException"
.globl _class_by_name_str_58
_class_by_name_str_58:
    .ascii "ReflectionException"
.globl _class_by_name_str_76
_class_by_name_str_76:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_77
_class_by_name_str_77:
    .ascii "ArithmeticError"
.globl _class_by_name_str_89
_class_by_name_str_89:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_91
_class_by_name_str_91:
    .ascii "ValueError"
.globl _class_by_name_str_96
_class_by_name_str_96:
    .ascii "stdClass"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 15
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_0
    .quad 9
    .quad 0
    .quad 72
    .quad _class_by_name_str_1
    .quad 14
    .quad 1
    .quad 72
    .quad _class_by_name_str_4
    .quad 16
    .quad 4
    .quad 72
    .quad _class_by_name_str_6
    .quad 5
    .quad 6
    .quad 72
    .quad _class_by_name_str_21
    .quad 19
    .quad 21
    .quad 72
    .quad _class_by_name_str_46
    .quad 9
    .quad 46
    .quad 72
    .quad _class_by_name_str_47
    .quad 11
    .quad 47
    .quad 8
    .quad _class_by_name_str_54
    .quad 24
    .quad 54
    .quad 72
    .quad _class_by_name_str_57
    .quad 13
    .quad 57
    .quad 72
    .quad _class_by_name_str_58
    .quad 19
    .quad 58
    .quad 72
    .quad _class_by_name_str_76
    .quad 20
    .quad 76
    .quad 72
    .quad _class_by_name_str_77
    .quad 15
    .quad 77
    .quad 72
    .quad _class_by_name_str_89
    .quad 19
    .quad 89
    .quad 72
    .quad _class_by_name_str_91
    .quad 10
    .quad 91
    .quad 72
    .quad _class_by_name_str_96
    .quad 8
    .quad 96
    .quad 16
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 97
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
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_10
_interface_methods_10:
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
.globl _class_interfaces_0
_class_interfaces_0:
    .quad 2
    .quad 14
    .quad _class_interface_impl_0_14
    .quad 10
    .quad _class_interface_impl_0_10
.globl _class_interface_impl_0_14
_class_interface_impl_0_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_0_10
_class_interface_impl_0_10:
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
    .quad 14
    .quad _class_interface_impl_1_14
    .quad 10
    .quad _class_interface_impl_1_10
.globl _class_interface_impl_1_14
_class_interface_impl_1_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_10
_class_interface_impl_1_10:
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
.globl _class_interfaces_4
_class_interfaces_4:
    .quad 2
    .quad 14
    .quad _class_interface_impl_4_14
    .quad 10
    .quad _class_interface_impl_4_10
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
.globl _class_interface_impl_4_10
_class_interface_impl_4_10:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_4
_class_vtable_4:
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
.globl _class_interfaces_6
_class_interfaces_6:
    .quad 2
    .quad 14
    .quad _class_interface_impl_6_14
    .quad 10
    .quad _class_interface_impl_6_10
.globl _class_interface_impl_6_14
_class_interface_impl_6_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_6_10
_class_interface_impl_6_10:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_6
_class_vtable_6:
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
.globl _class_interfaces_21
_class_interfaces_21:
    .quad 2
    .quad 14
    .quad _class_interface_impl_21_14
    .quad 10
    .quad _class_interface_impl_21_10
.globl _class_interface_impl_21_14
_class_interface_impl_21_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_21_10
_class_interface_impl_21_10:
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
.globl _class_interfaces_46
_class_interfaces_46:
    .quad 2
    .quad 14
    .quad _class_interface_impl_46_14
    .quad 10
    .quad _class_interface_impl_46_10
.globl _class_interface_impl_46_14
_class_interface_impl_46_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_46_10
_class_interface_impl_46_10:
    .quad 0
.globl _class_json_pname_46_0
_class_json_pname_46_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_46
_class_json_desc_46:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_46_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_46
_class_gc_desc_46:
    .byte 1, 0, 7, 4
.globl _class_serpname_46_0
_class_serpname_46_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_46_1
_class_serpname_46_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_46_2
_class_serpname_46_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_46_3
_class_serpname_46_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_46
_class_serprop_46:
    .quad 4
    .quad _class_serpname_46_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_46_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_46_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_46_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_46
_class_vtable_46:
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
.globl _class_static_vtable_46
_class_static_vtable_46:
    .quad 0
.globl _class_callable_method_name_46__u__u_construct
_class_callable_method_name_46__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_46__u__u_tostring
_class_callable_method_name_46__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_46_getcode
_class_callable_method_name_46_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_46_getfile
_class_callable_method_name_46_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_46_getline
_class_callable_method_name_46_getline:
    .ascii "getline"
.globl _class_callable_method_name_46_getmessage
_class_callable_method_name_46_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_46_getprevious
_class_callable_method_name_46_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_46_gettrace
_class_callable_method_name_46_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_46_gettraceasstring
_class_callable_method_name_46_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_46
_class_callable_methods_46:
    .quad 9
    .quad _class_callable_method_name_46__u__u_construct
    .quad 11
    .quad _class_callable_method_name_46__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_46_getcode
    .quad 7
    .quad _class_callable_method_name_46_getfile
    .quad 7
    .quad _class_callable_method_name_46_getline
    .quad 7
    .quad _class_callable_method_name_46_getmessage
    .quad 10
    .quad _class_callable_method_name_46_getprevious
    .quad 11
    .quad _class_callable_method_name_46_gettrace
    .quad 8
    .quad _class_callable_method_name_46_gettraceasstring
    .quad 16
.globl _class_interfaces_47
_class_interfaces_47:
    .quad 0
    .p2align 3
.globl _class_json_desc_47
_class_json_desc_47:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_47
_class_gc_desc_47:
    .byte 0
    .p2align 3
.globl _class_serprop_47
_class_serprop_47:
    .quad 0
    .p2align 3
.globl _class_vtable_47
_class_vtable_47:
    .quad _method_App_N_Greeter_hello
    .p2align 3
.globl _class_static_vtable_47
_class_static_vtable_47:
    .quad 0
.globl _class_callable_method_name_47_hello
_class_callable_method_name_47_hello:
    .ascii "hello"
.p2align 3
.globl _class_callable_methods_47
_class_callable_methods_47:
    .quad 1
    .quad _class_callable_method_name_47_hello
    .quad 5
.globl _class_interfaces_54
_class_interfaces_54:
    .quad 2
    .quad 14
    .quad _class_interface_impl_54_14
    .quad 10
    .quad _class_interface_impl_54_10
.globl _class_interface_impl_54_14
_class_interface_impl_54_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_54_10
_class_interface_impl_54_10:
    .quad 0
.globl _class_json_pname_54_0
_class_json_pname_54_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_54
_class_json_desc_54:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_54_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_54
_class_gc_desc_54:
    .byte 1, 0, 7, 4
.globl _class_serpname_54_0
_class_serpname_54_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_54_1
_class_serpname_54_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_54_2
_class_serpname_54_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_54_3
_class_serpname_54_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_54
_class_serprop_54:
    .quad 4
    .quad _class_serpname_54_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_54_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_54_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_54_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_54
_class_vtable_54:
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
.globl _class_static_vtable_54
_class_static_vtable_54:
    .quad 0
.globl _class_callable_method_name_54__u__u_construct
_class_callable_method_name_54__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_54__u__u_tostring
_class_callable_method_name_54__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_54_getcode
_class_callable_method_name_54_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_54_getfile
_class_callable_method_name_54_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_54_getline
_class_callable_method_name_54_getline:
    .ascii "getline"
.globl _class_callable_method_name_54_getmessage
_class_callable_method_name_54_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_54_getprevious
_class_callable_method_name_54_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_54_gettrace
_class_callable_method_name_54_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_54_gettraceasstring
_class_callable_method_name_54_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_54
_class_callable_methods_54:
    .quad 9
    .quad _class_callable_method_name_54__u__u_construct
    .quad 11
    .quad _class_callable_method_name_54__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_54_getcode
    .quad 7
    .quad _class_callable_method_name_54_getfile
    .quad 7
    .quad _class_callable_method_name_54_getline
    .quad 7
    .quad _class_callable_method_name_54_getmessage
    .quad 10
    .quad _class_callable_method_name_54_getprevious
    .quad 11
    .quad _class_callable_method_name_54_gettrace
    .quad 8
    .quad _class_callable_method_name_54_gettraceasstring
    .quad 16
.globl _class_interfaces_57
_class_interfaces_57:
    .quad 2
    .quad 14
    .quad _class_interface_impl_57_14
    .quad 10
    .quad _class_interface_impl_57_10
.globl _class_interface_impl_57_14
_class_interface_impl_57_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_57_10
_class_interface_impl_57_10:
    .quad 0
.globl _class_json_pname_57_0
_class_json_pname_57_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_57
_class_json_desc_57:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_57_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_57
_class_gc_desc_57:
    .byte 1, 0, 7, 4
.globl _class_serpname_57_0
_class_serpname_57_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_57_1
_class_serpname_57_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_57_2
_class_serpname_57_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_57_3
_class_serpname_57_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_57
_class_serprop_57:
    .quad 4
    .quad _class_serpname_57_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_57_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_57_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_57_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_57
_class_vtable_57:
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
.globl _class_static_vtable_57
_class_static_vtable_57:
    .quad 0
.globl _class_callable_method_name_57__u__u_construct
_class_callable_method_name_57__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_57__u__u_tostring
_class_callable_method_name_57__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_57_getcode
_class_callable_method_name_57_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_57_getfile
_class_callable_method_name_57_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_57_getline
_class_callable_method_name_57_getline:
    .ascii "getline"
.globl _class_callable_method_name_57_getmessage
_class_callable_method_name_57_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_57_getprevious
_class_callable_method_name_57_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_57_gettrace
_class_callable_method_name_57_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_57_gettraceasstring
_class_callable_method_name_57_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_57
_class_callable_methods_57:
    .quad 9
    .quad _class_callable_method_name_57__u__u_construct
    .quad 11
    .quad _class_callable_method_name_57__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_57_getcode
    .quad 7
    .quad _class_callable_method_name_57_getfile
    .quad 7
    .quad _class_callable_method_name_57_getline
    .quad 7
    .quad _class_callable_method_name_57_getmessage
    .quad 10
    .quad _class_callable_method_name_57_getprevious
    .quad 11
    .quad _class_callable_method_name_57_gettrace
    .quad 8
    .quad _class_callable_method_name_57_gettraceasstring
    .quad 16
.globl _class_interfaces_58
_class_interfaces_58:
    .quad 2
    .quad 14
    .quad _class_interface_impl_58_14
    .quad 10
    .quad _class_interface_impl_58_10
.globl _class_interface_impl_58_14
_class_interface_impl_58_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_58_10
_class_interface_impl_58_10:
    .quad 0
.globl _class_json_pname_58_0
_class_json_pname_58_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_58
_class_json_desc_58:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_58_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_58
_class_gc_desc_58:
    .byte 1, 0, 7, 4
.globl _class_serpname_58_0
_class_serpname_58_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_58_1
_class_serpname_58_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_58_2
_class_serpname_58_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_58_3
_class_serpname_58_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_58
_class_serprop_58:
    .quad 4
    .quad _class_serpname_58_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_58_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_58_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_58_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_58
_class_vtable_58:
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
.globl _class_static_vtable_58
_class_static_vtable_58:
    .quad 0
.globl _class_callable_method_name_58__u__u_construct
_class_callable_method_name_58__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_58__u__u_tostring
_class_callable_method_name_58__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_58_getcode
_class_callable_method_name_58_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_58_getfile
_class_callable_method_name_58_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_58_getline
_class_callable_method_name_58_getline:
    .ascii "getline"
.globl _class_callable_method_name_58_getmessage
_class_callable_method_name_58_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_58_getprevious
_class_callable_method_name_58_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_58_gettrace
_class_callable_method_name_58_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_58_gettraceasstring
_class_callable_method_name_58_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_58
_class_callable_methods_58:
    .quad 9
    .quad _class_callable_method_name_58__u__u_construct
    .quad 11
    .quad _class_callable_method_name_58__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_58_getcode
    .quad 7
    .quad _class_callable_method_name_58_getfile
    .quad 7
    .quad _class_callable_method_name_58_getline
    .quad 7
    .quad _class_callable_method_name_58_getmessage
    .quad 10
    .quad _class_callable_method_name_58_getprevious
    .quad 11
    .quad _class_callable_method_name_58_gettrace
    .quad 8
    .quad _class_callable_method_name_58_gettraceasstring
    .quad 16
.globl _class_interfaces_76
_class_interfaces_76:
    .quad 2
    .quad 14
    .quad _class_interface_impl_76_14
    .quad 10
    .quad _class_interface_impl_76_10
.globl _class_interface_impl_76_14
_class_interface_impl_76_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_76_10
_class_interface_impl_76_10:
    .quad 0
.globl _class_json_pname_76_0
_class_json_pname_76_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_76
_class_json_desc_76:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_76_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_76
_class_gc_desc_76:
    .byte 1, 0, 7, 4
.globl _class_serpname_76_0
_class_serpname_76_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_76_1
_class_serpname_76_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_76_2
_class_serpname_76_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_76_3
_class_serpname_76_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_76
_class_serprop_76:
    .quad 4
    .quad _class_serpname_76_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_76_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_76_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_76_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_76
_class_vtable_76:
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
.globl _class_static_vtable_76
_class_static_vtable_76:
    .quad 0
.globl _class_callable_method_name_76__u__u_construct
_class_callable_method_name_76__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_76__u__u_tostring
_class_callable_method_name_76__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_76_getcode
_class_callable_method_name_76_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_76_getfile
_class_callable_method_name_76_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_76_getline
_class_callable_method_name_76_getline:
    .ascii "getline"
.globl _class_callable_method_name_76_getmessage
_class_callable_method_name_76_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_76_getprevious
_class_callable_method_name_76_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_76_gettrace
_class_callable_method_name_76_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_76_gettraceasstring
_class_callable_method_name_76_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_76
_class_callable_methods_76:
    .quad 9
    .quad _class_callable_method_name_76__u__u_construct
    .quad 11
    .quad _class_callable_method_name_76__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_76_getcode
    .quad 7
    .quad _class_callable_method_name_76_getfile
    .quad 7
    .quad _class_callable_method_name_76_getline
    .quad 7
    .quad _class_callable_method_name_76_getmessage
    .quad 10
    .quad _class_callable_method_name_76_getprevious
    .quad 11
    .quad _class_callable_method_name_76_gettrace
    .quad 8
    .quad _class_callable_method_name_76_gettraceasstring
    .quad 16
.globl _class_interfaces_77
_class_interfaces_77:
    .quad 2
    .quad 14
    .quad _class_interface_impl_77_14
    .quad 10
    .quad _class_interface_impl_77_10
.globl _class_interface_impl_77_14
_class_interface_impl_77_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_77_10
_class_interface_impl_77_10:
    .quad 0
.globl _class_json_pname_77_0
_class_json_pname_77_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_77
_class_json_desc_77:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_77_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_77
_class_gc_desc_77:
    .byte 1, 0, 7, 4
.globl _class_serpname_77_0
_class_serpname_77_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_77_1
_class_serpname_77_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_77_2
_class_serpname_77_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_77_3
_class_serpname_77_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_77
_class_serprop_77:
    .quad 4
    .quad _class_serpname_77_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_77_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_77_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_77_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_77
_class_vtable_77:
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
.globl _class_static_vtable_77
_class_static_vtable_77:
    .quad 0
.globl _class_callable_method_name_77__u__u_construct
_class_callable_method_name_77__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_77__u__u_tostring
_class_callable_method_name_77__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_77_getcode
_class_callable_method_name_77_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_77_getfile
_class_callable_method_name_77_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_77_getline
_class_callable_method_name_77_getline:
    .ascii "getline"
.globl _class_callable_method_name_77_getmessage
_class_callable_method_name_77_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_77_getprevious
_class_callable_method_name_77_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_77_gettrace
_class_callable_method_name_77_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_77_gettraceasstring
_class_callable_method_name_77_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_77
_class_callable_methods_77:
    .quad 9
    .quad _class_callable_method_name_77__u__u_construct
    .quad 11
    .quad _class_callable_method_name_77__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_77_getcode
    .quad 7
    .quad _class_callable_method_name_77_getfile
    .quad 7
    .quad _class_callable_method_name_77_getline
    .quad 7
    .quad _class_callable_method_name_77_getmessage
    .quad 10
    .quad _class_callable_method_name_77_getprevious
    .quad 11
    .quad _class_callable_method_name_77_gettrace
    .quad 8
    .quad _class_callable_method_name_77_gettraceasstring
    .quad 16
.globl _class_interfaces_89
_class_interfaces_89:
    .quad 2
    .quad 14
    .quad _class_interface_impl_89_14
    .quad 10
    .quad _class_interface_impl_89_10
.globl _class_interface_impl_89_14
_class_interface_impl_89_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_89_10
_class_interface_impl_89_10:
    .quad 0
.globl _class_json_pname_89_0
_class_json_pname_89_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_89
_class_json_desc_89:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_89_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_89
_class_gc_desc_89:
    .byte 1, 0, 7, 4
.globl _class_serpname_89_0
_class_serpname_89_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_89_1
_class_serpname_89_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_89_2
_class_serpname_89_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_89_3
_class_serpname_89_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_89
_class_serprop_89:
    .quad 4
    .quad _class_serpname_89_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_89_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_89_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_89_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_89
_class_vtable_89:
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
.globl _class_static_vtable_89
_class_static_vtable_89:
    .quad 0
.globl _class_callable_method_name_89__u__u_construct
_class_callable_method_name_89__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_89__u__u_tostring
_class_callable_method_name_89__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_89_getcode
_class_callable_method_name_89_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_89_getfile
_class_callable_method_name_89_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_89_getline
_class_callable_method_name_89_getline:
    .ascii "getline"
.globl _class_callable_method_name_89_getmessage
_class_callable_method_name_89_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_89_getprevious
_class_callable_method_name_89_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_89_gettrace
_class_callable_method_name_89_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_89_gettraceasstring
_class_callable_method_name_89_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_89
_class_callable_methods_89:
    .quad 9
    .quad _class_callable_method_name_89__u__u_construct
    .quad 11
    .quad _class_callable_method_name_89__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_89_getcode
    .quad 7
    .quad _class_callable_method_name_89_getfile
    .quad 7
    .quad _class_callable_method_name_89_getline
    .quad 7
    .quad _class_callable_method_name_89_getmessage
    .quad 10
    .quad _class_callable_method_name_89_getprevious
    .quad 11
    .quad _class_callable_method_name_89_gettrace
    .quad 8
    .quad _class_callable_method_name_89_gettraceasstring
    .quad 16
.globl _class_interfaces_91
_class_interfaces_91:
    .quad 2
    .quad 14
    .quad _class_interface_impl_91_14
    .quad 10
    .quad _class_interface_impl_91_10
.globl _class_interface_impl_91_14
_class_interface_impl_91_14:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_91_10
_class_interface_impl_91_10:
    .quad 0
.globl _class_json_pname_91_0
_class_json_pname_91_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_91
_class_json_desc_91:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_91_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_91
_class_gc_desc_91:
    .byte 1, 0, 7, 4
.globl _class_serpname_91_0
_class_serpname_91_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_91_1
_class_serpname_91_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_91_2
_class_serpname_91_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_91_3
_class_serpname_91_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_91
_class_serprop_91:
    .quad 4
    .quad _class_serpname_91_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_91_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_91_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_91_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_91
_class_vtable_91:
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
.globl _class_static_vtable_91
_class_static_vtable_91:
    .quad 0
.globl _class_callable_method_name_91__u__u_construct
_class_callable_method_name_91__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_91__u__u_tostring
_class_callable_method_name_91__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_91_getcode
_class_callable_method_name_91_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_91_getfile
_class_callable_method_name_91_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_91_getline
_class_callable_method_name_91_getline:
    .ascii "getline"
.globl _class_callable_method_name_91_getmessage
_class_callable_method_name_91_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_91_getprevious
_class_callable_method_name_91_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_91_gettrace
_class_callable_method_name_91_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_91_gettraceasstring
_class_callable_method_name_91_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_91
_class_callable_methods_91:
    .quad 9
    .quad _class_callable_method_name_91__u__u_construct
    .quad 11
    .quad _class_callable_method_name_91__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_91_getcode
    .quad 7
    .quad _class_callable_method_name_91_getfile
    .quad 7
    .quad _class_callable_method_name_91_getline
    .quad 7
    .quad _class_callable_method_name_91_getmessage
    .quad 10
    .quad _class_callable_method_name_91_getprevious
    .quad 11
    .quad _class_callable_method_name_91_gettrace
    .quad 8
    .quad _class_callable_method_name_91_gettraceasstring
    .quad 16
.globl _class_interfaces_96
_class_interfaces_96:
    .quad 0
    .p2align 3
.globl _class_json_desc_96
_class_json_desc_96:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_96
_class_gc_desc_96:
    .byte 0
    .p2align 3
.globl _class_serprop_96
_class_serprop_96:
    .quad 0
    .p2align 3
.globl _class_vtable_96
_class_vtable_96:
    .quad 0
    .p2align 3
.globl _class_static_vtable_96
_class_static_vtable_96:
    .quad 0
.p2align 3
.globl _class_callable_methods_96
_class_callable_methods_96:
    .quad 0
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 96

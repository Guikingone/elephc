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
    ; @fn name=_class_propinit_11 symbol=_class_propinit_11 synthetic=1
.align 2

.globl _class_propinit_11
_class_propinit_11:
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
_eir__class_propinit_11_entry_0:
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
_class_propinit_11_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_11
    ; @fn name=_class_propinit_12 symbol=_class_propinit_12 synthetic=1
.align 2

.globl _class_propinit_12
_class_propinit_12:
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
_eir__class_propinit_12_entry_0:
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
_class_propinit_12_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_12
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
    ; @fn name=_class_propinit_16 symbol=_class_propinit_16 synthetic=1
.align 2

.globl _class_propinit_16
_class_propinit_16:
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
_eir__class_propinit_16_entry_0:
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
_class_propinit_16_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
    ; @fn name=_class_propinit_18 symbol=_class_propinit_18 synthetic=1
.align 2

.globl _class_propinit_18
_class_propinit_18:
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
_eir__class_propinit_18_entry_0:
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
_class_propinit_18_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_18
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
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_28_entry_0:
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
_class_propinit_28_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_29_entry_0:
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
_class_propinit_29_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
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
_eir__class_propinit_31_entry_0:
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
_class_propinit_31_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
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
_class_propinit_38_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_38
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
_eir__class_propinit_44_entry_0:
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
_class_propinit_44_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_44
    ; @fn name=_class_propinit_46 symbol=_class_propinit_46 synthetic=1
.align 2

.globl _class_propinit_46
_class_propinit_46:
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
_eir__class_propinit_46_entry_0:
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
_class_propinit_46_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
_eir__class_propinit_61_entry_0:
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
_class_propinit_61_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_61
    ; @fn name=_class_propinit_64 symbol=_class_propinit_64 synthetic=1
.align 2

.globl _class_propinit_64
_class_propinit_64:
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
_eir__class_propinit_64_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_64_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_64
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
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
_eir__class_propinit_65_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_65_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_65
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
    ; @fn name=_class_propinit_68 symbol=_class_propinit_68 synthetic=1
.align 2

.globl _class_propinit_68
_class_propinit_68:
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
_eir__class_propinit_68_entry_0:
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
_class_propinit_68_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_68
    ; @fn name=_class_propinit_69 symbol=_class_propinit_69 synthetic=1
.align 2

.globl _class_propinit_69
_class_propinit_69:
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
_eir__class_propinit_69_entry_0:
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
_class_propinit_69_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_69
    ; @fn name=_class_propinit_71 symbol=_class_propinit_71 synthetic=1
.align 2

.globl _class_propinit_71
_class_propinit_71:
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
_eir__class_propinit_71_entry_0:
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
_class_propinit_71_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
_eir__class_propinit_86_entry_0:
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
_class_propinit_86_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_86
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
    ; @fn name=_class_propinit_93 symbol=_class_propinit_93 synthetic=1
.align 2

.globl _class_propinit_93
_class_propinit_93:
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
_eir__class_propinit_93_entry_0:
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
_class_propinit_93_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_93
    ; @fn name=_class_propinit_94 symbol=_class_propinit_94 synthetic=1
.align 2

.globl _class_propinit_94
_class_propinit_94:
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
_eir__class_propinit_94_entry_0:
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
_class_propinit_94_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_94
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
    sub sp, sp, #2144
    mov x9, sp
    add x9, x9, #2128
    stp x29, x30, [x9]
    add x29, sp, #2128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #2120
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #2104
    str x21, [x9]
    sub x9, x29, #2112
    str x22, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #1928
    str xzr, [x9]
    sub x9, x29, #1936
    str xzr, [x9]
    sub x9, x29, #1944
    str xzr, [x9]
    sub x9, x29, #1952
    str xzr, [x9]
    sub x9, x29, #1976
    str xzr, [x9]
    sub x9, x29, #1968
    str xzr, [x9]
    sub x9, x29, #1992
    str xzr, [x9]
    sub x9, x29, #1984
    str xzr, [x9]
    sub x9, x29, #2008
    str xzr, [x9]
    sub x9, x29, #2000
    str xzr, [x9]
    sub x9, x29, #2016
    str xzr, [x9]
    sub x9, x29, #2024
    str xzr, [x9]
    sub x9, x29, #2040
    str xzr, [x9]
    sub x9, x29, #2032
    str xzr, [x9]
    sub x9, x29, #2056
    str xzr, [x9]
    sub x9, x29, #2048
    str xzr, [x9]
    sub x9, x29, #2064
    str xzr, [x9]
    sub x9, x29, #2072
    str xzr, [x9]
    sub x9, x29, #2080
    str xzr, [x9]
    sub x9, x29, #2088
    str xzr, [x9]
    sub x9, x29, #2096
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=5 col=1 end=5:7 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=5 col=10 end=5:11 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #4
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-8]
    ; @src line=5 col=11 end=5:12 op=array_new
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
    stur x0, [x29, #-16]
    ; @src line=5 col=12 op=const_str
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #8
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=5 col=12 op=array_push
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldur x9, [x29, #-16]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-16]
    ; @src line=5 col=24 op=const_str
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #9
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=5 col=24 op=array_push
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldur x9, [x29, #-16]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-16]
    ; @src line=5 col=11 end=5:12 op=array_push
    ldur x1, [x29, #-16]
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-8]
    ; @src line=5 col=11 end=5:12 op=release
    ldur x0, [x29, #-16]
    bl __rt_decref_any
    ; @src line=5 col=38 end=5:39 op=array_new
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
    stur x0, [x29, #-56]
    ; @src line=5 col=39 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #4
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=5 col=39 op=array_push
    ldur x1, [x29, #-72]
    ldur x2, [x29, #-64]
    ldur x9, [x29, #-56]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-56]
    ; @src line=5 col=47 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-88]
    stur x2, [x29, #-80]
    ; @src line=5 col=47 op=array_push
    ldur x1, [x29, #-88]
    ldur x2, [x29, #-80]
    ldur x9, [x29, #-56]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-56]
    ; @src line=5 col=38 end=5:39 op=array_push
    ldur x1, [x29, #-56]
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-8]
    ; @src line=5 col=38 end=5:39 op=release
    ldur x0, [x29, #-56]
    bl __rt_decref_any
    ; @src line=5 col=57 end=5:58 op=array_new
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
    stur x0, [x29, #-96]
    ; @src line=5 col=58 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-112]
    stur x2, [x29, #-104]
    ; @src line=5 col=58 op=array_push
    ldur x1, [x29, #-112]
    ldur x2, [x29, #-104]
    ldur x9, [x29, #-96]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-96]
    ; @src line=5 col=69 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #6
    stur x1, [x29, #-128]
    stur x2, [x29, #-120]
    ; @src line=5 col=69 op=array_push
    ldur x1, [x29, #-128]
    ldur x2, [x29, #-120]
    ldur x9, [x29, #-96]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-96]
    ; @src line=5 col=57 end=5:58 op=array_push
    ldur x1, [x29, #-96]
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-8]
    ; @src line=5 col=57 end=5:58 op=release
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; @src line=5 col=1 end=5:7 op=acquire
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-136]
    ; @src line=5 col=1 end=5:7 op=store_local
    ldur x0, [x29, #-136]
    sub x9, x29, #1928
    str x0, [x9]
    ; @src line=5 col=1 end=5:7 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_any
    ; @src line=6 col=1 end=6:8 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=10 end=6:16 op=load_local
    sub x9, x29, #1928
    ldr x0, [x9]
    stur x0, [x29, #-144]
    ; @src line=6 col=10 end=6:16 op=iter_start
    ldur x0, [x29, #-144]
    stur x0, [x29, #-216]
    mov x0, #-1
    stur x0, [x29, #-208]
    ldur x0, [x29, #-216]
    cbz x0, _eir_main_iter_len_null_source_0
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir_main_iter_len_null_source_0
    ldr x10, [x0]
    b _eir_main_iter_len_done_1
_eir_main_iter_len_null_source_0:
    mov x10, #0
_eir_main_iter_len_done_1:
    stur x10, [x29, #-152]
    ; @src line=6 col=10 end=6:16 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=6 col=10 end=6:16 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-232]
    ; @src line=6 col=10 end=6:16 op=acquire
    ldur x0, [x29, #-232]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-240]
    ; @src line=6 col=10 end=6:16 op=store_local
    ldur x0, [x29, #-240]
    sub x9, x29, #1936
    str x0, [x9]
    ; @src line=6 col=10 end=6:16 op=release
    ldur x0, [x29, #-232]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_1
    ; @block name=foreach.next
_eir_main_foreach_next_1:
    ; @src line=6 col=10 end=6:16 op=iter_next
    ldur x10, [x29, #-208]
    add x10, x10, #1
    ldur x11, [x29, #-152]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_2
    stur x10, [x29, #-208]
_eir_main_iter_next_done_2:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_2
    b _eir_main_foreach_exit_3
    ; @block name=foreach.body
_eir_main_foreach_body_2:
    ; @src line=6 col=10 end=6:16 op=iter_current_value
    ldur x12, [x29, #-216]
    ldur x10, [x29, #-208]
    add x12, x12, #24
    ldr x0, [x12, x10, lsl #3]
    mov x1, x0
    mov x2, xzr
    mov x0, #4
    bl __rt_mixed_from_value
    sub x9, x29, #256
    str x0, [x9]
    ; @src line=6 col=10 end=6:16 op=acquire
    sub x9, x29, #256
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #264
    str x0, [x9]
    ; @src line=6 col=10 end=6:16 op=load_local
    sub x9, x29, #1936
    ldr x0, [x9]
    sub x9, x29, #272
    str x0, [x9]
    ; @src line=6 col=10 end=6:16 op=release
    sub x9, x29, #272
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=6 col=10 end=6:16 op=store_local
    sub x9, x29, #264
    ldr x0, [x9]
    sub x9, x29, #1936
    str x0, [x9]
    ; @src line=6 col=10 end=6:16 op=release
    sub x9, x29, #256
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:8 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=1 end=6:8 op=load_local
    sub x9, x29, #1936
    ldr x0, [x9]
    sub x9, x29, #280
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=6 col=1 end=6:8 op=const_bool
    mov x0, #1
    mov x22, x0
    ; @src line=6 col=1 end=6:8 op=runtime_call
    mov x1, x21
    mov x2, #-1
    mov x3, x22
    sub x9, x29, #280
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #304
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=acquire
    sub x9, x29, #304
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #312
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=load_local
    sub x9, x29, #1944
    ldr x0, [x9]
    sub x9, x29, #320
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=release
    sub x9, x29, #320
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:8 op=store_local
    sub x9, x29, #312
    ldr x0, [x9]
    sub x9, x29, #1944
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=release
    sub x9, x29, #304
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:8 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=6 col=1 end=6:8 op=const_bool
    mov x0, #1
    mov x21, x0
    ; @src line=6 col=1 end=6:8 op=runtime_call
    mov x1, x22
    mov x2, #-1
    mov x3, x21
    sub x9, x29, #280
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #344
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=acquire
    sub x9, x29, #344
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #352
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=load_local
    sub x9, x29, #1952
    ldr x0, [x9]
    sub x9, x29, #360
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=release
    sub x9, x29, #360
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=6 col=1 end=6:8 op=store_local
    sub x9, x29, #352
    ldr x0, [x9]
    sub x9, x29, #1952
    str x0, [x9]
    ; @src line=6 col=1 end=6:8 op=release
    sub x9, x29, #344
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=7 col=5 end=7:9 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=7 col=22 end=7:24 op=load_local
    sub x9, x29, #1944
    ldr x0, [x9]
    sub x9, x29, #368
    str x0, [x9]
    ; @src line=7 col=26 end=7:28 op=load_local
    sub x9, x29, #1952
    ldr x0, [x9]
    sub x9, x29, #376
    str x0, [x9]
    ; @src line=7 col=12 end=7:29 op=builtin_call
    sub x9, x29, #368
    ldr x0, [x9]
    bl __rt_mixed_unbox
    cmp x0, #0
    b.eq _eir_main_mixed_arg_string_from_int_3
    cmp x0, #1
    b.eq _eir_main_mixed_arg_string_from_string_4
    cmp x0, #2
    b.eq _eir_main_mixed_arg_string_from_float_5
    cmp x0, #3
    b.eq _eir_main_mixed_arg_string_from_bool_6
    mov x1, xzr
    mov x2, xzr
    b _eir_main_mixed_arg_string_done_8
_eir_main_mixed_arg_string_from_int_3:
    mov x0, x1
    bl __rt_itoa
    b _eir_main_mixed_arg_string_done_8
_eir_main_mixed_arg_string_from_string_4:
    b _eir_main_mixed_arg_string_done_8
_eir_main_mixed_arg_string_from_float_5:
    fmov d0, x1
    bl __rt_ftoa
    b _eir_main_mixed_arg_string_done_8
_eir_main_mixed_arg_string_from_bool_6:
    cbz x1, _eir_main_mixed_arg_string_false_bool_7
    mov x0, x1
    bl __rt_itoa
    b _eir_main_mixed_arg_string_done_8
_eir_main_mixed_arg_string_false_bool_7:
    mov x1, xzr
    mov x2, xzr
_eir_main_mixed_arg_string_done_8:
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #376
    ldr x0, [x9]
    bl __rt_mixed_unbox
    cmp x0, #0
    b.eq _eir_main_mixed_arg_string_from_int_9
    cmp x0, #1
    b.eq _eir_main_mixed_arg_string_from_string_10
    cmp x0, #2
    b.eq _eir_main_mixed_arg_string_from_float_11
    cmp x0, #3
    b.eq _eir_main_mixed_arg_string_from_bool_12
    mov x1, xzr
    mov x2, xzr
    b _eir_main_mixed_arg_string_done_14
_eir_main_mixed_arg_string_from_int_9:
    mov x0, x1
    bl __rt_itoa
    b _eir_main_mixed_arg_string_done_14
_eir_main_mixed_arg_string_from_string_10:
    b _eir_main_mixed_arg_string_done_14
_eir_main_mixed_arg_string_from_float_11:
    fmov d0, x1
    bl __rt_ftoa
    b _eir_main_mixed_arg_string_done_14
_eir_main_mixed_arg_string_from_bool_12:
    cbz x1, _eir_main_mixed_arg_string_false_bool_13
    mov x0, x1
    bl __rt_itoa
    b _eir_main_mixed_arg_string_done_14
_eir_main_mixed_arg_string_false_bool_13:
    mov x1, xzr
    mov x2, xzr
_eir_main_mixed_arg_string_done_14:
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    bl __rt_strnatcmp
    mov x21, x0
    ; @src line=7 col=5 end=7:9 op=nop
    ; @src line=7 col=5 end=7:9 op=store_local
    mov x0, x21
    sub x9, x29, #1960
    str x0, [x9]
    ; @src line=8 col=5 end=8:9 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=12 end=8:16 op=load_local
    sub x9, x29, #1960
    ldr x0, [x9]
    mov x21, x0
    ; @src line=8 col=19 end=8:20 op=const_i64
    mov x0, #0
    mov x12, x0
    ; @src line=8 col=17 end=8:20 op=icmp
    mov x0, x21
    mov x10, x12
    cmp x0, x10
    cset x0, lt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir_main_ternary_then_4
    b _eir_main_ternary_else_5
    ; @block name=foreach.exit
_eir_main_foreach_exit_3:
    ; @src line=6 col=10 end=6:16 op=nop
    ; @src line=13 col=1 end=13:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=1 end=13:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=6 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #20
    sub x9, x29, #912
    str x1, [x9]
    sub x9, x29, #904
    str x2, [x9]
    ; @src line=13 col=1 end=13:5 op=echo_value
    sub x9, x29, #912
    ldr x1, [x9]
    sub x9, x29, #904
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=13 col=1 end=13:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=44 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #5
    sub x9, x29, #928
    str x1, [x9]
    sub x9, x29, #920
    str x2, [x9]
    ; @src line=13 col=53 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #4
    sub x9, x29, #944
    str x1, [x9]
    sub x9, x29, #936
    str x2, [x9]
    ; @src line=13 col=30 end=13:60 op=builtin_call
    sub x9, x29, #928
    ldr x1, [x9]
    sub x9, x29, #920
    ldr x2, [x9]
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #944
    ldr x1, [x9]
    sub x9, x29, #936
    ldr x2, [x9]
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    bl __rt_strnatcasecmp
    mov x21, x0
    ; @src line=13 col=1 end=13:5 op=echo_value
    mov x0, x21

    ; echo
    bl __rt_itoa
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=13 col=1 end=13:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=62 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    sub x9, x29, #968
    str x1, [x9]
    sub x9, x29, #960
    str x2, [x9]
    ; @src line=13 col=1 end=13:5 op=echo_value
    sub x9, x29, #968
    ldr x1, [x9]
    sub x9, x29, #960
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=16 col=1 end=16:6 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=16 col=9 end=16:10 op=array_new
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
    sub x9, x29, #976
    str x0, [x9]
    ; @src line=16 col=10 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #992
    str x1, [x9]
    sub x9, x29, #984
    str x2, [x9]
    ; @src line=16 col=10 op=array_push
    sub x9, x29, #992
    ldr x1, [x9]
    sub x9, x29, #984
    ldr x2, [x9]
    sub x9, x29, #976
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #976
    str x0, [x9]
    ; @src line=16 col=15 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #1
    sub x9, x29, #1008
    str x1, [x9]
    sub x9, x29, #1000
    str x2, [x9]
    ; @src line=16 col=15 op=array_push
    sub x9, x29, #1008
    ldr x1, [x9]
    sub x9, x29, #1000
    ldr x2, [x9]
    sub x9, x29, #976
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #976
    str x0, [x9]
    ; @src line=16 col=20 op=const_str
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #1
    sub x9, x29, #1024
    str x1, [x9]
    sub x9, x29, #1016
    str x2, [x9]
    ; @src line=16 col=20 op=array_push
    sub x9, x29, #1024
    ldr x1, [x9]
    sub x9, x29, #1016
    ldr x2, [x9]
    sub x9, x29, #976
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #976
    str x0, [x9]
    ; @src line=16 col=1 end=16:6 op=acquire
    sub x9, x29, #976
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1032
    str x0, [x9]
    ; @src line=16 col=1 end=16:6 op=store_local
    sub x9, x29, #1032
    ldr x0, [x9]
    sub x9, x29, #2016
    str x0, [x9]
    ; @src line=16 col=1 end=16:6 op=release
    sub x9, x29, #976
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=17 col=1 end=17:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=8 end=17:9 op=hash_new
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #1040
    str x0, [x9]
    ; @src line=17 col=9 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #4
    sub x9, x29, #1056
    str x1, [x9]
    sub x9, x29, #1048
    str x2, [x9]
    ; @src line=17 col=19 op=const_str
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #9
    sub x9, x29, #1072
    str x1, [x9]
    sub x9, x29, #1064
    str x2, [x9]
    ; @src line=17 col=8 end=17:9 op=hash_set
    sub x9, x29, #1056
    ldr x1, [x9]
    sub x9, x29, #1048
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1072
    ldr x1, [x9]
    sub x9, x29, #1064
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1040
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1040
    str x0, [x9]
    ; @src line=17 col=32 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #4
    sub x9, x29, #1088
    str x1, [x9]
    sub x9, x29, #1080
    str x2, [x9]
    ; @src line=17 col=42 end=17:46 op=const_i64
    mov x0, #5432
    mov x21, x0
    ; @src line=17 col=8 end=17:9 op=hash_set
    sub x9, x29, #1088
    ldr x1, [x9]
    sub x9, x29, #1080
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1040
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1040
    str x0, [x9]
    ; @src line=17 col=1 end=17:5 op=acquire
    sub x9, x29, #1040
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1104
    str x0, [x9]
    ; @src line=17 col=1 end=17:5 op=store_local
    sub x9, x29, #1104
    ldr x0, [x9]
    sub x9, x29, #2024
    str x0, [x9]
    ; @src line=17 col=1 end=17:5 op=release
    sub x9, x29, #1040
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=6 op=const_str
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #14
    sub x9, x29, #1120
    str x1, [x9]
    sub x9, x29, #1112
    str x2, [x9]
    ; @src line=18 col=1 end=18:5 op=echo_value
    sub x9, x29, #1120
    ldr x1, [x9]
    sub x9, x29, #1112
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=38 end=18:43 op=load_local
    sub x9, x29, #2016
    ldr x0, [x9]
    sub x9, x29, #1128
    str x0, [x9]
    ; @src line=18 col=24 end=18:44 op=runtime_call
    sub x9, x29, #1128
    ldr x0, [x9]
    bl __rt_array_is_list
    mov x21, x0
    ; @src line=18 col=24 end=18:44 op=nop
    mov x0, x21
    cbnz x0, _eir_main_ternary_then_10
    b _eir_main_ternary_else_11
    ; @block name=ternary.then
_eir_main_ternary_then_4:
    ; @src line=8 col=23 op=const_str
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #1
    sub x9, x29, #424
    str x1, [x9]
    sub x9, x29, #416
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=acquire
    sub x9, x29, #424
    ldr x1, [x9]
    sub x9, x29, #416
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #440
    str x1, [x9]
    sub x9, x29, #432
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=load_local
    sub x9, x29, #1976
    ldr x1, [x9]
    sub x9, x29, #1968
    ldr x2, [x9]
    sub x9, x29, #456
    str x1, [x9]
    sub x9, x29, #448
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=release
    sub x9, x29, #456
    ldr x1, [x9]
    sub x9, x29, #448
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=8 col=21 end=8:22 op=store_local
    sub x9, x29, #440
    ldr x1, [x9]
    sub x9, x29, #432
    ldr x2, [x9]
    sub x9, x29, #1976
    str x1, [x9]
    sub x9, x29, #1968
    str x2, [x9]
    b _eir_main_ternary_merge_6
    ; @block name=ternary.else
_eir_main_ternary_else_5:
    ; @src line=8 col=30 end=8:34 op=load_local
    sub x9, x29, #1960
    ldr x0, [x9]
    mov x21, x0
    ; @src line=8 col=37 end=8:38 op=const_i64
    mov x0, #0
    mov x12, x0
    ; @src line=8 col=35 end=8:38 op=icmp
    mov x0, x21
    mov x10, x12
    cmp x0, x10
    cset x0, gt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir_main_ternary_then_7
    b _eir_main_ternary_else_8
    ; @block name=ternary.merge
_eir_main_ternary_merge_6:
    ; @src line=8 col=21 end=8:22 op=load_local
    sub x9, x29, #1976
    ldr x1, [x9]
    sub x9, x29, #1968
    ldr x2, [x9]
    sub x9, x29, #640
    str x1, [x9]
    sub x9, x29, #632
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=unset_local
    sub x9, x29, #1976
    str xzr, [x9]
    sub x9, x29, #1968
    str xzr, [x9]
    ; @src line=8 col=5 end=8:9 op=acquire
    sub x9, x29, #640
    ldr x1, [x9]
    sub x9, x29, #632
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #656
    str x1, [x9]
    sub x9, x29, #648
    str x2, [x9]
    ; @src line=8 col=5 end=8:9 op=load_local
    sub x9, x29, #2008
    ldr x1, [x9]
    sub x9, x29, #2000
    ldr x2, [x9]
    sub x9, x29, #672
    str x1, [x9]
    sub x9, x29, #664
    str x2, [x9]
    ; @src line=8 col=5 end=8:9 op=release
    sub x9, x29, #672
    ldr x1, [x9]
    sub x9, x29, #664
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=8 col=5 end=8:9 op=store_local
    sub x9, x29, #656
    ldr x1, [x9]
    sub x9, x29, #648
    ldr x2, [x9]
    sub x9, x29, #2008
    str x1, [x9]
    sub x9, x29, #2000
    str x2, [x9]
    ; @src line=8 col=5 end=8:9 op=release
    sub x9, x29, #640
    ldr x1, [x9]
    sub x9, x29, #632
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=9 col=5 end=9:9 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=10 op=const_str
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    sub x9, x29, #688
    str x1, [x9]
    sub x9, x29, #680
    str x2, [x9]
    ; @src line=9 col=10 op=load_local
    sub x9, x29, #1944
    ldr x0, [x9]
    sub x9, x29, #696
    str x0, [x9]
    ; @src line=9 col=10 op=cast
    sub x9, x29, #696
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_15
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_object_15:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_18
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_19
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_20
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_21
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_22
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_23
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_24
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_25
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_26
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_27
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_28
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_29
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_30
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_31
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_32
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_33
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_34
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_35
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_36
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_37
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_38
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_39
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_40
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_41
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_42
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_43
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_44
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_45
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_46
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_47
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_48
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_49
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_50
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_51
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_52
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_53
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_54
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_55
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_56
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_57
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_58
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_59
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_60
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_61
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_62
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_63
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_64
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_65
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_66
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_67
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_68
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_69
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_70
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_71
    mov x10, #98
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_72
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_73
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_74
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_75
    b _eir_main_mixed_string_no_match_16
_eir_main_mixed_string_Exception_18:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateException_19:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateMalformedIntervalStringException_20:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_CachingIterator_21:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_RecursiveCachingIterator_22:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_Error_23:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ArithmeticError_24:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_RuntimeException_25:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_OutOfBoundsException_26:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateError_27:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_LogicException_28:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_BadFunctionCallException_29:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_SplFileInfo_30:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_PharFileInfo_31:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionClass_32:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateRangeError_33:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionFunctionAbstract_34:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionNamedType_35:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_FiberError_36:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionMethod_37:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_OverflowException_38:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionEnumBackedCase_39:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionClassConstant_40:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DomainException_41:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionIntersectionType_42:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_UnhandledMatchError_43:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateMalformedPeriodStringException_44:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionProperty_45:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionUnionType_46:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateInvalidTimeZoneException_47:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DirectoryIterator_48:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_FilesystemIterator_49:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_GlobIterator_50:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_TypeError_51:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionException_52:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_SplFileObject_53:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_InvalidArgumentException_54:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionFunction_55:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateMalformedStringException_56:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_RangeException_57:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_JsonException_58:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateUnknownException_59:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_UnderflowException_60:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_PharData_61:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionParameter_62:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_BadMethodCallException_63:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_SplTempFileObject_64:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_UnexpectedValueException_65:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_LengthException_66:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ValueError_67:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateInvalidOperationException_68:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_DateObjectError_69:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_RecursiveDirectoryIterator_70:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_Phar_71:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionObject_72:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionEnum_73:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_ReflectionEnumUnitCase_74:
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
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_OutOfRangeException_75:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_17
_eir_main_mixed_string_no_match_16:
    mov x0, #2
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_17:
    sub x9, x29, #712
    str x1, [x9]
    sub x9, x29, #704
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #688
    ldr x1, [x9]
    sub x9, x29, #680
    ldr x2, [x9]
    sub x9, x29, #712
    ldr x3, [x9]
    sub x9, x29, #704
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #728
    str x1, [x9]
    sub x9, x29, #720
    str x2, [x9]
    ; @src line=9 col=10 op=release
    sub x9, x29, #712
    ldr x1, [x9]
    sub x9, x29, #704
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=9 col=10 op=const_str
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #1
    sub x9, x29, #744
    str x1, [x9]
    sub x9, x29, #736
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #728
    ldr x1, [x9]
    sub x9, x29, #720
    ldr x2, [x9]
    sub x9, x29, #744
    ldr x3, [x9]
    sub x9, x29, #736
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #760
    str x1, [x9]
    sub x9, x29, #752
    str x2, [x9]
    ; @src line=9 col=10 op=release
    ; @src line=9 col=10 op=load_local
    sub x9, x29, #2008
    ldr x1, [x9]
    sub x9, x29, #2000
    ldr x2, [x9]
    sub x9, x29, #776
    str x1, [x9]
    sub x9, x29, #768
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #760
    ldr x1, [x9]
    sub x9, x29, #752
    ldr x2, [x9]
    sub x9, x29, #776
    ldr x3, [x9]
    sub x9, x29, #768
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #792
    str x1, [x9]
    sub x9, x29, #784
    str x2, [x9]
    ; @src line=9 col=10 op=release
    ; @src line=9 col=10 op=const_str
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #1
    sub x9, x29, #808
    str x1, [x9]
    sub x9, x29, #800
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #792
    ldr x1, [x9]
    sub x9, x29, #784
    ldr x2, [x9]
    sub x9, x29, #808
    ldr x3, [x9]
    sub x9, x29, #800
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #824
    str x1, [x9]
    sub x9, x29, #816
    str x2, [x9]
    ; @src line=9 col=10 op=release
    ; @src line=9 col=10 op=load_local
    sub x9, x29, #1952
    ldr x0, [x9]
    sub x9, x29, #832
    str x0, [x9]
    ; @src line=9 col=10 op=cast
    sub x9, x29, #832
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_76
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_object_76:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_79
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_80
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_81
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_82
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_83
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_84
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_85
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_86
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_87
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_88
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_89
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_90
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_91
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_92
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_93
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_94
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_95
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_96
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_97
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_98
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_99
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_100
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_101
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_102
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_103
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_104
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_105
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_106
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_107
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_108
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_109
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_110
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_111
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_112
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_113
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_114
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_115
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_116
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_117
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_118
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_119
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_120
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_121
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_122
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_123
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_124
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_125
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_126
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_127
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_128
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_129
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_130
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_131
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_132
    mov x10, #98
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_133
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_134
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_135
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_136
    b _eir_main_mixed_string_no_match_77
_eir_main_mixed_string_Exception_79:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateException_80:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateMalformedIntervalStringException_81:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_CachingIterator_82:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_RecursiveCachingIterator_83:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_Error_84:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ArithmeticError_85:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_RuntimeException_86:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_OutOfBoundsException_87:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateError_88:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_LogicException_89:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_BadFunctionCallException_90:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_SplFileInfo_91:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_PharFileInfo_92:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionClass_93:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateRangeError_94:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionFunctionAbstract_95:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionNamedType_96:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_FiberError_97:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionMethod_98:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_OverflowException_99:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionEnumBackedCase_100:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionClassConstant_101:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DomainException_102:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionIntersectionType_103:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_UnhandledMatchError_104:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateMalformedPeriodStringException_105:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionProperty_106:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionUnionType_107:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateInvalidTimeZoneException_108:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DirectoryIterator_109:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_FilesystemIterator_110:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_GlobIterator_111:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_TypeError_112:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionException_113:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_SplFileObject_114:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_InvalidArgumentException_115:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionFunction_116:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateMalformedStringException_117:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_RangeException_118:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_JsonException_119:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateUnknownException_120:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_UnderflowException_121:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_PharData_122:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionParameter_123:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_BadMethodCallException_124:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_SplTempFileObject_125:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_UnexpectedValueException_126:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_LengthException_127:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ValueError_128:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateInvalidOperationException_129:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_DateObjectError_130:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_RecursiveDirectoryIterator_131:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_Phar_132:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionObject_133:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionEnum_134:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_ReflectionEnumUnitCase_135:
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
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_OutOfRangeException_136:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_78
_eir_main_mixed_string_no_match_77:
    mov x0, #2
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_78:
    sub x9, x29, #848
    str x1, [x9]
    sub x9, x29, #840
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #824
    ldr x1, [x9]
    sub x9, x29, #816
    ldr x2, [x9]
    sub x9, x29, #848
    ldr x3, [x9]
    sub x9, x29, #840
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #864
    str x1, [x9]
    sub x9, x29, #856
    str x2, [x9]
    ; @src line=9 col=10 op=release
    ; @src line=9 col=10 op=release
    sub x9, x29, #848
    ldr x1, [x9]
    sub x9, x29, #840
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=9 col=10 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    sub x9, x29, #880
    str x1, [x9]
    sub x9, x29, #872
    str x2, [x9]
    ; @src line=9 col=10 op=str_concat
    sub x9, x29, #864
    ldr x1, [x9]
    sub x9, x29, #856
    ldr x2, [x9]
    sub x9, x29, #880
    ldr x3, [x9]
    sub x9, x29, #872
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #896
    str x1, [x9]
    sub x9, x29, #888
    str x2, [x9]
    ; @src line=9 col=10 op=release
    ; @src line=9 col=5 end=9:9 op=echo_value
    sub x9, x29, #896
    ldr x1, [x9]
    sub x9, x29, #888
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=9 col=5 end=9:9 op=release
    b _eir_main_foreach_next_1
    ; @block name=ternary.then
_eir_main_ternary_then_7:
    ; @src line=8 col=41 op=const_str
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #1
    sub x9, x29, #496
    str x1, [x9]
    sub x9, x29, #488
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=acquire
    sub x9, x29, #496
    ldr x1, [x9]
    sub x9, x29, #488
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #512
    str x1, [x9]
    sub x9, x29, #504
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=load_local
    sub x9, x29, #1992
    ldr x1, [x9]
    sub x9, x29, #1984
    ldr x2, [x9]
    sub x9, x29, #528
    str x1, [x9]
    sub x9, x29, #520
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=release
    sub x9, x29, #528
    ldr x1, [x9]
    sub x9, x29, #520
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=8 col=39 end=8:40 op=store_local
    sub x9, x29, #512
    ldr x1, [x9]
    sub x9, x29, #504
    ldr x2, [x9]
    sub x9, x29, #1992
    str x1, [x9]
    sub x9, x29, #1984
    str x2, [x9]
    b _eir_main_ternary_merge_9
    ; @block name=ternary.else
_eir_main_ternary_else_8:
    ; @src line=8 col=47 op=const_str
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #2
    sub x9, x29, #544
    str x1, [x9]
    sub x9, x29, #536
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=acquire
    sub x9, x29, #544
    ldr x1, [x9]
    sub x9, x29, #536
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #560
    str x1, [x9]
    sub x9, x29, #552
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=load_local
    sub x9, x29, #1992
    ldr x1, [x9]
    sub x9, x29, #1984
    ldr x2, [x9]
    sub x9, x29, #576
    str x1, [x9]
    sub x9, x29, #568
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=release
    sub x9, x29, #576
    ldr x1, [x9]
    sub x9, x29, #568
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=8 col=39 end=8:40 op=store_local
    sub x9, x29, #560
    ldr x1, [x9]
    sub x9, x29, #552
    ldr x2, [x9]
    sub x9, x29, #1992
    str x1, [x9]
    sub x9, x29, #1984
    str x2, [x9]
    b _eir_main_ternary_merge_9
    ; @block name=ternary.merge
_eir_main_ternary_merge_9:
    ; @src line=8 col=39 end=8:40 op=load_local
    sub x9, x29, #1992
    ldr x1, [x9]
    sub x9, x29, #1984
    ldr x2, [x9]
    sub x9, x29, #592
    str x1, [x9]
    sub x9, x29, #584
    str x2, [x9]
    ; @src line=8 col=39 end=8:40 op=unset_local
    sub x9, x29, #1992
    str xzr, [x9]
    sub x9, x29, #1984
    str xzr, [x9]
    ; @src line=8 col=21 end=8:22 op=acquire
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #608
    str x1, [x9]
    sub x9, x29, #600
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=load_local
    sub x9, x29, #1976
    ldr x1, [x9]
    sub x9, x29, #1968
    ldr x2, [x9]
    sub x9, x29, #624
    str x1, [x9]
    sub x9, x29, #616
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=release
    sub x9, x29, #624
    ldr x1, [x9]
    sub x9, x29, #616
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=8 col=21 end=8:22 op=store_local
    sub x9, x29, #608
    ldr x1, [x9]
    sub x9, x29, #600
    ldr x2, [x9]
    sub x9, x29, #1976
    str x1, [x9]
    sub x9, x29, #1968
    str x2, [x9]
    ; @src line=8 col=21 end=8:22 op=release
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    b _eir_main_ternary_merge_6
    ; @block name=ternary.then
_eir_main_ternary_then_10:
    ; @src line=18 col=47 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #3
    sub x9, x29, #1152
    str x1, [x9]
    sub x9, x29, #1144
    str x2, [x9]
    ; @src line=18 col=45 end=18:46 op=acquire
    sub x9, x29, #1152
    ldr x1, [x9]
    sub x9, x29, #1144
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1168
    str x1, [x9]
    sub x9, x29, #1160
    str x2, [x9]
    ; @src line=18 col=45 end=18:46 op=store_local
    sub x9, x29, #1168
    ldr x1, [x9]
    sub x9, x29, #1160
    ldr x2, [x9]
    sub x9, x29, #2040
    str x1, [x9]
    sub x9, x29, #2032
    str x2, [x9]
    b _eir_main_ternary_merge_12
    ; @block name=ternary.else
_eir_main_ternary_else_11:
    ; @src line=18 col=55 op=const_str
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #2
    sub x9, x29, #1184
    str x1, [x9]
    sub x9, x29, #1176
    str x2, [x9]
    ; @src line=18 col=45 end=18:46 op=acquire
    sub x9, x29, #1184
    ldr x1, [x9]
    sub x9, x29, #1176
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1200
    str x1, [x9]
    sub x9, x29, #1192
    str x2, [x9]
    ; @src line=18 col=45 end=18:46 op=store_local
    sub x9, x29, #1200
    ldr x1, [x9]
    sub x9, x29, #1192
    ldr x2, [x9]
    sub x9, x29, #2040
    str x1, [x9]
    sub x9, x29, #2032
    str x2, [x9]
    b _eir_main_ternary_merge_12
    ; @block name=ternary.merge
_eir_main_ternary_merge_12:
    ; @src line=18 col=45 end=18:46 op=load_local
    sub x9, x29, #2040
    ldr x1, [x9]
    sub x9, x29, #2032
    ldr x2, [x9]
    sub x9, x29, #1216
    str x1, [x9]
    sub x9, x29, #1208
    str x2, [x9]
    ; @src line=18 col=45 end=18:46 op=unset_local
    sub x9, x29, #2040
    str xzr, [x9]
    sub x9, x29, #2032
    str xzr, [x9]
    ; @src line=18 col=1 end=18:5 op=echo_value
    sub x9, x29, #1216
    ldr x1, [x9]
    sub x9, x29, #1208
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=18 col=1 end=18:5 op=release
    sub x9, x29, #1216
    ldr x1, [x9]
    sub x9, x29, #1208
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=61 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    sub x9, x29, #1232
    str x1, [x9]
    sub x9, x29, #1224
    str x2, [x9]
    ; @src line=18 col=1 end=18:5 op=echo_value
    sub x9, x29, #1232
    ldr x1, [x9]
    sub x9, x29, #1224
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=6 op=const_str
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #13
    sub x9, x29, #1248
    str x1, [x9]
    sub x9, x29, #1240
    str x2, [x9]
    ; @src line=19 col=1 end=19:5 op=echo_value
    sub x9, x29, #1248
    ldr x1, [x9]
    sub x9, x29, #1240
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=37 end=19:41 op=load_local
    sub x9, x29, #2024
    ldr x0, [x9]
    sub x9, x29, #1256
    str x0, [x9]
    ; @src line=19 col=23 end=19:42 op=runtime_call
    sub x9, x29, #1256
    ldr x0, [x9]
    bl __rt_array_is_list
    mov x21, x0
    ; @src line=19 col=23 end=19:42 op=nop
    mov x0, x21
    cbnz x0, _eir_main_ternary_then_13
    b _eir_main_ternary_else_14
    ; @block name=ternary.then
_eir_main_ternary_then_13:
    ; @src line=19 col=45 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #3
    sub x9, x29, #1280
    str x1, [x9]
    sub x9, x29, #1272
    str x2, [x9]
    ; @src line=19 col=43 end=19:44 op=acquire
    sub x9, x29, #1280
    ldr x1, [x9]
    sub x9, x29, #1272
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1296
    str x1, [x9]
    sub x9, x29, #1288
    str x2, [x9]
    ; @src line=19 col=43 end=19:44 op=store_local
    sub x9, x29, #1296
    ldr x1, [x9]
    sub x9, x29, #1288
    ldr x2, [x9]
    sub x9, x29, #2056
    str x1, [x9]
    sub x9, x29, #2048
    str x2, [x9]
    b _eir_main_ternary_merge_15
    ; @block name=ternary.else
_eir_main_ternary_else_14:
    ; @src line=19 col=53 op=const_str
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #2
    sub x9, x29, #1312
    str x1, [x9]
    sub x9, x29, #1304
    str x2, [x9]
    ; @src line=19 col=43 end=19:44 op=acquire
    sub x9, x29, #1312
    ldr x1, [x9]
    sub x9, x29, #1304
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1328
    str x1, [x9]
    sub x9, x29, #1320
    str x2, [x9]
    ; @src line=19 col=43 end=19:44 op=store_local
    sub x9, x29, #1328
    ldr x1, [x9]
    sub x9, x29, #1320
    ldr x2, [x9]
    sub x9, x29, #2056
    str x1, [x9]
    sub x9, x29, #2048
    str x2, [x9]
    b _eir_main_ternary_merge_15
    ; @block name=ternary.merge
_eir_main_ternary_merge_15:
    ; @src line=19 col=43 end=19:44 op=load_local
    sub x9, x29, #2056
    ldr x1, [x9]
    sub x9, x29, #2048
    ldr x2, [x9]
    sub x9, x29, #1344
    str x1, [x9]
    sub x9, x29, #1336
    str x2, [x9]
    ; @src line=19 col=43 end=19:44 op=unset_local
    sub x9, x29, #2056
    str xzr, [x9]
    sub x9, x29, #2048
    str xzr, [x9]
    ; @src line=19 col=1 end=19:5 op=echo_value
    sub x9, x29, #1344
    ldr x1, [x9]
    sub x9, x29, #1336
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=1 end=19:5 op=release
    sub x9, x29, #1344
    ldr x1, [x9]
    sub x9, x29, #1336
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=59 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    sub x9, x29, #1360
    str x1, [x9]
    sub x9, x29, #1352
    str x2, [x9]
    ; @src line=19 col=1 end=19:5 op=echo_value
    sub x9, x29, #1360
    ldr x1, [x9]
    sub x9, x29, #1352
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=22 col=1 end=22:10 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=13 end=22:14 op=hash_new
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #1368
    str x0, [x9]
    ; @src line=22 col=14 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #4
    sub x9, x29, #1384
    str x1, [x9]
    sub x9, x29, #1376
    str x2, [x9]
    ; @src line=22 col=24 op=const_str
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #9
    sub x9, x29, #1400
    str x1, [x9]
    sub x9, x29, #1392
    str x2, [x9]
    ; @src line=22 col=13 end=22:14 op=hash_set
    sub x9, x29, #1384
    ldr x1, [x9]
    sub x9, x29, #1376
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1400
    ldr x1, [x9]
    sub x9, x29, #1392
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1368
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1368
    str x0, [x9]
    ; @src line=22 col=37 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #4
    sub x9, x29, #1416
    str x1, [x9]
    sub x9, x29, #1408
    str x2, [x9]
    ; @src line=22 col=47 end=22:51 op=const_i64
    mov x0, #5432
    mov x21, x0
    ; @src line=22 col=13 end=22:14 op=hash_set
    sub x9, x29, #1416
    ldr x1, [x9]
    sub x9, x29, #1408
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1368
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1368
    str x0, [x9]
    ; @src line=22 col=53 op=const_str
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #3
    sub x9, x29, #1440
    str x1, [x9]
    sub x9, x29, #1432
    str x2, [x9]
    ; @src line=22 col=62 op=const_str
    adrp x1, _str_29@PAGE
    add x1, x1, _str_29@PAGEOFF
    mov x2, #3
    sub x9, x29, #1456
    str x1, [x9]
    sub x9, x29, #1448
    str x2, [x9]
    ; @src line=22 col=13 end=22:14 op=hash_set
    sub x9, x29, #1440
    ldr x1, [x9]
    sub x9, x29, #1432
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1456
    ldr x1, [x9]
    sub x9, x29, #1448
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1368
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1368
    str x0, [x9]
    ; @src line=22 col=1 end=22:10 op=acquire
    sub x9, x29, #1368
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1464
    str x0, [x9]
    ; @src line=22 col=1 end=22:10 op=store_local
    sub x9, x29, #1464
    ldr x0, [x9]
    sub x9, x29, #2064
    str x0, [x9]
    ; @src line=22 col=1 end=22:10 op=release
    sub x9, x29, #1368
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=23 col=1 end=23:11 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=23 col=14 end=23:15 op=hash_new
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #1472
    str x0, [x9]
    ; @src line=23 col=15 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #4
    sub x9, x29, #1488
    str x1, [x9]
    sub x9, x29, #1480
    str x2, [x9]
    ; @src line=23 col=25 end=23:29 op=const_i64
    mov x0, #6379
    mov x21, x0
    ; @src line=23 col=14 end=23:15 op=hash_set
    sub x9, x29, #1488
    ldr x1, [x9]
    sub x9, x29, #1480
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1472
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1472
    str x0, [x9]
    ; @src line=23 col=31 op=const_str
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #3
    sub x9, x29, #1512
    str x1, [x9]
    sub x9, x29, #1504
    str x2, [x9]
    ; @src line=23 col=40 op=const_str
    adrp x1, _str_30@PAGE
    add x1, x1, _str_30@PAGEOFF
    mov x2, #2
    sub x9, x29, #1528
    str x1, [x9]
    sub x9, x29, #1520
    str x2, [x9]
    ; @src line=23 col=14 end=23:15 op=hash_set
    sub x9, x29, #1512
    ldr x1, [x9]
    sub x9, x29, #1504
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1528
    ldr x1, [x9]
    sub x9, x29, #1520
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1472
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1472
    str x0, [x9]
    ; @src line=23 col=1 end=23:11 op=acquire
    sub x9, x29, #1472
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1536
    str x0, [x9]
    ; @src line=23 col=1 end=23:11 op=store_local
    sub x9, x29, #1536
    ldr x0, [x9]
    sub x9, x29, #2072
    str x0, [x9]
    ; @src line=23 col=1 end=23:11 op=release
    sub x9, x29, #1472
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=24 col=1 end=24:8 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=24 col=25 end=24:34 op=load_local
    sub x9, x29, #2064
    ldr x0, [x9]
    sub x9, x29, #1544
    str x0, [x9]
    ; @src line=24 col=36 end=24:46 op=load_local
    sub x9, x29, #2072
    ldr x0, [x9]
    sub x9, x29, #1552
    str x0, [x9]
    ; @src line=24 col=11 end=24:47 op=runtime_call
    sub x9, x29, #1544
    ldr x0, [x9]
    str x0, [sp, #-16]!
    sub x9, x29, #1552
    ldr x0, [x9]
    mov x1, x0
    ldr x0, [sp], #16
    bl __rt_array_replace
    sub x9, x29, #1560
    str x0, [x9]
    ; @src line=24 col=11 end=24:47 op=nop
    ; @src line=24 col=11 end=24:47 op=nop
    ; @src line=24 col=1 end=24:8 op=acquire
    sub x9, x29, #1560
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1568
    str x0, [x9]
    ; @src line=24 col=1 end=24:8 op=store_local
    sub x9, x29, #1568
    ldr x0, [x9]
    sub x9, x29, #2080
    str x0, [x9]
    ; @src line=24 col=1 end=24:8 op=release
    sub x9, x29, #1560
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=25 col=1 end=25:8 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #2080
    ldr x0, [x9]
    sub x9, x29, #1576
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=iter_start
    sub x9, x29, #1576
    ldr x0, [x9]
    sub x9, x29, #1648
    str x0, [x9]
    mov x0, #0
    sub x9, x29, #1640
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
    sub x9, x29, #1664
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #1664
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1672
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #1672
    ldr x0, [x9]
    sub x9, x29, #2088
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1664
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
    sub x9, x29, #1688
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #1688
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1696
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #1696
    ldr x0, [x9]
    sub x9, x29, #2096
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1688
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_16
    ; @block name=foreach.next
_eir_main_foreach_next_16:
    ; @src line=25 col=10 end=25:17 op=iter_next
    sub x9, x29, #1648
    ldr x0, [x9]
    sub x9, x29, #1640
    ldr x1, [x9]
    bl __rt_hash_iter_next
    cmn x0, #1
    sub x9, x29, #1640
    str x0, [x9]
    sub x9, x29, #1632
    str x1, [x9]
    sub x9, x29, #1624
    str x2, [x9]
    sub x9, x29, #1616
    str x3, [x9]
    sub x9, x29, #1608
    str x4, [x9]
    sub x9, x29, #1600
    str x5, [x9]
    sub x9, x29, #1592
    str x6, [x9]
    cset x0, ne
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_17
    b _eir_main_foreach_exit_18
    ; @block name=foreach.body
_eir_main_foreach_body_17:
    ; @src line=25 col=10 end=25:17 op=iter_current_key
    sub x9, x29, #1632
    ldr x1, [x9]
    sub x9, x29, #1624
    ldr x2, [x9]
    cmn x2, #1
    b.ne _eir_main_iter_hash_key_string_137
    mov x0, #0
    mov x2, xzr
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_key_done_138
_eir_main_iter_hash_key_string_137:
    mov x0, #1
    bl __rt_mixed_from_value
_eir_main_iter_hash_key_done_138:
    sub x9, x29, #1712
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #1712
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1720
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #2088
    ldr x0, [x9]
    sub x9, x29, #1728
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1728
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #1720
    ldr x0, [x9]
    sub x9, x29, #2088
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1712
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=iter_current_value
    sub x9, x29, #1600
    ldr x5, [x9]
    sub x9, x29, #1616
    ldr x3, [x9]
    sub x9, x29, #1608
    ldr x4, [x9]
    mov x1, x3
    mov x3, x5
    bl __rt_deref_if_reference
    mov x5, x3
    mov x3, x1
    cmp x5, #7
    b.eq _eir_main_iter_hash_value_inspect_box_139
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_value_boxed_140
_eir_main_iter_hash_value_inspect_box_139:
    str x3, [sp, #-16]!
    mov x0, x3
    bl __rt_heap_kind
    cmp x0, #5
    b.eq _eir_main_iter_hash_value_reuse_box_141
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
    b _eir_main_iter_hash_value_tagged_done_142
_eir_main_iter_hash_value_reuse_box_141:
    ldr x0, [sp], #16
    bl __rt_incref
_eir_main_iter_hash_value_tagged_done_142:
_eir_main_iter_hash_value_boxed_140:
    sub x9, x29, #1736
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=acquire
    sub x9, x29, #1736
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1744
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=load_local
    sub x9, x29, #2096
    ldr x0, [x9]
    sub x9, x29, #1752
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1752
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10 end=25:17 op=store_local
    sub x9, x29, #1744
    ldr x0, [x9]
    sub x9, x29, #2096
    str x0, [x9]
    ; @src line=25 col=10 end=25:17 op=release
    sub x9, x29, #1736
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=26 col=5 end=26:9 op=concat_reset
    sub x9, x29, #2120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=10 op=const_str
    adrp x1, _str_31@PAGE
    add x1, x1, _str_31@PAGEOFF
    mov x2, #2
    sub x9, x29, #1768
    str x1, [x9]
    sub x9, x29, #1760
    str x2, [x9]
    ; @src line=26 col=10 op=load_local
    sub x9, x29, #2088
    ldr x0, [x9]
    sub x9, x29, #1776
    str x0, [x9]
    ; @src line=26 col=10 op=cast
    sub x9, x29, #1776
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_143
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_object_143:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_146
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_147
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_148
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_149
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_150
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_151
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_152
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_153
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_154
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_155
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_156
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_157
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_158
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_159
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_160
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_161
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_162
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_163
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_164
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_165
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_166
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_167
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_168
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_169
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_170
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_171
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_172
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_173
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_174
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_175
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_176
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_177
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_178
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_179
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_180
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_181
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_182
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_183
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_184
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_185
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_186
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_187
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_188
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_189
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_190
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_191
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_192
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_193
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_194
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_195
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_196
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_197
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_198
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_199
    mov x10, #98
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_200
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_201
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_202
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_203
    b _eir_main_mixed_string_no_match_144
_eir_main_mixed_string_Exception_146:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateException_147:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateMalformedIntervalStringException_148:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_CachingIterator_149:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_RecursiveCachingIterator_150:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_Error_151:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ArithmeticError_152:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_RuntimeException_153:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_OutOfBoundsException_154:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateError_155:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_LogicException_156:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_BadFunctionCallException_157:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_SplFileInfo_158:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_PharFileInfo_159:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionClass_160:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateRangeError_161:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionFunctionAbstract_162:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionNamedType_163:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_FiberError_164:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionMethod_165:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_OverflowException_166:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionEnumBackedCase_167:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionClassConstant_168:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DomainException_169:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionIntersectionType_170:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_UnhandledMatchError_171:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateMalformedPeriodStringException_172:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionProperty_173:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionUnionType_174:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateInvalidTimeZoneException_175:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DirectoryIterator_176:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_FilesystemIterator_177:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_GlobIterator_178:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_TypeError_179:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionException_180:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_SplFileObject_181:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_InvalidArgumentException_182:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionFunction_183:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateMalformedStringException_184:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_RangeException_185:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_JsonException_186:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateUnknownException_187:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_UnderflowException_188:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_PharData_189:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionParameter_190:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_BadMethodCallException_191:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_SplTempFileObject_192:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_UnexpectedValueException_193:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_LengthException_194:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ValueError_195:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateInvalidOperationException_196:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_DateObjectError_197:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_RecursiveDirectoryIterator_198:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_Phar_199:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionObject_200:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionEnum_201:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_ReflectionEnumUnitCase_202:
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
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_OutOfRangeException_203:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_145
_eir_main_mixed_string_no_match_144:
    mov x0, #2
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_145:
    sub x9, x29, #1792
    str x1, [x9]
    sub x9, x29, #1784
    str x2, [x9]
    ; @src line=26 col=10 op=str_concat
    sub x9, x29, #1768
    ldr x1, [x9]
    sub x9, x29, #1760
    ldr x2, [x9]
    sub x9, x29, #1792
    ldr x3, [x9]
    sub x9, x29, #1784
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1808
    str x1, [x9]
    sub x9, x29, #1800
    str x2, [x9]
    ; @src line=26 col=10 op=release
    sub x9, x29, #1792
    ldr x1, [x9]
    sub x9, x29, #1784
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=10 op=const_str
    adrp x1, _str_32@PAGE
    add x1, x1, _str_32@PAGEOFF
    mov x2, #3
    sub x9, x29, #1824
    str x1, [x9]
    sub x9, x29, #1816
    str x2, [x9]
    ; @src line=26 col=10 op=str_concat
    sub x9, x29, #1808
    ldr x1, [x9]
    sub x9, x29, #1800
    ldr x2, [x9]
    sub x9, x29, #1824
    ldr x3, [x9]
    sub x9, x29, #1816
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1840
    str x1, [x9]
    sub x9, x29, #1832
    str x2, [x9]
    ; @src line=26 col=10 op=release
    ; @src line=26 col=10 op=load_local
    sub x9, x29, #2096
    ldr x0, [x9]
    sub x9, x29, #1848
    str x0, [x9]
    ; @src line=26 col=10 op=cast
    sub x9, x29, #1848
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_204
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_object_204:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_207
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_208
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_209
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_210
    mov x10, #8
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_211
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_212
    mov x10, #15
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_213
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_214
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_215
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_216
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_217
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_218
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_219
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_220
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_221
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_222
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_223
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_224
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_225
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_226
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_227
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_228
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_229
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_230
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_231
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_232
    mov x10, #48
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_233
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_234
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_235
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_236
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_237
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_238
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_239
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_240
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_241
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_242
    mov x10, #60
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_243
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_244
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_245
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_246
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_247
    mov x10, #71
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_248
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_249
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_250
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_251
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_252
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_253
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_254
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_255
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_256
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_257
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_258
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_259
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_260
    mov x10, #98
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_261
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_262
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_263
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_264
    b _eir_main_mixed_string_no_match_205
_eir_main_mixed_string_Exception_207:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateException_208:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateMalformedIntervalStringException_209:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_CachingIterator_210:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_RecursiveCachingIterator_211:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_Error_212:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ArithmeticError_213:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_RuntimeException_214:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_OutOfBoundsException_215:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateError_216:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_LogicException_217:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_BadFunctionCallException_218:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_SplFileInfo_219:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_PharFileInfo_220:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionClass_221:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateRangeError_222:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionFunctionAbstract_223:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionNamedType_224:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_FiberError_225:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionMethod_226:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_OverflowException_227:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionEnumBackedCase_228:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionClassConstant_229:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DomainException_230:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionIntersectionType_231:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_UnhandledMatchError_232:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateMalformedPeriodStringException_233:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionProperty_234:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionUnionType_235:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateInvalidTimeZoneException_236:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DirectoryIterator_237:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_FilesystemIterator_238:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_GlobIterator_239:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_TypeError_240:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionException_241:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_SplFileObject_242:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_InvalidArgumentException_243:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionFunction_244:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateMalformedStringException_245:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_RangeException_246:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_JsonException_247:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateUnknownException_248:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_UnderflowException_249:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_PharData_250:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionParameter_251:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_BadMethodCallException_252:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_SplTempFileObject_253:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_UnexpectedValueException_254:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_LengthException_255:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ValueError_256:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateInvalidOperationException_257:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_DateObjectError_258:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_RecursiveDirectoryIterator_259:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_Phar_260:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionObject_261:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionEnum_262:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_ReflectionEnumUnitCase_263:
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
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_OutOfRangeException_264:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_206
_eir_main_mixed_string_no_match_205:
    mov x0, #2
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_206:
    sub x9, x29, #1864
    str x1, [x9]
    sub x9, x29, #1856
    str x2, [x9]
    ; @src line=26 col=10 op=str_concat
    sub x9, x29, #1840
    ldr x1, [x9]
    sub x9, x29, #1832
    ldr x2, [x9]
    sub x9, x29, #1864
    ldr x3, [x9]
    sub x9, x29, #1856
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1880
    str x1, [x9]
    sub x9, x29, #1872
    str x2, [x9]
    ; @src line=26 col=10 op=release
    ; @src line=26 col=10 op=release
    sub x9, x29, #1864
    ldr x1, [x9]
    sub x9, x29, #1856
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=10 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #1
    sub x9, x29, #1896
    str x1, [x9]
    sub x9, x29, #1888
    str x2, [x9]
    ; @src line=26 col=10 op=str_concat
    sub x9, x29, #1880
    ldr x1, [x9]
    sub x9, x29, #1872
    ldr x2, [x9]
    sub x9, x29, #1896
    ldr x3, [x9]
    sub x9, x29, #1888
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1912
    str x1, [x9]
    sub x9, x29, #1904
    str x2, [x9]
    ; @src line=26 col=10 op=release
    ; @src line=26 col=5 end=26:9 op=echo_value
    sub x9, x29, #1912
    ldr x1, [x9]
    sub x9, x29, #1904
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=26 col=5 end=26:9 op=release
    b _eir_main_foreach_next_16
    ; @block name=foreach.exit
_eir_main_foreach_exit_18:
    ; @src line=25 col=10 end=25:17 op=nop

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $pairs
    sub x9, x29, #1928
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_265
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_265:
    ; epilogue cleanup $__elephc_foreach_destructure_6_1
    sub x9, x29, #1936
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_266
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_266:
    ; epilogue cleanup $a
    sub x9, x29, #1944
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_267
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_267:
    ; epilogue cleanup $b
    sub x9, x29, #1952
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_268
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_268:
    ; epilogue cleanup $__eir_tmp0
    sub x9, x29, #1976
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $__eir_tmp1
    sub x9, x29, #1992
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $rel
    sub x9, x29, #2008
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $list
    sub x9, x29, #2016
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_269
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_269:
    ; epilogue cleanup $map
    sub x9, x29, #2024
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_270
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_270:
    ; epilogue cleanup $__eir_tmp2
    sub x9, x29, #2040
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $__eir_tmp3
    sub x9, x29, #2056
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $defaults
    sub x9, x29, #2064
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_271
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_271:
    ; epilogue cleanup $overrides
    sub x9, x29, #2072
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_272
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_272:
    ; epilogue cleanup $config
    sub x9, x29, #2080
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_273
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_273:
    ; epilogue cleanup $key
    sub x9, x29, #2088
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_274
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_274:
    ; epilogue cleanup $value
    sub x9, x29, #2096
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_275
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_275:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #2104
    ldr x21, [x9]
    sub x9, x29, #2112
    ldr x22, [x9]
    mov x9, sp
    add x9, x9, #2128
    ldp x29, x30, [x9]
    add sp, sp, #2144
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_276
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_276:
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
    .ascii "img2.png"
.globl _str_4
_str_4:
    .ascii "img10.png"
.globl _str_5
_str_5:
    .ascii "v1.9"
.globl _str_6
_str_6:
    .ascii "v1.10"
.globl _str_7
_str_7:
    .ascii "file001"
.globl _str_8
_str_8:
    .ascii "file10"
.globl _str_9
_str_9:
    .ascii "IMG10 vs img2 (ci): "
.globl _str_10
_str_10:
    .ascii "IMG10"
.globl _str_11
_str_11:
    .ascii "img2"
.globl _str_12
_str_12:
    .ascii "\n"
.globl _str_13
_str_13:
    .ascii "a"
.globl _str_14
_str_14:
    .ascii "b"
.globl _str_15
_str_15:
    .ascii "c"
.globl _str_16
_str_16:
    .ascii "host"
.globl _str_17
_str_17:
    .ascii "localhost"
.globl _str_18
_str_18:
    .ascii "port"
.globl _str_19
_str_19:
    .ascii "list is list? "
.globl _str_20
_str_20:
    .ascii "<"
.globl _str_21
_str_21:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_22
_str_22:
    .ascii " "
.globl _str_23
_str_23:
    .ascii ">"
.globl _str_24
_str_24:
    .ascii "=="
.globl _str_25
_str_25:
    .ascii "yes"
.globl _str_26
_str_26:
    .ascii "no"
.globl _str_27
_str_27:
    .ascii "map is list? "
.globl _str_28
_str_28:
    .ascii "ssl"
.globl _str_29
_str_29:
    .ascii "off"
.globl _str_30
_str_30:
    .ascii "on"
.globl _str_31
_str_31:
    .ascii "  "
.globl _str_32
_str_32:
    .ascii " = "
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
    .quad _instanceof_name_class_1
    .quad 9
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 10
    .quad 1
    .quad 0
    .quad _instanceof_name_class_14
    .quad 5
    .quad 14
    .quad 0
    .quad _instanceof_name_class_abs_14
    .quad 6
    .quad 14
    .quad 0
    .quad _instanceof_name_class_15
    .quad 15
    .quad 15
    .quad 0
    .quad _instanceof_name_class_abs_15
    .quad 16
    .quad 15
    .quad 0
    .quad _instanceof_name_class_16
    .quad 16
    .quad 16
    .quad 0
    .quad _instanceof_name_class_abs_16
    .quad 17
    .quad 16
    .quad 0
    .quad _instanceof_name_class_17
    .quad 20
    .quad 17
    .quad 0
    .quad _instanceof_name_class_abs_17
    .quad 21
    .quad 17
    .quad 0
    .quad _instanceof_name_class_19
    .quad 14
    .quad 19
    .quad 0
    .quad _instanceof_name_class_abs_19
    .quad 15
    .quad 19
    .quad 0
    .quad _instanceof_name_class_43
    .quad 19
    .quad 43
    .quad 0
    .quad _instanceof_name_class_abs_43
    .quad 20
    .quad 43
    .quad 0
    .quad _instanceof_name_class_55
    .quad 9
    .quad 55
    .quad 0
    .quad _instanceof_name_class_abs_55
    .quad 10
    .quad 55
    .quad 0
    .quad _instanceof_name_class_56
    .quad 19
    .quad 56
    .quad 0
    .quad _instanceof_name_class_abs_56
    .quad 20
    .quad 56
    .quad 0
    .quad _instanceof_name_class_60
    .quad 24
    .quad 60
    .quad 0
    .quad _instanceof_name_class_abs_60
    .quad 25
    .quad 60
    .quad 0
    .quad _instanceof_name_class_69
    .quad 13
    .quad 69
    .quad 0
    .quad _instanceof_name_class_abs_69
    .quad 14
    .quad 69
    .quad 0
    .quad _instanceof_name_class_85
    .quad 8
    .quad 85
    .quad 0
    .quad _instanceof_name_class_abs_85
    .quad 9
    .quad 85
    .quad 0
    .quad _instanceof_name_class_90
    .quad 10
    .quad 90
    .quad 0
    .quad _instanceof_name_class_abs_90
    .quad 11
    .quad 90
    .quad 0
    .quad _instanceof_name_class_102
    .quad 19
    .quad 102
    .quad 0
    .quad _instanceof_name_class_abs_102
    .quad 20
    .quad 102
    .quad 0
    .quad _instanceof_name_interface_6
    .quad 9
    .quad 6
    .quad 1
    .quad _instanceof_name_interface_abs_6
    .quad 10
    .quad 6
    .quad 1
    .quad _instanceof_name_interface_13
    .quad 10
    .quad 13
    .quad 1
    .quad _instanceof_name_interface_abs_13
    .quad 11
    .quad 13
    .quad 1
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "Exception"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\Exception"
.globl _instanceof_name_class_14
_instanceof_name_class_14:
    .ascii "Error"
.globl _instanceof_name_class_abs_14
_instanceof_name_class_abs_14:
    .ascii "\\Error"
.globl _instanceof_name_class_15
_instanceof_name_class_15:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_15
_instanceof_name_class_abs_15:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_16
_instanceof_name_class_16:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_16
_instanceof_name_class_abs_16:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_17
_instanceof_name_class_17:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_17
_instanceof_name_class_abs_17:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_19
_instanceof_name_class_19:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_19
_instanceof_name_class_abs_19:
    .ascii "\\LogicException"
.globl _instanceof_name_class_43
_instanceof_name_class_43:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_43
_instanceof_name_class_abs_43:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_55
_instanceof_name_class_55:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_55
_instanceof_name_class_abs_55:
    .ascii "\\TypeError"
.globl _instanceof_name_class_56
_instanceof_name_class_56:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_56
_instanceof_name_class_abs_56:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_60
_instanceof_name_class_60:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_60
_instanceof_name_class_abs_60:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_69
_instanceof_name_class_69:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_69
_instanceof_name_class_abs_69:
    .ascii "\\JsonException"
.globl _instanceof_name_class_85
_instanceof_name_class_85:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_85
_instanceof_name_class_abs_85:
    .ascii "\\stdClass"
.globl _instanceof_name_class_90
_instanceof_name_class_90:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_90
_instanceof_name_class_abs_90:
    .ascii "\\ValueError"
.globl _instanceof_name_class_102
_instanceof_name_class_102:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_102
_instanceof_name_class_abs_102:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_interface_6
_instanceof_name_interface_6:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_6
_instanceof_name_interface_abs_6:
    .ascii "\\Throwable"
.globl _instanceof_name_interface_13
_instanceof_name_interface_13:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_13
_instanceof_name_interface_abs_13:
    .ascii "\\Stringable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 103
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_missing
    .quad 0
    .quad _class_name_1
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
    .quad _class_name_missing
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
    .quad _class_name_14
    .quad 5
    .quad _class_name_15
    .quad 15
    .quad _class_name_16
    .quad 16
    .quad _class_name_17
    .quad 20
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
    .quad _class_name_missing
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
    .quad _class_name_55
    .quad 9
    .quad _class_name_56
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_60
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_69
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_85
    .quad 8
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_90
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
    .quad _class_name_102
    .quad 19
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_1
_class_name_1:
    .ascii "Exception"
.globl _class_name_14
_class_name_14:
    .ascii "Error"
.globl _class_name_15
_class_name_15:
    .ascii "ArithmeticError"
.globl _class_name_16
_class_name_16:
    .ascii "RuntimeException"
.globl _class_name_17
_class_name_17:
    .ascii "OutOfBoundsException"
.globl _class_name_19
_class_name_19:
    .ascii "LogicException"
.globl _class_name_43
_class_name_43:
    .ascii "UnhandledMatchError"
.globl _class_name_55
_class_name_55:
    .ascii "TypeError"
.globl _class_name_56
_class_name_56:
    .ascii "ReflectionException"
.globl _class_name_60
_class_name_60:
    .ascii "InvalidArgumentException"
.globl _class_name_69
_class_name_69:
    .ascii "JsonException"
.globl _class_name_85
_class_name_85:
    .ascii "stdClass"
.globl _class_name_90
_class_name_90:
    .ascii "ValueError"
.globl _class_name_102
_class_name_102:
    .ascii "OutOfRangeException"
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
    .quad 5
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 30
.globl _generator_class_id
_generator_class_id:
    .quad 9
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 76
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 77
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 91
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 32
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 14
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 19
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 16
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 102
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 17
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 60
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 55
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 90
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 56
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 15
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_6
    .quad _interface_methods_13
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_1
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
    .quad _class_interfaces_14
    .quad _class_interfaces_15
    .quad _class_interfaces_16
    .quad _class_interfaces_17
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
    .quad _class_interfaces_missing
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_55
    .quad _class_interfaces_56
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_60
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_69
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
    .quad _class_interfaces_85
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_90
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
    .quad _class_interfaces_102
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_missing
    .quad _class_json_desc_1
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
    .quad _class_json_desc_14
    .quad _class_json_desc_15
    .quad _class_json_desc_16
    .quad _class_json_desc_17
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
    .quad _class_json_desc_missing
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_55
    .quad _class_json_desc_56
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_60
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_69
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
    .quad _class_json_desc_85
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_90
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
    .quad _class_json_desc_102
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 69
.globl _class_parent_ids
_class_parent_ids:
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
    .quad 14
    .quad 1
    .quad 16
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
    .quad 14
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
    .quad 14
    .quad 1
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
    .quad 16
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
    .quad 14
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
    .quad 72
    .quad 72
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
    .quad 72
    .quad 72
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
    .quad 16
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
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 103
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_1
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
    .quad _class_gc_desc_14
    .quad _class_gc_desc_15
    .quad _class_gc_desc_16
    .quad _class_gc_desc_17
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
    .quad _class_gc_desc_missing
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_55
    .quad _class_gc_desc_56
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_60
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_69
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
    .quad _class_gc_desc_85
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_90
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
    .quad _class_gc_desc_102
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_missing
    .quad _class_vtable_1
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
    .quad _class_vtable_14
    .quad _class_vtable_15
    .quad _class_vtable_16
    .quad _class_vtable_17
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
    .quad _class_vtable_missing
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
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_55
    .quad _class_vtable_56
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_60
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_69
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
    .quad _class_vtable_85
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_90
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
    .quad 0
    .quad _class_propinit_1
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_14
    .quad _class_propinit_15
    .quad _class_propinit_16
    .quad _class_propinit_17
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_43
    .quad 0
    .quad 0
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
    .quad _class_propinit_56
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_60
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_69
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_90
    .quad 0
    .quad 0
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
    .quad _class_serprop_missing
    .quad _class_serprop_1
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
    .quad _class_serprop_14
    .quad _class_serprop_15
    .quad _class_serprop_16
    .quad _class_serprop_17
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
    .quad _class_serprop_missing
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
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_55
    .quad _class_serprop_56
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_60
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_69
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
    .quad _class_serprop_85
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_90
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
    .quad _class_serprop_102
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_1
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
    .quad _class_static_vtable_14
    .quad _class_static_vtable_15
    .quad _class_static_vtable_16
    .quad _class_static_vtable_17
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
    .quad _class_static_vtable_missing
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_55
    .quad _class_static_vtable_56
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_60
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_69
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
    .quad _class_static_vtable_85
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_90
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
    .quad _class_static_vtable_102
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_1
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
    .quad _class_callable_methods_14
    .quad _class_callable_methods_15
    .quad _class_callable_methods_16
    .quad _class_callable_methods_17
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
    .quad _class_callable_methods_missing
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_55
    .quad _class_callable_methods_56
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_60
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_69
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
    .quad _class_callable_methods_85
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_90
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
.globl _class_by_name_str_1
_class_by_name_str_1:
    .ascii "Exception"
.globl _class_by_name_str_14
_class_by_name_str_14:
    .ascii "Error"
.globl _class_by_name_str_15
_class_by_name_str_15:
    .ascii "ArithmeticError"
.globl _class_by_name_str_16
_class_by_name_str_16:
    .ascii "RuntimeException"
.globl _class_by_name_str_17
_class_by_name_str_17:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_19
_class_by_name_str_19:
    .ascii "LogicException"
.globl _class_by_name_str_43
_class_by_name_str_43:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_55
_class_by_name_str_55:
    .ascii "TypeError"
.globl _class_by_name_str_56
_class_by_name_str_56:
    .ascii "ReflectionException"
.globl _class_by_name_str_60
_class_by_name_str_60:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_69
_class_by_name_str_69:
    .ascii "JsonException"
.globl _class_by_name_str_85
_class_by_name_str_85:
    .ascii "stdClass"
.globl _class_by_name_str_90
_class_by_name_str_90:
    .ascii "ValueError"
.globl _class_by_name_str_102
_class_by_name_str_102:
    .ascii "OutOfRangeException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 14
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_1
    .quad 9
    .quad 1
    .quad 72
    .quad _class_by_name_str_14
    .quad 5
    .quad 14
    .quad 72
    .quad _class_by_name_str_15
    .quad 15
    .quad 15
    .quad 72
    .quad _class_by_name_str_16
    .quad 16
    .quad 16
    .quad 72
    .quad _class_by_name_str_17
    .quad 20
    .quad 17
    .quad 72
    .quad _class_by_name_str_19
    .quad 14
    .quad 19
    .quad 72
    .quad _class_by_name_str_43
    .quad 19
    .quad 43
    .quad 72
    .quad _class_by_name_str_55
    .quad 9
    .quad 55
    .quad 72
    .quad _class_by_name_str_56
    .quad 19
    .quad 56
    .quad 72
    .quad _class_by_name_str_60
    .quad 24
    .quad 60
    .quad 72
    .quad _class_by_name_str_69
    .quad 13
    .quad 69
    .quad 72
    .quad _class_by_name_str_85
    .quad 8
    .quad 85
    .quad 16
    .quad _class_by_name_str_90
    .quad 10
    .quad 90
    .quad 72
    .quad _class_by_name_str_102
    .quad 19
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
.globl _interface_methods_6
_interface_methods_6:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _interface_methods_13
_interface_methods_13:
    .quad 1
    .quad 0
.globl _class_interfaces_1
_class_interfaces_1:
    .quad 2
    .quad 6
    .quad _class_interface_impl_1_6
    .quad 13
    .quad _class_interface_impl_1_13
.globl _class_interface_impl_1_6
_class_interface_impl_1_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_13
_class_interface_impl_1_13:
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
.globl _class_interfaces_14
_class_interfaces_14:
    .quad 2
    .quad 6
    .quad _class_interface_impl_14_6
    .quad 13
    .quad _class_interface_impl_14_13
.globl _class_interface_impl_14_6
_class_interface_impl_14_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_14_13
_class_interface_impl_14_13:
    .quad 0
.globl _class_json_pname_14_0
_class_json_pname_14_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_14
_class_json_desc_14:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_14_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_14
_class_gc_desc_14:
    .byte 1, 0, 7, 4
.globl _class_serpname_14_0
_class_serpname_14_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_14_1
_class_serpname_14_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_14_2
_class_serpname_14_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_14_3
_class_serpname_14_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_14
_class_serprop_14:
    .quad 4
    .quad _class_serpname_14_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_14_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_14_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_14_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_14
_class_vtable_14:
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
.globl _class_static_vtable_14
_class_static_vtable_14:
    .quad 0
.globl _class_callable_method_name_14__u__u_construct
_class_callable_method_name_14__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_14__u__u_tostring
_class_callable_method_name_14__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_14_getcode
_class_callable_method_name_14_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_14_getfile
_class_callable_method_name_14_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_14_getline
_class_callable_method_name_14_getline:
    .ascii "getline"
.globl _class_callable_method_name_14_getmessage
_class_callable_method_name_14_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_14_getprevious
_class_callable_method_name_14_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_14_gettrace
_class_callable_method_name_14_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_14_gettraceasstring
_class_callable_method_name_14_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_14
_class_callable_methods_14:
    .quad 9
    .quad _class_callable_method_name_14__u__u_construct
    .quad 11
    .quad _class_callable_method_name_14__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_14_getcode
    .quad 7
    .quad _class_callable_method_name_14_getfile
    .quad 7
    .quad _class_callable_method_name_14_getline
    .quad 7
    .quad _class_callable_method_name_14_getmessage
    .quad 10
    .quad _class_callable_method_name_14_getprevious
    .quad 11
    .quad _class_callable_method_name_14_gettrace
    .quad 8
    .quad _class_callable_method_name_14_gettraceasstring
    .quad 16
.globl _class_interfaces_15
_class_interfaces_15:
    .quad 2
    .quad 6
    .quad _class_interface_impl_15_6
    .quad 13
    .quad _class_interface_impl_15_13
.globl _class_interface_impl_15_6
_class_interface_impl_15_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_15_13
_class_interface_impl_15_13:
    .quad 0
.globl _class_json_pname_15_0
_class_json_pname_15_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_15
_class_json_desc_15:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_15_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_15
_class_gc_desc_15:
    .byte 1, 0, 7, 4
.globl _class_serpname_15_0
_class_serpname_15_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_15_1
_class_serpname_15_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_15_2
_class_serpname_15_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_15_3
_class_serpname_15_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_15
_class_serprop_15:
    .quad 4
    .quad _class_serpname_15_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_15_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_15_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_15_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_15
_class_vtable_15:
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
.globl _class_static_vtable_15
_class_static_vtable_15:
    .quad 0
.globl _class_callable_method_name_15__u__u_construct
_class_callable_method_name_15__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_15__u__u_tostring
_class_callable_method_name_15__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_15_getcode
_class_callable_method_name_15_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_15_getfile
_class_callable_method_name_15_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_15_getline
_class_callable_method_name_15_getline:
    .ascii "getline"
.globl _class_callable_method_name_15_getmessage
_class_callable_method_name_15_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_15_getprevious
_class_callable_method_name_15_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_15_gettrace
_class_callable_method_name_15_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_15_gettraceasstring
_class_callable_method_name_15_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_15
_class_callable_methods_15:
    .quad 9
    .quad _class_callable_method_name_15__u__u_construct
    .quad 11
    .quad _class_callable_method_name_15__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_15_getcode
    .quad 7
    .quad _class_callable_method_name_15_getfile
    .quad 7
    .quad _class_callable_method_name_15_getline
    .quad 7
    .quad _class_callable_method_name_15_getmessage
    .quad 10
    .quad _class_callable_method_name_15_getprevious
    .quad 11
    .quad _class_callable_method_name_15_gettrace
    .quad 8
    .quad _class_callable_method_name_15_gettraceasstring
    .quad 16
.globl _class_interfaces_16
_class_interfaces_16:
    .quad 2
    .quad 6
    .quad _class_interface_impl_16_6
    .quad 13
    .quad _class_interface_impl_16_13
.globl _class_interface_impl_16_6
_class_interface_impl_16_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_16_13
_class_interface_impl_16_13:
    .quad 0
.globl _class_json_pname_16_0
_class_json_pname_16_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_16
_class_json_desc_16:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_16_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_16
_class_gc_desc_16:
    .byte 1, 0, 7, 4
.globl _class_serpname_16_0
_class_serpname_16_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_16_1
_class_serpname_16_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_16_2
_class_serpname_16_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_16_3
_class_serpname_16_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_16
_class_serprop_16:
    .quad 4
    .quad _class_serpname_16_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_16_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_16_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_16_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_16
_class_vtable_16:
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
.globl _class_static_vtable_16
_class_static_vtable_16:
    .quad 0
.globl _class_callable_method_name_16__u__u_construct
_class_callable_method_name_16__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_16__u__u_tostring
_class_callable_method_name_16__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_16_getcode
_class_callable_method_name_16_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_16_getfile
_class_callable_method_name_16_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_16_getline
_class_callable_method_name_16_getline:
    .ascii "getline"
.globl _class_callable_method_name_16_getmessage
_class_callable_method_name_16_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_16_getprevious
_class_callable_method_name_16_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_16_gettrace
_class_callable_method_name_16_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_16_gettraceasstring
_class_callable_method_name_16_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_16
_class_callable_methods_16:
    .quad 9
    .quad _class_callable_method_name_16__u__u_construct
    .quad 11
    .quad _class_callable_method_name_16__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_16_getcode
    .quad 7
    .quad _class_callable_method_name_16_getfile
    .quad 7
    .quad _class_callable_method_name_16_getline
    .quad 7
    .quad _class_callable_method_name_16_getmessage
    .quad 10
    .quad _class_callable_method_name_16_getprevious
    .quad 11
    .quad _class_callable_method_name_16_gettrace
    .quad 8
    .quad _class_callable_method_name_16_gettraceasstring
    .quad 16
.globl _class_interfaces_17
_class_interfaces_17:
    .quad 2
    .quad 6
    .quad _class_interface_impl_17_6
    .quad 13
    .quad _class_interface_impl_17_13
.globl _class_interface_impl_17_6
_class_interface_impl_17_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_17_13
_class_interface_impl_17_13:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_17
_class_vtable_17:
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
.globl _class_interfaces_19
_class_interfaces_19:
    .quad 2
    .quad 6
    .quad _class_interface_impl_19_6
    .quad 13
    .quad _class_interface_impl_19_13
.globl _class_interface_impl_19_6
_class_interface_impl_19_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_19_13
_class_interface_impl_19_13:
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
.globl _class_interfaces_43
_class_interfaces_43:
    .quad 2
    .quad 6
    .quad _class_interface_impl_43_6
    .quad 13
    .quad _class_interface_impl_43_13
.globl _class_interface_impl_43_6
_class_interface_impl_43_6:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_43_0
_class_serpname_43_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_43_1
_class_serpname_43_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_43_2
_class_serpname_43_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_43_3
_class_serpname_43_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_43
_class_serprop_43:
    .quad 4
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
    .quad _class_serpname_43_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_43
_class_vtable_43:
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
.globl _class_interfaces_55
_class_interfaces_55:
    .quad 2
    .quad 6
    .quad _class_interface_impl_55_6
    .quad 13
    .quad _class_interface_impl_55_13
.globl _class_interface_impl_55_6
_class_interface_impl_55_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_55_13
_class_interface_impl_55_13:
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
.globl _class_interfaces_56
_class_interfaces_56:
    .quad 2
    .quad 6
    .quad _class_interface_impl_56_6
    .quad 13
    .quad _class_interface_impl_56_13
.globl _class_interface_impl_56_6
_class_interface_impl_56_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_56_13
_class_interface_impl_56_13:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_56
_class_vtable_56:
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
.globl _class_interfaces_60
_class_interfaces_60:
    .quad 2
    .quad 6
    .quad _class_interface_impl_60_6
    .quad 13
    .quad _class_interface_impl_60_13
.globl _class_interface_impl_60_6
_class_interface_impl_60_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_60_13
_class_interface_impl_60_13:
    .quad 0
.globl _class_json_pname_60_0
_class_json_pname_60_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_60
_class_json_desc_60:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_60_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_60
_class_gc_desc_60:
    .byte 1, 0, 7, 4
.globl _class_serpname_60_0
_class_serpname_60_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_60_1
_class_serpname_60_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_60_2
_class_serpname_60_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_60_3
_class_serpname_60_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_60
_class_serprop_60:
    .quad 4
    .quad _class_serpname_60_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_60_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_60_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_60_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_60
_class_vtable_60:
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
.globl _class_static_vtable_60
_class_static_vtable_60:
    .quad 0
.globl _class_callable_method_name_60__u__u_construct
_class_callable_method_name_60__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_60__u__u_tostring
_class_callable_method_name_60__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_60_getcode
_class_callable_method_name_60_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_60_getfile
_class_callable_method_name_60_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_60_getline
_class_callable_method_name_60_getline:
    .ascii "getline"
.globl _class_callable_method_name_60_getmessage
_class_callable_method_name_60_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_60_getprevious
_class_callable_method_name_60_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_60_gettrace
_class_callable_method_name_60_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_60_gettraceasstring
_class_callable_method_name_60_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_60
_class_callable_methods_60:
    .quad 9
    .quad _class_callable_method_name_60__u__u_construct
    .quad 11
    .quad _class_callable_method_name_60__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_60_getcode
    .quad 7
    .quad _class_callable_method_name_60_getfile
    .quad 7
    .quad _class_callable_method_name_60_getline
    .quad 7
    .quad _class_callable_method_name_60_getmessage
    .quad 10
    .quad _class_callable_method_name_60_getprevious
    .quad 11
    .quad _class_callable_method_name_60_gettrace
    .quad 8
    .quad _class_callable_method_name_60_gettraceasstring
    .quad 16
.globl _class_interfaces_69
_class_interfaces_69:
    .quad 2
    .quad 6
    .quad _class_interface_impl_69_6
    .quad 13
    .quad _class_interface_impl_69_13
.globl _class_interface_impl_69_6
_class_interface_impl_69_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_69_13
_class_interface_impl_69_13:
    .quad 0
.globl _class_json_pname_69_0
_class_json_pname_69_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_69
_class_json_desc_69:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_69_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_69
_class_gc_desc_69:
    .byte 1, 0, 7, 4
.globl _class_serpname_69_0
_class_serpname_69_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_69_1
_class_serpname_69_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_69_2
_class_serpname_69_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_69_3
_class_serpname_69_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_69
_class_serprop_69:
    .quad 4
    .quad _class_serpname_69_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_69_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_69_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_69_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_69
_class_vtable_69:
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
.globl _class_static_vtable_69
_class_static_vtable_69:
    .quad 0
.globl _class_callable_method_name_69__u__u_construct
_class_callable_method_name_69__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_69__u__u_tostring
_class_callable_method_name_69__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_69_getcode
_class_callable_method_name_69_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_69_getfile
_class_callable_method_name_69_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_69_getline
_class_callable_method_name_69_getline:
    .ascii "getline"
.globl _class_callable_method_name_69_getmessage
_class_callable_method_name_69_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_69_getprevious
_class_callable_method_name_69_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_69_gettrace
_class_callable_method_name_69_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_69_gettraceasstring
_class_callable_method_name_69_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_69
_class_callable_methods_69:
    .quad 9
    .quad _class_callable_method_name_69__u__u_construct
    .quad 11
    .quad _class_callable_method_name_69__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_69_getcode
    .quad 7
    .quad _class_callable_method_name_69_getfile
    .quad 7
    .quad _class_callable_method_name_69_getline
    .quad 7
    .quad _class_callable_method_name_69_getmessage
    .quad 10
    .quad _class_callable_method_name_69_getprevious
    .quad 11
    .quad _class_callable_method_name_69_gettrace
    .quad 8
    .quad _class_callable_method_name_69_gettraceasstring
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
    .quad 0
.p2align 3
.globl _class_callable_methods_85
_class_callable_methods_85:
    .quad 0
.globl _class_interfaces_90
_class_interfaces_90:
    .quad 2
    .quad 6
    .quad _class_interface_impl_90_6
    .quad 13
    .quad _class_interface_impl_90_13
.globl _class_interface_impl_90_6
_class_interface_impl_90_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_90_13
_class_interface_impl_90_13:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_90
_class_vtable_90:
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
.globl _class_interfaces_102
_class_interfaces_102:
    .quad 2
    .quad 6
    .quad _class_interface_impl_102_6
    .quad 13
    .quad _class_interface_impl_102_13
.globl _class_interface_impl_102_6
_class_interface_impl_102_6:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_102_13
_class_interface_impl_102_13:
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
    .quad 85

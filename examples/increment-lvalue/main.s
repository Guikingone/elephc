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
    ; @fn name=_class_propinit_13 symbol=_class_propinit_13 synthetic=1
.align 2

.globl _class_propinit_13
_class_propinit_13:
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
_eir__class_propinit_13_entry_0:
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
_class_propinit_13_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_13
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
    stur x10, [x29, #-56]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__class_propinit_15_entry_0:
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
_class_propinit_15_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_15
    ; @fn name=_class_propinit_21 symbol=_class_propinit_21 synthetic=1
.align 2

.globl _class_propinit_21
_class_propinit_21:
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
_eir__class_propinit_21_entry_0:
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
_class_propinit_21_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_22_entry_0:
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
_class_propinit_22_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_22
    ; @fn name=_class_propinit_23 symbol=_class_propinit_23 synthetic=1
.align 2

.globl _class_propinit_23
_class_propinit_23:
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
_eir__class_propinit_23_entry_0:
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
_class_propinit_23_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_23
    ; @fn name=_class_propinit_24 symbol=_class_propinit_24 synthetic=1
.align 2

.globl _class_propinit_24
_class_propinit_24:
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
_eir__class_propinit_24_entry_0:
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
_class_propinit_24_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_24
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
    ; @fn name=_class_propinit_27 symbol=_class_propinit_27 synthetic=1
.align 2

.globl _class_propinit_27
_class_propinit_27:
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
_eir__class_propinit_27_entry_0:
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
_class_propinit_27_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_27
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
    ; @fn name=_class_propinit_54 symbol=_class_propinit_54 synthetic=1
.align 2

.globl _class_propinit_54
_class_propinit_54:
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
_eir__class_propinit_54_entry_0:
    ; @src line=38 col=23 end=38:24 op=nop
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    ; @src line=38 col=23 end=38:24 op=const_i64
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_54_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_54
    ; @fn name=_class_propinit_55 symbol=_class_propinit_55 synthetic=1
.align 2

.globl _class_propinit_55
_class_propinit_55:
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
_eir__class_propinit_55_entry_0:
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
_class_propinit_55_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    ; @fn name=_class_propinit_62 symbol=_class_propinit_62 synthetic=1
.align 2

.globl _class_propinit_62
_class_propinit_62:
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
_eir__class_propinit_62_entry_0:
    ; @src line=9 col=25 end=9:26 op=nop
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    ; @src line=9 col=25 end=9:26 op=const_i64
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_62_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_62
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
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
_eir__class_propinit_65_entry_0:
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
_class_propinit_65_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
_eir__class_propinit_67_entry_0:
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
_class_propinit_67_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
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
_eir__class_propinit_69_entry_0:
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
_class_propinit_69_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_69
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
    ; @fn name=_class_propinit_75 symbol=_class_propinit_75 synthetic=1
.align 2

.globl _class_propinit_75
_class_propinit_75:
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
_eir__class_propinit_75_entry_0:
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
_class_propinit_75_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_75
    ; @fn name=_class_propinit_76 symbol=_class_propinit_76 synthetic=1
.align 2

.globl _class_propinit_76
_class_propinit_76:
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
_eir__class_propinit_76_entry_0:
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
_class_propinit_76_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_76
    ; @fn name=_class_propinit_77 symbol=_class_propinit_77 synthetic=1
.align 2

.globl _class_propinit_77
_class_propinit_77:
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
_eir__class_propinit_77_entry_0:
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
_class_propinit_77_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_77
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
_eir__class_propinit_79_entry_0:
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
_class_propinit_79_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
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
    ; @fn name=_class_propinit_93 symbol=_class_propinit_93 synthetic=1
.align 2

.globl _class_propinit_93
_class_propinit_93:
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
_eir__class_propinit_93_entry_0:
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
_class_propinit_93_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_93
    ; @fn name=_class_propinit_95 symbol=_class_propinit_95 synthetic=1
.align 2

.globl _class_propinit_95
_class_propinit_95:
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
_eir__class_propinit_95_entry_0:
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
_class_propinit_95_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_95
    ; @fn name=_class_propinit_96 symbol=_class_propinit_96 synthetic=1
.align 2

.globl _class_propinit_96
_class_propinit_96:
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
_eir__class_propinit_96_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_96_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_96
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
    ; @fn name=Cursor::next symbol=_method_Cursor_next
.align 2

.globl _method_Cursor_next
_method_Cursor_next:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-120]
    stur x22, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-104]
    stur xzr, [x29, #-112]
    ; @block name=entry
_eir_Cursor__next_entry_0:
    ; @src line=43 col=9 end=43:15 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=43 col=26 end=43:28 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=43 col=16 end=43:21 op=load_local
    ldur x0, [x29, #-104]
    stur x0, [x29, #-8]
    ; @src line=43 col=21 end=43:23 op=prop_get
    ldur x9, [x29, #-8]
    cbz x9, _eir_Cursor__next_prop_get_null_receiver_0
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_Cursor__next_prop_get_null_receiver_0
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_Cursor__next_typed_prop_initialized_2
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_Cursor__next_typed_property_throw_3
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #84
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_Cursor__next_typed_property_throw_3:
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
    mov x9, #70
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_Cursor__next_typed_prop_initialized_2:
    ldr x0, [x9, #8]
    mov x21, x0
    b _eir_Cursor__next_prop_get_done_1
_eir_Cursor__next_prop_get_null_receiver_0:
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #48
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
_eir_Cursor__next_prop_get_done_1:
    ; @src line=43 col=21 end=43:23 op=nop
    ; @src line=43 col=26 end=43:28 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=43 col=26 end=43:28 op=ichecked_add
    mov x0, x21
    mov x10, x22
    mov x1, x10
    bl __rt_int_add_checked
    stur x0, [x29, #-32]
    ; @src line=43 col=26 end=43:28 op=acquire
    ldur x0, [x29, #-32]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-40]
    ; @src line=43 col=26 end=43:28 op=store_local
    ldur x0, [x29, #-40]
    stur x0, [x29, #-112]
    ; @src line=43 col=26 end=43:28 op=release
    ldur x0, [x29, #-32]
    bl __rt_decref_mixed
    ; @src line=43 col=26 end=43:28 op=load_local
    ldur x0, [x29, #-112]
    stur x0, [x29, #-48]
    ; @src line=43 col=26 end=43:28 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=43 col=16 end=43:21 op=load_local
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    ; @src line=43 col=26 end=43:28 op=load_local
    ldur x0, [x29, #-112]
    stur x0, [x29, #-64]
    ; @src line=43 col=26 end=43:28 op=prop_set
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-64]
    bl __rt_mixed_cast_int
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; @src line=43 col=26 end=43:28 op=load_local
    ldur x0, [x29, #-112]
    stur x0, [x29, #-72]
    ; @src line=43 col=26 end=43:28 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=43 col=26 end=43:28 op=mixed_numeric_binop
    ldur x0, [x29, #-72]
    str x0, [sp, #-16]!
    mov x0, x22
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_numeric_sub
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    stur x0, [x29, #-88]
    ; @src line=43 col=9 end=43:15 op=cast
    ldur x0, [x29, #-88]
    bl __rt_mixed_cast_int
    mov x22, x0
    ; @src line=43 col=9 end=43:15 op=release
    ldur x0, [x29, #-88]
    bl __rt_decref_mixed
    mov x0, x22
    str x0, [sp, #-16]!
    ; epilogue cleanup $__elephc_assign_expr_43_26_0
    ldur x0, [x29, #-112]
    cbz x0, _eir_Cursor__next_main_refcounted_cleanup_done_4
    bl __rt_decref_mixed
_eir_Cursor__next_main_refcounted_cleanup_done_4:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-120]
    ldur x22, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_method_Cursor_next_epilogue:
    str x0, [sp, #-16]!
    ; epilogue cleanup $__elephc_assign_expr_43_26_0
    ldur x0, [x29, #-112]
    cbz x0, _eir_Cursor__next_main_refcounted_cleanup_done_5
    bl __rt_decref_mixed
_eir_Cursor__next_main_refcounted_cleanup_done_5:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-120]
    ldur x22, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=Cursor::next
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #1920
    mov x9, sp
    add x9, x9, #1904
    stp x29, x30, [x9]
    add x29, sp, #1904
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #1896
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #1880
    str x21, [x9]
    sub x9, x29, #1888
    str x22, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #1800
    str xzr, [x9]
    sub x9, x29, #1808
    str xzr, [x9]
    sub x9, x29, #1816
    str xzr, [x9]
    sub x9, x29, #1832
    str xzr, [x9]
    sub x9, x29, #1824
    str xzr, [x9]
    sub x9, x29, #1840
    str xzr, [x9]
    sub x9, x29, #1848
    str xzr, [x9]
    sub x9, x29, #1856
    str xzr, [x9]
    sub x9, x29, #1864
    str xzr, [x9]
    sub x9, x29, #1872
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=7 col=1 end=7:6 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=7 col=1 end=7:6 op=nop
    ; @src line=12 col=1 end=12:7 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=10 end=12:21 op=object_new
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    mov x10, #62
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    stur x0, [x29, #-8]
    ldur x10, [x29, #-8]
    mov x0, #0
    str x0, [x10, #8]
    str xzr, [x10, #16]
    ; @src line=12 col=1 end=12:7 op=acquire
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-16]
    ; @src line=12 col=1 end=12:7 op=store_local
    ldur x0, [x29, #-16]
    sub x9, x29, #1800
    str x0, [x9]
    ; @src line=12 col=1 end=12:7 op=release
    ldur x0, [x29, #-8]
    bl __rt_decref_object
    ; @src line=13 col=1 end=13:10 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=13 end=13:14 op=hash_new
    mov x0, #16
    mov x1, #0
    bl __rt_hash_new
    stur x0, [x29, #-24]
    ; @src line=13 col=14 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=13 col=23 end=13:24 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=13 col=13 end=13:14 op=hash_set
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    ldur x0, [x29, #-24]
    mov x5, #0
    bl __rt_hash_set
    stur x0, [x29, #-24]
    ; @src line=13 col=26 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=13 col=34 end=13:35 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=13 col=13 end=13:14 op=hash_set
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    ldur x0, [x29, #-24]
    mov x5, #0
    bl __rt_hash_set
    stur x0, [x29, #-24]
    ; @src line=13 col=1 end=13:10 op=acquire
    ldur x0, [x29, #-24]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-80]
    ; @src line=13 col=1 end=13:10 op=store_local
    ldur x0, [x29, #-80]
    sub x9, x29, #1808
    str x0, [x9]
    ; @src line=13 col=1 end=13:10 op=release
    ldur x0, [x29, #-24]
    bl __rt_decref_hash
    ; @src line=15 col=1 end=15:7 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=15 col=10 end=15:11 op=array_new
    mov x0, #5
    mov x1, #16
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #1
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-88]
    ; @src line=15 col=11 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=15 col=11 op=array_push
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-88]
    ; @src line=15 col=18 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=15 col=18 op=array_push
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-88]
    ; @src line=15 col=24 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=15 col=24 op=array_push
    ldur x1, [x29, #-136]
    ldur x2, [x29, #-128]
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-88]
    ; @src line=15 col=31 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=15 col=31 op=array_push
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-88]
    ; @src line=15 col=38 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-168]
    stur x2, [x29, #-160]
    ; @src line=15 col=38 op=array_push
    ldur x1, [x29, #-168]
    ldur x2, [x29, #-160]
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_str
    stur x0, [x29, #-88]
    ; @src line=15 col=1 end=15:7 op=acquire
    ldur x0, [x29, #-88]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-176]
    ; @src line=15 col=1 end=15:7 op=store_local
    ldur x0, [x29, #-176]
    sub x9, x29, #1816
    str x0, [x9]
    ; @src line=15 col=1 end=15:7 op=release
    ldur x0, [x29, #-88]
    bl __rt_decref_any
    ; @src line=17 col=1 end=17:8 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=10 end=17:16 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    stur x0, [x29, #-184]
    ; @src line=17 col=10 end=17:16 op=hash_to_mixed
    ldur x0, [x29, #-184]
    cbz x0, _eir_main_hash_to_mixed_done_0
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_to_mixed_done_0
    bl __rt_hash_to_mixed
_eir_main_hash_to_mixed_done_0:
    stur x0, [x29, #-192]
    ; @src line=17 col=10 end=17:16 op=store_local
    ldur x0, [x29, #-192]
    sub x9, x29, #1808
    str x0, [x9]
    ; @src line=17 col=10 end=17:16 op=load_local
    sub x9, x29, #1816
    ldr x0, [x9]
    stur x0, [x29, #-200]
    ; @src line=17 col=10 end=17:16 op=iter_start
    ldur x0, [x29, #-200]
    sub x9, x29, #272
    str x0, [x9]
    mov x0, #-1
    sub x9, x29, #264
    str x0, [x9]
    sub x9, x29, #272
    ldr x0, [x9]
    cbz x0, _eir_main_iter_len_null_source_1
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir_main_iter_len_null_source_1
    ldr x10, [x0]
    b _eir_main_iter_len_done_2
_eir_main_iter_len_null_source_1:
    mov x10, #0
_eir_main_iter_len_done_2:
    stur x10, [x29, #-208]
    b _eir_main_foreach_next_1
    ; @block name=foreach.next
_eir_main_foreach_next_1:
    ; @src line=17 col=10 end=17:16 op=iter_next
    sub x9, x29, #264
    ldr x10, [x9]
    add x10, x10, #1
    ldur x11, [x29, #-208]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_3
    sub x9, x29, #264
    str x10, [x9]
_eir_main_iter_next_done_3:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_2
    b _eir_main_foreach_exit_3
    ; @block name=foreach.body
_eir_main_foreach_body_2:
    ; @src line=17 col=10 end=17:16 op=iter_current_value
    sub x9, x29, #272
    ldr x12, [x9]
    sub x9, x29, #264
    ldr x10, [x9]
    lsl x10, x10, #4
    add x12, x12, x10
    add x12, x12, #24
    ldr x1, [x12]
    ldr x2, [x12, #8]
    sub x9, x29, #296
    str x1, [x9]
    sub x9, x29, #288
    str x2, [x9]
    ; @src line=17 col=10 end=17:16 op=acquire
    sub x9, x29, #296
    ldr x1, [x9]
    sub x9, x29, #288
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #312
    str x1, [x9]
    sub x9, x29, #304
    str x2, [x9]
    ; @src line=17 col=10 end=17:16 op=load_local
    sub x9, x29, #1832
    ldr x1, [x9]
    sub x9, x29, #1824
    ldr x2, [x9]
    sub x9, x29, #328
    str x1, [x9]
    sub x9, x29, #320
    str x2, [x9]
    ; @src line=17 col=10 end=17:16 op=release
    sub x9, x29, #328
    ldr x1, [x9]
    sub x9, x29, #320
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=17 col=10 end=17:16 op=store_local
    sub x9, x29, #312
    ldr x1, [x9]
    sub x9, x29, #304
    ldr x2, [x9]
    sub x9, x29, #1832
    str x1, [x9]
    sub x9, x29, #1824
    str x2, [x9]
    ; @src line=18 col=5 end=18:7 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=5 end=18:7 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #336
    str x0, [x9]
    ; @src line=18 col=17 end=18:22 op=load_local
    sub x9, x29, #1832
    ldr x1, [x9]
    sub x9, x29, #1824
    ldr x2, [x9]
    sub x9, x29, #352
    str x1, [x9]
    sub x9, x29, #344
    str x2, [x9]
    ; @src line=18 col=7 end=18:16 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #360
    str x0, [x9]
    ; @src line=18 col=17 end=18:22 op=load_local
    sub x9, x29, #1832
    ldr x1, [x9]
    sub x9, x29, #1824
    ldr x2, [x9]
    sub x9, x29, #376
    str x1, [x9]
    sub x9, x29, #368
    str x2, [x9]
    ; @src line=18 col=16 end=18:17 op=hash_get
    sub x9, x29, #376
    ldr x1, [x9]
    sub x9, x29, #368
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #360
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_5
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_5
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_4
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_8
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_9
_eir_main_hash_get_mixed_box_8:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_9:
    b _eir_main_hash_get_done_7
_eir_main_hash_get_miss_4:
    sub x9, x29, #376
    ldr x1, [x9]
    sub x9, x29, #368
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_10
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_11
_eir_main_hash_warn_integer_key_10:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_11:
    b _eir_main_hash_get_fallback_6
_eir_main_hash_get_null_recv_5:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_6:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_7:
    sub x9, x29, #384
    str x0, [x9]
    ; @src line=18 col=16 end=18:17 op=nop
    ; @src line=18 col=5 end=18:7 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=18 col=5 end=18:7 op=mixed_numeric_binop
    sub x9, x29, #384
    ldr x0, [x9]
    str x0, [sp, #-16]!
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_numeric_add
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    sub x9, x29, #400
    str x0, [x9]
    ; @src line=18 col=5 end=18:7 op=release
    sub x9, x29, #384
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=18 col=5 end=18:7 op=hash_set
    sub x9, x29, #352
    ldr x1, [x9]
    sub x9, x29, #344
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #400
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #336
    ldr x0, [x9]
    mov x5, #7
    bl __rt_hash_set
    sub x9, x29, #336
    str x0, [x9]
    sub x9, x29, #336
    ldr x0, [x9]
    sub x9, x29, #1808
    str x0, [x9]
    ; @src line=19 col=5 end=19:7 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=7 end=19:13 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #408
    str x0, [x9]
    ; @src line=19 col=7 end=19:13 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #416
    str x0, [x9]
    ; @src line=19 col=13 end=19:15 op=prop_get
    sub x9, x29, #416
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_12
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_12
    sub x9, x29, #416
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_14
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_15
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #85
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_15:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_9@PAGE
    add x9, x9, _str_9@PAGEOFF
    str x9, [x0, #8]
    mov x9, #71
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_14:
    ldr x0, [x9, #8]
    mov x21, x0
    b _eir_main_prop_get_done_13
_eir_main_prop_get_null_receiver_12:
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
_eir_main_prop_get_done_13:
    ; @src line=19 col=13 end=19:15 op=nop
    ; @src line=19 col=5 end=19:7 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=19 col=5 end=19:7 op=ichecked_add
    mov x0, x21
    mov x10, x22
    mov x1, x10
    bl __rt_int_add_checked
    sub x9, x29, #440
    str x0, [x9]
    ; @src line=19 col=5 end=19:7 op=prop_set
    sub x9, x29, #408
    ldr x9, [x9]
    str x9, [sp, #-16]!
    sub x9, x29, #440
    ldr x0, [x9]
    bl __rt_mixed_cast_int
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; @src line=19 col=5 end=19:7 op=release
    sub x9, x29, #440
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_1
    ; @block name=foreach.exit
_eir_main_foreach_exit_3:
    ; @src line=17 col=10 end=17:16 op=nop
    ; @src line=22 col=1 end=22:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=6 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #5
    sub x9, x29, #456
    str x1, [x9]
    sub x9, x29, #448
    str x2, [x9]
    ; @src line=22 col=16 end=22:25 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #464
    str x0, [x9]
    ; @src line=22 col=26 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    sub x9, x29, #480
    str x1, [x9]
    sub x9, x29, #472
    str x2, [x9]
    ; @src line=22 col=25 end=22:26 op=hash_get
    sub x9, x29, #480
    ldr x1, [x9]
    sub x9, x29, #472
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #464
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_17
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_17
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_16
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_20
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_21
_eir_main_hash_get_mixed_box_20:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_21:
    b _eir_main_hash_get_done_19
_eir_main_hash_get_miss_16:
    sub x9, x29, #480
    ldr x1, [x9]
    sub x9, x29, #472
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_22
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_23
_eir_main_hash_warn_integer_key_22:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_23:
    b _eir_main_hash_get_fallback_18
_eir_main_hash_get_null_recv_17:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_18:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_19:
    sub x9, x29, #488
    str x0, [x9]
    ; @src line=22 col=25 end=22:26 op=nop
    ; @src line=22 col=14 end=22:26 op=cast
    sub x9, x29, #488
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_24
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_object_24:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_27
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_28
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_29
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_30
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_31
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_32
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_33
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_34
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_35
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_36
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_37
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_38
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_39
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_40
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_41
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_42
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_43
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_44
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_45
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_46
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_47
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_48
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_49
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_50
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_51
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_52
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_53
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_54
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_55
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_56
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_57
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_58
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_59
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_60
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_61
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_62
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_63
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_64
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_65
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_66
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_67
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_68
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_69
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_70
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_71
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_72
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_73
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_74
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_75
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_76
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_77
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_78
    mov x10, #93
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_79
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_80
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_81
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_82
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_83
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_84
    b _eir_main_mixed_string_no_match_25
_eir_main_mixed_string_Exception_27:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_RuntimeException_28:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_RangeException_29:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateException_30:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateInvalidTimeZoneException_31:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_SplFileInfo_32:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_PharFileInfo_33:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_PharData_34:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionFunctionAbstract_35:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionClassConstant_36:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_LogicException_37:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_BadFunctionCallException_38:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_BadMethodCallException_39:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_CachingIterator_40:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_Error_41:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
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
    b _eir_main_mixed_string_done_26
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DomainException_44:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateError_45:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateRangeError_46:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_UnexpectedValueException_47:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_UnderflowException_48:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionMethod_49:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionNamedType_50:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionEnum_51:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionProperty_52:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateObjectError_53:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DirectoryIterator_54:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_FilesystemIterator_55:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_GlobIterator_56:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionUnionType_57:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateUnknownException_58:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_OverflowException_59:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_SplFileObject_60:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_SplTempFileObject_61:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateMalformedStringException_62:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateInvalidOperationException_64:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_Phar_65:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionFunction_66:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_JsonException_67:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ValueError_68:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_LengthException_69:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_TypeError_71:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionClass_72:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionObject_73:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ArithmeticError_74:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionException_75:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionEnumUnitCase_76:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_OutOfBoundsException_77:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionEnumBackedCase_78:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_FiberError_79:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_OutOfRangeException_80:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_InvalidArgumentException_81:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_ReflectionParameter_82:
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
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_DateMalformedPeriodStringException_83:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_RecursiveCachingIterator_84:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_26
_eir_main_mixed_string_no_match_25:
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_26:
    sub x9, x29, #504
    str x1, [x9]
    sub x9, x29, #496
    str x2, [x9]
    ; @src line=22 col=14 end=22:26 op=release
    sub x9, x29, #488
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=22 col=14 end=22:26 op=str_concat
    sub x9, x29, #456
    ldr x1, [x9]
    sub x9, x29, #448
    ldr x2, [x9]
    sub x9, x29, #504
    ldr x3, [x9]
    sub x9, x29, #496
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #520
    str x1, [x9]
    sub x9, x29, #512
    str x2, [x9]
    ; @src line=22 col=14 end=22:26 op=release
    sub x9, x29, #504
    ldr x1, [x9]
    sub x9, x29, #496
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=22 col=35 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #536
    str x1, [x9]
    sub x9, x29, #528
    str x2, [x9]
    ; @src line=22 col=33 end=22:35 op=str_concat
    sub x9, x29, #520
    ldr x1, [x9]
    sub x9, x29, #512
    ldr x2, [x9]
    sub x9, x29, #536
    ldr x3, [x9]
    sub x9, x29, #528
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #552
    str x1, [x9]
    sub x9, x29, #544
    str x2, [x9]
    ; @src line=22 col=33 end=22:35 op=release
    ; @src line=22 col=1 end=22:5 op=echo_value
    sub x9, x29, #552
    ldr x1, [x9]
    sub x9, x29, #544
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=22 col=1 end=22:5 op=release
    ; @src line=23 col=1 end=23:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=23 col=6 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #4
    sub x9, x29, #568
    str x1, [x9]
    sub x9, x29, #560
    str x2, [x9]
    ; @src line=23 col=15 end=23:24 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #576
    str x0, [x9]
    ; @src line=23 col=25 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #2
    sub x9, x29, #592
    str x1, [x9]
    sub x9, x29, #584
    str x2, [x9]
    ; @src line=23 col=24 end=23:25 op=hash_get
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #576
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_86
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_86
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_85
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_89
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_90
_eir_main_hash_get_mixed_box_89:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_90:
    b _eir_main_hash_get_done_88
_eir_main_hash_get_miss_85:
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_91
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_92
_eir_main_hash_warn_integer_key_91:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_92:
    b _eir_main_hash_get_fallback_87
_eir_main_hash_get_null_recv_86:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_87:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_88:
    sub x9, x29, #600
    str x0, [x9]
    ; @src line=23 col=24 end=23:25 op=nop
    ; @src line=23 col=13 end=23:25 op=cast
    sub x9, x29, #600
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_93
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_object_93:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_96
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_97
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_98
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_99
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_100
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_101
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_102
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_103
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_104
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_105
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_106
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_107
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_108
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_109
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_110
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_111
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_112
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_113
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_114
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_115
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_116
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_117
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_118
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_119
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_120
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_121
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_122
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_123
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_124
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_125
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_126
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_127
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_128
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_129
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_130
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_131
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_132
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_133
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_134
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_135
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_136
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_137
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_138
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_139
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_140
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_141
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_142
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_143
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_144
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_145
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_146
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_147
    mov x10, #93
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_148
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_149
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_150
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_151
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_152
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_153
    b _eir_main_mixed_string_no_match_94
_eir_main_mixed_string_Exception_96:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_RuntimeException_97:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_RangeException_98:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateException_99:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateInvalidTimeZoneException_100:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_SplFileInfo_101:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_PharFileInfo_102:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_PharData_103:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionFunctionAbstract_104:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionClassConstant_105:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_LogicException_106:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_BadFunctionCallException_107:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_BadMethodCallException_108:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_CachingIterator_109:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_Error_110:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_UnhandledMatchError_111:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateMalformedIntervalStringException_112:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DomainException_113:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateError_114:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateRangeError_115:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_UnexpectedValueException_116:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionMethod_118:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionNamedType_119:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionEnum_120:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionProperty_121:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateObjectError_122:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DirectoryIterator_123:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_FilesystemIterator_124:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_GlobIterator_125:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionUnionType_126:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateUnknownException_127:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_OverflowException_128:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_SplFileObject_129:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_SplTempFileObject_130:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateMalformedStringException_131:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionIntersectionType_132:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateInvalidOperationException_133:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_Phar_134:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionFunction_135:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_JsonException_136:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ValueError_137:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_LengthException_138:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_RecursiveDirectoryIterator_139:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_TypeError_140:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionClass_141:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionObject_142:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ArithmeticError_143:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionException_144:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionEnumUnitCase_145:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_OutOfBoundsException_146:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionEnumBackedCase_147:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_FiberError_148:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_OutOfRangeException_149:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_InvalidArgumentException_150:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_ReflectionParameter_151:
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
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_DateMalformedPeriodStringException_152:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_RecursiveCachingIterator_153:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_95
_eir_main_mixed_string_no_match_94:
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_95:
    sub x9, x29, #616
    str x1, [x9]
    sub x9, x29, #608
    str x2, [x9]
    ; @src line=23 col=13 end=23:25 op=release
    sub x9, x29, #600
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=23 col=13 end=23:25 op=str_concat
    sub x9, x29, #568
    ldr x1, [x9]
    sub x9, x29, #560
    ldr x2, [x9]
    sub x9, x29, #616
    ldr x3, [x9]
    sub x9, x29, #608
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #632
    str x1, [x9]
    sub x9, x29, #624
    str x2, [x9]
    ; @src line=23 col=13 end=23:25 op=release
    sub x9, x29, #616
    ldr x1, [x9]
    sub x9, x29, #608
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=23 col=33 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #648
    str x1, [x9]
    sub x9, x29, #640
    str x2, [x9]
    ; @src line=23 col=31 end=23:33 op=str_concat
    sub x9, x29, #632
    ldr x1, [x9]
    sub x9, x29, #624
    ldr x2, [x9]
    sub x9, x29, #648
    ldr x3, [x9]
    sub x9, x29, #640
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #664
    str x1, [x9]
    sub x9, x29, #656
    str x2, [x9]
    ; @src line=23 col=31 end=23:33 op=release
    ; @src line=23 col=1 end=23:5 op=echo_value
    sub x9, x29, #664
    ldr x1, [x9]
    sub x9, x29, #656
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=23 col=1 end=23:5 op=release
    ; @src line=24 col=1 end=24:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=24 col=6 op=const_str
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #7
    sub x9, x29, #680
    str x1, [x9]
    sub x9, x29, #672
    str x2, [x9]
    ; @src line=24 col=18 end=24:24 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #688
    str x0, [x9]
    ; @src line=24 col=24 end=24:26 op=prop_get
    sub x9, x29, #688
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_154
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_154
    sub x9, x29, #688
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_156
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_157
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #85
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_157:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_9@PAGE
    add x9, x9, _str_9@PAGEOFF
    str x9, [x0, #8]
    mov x9, #71
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_156:
    ldr x0, [x9, #8]
    mov x22, x0
    b _eir_main_prop_get_done_155
_eir_main_prop_get_null_receiver_154:
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x22, x0
_eir_main_prop_get_done_155:
    ; @src line=24 col=24 end=24:26 op=nop
    ; @src line=24 col=16 end=24:26 op=i_to_str
    mov x0, x22
    bl __rt_itoa
    sub x9, x29, #712
    str x1, [x9]
    sub x9, x29, #704
    str x2, [x9]
    ; @src line=24 col=16 end=24:26 op=str_concat
    sub x9, x29, #680
    ldr x1, [x9]
    sub x9, x29, #672
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
    ; @src line=24 col=16 end=24:26 op=release
    ; @src line=24 col=34 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #744
    str x1, [x9]
    sub x9, x29, #736
    str x2, [x9]
    ; @src line=24 col=32 end=24:34 op=str_concat
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
    ; @src line=24 col=32 end=24:34 op=release
    ; @src line=24 col=1 end=24:5 op=echo_value
    sub x9, x29, #760
    ldr x1, [x9]
    sub x9, x29, #752
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=24 col=1 end=24:5 op=release
    ; @src line=27 col=1 end=27:3 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=1 end=27:3 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #768
    str x0, [x9]
    ; @src line=27 col=13 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    sub x9, x29, #784
    str x1, [x9]
    sub x9, x29, #776
    str x2, [x9]
    ; @src line=27 col=3 end=27:12 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #792
    str x0, [x9]
    ; @src line=27 col=13 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    sub x9, x29, #808
    str x1, [x9]
    sub x9, x29, #800
    str x2, [x9]
    ; @src line=27 col=12 end=27:13 op=hash_get
    sub x9, x29, #808
    ldr x1, [x9]
    sub x9, x29, #800
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #792
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_159
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_159
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_158
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_162
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_163
_eir_main_hash_get_mixed_box_162:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_163:
    b _eir_main_hash_get_done_161
_eir_main_hash_get_miss_158:
    sub x9, x29, #808
    ldr x1, [x9]
    sub x9, x29, #800
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_164
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_165
_eir_main_hash_warn_integer_key_164:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_165:
    b _eir_main_hash_get_fallback_160
_eir_main_hash_get_null_recv_159:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_160:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_161:
    sub x9, x29, #816
    str x0, [x9]
    ; @src line=27 col=12 end=27:13 op=nop
    ; @src line=27 col=1 end=27:3 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=27 col=1 end=27:3 op=mixed_numeric_binop
    sub x9, x29, #816
    ldr x0, [x9]
    str x0, [sp, #-16]!
    mov x0, x22
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_numeric_sub
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    sub x9, x29, #832
    str x0, [x9]
    ; @src line=27 col=1 end=27:3 op=release
    sub x9, x29, #816
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=27 col=1 end=27:3 op=hash_set
    sub x9, x29, #784
    ldr x1, [x9]
    sub x9, x29, #776
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #832
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #768
    ldr x0, [x9]
    mov x5, #7
    bl __rt_hash_set
    sub x9, x29, #768
    str x0, [x9]
    sub x9, x29, #768
    ldr x0, [x9]
    sub x9, x29, #1808
    str x0, [x9]
    ; @src line=28 col=1 end=28:3 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=28 col=3 end=28:9 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #840
    str x0, [x9]
    ; @src line=28 col=3 end=28:9 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #848
    str x0, [x9]
    ; @src line=28 col=9 end=28:11 op=prop_get
    sub x9, x29, #848
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_166
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_166
    sub x9, x29, #848
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_168
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_169
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #85
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_169:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_9@PAGE
    add x9, x9, _str_9@PAGEOFF
    str x9, [x0, #8]
    mov x9, #71
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_168:
    ldr x0, [x9, #8]
    mov x22, x0
    b _eir_main_prop_get_done_167
_eir_main_prop_get_null_receiver_166:
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x22, x0
_eir_main_prop_get_done_167:
    ; @src line=28 col=9 end=28:11 op=nop
    ; @src line=28 col=1 end=28:3 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=28 col=1 end=28:3 op=ichecked_sub
    mov x0, x22
    mov x10, x21
    mov x1, x10
    bl __rt_int_sub_checked
    sub x9, x29, #872
    str x0, [x9]
    ; @src line=28 col=1 end=28:3 op=prop_set
    sub x9, x29, #840
    ldr x9, [x9]
    str x9, [sp, #-16]!
    sub x9, x29, #872
    ldr x0, [x9]
    bl __rt_mixed_cast_int
    ldr x9, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ; @src line=28 col=1 end=28:3 op=release
    sub x9, x29, #872
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1 end=30:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=6 op=const_str
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #22
    sub x9, x29, #888
    str x1, [x9]
    sub x9, x29, #880
    str x2, [x9]
    ; @src line=30 col=33 end=30:42 op=load_local
    sub x9, x29, #1808
    ldr x0, [x9]
    sub x9, x29, #896
    str x0, [x9]
    ; @src line=30 col=43 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #3
    sub x9, x29, #912
    str x1, [x9]
    sub x9, x29, #904
    str x2, [x9]
    ; @src line=30 col=42 end=30:43 op=hash_get
    sub x9, x29, #912
    ldr x1, [x9]
    sub x9, x29, #904
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #896
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_171
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_171
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_170
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_174
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_175
_eir_main_hash_get_mixed_box_174:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_175:
    b _eir_main_hash_get_done_173
_eir_main_hash_get_miss_170:
    sub x9, x29, #912
    ldr x1, [x9]
    sub x9, x29, #904
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_176
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_177
_eir_main_hash_warn_integer_key_176:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_177:
    b _eir_main_hash_get_fallback_172
_eir_main_hash_get_null_recv_171:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_172:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_173:
    sub x9, x29, #920
    str x0, [x9]
    ; @src line=30 col=42 end=30:43 op=nop
    ; @src line=30 col=31 end=30:43 op=cast
    sub x9, x29, #920
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_178
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_object_178:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_181
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_182
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_183
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_184
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_185
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_186
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_187
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_188
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_189
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_190
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_191
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_192
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_193
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_194
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_195
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_196
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_197
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_198
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_199
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_200
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_201
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_202
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_203
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_204
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_205
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_206
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_207
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_208
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_209
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_210
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_211
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_212
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_213
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_214
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_215
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_216
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_217
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_218
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_219
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_220
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_221
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_222
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_223
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_224
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_225
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_226
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_227
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_228
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_229
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_230
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_231
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_232
    mov x10, #93
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_233
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_234
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_235
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_236
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_237
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_238
    b _eir_main_mixed_string_no_match_179
_eir_main_mixed_string_Exception_181:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_RuntimeException_182:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_RangeException_183:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateException_184:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateInvalidTimeZoneException_185:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_SplFileInfo_186:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_PharFileInfo_187:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_PharData_188:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionFunctionAbstract_189:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionClassConstant_190:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_LogicException_191:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_BadFunctionCallException_192:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_BadMethodCallException_193:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_CachingIterator_194:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_Error_195:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_UnhandledMatchError_196:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateMalformedIntervalStringException_197:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DomainException_198:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateError_199:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateRangeError_200:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_UnexpectedValueException_201:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_UnderflowException_202:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionMethod_203:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionNamedType_204:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionEnum_205:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionProperty_206:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateObjectError_207:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DirectoryIterator_208:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_FilesystemIterator_209:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_GlobIterator_210:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionUnionType_211:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateUnknownException_212:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_OverflowException_213:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_SplFileObject_214:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_SplTempFileObject_215:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateMalformedStringException_216:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionIntersectionType_217:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateInvalidOperationException_218:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_Phar_219:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionFunction_220:
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
    b _eir_main_mixed_string_done_180
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ValueError_222:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_LengthException_223:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_RecursiveDirectoryIterator_224:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_TypeError_225:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionClass_226:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionObject_227:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ArithmeticError_228:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionException_229:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionEnumUnitCase_230:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_OutOfBoundsException_231:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionEnumBackedCase_232:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_FiberError_233:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_InvalidArgumentException_235:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_ReflectionParameter_236:
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
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_DateMalformedPeriodStringException_237:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_RecursiveCachingIterator_238:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_180
_eir_main_mixed_string_no_match_179:
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_180:
    sub x9, x29, #936
    str x1, [x9]
    sub x9, x29, #928
    str x2, [x9]
    ; @src line=30 col=31 end=30:43 op=release
    sub x9, x29, #920
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=31 end=30:43 op=str_concat
    sub x9, x29, #888
    ldr x1, [x9]
    sub x9, x29, #880
    ldr x2, [x9]
    sub x9, x29, #936
    ldr x3, [x9]
    sub x9, x29, #928
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #952
    str x1, [x9]
    sub x9, x29, #944
    str x2, [x9]
    ; @src line=30 col=31 end=30:43 op=release
    sub x9, x29, #936
    ldr x1, [x9]
    sub x9, x29, #928
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=30 col=52 op=const_str
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #9
    sub x9, x29, #968
    str x1, [x9]
    sub x9, x29, #960
    str x2, [x9]
    ; @src line=30 col=50 end=30:52 op=str_concat
    sub x9, x29, #952
    ldr x1, [x9]
    sub x9, x29, #944
    ldr x2, [x9]
    sub x9, x29, #968
    ldr x3, [x9]
    sub x9, x29, #960
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #984
    str x1, [x9]
    sub x9, x29, #976
    str x2, [x9]
    ; @src line=30 col=50 end=30:52 op=release
    ; @src line=30 col=66 end=30:72 op=load_local
    sub x9, x29, #1800
    ldr x0, [x9]
    sub x9, x29, #992
    str x0, [x9]
    ; @src line=30 col=72 end=30:74 op=prop_get
    sub x9, x29, #992
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_239
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_239
    sub x9, x29, #992
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_241
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_242
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #85
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_242:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_9@PAGE
    add x9, x9, _str_9@PAGEOFF
    str x9, [x0, #8]
    mov x9, #71
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_241:
    ldr x0, [x9, #8]
    mov x21, x0
    b _eir_main_prop_get_done_240
_eir_main_prop_get_null_receiver_239:
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
_eir_main_prop_get_done_240:
    ; @src line=30 col=72 end=30:74 op=nop
    ; @src line=30 col=64 end=30:74 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    sub x9, x29, #1016
    str x1, [x9]
    sub x9, x29, #1008
    str x2, [x9]
    ; @src line=30 col=64 end=30:74 op=str_concat
    sub x9, x29, #984
    ldr x1, [x9]
    sub x9, x29, #976
    ldr x2, [x9]
    sub x9, x29, #1016
    ldr x3, [x9]
    sub x9, x29, #1008
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1032
    str x1, [x9]
    sub x9, x29, #1024
    str x2, [x9]
    ; @src line=30 col=64 end=30:74 op=release
    ; @src line=30 col=64 end=30:74 op=release
    ; @src line=30 col=82 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #1048
    str x1, [x9]
    sub x9, x29, #1040
    str x2, [x9]
    ; @src line=30 col=80 end=30:82 op=str_concat
    sub x9, x29, #1032
    ldr x1, [x9]
    sub x9, x29, #1024
    ldr x2, [x9]
    sub x9, x29, #1048
    ldr x3, [x9]
    sub x9, x29, #1040
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1064
    str x1, [x9]
    sub x9, x29, #1056
    str x2, [x9]
    ; @src line=30 col=80 end=30:82 op=release
    ; @src line=30 col=1 end=30:5 op=echo_value
    sub x9, x29, #1064
    ldr x1, [x9]
    sub x9, x29, #1056
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=30 col=1 end=30:5 op=release
    ; @src line=36 col=1 end=36:6 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=36 col=1 end=36:6 op=nop
    ; @src line=47 col=1 end=47:8 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=47 col=11 end=47:23 op=object_new
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    mov x10, #54
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    sub x9, x29, #1072
    str x0, [x9]
    sub x9, x29, #1072
    ldr x10, [x9]
    mov x0, #0
    str x0, [x10, #8]
    str xzr, [x10, #16]
    ; @src line=47 col=1 end=47:8 op=acquire
    sub x9, x29, #1072
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1080
    str x0, [x9]
    ; @src line=47 col=1 end=47:8 op=store_local
    sub x9, x29, #1080
    ldr x0, [x9]
    sub x9, x29, #1840
    str x0, [x9]
    ; @src line=47 col=1 end=47:8 op=release
    sub x9, x29, #1072
    ldr x0, [x9]
    bl __rt_decref_object
    ; @src line=48 col=1 end=48:8 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=48 col=11 end=48:12 op=array_new
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
    sub x9, x29, #1088
    str x0, [x9]
    ; @src line=48 col=12 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #1
    sub x9, x29, #1104
    str x1, [x9]
    sub x9, x29, #1096
    str x2, [x9]
    ; @src line=48 col=12 op=array_push
    sub x9, x29, #1104
    ldr x1, [x9]
    sub x9, x29, #1096
    ldr x2, [x9]
    sub x9, x29, #1088
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #1088
    str x0, [x9]
    ; @src line=48 col=17 op=const_str
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #1
    sub x9, x29, #1120
    str x1, [x9]
    sub x9, x29, #1112
    str x2, [x9]
    ; @src line=48 col=17 op=array_push
    sub x9, x29, #1120
    ldr x1, [x9]
    sub x9, x29, #1112
    ldr x2, [x9]
    sub x9, x29, #1088
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #1088
    str x0, [x9]
    ; @src line=48 col=22 op=const_str
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #1
    sub x9, x29, #1136
    str x1, [x9]
    sub x9, x29, #1128
    str x2, [x9]
    ; @src line=48 col=22 op=array_push
    sub x9, x29, #1136
    ldr x1, [x9]
    sub x9, x29, #1128
    ldr x2, [x9]
    sub x9, x29, #1088
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_str
    sub x9, x29, #1088
    str x0, [x9]
    ; @src line=48 col=1 end=48:8 op=acquire
    sub x9, x29, #1088
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1144
    str x0, [x9]
    ; @src line=48 col=1 end=48:8 op=store_local
    sub x9, x29, #1144
    ldr x0, [x9]
    sub x9, x29, #1848
    str x0, [x9]
    ; @src line=48 col=1 end=48:8 op=release
    sub x9, x29, #1088
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=49 col=1 end=49:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=6 op=const_str
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #6
    sub x9, x29, #1160
    str x1, [x9]
    sub x9, x29, #1152
    str x2, [x9]
    ; @src line=49 col=17 end=49:24 op=load_local
    sub x9, x29, #1848
    ldr x0, [x9]
    sub x9, x29, #1168
    str x0, [x9]
    ; @src line=49 col=25 end=49:32 op=load_local
    sub x9, x29, #1840
    ldr x0, [x9]
    sub x9, x29, #1176
    str x0, [x9]
    ; @src line=49 col=32 end=49:40 op=method_call
    sub x9, x29, #1176
    ldr x9, [x9]
    cbz x9, _eir_main_static_method_receiver_null_243
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_static_method_receiver_null_243
    b _eir_main_static_method_receiver_checked_244
_eir_main_static_method_receiver_null_243:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_245
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #70
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_static_exception_throw_245:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_23@PAGE
    add x9, x9, _str_23@PAGEOFF
    str x9, [x0, #8]
    mov x9, #40
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_static_method_receiver_checked_244:
    sub x9, x29, #1176
    ldr x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x21, x0
    ; @src line=49 col=32 end=49:40 op=nop
    ; @src line=49 col=24 end=49:25 op=array_get
    mov x0, x21
    sub x9, x29, #1168
    ldr x9, [x9]
    cbz x9, _eir_main_array_get_null_recv_247
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_array_get_null_recv_247
    cmp x0, #0
    b.lt _eir_main_array_get_null_246
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_246
    lsl x0, x0, #4
    add x9, x9, x0
    add x9, x9, #24
    ldr x1, [x9]
    ldr x2, [x9, #8]
    b _eir_main_array_get_done_249
_eir_main_array_get_null_246:
    bl __rt_warn_undefined_array_key_int
    b _eir_main_array_get_fallback_248
_eir_main_array_get_null_recv_247:
    bl __rt_warn_array_offset_on_null
_eir_main_array_get_fallback_248:
    movz x1, #0xfffe
    movk x1, #0xffff, lsl #16
    movk x1, #0xffff, lsl #32
    movk x1, #0x7fff, lsl #48
    mov x2, #0
_eir_main_array_get_done_249:
    sub x9, x29, #1200
    str x1, [x9]
    sub x9, x29, #1192
    str x2, [x9]
    ; @src line=49 col=24 end=49:25 op=acquire
    sub x9, x29, #1200
    ldr x1, [x9]
    sub x9, x29, #1192
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1216
    str x1, [x9]
    sub x9, x29, #1208
    str x2, [x9]
    ; @src line=49 col=24 end=49:25 op=nop
    ; @src line=49 col=15 end=49:25 op=str_concat
    sub x9, x29, #1160
    ldr x1, [x9]
    sub x9, x29, #1152
    ldr x2, [x9]
    sub x9, x29, #1216
    ldr x3, [x9]
    sub x9, x29, #1208
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1232
    str x1, [x9]
    sub x9, x29, #1224
    str x2, [x9]
    ; @src line=49 col=15 end=49:25 op=release
    sub x9, x29, #1216
    ldr x1, [x9]
    sub x9, x29, #1208
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=49 col=42 end=49:52 op=str_persist
    sub x9, x29, #1232
    ldr x1, [x9]
    sub x9, x29, #1224
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1248
    str x1, [x9]
    sub x9, x29, #1240
    str x2, [x9]
    ; @src line=49 col=44 end=49:51 op=load_local
    sub x9, x29, #1848
    ldr x0, [x9]
    sub x9, x29, #1256
    str x0, [x9]
    ; @src line=49 col=52 end=49:59 op=load_local
    sub x9, x29, #1840
    ldr x0, [x9]
    sub x9, x29, #1264
    str x0, [x9]
    ; @src line=49 col=59 end=49:67 op=method_call
    sub x9, x29, #1264
    ldr x9, [x9]
    cbz x9, _eir_main_static_method_receiver_null_250
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_static_method_receiver_null_250
    b _eir_main_static_method_receiver_checked_251
_eir_main_static_method_receiver_null_250:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_static_exception_throw_252
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #70
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_static_exception_throw_252:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_23@PAGE
    add x9, x9, _str_23@PAGEOFF
    str x9, [x0, #8]
    mov x9, #40
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_static_method_receiver_checked_251:
    sub x9, x29, #1264
    ldr x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9]
    blr x9
    mov x21, x0
    ; @src line=49 col=59 end=49:67 op=nop
    ; @src line=49 col=51 end=49:52 op=array_get
    mov x0, x21
    sub x9, x29, #1256
    ldr x9, [x9]
    cbz x9, _eir_main_array_get_null_recv_254
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_array_get_null_recv_254
    cmp x0, #0
    b.lt _eir_main_array_get_null_253
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_253
    lsl x0, x0, #4
    add x9, x9, x0
    add x9, x9, #24
    ldr x1, [x9]
    ldr x2, [x9, #8]
    b _eir_main_array_get_done_256
_eir_main_array_get_null_253:
    bl __rt_warn_undefined_array_key_int
    b _eir_main_array_get_fallback_255
_eir_main_array_get_null_recv_254:
    bl __rt_warn_array_offset_on_null
_eir_main_array_get_fallback_255:
    movz x1, #0xfffe
    movk x1, #0xffff, lsl #16
    movk x1, #0xffff, lsl #32
    movk x1, #0x7fff, lsl #48
    mov x2, #0
_eir_main_array_get_done_256:
    sub x9, x29, #1288
    str x1, [x9]
    sub x9, x29, #1280
    str x2, [x9]
    ; @src line=49 col=51 end=49:52 op=acquire
    sub x9, x29, #1288
    ldr x1, [x9]
    sub x9, x29, #1280
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #1304
    str x1, [x9]
    sub x9, x29, #1296
    str x2, [x9]
    ; @src line=49 col=51 end=49:52 op=nop
    ; @src line=49 col=42 end=49:52 op=str_concat
    sub x9, x29, #1248
    ldr x1, [x9]
    sub x9, x29, #1240
    ldr x2, [x9]
    sub x9, x29, #1304
    ldr x3, [x9]
    sub x9, x29, #1296
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1320
    str x1, [x9]
    sub x9, x29, #1312
    str x2, [x9]
    ; @src line=49 col=42 end=49:52 op=release
    sub x9, x29, #1248
    ldr x1, [x9]
    sub x9, x29, #1240
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=49 col=42 end=49:52 op=release
    sub x9, x29, #1304
    ldr x1, [x9]
    sub x9, x29, #1296
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=49 col=71 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #1336
    str x1, [x9]
    sub x9, x29, #1328
    str x2, [x9]
    ; @src line=49 col=69 end=49:71 op=str_concat
    sub x9, x29, #1320
    ldr x1, [x9]
    sub x9, x29, #1312
    ldr x2, [x9]
    sub x9, x29, #1336
    ldr x3, [x9]
    sub x9, x29, #1328
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1352
    str x1, [x9]
    sub x9, x29, #1344
    str x2, [x9]
    ; @src line=49 col=69 end=49:71 op=release
    ; @src line=49 col=1 end=49:5 op=echo_value
    sub x9, x29, #1352
    ldr x1, [x9]
    sub x9, x29, #1344
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=49 col=1 end=49:5 op=release
    ; @src line=50 col=1 end=50:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=50 col=6 op=const_str
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #21
    sub x9, x29, #1368
    str x1, [x9]
    sub x9, x29, #1360
    str x2, [x9]
    ; @src line=50 col=32 end=50:39 op=load_local
    sub x9, x29, #1840
    ldr x0, [x9]
    sub x9, x29, #1376
    str x0, [x9]
    ; @src line=50 col=39 end=50:41 op=prop_get
    sub x9, x29, #1376
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_257
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_257
    sub x9, x29, #1376
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_259
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_260
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #84
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_260:
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
    mov x9, #70
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_259:
    ldr x0, [x9, #8]
    mov x21, x0
    b _eir_main_prop_get_done_258
_eir_main_prop_get_null_receiver_257:
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #48
    bl __rt_diag_warning
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
_eir_main_prop_get_done_258:
    ; @src line=50 col=39 end=50:41 op=nop
    ; @src line=50 col=30 end=50:41 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    sub x9, x29, #1400
    str x1, [x9]
    sub x9, x29, #1392
    str x2, [x9]
    ; @src line=50 col=30 end=50:41 op=str_concat
    sub x9, x29, #1368
    ldr x1, [x9]
    sub x9, x29, #1360
    ldr x2, [x9]
    sub x9, x29, #1400
    ldr x3, [x9]
    sub x9, x29, #1392
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1416
    str x1, [x9]
    sub x9, x29, #1408
    str x2, [x9]
    ; @src line=50 col=30 end=50:41 op=release
    ; @src line=50 col=47 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #1
    sub x9, x29, #1432
    str x1, [x9]
    sub x9, x29, #1424
    str x2, [x9]
    ; @src line=50 col=45 end=50:47 op=str_concat
    sub x9, x29, #1416
    ldr x1, [x9]
    sub x9, x29, #1408
    ldr x2, [x9]
    sub x9, x29, #1432
    ldr x3, [x9]
    sub x9, x29, #1424
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1448
    str x1, [x9]
    sub x9, x29, #1440
    str x2, [x9]
    ; @src line=50 col=45 end=50:47 op=release
    ; @src line=50 col=1 end=50:5 op=echo_value
    sub x9, x29, #1448
    ldr x1, [x9]
    sub x9, x29, #1440
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=50 col=1 end=50:5 op=release
    ; @src line=52 col=1 end=52:9 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=12 end=52:13 op=hash_new
    mov x0, #16
    mov x1, #0
    bl __rt_hash_new
    sub x9, x29, #1456
    str x0, [x9]
    ; @src line=52 col=13 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #4
    sub x9, x29, #1472
    str x1, [x9]
    sub x9, x29, #1464
    str x2, [x9]
    ; @src line=52 col=23 end=52:25 op=const_i64
    mov x0, #41
    mov x21, x0
    ; @src line=52 col=12 end=52:13 op=hash_set
    sub x9, x29, #1472
    ldr x1, [x9]
    sub x9, x29, #1464
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1456
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1456
    str x0, [x9]
    ; @src line=52 col=1 end=52:9 op=hash_to_mixed
    sub x9, x29, #1456
    ldr x0, [x9]
    cbz x0, _eir_main_hash_to_mixed_done_261
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_to_mixed_done_261
    bl __rt_hash_to_mixed
_eir_main_hash_to_mixed_done_261:
    sub x9, x29, #1488
    str x0, [x9]
    ; @src line=52 col=1 end=52:9 op=acquire
    sub x9, x29, #1488
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1496
    str x0, [x9]
    ; @src line=52 col=1 end=52:9 op=store_local
    sub x9, x29, #1496
    ldr x0, [x9]
    sub x9, x29, #1856
    str x0, [x9]
    ; @src line=52 col=1 end=52:9 op=release
    sub x9, x29, #1488
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=53 col=1 end=53:6 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=53 col=9 end=53:11 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=53 col=11 end=53:19 op=load_local
    sub x9, x29, #1856
    ldr x0, [x9]
    sub x9, x29, #1504
    str x0, [x9]
    ; @src line=53 col=20 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #4
    sub x9, x29, #1520
    str x1, [x9]
    sub x9, x29, #1512
    str x2, [x9]
    ; @src line=53 col=19 end=53:20 op=hash_get
    sub x9, x29, #1520
    ldr x1, [x9]
    sub x9, x29, #1512
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #1504
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_263
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_263
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_262
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_266
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_267
_eir_main_hash_get_mixed_box_266:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_267:
    b _eir_main_hash_get_done_265
_eir_main_hash_get_miss_262:
    sub x9, x29, #1520
    ldr x1, [x9]
    sub x9, x29, #1512
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_268
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_269
_eir_main_hash_warn_integer_key_268:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_269:
    b _eir_main_hash_get_fallback_264
_eir_main_hash_get_null_recv_263:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_264:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_265:
    sub x9, x29, #1528
    str x0, [x9]
    ; @src line=53 col=19 end=53:20 op=nop
    ; @src line=53 col=9 end=53:11 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=53 col=9 end=53:11 op=mixed_numeric_binop
    sub x9, x29, #1528
    ldr x0, [x9]
    str x0, [sp, #-16]!
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_numeric_add
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    sub x9, x29, #1544
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=release
    sub x9, x29, #1528
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=53 col=9 end=53:11 op=acquire
    sub x9, x29, #1544
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1552
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=store_local
    sub x9, x29, #1552
    ldr x0, [x9]
    sub x9, x29, #1864
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=release
    sub x9, x29, #1544
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=53 col=9 end=53:11 op=load_local
    sub x9, x29, #1864
    ldr x0, [x9]
    sub x9, x29, #1560
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=53 col=9 end=53:11 op=load_local
    sub x9, x29, #1856
    ldr x0, [x9]
    sub x9, x29, #1568
    str x0, [x9]
    ; @src line=53 col=20 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #4
    sub x9, x29, #1584
    str x1, [x9]
    sub x9, x29, #1576
    str x2, [x9]
    ; @src line=53 col=9 end=53:11 op=load_local
    sub x9, x29, #1864
    ldr x0, [x9]
    sub x9, x29, #1592
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=hash_set
    sub x9, x29, #1584
    ldr x1, [x9]
    sub x9, x29, #1576
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1592
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1568
    ldr x0, [x9]
    mov x5, #7
    bl __rt_hash_set
    sub x9, x29, #1568
    str x0, [x9]
    sub x9, x29, #1568
    ldr x0, [x9]
    sub x9, x29, #1856
    str x0, [x9]
    ; @src line=53 col=9 end=53:11 op=load_local
    sub x9, x29, #1864
    ldr x0, [x9]
    sub x9, x29, #1600
    str x0, [x9]
    ; @src line=53 col=1 end=53:6 op=acquire
    sub x9, x29, #1600
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1608
    str x0, [x9]
    ; @src line=53 col=1 end=53:6 op=store_local
    sub x9, x29, #1608
    ldr x0, [x9]
    sub x9, x29, #1872
    str x0, [x9]
    ; @src line=54 col=1 end=54:5 op=concat_reset
    sub x9, x29, #1896
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=54 col=6 op=const_str
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #13
    sub x9, x29, #1624
    str x1, [x9]
    sub x9, x29, #1616
    str x2, [x9]
    ; @src line=54 col=24 end=54:29 op=load_local
    sub x9, x29, #1872
    ldr x0, [x9]
    sub x9, x29, #1632
    str x0, [x9]
    ; @src line=54 col=22 end=54:29 op=cast
    sub x9, x29, #1632
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_270
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_object_270:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_273
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_274
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_275
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_276
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_277
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_278
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_279
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_280
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_281
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_282
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_283
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_284
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_285
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_286
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_287
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_288
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_289
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_290
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_291
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_292
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_293
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_294
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_295
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_296
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_297
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_298
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_299
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_300
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_301
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_302
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_303
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_304
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_305
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_306
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_307
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_308
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_309
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_310
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_311
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_312
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_313
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_314
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_315
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_316
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_317
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_318
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_319
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_320
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_321
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_322
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_323
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_324
    mov x10, #93
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_325
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_326
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_327
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_328
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_329
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_330
    b _eir_main_mixed_string_no_match_271
_eir_main_mixed_string_Exception_273:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_RuntimeException_274:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_RangeException_275:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateException_276:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateInvalidTimeZoneException_277:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_SplFileInfo_278:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_PharFileInfo_279:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_PharData_280:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionFunctionAbstract_281:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionClassConstant_282:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_LogicException_283:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_BadFunctionCallException_284:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_BadMethodCallException_285:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_CachingIterator_286:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_Error_287:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_UnhandledMatchError_288:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateMalformedIntervalStringException_289:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DomainException_290:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateError_291:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateRangeError_292:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_UnexpectedValueException_293:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_UnderflowException_294:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionMethod_295:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionNamedType_296:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionEnum_297:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionProperty_298:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateObjectError_299:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DirectoryIterator_300:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_FilesystemIterator_301:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_GlobIterator_302:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionUnionType_303:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateUnknownException_304:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_OverflowException_305:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_SplFileObject_306:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_SplTempFileObject_307:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateMalformedStringException_308:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionIntersectionType_309:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateInvalidOperationException_310:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_Phar_311:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionFunction_312:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_JsonException_313:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ValueError_314:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_LengthException_315:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_RecursiveDirectoryIterator_316:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_TypeError_317:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionClass_318:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionObject_319:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ArithmeticError_320:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionException_321:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionEnumUnitCase_322:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_OutOfBoundsException_323:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionEnumBackedCase_324:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_FiberError_325:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_OutOfRangeException_326:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_InvalidArgumentException_327:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_ReflectionParameter_328:
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
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_DateMalformedPeriodStringException_329:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_RecursiveCachingIterator_330:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_272
_eir_main_mixed_string_no_match_271:
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_272:
    sub x9, x29, #1648
    str x1, [x9]
    sub x9, x29, #1640
    str x2, [x9]
    ; @src line=54 col=22 end=54:29 op=str_concat
    sub x9, x29, #1624
    ldr x1, [x9]
    sub x9, x29, #1616
    ldr x2, [x9]
    sub x9, x29, #1648
    ldr x3, [x9]
    sub x9, x29, #1640
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1664
    str x1, [x9]
    sub x9, x29, #1656
    str x2, [x9]
    ; @src line=54 col=22 end=54:29 op=release
    sub x9, x29, #1648
    ldr x1, [x9]
    sub x9, x29, #1640
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=54 col=32 op=const_str
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #9
    sub x9, x29, #1680
    str x1, [x9]
    sub x9, x29, #1672
    str x2, [x9]
    ; @src line=54 col=30 end=54:32 op=str_concat
    sub x9, x29, #1664
    ldr x1, [x9]
    sub x9, x29, #1656
    ldr x2, [x9]
    sub x9, x29, #1680
    ldr x3, [x9]
    sub x9, x29, #1672
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1696
    str x1, [x9]
    sub x9, x29, #1688
    str x2, [x9]
    ; @src line=54 col=30 end=54:32 op=release
    ; @src line=54 col=46 end=54:54 op=load_local
    sub x9, x29, #1856
    ldr x0, [x9]
    sub x9, x29, #1704
    str x0, [x9]
    ; @src line=54 col=55 op=const_str
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #4
    sub x9, x29, #1720
    str x1, [x9]
    sub x9, x29, #1712
    str x2, [x9]
    ; @src line=54 col=54 end=54:55 op=hash_get
    sub x9, x29, #1720
    ldr x1, [x9]
    sub x9, x29, #1712
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #1704
    ldr x0, [x9]
    cbz x0, _eir_main_hash_get_null_recv_332
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_main_hash_get_null_recv_332
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_331
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_335
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_336
_eir_main_hash_get_mixed_box_335:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_336:
    b _eir_main_hash_get_done_334
_eir_main_hash_get_miss_331:
    sub x9, x29, #1720
    ldr x1, [x9]
    sub x9, x29, #1712
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    cmn x2, #1
    b.eq _eir_main_hash_warn_integer_key_337
    bl __rt_warn_undefined_array_key_str
    b _eir_main_hash_warn_key_done_338
_eir_main_hash_warn_integer_key_337:
    mov x0, x1
    bl __rt_warn_undefined_array_key_int
_eir_main_hash_warn_key_done_338:
    b _eir_main_hash_get_fallback_333
_eir_main_hash_get_null_recv_332:
    bl __rt_warn_array_offset_on_null
_eir_main_hash_get_fallback_333:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_334:
    sub x9, x29, #1728
    str x0, [x9]
    ; @src line=54 col=54 end=54:55 op=nop
    ; @src line=54 col=44 end=54:55 op=cast
    sub x9, x29, #1728
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_339
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_object_339:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_342
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_343
    mov x10, #4
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_344
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_345
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_346
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_347
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_348
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_349
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_350
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_351
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_352
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_353
    mov x10, #25
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_354
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_355
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_356
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_357
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_358
    mov x10, #32
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_359
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_360
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_361
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_362
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_363
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_364
    mov x10, #39
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_365
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_366
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_367
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_368
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_369
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_370
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_371
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_372
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_373
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_374
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_375
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_376
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_377
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_378
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_379
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_380
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_381
    mov x10, #78
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_382
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_383
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_384
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_385
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_386
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_387
    mov x10, #87
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_388
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_389
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_390
    mov x10, #90
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_391
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_392
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_393
    mov x10, #93
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_394
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_395
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_396
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_397
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_398
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_399
    b _eir_main_mixed_string_no_match_340
_eir_main_mixed_string_Exception_342:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_RuntimeException_343:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_RangeException_344:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateException_345:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateInvalidTimeZoneException_346:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_SplFileInfo_347:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_PharFileInfo_348:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_PharData_349:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionFunctionAbstract_350:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionClassConstant_351:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_LogicException_352:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_BadFunctionCallException_353:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_BadMethodCallException_354:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_Error_356:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_UnhandledMatchError_357:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateMalformedIntervalStringException_358:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DomainException_359:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateError_360:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateRangeError_361:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_UnexpectedValueException_362:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionMethod_364:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionNamedType_365:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionEnum_366:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionProperty_367:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateObjectError_368:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DirectoryIterator_369:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_FilesystemIterator_370:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_GlobIterator_371:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionUnionType_372:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateUnknownException_373:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_OverflowException_374:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_SplFileObject_375:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_SplTempFileObject_376:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateMalformedStringException_377:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionIntersectionType_378:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateInvalidOperationException_379:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_Phar_380:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionFunction_381:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_JsonException_382:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ValueError_383:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_LengthException_384:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_RecursiveDirectoryIterator_385:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_TypeError_386:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionClass_387:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionObject_388:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ArithmeticError_389:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionException_390:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionEnumUnitCase_391:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_OutOfBoundsException_392:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionEnumBackedCase_393:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_FiberError_394:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_OutOfRangeException_395:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_InvalidArgumentException_396:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_ReflectionParameter_397:
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
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_DateMalformedPeriodStringException_398:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_RecursiveCachingIterator_399:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_341
_eir_main_mixed_string_no_match_340:
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_341:
    sub x9, x29, #1744
    str x1, [x9]
    sub x9, x29, #1736
    str x2, [x9]
    ; @src line=54 col=44 end=54:55 op=release
    sub x9, x29, #1728
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=54 col=44 end=54:55 op=str_concat
    sub x9, x29, #1696
    ldr x1, [x9]
    sub x9, x29, #1688
    ldr x2, [x9]
    sub x9, x29, #1744
    ldr x3, [x9]
    sub x9, x29, #1736
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1760
    str x1, [x9]
    sub x9, x29, #1752
    str x2, [x9]
    ; @src line=54 col=44 end=54:55 op=release
    ; @src line=54 col=44 end=54:55 op=release
    sub x9, x29, #1744
    ldr x1, [x9]
    sub x9, x29, #1736
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=54 col=65 op=const_str
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #1776
    str x1, [x9]
    sub x9, x29, #1768
    str x2, [x9]
    ; @src line=54 col=63 end=54:65 op=str_concat
    sub x9, x29, #1760
    ldr x1, [x9]
    sub x9, x29, #1752
    ldr x2, [x9]
    sub x9, x29, #1776
    ldr x3, [x9]
    sub x9, x29, #1768
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1792
    str x1, [x9]
    sub x9, x29, #1784
    str x2, [x9]
    ; @src line=54 col=63 end=54:65 op=release
    ; @src line=54 col=1 end=54:5 op=echo_value
    sub x9, x29, #1792
    ldr x1, [x9]
    sub x9, x29, #1784
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=54 col=1 end=54:5 op=release

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $tally
    sub x9, x29, #1800
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_400
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_400:
    ; epilogue cleanup $byChoice
    sub x9, x29, #1808
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_401
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_401:
    ; epilogue cleanup $votes
    sub x9, x29, #1816
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_402
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_402:
    ; epilogue cleanup $vote
    sub x9, x29, #1832
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $cursor
    sub x9, x29, #1840
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_403
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_403:
    ; epilogue cleanup $tokens
    sub x9, x29, #1848
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_404
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_404:
    ; epilogue cleanup $counter
    sub x9, x29, #1856
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_405
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_405:
    ; epilogue cleanup $__elephc_assign_expr_53_9_0
    sub x9, x29, #1864
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_406
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_406:
    ; epilogue cleanup $next
    sub x9, x29, #1872
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_407
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_407:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #1880
    ldr x21, [x9]
    sub x9, x29, #1888
    ldr x22, [x9]
    mov x9, sp
    add x9, x9, #1904
    ldp x29, x30, [x9]
    add sp, sp, #1920
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_408
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_408:
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
    .ascii "Fatal error: Typed property Cursor::$pos must not be accessed before initialization\n"
.globl _str_4
_str_4:
    .ascii "Typed property Cursor::$pos must not be accessed before initialization"
.globl _str_5
_str_5:
    .ascii "Warning: Attempt to read property \"pos\" on null\n"
.globl _str_6
_str_6:
    .ascii "yes"
.globl _str_7
_str_7:
    .ascii "no"
.globl _str_8
_str_8:
    .ascii "Fatal error: Typed property Tally::$total must not be accessed before initialization\n"
.globl _str_9
_str_9:
    .ascii "Typed property Tally::$total must not be accessed before initialization"
.globl _str_10
_str_10:
    .ascii "Warning: Attempt to read property \"total\" on null\n"
.globl _str_11
_str_11:
    .ascii "yes: "
.globl _str_12
_str_12:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_13
_str_13:
    .ascii "\n"
.globl _str_14
_str_14:
    .ascii "no: "
.globl _str_15
_str_15:
    .ascii "total: "
.globl _str_16
_str_16:
    .ascii "after retract -> yes: "
.globl _str_17
_str_17:
    .ascii ", total: "
.globl _str_18
_str_18:
    .ascii "a"
.globl _str_19
_str_19:
    .ascii "b"
.globl _str_20
_str_20:
    .ascii "c"
.globl _str_21
_str_21:
    .ascii "walk: "
.globl _str_22
_str_22:
    .ascii "Fatal error: Uncaught Error: Call to a member function next() on null\n"
.globl _str_23
_str_23:
    .ascii "Call to a member function next() on null"
.globl _str_24
_str_24:
    .ascii "pos after two reads: "
.globl _str_25
_str_25:
    .ascii "hits"
.globl _str_26
_str_26:
    .ascii "next hit id: "
.globl _str_27
_str_27:
    .ascii " (stored "
.globl _str_28
_str_28:
    .ascii ")\n"
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
    .quad 36
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_2
    .quad 9
    .quad 2
    .quad 0
    .quad _instanceof_name_class_abs_2
    .quad 10
    .quad 2
    .quad 0
    .quad _instanceof_name_class_3
    .quad 16
    .quad 3
    .quad 0
    .quad _instanceof_name_class_abs_3
    .quad 17
    .quad 3
    .quad 0
    .quad _instanceof_name_class_23
    .quad 14
    .quad 23
    .quad 0
    .quad _instanceof_name_class_abs_23
    .quad 15
    .quad 23
    .quad 0
    .quad _instanceof_name_class_27
    .quad 5
    .quad 27
    .quad 0
    .quad _instanceof_name_class_abs_27
    .quad 6
    .quad 27
    .quad 0
    .quad _instanceof_name_class_28
    .quad 19
    .quad 28
    .quad 0
    .quad _instanceof_name_class_abs_28
    .quad 20
    .quad 28
    .quad 0
    .quad _instanceof_name_class_54
    .quad 6
    .quad 54
    .quad 0
    .quad _instanceof_name_class_abs_54
    .quad 7
    .quad 54
    .quad 0
    .quad _instanceof_name_class_62
    .quad 5
    .quad 62
    .quad 0
    .quad _instanceof_name_class_abs_62
    .quad 6
    .quad 62
    .quad 0
    .quad _instanceof_name_class_78
    .quad 13
    .quad 78
    .quad 0
    .quad _instanceof_name_class_abs_78
    .quad 14
    .quad 78
    .quad 0
    .quad _instanceof_name_class_80
    .quad 10
    .quad 80
    .quad 0
    .quad _instanceof_name_class_abs_80
    .quad 11
    .quad 80
    .quad 0
    .quad _instanceof_name_class_84
    .quad 8
    .quad 84
    .quad 0
    .quad _instanceof_name_class_abs_84
    .quad 9
    .quad 84
    .quad 0
    .quad _instanceof_name_class_85
    .quad 9
    .quad 85
    .quad 0
    .quad _instanceof_name_class_abs_85
    .quad 10
    .quad 85
    .quad 0
    .quad _instanceof_name_class_88
    .quad 15
    .quad 88
    .quad 0
    .quad _instanceof_name_class_abs_88
    .quad 16
    .quad 88
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
    .quad 20
    .quad 91
    .quad 0
    .quad _instanceof_name_class_abs_91
    .quad 21
    .quad 91
    .quad 0
    .quad _instanceof_name_class_95
    .quad 19
    .quad 95
    .quad 0
    .quad _instanceof_name_class_abs_95
    .quad 20
    .quad 95
    .quad 0
    .quad _instanceof_name_class_99
    .quad 24
    .quad 99
    .quad 0
    .quad _instanceof_name_class_abs_99
    .quad 25
    .quad 99
    .quad 0
    .quad _instanceof_name_interface_0
    .quad 9
    .quad 0
    .quad 1
    .quad _instanceof_name_interface_abs_0
    .quad 10
    .quad 0
    .quad 1
    .quad _instanceof_name_interface_5
    .quad 10
    .quad 5
    .quad 1
    .quad _instanceof_name_interface_abs_5
    .quad 11
    .quad 5
    .quad 1
.globl _instanceof_name_class_2
_instanceof_name_class_2:
    .ascii "Exception"
.globl _instanceof_name_class_abs_2
_instanceof_name_class_abs_2:
    .ascii "\\Exception"
.globl _instanceof_name_class_3
_instanceof_name_class_3:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_3
_instanceof_name_class_abs_3:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_23
_instanceof_name_class_23:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_23
_instanceof_name_class_abs_23:
    .ascii "\\LogicException"
.globl _instanceof_name_class_27
_instanceof_name_class_27:
    .ascii "Error"
.globl _instanceof_name_class_abs_27
_instanceof_name_class_abs_27:
    .ascii "\\Error"
.globl _instanceof_name_class_28
_instanceof_name_class_28:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_28
_instanceof_name_class_abs_28:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_54
_instanceof_name_class_54:
    .ascii "Cursor"
.globl _instanceof_name_class_abs_54
_instanceof_name_class_abs_54:
    .ascii "\\Cursor"
.globl _instanceof_name_class_62
_instanceof_name_class_62:
    .ascii "Tally"
.globl _instanceof_name_class_abs_62
_instanceof_name_class_abs_62:
    .ascii "\\Tally"
.globl _instanceof_name_class_78
_instanceof_name_class_78:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_78
_instanceof_name_class_abs_78:
    .ascii "\\JsonException"
.globl _instanceof_name_class_80
_instanceof_name_class_80:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_80
_instanceof_name_class_abs_80:
    .ascii "\\ValueError"
.globl _instanceof_name_class_84
_instanceof_name_class_84:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_84
_instanceof_name_class_abs_84:
    .ascii "\\stdClass"
.globl _instanceof_name_class_85
_instanceof_name_class_85:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_85
_instanceof_name_class_abs_85:
    .ascii "\\TypeError"
.globl _instanceof_name_class_88
_instanceof_name_class_88:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_88
_instanceof_name_class_abs_88:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_89
_instanceof_name_class_89:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_89
_instanceof_name_class_abs_89:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_91
_instanceof_name_class_91:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_91
_instanceof_name_class_abs_91:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_95
_instanceof_name_class_95:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_95
_instanceof_name_class_abs_95:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_99
_instanceof_name_class_99:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_99
_instanceof_name_class_abs_99:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_interface_0
_instanceof_name_interface_0:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_0
_instanceof_name_interface_abs_0:
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
    .quad 100
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_2
    .quad 9
    .quad _class_name_3
    .quad 16
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_23
    .quad 14
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_27
    .quad 5
    .quad _class_name_28
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
    .quad _class_name_54
    .quad 6
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_62
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
    .quad _class_name_78
    .quad 13
    .quad _class_name_missing
    .quad 0
    .quad _class_name_80
    .quad 10
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_84
    .quad 8
    .quad _class_name_85
    .quad 9
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_88
    .quad 15
    .quad _class_name_89
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_91
    .quad 20
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_95
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_99
    .quad 24
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_2
_class_name_2:
    .ascii "Exception"
.globl _class_name_3
_class_name_3:
    .ascii "RuntimeException"
.globl _class_name_23
_class_name_23:
    .ascii "LogicException"
.globl _class_name_27
_class_name_27:
    .ascii "Error"
.globl _class_name_28
_class_name_28:
    .ascii "UnhandledMatchError"
.globl _class_name_54
_class_name_54:
    .ascii "Cursor"
.globl _class_name_62
_class_name_62:
    .ascii "Tally"
.globl _class_name_78
_class_name_78:
    .ascii "JsonException"
.globl _class_name_80
_class_name_80:
    .ascii "ValueError"
.globl _class_name_84
_class_name_84:
    .ascii "stdClass"
.globl _class_name_85
_class_name_85:
    .ascii "TypeError"
.globl _class_name_88
_class_name_88:
    .ascii "ArithmeticError"
.globl _class_name_89
_class_name_89:
    .ascii "ReflectionException"
.globl _class_name_91
_class_name_91:
    .ascii "OutOfBoundsException"
.globl _class_name_95
_class_name_95:
    .ascii "OutOfRangeException"
.globl _class_name_99
_class_name_99:
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
    .quad 46
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 93
.globl _generator_class_id
_generator_class_id:
    .quad 7
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 0
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 43
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 1
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 29
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 27
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 23
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 3
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 95
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 91
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 99
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 85
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 80
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 89
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 88
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_0
    .quad _interface_methods_5
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_23
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_27
    .quad _class_interfaces_28
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
    .quad _class_interfaces_54
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_62
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
    .quad _class_interfaces_78
    .quad _class_interfaces_missing
    .quad _class_interfaces_80
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_84
    .quad _class_interfaces_85
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_88
    .quad _class_interfaces_89
    .quad _class_interfaces_missing
    .quad _class_interfaces_91
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
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
    .quad _class_json_desc_23
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_27
    .quad _class_json_desc_28
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
    .quad _class_json_desc_54
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_62
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
    .quad _class_json_desc_78
    .quad _class_json_desc_missing
    .quad _class_json_desc_80
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_84
    .quad _class_json_desc_85
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_88
    .quad _class_json_desc_89
    .quad _class_json_desc_missing
    .quad _class_json_desc_91
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_95
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_99
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 78
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 2
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 27
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 27
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 27
    .quad -1
    .quad -1
    .quad 27
    .quad 2
    .quad -1
    .quad 3
    .quad -1
    .quad -1
    .quad -1
    .quad 23
    .quad -1
    .quad -1
    .quad -1
    .quad 23
.globl _class_object_payload_sizes
_class_object_payload_sizes:
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
    .quad 0
    .quad 0
    .quad 72
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 24
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 24
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 16
    .quad 72
    .quad 0
    .quad 0
    .quad 72
    .quad 72
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 0
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 100
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_23
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_27
    .quad _class_gc_desc_28
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
    .quad _class_gc_desc_54
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_62
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
    .quad _class_gc_desc_78
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_80
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_84
    .quad _class_gc_desc_85
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_88
    .quad _class_gc_desc_89
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_91
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
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
    .quad _class_vtable_23
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_27
    .quad _class_vtable_28
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
    .quad _class_vtable_54
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_62
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
    .quad _class_vtable_78
    .quad _class_vtable_missing
    .quad _class_vtable_80
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_84
    .quad _class_vtable_85
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_88
    .quad _class_vtable_89
    .quad _class_vtable_missing
    .quad _class_vtable_91
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_23
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_27
    .quad _class_propinit_28
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_62
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_78
    .quad 0
    .quad _class_propinit_80
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_85
    .quad 0
    .quad 0
    .quad _class_propinit_88
    .quad _class_propinit_89
    .quad 0
    .quad _class_propinit_91
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_95
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_99
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_missing
    .quad _class_serprop_missing
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
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_23
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_27
    .quad _class_serprop_28
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
    .quad _class_serprop_54
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_62
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
    .quad _class_serprop_78
    .quad _class_serprop_missing
    .quad _class_serprop_80
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_84
    .quad _class_serprop_85
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_88
    .quad _class_serprop_89
    .quad _class_serprop_missing
    .quad _class_serprop_91
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
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
    .quad _class_static_vtable_23
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_27
    .quad _class_static_vtable_28
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
    .quad _class_static_vtable_54
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_62
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
    .quad _class_static_vtable_78
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_80
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_84
    .quad _class_static_vtable_85
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_88
    .quad _class_static_vtable_89
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_91
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
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
    .quad _class_callable_methods_23
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_27
    .quad _class_callable_methods_28
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
    .quad _class_callable_methods_54
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_62
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
    .quad _class_callable_methods_78
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_80
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_84
    .quad _class_callable_methods_85
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_88
    .quad _class_callable_methods_89
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_91
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
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
.globl _class_by_name_str_2
_class_by_name_str_2:
    .ascii "Exception"
.globl _class_by_name_str_3
_class_by_name_str_3:
    .ascii "RuntimeException"
.globl _class_by_name_str_23
_class_by_name_str_23:
    .ascii "LogicException"
.globl _class_by_name_str_27
_class_by_name_str_27:
    .ascii "Error"
.globl _class_by_name_str_28
_class_by_name_str_28:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_54
_class_by_name_str_54:
    .ascii "Cursor"
.globl _class_by_name_str_62
_class_by_name_str_62:
    .ascii "Tally"
.globl _class_by_name_str_78
_class_by_name_str_78:
    .ascii "JsonException"
.globl _class_by_name_str_80
_class_by_name_str_80:
    .ascii "ValueError"
.globl _class_by_name_str_84
_class_by_name_str_84:
    .ascii "stdClass"
.globl _class_by_name_str_85
_class_by_name_str_85:
    .ascii "TypeError"
.globl _class_by_name_str_88
_class_by_name_str_88:
    .ascii "ArithmeticError"
.globl _class_by_name_str_89
_class_by_name_str_89:
    .ascii "ReflectionException"
.globl _class_by_name_str_91
_class_by_name_str_91:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_95
_class_by_name_str_95:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_99
_class_by_name_str_99:
    .ascii "InvalidArgumentException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 16
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_2
    .quad 9
    .quad 2
    .quad 72
    .quad _class_by_name_str_3
    .quad 16
    .quad 3
    .quad 72
    .quad _class_by_name_str_23
    .quad 14
    .quad 23
    .quad 72
    .quad _class_by_name_str_27
    .quad 5
    .quad 27
    .quad 72
    .quad _class_by_name_str_28
    .quad 19
    .quad 28
    .quad 72
    .quad _class_by_name_str_54
    .quad 6
    .quad 54
    .quad 24
    .quad _class_by_name_str_62
    .quad 5
    .quad 62
    .quad 24
    .quad _class_by_name_str_78
    .quad 13
    .quad 78
    .quad 72
    .quad _class_by_name_str_80
    .quad 10
    .quad 80
    .quad 72
    .quad _class_by_name_str_84
    .quad 8
    .quad 84
    .quad 16
    .quad _class_by_name_str_85
    .quad 9
    .quad 85
    .quad 72
    .quad _class_by_name_str_88
    .quad 15
    .quad 88
    .quad 72
    .quad _class_by_name_str_89
    .quad 19
    .quad 89
    .quad 72
    .quad _class_by_name_str_91
    .quad 20
    .quad 91
    .quad 72
    .quad _class_by_name_str_95
    .quad 19
    .quad 95
    .quad 72
    .quad _class_by_name_str_99
    .quad 24
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
.globl _interface_methods_0
_interface_methods_0:
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
.globl _class_interfaces_2
_class_interfaces_2:
    .quad 2
    .quad 0
    .quad _class_interface_impl_2_0
    .quad 5
    .quad _class_interface_impl_2_5
.globl _class_interface_impl_2_0
_class_interface_impl_2_0:
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
    .quad 0
    .quad _class_interface_impl_3_0
    .quad 5
    .quad _class_interface_impl_3_5
.globl _class_interface_impl_3_0
_class_interface_impl_3_0:
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
.globl _class_interfaces_23
_class_interfaces_23:
    .quad 2
    .quad 0
    .quad _class_interface_impl_23_0
    .quad 5
    .quad _class_interface_impl_23_5
.globl _class_interface_impl_23_0
_class_interface_impl_23_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_23_5
_class_interface_impl_23_5:
    .quad 0
.globl _class_json_pname_23_0
_class_json_pname_23_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_23
_class_json_desc_23:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_23_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_23
_class_gc_desc_23:
    .byte 1, 0, 7, 4
.globl _class_serpname_23_0
_class_serpname_23_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_23_1
_class_serpname_23_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_23_2
_class_serpname_23_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_23_3
_class_serpname_23_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_23
_class_serprop_23:
    .quad 4
    .quad _class_serpname_23_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_23_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_23_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_23_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_23
_class_vtable_23:
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
.globl _class_static_vtable_23
_class_static_vtable_23:
    .quad 0
.globl _class_callable_method_name_23__u__u_construct
_class_callable_method_name_23__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_23__u__u_tostring
_class_callable_method_name_23__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_23_getcode
_class_callable_method_name_23_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_23_getfile
_class_callable_method_name_23_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_23_getline
_class_callable_method_name_23_getline:
    .ascii "getline"
.globl _class_callable_method_name_23_getmessage
_class_callable_method_name_23_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_23_getprevious
_class_callable_method_name_23_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_23_gettrace
_class_callable_method_name_23_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_23_gettraceasstring
_class_callable_method_name_23_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_23
_class_callable_methods_23:
    .quad 9
    .quad _class_callable_method_name_23__u__u_construct
    .quad 11
    .quad _class_callable_method_name_23__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_23_getcode
    .quad 7
    .quad _class_callable_method_name_23_getfile
    .quad 7
    .quad _class_callable_method_name_23_getline
    .quad 7
    .quad _class_callable_method_name_23_getmessage
    .quad 10
    .quad _class_callable_method_name_23_getprevious
    .quad 11
    .quad _class_callable_method_name_23_gettrace
    .quad 8
    .quad _class_callable_method_name_23_gettraceasstring
    .quad 16
.globl _class_interfaces_27
_class_interfaces_27:
    .quad 2
    .quad 0
    .quad _class_interface_impl_27_0
    .quad 5
    .quad _class_interface_impl_27_5
.globl _class_interface_impl_27_0
_class_interface_impl_27_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_27_5
_class_interface_impl_27_5:
    .quad 0
.globl _class_json_pname_27_0
_class_json_pname_27_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_27
_class_json_desc_27:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_27_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_27
_class_gc_desc_27:
    .byte 1, 0, 7, 4
.globl _class_serpname_27_0
_class_serpname_27_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_27_1
_class_serpname_27_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_27_2
_class_serpname_27_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_27_3
_class_serpname_27_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_27
_class_serprop_27:
    .quad 4
    .quad _class_serpname_27_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_27_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_27_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_27_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_27
_class_vtable_27:
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
.globl _class_static_vtable_27
_class_static_vtable_27:
    .quad 0
.globl _class_callable_method_name_27__u__u_construct
_class_callable_method_name_27__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_27__u__u_tostring
_class_callable_method_name_27__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_27_getcode
_class_callable_method_name_27_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_27_getfile
_class_callable_method_name_27_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_27_getline
_class_callable_method_name_27_getline:
    .ascii "getline"
.globl _class_callable_method_name_27_getmessage
_class_callable_method_name_27_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_27_getprevious
_class_callable_method_name_27_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_27_gettrace
_class_callable_method_name_27_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_27_gettraceasstring
_class_callable_method_name_27_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_27
_class_callable_methods_27:
    .quad 9
    .quad _class_callable_method_name_27__u__u_construct
    .quad 11
    .quad _class_callable_method_name_27__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_27_getcode
    .quad 7
    .quad _class_callable_method_name_27_getfile
    .quad 7
    .quad _class_callable_method_name_27_getline
    .quad 7
    .quad _class_callable_method_name_27_getmessage
    .quad 10
    .quad _class_callable_method_name_27_getprevious
    .quad 11
    .quad _class_callable_method_name_27_gettrace
    .quad 8
    .quad _class_callable_method_name_27_gettraceasstring
    .quad 16
.globl _class_interfaces_28
_class_interfaces_28:
    .quad 2
    .quad 0
    .quad _class_interface_impl_28_0
    .quad 5
    .quad _class_interface_impl_28_5
.globl _class_interface_impl_28_0
_class_interface_impl_28_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_28_5
_class_interface_impl_28_5:
    .quad 0
.globl _class_json_pname_28_0
_class_json_pname_28_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_28
_class_json_desc_28:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_28_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_28
_class_gc_desc_28:
    .byte 1, 0, 7, 4
.globl _class_serpname_28_0
_class_serpname_28_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_28_1
_class_serpname_28_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_28_2
_class_serpname_28_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_28_3
_class_serpname_28_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_28
_class_serprop_28:
    .quad 4
    .quad _class_serpname_28_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_28_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_28_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_28_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_28
_class_vtable_28:
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
.globl _class_static_vtable_28
_class_static_vtable_28:
    .quad 0
.globl _class_callable_method_name_28__u__u_construct
_class_callable_method_name_28__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_28__u__u_tostring
_class_callable_method_name_28__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_28_getcode
_class_callable_method_name_28_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_28_getfile
_class_callable_method_name_28_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_28_getline
_class_callable_method_name_28_getline:
    .ascii "getline"
.globl _class_callable_method_name_28_getmessage
_class_callable_method_name_28_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_28_getprevious
_class_callable_method_name_28_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_28_gettrace
_class_callable_method_name_28_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_28_gettraceasstring
_class_callable_method_name_28_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_28
_class_callable_methods_28:
    .quad 9
    .quad _class_callable_method_name_28__u__u_construct
    .quad 11
    .quad _class_callable_method_name_28__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_28_getcode
    .quad 7
    .quad _class_callable_method_name_28_getfile
    .quad 7
    .quad _class_callable_method_name_28_getline
    .quad 7
    .quad _class_callable_method_name_28_getmessage
    .quad 10
    .quad _class_callable_method_name_28_getprevious
    .quad 11
    .quad _class_callable_method_name_28_gettrace
    .quad 8
    .quad _class_callable_method_name_28_gettraceasstring
    .quad 16
.globl _class_interfaces_54
_class_interfaces_54:
    .quad 0
.globl _class_json_pname_54_0
_class_json_pname_54_0:
    .ascii "pos"
    .p2align 3
.globl _class_json_desc_54
_class_json_desc_54:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_54_0
    .quad 3
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_54
_class_gc_desc_54:
    .byte 0
.globl _class_serpname_54_0
_class_serpname_54_0:
    .byte 112, 111, 115
    .p2align 3
.globl _class_serprop_54
_class_serprop_54:
    .quad 1
    .quad _class_serpname_54_0
    .quad 3
    .quad 8
    .quad 0
    .p2align 3
.globl _class_vtable_54
_class_vtable_54:
    .quad _method_Cursor_next
    .p2align 3
.globl _class_static_vtable_54
_class_static_vtable_54:
    .quad 0
.globl _class_callable_method_name_54_next
_class_callable_method_name_54_next:
    .ascii "next"
.p2align 3
.globl _class_callable_methods_54
_class_callable_methods_54:
    .quad 1
    .quad _class_callable_method_name_54_next
    .quad 4
.globl _class_interfaces_62
_class_interfaces_62:
    .quad 0
.globl _class_json_pname_62_0
_class_json_pname_62_0:
    .ascii "total"
    .p2align 3
.globl _class_json_desc_62
_class_json_desc_62:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_62_0
    .quad 5
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_62
_class_gc_desc_62:
    .byte 0
.globl _class_serpname_62_0
_class_serpname_62_0:
    .byte 116, 111, 116, 97, 108
    .p2align 3
.globl _class_serprop_62
_class_serprop_62:
    .quad 1
    .quad _class_serpname_62_0
    .quad 5
    .quad 8
    .quad 0
    .p2align 3
.globl _class_vtable_62
_class_vtable_62:
    .quad 0
    .p2align 3
.globl _class_static_vtable_62
_class_static_vtable_62:
    .quad 0
.p2align 3
.globl _class_callable_methods_62
_class_callable_methods_62:
    .quad 0
.globl _class_interfaces_78
_class_interfaces_78:
    .quad 2
    .quad 0
    .quad _class_interface_impl_78_0
    .quad 5
    .quad _class_interface_impl_78_5
.globl _class_interface_impl_78_0
_class_interface_impl_78_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_78_5
_class_interface_impl_78_5:
    .quad 0
.globl _class_json_pname_78_0
_class_json_pname_78_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_78
_class_json_desc_78:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_78_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_78
_class_gc_desc_78:
    .byte 1, 0, 7, 4
.globl _class_serpname_78_0
_class_serpname_78_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_78_1
_class_serpname_78_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_78_2
_class_serpname_78_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_78_3
_class_serpname_78_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_78
_class_serprop_78:
    .quad 4
    .quad _class_serpname_78_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_78_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_78_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_78_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_78
_class_vtable_78:
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
.globl _class_static_vtable_78
_class_static_vtable_78:
    .quad 0
.globl _class_callable_method_name_78__u__u_construct
_class_callable_method_name_78__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_78__u__u_tostring
_class_callable_method_name_78__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_78_getcode
_class_callable_method_name_78_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_78_getfile
_class_callable_method_name_78_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_78_getline
_class_callable_method_name_78_getline:
    .ascii "getline"
.globl _class_callable_method_name_78_getmessage
_class_callable_method_name_78_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_78_getprevious
_class_callable_method_name_78_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_78_gettrace
_class_callable_method_name_78_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_78_gettraceasstring
_class_callable_method_name_78_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_78
_class_callable_methods_78:
    .quad 9
    .quad _class_callable_method_name_78__u__u_construct
    .quad 11
    .quad _class_callable_method_name_78__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_78_getcode
    .quad 7
    .quad _class_callable_method_name_78_getfile
    .quad 7
    .quad _class_callable_method_name_78_getline
    .quad 7
    .quad _class_callable_method_name_78_getmessage
    .quad 10
    .quad _class_callable_method_name_78_getprevious
    .quad 11
    .quad _class_callable_method_name_78_gettrace
    .quad 8
    .quad _class_callable_method_name_78_gettraceasstring
    .quad 16
.globl _class_interfaces_80
_class_interfaces_80:
    .quad 2
    .quad 0
    .quad _class_interface_impl_80_0
    .quad 5
    .quad _class_interface_impl_80_5
.globl _class_interface_impl_80_0
_class_interface_impl_80_0:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_80
_class_vtable_80:
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
.globl _class_interfaces_84
_class_interfaces_84:
    .quad 0
    .p2align 3
.globl _class_json_desc_84
_class_json_desc_84:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_84
_class_gc_desc_84:
    .byte 0
    .p2align 3
.globl _class_serprop_84
_class_serprop_84:
    .quad 0
    .p2align 3
.globl _class_vtable_84
_class_vtable_84:
    .quad 0
    .p2align 3
.globl _class_static_vtable_84
_class_static_vtable_84:
    .quad 0
.p2align 3
.globl _class_callable_methods_84
_class_callable_methods_84:
    .quad 0
.globl _class_interfaces_85
_class_interfaces_85:
    .quad 2
    .quad 0
    .quad _class_interface_impl_85_0
    .quad 5
    .quad _class_interface_impl_85_5
.globl _class_interface_impl_85_0
_class_interface_impl_85_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_85_5
_class_interface_impl_85_5:
    .quad 0
.globl _class_json_pname_85_0
_class_json_pname_85_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_85
_class_json_desc_85:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_85_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_85
_class_gc_desc_85:
    .byte 1, 0, 7, 4
.globl _class_serpname_85_0
_class_serpname_85_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_85_1
_class_serpname_85_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_85_2
_class_serpname_85_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_85_3
_class_serpname_85_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_85
_class_serprop_85:
    .quad 4
    .quad _class_serpname_85_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_85_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_85_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_85_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_85
_class_vtable_85:
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
.globl _class_static_vtable_85
_class_static_vtable_85:
    .quad 0
.globl _class_callable_method_name_85__u__u_construct
_class_callable_method_name_85__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_85__u__u_tostring
_class_callable_method_name_85__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_85_getcode
_class_callable_method_name_85_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_85_getfile
_class_callable_method_name_85_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_85_getline
_class_callable_method_name_85_getline:
    .ascii "getline"
.globl _class_callable_method_name_85_getmessage
_class_callable_method_name_85_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_85_getprevious
_class_callable_method_name_85_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_85_gettrace
_class_callable_method_name_85_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_85_gettraceasstring
_class_callable_method_name_85_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_85
_class_callable_methods_85:
    .quad 9
    .quad _class_callable_method_name_85__u__u_construct
    .quad 11
    .quad _class_callable_method_name_85__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_85_getcode
    .quad 7
    .quad _class_callable_method_name_85_getfile
    .quad 7
    .quad _class_callable_method_name_85_getline
    .quad 7
    .quad _class_callable_method_name_85_getmessage
    .quad 10
    .quad _class_callable_method_name_85_getprevious
    .quad 11
    .quad _class_callable_method_name_85_gettrace
    .quad 8
    .quad _class_callable_method_name_85_gettraceasstring
    .quad 16
.globl _class_interfaces_88
_class_interfaces_88:
    .quad 2
    .quad 0
    .quad _class_interface_impl_88_0
    .quad 5
    .quad _class_interface_impl_88_5
.globl _class_interface_impl_88_0
_class_interface_impl_88_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_88_5
_class_interface_impl_88_5:
    .quad 0
.globl _class_json_pname_88_0
_class_json_pname_88_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_88
_class_json_desc_88:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_88_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_88
_class_gc_desc_88:
    .byte 1, 0, 7, 4
.globl _class_serpname_88_0
_class_serpname_88_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_88_1
_class_serpname_88_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_88_2
_class_serpname_88_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_88_3
_class_serpname_88_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_88
_class_serprop_88:
    .quad 4
    .quad _class_serpname_88_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_88_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_88_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_88_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_88
_class_vtable_88:
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
.globl _class_static_vtable_88
_class_static_vtable_88:
    .quad 0
.globl _class_callable_method_name_88__u__u_construct
_class_callable_method_name_88__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_88__u__u_tostring
_class_callable_method_name_88__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_88_getcode
_class_callable_method_name_88_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_88_getfile
_class_callable_method_name_88_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_88_getline
_class_callable_method_name_88_getline:
    .ascii "getline"
.globl _class_callable_method_name_88_getmessage
_class_callable_method_name_88_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_88_getprevious
_class_callable_method_name_88_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_88_gettrace
_class_callable_method_name_88_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_88_gettraceasstring
_class_callable_method_name_88_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_88
_class_callable_methods_88:
    .quad 9
    .quad _class_callable_method_name_88__u__u_construct
    .quad 11
    .quad _class_callable_method_name_88__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_88_getcode
    .quad 7
    .quad _class_callable_method_name_88_getfile
    .quad 7
    .quad _class_callable_method_name_88_getline
    .quad 7
    .quad _class_callable_method_name_88_getmessage
    .quad 10
    .quad _class_callable_method_name_88_getprevious
    .quad 11
    .quad _class_callable_method_name_88_gettrace
    .quad 8
    .quad _class_callable_method_name_88_gettraceasstring
    .quad 16
.globl _class_interfaces_89
_class_interfaces_89:
    .quad 2
    .quad 0
    .quad _class_interface_impl_89_0
    .quad 5
    .quad _class_interface_impl_89_5
.globl _class_interface_impl_89_0
_class_interface_impl_89_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_89_5
_class_interface_impl_89_5:
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
    .quad 0
    .quad _class_interface_impl_91_0
    .quad 5
    .quad _class_interface_impl_91_5
.globl _class_interface_impl_91_0
_class_interface_impl_91_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_91_5
_class_interface_impl_91_5:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_91
_class_vtable_91:
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
.globl _class_interfaces_95
_class_interfaces_95:
    .quad 2
    .quad 0
    .quad _class_interface_impl_95_0
    .quad 5
    .quad _class_interface_impl_95_5
.globl _class_interface_impl_95_0
_class_interface_impl_95_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_95_5
_class_interface_impl_95_5:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_95_0
_class_serpname_95_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_95_1
_class_serpname_95_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_95_2
_class_serpname_95_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_95_3
_class_serpname_95_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_95
_class_serprop_95:
    .quad 4
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
    .quad _class_serpname_95_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_95
_class_vtable_95:
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
    .quad 2
    .quad 0
    .quad _class_interface_impl_99_0
    .quad 5
    .quad _class_interface_impl_99_5
.globl _class_interface_impl_99_0
_class_interface_impl_99_0:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_99_5
_class_interface_impl_99_5:
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
    .quad 84

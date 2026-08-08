.align 2

.globl _class_propinit_0
_class_propinit_0:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_0_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_0_epilogue
_class_propinit_0_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_1
_class_propinit_1:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_1_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_1_epilogue
_class_propinit_1_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_2
_class_propinit_2:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_2_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_2_epilogue
_class_propinit_2_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_3
_class_propinit_3:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_3_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_3_epilogue
_class_propinit_3_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_4
_class_propinit_4:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_4_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_4_epilogue
_class_propinit_4_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_5
_class_propinit_5:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_5_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_5_epilogue
_class_propinit_5_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_6
_class_propinit_6:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_6_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_6_epilogue
_class_propinit_6_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_15
_class_propinit_15:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_15_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_15_epilogue
_class_propinit_15_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_23
_class_propinit_23:
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
_eir__class_propinit_23_entry_0:
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
    b _class_propinit_23_epilogue
_class_propinit_23_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_24
_class_propinit_24:
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
_eir__class_propinit_24_entry_0:
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
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-48]
    ldur x0, [x29, #-104]
    stur x0, [x29, #-56]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-64]
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-64]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #24]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-64]
    bl __rt_decref_any
    ldur x0, [x29, #-48]
    bl __rt_decref_any
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
    b _class_propinit_24_epilogue
_class_propinit_24_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
.align 2

.globl _class_propinit_25
_class_propinit_25:
    ; prologue
    sub sp, sp, #176
    stp x29, x30, [sp, #160]
    add x29, sp, #160
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-160]
    ; param $this from x0
    stur x0, [x29, #-152]
_eir__class_propinit_25_entry_0:
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-152]
    stur x0, [x29, #-16]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-24]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-24]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-24]
    bl __rt_decref_any
    ldur x0, [x29, #-8]
    bl __rt_decref_any
    ldur x0, [x29, #-152]
    stur x0, [x29, #-48]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
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
    ldur x0, [x29, #-152]
    stur x0, [x29, #-88]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ldur x9, [x29, #-88]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #40]
    str x2, [x9, #48]
    ldur x0, [x29, #-152]
    stur x0, [x29, #-128]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-144]
    stur x2, [x29, #-136]
    ldur x9, [x29, #-128]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-144]
    ldur x2, [x29, #-136]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #56]
    str x2, [x9, #64]
    b _class_propinit_25_epilogue
_class_propinit_25_epilogue:
    ldp x29, x30, [sp, #160]
    add sp, sp, #176
    ret
.align 2

.globl _class_propinit_26
_class_propinit_26:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
_eir__class_propinit_26_entry_0:
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
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-48]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-64]
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-64]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #24]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-64]
    bl __rt_decref_any
    ldur x0, [x29, #-48]
    bl __rt_decref_any
    ldur x0, [x29, #-120]
    stur x0, [x29, #-88]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ldur x9, [x29, #-88]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #40]
    str x2, [x9, #48]
    b _class_propinit_26_epilogue
_class_propinit_26_epilogue:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
.align 2

.globl _class_propinit_27
_class_propinit_27:
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
_eir__class_propinit_27_entry_0:
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
    b _class_propinit_27_epilogue
_class_propinit_27_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_28
_class_propinit_28:
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
_eir__class_propinit_28_entry_0:
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
    b _class_propinit_28_epilogue
_class_propinit_28_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
.align 2

.globl _class_propinit_29
_class_propinit_29:
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
_eir__class_propinit_29_entry_0:
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
    b _class_propinit_29_epilogue
_class_propinit_29_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_30
_class_propinit_30:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_30_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_30_epilogue
_class_propinit_30_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_34
_class_propinit_34:
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
_eir__class_propinit_34_entry_0:
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
    b _class_propinit_34_epilogue
_class_propinit_34_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_35
_class_propinit_35:
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
_eir__class_propinit_35_entry_0:
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
    b _class_propinit_35_epilogue
_class_propinit_35_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_37
_class_propinit_37:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_37_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_37_epilogue
_class_propinit_37_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_38
_class_propinit_38:
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
_eir__class_propinit_38_entry_0:
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
    b _class_propinit_38_epilogue
_class_propinit_38_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
.align 2

.globl _class_propinit_40
_class_propinit_40:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_40_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_40_epilogue
_class_propinit_40_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_41
_class_propinit_41:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_41_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_41_epilogue
_class_propinit_41_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_43
_class_propinit_43:
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
_eir__class_propinit_43_entry_0:
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
    b _class_propinit_43_epilogue
_class_propinit_43_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_45
_class_propinit_45:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_45_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_45_epilogue
_class_propinit_45_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
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
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
_eir__class_propinit_46_entry_0:
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
    b _class_propinit_46_epilogue
_class_propinit_46_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_47
_class_propinit_47:
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
_eir__class_propinit_47_entry_0:
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
    b _class_propinit_47_epilogue
_class_propinit_47_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
.align 2

.globl _class_propinit_50
_class_propinit_50:
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
_eir__class_propinit_50_entry_0:
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
    b _class_propinit_50_epilogue
_class_propinit_50_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
.align 2

.globl _class_propinit_53
_class_propinit_53:
    ; prologue
    sub sp, sp, #240
    stp x29, x30, [sp, #224]
    add x29, sp, #224
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-216]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-208]
    ; param $this from x0
    stur x0, [x29, #-200]
_eir__class_propinit_53_entry_0:
    ldur x0, [x29, #-200]
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
    ldur x0, [x29, #-200]
    stur x0, [x29, #-64]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ldur x9, [x29, #-64]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
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
    ldur x0, [x29, #-200]
    stur x0, [x29, #-96]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-96]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ldur x0, [x29, #-200]
    stur x0, [x29, #-120]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-120]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-136]
    ldur x0, [x29, #-200]
    stur x0, [x29, #-144]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-152]
    ldur x9, [x29, #-144]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-152]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #72]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #72]
    str xzr, [x9, #80]
    ldur x0, [x29, #-152]
    bl __rt_decref_any
    ldur x0, [x29, #-136]
    bl __rt_decref_any
    ldur x0, [x29, #-200]
    stur x0, [x29, #-176]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ldur x9, [x29, #-176]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-192]
    ldur x2, [x29, #-184]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #88]
    str x2, [x9, #96]
    b _class_propinit_53_epilogue
_class_propinit_53_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-208]
    ldp x29, x30, [sp, #224]
    add sp, sp, #240
    ret
.align 2

.globl _class_propinit_57
_class_propinit_57:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_57_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_57_epilogue
_class_propinit_57_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_58
_class_propinit_58:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_58_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_58_epilogue
_class_propinit_58_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_59
_class_propinit_59:
    ; prologue
    sub sp, sp, #176
    stp x29, x30, [sp, #160]
    add x29, sp, #160
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-160]
    ; param $this from x0
    stur x0, [x29, #-152]
_eir__class_propinit_59_entry_0:
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-152]
    stur x0, [x29, #-16]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-24]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-24]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-24]
    bl __rt_decref_any
    ldur x0, [x29, #-8]
    bl __rt_decref_any
    ldur x0, [x29, #-152]
    stur x0, [x29, #-48]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
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
    ldur x0, [x29, #-152]
    stur x0, [x29, #-88]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ldur x9, [x29, #-88]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #40]
    str x2, [x9, #48]
    ldur x0, [x29, #-152]
    stur x0, [x29, #-128]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-144]
    stur x2, [x29, #-136]
    ldur x9, [x29, #-128]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-144]
    ldur x2, [x29, #-136]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #56]
    str x2, [x9, #64]
    b _class_propinit_59_epilogue
_class_propinit_59_epilogue:
    ldp x29, x30, [sp, #160]
    add sp, sp, #176
    ret
.align 2

.globl _class_propinit_60
_class_propinit_60:
    ; prologue
    sub sp, sp, #256
    stp x29, x30, [sp, #240]
    add x29, sp, #240
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-232]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-224]
    ; param $this from x0
    stur x0, [x29, #-216]
_eir__class_propinit_60_entry_0:
    ldur x0, [x29, #-216]
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
    ldur x0, [x29, #-216]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-216]
    stur x0, [x29, #-80]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    ldur x0, [x29, #-216]
    stur x0, [x29, #-104]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-216]
    stur x0, [x29, #-128]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-128]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #72]
    str xzr, [x9, #80]
    ldur x0, [x29, #-216]
    stur x0, [x29, #-152]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-152]
    str x9, [sp, #-16]!
    ldr x0, [x9, #88]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #88]
    str xzr, [x9, #96]
    ldur x0, [x29, #-216]
    stur x0, [x29, #-184]
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-200]
    stur x2, [x29, #-192]
    ldur x9, [x29, #-184]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-200]
    ldur x2, [x29, #-192]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #104]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #104]
    str x2, [x9, #112]
    b _class_propinit_60_epilogue
_class_propinit_60_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-224]
    ldp x29, x30, [sp, #240]
    add sp, sp, #256
    ret
.align 2

.globl _class_propinit_63
_class_propinit_63:
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
_eir__class_propinit_63_entry_0:
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
    b _class_propinit_63_epilogue
_class_propinit_63_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
.align 2

.globl _class_propinit_70
_class_propinit_70:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_70_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_70_epilogue
_class_propinit_70_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_71
_class_propinit_71:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_71_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_71_epilogue
_class_propinit_71_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_72
_class_propinit_72:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_72_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_72_epilogue
_class_propinit_72_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_75
_class_propinit_75:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_75_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_75_epilogue
_class_propinit_75_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_76
_class_propinit_76:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_76_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_76_epilogue
_class_propinit_76_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_77
_class_propinit_77:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_77_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_77_epilogue
_class_propinit_77_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_78
_class_propinit_78:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_78_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_78_epilogue
_class_propinit_78_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_79
_class_propinit_79:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_79_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_79_epilogue
_class_propinit_79_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_81
_class_propinit_81:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_81_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_81_epilogue
_class_propinit_81_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_82
_class_propinit_82:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_82_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_82_epilogue
_class_propinit_82_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_84
_class_propinit_84:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_84_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_84_epilogue
_class_propinit_84_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_86
_class_propinit_86:
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
_eir__class_propinit_86_entry_0:
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
    b _class_propinit_86_epilogue
_class_propinit_86_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _class_propinit_88
_class_propinit_88:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
_eir__class_propinit_88_entry_0:
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-24]
    ldur x9, [x29, #-16]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-24]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #8]
    str xzr, [x9, #16]
    ldur x0, [x29, #-24]
    bl __rt_decref_any
    ldur x0, [x29, #-8]
    bl __rt_decref_any
    b _class_propinit_88_epilogue
_class_propinit_88_epilogue:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _class_propinit_89
_class_propinit_89:
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
_eir__class_propinit_89_entry_0:
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
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #40]
    str xzr, [x9, #48]
    b _class_propinit_89_epilogue
_class_propinit_89_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
.align 2

.globl _class_propinit_90
_class_propinit_90:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_90_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_90_epilogue
_class_propinit_90_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_91
_class_propinit_91:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_91_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_91_epilogue
_class_propinit_91_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _class_propinit_92
_class_propinit_92:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-80]
    ; param $this from x0
    stur x0, [x29, #-72]
_eir__class_propinit_92_entry_0:
    ldur x0, [x29, #-72]
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
    ldur x0, [x29, #-72]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    b _class_propinit_92_epilogue
_class_propinit_92_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-80]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
.align 2

.globl _method_ReflectionAttribute__u__u_construct
_method_ReflectionAttribute__u__u_construct:
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
_eir_ReflectionAttribute____construct_entry_0:
    b _method_ReflectionAttribute__u__u_construct_epilogue
_method_ReflectionAttribute__u__u_construct_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
.align 2

.globl _method_ReflectionAttribute_getname
_method_ReflectionAttribute_getname:
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
_eir_ReflectionAttribute__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionAttribute__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionAttribute__getName_typed_prop_initialized_0:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionAttribute_getname_epilogue
_method_ReflectionAttribute_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionAttribute_getarguments
_method_ReflectionAttribute_getarguments:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionAttribute__getArguments_entry_0:
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    b _method_ReflectionAttribute_getarguments_epilogue
_method_ReflectionAttribute_getarguments_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionAttribute_newinstance
_method_ReflectionAttribute_newinstance:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionAttribute__newInstance_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionAttribute_newinstance_epilogue
_method_ReflectionAttribute_newinstance_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionClass__u__u_construct
_method_ReflectionClass__u__u_construct:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; param $class_name from x1,x2
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
_eir_ReflectionClass____construct_entry_0:
    b _method_ReflectionClass__u__u_construct_epilogue
_method_ReflectionClass__u__u_construct_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionClass_getname
_method_ReflectionClass_getname:
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
_eir_ReflectionClass__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionClass__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #96
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionClass__getName_typed_prop_initialized_0:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionClass_getname_epilogue
_method_ReflectionClass_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionClass_getattributes
_method_ReflectionClass_getattributes:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; param $name from x1
    stur x1, [x29, #-48]
    ; param $flags from x2
    stur x2, [x29, #-56]
_eir_ReflectionClass__getAttributes_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionClass__getAttributes_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #97
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionClass__getAttributes_typed_prop_initialized_0:
    ldr x0, [x9, #24]
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    b _method_ReflectionClass_getattributes_epilogue
_method_ReflectionClass_getattributes_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionClass_newinstancewithoutconstructor
_method_ReflectionClass_newinstancewithoutconstructor:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionClass__newInstanceWithoutConstructor_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionClass_newinstancewithoutconstructor_epilogue
_method_ReflectionClass_newinstancewithoutconstructor_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionClass_getproperty
_method_ReflectionClass_getproperty:
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
    stur x0, [x29, #-24]
    ; param $name from x1,x2
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
_eir_ReflectionClass__getProperty_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionClass_getproperty_epilogue
_method_ReflectionClass_getproperty_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionMethod__u__u_construct
_method_ReflectionMethod__u__u_construct:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; param $class_name from x1
    stur x1, [x29, #-16]
    ; param $method_name from x2,x3
    stur x2, [x29, #-32]
    stur x3, [x29, #-24]
_eir_ReflectionMethod____construct_entry_0:
    b _method_ReflectionMethod__u__u_construct_epilogue
_method_ReflectionMethod__u__u_construct_epilogue:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionMethod_getattributes
_method_ReflectionMethod_getattributes:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; param $name from x1
    stur x1, [x29, #-48]
    ; param $flags from x2
    stur x2, [x29, #-56]
_eir_ReflectionMethod__getAttributes_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionMethod__getAttributes_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #98
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionMethod__getAttributes_typed_prop_initialized_0:
    ldr x0, [x9, #8]
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    b _method_ReflectionMethod_getattributes_epilogue
_method_ReflectionMethod_getattributes_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionMethod_getname
_method_ReflectionMethod_getname:
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
_eir_ReflectionMethod__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionMethod__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #97
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionMethod__getName_typed_prop_initialized_0:
    ldr x1, [x9, #24]
    ldr x2, [x9, #32]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionMethod_getname_epilogue
_method_ReflectionMethod_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionMethod_ispublic
_method_ReflectionMethod_ispublic:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionMethod__isPublic_entry_0:
    mov x0, #1
    mov x12, x0
    mov x0, x12
    b _method_ReflectionMethod_ispublic_epilogue
_method_ReflectionMethod_ispublic_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionMethod_getclosure
_method_ReflectionMethod_getclosure:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-48]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-40]
    ; param $this from x0
    stur x0, [x29, #-24]
    ; param $object from x1
    stur x1, [x29, #-32]
_eir_ReflectionMethod__getClosure_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionMethod_getclosure_epilogue
_method_ReflectionMethod_getclosure_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-40]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionMethod_getdeclaringclass
_method_ReflectionMethod_getdeclaringclass:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionMethod__getDeclaringClass_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionMethod_getdeclaringclass_epilogue
_method_ReflectionMethod_getdeclaringclass_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionMethod_isstatic
_method_ReflectionMethod_isstatic:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionMethod__isStatic_entry_0:
    mov x0, #0
    mov x12, x0
    mov x0, x12
    b _method_ReflectionMethod_isstatic_epilogue
_method_ReflectionMethod_isstatic_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionProperty__u__u_construct
_method_ReflectionProperty__u__u_construct:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; param $class_name from x1
    stur x1, [x29, #-16]
    ; param $property_name from x2,x3
    stur x2, [x29, #-32]
    stur x3, [x29, #-24]
_eir_ReflectionProperty____construct_entry_0:
    b _method_ReflectionProperty__u__u_construct_epilogue
_method_ReflectionProperty__u__u_construct_epilogue:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionProperty_getattributes
_method_ReflectionProperty_getattributes:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-40]
    ; param $name from x1
    stur x1, [x29, #-48]
    ; param $flags from x2
    stur x2, [x29, #-56]
_eir_ReflectionProperty__getAttributes_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionProperty__getAttributes_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionProperty__getAttributes_typed_prop_initialized_0:
    ldr x0, [x9, #8]
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    b _method_ReflectionProperty_getattributes_epilogue
_method_ReflectionProperty_getattributes_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionProperty_gettype
_method_ReflectionProperty_gettype:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__getType_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionProperty_gettype_epilogue
_method_ReflectionProperty_gettype_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionProperty_getname
_method_ReflectionProperty_getname:
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
_eir_ReflectionProperty__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionProperty__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #99
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionProperty__getName_typed_prop_initialized_0:
    ldr x1, [x9, #24]
    ldr x2, [x9, #32]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionProperty_getname_epilogue
_method_ReflectionProperty_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionProperty_getdeclaringfunction
_method_ReflectionProperty_getdeclaringfunction:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__getDeclaringFunction_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionProperty_getdeclaringfunction_epilogue
_method_ReflectionProperty_getdeclaringfunction_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionProperty_setvalue
_method_ReflectionProperty_setvalue:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; param $objectOrValue from x1
    stur x1, [x29, #-16]
    ; param $value from x2
    stur x2, [x29, #-24]
_eir_ReflectionProperty__setValue_entry_0:
    b _method_ReflectionProperty_setvalue_epilogue
_method_ReflectionProperty_setvalue_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionProperty_getdefaultvalue
_method_ReflectionProperty_getdefaultvalue:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__getDefaultValue_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionProperty_getdefaultvalue_epilogue
_method_ReflectionProperty_getdefaultvalue_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionProperty_hasdefaultvalue
_method_ReflectionProperty_hasdefaultvalue:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__hasDefaultValue_entry_0:
    mov x0, #0
    mov x12, x0
    mov x0, x12
    b _method_ReflectionProperty_hasdefaultvalue_epilogue
_method_ReflectionProperty_hasdefaultvalue_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionProperty_isdefaultvalueavailable
_method_ReflectionProperty_isdefaultvalueavailable:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__isDefaultValueAvailable_entry_0:
    mov x0, #0
    mov x12, x0
    mov x0, x12
    b _method_ReflectionProperty_isdefaultvalueavailable_epilogue
_method_ReflectionProperty_isdefaultvalueavailable_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionProperty_getdeclaringclass
_method_ReflectionProperty_getdeclaringclass:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionProperty__getDeclaringClass_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionProperty_getdeclaringclass_epilogue
_method_ReflectionProperty_getdeclaringclass_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction__u__u_construct
_method_ReflectionFunction__u__u_construct:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-8]
    ; param $name from x1,x2
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
_eir_ReflectionFunction____construct_entry_0:
    b _method_ReflectionFunction__u__u_construct_epilogue
_method_ReflectionFunction__u__u_construct_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionFunction_getname
_method_ReflectionFunction_getname:
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
_eir_ReflectionFunction__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionFunction__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #99
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionFunction__getName_typed_prop_initialized_0:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionFunction_getname_epilogue
_method_ReflectionFunction_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionFunction_getshortname
_method_ReflectionFunction_getshortname:
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
_eir_ReflectionFunction__getShortName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionFunction__getShortName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionFunction__getShortName_typed_prop_initialized_0:
    ldr x1, [x9, #24]
    ldr x2, [x9, #32]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionFunction_getshortname_epilogue
_method_ReflectionFunction_getshortname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionFunction_getnumberofparameters
_method_ReflectionFunction_getnumberofparameters:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionFunction__getNumberOfParameters_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #48]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionFunction__getNumberOfParameters_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #105
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionFunction__getNumberOfParameters_typed_prop_initialized_0:
    ldr x0, [x9, #40]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionFunction_getnumberofparameters_epilogue
_method_ReflectionFunction_getnumberofparameters_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction_getnumberofrequiredparameters
_method_ReflectionFunction_getnumberofrequiredparameters:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionFunction__getNumberOfRequiredParameters_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #64]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionFunction__getNumberOfRequiredParameters_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #107
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionFunction__getNumberOfRequiredParameters_typed_prop_initialized_0:
    ldr x0, [x9, #56]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionFunction_getnumberofrequiredparameters_epilogue
_method_ReflectionFunction_getnumberofrequiredparameters_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction_getparameters
_method_ReflectionFunction_getparameters:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-48]
    ; param $this from x0
    stur x0, [x29, #-40]
_eir_ReflectionFunction__getParameters_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #80]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionFunction__getParameters_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #101
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionFunction__getParameters_typed_prop_initialized_0:
    ldr x0, [x9, #72]
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    b _method_ReflectionFunction_getparameters_epilogue
_method_ReflectionFunction_getparameters_epilogue:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction_getclosurethis
_method_ReflectionFunction_getclosurethis:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionFunction__getClosureThis_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionFunction_getclosurethis_epilogue
_method_ReflectionFunction_getclosurethis_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction_invoke
_method_ReflectionFunction_invoke:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-48]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-40]
    ; param $this from x0
    stur x0, [x29, #-24]
    ; param $args from x1
    stur x1, [x29, #-32]
_eir_ReflectionFunction__invoke_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionFunction_invoke_epilogue
_method_ReflectionFunction_invoke_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-40]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionFunction_getclosurecalledclass
_method_ReflectionFunction_getclosurecalledclass:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionFunction__getClosureCalledClass_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionFunction_getclosurecalledclass_epilogue
_method_ReflectionFunction_getclosurecalledclass_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_getname
_method_ReflectionParameter_getname:
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
_eir_ReflectionParameter__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__getName_typed_prop_initialized_0:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionParameter_getname_epilogue
_method_ReflectionParameter_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionParameter_getposition
_method_ReflectionParameter_getposition:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__getPosition_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__getPosition_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #104
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__getPosition_typed_prop_initialized_0:
    ldr x0, [x9, #24]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionParameter_getposition_epilogue
_method_ReflectionParameter_getposition_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_isoptional
_method_ReflectionParameter_isoptional:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__isOptional_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #48]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__isOptional_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #104
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__isOptional_typed_prop_initialized_0:
    ldr x0, [x9, #40]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionParameter_isoptional_epilogue
_method_ReflectionParameter_isoptional_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_isvariadic
_method_ReflectionParameter_isvariadic:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__isVariadic_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #64]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__isVariadic_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #104
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__isVariadic_typed_prop_initialized_0:
    ldr x0, [x9, #56]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionParameter_isvariadic_epilogue
_method_ReflectionParameter_isvariadic_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_hastype
_method_ReflectionParameter_hastype:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__hasType_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #80]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__hasType_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_19@PAGE
    add x1, x1, _str_19@PAGEOFF
    mov x2, #104
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__hasType_typed_prop_initialized_0:
    ldr x0, [x9, #72]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionParameter_hastype_epilogue
_method_ReflectionParameter_hastype_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_gettype
_method_ReflectionParameter_gettype:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__getType_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #96]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionParameter__getType_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_20@PAGE
    add x1, x1, _str_20@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionParameter__getType_typed_prop_initialized_0:
    ldr x0, [x9, #88]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionParameter_gettype_epilogue
_method_ReflectionParameter_gettype_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionParameter_getdeclaringfunction
_method_ReflectionParameter_getdeclaringfunction:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__getDeclaringFunction_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionParameter_getdeclaringfunction_epilogue
_method_ReflectionParameter_getdeclaringfunction_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_isdefaultvalueavailable
_method_ReflectionParameter_isdefaultvalueavailable:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__isDefaultValueAvailable_entry_0:
    mov x0, #0
    mov x12, x0
    mov x0, x12
    b _method_ReflectionParameter_isdefaultvalueavailable_epilogue
_method_ReflectionParameter_isdefaultvalueavailable_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionParameter_hasdefaultvalue
_method_ReflectionParameter_hasdefaultvalue:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__hasDefaultValue_entry_0:
    mov x0, #0
    mov x12, x0
    mov x0, x12
    b _method_ReflectionParameter_hasdefaultvalue_epilogue
_method_ReflectionParameter_hasdefaultvalue_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
.align 2

.globl _method_ReflectionParameter_getdefaultvalue
_method_ReflectionParameter_getdefaultvalue:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__getDefaultValue_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionParameter_getdefaultvalue_epilogue
_method_ReflectionParameter_getdefaultvalue_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionParameter_getdeclaringclass
_method_ReflectionParameter_getdeclaringclass:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionParameter__getDeclaringClass_entry_0:
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
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    b _method_ReflectionParameter_getdeclaringclass_epilogue
_method_ReflectionParameter_getdeclaringclass_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionNamedType_getname
_method_ReflectionNamedType_getname:
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
_eir_ReflectionNamedType__getName_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionNamedType__getName_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_21@PAGE
    add x1, x1, _str_21@PAGEOFF
    mov x2, #100
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionNamedType__getName_typed_prop_initialized_0:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    bl __rt_str_persist
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    b _method_ReflectionNamedType_getname_epilogue
_method_ReflectionNamedType_getname_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
.align 2

.globl _method_ReflectionNamedType_allowsnull
_method_ReflectionNamedType_allowsnull:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionNamedType__allowsNull_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #32]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionNamedType__allowsNull_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_22@PAGE
    add x1, x1, _str_22@PAGEOFF
    mov x2, #107
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionNamedType__allowsNull_typed_prop_initialized_0:
    ldr x0, [x9, #24]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionNamedType_allowsnull_epilogue
_method_ReflectionNamedType_allowsnull_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _method_ReflectionNamedType_isbuiltin
_method_ReflectionNamedType_isbuiltin:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-40]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-32]
    ; param $this from x0
    stur x0, [x29, #-24]
_eir_ReflectionNamedType__isBuiltin_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x9, [x29, #-8]
    ldr x10, [x9, #48]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_ReflectionNamedType__isBuiltin_typed_prop_initialized_0
    mov x0, #2
    adrp x1, _str_23@PAGE
    add x1, x1, _str_23@PAGEOFF
    mov x2, #103
    mov x16, #4
    svc #0x80
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_ReflectionNamedType__isBuiltin_typed_prop_initialized_0:
    ldr x0, [x9, #40]
    mov x21, x0
    mov x0, x21
    b _method_ReflectionNamedType_isbuiltin_epilogue
_method_ReflectionNamedType_isbuiltin_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-32]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #4080
    sub sp, sp, #1776
    mov x9, sp
    add x9, x9, #4080
    add x9, x9, #1760
    stp x29, x30, [x9]
    mov x29, sp
    add x29, x29, #4080
    add x29, x29, #1760
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1737
    str x21, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1265
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1289
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1297
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1305
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1313
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1321
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1361
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1417
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1425
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1449
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1457
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1465
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1481
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1489
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1505
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1537
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1545
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1561
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1601
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    str xzr, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    str xzr, [x9]
_eir_main_entry_0:
    ; @src line=9 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=29
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    stur x0, [x29, #-8]
    ; @src line=9 col=30
    mov x0, #10
    mov x21, x0
    ; @src line=9 col=30
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=9 col=34
    mov x0, #20
    mov x21, x0
    ; @src line=9 col=34
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=9 col=38
    mov x0, #30
    mov x21, x0
    ; @src line=9 col=38
    mov x1, x21
    ldur x9, [x29, #-8]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-8]
    ; @src line=9 col=1
    mov x0, #0
    mov x21, x0
    ; @src line=9 col=1
    mov x0, x21
    ldur x9, [x29, #-8]
    cmp x0, #0
    b.lt _eir_main_array_get_null_0
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_0
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    b _eir_main_array_get_done_1
_eir_main_array_get_null_0:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_1:
    mov x21, x0
    ; @src line=9 col=1
    ; @src line=9 col=1
    mov x0, #1
    mov x21, x0
    ; @src line=9 col=1
    mov x0, x21
    ldur x9, [x29, #-8]
    cmp x0, #0
    b.lt _eir_main_array_get_null_2
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_2
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    b _eir_main_array_get_done_3
_eir_main_array_get_null_2:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_3:
    mov x21, x0
    ; @src line=9 col=1
    ; @src line=9 col=1
    mov x0, #2
    mov x21, x0
    ; @src line=9 col=1
    mov x0, x21
    ldur x9, [x29, #-8]
    cmp x0, #0
    b.lt _eir_main_array_get_null_4
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_4
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    b _eir_main_array_get_done_5
_eir_main_array_get_null_4:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_5:
    mov x21, x0
    ; @src line=9 col=1
    ; @src line=10 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=6
    mov x0, #10
    mov x21, x0
    ; @src line=10 col=13
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=10 col=15
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=10 col=13
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    ldur x3, [x29, #-120]
    ldur x4, [x29, #-112]
    bl __rt_concat
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=10 col=13
    ; @src line=10 col=21
    mov x0, #20
    mov x21, x0
    ; @src line=10 col=19
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-160]
    stur x2, [x29, #-152]
    ; @src line=10 col=19
    ldur x1, [x29, #-136]
    ldur x2, [x29, #-128]
    ldur x3, [x29, #-160]
    ldur x4, [x29, #-152]
    bl __rt_concat
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=10 col=19
    ; @src line=10 col=19
    ; @src line=10 col=31
    adrp x1, _str_24@PAGE
    add x1, x1, _str_24@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ; @src line=10 col=29
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    ldur x3, [x29, #-192]
    ldur x4, [x29, #-184]
    bl __rt_concat
    stur x1, [x29, #-208]
    stur x2, [x29, #-200]
    ; @src line=10 col=29
    ; @src line=10 col=37
    mov x0, #30
    mov x21, x0
    ; @src line=10 col=35
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-232]
    stur x2, [x29, #-224]
    ; @src line=10 col=35
    ldur x1, [x29, #-208]
    ldur x2, [x29, #-200]
    ldur x3, [x29, #-232]
    ldur x4, [x29, #-224]
    bl __rt_concat
    stur x1, [x29, #-248]
    stur x2, [x29, #-240]
    ; @src line=10 col=35
    ; @src line=10 col=35
    ; @src line=10 col=46
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #264
    str x1, [x9]
    sub x9, x29, #256
    str x2, [x9]
    ; @src line=10 col=44
    ldur x1, [x29, #-248]
    ldur x2, [x29, #-240]
    sub x9, x29, #264
    ldr x3, [x9]
    sub x9, x29, #256
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #280
    str x1, [x9]
    sub x9, x29, #272
    str x2, [x9]
    ; @src line=10 col=44
    ; @src line=10 col=1
    sub x9, x29, #280
    ldr x1, [x9]
    sub x9, x29, #272
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=10 col=1
    ; @src line=13 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=16
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #288
    str x0, [x9]
    ; @src line=13 col=17
    mov x0, #100
    mov x21, x0
    ; @src line=13 col=17
    mov x1, x21
    sub x9, x29, #288
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #288
    str x0, [x9]
    ; @src line=13 col=22
    mov x0, #200
    mov x21, x0
    ; @src line=13 col=22
    mov x1, x21
    sub x9, x29, #288
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #288
    str x0, [x9]
    ; @src line=13 col=27
    mov x0, #300
    mov x21, x0
    ; @src line=13 col=27
    mov x1, x21
    sub x9, x29, #288
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #288
    str x0, [x9]
    ; @src line=13 col=1
    sub x9, x29, #288
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #320
    str x0, [x9]
    ; @src line=13 col=1
    sub x9, x29, #320
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1265
    str x0, [x9]
    ; @src line=13 col=1
    sub x9, x29, #288
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=13 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1265
    ldr x0, [x9]
    sub x9, x29, #328
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=13 col=1
    mov x0, x21
    sub x9, x29, #328
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_6
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_6
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    mov x1, #0
    b _eir_main_array_get_done_7
_eir_main_array_get_null_6:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, #8
_eir_main_array_get_done_7:
    sub x9, x29, #352
    str x0, [x9]
    sub x9, x29, #344
    str x1, [x9]
    ; @src line=13 col=1
    sub x9, x29, #352
    ldr x0, [x9]
    sub x9, x29, #344
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1281
    str x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1273
    str x1, [x9]
    ; @src line=14 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=6
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1281
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1273
    ldr x1, [x9]
    sub x9, x29, #368
    str x0, [x9]
    sub x9, x29, #360
    str x1, [x9]
    ; @src line=14 col=14
    sub x9, x29, #368
    ldr x0, [x9]
    sub x9, x29, #360
    ldr x1, [x9]
    cmp x1, #8
    b.eq _eir_main_tagged_to_str_null_8
    bl __rt_itoa
    b _eir_main_tagged_to_str_done_9
_eir_main_tagged_to_str_null_8:
    mov x2, #0
_eir_main_tagged_to_str_done_9:
    sub x9, x29, #384
    str x1, [x9]
    sub x9, x29, #376
    str x2, [x9]
    ; @src line=14 col=16
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #400
    str x1, [x9]
    sub x9, x29, #392
    str x2, [x9]
    ; @src line=14 col=14
    sub x9, x29, #384
    ldr x1, [x9]
    sub x9, x29, #376
    ldr x2, [x9]
    sub x9, x29, #400
    ldr x3, [x9]
    sub x9, x29, #392
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #416
    str x1, [x9]
    sub x9, x29, #408
    str x2, [x9]
    ; @src line=14 col=14
    ; @src line=14 col=1
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=14 col=1
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=34
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #424
    str x0, [x9]
    ; @src line=17 col=35
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #4
    sub x9, x29, #440
    str x1, [x9]
    sub x9, x29, #432
    str x2, [x9]
    ; @src line=17 col=45
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #3
    sub x9, x29, #456
    str x1, [x9]
    sub x9, x29, #448
    str x2, [x9]
    ; @src line=17 col=34
    sub x9, x29, #440
    ldr x1, [x9]
    sub x9, x29, #432
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #456
    ldr x1, [x9]
    sub x9, x29, #448
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #424
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #424
    str x0, [x9]
    ; @src line=17 col=52
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #472
    str x1, [x9]
    sub x9, x29, #464
    str x2, [x9]
    ; @src line=17 col=60
    mov x0, #7
    mov x21, x0
    ; @src line=17 col=34
    sub x9, x29, #472
    ldr x1, [x9]
    sub x9, x29, #464
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #424
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #424
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #424
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #488
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #488
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1289
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #424
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1289
    ldr x0, [x9]
    sub x9, x29, #496
    str x0, [x9]
    ; @src line=17 col=2
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #512
    str x1, [x9]
    sub x9, x29, #504
    str x2, [x9]
    ; @src line=17 col=1
    sub x9, x29, #512
    ldr x1, [x9]
    sub x9, x29, #504
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #496
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_10
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_12
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_13
_eir_main_hash_get_mixed_box_12:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_13:
    b _eir_main_hash_get_done_11
_eir_main_hash_get_miss_10:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_11:
    sub x9, x29, #520
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #520
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #528
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #528
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1297
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #520
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1289
    ldr x0, [x9]
    sub x9, x29, #536
    str x0, [x9]
    ; @src line=17 col=15
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #4
    sub x9, x29, #552
    str x1, [x9]
    sub x9, x29, #544
    str x2, [x9]
    ; @src line=17 col=1
    sub x9, x29, #552
    ldr x1, [x9]
    sub x9, x29, #544
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #536
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_14
    bl __rt_deref_if_reference
    cmp x3, #7
    b.ne _eir_main_hash_get_mixed_box_16
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_mixed_done_17
_eir_main_hash_get_mixed_box_16:
    mov x0, x3
    bl __rt_mixed_from_value
_eir_main_hash_get_mixed_done_17:
    b _eir_main_hash_get_done_15
_eir_main_hash_get_miss_14:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir_main_hash_get_done_15:
    sub x9, x29, #560
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #560
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #568
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #568
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1305
    str x0, [x9]
    ; @src line=17 col=1
    sub x9, x29, #560
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=18 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=6
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1297
    ldr x0, [x9]
    sub x9, x29, #576
    str x0, [x9]
    ; @src line=18 col=10
    sub x9, x29, #576
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #592
    str x1, [x9]
    sub x9, x29, #584
    str x2, [x9]
    ; @src line=18 col=12
    adrp x1, _str_29@PAGE
    add x1, x1, _str_29@PAGEOFF
    mov x2, #1
    sub x9, x29, #608
    str x1, [x9]
    sub x9, x29, #600
    str x2, [x9]
    ; @src line=18 col=10
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    sub x9, x29, #608
    ldr x3, [x9]
    sub x9, x29, #600
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #624
    str x1, [x9]
    sub x9, x29, #616
    str x2, [x9]
    ; @src line=18 col=10
    sub x9, x29, #592
    ldr x1, [x9]
    sub x9, x29, #584
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=18 col=18
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1305
    ldr x0, [x9]
    sub x9, x29, #632
    str x0, [x9]
    ; @src line=18 col=16
    sub x9, x29, #632
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #648
    str x1, [x9]
    sub x9, x29, #640
    str x2, [x9]
    ; @src line=18 col=16
    sub x9, x29, #624
    ldr x1, [x9]
    sub x9, x29, #616
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
    ; @src line=18 col=16
    ; @src line=18 col=16
    sub x9, x29, #648
    ldr x1, [x9]
    sub x9, x29, #640
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=18 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #680
    str x1, [x9]
    sub x9, x29, #672
    str x2, [x9]
    ; @src line=18 col=24
    sub x9, x29, #664
    ldr x1, [x9]
    sub x9, x29, #656
    ldr x2, [x9]
    sub x9, x29, #680
    ldr x3, [x9]
    sub x9, x29, #672
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #696
    str x1, [x9]
    sub x9, x29, #688
    str x2, [x9]
    ; @src line=18 col=24
    ; @src line=18 col=1
    sub x9, x29, #696
    ldr x1, [x9]
    sub x9, x29, #688
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=18 col=1
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=24
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
    sub x9, x29, #704
    str x0, [x9]
    ; @src line=21 col=25
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #712
    str x0, [x9]
    ; @src line=21 col=26
    mov x0, #1
    mov x21, x0
    ; @src line=21 col=26
    mov x1, x21
    sub x9, x29, #712
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #712
    str x0, [x9]
    ; @src line=21 col=29
    mov x0, #2
    mov x21, x0
    ; @src line=21 col=29
    mov x1, x21
    sub x9, x29, #712
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #712
    str x0, [x9]
    ; @src line=21 col=25
    sub x9, x29, #712
    ldr x1, [x9]
    sub x9, x29, #704
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #704
    str x0, [x9]
    ; @src line=21 col=25
    sub x9, x29, #712
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=21 col=33
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    sub x9, x29, #736
    str x0, [x9]
    ; @src line=21 col=34
    mov x0, #3
    mov x21, x0
    ; @src line=21 col=34
    mov x1, x21
    sub x9, x29, #736
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #736
    str x0, [x9]
    ; @src line=21 col=37
    mov x0, #4
    mov x21, x0
    ; @src line=21 col=37
    mov x1, x21
    sub x9, x29, #736
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_int
    sub x9, x29, #736
    str x0, [x9]
    ; @src line=21 col=33
    sub x9, x29, #736
    ldr x1, [x9]
    sub x9, x29, #704
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #704
    str x0, [x9]
    ; @src line=21 col=33
    sub x9, x29, #736
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=21 col=1
    sub x9, x29, #704
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #760
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #760
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1313
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #704
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1313
    ldr x0, [x9]
    sub x9, x29, #768
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #768
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_18
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_18
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_19
_eir_main_array_get_null_18:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_19:
    sub x9, x29, #784
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #784
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #792
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #792
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1321
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #784
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1321
    ldr x0, [x9]
    sub x9, x29, #800
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #800
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_20
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_20
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    mov x1, #0
    b _eir_main_array_get_done_21
_eir_main_array_get_null_20:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, #8
_eir_main_array_get_done_21:
    sub x9, x29, #824
    str x0, [x9]
    sub x9, x29, #816
    str x1, [x9]
    ; @src line=21 col=1
    sub x9, x29, #824
    ldr x0, [x9]
    sub x9, x29, #816
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1337
    str x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1329
    str x1, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1321
    ldr x0, [x9]
    sub x9, x29, #832
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #832
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_22
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_22
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    mov x1, #0
    b _eir_main_array_get_done_23
_eir_main_array_get_null_22:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, #8
_eir_main_array_get_done_23:
    sub x9, x29, #856
    str x0, [x9]
    sub x9, x29, #848
    str x1, [x9]
    ; @src line=21 col=1
    sub x9, x29, #856
    ldr x0, [x9]
    sub x9, x29, #848
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1353
    str x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1345
    str x1, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1313
    ldr x0, [x9]
    sub x9, x29, #864
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #864
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_24
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_24
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_25
_eir_main_array_get_null_24:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_25:
    sub x9, x29, #880
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #880
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #888
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #888
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1361
    str x0, [x9]
    ; @src line=21 col=1
    sub x9, x29, #880
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1361
    ldr x0, [x9]
    sub x9, x29, #896
    str x0, [x9]
    mov x0, #0
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #896
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_26
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_26
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    mov x1, #0
    b _eir_main_array_get_done_27
_eir_main_array_get_null_26:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, #8
_eir_main_array_get_done_27:
    sub x9, x29, #920
    str x0, [x9]
    sub x9, x29, #912
    str x1, [x9]
    ; @src line=21 col=1
    sub x9, x29, #920
    ldr x0, [x9]
    sub x9, x29, #912
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1377
    str x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1369
    str x1, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1361
    ldr x0, [x9]
    sub x9, x29, #928
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=21 col=1
    mov x0, x21
    sub x9, x29, #928
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_28
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_28
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    mov x1, #0
    b _eir_main_array_get_done_29
_eir_main_array_get_null_28:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, #8
_eir_main_array_get_done_29:
    sub x9, x29, #952
    str x0, [x9]
    sub x9, x29, #944
    str x1, [x9]
    ; @src line=21 col=1
    sub x9, x29, #952
    ldr x0, [x9]
    sub x9, x29, #944
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1393
    str x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1385
    str x1, [x9]
    ; @src line=22 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=6
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1337
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1329
    ldr x1, [x9]
    sub x9, x29, #968
    str x0, [x9]
    sub x9, x29, #960
    str x1, [x9]
    ; @src line=22 col=9
    sub x9, x29, #968
    ldr x0, [x9]
    sub x9, x29, #960
    ldr x1, [x9]
    cmp x1, #8
    b.eq _eir_main_tagged_to_str_null_30
    bl __rt_itoa
    b _eir_main_tagged_to_str_done_31
_eir_main_tagged_to_str_null_30:
    mov x2, #0
_eir_main_tagged_to_str_done_31:
    sub x9, x29, #984
    str x1, [x9]
    sub x9, x29, #976
    str x2, [x9]
    ; @src line=22 col=11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1353
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1345
    ldr x1, [x9]
    sub x9, x29, #1000
    str x0, [x9]
    sub x9, x29, #992
    str x1, [x9]
    ; @src line=22 col=9
    sub x9, x29, #1000
    ldr x0, [x9]
    sub x9, x29, #992
    ldr x1, [x9]
    cmp x1, #8
    b.eq _eir_main_tagged_to_str_null_32
    bl __rt_itoa
    b _eir_main_tagged_to_str_done_33
_eir_main_tagged_to_str_null_32:
    mov x2, #0
_eir_main_tagged_to_str_done_33:
    sub x9, x29, #1016
    str x1, [x9]
    sub x9, x29, #1008
    str x2, [x9]
    ; @src line=22 col=9
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
    ; @src line=22 col=9
    ; @src line=22 col=9
    ; @src line=22 col=16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1377
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1369
    ldr x1, [x9]
    sub x9, x29, #1048
    str x0, [x9]
    sub x9, x29, #1040
    str x1, [x9]
    ; @src line=22 col=14
    sub x9, x29, #1048
    ldr x0, [x9]
    sub x9, x29, #1040
    ldr x1, [x9]
    cmp x1, #8
    b.eq _eir_main_tagged_to_str_null_34
    bl __rt_itoa
    b _eir_main_tagged_to_str_done_35
_eir_main_tagged_to_str_null_34:
    mov x2, #0
_eir_main_tagged_to_str_done_35:
    sub x9, x29, #1064
    str x1, [x9]
    sub x9, x29, #1056
    str x2, [x9]
    ; @src line=22 col=14
    sub x9, x29, #1032
    ldr x1, [x9]
    sub x9, x29, #1024
    ldr x2, [x9]
    sub x9, x29, #1064
    ldr x3, [x9]
    sub x9, x29, #1056
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1080
    str x1, [x9]
    sub x9, x29, #1072
    str x2, [x9]
    ; @src line=22 col=14
    ; @src line=22 col=14
    ; @src line=22 col=21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1393
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1385
    ldr x1, [x9]
    sub x9, x29, #1096
    str x0, [x9]
    sub x9, x29, #1088
    str x1, [x9]
    ; @src line=22 col=19
    sub x9, x29, #1096
    ldr x0, [x9]
    sub x9, x29, #1088
    ldr x1, [x9]
    cmp x1, #8
    b.eq _eir_main_tagged_to_str_null_36
    bl __rt_itoa
    b _eir_main_tagged_to_str_done_37
_eir_main_tagged_to_str_null_36:
    mov x2, #0
_eir_main_tagged_to_str_done_37:
    sub x9, x29, #1112
    str x1, [x9]
    sub x9, x29, #1104
    str x2, [x9]
    ; @src line=22 col=19
    sub x9, x29, #1080
    ldr x1, [x9]
    sub x9, x29, #1072
    ldr x2, [x9]
    sub x9, x29, #1112
    ldr x3, [x9]
    sub x9, x29, #1104
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1128
    str x1, [x9]
    sub x9, x29, #1120
    str x2, [x9]
    ; @src line=22 col=19
    ; @src line=22 col=19
    ; @src line=22 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #1144
    str x1, [x9]
    sub x9, x29, #1136
    str x2, [x9]
    ; @src line=22 col=24
    sub x9, x29, #1128
    ldr x1, [x9]
    sub x9, x29, #1120
    ldr x2, [x9]
    sub x9, x29, #1144
    ldr x3, [x9]
    sub x9, x29, #1136
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1160
    str x1, [x9]
    sub x9, x29, #1152
    str x2, [x9]
    ; @src line=22 col=24
    ; @src line=22 col=1
    sub x9, x29, #1160
    ldr x1, [x9]
    sub x9, x29, #1152
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=22 col=1
    ; @src line=25 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=10
    mov x0, #16
    mov x1, #4
    bl __rt_hash_new
    sub x9, x29, #1168
    str x0, [x9]
    ; @src line=25 col=11
    adrp x1, _str_30@PAGE
    add x1, x1, _str_30@PAGEOFF
    mov x2, #5
    sub x9, x29, #1184
    str x1, [x9]
    sub x9, x29, #1176
    str x2, [x9]
    ; @src line=25 col=22
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #1192
    str x0, [x9]
    ; @src line=25 col=23
    adrp x1, _str_31@PAGE
    add x1, x1, _str_31@PAGEOFF
    mov x2, #5
    sub x9, x29, #1208
    str x1, [x9]
    sub x9, x29, #1200
    str x2, [x9]
    ; @src line=25 col=23
    sub x9, x29, #1208
    ldr x1, [x9]
    sub x9, x29, #1200
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #1192
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #1192
    str x0, [x9]
    ; @src line=25 col=32
    mov x0, #30
    mov x21, x0
    ; @src line=25 col=32
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #1192
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #1192
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1184
    ldr x1, [x9]
    sub x9, x29, #1176
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1192
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1168
    ldr x0, [x9]
    mov x5, #4
    bl __rt_hash_set
    sub x9, x29, #1168
    str x0, [x9]
    ; @src line=25 col=37
    adrp x1, _str_32@PAGE
    add x1, x1, _str_32@PAGEOFF
    mov x2, #3
    sub x9, x29, #1232
    str x1, [x9]
    sub x9, x29, #1224
    str x2, [x9]
    ; @src line=25 col=46
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #1240
    str x0, [x9]
    ; @src line=25 col=47
    adrp x1, _str_33@PAGE
    add x1, x1, _str_33@PAGEOFF
    mov x2, #3
    sub x9, x29, #1256
    str x1, [x9]
    sub x9, x29, #1248
    str x2, [x9]
    ; @src line=25 col=47
    sub x9, x29, #1256
    ldr x1, [x9]
    sub x9, x29, #1248
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #1240
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #1240
    str x0, [x9]
    ; @src line=25 col=54
    mov x0, #25
    mov x21, x0
    ; @src line=25 col=54
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #1240
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #1240
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1232
    ldr x1, [x9]
    sub x9, x29, #1224
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1240
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1168
    ldr x0, [x9]
    mov x5, #4
    bl __rt_hash_set
    sub x9, x29, #1168
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1168
    ldr x0, [x9]
    sub x9, x29, #1328
    str x0, [x9]
    mov x0, #0
    sub x9, x29, #1320
    str x0, [x9]
    ; @src line=25 col=10
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=25 col=10
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #1344
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1344
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1352
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1352
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1344
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=25 col=10
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #1368
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1368
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1376
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1376
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1368
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_1
_eir_main_foreach_next_1:
    ; @src line=25 col=10
    sub x9, x29, #1328
    ldr x0, [x9]
    sub x9, x29, #1320
    ldr x1, [x9]
    bl __rt_hash_iter_next
    cmn x0, #1
    sub x9, x29, #1320
    str x0, [x9]
    sub x9, x29, #1312
    str x1, [x9]
    sub x9, x29, #1304
    str x2, [x9]
    sub x9, x29, #1296
    str x3, [x9]
    sub x9, x29, #1288
    str x4, [x9]
    sub x9, x29, #1280
    str x5, [x9]
    sub x9, x29, #1272
    str x6, [x9]
    cset x0, ne
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_2
    b _eir_main_foreach_exit_3
_eir_main_foreach_body_2:
    ; @src line=25 col=10
    sub x9, x29, #1312
    ldr x1, [x9]
    sub x9, x29, #1304
    ldr x2, [x9]
    cmn x2, #1
    b.ne _eir_main_iter_hash_key_string_38
    mov x0, #0
    mov x2, xzr
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_key_done_39
_eir_main_iter_hash_key_string_38:
    mov x0, #1
    bl __rt_mixed_from_value
_eir_main_iter_hash_key_done_39:
    sub x9, x29, #1392
    str x0, [x9]
    ; @src line=25 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    ldr x0, [x9]
    sub x9, x29, #1400
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1400
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10
    sub x9, x29, #1392
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1408
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1408
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1392
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10
    sub x9, x29, #1280
    ldr x5, [x9]
    sub x9, x29, #1296
    ldr x3, [x9]
    sub x9, x29, #1288
    ldr x4, [x9]
    mov x1, x3
    mov x3, x5
    bl __rt_deref_if_reference
    mov x5, x3
    mov x3, x1
    cmp x5, #7
    b.eq _eir_main_iter_hash_value_inspect_box_40
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_value_boxed_41
_eir_main_iter_hash_value_inspect_box_40:
    str x3, [sp, #-16]!
    mov x0, x3
    bl __rt_heap_kind
    cmp x0, #5
    b.eq _eir_main_iter_hash_value_reuse_box_42
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
    b _eir_main_iter_hash_value_tagged_done_43
_eir_main_iter_hash_value_reuse_box_42:
    ldr x0, [sp], #16
    bl __rt_incref
_eir_main_iter_hash_value_tagged_done_43:
_eir_main_iter_hash_value_boxed_41:
    sub x9, x29, #1416
    str x0, [x9]
    ; @src line=25 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    ldr x0, [x9]
    sub x9, x29, #1424
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1424
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=10
    sub x9, x29, #1416
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1432
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1432
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    str x0, [x9]
    ; @src line=25 col=10
    sub x9, x29, #1416
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    ldr x0, [x9]
    sub x9, x29, #1440
    str x0, [x9]
    ; @src line=25 col=1
    mov x0, #0
    mov x21, x0
    ; @src line=25 col=1
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #1440
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #1456
    str x0, [x9]
    ; @src line=25 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1417
    ldr x0, [x9]
    sub x9, x29, #1464
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1464
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=1
    sub x9, x29, #1456
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1472
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1472
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1417
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1456
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=1
    mov x0, #1
    mov x21, x0
    ; @src line=25 col=1
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #1440
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #1488
    str x0, [x9]
    ; @src line=25 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1425
    ldr x0, [x9]
    sub x9, x29, #1496
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1496
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=25 col=1
    sub x9, x29, #1488
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1504
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1504
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1425
    str x0, [x9]
    ; @src line=25 col=1
    sub x9, x29, #1488
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=26 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    ldr x0, [x9]
    sub x9, x29, #1512
    str x0, [x9]
    ; @src line=26 col=15
    sub x9, x29, #1512
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #1528
    str x1, [x9]
    sub x9, x29, #1520
    str x2, [x9]
    ; @src line=26 col=17
    adrp x1, _str_34@PAGE
    add x1, x1, _str_34@PAGEOFF
    mov x2, #1
    sub x9, x29, #1544
    str x1, [x9]
    sub x9, x29, #1536
    str x2, [x9]
    ; @src line=26 col=15
    sub x9, x29, #1528
    ldr x1, [x9]
    sub x9, x29, #1520
    ldr x2, [x9]
    sub x9, x29, #1544
    ldr x3, [x9]
    sub x9, x29, #1536
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1560
    str x1, [x9]
    sub x9, x29, #1552
    str x2, [x9]
    ; @src line=26 col=15
    sub x9, x29, #1528
    ldr x1, [x9]
    sub x9, x29, #1520
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1417
    ldr x0, [x9]
    sub x9, x29, #1568
    str x0, [x9]
    ; @src line=26 col=21
    sub x9, x29, #1568
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #1584
    str x1, [x9]
    sub x9, x29, #1576
    str x2, [x9]
    ; @src line=26 col=21
    sub x9, x29, #1560
    ldr x1, [x9]
    sub x9, x29, #1552
    ldr x2, [x9]
    sub x9, x29, #1584
    ldr x3, [x9]
    sub x9, x29, #1576
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1600
    str x1, [x9]
    sub x9, x29, #1592
    str x2, [x9]
    ; @src line=26 col=21
    ; @src line=26 col=21
    sub x9, x29, #1584
    ldr x1, [x9]
    sub x9, x29, #1576
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=30
    adrp x1, _str_35@PAGE
    add x1, x1, _str_35@PAGEOFF
    mov x2, #1
    sub x9, x29, #1616
    str x1, [x9]
    sub x9, x29, #1608
    str x2, [x9]
    ; @src line=26 col=28
    sub x9, x29, #1600
    ldr x1, [x9]
    sub x9, x29, #1592
    ldr x2, [x9]
    sub x9, x29, #1616
    ldr x3, [x9]
    sub x9, x29, #1608
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1632
    str x1, [x9]
    sub x9, x29, #1624
    str x2, [x9]
    ; @src line=26 col=28
    ; @src line=26 col=36
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1425
    ldr x0, [x9]
    sub x9, x29, #1640
    str x0, [x9]
    ; @src line=26 col=34
    sub x9, x29, #1640
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #1656
    str x1, [x9]
    sub x9, x29, #1648
    str x2, [x9]
    ; @src line=26 col=34
    sub x9, x29, #1632
    ldr x1, [x9]
    sub x9, x29, #1624
    ldr x2, [x9]
    sub x9, x29, #1656
    ldr x3, [x9]
    sub x9, x29, #1648
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1672
    str x1, [x9]
    sub x9, x29, #1664
    str x2, [x9]
    ; @src line=26 col=34
    ; @src line=26 col=34
    sub x9, x29, #1656
    ldr x1, [x9]
    sub x9, x29, #1648
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=43
    adrp x1, _str_36@PAGE
    add x1, x1, _str_36@PAGEOFF
    mov x2, #2
    sub x9, x29, #1688
    str x1, [x9]
    sub x9, x29, #1680
    str x2, [x9]
    ; @src line=26 col=41
    sub x9, x29, #1672
    ldr x1, [x9]
    sub x9, x29, #1664
    ldr x2, [x9]
    sub x9, x29, #1688
    ldr x3, [x9]
    sub x9, x29, #1680
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1704
    str x1, [x9]
    sub x9, x29, #1696
    str x2, [x9]
    ; @src line=26 col=41
    ; @src line=26 col=5
    sub x9, x29, #1704
    ldr x1, [x9]
    sub x9, x29, #1696
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=26 col=5
    b _eir_main_foreach_next_1
_eir_main_foreach_exit_3:
    ; @src line=25 col=10
    sub x9, x29, #1168
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=10
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #5
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #1712
    str x0, [x9]
    ; @src line=30 col=11
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #1720
    str x0, [x9]
    ; @src line=30 col=12
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #1736
    str x1, [x9]
    sub x9, x29, #1728
    str x2, [x9]
    ; @src line=30 col=20
    mov x0, #1
    mov x21, x0
    ; @src line=30 col=11
    sub x9, x29, #1736
    ldr x1, [x9]
    sub x9, x29, #1728
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1720
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1720
    str x0, [x9]
    ; @src line=30 col=23
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #4
    sub x9, x29, #1760
    str x1, [x9]
    sub x9, x29, #1752
    str x2, [x9]
    ; @src line=30 col=33
    adrp x1, _str_31@PAGE
    add x1, x1, _str_31@PAGEOFF
    mov x2, #5
    sub x9, x29, #1776
    str x1, [x9]
    sub x9, x29, #1768
    str x2, [x9]
    ; @src line=30 col=11
    sub x9, x29, #1760
    ldr x1, [x9]
    sub x9, x29, #1752
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1776
    ldr x1, [x9]
    sub x9, x29, #1768
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1720
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1720
    str x0, [x9]
    ; @src line=30 col=11
    sub x9, x29, #1720
    ldr x1, [x9]
    sub x9, x29, #1712
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #1712
    str x0, [x9]
    ; @src line=30 col=11
    sub x9, x29, #1720
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=30 col=43
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    sub x9, x29, #1784
    str x0, [x9]
    ; @src line=30 col=44
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #1800
    str x1, [x9]
    sub x9, x29, #1792
    str x2, [x9]
    ; @src line=30 col=52
    mov x0, #2
    mov x21, x0
    ; @src line=30 col=43
    sub x9, x29, #1800
    ldr x1, [x9]
    sub x9, x29, #1792
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    mov x3, x21
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #1784
    ldr x0, [x9]
    mov x5, #0
    bl __rt_hash_set
    sub x9, x29, #1784
    str x0, [x9]
    ; @src line=30 col=55
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #4
    sub x9, x29, #1824
    str x1, [x9]
    sub x9, x29, #1816
    str x2, [x9]
    ; @src line=30 col=65
    adrp x1, _str_33@PAGE
    add x1, x1, _str_33@PAGEOFF
    mov x2, #3
    sub x9, x29, #1840
    str x1, [x9]
    sub x9, x29, #1832
    str x2, [x9]
    ; @src line=30 col=43
    sub x9, x29, #1824
    ldr x1, [x9]
    sub x9, x29, #1816
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #1840
    ldr x1, [x9]
    sub x9, x29, #1832
    ldr x2, [x9]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    sub x9, x29, #1784
    ldr x0, [x9]
    mov x5, #1
    bl __rt_hash_set
    sub x9, x29, #1784
    str x0, [x9]
    ; @src line=30 col=43
    sub x9, x29, #1784
    ldr x1, [x9]
    sub x9, x29, #1712
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #1712
    str x0, [x9]
    ; @src line=30 col=43
    sub x9, x29, #1784
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=30 col=10
    sub x9, x29, #1712
    ldr x0, [x9]
    sub x9, x29, #1904
    str x0, [x9]
    mov x0, #-1
    sub x9, x29, #1896
    str x0, [x9]
    ; @src line=30 col=10
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=30 col=10
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #1920
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1920
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1928
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1928
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1920
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_4
_eir_main_foreach_next_4:
    ; @src line=30 col=10
    sub x9, x29, #1904
    ldr x12, [x9]
    sub x9, x29, #1896
    ldr x10, [x9]
    add x10, x10, #1
    ldr x11, [x12]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_44
    sub x9, x29, #1896
    str x10, [x9]
_eir_main_iter_next_done_44:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_5
    b _eir_main_foreach_exit_6
_eir_main_foreach_body_5:
    ; @src line=30 col=10
    sub x9, x29, #1904
    ldr x12, [x9]
    sub x9, x29, #1896
    ldr x10, [x9]
    add x12, x12, #24
    ldr x0, [x12, x10, lsl #3]
    mov x1, x0
    mov x2, xzr
    mov x0, #5
    bl __rt_mixed_from_value
    sub x9, x29, #1944
    str x0, [x9]
    ; @src line=30 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    ldr x0, [x9]
    sub x9, x29, #1952
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1952
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=10
    sub x9, x29, #1944
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1960
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1960
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    str x0, [x9]
    ; @src line=30 col=10
    sub x9, x29, #1944
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    ldr x0, [x9]
    sub x9, x29, #1968
    str x0, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    ldr x0, [x9]
    sub x9, x29, #1976
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #1976
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1
    sub x9, x29, #1968
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #1984
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #1984
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    str x0, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    ldr x0, [x9]
    sub x9, x29, #1992
    str x0, [x9]
    ; @src line=30 col=77
    adrp x1, _str_28@PAGE
    add x1, x1, _str_28@PAGEOFF
    mov x2, #2
    sub x9, x29, #2008
    str x1, [x9]
    sub x9, x29, #2000
    str x2, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2008
    ldr x1, [x9]
    sub x9, x29, #2000
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #1992
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #2016
    str x0, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1449
    ldr x0, [x9]
    sub x9, x29, #2024
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2024
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1
    sub x9, x29, #2016
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2032
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2032
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1449
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2016
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    ldr x0, [x9]
    sub x9, x29, #2040
    str x0, [x9]
    ; @src line=30 col=93
    adrp x1, _str_26@PAGE
    add x1, x1, _str_26@PAGEOFF
    mov x2, #4
    sub x9, x29, #2056
    str x1, [x9]
    sub x9, x29, #2048
    str x2, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2056
    ldr x1, [x9]
    sub x9, x29, #2048
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #2040
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #2064
    str x0, [x9]
    ; @src line=30 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1457
    ldr x0, [x9]
    sub x9, x29, #2072
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2072
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=30 col=1
    sub x9, x29, #2064
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2080
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2080
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1457
    str x0, [x9]
    ; @src line=30 col=1
    sub x9, x29, #2064
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=31 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=31 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1449
    ldr x0, [x9]
    sub x9, x29, #2088
    str x0, [x9]
    ; @src line=31 col=17
    sub x9, x29, #2088
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #2104
    str x1, [x9]
    sub x9, x29, #2096
    str x2, [x9]
    ; @src line=31 col=19
    adrp x1, _str_29@PAGE
    add x1, x1, _str_29@PAGEOFF
    mov x2, #1
    sub x9, x29, #2120
    str x1, [x9]
    sub x9, x29, #2112
    str x2, [x9]
    ; @src line=31 col=17
    sub x9, x29, #2104
    ldr x1, [x9]
    sub x9, x29, #2096
    ldr x2, [x9]
    sub x9, x29, #2120
    ldr x3, [x9]
    sub x9, x29, #2112
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2136
    str x1, [x9]
    sub x9, x29, #2128
    str x2, [x9]
    ; @src line=31 col=17
    sub x9, x29, #2104
    ldr x1, [x9]
    sub x9, x29, #2096
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=31 col=25
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1457
    ldr x0, [x9]
    sub x9, x29, #2144
    str x0, [x9]
    ; @src line=31 col=23
    sub x9, x29, #2144
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #2160
    str x1, [x9]
    sub x9, x29, #2152
    str x2, [x9]
    ; @src line=31 col=23
    sub x9, x29, #2136
    ldr x1, [x9]
    sub x9, x29, #2128
    ldr x2, [x9]
    sub x9, x29, #2160
    ldr x3, [x9]
    sub x9, x29, #2152
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2176
    str x1, [x9]
    sub x9, x29, #2168
    str x2, [x9]
    ; @src line=31 col=23
    ; @src line=31 col=23
    sub x9, x29, #2160
    ldr x1, [x9]
    sub x9, x29, #2152
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=31 col=36
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #2192
    str x1, [x9]
    sub x9, x29, #2184
    str x2, [x9]
    ; @src line=31 col=34
    sub x9, x29, #2176
    ldr x1, [x9]
    sub x9, x29, #2168
    ldr x2, [x9]
    sub x9, x29, #2192
    ldr x3, [x9]
    sub x9, x29, #2184
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2208
    str x1, [x9]
    sub x9, x29, #2200
    str x2, [x9]
    ; @src line=31 col=34
    ; @src line=31 col=5
    sub x9, x29, #2208
    ldr x1, [x9]
    sub x9, x29, #2200
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=31 col=5
    b _eir_main_foreach_next_4
_eir_main_foreach_exit_6:
    ; @src line=30 col=10
    sub x9, x29, #1712
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=37 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=37 col=9
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
    sub x9, x29, #2216
    str x0, [x9]
    ; @src line=37 col=10
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #2224
    str x0, [x9]
    ; @src line=37 col=11
    mov x0, #1
    mov x21, x0
    ; @src line=37 col=11
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2224
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2224
    str x0, [x9]
    ; @src line=37 col=14
    adrp x1, _str_37@PAGE
    add x1, x1, _str_37@PAGEOFF
    mov x2, #3
    sub x9, x29, #2248
    str x1, [x9]
    sub x9, x29, #2240
    str x2, [x9]
    ; @src line=37 col=14
    sub x9, x29, #2248
    ldr x1, [x9]
    sub x9, x29, #2240
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2224
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2224
    str x0, [x9]
    ; @src line=37 col=10
    sub x9, x29, #2224
    ldr x1, [x9]
    sub x9, x29, #2216
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #2216
    str x0, [x9]
    ; @src line=37 col=10
    sub x9, x29, #2224
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=37 col=22
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #2256
    str x0, [x9]
    ; @src line=37 col=23
    mov x0, #2
    mov x21, x0
    ; @src line=37 col=23
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2256
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2256
    str x0, [x9]
    ; @src line=37 col=26
    adrp x1, _str_38@PAGE
    add x1, x1, _str_38@PAGEOFF
    mov x2, #3
    sub x9, x29, #2280
    str x1, [x9]
    sub x9, x29, #2272
    str x2, [x9]
    ; @src line=37 col=26
    sub x9, x29, #2280
    ldr x1, [x9]
    sub x9, x29, #2272
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2256
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2256
    str x0, [x9]
    ; @src line=37 col=22
    sub x9, x29, #2256
    ldr x1, [x9]
    sub x9, x29, #2216
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #2216
    str x0, [x9]
    ; @src line=37 col=22
    sub x9, x29, #2256
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=37 col=34
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #2288
    str x0, [x9]
    ; @src line=37 col=35
    mov x0, #3
    mov x21, x0
    ; @src line=37 col=35
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2288
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2288
    str x0, [x9]
    ; @src line=37 col=38
    adrp x1, _str_39@PAGE
    add x1, x1, _str_39@PAGEOFF
    mov x2, #5
    sub x9, x29, #2312
    str x1, [x9]
    sub x9, x29, #2304
    str x2, [x9]
    ; @src line=37 col=38
    sub x9, x29, #2312
    ldr x1, [x9]
    sub x9, x29, #2304
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2288
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2288
    str x0, [x9]
    ; @src line=37 col=34
    sub x9, x29, #2288
    ldr x1, [x9]
    sub x9, x29, #2216
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #2216
    str x0, [x9]
    ; @src line=37 col=34
    sub x9, x29, #2288
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=37 col=1
    sub x9, x29, #2216
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2320
    str x0, [x9]
    ; @src line=37 col=1
    sub x9, x29, #2320
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1465
    str x0, [x9]
    ; @src line=37 col=1
    sub x9, x29, #2216
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=38 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=38 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1465
    ldr x0, [x9]
    sub x9, x29, #2328
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2328
    ldr x0, [x9]
    sub x9, x29, #2392
    str x0, [x9]
    mov x0, #-1
    sub x9, x29, #2384
    str x0, [x9]
    ; @src line=38 col=10
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=38 col=10
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #2408
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2408
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2416
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2416
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2408
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_7
_eir_main_foreach_next_7:
    ; @src line=38 col=10
    sub x9, x29, #2392
    ldr x12, [x9]
    sub x9, x29, #2384
    ldr x10, [x9]
    add x10, x10, #1
    ldr x11, [x12]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_45
    sub x9, x29, #2384
    str x10, [x9]
_eir_main_iter_next_done_45:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_8
    b _eir_main_foreach_exit_9
_eir_main_foreach_body_8:
    ; @src line=38 col=10
    sub x9, x29, #2392
    ldr x12, [x9]
    sub x9, x29, #2384
    ldr x10, [x9]
    add x12, x12, #24
    ldr x0, [x12, x10, lsl #3]
    bl __rt_mixed_from_array_kind
    sub x9, x29, #2432
    str x0, [x9]
    ; @src line=38 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    ldr x0, [x9]
    sub x9, x29, #2440
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2440
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=38 col=10
    sub x9, x29, #2432
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2448
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2448
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    str x0, [x9]
    ; @src line=38 col=10
    sub x9, x29, #2432
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=39 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=39 col=26
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    ldr x0, [x9]
    sub x9, x29, #2456
    str x0, [x9]
    ; @src line=39 col=24
    mov x0, #0
    mov x21, x0
    ; @src line=39 col=24
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #2456
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #2472
    str x0, [x9]
    ; @src line=39 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1481
    ldr x0, [x9]
    sub x9, x29, #2480
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2480
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=39 col=24
    sub x9, x29, #2472
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2488
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2488
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1481
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2472
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=39 col=24
    mov x0, #1
    mov x21, x0
    ; @src line=39 col=24
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #2456
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #2504
    str x0, [x9]
    ; @src line=39 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1489
    ldr x0, [x9]
    sub x9, x29, #2512
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2512
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=39 col=24
    sub x9, x29, #2504
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2520
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2520
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1489
    str x0, [x9]
    ; @src line=39 col=24
    sub x9, x29, #2504
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=39 col=24
    sub x9, x29, #2456
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_if_then_11
    b _eir_main_foreach_next_7
_eir_main_foreach_exit_9:
    ; @src line=47 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=47 col=11
    mov x0, #16
    mov x1, #4
    bl __rt_hash_new
    sub x9, x29, #2696
    str x0, [x9]
    ; @src line=47 col=12
    adrp x1, _str_40@PAGE
    add x1, x1, _str_40@PAGEOFF
    mov x2, #4
    sub x9, x29, #2712
    str x1, [x9]
    sub x9, x29, #2704
    str x2, [x9]
    ; @src line=47 col=22
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #2720
    str x0, [x9]
    ; @src line=47 col=23
    adrp x1, _str_41@PAGE
    add x1, x1, _str_41@PAGEOFF
    mov x2, #8
    sub x9, x29, #2736
    str x1, [x9]
    sub x9, x29, #2728
    str x2, [x9]
    ; @src line=47 col=23
    sub x9, x29, #2736
    ldr x1, [x9]
    sub x9, x29, #2728
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2720
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2720
    str x0, [x9]
    ; @src line=47 col=36
    adrp x1, _str_42@PAGE
    add x1, x1, _str_42@PAGEOFF
    mov x2, #7
    sub x9, x29, #2752
    str x1, [x9]
    sub x9, x29, #2744
    str x2, [x9]
    ; @src line=47 col=36
    sub x9, x29, #2752
    ldr x1, [x9]
    sub x9, x29, #2744
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2720
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2720
    str x0, [x9]
    ; @src line=47 col=47
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=47 col=47
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2720
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2720
    str x0, [x9]
    ; @src line=47 col=53
    adrp x1, _str_43@PAGE
    add x1, x1, _str_43@PAGEOFF
    mov x2, #8
    sub x9, x29, #2776
    str x1, [x9]
    sub x9, x29, #2768
    str x2, [x9]
    ; @src line=47 col=53
    sub x9, x29, #2776
    ldr x1, [x9]
    sub x9, x29, #2768
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #2720
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #2720
    str x0, [x9]
    ; @src line=47 col=11
    sub x9, x29, #2712
    ldr x1, [x9]
    sub x9, x29, #2704
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    stp x1, x2, [sp, #-16]!
    sub x9, x29, #2720
    ldr x0, [x9]
    mov x3, x0
    mov x4, xzr
    ldp x1, x2, [sp], #16
    sub x9, x29, #2696
    ldr x0, [x9]
    mov x5, #4
    bl __rt_hash_set
    sub x9, x29, #2696
    str x0, [x9]
    ; @src line=47 col=1
    sub x9, x29, #2696
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2784
    str x0, [x9]
    ; @src line=47 col=1
    sub x9, x29, #2784
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    str x0, [x9]
    ; @src line=47 col=1
    sub x9, x29, #2696
    ldr x0, [x9]
    bl __rt_decref_hash
    ; @src line=48 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=48 col=11
    adrp x1, _str_40@PAGE
    add x1, x1, _str_40@PAGEOFF
    mov x2, #4
    sub x9, x29, #2800
    str x1, [x9]
    sub x9, x29, #2792
    str x2, [x9]
    ; @src line=48 col=1
    sub x9, x29, #2800
    ldr x1, [x9]
    sub x9, x29, #2792
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #2816
    str x1, [x9]
    sub x9, x29, #2808
    str x2, [x9]
    ; @src line=48 col=1
    sub x9, x29, #2816
    ldr x1, [x9]
    sub x9, x29, #2808
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1505
    str x2, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    sub x9, x29, #2824
    str x0, [x9]
    ; @src line=49 col=31
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1505
    ldr x2, [x9]
    sub x9, x29, #2840
    str x1, [x9]
    sub x9, x29, #2832
    str x2, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2840
    ldr x1, [x9]
    sub x9, x29, #2832
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #2824
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_isset_hash_missing_46
    cmp x3, #8
    b.eq _eir_main_isset_hash_missing_46
    mov x0, #0
    b _eir_main_isset_hash_done_47
_eir_main_isset_hash_missing_46:
    mov x0, #1
_eir_main_isset_hash_done_47:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_16
    b _eir_main_isset_lazy_false_14
_eir_main_if_merge_10:
    udf #0
_eir_main_if_then_11:
    ; @src line=40 col=9
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=40 col=14
    adrp x1, _str_44@PAGE
    add x1, x1, _str_44@PAGEOFF
    mov x2, #4
    sub x9, x29, #2544
    str x1, [x9]
    sub x9, x29, #2536
    str x2, [x9]
    ; @src line=40 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1481
    ldr x0, [x9]
    sub x9, x29, #2552
    str x0, [x9]
    ; @src line=40 col=21
    sub x9, x29, #2552
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #2568
    str x1, [x9]
    sub x9, x29, #2560
    str x2, [x9]
    ; @src line=40 col=21
    sub x9, x29, #2544
    ldr x1, [x9]
    sub x9, x29, #2536
    ldr x2, [x9]
    sub x9, x29, #2568
    ldr x3, [x9]
    sub x9, x29, #2560
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2584
    str x1, [x9]
    sub x9, x29, #2576
    str x2, [x9]
    ; @src line=40 col=21
    sub x9, x29, #2568
    ldr x1, [x9]
    sub x9, x29, #2560
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=40 col=30
    adrp x1, _str_45@PAGE
    add x1, x1, _str_45@PAGEOFF
    mov x2, #3
    sub x9, x29, #2600
    str x1, [x9]
    sub x9, x29, #2592
    str x2, [x9]
    ; @src line=40 col=28
    sub x9, x29, #2584
    ldr x1, [x9]
    sub x9, x29, #2576
    ldr x2, [x9]
    sub x9, x29, #2600
    ldr x3, [x9]
    sub x9, x29, #2592
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2616
    str x1, [x9]
    sub x9, x29, #2608
    str x2, [x9]
    ; @src line=40 col=28
    ; @src line=40 col=38
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1489
    ldr x0, [x9]
    sub x9, x29, #2624
    str x0, [x9]
    ; @src line=40 col=36
    sub x9, x29, #2624
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #2640
    str x1, [x9]
    sub x9, x29, #2632
    str x2, [x9]
    ; @src line=40 col=36
    sub x9, x29, #2616
    ldr x1, [x9]
    sub x9, x29, #2608
    ldr x2, [x9]
    sub x9, x29, #2640
    ldr x3, [x9]
    sub x9, x29, #2632
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2656
    str x1, [x9]
    sub x9, x29, #2648
    str x2, [x9]
    ; @src line=40 col=36
    ; @src line=40 col=36
    sub x9, x29, #2640
    ldr x1, [x9]
    sub x9, x29, #2632
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=40 col=47
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #2672
    str x1, [x9]
    sub x9, x29, #2664
    str x2, [x9]
    ; @src line=40 col=45
    sub x9, x29, #2656
    ldr x1, [x9]
    sub x9, x29, #2648
    ldr x2, [x9]
    sub x9, x29, #2672
    ldr x3, [x9]
    sub x9, x29, #2664
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #2688
    str x1, [x9]
    sub x9, x29, #2680
    str x2, [x9]
    ; @src line=40 col=45
    ; @src line=40 col=9
    sub x9, x29, #2688
    ldr x1, [x9]
    sub x9, x29, #2680
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=40 col=9
    b _eir_main_foreach_next_7
_eir_main_if_else_12:
    ; @src line=39 col=24
    udf #0
_eir_main_if_merge_13:

    ; epilogue + exit(0)
    ; epilogue cleanup $__elephc_list_13_1_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1265
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_48
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_48:
    ; epilogue cleanup $__elephc_list_17_1_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1289
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_49
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_49:
    ; epilogue cleanup $id
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1297
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_50
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_50:
    ; epilogue cleanup $name
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1305
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_51
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_51:
    ; epilogue cleanup $__elephc_list_21_1_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1313
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_52
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_52:
    ; epilogue cleanup $__elephc_list_21_1_1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1321
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_53
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_53:
    ; epilogue cleanup $__elephc_list_21_1_2
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1361
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_54
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_54:
    ; epilogue cleanup $key
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1401
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_55
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_55:
    ; epilogue cleanup $__elephc_foreach_destructure_25_1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1409
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_56
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_56:
    ; epilogue cleanup $who
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1417
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_57
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_57:
    ; epilogue cleanup $age
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1425
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_58
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_58:
    ; epilogue cleanup $__elephc_foreach_destructure_30_1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1433
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_59
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_59:
    ; epilogue cleanup $__elephc_list_30_1_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1441
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_60
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_60:
    ; epilogue cleanup $rowId
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1449
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_61
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_61:
    ; epilogue cleanup $rowName
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1457
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_62
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_62:
    ; epilogue cleanup $rows
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1465
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_63
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_63:
    ; epilogue cleanup $row
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1473
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_64
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_64:
    ; epilogue cleanup $num
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1481
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_65
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_65:
    ; epilogue cleanup $label
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1489
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_66
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_66:
    ; epilogue cleanup $scopes
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_67
    bl __rt_decref_hash
_eir_main_main_refcounted_cleanup_done_67:
    ; epilogue cleanup $lookup
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $__eir_tmp1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_68
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_68:
    ; epilogue cleanup $__elephc_destr_49_5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1537
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_69
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_69:
    ; epilogue cleanup $__elephc_list_49_5_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1545
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_70
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_70:
    ; epilogue cleanup $access
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_71
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_71:
    ; epilogue cleanup $__elephc_destr_yield_49_5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1561
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_72
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_72:
    ; epilogue cleanup $__eir_tmp3
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_73
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_73:
    ; epilogue cleanup $__elephc_destr_52_5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_74
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_74:
    ; epilogue cleanup $__elephc_list_52_5_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_75
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_75:
    ; epilogue cleanup $__elephc_destr_yield_52_5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1601
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_76
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_76:
    ; epilogue cleanup $queue
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_77
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_77:
    ; epilogue cleanup $__eir_tmp5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_78
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_78:
    ; epilogue cleanup $__elephc_destr_62_8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_79
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_79:
    ; epilogue cleanup $__elephc_list_62_8_0
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_80
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_80:
    ; epilogue cleanup $tag
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_81
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_81:
    ; epilogue cleanup $__elephc_destr_yield_62_8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_82
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_82:
    ; epilogue cleanup $__eir_tmp7
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_83
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_83:
    ; epilogue cleanup $__eir_tmp9
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_84
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_84:
    ; epilogue cleanup $__eir_tmp11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_85
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_85:
    ; epilogue cleanup $__eir_tmp13
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_86
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_86:
    ; restore callee-saved registers used by the register allocator
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1737
    ldr x21, [x9]
    mov x9, sp
    add x9, x9, #4080
    add x9, x9, #1760
    ldp x29, x30, [x9]
    add sp, sp, #4080
    add sp, sp, #1776
    mov x0, #0
    mov x16, #1
    svc #0x80
_eir_main_isset_lazy_false_14:
    ; @src line=49 col=40
    mov x0, #0
    mov x21, x0
    ; @src line=49 col=40
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1521
    str x0, [x9]
    b _eir_main_isset_lazy_merge_15
_eir_main_isset_lazy_merge_15:
    ; @src line=49 col=40
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1521
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_17
    b _eir_main_coalesce_absent_18
_eir_main_isset_lazy_true_16:
    ; @src line=49 col=40
    mov x0, #1
    mov x21, x0
    ; @src line=49 col=40
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1521
    str x0, [x9]
    b _eir_main_isset_lazy_merge_15
_eir_main_coalesce_present_17:
    ; @src line=49 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    sub x9, x29, #2880
    str x0, [x9]
    ; @src line=49 col=31
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1505
    ldr x2, [x9]
    sub x9, x29, #2896
    str x1, [x9]
    sub x9, x29, #2888
    str x2, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2896
    ldr x1, [x9]
    sub x9, x29, #2888
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #2880
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_87
    bl __rt_deref_if_reference
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_done_88
_eir_main_hash_get_miss_87:
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_hash_get_done_88:
    sub x9, x29, #2904
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2904
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    sub x9, x29, #2912
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2904
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=49 col=40
    sub x9, x29, #2912
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2920
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2920
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2912
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_19
_eir_main_coalesce_absent_18:
    ; @src line=49 col=43
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=49 col=40
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #2936
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2936
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2944
    str x0, [x9]
    ; @src line=49 col=40
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    ldr x0, [x9]
    sub x9, x29, #2952
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2952
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=49 col=40
    sub x9, x29, #2944
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    str x0, [x9]
    ; @src line=49 col=40
    sub x9, x29, #2936
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_19
_eir_main_coalesce_merge_19:
    ; @src line=49 col=40
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1529
    ldr x0, [x9]
    sub x9, x29, #2960
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #2960
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2968
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #2968
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1537
    str x0, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1537
    ldr x0, [x9]
    sub x9, x29, #2976
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #2976
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #2984
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #2984
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1545
    str x0, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1545
    ldr x0, [x9]
    sub x9, x29, #2992
    str x0, [x9]
    mov x0, #3
    mov x21, x0
    ; @src line=49 col=5
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #2992
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #3008
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3008
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3016
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3016
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3008
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=49 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1537
    ldr x0, [x9]
    sub x9, x29, #3024
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3024
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3032
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3032
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1561
    str x0, [x9]
    ; @src line=49 col=5
    sub x9, x29, #3032
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_if_then_20
    b _eir_main_if_else_21
_eir_main_if_then_20:
    ; @src line=50 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=50 col=10
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1513
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1505
    ldr x2, [x9]
    sub x9, x29, #3056
    str x1, [x9]
    sub x9, x29, #3048
    str x2, [x9]
    ; @src line=50 col=20
    adrp x1, _str_46@PAGE
    add x1, x1, _str_46@PAGEOFF
    mov x2, #10
    sub x9, x29, #3072
    str x1, [x9]
    sub x9, x29, #3064
    str x2, [x9]
    ; @src line=50 col=18
    sub x9, x29, #3056
    ldr x1, [x9]
    sub x9, x29, #3048
    ldr x2, [x9]
    sub x9, x29, #3072
    ldr x3, [x9]
    sub x9, x29, #3064
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #3088
    str x1, [x9]
    sub x9, x29, #3080
    str x2, [x9]
    ; @src line=50 col=35
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    ldr x0, [x9]
    sub x9, x29, #3096
    str x0, [x9]
    ; @src line=50 col=33
    sub x9, x29, #3096
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #3112
    str x1, [x9]
    sub x9, x29, #3104
    str x2, [x9]
    ; @src line=50 col=33
    sub x9, x29, #3088
    ldr x1, [x9]
    sub x9, x29, #3080
    ldr x2, [x9]
    sub x9, x29, #3112
    ldr x3, [x9]
    sub x9, x29, #3104
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #3128
    str x1, [x9]
    sub x9, x29, #3120
    str x2, [x9]
    ; @src line=50 col=33
    ; @src line=50 col=33
    sub x9, x29, #3112
    ldr x1, [x9]
    sub x9, x29, #3104
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=50 col=45
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #3144
    str x1, [x9]
    sub x9, x29, #3136
    str x2, [x9]
    ; @src line=50 col=43
    sub x9, x29, #3128
    ldr x1, [x9]
    sub x9, x29, #3120
    ldr x2, [x9]
    sub x9, x29, #3144
    ldr x3, [x9]
    sub x9, x29, #3136
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #3160
    str x1, [x9]
    sub x9, x29, #3152
    str x2, [x9]
    ; @src line=50 col=43
    ; @src line=50 col=5
    sub x9, x29, #3160
    ldr x1, [x9]
    sub x9, x29, #3152
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=50 col=5
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    sub x9, x29, #3168
    str x0, [x9]
    ; @src line=52 col=31
    adrp x1, _str_47@PAGE
    add x1, x1, _str_47@PAGEOFF
    mov x2, #7
    sub x9, x29, #3184
    str x1, [x9]
    sub x9, x29, #3176
    str x2, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3184
    ldr x1, [x9]
    sub x9, x29, #3176
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #3168
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_isset_hash_missing_89
    cmp x3, #8
    b.eq _eir_main_isset_hash_missing_89
    mov x0, #0
    b _eir_main_isset_hash_done_90
_eir_main_isset_hash_missing_89:
    mov x0, #1
_eir_main_isset_hash_done_90:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_25
    b _eir_main_isset_lazy_false_23
_eir_main_if_else_21:
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #153
    str x0, [x9]
    ; @src line=52 col=31
    adrp x1, _str_47@PAGE
    add x1, x1, _str_47@PAGEOFF
    mov x2, #7
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #169
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #161
    str x2, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #169
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #161
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #153
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_isset_hash_missing_91
    cmp x3, #8
    b.eq _eir_main_isset_hash_missing_91
    mov x0, #0
    b _eir_main_isset_hash_done_92
_eir_main_isset_hash_missing_91:
    mov x0, #1
_eir_main_isset_hash_done_92:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_52
    b _eir_main_isset_lazy_false_50
_eir_main_if_merge_22:
    udf #0
_eir_main_isset_lazy_false_23:
    ; @src line=52 col=42
    mov x0, #0
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1569
    str x0, [x9]
    b _eir_main_isset_lazy_merge_24
_eir_main_isset_lazy_merge_24:
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1569
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_26
    b _eir_main_coalesce_absent_27
_eir_main_isset_lazy_true_25:
    ; @src line=52 col=42
    mov x0, #1
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1569
    str x0, [x9]
    b _eir_main_isset_lazy_merge_24
_eir_main_coalesce_present_26:
    ; @src line=52 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    sub x9, x29, #3224
    str x0, [x9]
    ; @src line=52 col=31
    adrp x1, _str_47@PAGE
    add x1, x1, _str_47@PAGEOFF
    mov x2, #7
    sub x9, x29, #3240
    str x1, [x9]
    sub x9, x29, #3232
    str x2, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3240
    ldr x1, [x9]
    sub x9, x29, #3232
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    sub x9, x29, #3224
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_93
    bl __rt_deref_if_reference
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_done_94
_eir_main_hash_get_miss_93:
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_hash_get_done_94:
    sub x9, x29, #3248
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3248
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    sub x9, x29, #3256
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3248
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=52 col=42
    sub x9, x29, #3256
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3264
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3264
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3256
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_28
_eir_main_coalesce_absent_27:
    ; @src line=52 col=45
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #3280
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3280
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3288
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    ldr x0, [x9]
    sub x9, x29, #3296
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3296
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=42
    sub x9, x29, #3288
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    str x0, [x9]
    ; @src line=52 col=42
    sub x9, x29, #3280
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_28
_eir_main_coalesce_merge_28:
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1577
    ldr x0, [x9]
    sub x9, x29, #3304
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3304
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3312
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3312
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    ldr x0, [x9]
    sub x9, x29, #3320
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3320
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3328
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3328
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    ldr x0, [x9]
    sub x9, x29, #3336
    str x0, [x9]
    mov x0, #3
    mov x21, x0
    ; @src line=52 col=5
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #3336
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #3352
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    ldr x0, [x9]
    sub x9, x29, #3360
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3360
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=5
    sub x9, x29, #3352
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3368
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3368
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3352
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    ldr x0, [x9]
    sub x9, x29, #3376
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3376
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3384
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3384
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1601
    str x0, [x9]
    ; @src line=52 col=5
    sub x9, x29, #3384
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_if_then_29
    b _eir_main_if_else_30
_eir_main_if_then_29:
    ; @src line=53 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=53 col=24
    adrp x1, _str_48@PAGE
    add x1, x1, _str_48@PAGEOFF
    mov x2, #12
    sub x9, x29, #3408
    str x1, [x9]
    sub x9, x29, #3400
    str x2, [x9]
    ; @src line=53 col=5
    sub x9, x29, #3408
    ldr x1, [x9]
    sub x9, x29, #3400
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=60 col=10
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
    sub x9, x29, #3416
    str x0, [x9]
    ; @src line=60 col=11
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #3424
    str x0, [x9]
    ; @src line=60 col=12
    mov x0, #1
    mov x21, x0
    ; @src line=60 col=12
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3424
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3424
    str x0, [x9]
    ; @src line=60 col=15
    adrp x1, _str_49@PAGE
    add x1, x1, _str_49@PAGEOFF
    mov x2, #5
    sub x9, x29, #3448
    str x1, [x9]
    sub x9, x29, #3440
    str x2, [x9]
    ; @src line=60 col=15
    sub x9, x29, #3448
    ldr x1, [x9]
    sub x9, x29, #3440
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3424
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3424
    str x0, [x9]
    ; @src line=60 col=11
    sub x9, x29, #3424
    ldr x1, [x9]
    sub x9, x29, #3416
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #3416
    str x0, [x9]
    ; @src line=60 col=11
    sub x9, x29, #3424
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=25
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #3456
    str x0, [x9]
    ; @src line=60 col=26
    mov x0, #2
    mov x21, x0
    ; @src line=60 col=26
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3456
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3456
    str x0, [x9]
    ; @src line=60 col=29
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #4
    sub x9, x29, #3480
    str x1, [x9]
    sub x9, x29, #3472
    str x2, [x9]
    ; @src line=60 col=29
    sub x9, x29, #3480
    ldr x1, [x9]
    sub x9, x29, #3472
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3456
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3456
    str x0, [x9]
    ; @src line=60 col=25
    sub x9, x29, #3456
    ldr x1, [x9]
    sub x9, x29, #3416
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #3416
    str x0, [x9]
    ; @src line=60 col=25
    sub x9, x29, #3456
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=1
    sub x9, x29, #3416
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3488
    str x0, [x9]
    ; @src line=60 col=1
    sub x9, x29, #3488
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    str x0, [x9]
    ; @src line=60 col=1
    sub x9, x29, #3416
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=61 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=61 col=6
    mov x0, #0
    mov x21, x0
    ; @src line=61 col=1
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    ; @src line=62 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_while_cond_31
_eir_main_if_else_30:
    ; @src line=55 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=55 col=34
    adrp x1, _str_51@PAGE
    add x1, x1, _str_51@PAGEOFF
    mov x2, #22
    sub x9, x29, #3832
    str x1, [x9]
    sub x9, x29, #3824
    str x2, [x9]
    ; @src line=55 col=5
    sub x9, x29, #3832
    ldr x1, [x9]
    sub x9, x29, #3824
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=60 col=10
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
    sub x9, x29, #3840
    str x0, [x9]
    ; @src line=60 col=11
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #3848
    str x0, [x9]
    ; @src line=60 col=12
    mov x0, #1
    mov x21, x0
    ; @src line=60 col=12
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3848
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3848
    str x0, [x9]
    ; @src line=60 col=15
    adrp x1, _str_49@PAGE
    add x1, x1, _str_49@PAGEOFF
    mov x2, #5
    sub x9, x29, #3872
    str x1, [x9]
    sub x9, x29, #3864
    str x2, [x9]
    ; @src line=60 col=15
    sub x9, x29, #3872
    ldr x1, [x9]
    sub x9, x29, #3864
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3848
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3848
    str x0, [x9]
    ; @src line=60 col=11
    sub x9, x29, #3848
    ldr x1, [x9]
    sub x9, x29, #3840
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #3840
    str x0, [x9]
    ; @src line=60 col=11
    sub x9, x29, #3848
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=25
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    sub x9, x29, #3880
    str x0, [x9]
    ; @src line=60 col=26
    mov x0, #2
    mov x21, x0
    ; @src line=60 col=26
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3880
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3880
    str x0, [x9]
    ; @src line=60 col=29
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #4
    sub x9, x29, #3904
    str x1, [x9]
    sub x9, x29, #3896
    str x2, [x9]
    ; @src line=60 col=29
    sub x9, x29, #3904
    ldr x1, [x9]
    sub x9, x29, #3896
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    sub x9, x29, #3880
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    sub x9, x29, #3880
    str x0, [x9]
    ; @src line=60 col=25
    sub x9, x29, #3880
    ldr x1, [x9]
    sub x9, x29, #3840
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    sub x9, x29, #3840
    str x0, [x9]
    ; @src line=60 col=25
    sub x9, x29, #3880
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=1
    sub x9, x29, #3840
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3912
    str x0, [x9]
    ; @src line=60 col=1
    sub x9, x29, #3912
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    str x0, [x9]
    ; @src line=60 col=1
    sub x9, x29, #3840
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=61 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=61 col=6
    mov x0, #0
    mov x21, x0
    ; @src line=61 col=1
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    ; @src line=62 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_while_cond_40
_eir_main_while_cond_31:
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    sub x9, x29, #3504
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    sub x9, x29, #3504
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_isset_array_idx_missing_95
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_isset_array_idx_missing_95
    mov x0, #0
    b _eir_main_isset_array_idx_done_96
_eir_main_isset_array_idx_missing_95:
    mov x0, #1
_eir_main_isset_array_idx_done_96:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_36
    b _eir_main_isset_lazy_false_34
_eir_main_while_body_32:
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=63 col=10
    adrp x1, _str_52@PAGE
    add x1, x1, _str_52@PAGEOFF
    mov x2, #4
    sub x9, x29, #3720
    str x1, [x9]
    sub x9, x29, #3712
    str x2, [x9]
    ; @src line=63 col=19
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    ldr x0, [x9]
    sub x9, x29, #3728
    str x0, [x9]
    ; @src line=63 col=17
    sub x9, x29, #3728
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    sub x9, x29, #3744
    str x1, [x9]
    sub x9, x29, #3736
    str x2, [x9]
    ; @src line=63 col=17
    sub x9, x29, #3720
    ldr x1, [x9]
    sub x9, x29, #3712
    ldr x2, [x9]
    sub x9, x29, #3744
    ldr x3, [x9]
    sub x9, x29, #3736
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #3760
    str x1, [x9]
    sub x9, x29, #3752
    str x2, [x9]
    ; @src line=63 col=17
    sub x9, x29, #3744
    ldr x1, [x9]
    sub x9, x29, #3736
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=63 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    sub x9, x29, #3776
    str x1, [x9]
    sub x9, x29, #3768
    str x2, [x9]
    ; @src line=63 col=24
    sub x9, x29, #3760
    ldr x1, [x9]
    sub x9, x29, #3752
    ldr x2, [x9]
    sub x9, x29, #3776
    ldr x3, [x9]
    sub x9, x29, #3768
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #3792
    str x1, [x9]
    sub x9, x29, #3784
    str x2, [x9]
    ; @src line=63 col=24
    ; @src line=63 col=5
    sub x9, x29, #3792
    ldr x1, [x9]
    sub x9, x29, #3784
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=63 col=5
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=64 col=5
    mov x0, #1
    mov x12, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x10, x12
    add x0, x0, x10
    mov x21, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    b _eir_main_while_cond_31
_eir_main_while_exit_33:
    udf #0
_eir_main_isset_lazy_false_34:
    ; @src line=62 col=34
    mov x0, #0
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1625
    str x0, [x9]
    b _eir_main_isset_lazy_merge_35
_eir_main_isset_lazy_merge_35:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1625
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_37
    b _eir_main_coalesce_absent_38
_eir_main_isset_lazy_true_36:
    ; @src line=62 col=34
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1625
    str x0, [x9]
    b _eir_main_isset_lazy_merge_35
_eir_main_coalesce_present_37:
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    sub x9, x29, #3552
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    sub x9, x29, #3552
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_97
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_97
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_98
_eir_main_array_get_null_97:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_98:
    sub x9, x29, #3568
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3568
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    sub x9, x29, #3576
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3568
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=62 col=34
    sub x9, x29, #3576
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3584
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3584
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3576
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_39
_eir_main_coalesce_absent_38:
    ; @src line=62 col=37
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #3600
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3600
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3608
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    ldr x0, [x9]
    sub x9, x29, #3616
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3616
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=34
    sub x9, x29, #3608
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3600
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_39
_eir_main_coalesce_merge_39:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1633
    ldr x0, [x9]
    sub x9, x29, #3624
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3624
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3632
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3632
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    sub x9, x29, #3640
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3640
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3648
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3648
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    ldr x0, [x9]
    sub x9, x29, #3656
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=8
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #3656
    ldr x0, [x9]
    bl __rt_mixed_array_get
    sub x9, x29, #3672
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3672
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3680
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3680
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3672
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    sub x9, x29, #3688
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3688
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #3696
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3696
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #3696
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_while_body_32
    b _eir_main_if_merge_13
_eir_main_while_cond_40:
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    sub x9, x29, #3928
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    sub x9, x29, #3928
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_isset_array_idx_missing_99
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_isset_array_idx_missing_99
    mov x0, #0
    b _eir_main_isset_array_idx_done_100
_eir_main_isset_array_idx_missing_99:
    mov x0, #1
_eir_main_isset_array_idx_done_100:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_45
    b _eir_main_isset_lazy_false_43
_eir_main_while_body_41:
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=63 col=10
    adrp x1, _str_52@PAGE
    add x1, x1, _str_52@PAGEOFF
    mov x2, #4
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #49
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #41
    str x2, [x9]
    ; @src line=63 col=19
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #57
    str x0, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #57
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #73
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #65
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #49
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #41
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #73
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #65
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #89
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #81
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #73
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #65
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=63 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #105
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #97
    str x2, [x9]
    ; @src line=63 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #89
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #81
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #105
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #97
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #121
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #113
    str x2, [x9]
    ; @src line=63 col=24
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #121
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #113
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=63 col=5
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=64 col=5
    mov x0, #1
    mov x12, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x10, x12
    add x0, x0, x10
    mov x21, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    b _eir_main_while_cond_40
_eir_main_while_exit_42:
    udf #0
_eir_main_isset_lazy_false_43:
    ; @src line=62 col=34
    mov x0, #0
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1673
    str x0, [x9]
    b _eir_main_isset_lazy_merge_44
_eir_main_isset_lazy_merge_44:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1673
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_46
    b _eir_main_coalesce_absent_47
_eir_main_isset_lazy_true_45:
    ; @src line=62 col=34
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1673
    str x0, [x9]
    b _eir_main_isset_lazy_merge_44
_eir_main_coalesce_present_46:
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    sub x9, x29, #3976
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    sub x9, x29, #3976
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_101
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_101
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_102
_eir_main_array_get_null_101:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_102:
    sub x9, x29, #3992
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3992
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    sub x9, x29, #4000
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #3992
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=62 col=34
    sub x9, x29, #4000
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #4008
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #4008
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #4000
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_48
_eir_main_coalesce_absent_47:
    ; @src line=62 col=37
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #4024
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #4024
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #4032
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    ldr x0, [x9]
    sub x9, x29, #4040
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #4040
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=34
    sub x9, x29, #4032
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    str x0, [x9]
    ; @src line=62 col=34
    sub x9, x29, #4024
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_48
_eir_main_coalesce_merge_48:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1681
    ldr x0, [x9]
    sub x9, x29, #4048
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #4048
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #4056
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #4056
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    sub x9, x29, #4064
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #4064
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #4072
    str x0, [x9]
    ; @src line=62 col=8
    sub x9, x29, #4072
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    ldr x0, [x9]
    sub x9, x29, #4080
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=8
    mov x1, x21
    mov x2, #-1
    sub x9, x29, #4080
    ldr x0, [x9]
    bl __rt_mixed_array_get
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #9
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #9
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #17
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #17
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #25
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #25
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #25
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_while_body_41
    b _eir_main_if_merge_13
_eir_main_if_merge_49:
    udf #0
_eir_main_isset_lazy_false_50:
    ; @src line=52 col=42
    mov x0, #0
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1689
    str x0, [x9]
    b _eir_main_isset_lazy_merge_51
_eir_main_isset_lazy_merge_51:
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1689
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_53
    b _eir_main_coalesce_absent_54
_eir_main_isset_lazy_true_52:
    ; @src line=52 col=42
    mov x0, #1
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1689
    str x0, [x9]
    b _eir_main_isset_lazy_merge_51
_eir_main_coalesce_present_53:
    ; @src line=52 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1497
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #209
    str x0, [x9]
    ; @src line=52 col=31
    adrp x1, _str_47@PAGE
    add x1, x1, _str_47@PAGEOFF
    mov x2, #7
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #225
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #217
    str x2, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #225
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #217
    ldr x2, [x9]
    bl __rt_hash_normalize_key
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #209
    ldr x0, [x9]
    bl __rt_hash_get
    cbz x0, _eir_main_hash_get_miss_103
    bl __rt_deref_if_reference
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_hash_get_done_104
_eir_main_hash_get_miss_103:
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_hash_get_done_104:
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #233
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #233
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #241
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #233
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #241
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #249
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #249
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #241
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_55
_eir_main_coalesce_absent_54:
    ; @src line=52 col=45
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=52 col=42
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #265
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #265
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #273
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #281
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #281
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #273
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    str x0, [x9]
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #265
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_55
_eir_main_coalesce_merge_55:
    ; @src line=52 col=42
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1697
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #289
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #289
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #297
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #297
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #305
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #305
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #313
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #313
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1593
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #321
    str x0, [x9]
    mov x0, #3
    mov x21, x0
    ; @src line=52 col=5
    mov x1, x21
    mov x2, #-1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #321
    ldr x0, [x9]
    bl __rt_mixed_array_get
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #337
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #345
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #345
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #337
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #353
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #353
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1553
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #337
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1585
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #361
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #361
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #369
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #369
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1601
    str x0, [x9]
    ; @src line=52 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #369
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_if_then_56
    b _eir_main_if_else_57
_eir_main_if_then_56:
    ; @src line=53 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=53 col=24
    adrp x1, _str_48@PAGE
    add x1, x1, _str_48@PAGEOFF
    mov x2, #12
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #393
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #385
    str x2, [x9]
    ; @src line=53 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #393
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #385
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=60 col=10
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
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    str x0, [x9]
    ; @src line=60 col=11
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    str x0, [x9]
    ; @src line=60 col=12
    mov x0, #1
    mov x21, x0
    ; @src line=60 col=12
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    str x0, [x9]
    ; @src line=60 col=15
    adrp x1, _str_49@PAGE
    add x1, x1, _str_49@PAGEOFF
    mov x2, #5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #433
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #425
    str x2, [x9]
    ; @src line=60 col=15
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #433
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #425
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    str x0, [x9]
    ; @src line=60 col=11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    str x0, [x9]
    ; @src line=60 col=11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #409
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=25
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    str x0, [x9]
    ; @src line=60 col=26
    mov x0, #2
    mov x21, x0
    ; @src line=60 col=26
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    str x0, [x9]
    ; @src line=60 col=29
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #4
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #465
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #457
    str x2, [x9]
    ; @src line=60 col=29
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #465
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #457
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    str x0, [x9]
    ; @src line=60 col=25
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    str x0, [x9]
    ; @src line=60 col=25
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #441
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #473
    str x0, [x9]
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #473
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    str x0, [x9]
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #401
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=61 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=61 col=6
    mov x0, #0
    mov x21, x0
    ; @src line=61 col=1
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    ; @src line=62 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_while_cond_58
_eir_main_if_else_57:
    ; @src line=55 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=55 col=34
    adrp x1, _str_51@PAGE
    add x1, x1, _str_51@PAGEOFF
    mov x2, #22
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #817
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #809
    str x2, [x9]
    ; @src line=55 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #817
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #809
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=60 col=10
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
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    str x0, [x9]
    ; @src line=60 col=11
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    str x0, [x9]
    ; @src line=60 col=12
    mov x0, #1
    mov x21, x0
    ; @src line=60 col=12
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    str x0, [x9]
    ; @src line=60 col=15
    adrp x1, _str_49@PAGE
    add x1, x1, _str_49@PAGEOFF
    mov x2, #5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #857
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #849
    str x2, [x9]
    ; @src line=60 col=15
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #857
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #849
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    str x0, [x9]
    ; @src line=60 col=11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    str x0, [x9]
    ; @src line=60 col=11
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #833
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=25
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    str x0, [x9]
    ; @src line=60 col=26
    mov x0, #2
    mov x21, x0
    ; @src line=60 col=26
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    str x0, [x9]
    ; @src line=60 col=29
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #4
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #889
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #881
    str x2, [x9]
    ; @src line=60 col=29
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #889
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #881
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    ldr x9, [x9]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    str x0, [x9]
    ; @src line=60 col=25
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    ldr x9, [x9]
    mov x0, x9
    bl __rt_array_push_refcounted
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    str x0, [x9]
    ; @src line=60 col=25
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #865
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #897
    str x0, [x9]
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #897
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    str x0, [x9]
    ; @src line=60 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #825
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=61 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=61 col=6
    mov x0, #0
    mov x21, x0
    ; @src line=61 col=1
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    ; @src line=62 col=1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_while_cond_67
_eir_main_while_cond_58:
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #489
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #489
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_isset_array_idx_missing_105
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_isset_array_idx_missing_105
    mov x0, #0
    b _eir_main_isset_array_idx_done_106
_eir_main_isset_array_idx_missing_105:
    mov x0, #1
_eir_main_isset_array_idx_done_106:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_63
    b _eir_main_isset_lazy_false_61
_eir_main_while_body_59:
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=63 col=10
    adrp x1, _str_52@PAGE
    add x1, x1, _str_52@PAGEOFF
    mov x2, #4
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #705
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #697
    str x2, [x9]
    ; @src line=63 col=19
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #713
    str x0, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #713
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #729
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #721
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #705
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #697
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #729
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #721
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #745
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #737
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #729
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #721
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=63 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #761
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #753
    str x2, [x9]
    ; @src line=63 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #745
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #737
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #761
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #753
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #777
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #769
    str x2, [x9]
    ; @src line=63 col=24
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #777
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #769
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=63 col=5
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=64 col=5
    mov x0, #1
    mov x12, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x10, x12
    add x0, x0, x10
    mov x21, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    b _eir_main_while_cond_58
_eir_main_while_exit_60:
    udf #0
_eir_main_isset_lazy_false_61:
    ; @src line=62 col=34
    mov x0, #0
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1705
    str x0, [x9]
    b _eir_main_isset_lazy_merge_62
_eir_main_isset_lazy_merge_62:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1705
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_64
    b _eir_main_coalesce_absent_65
_eir_main_isset_lazy_true_63:
    ; @src line=62 col=34
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1705
    str x0, [x9]
    b _eir_main_isset_lazy_merge_62
_eir_main_coalesce_present_64:
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #537
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #537
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_107
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_107
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_108
_eir_main_array_get_null_107:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_108:
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #553
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #553
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #561
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #553
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #561
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #569
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #569
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #561
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_66
_eir_main_coalesce_absent_65:
    ; @src line=62 col=37
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #585
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #585
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #593
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #601
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #601
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #593
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #585
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_66
_eir_main_coalesce_merge_66:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1713
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #609
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #609
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #617
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #617
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #625
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #625
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #633
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #633
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #641
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=8
    mov x1, x21
    mov x2, #-1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #641
    ldr x0, [x9]
    bl __rt_mixed_array_get
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #657
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #657
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #665
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #665
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #657
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #673
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #673
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #681
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #681
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #681
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_while_body_59
    b _eir_main_if_merge_13
_eir_main_while_cond_67:
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #913
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #913
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_isset_array_idx_missing_109
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_isset_array_idx_missing_109
    mov x0, #0
    b _eir_main_isset_array_idx_done_110
_eir_main_isset_array_idx_missing_109:
    mov x0, #1
_eir_main_isset_array_idx_done_110:
    cmp x0, #0
    cset x0, eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_isset_lazy_true_72
    b _eir_main_isset_lazy_false_70
_eir_main_while_body_68:
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=63 col=10
    adrp x1, _str_52@PAGE
    add x1, x1, _str_52@PAGEOFF
    mov x2, #4
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1129
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1121
    str x2, [x9]
    ; @src line=63 col=19
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1137
    str x0, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1137
    ldr x0, [x9]
    bl __rt_mixed_cast_string
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1153
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1145
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1129
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1121
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1153
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1145
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1169
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1161
    str x2, [x9]
    ; @src line=63 col=17
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1153
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1145
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=63 col=26
    adrp x1, _str_25@PAGE
    add x1, x1, _str_25@PAGEOFF
    mov x2, #1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1185
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1177
    str x2, [x9]
    ; @src line=63 col=24
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1169
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1161
    ldr x2, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1185
    ldr x3, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1177
    ldr x4, [x9]
    bl __rt_concat
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1201
    str x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1193
    str x2, [x9]
    ; @src line=63 col=24
    ; @src line=63 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1201
    ldr x1, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1193
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=63 col=5
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=64 col=5
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=64 col=5
    mov x0, #1
    mov x12, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x10, x12
    add x0, x0, x10
    mov x21, x0
    ; @src line=64 col=5
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    str x0, [x9]
    b _eir_main_while_cond_67
_eir_main_while_exit_69:
    udf #0
_eir_main_isset_lazy_false_70:
    ; @src line=62 col=34
    mov x0, #0
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1721
    str x0, [x9]
    b _eir_main_isset_lazy_merge_71
_eir_main_isset_lazy_merge_71:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1721
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_coalesce_present_73
    b _eir_main_coalesce_absent_74
_eir_main_isset_lazy_true_72:
    ; @src line=62 col=34
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1721
    str x0, [x9]
    b _eir_main_isset_lazy_merge_71
_eir_main_coalesce_present_73:
    ; @src line=62 col=23
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1609
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #961
    str x0, [x9]
    ; @src line=62 col=30
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1617
    ldr x0, [x9]
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #961
    ldr x9, [x9]
    cmp x0, #0
    b.lt _eir_main_array_get_null_111
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir_main_array_get_null_111
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir_main_array_get_done_112
_eir_main_array_get_null_111:
    bl __rt_warn_undefined_array_key_int
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
_eir_main_array_get_done_112:
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #977
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #977
    ldr x0, [x9]
    bl __rt_mixed_from_array_kind
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #985
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #977
    ldr x0, [x9]
    bl __rt_decref_any
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #985
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #993
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #993
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #985
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_75
_eir_main_coalesce_absent_74:
    ; @src line=62 col=37
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=62 col=34
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1009
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1009
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1017
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1025
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1025
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1017
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    str x0, [x9]
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1009
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_coalesce_merge_75
_eir_main_coalesce_merge_75:
    ; @src line=62 col=34
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1729
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1033
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1033
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1041
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1041
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1049
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1049
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1057
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1057
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1745
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1649
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1065
    str x0, [x9]
    mov x0, #1
    mov x21, x0
    ; @src line=62 col=8
    mov x1, x21
    mov x2, #-1
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1065
    ldr x0, [x9]
    bl __rt_mixed_array_get
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1081
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1081
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1089
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1089
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1657
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1081
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1641
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1097
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1097
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1105
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1105
    ldr x0, [x9]
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1665
    str x0, [x9]
    ; @src line=62 col=8
    mov x9, x29
    sub x9, x9, #4095
    sub x9, x9, #1105
    ldr x0, [x9]
    bl __rt_mixed_cast_bool
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_while_body_68
    b _eir_main_if_merge_13
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
    .ascii "Fatal error: Typed property ReflectionAttribute::$__name must not be accessed before initialization\n"
.globl _str_4
_str_4:
    .ascii "Fatal error: Typed property ReflectionClass::$__name must not be accessed before initialization\n"
.globl _str_5
_str_5:
    .ascii "Fatal error: Typed property ReflectionClass::$__attrs must not be accessed before initialization\n"
.globl _str_6
_str_6:
    .ascii "Fatal error: Typed property ReflectionMethod::$__attrs must not be accessed before initialization\n"
.globl _str_7
_str_7:
    .ascii "Fatal error: Typed property ReflectionMethod::$__name must not be accessed before initialization\n"
.globl _str_8
_str_8:
    .ascii "Fatal error: Typed property ReflectionProperty::$__attrs must not be accessed before initialization\n"
.globl _str_9
_str_9:
    .ascii "Fatal error: Typed property ReflectionProperty::$__name must not be accessed before initialization\n"
.globl _str_10
_str_10:
    .ascii "Fatal error: Typed property ReflectionFunction::$__name must not be accessed before initialization\n"
.globl _str_11
_str_11:
    .ascii "Fatal error: Typed property ReflectionFunction::$__short must not be accessed before initialization\n"
.globl _str_12
_str_12:
    .ascii "Fatal error: Typed property ReflectionFunction::$__num_params must not be accessed before initialization\n"
.globl _str_13
_str_13:
    .ascii "Fatal error: Typed property ReflectionFunction::$__num_required must not be accessed before initialization\n"
.globl _str_14
_str_14:
    .ascii "Fatal error: Typed property ReflectionFunction::$__params must not be accessed before initialization\n"
.globl _str_15
_str_15:
    .ascii "Fatal error: Typed property ReflectionParameter::$__name must not be accessed before initialization\n"
.globl _str_16
_str_16:
    .ascii "Fatal error: Typed property ReflectionParameter::$__position must not be accessed before initialization\n"
.globl _str_17
_str_17:
    .ascii "Fatal error: Typed property ReflectionParameter::$__optional must not be accessed before initialization\n"
.globl _str_18
_str_18:
    .ascii "Fatal error: Typed property ReflectionParameter::$__variadic must not be accessed before initialization\n"
.globl _str_19
_str_19:
    .ascii "Fatal error: Typed property ReflectionParameter::$__has_type must not be accessed before initialization\n"
.globl _str_20
_str_20:
    .ascii "Fatal error: Typed property ReflectionParameter::$__type must not be accessed before initialization\n"
.globl _str_21
_str_21:
    .ascii "Fatal error: Typed property ReflectionNamedType::$__name must not be accessed before initialization\n"
.globl _str_22
_str_22:
    .ascii "Fatal error: Typed property ReflectionNamedType::$__allows_null must not be accessed before initialization\n"
.globl _str_23
_str_23:
    .ascii "Fatal error: Typed property ReflectionNamedType::$__builtin must not be accessed before initialization\n"
.globl _str_24
_str_24:
    .ascii ","
.globl _str_25
_str_25:
    .ascii "\n"
.globl _str_26
_str_26:
    .ascii "name"
.globl _str_27
_str_27:
    .ascii "Ada"
.globl _str_28
_str_28:
    .ascii "id"
.globl _str_29
_str_29:
    .ascii ":"
.globl _str_30
_str_30:
    .ascii "alice"
.globl _str_31
_str_31:
    .ascii "Alice"
.globl _str_32
_str_32:
    .ascii "bob"
.globl _str_33
_str_33:
    .ascii "Bob"
.globl _str_34
_str_34:
    .ascii "="
.globl _str_35
_str_35:
    .ascii "("
.globl _str_36
_str_36:
    .ascii ")\n"
.globl _str_37
_str_37:
    .ascii "one"
.globl _str_38
_str_38:
    .ascii "two"
.globl _str_39
_str_39:
    .ascii "three"
.globl _str_40
_str_40:
    .ascii "user"
.globl _str_41
_str_41:
    .ascii "App\\User"
.globl _str_42
_str_42:
    .ascii "private"
.globl _str_43
_str_43:
    .ascii "readonly"
.globl _str_44
_str_44:
    .ascii "row "
.globl _str_45
_str_45:
    .ascii " = "
.globl _str_46
_str_46:
    .ascii " access = "
.globl _str_47
_str_47:
    .ascii "missing"
.globl _str_48
_str_48:
    .ascii "unreachable\n"
.globl _str_49
_str_49:
    .ascii "alpha"
.globl _str_50
_str_50:
    .ascii "beta"
.globl _str_51
_str_51:
    .ascii "missing scope skipped\n"
.globl _str_52
_str_52:
    .ascii "tag "
.p2align 3
.globl _float_2
_float_2:
    .quad 0x0000000000000000

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
    .quad 40
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_0
    .quad 5
    .quad 0
    .quad 0
    .quad _instanceof_name_class_abs_0
    .quad 6
    .quad 0
    .quad 0
    .quad _instanceof_name_class_1
    .quad 9
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 10
    .quad 1
    .quad 0
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
    .quad _instanceof_name_class_4
    .quad 13
    .quad 4
    .quad 0
    .quad _instanceof_name_class_abs_4
    .quad 14
    .quad 4
    .quad 0
    .quad _instanceof_name_class_5
    .quad 14
    .quad 5
    .quad 0
    .quad _instanceof_name_class_abs_5
    .quad 15
    .quad 5
    .quad 0
    .quad _instanceof_name_class_6
    .quad 24
    .quad 6
    .quad 0
    .quad _instanceof_name_class_abs_6
    .quad 25
    .quad 6
    .quad 0
    .quad _instanceof_name_class_24
    .quad 19
    .quad 24
    .quad 0
    .quad _instanceof_name_class_abs_24
    .quad 20
    .quad 24
    .quad 0
    .quad _instanceof_name_class_25
    .quad 16
    .quad 25
    .quad 0
    .quad _instanceof_name_class_abs_25
    .quad 17
    .quad 25
    .quad 0
    .quad _instanceof_name_class_26
    .quad 15
    .quad 26
    .quad 0
    .quad _instanceof_name_class_abs_26
    .quad 16
    .quad 26
    .quad 0
    .quad _instanceof_name_class_45
    .quad 20
    .quad 45
    .quad 0
    .quad _instanceof_name_class_abs_45
    .quad 21
    .quad 45
    .quad 0
    .quad _instanceof_name_class_53
    .quad 18
    .quad 53
    .quad 0
    .quad _instanceof_name_class_abs_53
    .quad 19
    .quad 53
    .quad 0
    .quad _instanceof_name_class_59
    .quad 18
    .quad 59
    .quad 0
    .quad _instanceof_name_class_abs_59
    .quad 19
    .quad 59
    .quad 0
    .quad _instanceof_name_class_60
    .quad 19
    .quad 60
    .quad 0
    .quad _instanceof_name_class_abs_60
    .quad 20
    .quad 60
    .quad 0
    .quad _instanceof_name_class_62
    .quad 14
    .quad 62
    .quad 0
    .quad _instanceof_name_class_abs_62
    .quad 15
    .quad 62
    .quad 0
    .quad _instanceof_name_class_77
    .quad 10
    .quad 77
    .quad 0
    .quad _instanceof_name_class_abs_77
    .quad 11
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
    .quad _instanceof_name_class_92
    .quad 19
    .quad 92
    .quad 0
    .quad _instanceof_name_class_abs_92
    .quad 20
    .quad 92
    .quad 0
    .quad _instanceof_name_interface_10
    .quad 9
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_abs_10
    .quad 10
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_14
    .quad 10
    .quad 14
    .quad 1
    .quad _instanceof_name_interface_abs_14
    .quad 11
    .quad 14
    .quad 1
.globl _instanceof_name_class_0
_instanceof_name_class_0:
    .ascii "Error"
.globl _instanceof_name_class_abs_0
_instanceof_name_class_abs_0:
    .ascii "\\Error"
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\TypeError"
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
.globl _instanceof_name_class_4
_instanceof_name_class_4:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_4
_instanceof_name_class_abs_4:
    .ascii "\\JsonException"
.globl _instanceof_name_class_5
_instanceof_name_class_5:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_5
_instanceof_name_class_abs_5:
    .ascii "\\LogicException"
.globl _instanceof_name_class_6
_instanceof_name_class_6:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_6
_instanceof_name_class_abs_6:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_24
_instanceof_name_class_24:
    .ascii "ReflectionAttribute"
.globl _instanceof_name_class_abs_24
_instanceof_name_class_abs_24:
    .ascii "\\ReflectionAttribute"
.globl _instanceof_name_class_25
_instanceof_name_class_25:
    .ascii "ReflectionMethod"
.globl _instanceof_name_class_abs_25
_instanceof_name_class_abs_25:
    .ascii "\\ReflectionMethod"
.globl _instanceof_name_class_26
_instanceof_name_class_26:
    .ascii "ReflectionClass"
.globl _instanceof_name_class_abs_26
_instanceof_name_class_abs_26:
    .ascii "\\ReflectionClass"
.globl _instanceof_name_class_45
_instanceof_name_class_45:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_45
_instanceof_name_class_abs_45:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_53
_instanceof_name_class_53:
    .ascii "ReflectionFunction"
.globl _instanceof_name_class_abs_53
_instanceof_name_class_abs_53:
    .ascii "\\ReflectionFunction"
.globl _instanceof_name_class_59
_instanceof_name_class_59:
    .ascii "ReflectionProperty"
.globl _instanceof_name_class_abs_59
_instanceof_name_class_abs_59:
    .ascii "\\ReflectionProperty"
.globl _instanceof_name_class_60
_instanceof_name_class_60:
    .ascii "ReflectionParameter"
.globl _instanceof_name_class_abs_60
_instanceof_name_class_abs_60:
    .ascii "\\ReflectionParameter"
.globl _instanceof_name_class_62
_instanceof_name_class_62:
    .ascii "ReflectionType"
.globl _instanceof_name_class_abs_62
_instanceof_name_class_abs_62:
    .ascii "\\ReflectionType"
.globl _instanceof_name_class_77
_instanceof_name_class_77:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_77
_instanceof_name_class_abs_77:
    .ascii "\\ValueError"
.globl _instanceof_name_class_89
_instanceof_name_class_89:
    .ascii "ReflectionNamedType"
.globl _instanceof_name_class_abs_89
_instanceof_name_class_abs_89:
    .ascii "\\ReflectionNamedType"
.globl _instanceof_name_class_92
_instanceof_name_class_92:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_92
_instanceof_name_class_abs_92:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_interface_10
_instanceof_name_interface_10:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_10
_instanceof_name_interface_abs_10:
    .ascii "\\Throwable"
.globl _instanceof_name_interface_14
_instanceof_name_interface_14:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_14
_instanceof_name_interface_abs_14:
    .ascii "\\Stringable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 93
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_0
    .quad 5
    .quad _class_name_1
    .quad 9
    .quad _class_name_2
    .quad 9
    .quad _class_name_3
    .quad 16
    .quad _class_name_4
    .quad 13
    .quad _class_name_5
    .quad 14
    .quad _class_name_6
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_24
    .quad 19
    .quad _class_name_25
    .quad 16
    .quad _class_name_26
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_45
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
    .quad _class_name_53
    .quad 18
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_59
    .quad 18
    .quad _class_name_60
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_62
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
    .quad _class_name_77
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
    .quad _class_name_89
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_92
    .quad 19
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_0
_class_name_0:
    .ascii "Error"
.globl _class_name_1
_class_name_1:
    .ascii "TypeError"
.globl _class_name_2
_class_name_2:
    .ascii "Exception"
.globl _class_name_3
_class_name_3:
    .ascii "RuntimeException"
.globl _class_name_4
_class_name_4:
    .ascii "JsonException"
.globl _class_name_5
_class_name_5:
    .ascii "LogicException"
.globl _class_name_6
_class_name_6:
    .ascii "InvalidArgumentException"
.globl _class_name_24
_class_name_24:
    .ascii "ReflectionAttribute"
.globl _class_name_25
_class_name_25:
    .ascii "ReflectionMethod"
.globl _class_name_26
_class_name_26:
    .ascii "ReflectionClass"
.globl _class_name_45
_class_name_45:
    .ascii "OutOfBoundsException"
.globl _class_name_53
_class_name_53:
    .ascii "ReflectionFunction"
.globl _class_name_59
_class_name_59:
    .ascii "ReflectionProperty"
.globl _class_name_60
_class_name_60:
    .ascii "ReflectionParameter"
.globl _class_name_62
_class_name_62:
    .ascii "ReflectionType"
.globl _class_name_77
_class_name_77:
    .ascii "ValueError"
.globl _class_name_89
_class_name_89:
    .ascii "ReflectionNamedType"
.globl _class_name_92
_class_name_92:
    .ascii "OutOfRangeException"
    .p2align 3
.globl _fiber_class_id
_fiber_class_id:
    .quad 32
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 91
.globl _generator_class_id
_generator_class_id:
    .quad 83
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 21
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 69
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 65
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 52
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 5
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 3
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 92
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 45
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 6
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 1
.globl _spl_value_error_class_id
_spl_value_error_class_id:
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
    .quad _class_interfaces_2
    .quad _class_interfaces_3
    .quad _class_interfaces_4
    .quad _class_interfaces_5
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_24
    .quad _class_interfaces_25
    .quad _class_interfaces_26
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
    .quad _class_interfaces_45
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_53
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_59
    .quad _class_interfaces_60
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_92
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_0
    .quad _class_json_desc_1
    .quad _class_json_desc_2
    .quad _class_json_desc_3
    .quad _class_json_desc_4
    .quad _class_json_desc_5
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_24
    .quad _class_json_desc_25
    .quad _class_json_desc_26
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
    .quad _class_json_desc_45
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_53
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_59
    .quad _class_json_desc_60
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_92
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 4
.globl _class_parent_ids
_class_parent_ids:
    .quad -1
    .quad 0
    .quad -1
    .quad 2
    .quad 3
    .quad 2
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 62
    .quad -1
    .quad -1
    .quad 5
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 93
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_0
    .quad _class_gc_desc_1
    .quad _class_gc_desc_2
    .quad _class_gc_desc_3
    .quad _class_gc_desc_4
    .quad _class_gc_desc_5
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_24
    .quad _class_gc_desc_25
    .quad _class_gc_desc_26
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
    .quad _class_gc_desc_45
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_53
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_59
    .quad _class_gc_desc_60
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_92
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_0
    .quad _class_vtable_1
    .quad _class_vtable_2
    .quad _class_vtable_3
    .quad _class_vtable_4
    .quad _class_vtable_5
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
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_24
    .quad _class_vtable_25
    .quad _class_vtable_26
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
    .quad _class_vtable_45
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_53
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_59
    .quad _class_vtable_60
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
    .quad _class_vtable_missing
    .quad _class_vtable_92
.globl _class_destruct_count
_class_destruct_count:
    .quad 93
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
.globl _class_clone_count
_class_clone_count:
    .quad 93
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
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad _class_propinit_0
    .quad _class_propinit_1
    .quad _class_propinit_2
    .quad _class_propinit_3
    .quad _class_propinit_4
    .quad _class_propinit_5
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
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_24
    .quad _class_propinit_25
    .quad _class_propinit_26
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_45
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_53
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_59
    .quad _class_propinit_60
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 0
    .quad _class_propinit_92
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_0
    .quad _class_static_vtable_1
    .quad _class_static_vtable_2
    .quad _class_static_vtable_3
    .quad _class_static_vtable_4
    .quad _class_static_vtable_5
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_24
    .quad _class_static_vtable_25
    .quad _class_static_vtable_26
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
    .quad _class_static_vtable_45
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_53
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_59
    .quad _class_static_vtable_60
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_92
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_0
    .quad _class_callable_methods_1
    .quad _class_callable_methods_2
    .quad _class_callable_methods_3
    .quad _class_callable_methods_4
    .quad _class_callable_methods_5
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_24
    .quad _class_callable_methods_25
    .quad _class_callable_methods_26
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
    .quad _class_callable_methods_45
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_53
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_59
    .quad _class_callable_methods_60
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_92
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
.globl _class_interfaces_missing
_class_interfaces_missing:
    .quad 0
.globl _class_gc_desc_missing
_class_gc_desc_missing:
    .byte 0
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
    .ascii "Error"
.globl _class_by_name_str_1
_class_by_name_str_1:
    .ascii "TypeError"
.globl _class_by_name_str_2
_class_by_name_str_2:
    .ascii "Exception"
.globl _class_by_name_str_3
_class_by_name_str_3:
    .ascii "RuntimeException"
.globl _class_by_name_str_4
_class_by_name_str_4:
    .ascii "JsonException"
.globl _class_by_name_str_5
_class_by_name_str_5:
    .ascii "LogicException"
.globl _class_by_name_str_6
_class_by_name_str_6:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_24
_class_by_name_str_24:
    .ascii "ReflectionAttribute"
.globl _class_by_name_str_25
_class_by_name_str_25:
    .ascii "ReflectionMethod"
.globl _class_by_name_str_26
_class_by_name_str_26:
    .ascii "ReflectionClass"
.globl _class_by_name_str_45
_class_by_name_str_45:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_53
_class_by_name_str_53:
    .ascii "ReflectionFunction"
.globl _class_by_name_str_59
_class_by_name_str_59:
    .ascii "ReflectionProperty"
.globl _class_by_name_str_60
_class_by_name_str_60:
    .ascii "ReflectionParameter"
.globl _class_by_name_str_62
_class_by_name_str_62:
    .ascii "ReflectionType"
.globl _class_by_name_str_77
_class_by_name_str_77:
    .ascii "ValueError"
.globl _class_by_name_str_89
_class_by_name_str_89:
    .ascii "ReflectionNamedType"
.globl _class_by_name_str_92
_class_by_name_str_92:
    .ascii "OutOfRangeException"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 18
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_0
    .quad 5
    .quad 0
    .quad 40
    .quad _class_by_name_str_1
    .quad 9
    .quad 1
    .quad 40
    .quad _class_by_name_str_2
    .quad 9
    .quad 2
    .quad 40
    .quad _class_by_name_str_3
    .quad 16
    .quad 3
    .quad 40
    .quad _class_by_name_str_4
    .quad 13
    .quad 4
    .quad 40
    .quad _class_by_name_str_5
    .quad 14
    .quad 5
    .quad 40
    .quad _class_by_name_str_6
    .quad 24
    .quad 6
    .quad 40
    .quad _class_by_name_str_24
    .quad 19
    .quad 24
    .quad 56
    .quad _class_by_name_str_25
    .quad 16
    .quad 25
    .quad 72
    .quad _class_by_name_str_26
    .quad 15
    .quad 26
    .quad 56
    .quad _class_by_name_str_45
    .quad 20
    .quad 45
    .quad 40
    .quad _class_by_name_str_53
    .quad 18
    .quad 53
    .quad 104
    .quad _class_by_name_str_59
    .quad 18
    .quad 59
    .quad 72
    .quad _class_by_name_str_60
    .quad 19
    .quad 60
    .quad 120
    .quad _class_by_name_str_62
    .quad 14
    .quad 62
    .quad 8
    .quad _class_by_name_str_77
    .quad 10
    .quad 77
    .quad 40
    .quad _class_by_name_str_89
    .quad 19
    .quad 89
    .quad 56
    .quad _class_by_name_str_92
    .quad 19
    .quad 92
    .quad 40
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 93
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
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_10
_interface_methods_10:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _interface_methods_14
_interface_methods_14:
    .quad 1
    .quad 0
.globl _class_interfaces_0
_class_interfaces_0:
    .quad 2
    .quad 10
    .quad _class_interface_impl_0_10
    .quad 14
    .quad _class_interface_impl_0_14
.globl _class_interface_impl_0_10
_class_interface_impl_0_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_0_14
_class_interface_impl_0_14:
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
    .byte 1, 0
    .p2align 3
.globl _class_vtable_0
_class_vtable_0:
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
    .quad 10
    .quad _class_interface_impl_1_10
    .quad 14
    .quad _class_interface_impl_1_14
.globl _class_interface_impl_1_10
_class_interface_impl_1_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_14
_class_interface_impl_1_14:
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
    .byte 1, 0
    .p2align 3
.globl _class_vtable_1
_class_vtable_1:
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
    .quad 10
    .quad _class_interface_impl_2_10
    .quad 14
    .quad _class_interface_impl_2_14
.globl _class_interface_impl_2_10
_class_interface_impl_2_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_2_14
_class_interface_impl_2_14:
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
    .byte 1, 0
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
    .quad 10
    .quad _class_interface_impl_3_10
    .quad 14
    .quad _class_interface_impl_3_14
.globl _class_interface_impl_3_10
_class_interface_impl_3_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_3_14
_class_interface_impl_3_14:
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
    .byte 1, 0
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
    .quad 10
    .quad _class_interface_impl_4_10
    .quad 14
    .quad _class_interface_impl_4_14
.globl _class_interface_impl_4_10
_class_interface_impl_4_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_4_14
_class_interface_impl_4_14:
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
    .byte 1, 0
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
.globl _class_interfaces_5
_class_interfaces_5:
    .quad 2
    .quad 10
    .quad _class_interface_impl_5_10
    .quad 14
    .quad _class_interface_impl_5_14
.globl _class_interface_impl_5_10
_class_interface_impl_5_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_5_14
_class_interface_impl_5_14:
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
    .byte 1, 0
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
    .quad 10
    .quad _class_interface_impl_6_10
    .quad 14
    .quad _class_interface_impl_6_14
.globl _class_interface_impl_6_10
_class_interface_impl_6_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_6_14
_class_interface_impl_6_14:
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
    .byte 1, 0
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
.globl _class_interfaces_24
_class_interfaces_24:
    .quad 0
    .p2align 3
.globl _class_json_desc_24
_class_json_desc_24:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_24
_class_gc_desc_24:
    .byte 1, 4, 0
    .p2align 3
.globl _class_vtable_24
_class_vtable_24:
    .quad _method_ReflectionAttribute_getname
    .quad _method_ReflectionAttribute_getarguments
    .quad _method_ReflectionAttribute_newinstance
    .p2align 3
.globl _class_static_vtable_24
_class_static_vtable_24:
    .quad 0
.globl _class_callable_method_name_24_getarguments
_class_callable_method_name_24_getarguments:
    .ascii "getarguments"
.globl _class_callable_method_name_24_getname
_class_callable_method_name_24_getname:
    .ascii "getname"
.globl _class_callable_method_name_24_newinstance
_class_callable_method_name_24_newinstance:
    .ascii "newinstance"
.p2align 3
.globl _class_callable_methods_24
_class_callable_methods_24:
    .quad 3
    .quad _class_callable_method_name_24_getarguments
    .quad 12
    .quad _class_callable_method_name_24_getname
    .quad 7
    .quad _class_callable_method_name_24_newinstance
    .quad 11
.globl _class_interfaces_25
_class_interfaces_25:
    .quad 0
.globl _class_json_pname_25_2
_class_json_pname_25_2:
    .ascii "name"
.globl _class_json_pname_25_3
_class_json_pname_25_3:
    .ascii "class"
    .p2align 3
.globl _class_json_desc_25
_class_json_desc_25:
    .quad 0
    .quad 0
    .quad 2
    .quad _class_json_pname_25_2
    .quad 4
    .quad 2
    .quad 1
    .quad _class_json_pname_25_3
    .quad 5
    .quad 3
    .quad 1
    .p2align 3
.globl _class_gc_desc_25
_class_gc_desc_25:
    .byte 4, 1, 1, 1
    .p2align 3
.globl _class_vtable_25
_class_vtable_25:
    .quad _method_ReflectionMethod__u__u_construct
    .quad _method_ReflectionMethod_getattributes
    .quad _method_ReflectionMethod_getname
    .quad _method_ReflectionMethod_ispublic
    .quad _method_ReflectionMethod_getclosure
    .quad _method_ReflectionMethod_getdeclaringclass
    .quad _method_ReflectionMethod_isstatic
    .p2align 3
.globl _class_static_vtable_25
_class_static_vtable_25:
    .quad 0
.globl _class_callable_method_name_25__u__u_construct
_class_callable_method_name_25__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_25_getattributes
_class_callable_method_name_25_getattributes:
    .ascii "getattributes"
.globl _class_callable_method_name_25_getclosure
_class_callable_method_name_25_getclosure:
    .ascii "getclosure"
.globl _class_callable_method_name_25_getdeclaringclass
_class_callable_method_name_25_getdeclaringclass:
    .ascii "getdeclaringclass"
.globl _class_callable_method_name_25_getname
_class_callable_method_name_25_getname:
    .ascii "getname"
.globl _class_callable_method_name_25_ispublic
_class_callable_method_name_25_ispublic:
    .ascii "ispublic"
.globl _class_callable_method_name_25_isstatic
_class_callable_method_name_25_isstatic:
    .ascii "isstatic"
.p2align 3
.globl _class_callable_methods_25
_class_callable_methods_25:
    .quad 7
    .quad _class_callable_method_name_25__u__u_construct
    .quad 11
    .quad _class_callable_method_name_25_getattributes
    .quad 13
    .quad _class_callable_method_name_25_getclosure
    .quad 10
    .quad _class_callable_method_name_25_getdeclaringclass
    .quad 17
    .quad _class_callable_method_name_25_getname
    .quad 7
    .quad _class_callable_method_name_25_ispublic
    .quad 8
    .quad _class_callable_method_name_25_isstatic
    .quad 8
.globl _class_interfaces_26
_class_interfaces_26:
    .quad 0
.globl _class_json_pname_26_2
_class_json_pname_26_2:
    .ascii "name"
    .p2align 3
.globl _class_json_desc_26
_class_json_desc_26:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_26_2
    .quad 4
    .quad 2
    .quad 1
    .p2align 3
.globl _class_gc_desc_26
_class_gc_desc_26:
    .byte 1, 4, 1
    .p2align 3
.globl _class_vtable_26
_class_vtable_26:
    .quad _method_ReflectionClass__u__u_construct
    .quad _method_ReflectionClass_getname
    .quad _method_ReflectionClass_getattributes
    .quad _method_ReflectionClass_newinstancewithoutconstructor
    .quad _method_ReflectionClass_getproperty
    .p2align 3
.globl _class_static_vtable_26
_class_static_vtable_26:
    .quad 0
.globl _class_callable_method_name_26__u__u_construct
_class_callable_method_name_26__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_26_getattributes
_class_callable_method_name_26_getattributes:
    .ascii "getattributes"
.globl _class_callable_method_name_26_getname
_class_callable_method_name_26_getname:
    .ascii "getname"
.globl _class_callable_method_name_26_getproperty
_class_callable_method_name_26_getproperty:
    .ascii "getproperty"
.globl _class_callable_method_name_26_newinstancewithoutconstructor
_class_callable_method_name_26_newinstancewithoutconstructor:
    .ascii "newinstancewithoutconstructor"
.p2align 3
.globl _class_callable_methods_26
_class_callable_methods_26:
    .quad 5
    .quad _class_callable_method_name_26__u__u_construct
    .quad 11
    .quad _class_callable_method_name_26_getattributes
    .quad 13
    .quad _class_callable_method_name_26_getname
    .quad 7
    .quad _class_callable_method_name_26_getproperty
    .quad 11
    .quad _class_callable_method_name_26_newinstancewithoutconstructor
    .quad 29
.globl _class_interfaces_45
_class_interfaces_45:
    .quad 2
    .quad 10
    .quad _class_interface_impl_45_10
    .quad 14
    .quad _class_interface_impl_45_14
.globl _class_interface_impl_45_10
_class_interface_impl_45_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_45_14
_class_interface_impl_45_14:
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
    .byte 1, 0
    .p2align 3
.globl _class_vtable_45
_class_vtable_45:
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
.globl _class_interfaces_53
_class_interfaces_53:
    .quad 0
.globl _class_json_pname_53_5
_class_json_pname_53_5:
    .ascii "name"
    .p2align 3
.globl _class_json_desc_53
_class_json_desc_53:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_53_5
    .quad 4
    .quad 5
    .quad 1
    .p2align 3
.globl _class_gc_desc_53
_class_gc_desc_53:
    .byte 1, 1, 0, 0, 4, 1
    .p2align 3
.globl _class_vtable_53
_class_vtable_53:
    .quad _method_ReflectionFunction__u__u_construct
    .quad _method_ReflectionFunction_getname
    .quad _method_ReflectionFunction_getshortname
    .quad _method_ReflectionFunction_getnumberofparameters
    .quad _method_ReflectionFunction_getnumberofrequiredparameters
    .quad _method_ReflectionFunction_getparameters
    .quad _method_ReflectionFunction_getclosurethis
    .quad _method_ReflectionFunction_invoke
    .quad _method_ReflectionFunction_getclosurecalledclass
    .p2align 3
.globl _class_static_vtable_53
_class_static_vtable_53:
    .quad 0
.globl _class_callable_method_name_53__u__u_construct
_class_callable_method_name_53__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_53_getclosurecalledclass
_class_callable_method_name_53_getclosurecalledclass:
    .ascii "getclosurecalledclass"
.globl _class_callable_method_name_53_getclosurethis
_class_callable_method_name_53_getclosurethis:
    .ascii "getclosurethis"
.globl _class_callable_method_name_53_getname
_class_callable_method_name_53_getname:
    .ascii "getname"
.globl _class_callable_method_name_53_getnumberofparameters
_class_callable_method_name_53_getnumberofparameters:
    .ascii "getnumberofparameters"
.globl _class_callable_method_name_53_getnumberofrequiredparameters
_class_callable_method_name_53_getnumberofrequiredparameters:
    .ascii "getnumberofrequiredparameters"
.globl _class_callable_method_name_53_getparameters
_class_callable_method_name_53_getparameters:
    .ascii "getparameters"
.globl _class_callable_method_name_53_getshortname
_class_callable_method_name_53_getshortname:
    .ascii "getshortname"
.globl _class_callable_method_name_53_invoke
_class_callable_method_name_53_invoke:
    .ascii "invoke"
.p2align 3
.globl _class_callable_methods_53
_class_callable_methods_53:
    .quad 9
    .quad _class_callable_method_name_53__u__u_construct
    .quad 11
    .quad _class_callable_method_name_53_getclosurecalledclass
    .quad 21
    .quad _class_callable_method_name_53_getclosurethis
    .quad 14
    .quad _class_callable_method_name_53_getname
    .quad 7
    .quad _class_callable_method_name_53_getnumberofparameters
    .quad 21
    .quad _class_callable_method_name_53_getnumberofrequiredparameters
    .quad 29
    .quad _class_callable_method_name_53_getparameters
    .quad 13
    .quad _class_callable_method_name_53_getshortname
    .quad 12
    .quad _class_callable_method_name_53_invoke
    .quad 6
.globl _class_interfaces_59
_class_interfaces_59:
    .quad 0
.globl _class_json_pname_59_2
_class_json_pname_59_2:
    .ascii "name"
.globl _class_json_pname_59_3
_class_json_pname_59_3:
    .ascii "class"
    .p2align 3
.globl _class_json_desc_59
_class_json_desc_59:
    .quad 0
    .quad 0
    .quad 2
    .quad _class_json_pname_59_2
    .quad 4
    .quad 2
    .quad 1
    .quad _class_json_pname_59_3
    .quad 5
    .quad 3
    .quad 1
    .p2align 3
.globl _class_gc_desc_59
_class_gc_desc_59:
    .byte 4, 1, 1, 1
    .p2align 3
.globl _class_vtable_59
_class_vtable_59:
    .quad _method_ReflectionProperty__u__u_construct
    .quad _method_ReflectionProperty_getattributes
    .quad _method_ReflectionProperty_gettype
    .quad _method_ReflectionProperty_getname
    .quad _method_ReflectionProperty_getdeclaringfunction
    .quad _method_ReflectionProperty_setvalue
    .quad _method_ReflectionProperty_getdefaultvalue
    .quad _method_ReflectionProperty_hasdefaultvalue
    .quad _method_ReflectionProperty_isdefaultvalueavailable
    .quad _method_ReflectionProperty_getdeclaringclass
    .p2align 3
.globl _class_static_vtable_59
_class_static_vtable_59:
    .quad 0
.globl _class_callable_method_name_59__u__u_construct
_class_callable_method_name_59__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_59_getattributes
_class_callable_method_name_59_getattributes:
    .ascii "getattributes"
.globl _class_callable_method_name_59_getdeclaringclass
_class_callable_method_name_59_getdeclaringclass:
    .ascii "getdeclaringclass"
.globl _class_callable_method_name_59_getdeclaringfunction
_class_callable_method_name_59_getdeclaringfunction:
    .ascii "getdeclaringfunction"
.globl _class_callable_method_name_59_getdefaultvalue
_class_callable_method_name_59_getdefaultvalue:
    .ascii "getdefaultvalue"
.globl _class_callable_method_name_59_getname
_class_callable_method_name_59_getname:
    .ascii "getname"
.globl _class_callable_method_name_59_gettype
_class_callable_method_name_59_gettype:
    .ascii "gettype"
.globl _class_callable_method_name_59_hasdefaultvalue
_class_callable_method_name_59_hasdefaultvalue:
    .ascii "hasdefaultvalue"
.globl _class_callable_method_name_59_isdefaultvalueavailable
_class_callable_method_name_59_isdefaultvalueavailable:
    .ascii "isdefaultvalueavailable"
.globl _class_callable_method_name_59_setvalue
_class_callable_method_name_59_setvalue:
    .ascii "setvalue"
.p2align 3
.globl _class_callable_methods_59
_class_callable_methods_59:
    .quad 10
    .quad _class_callable_method_name_59__u__u_construct
    .quad 11
    .quad _class_callable_method_name_59_getattributes
    .quad 13
    .quad _class_callable_method_name_59_getdeclaringclass
    .quad 17
    .quad _class_callable_method_name_59_getdeclaringfunction
    .quad 20
    .quad _class_callable_method_name_59_getdefaultvalue
    .quad 15
    .quad _class_callable_method_name_59_getname
    .quad 7
    .quad _class_callable_method_name_59_gettype
    .quad 7
    .quad _class_callable_method_name_59_hasdefaultvalue
    .quad 15
    .quad _class_callable_method_name_59_isdefaultvalueavailable
    .quad 23
    .quad _class_callable_method_name_59_setvalue
    .quad 8
.globl _class_interfaces_60
_class_interfaces_60:
    .quad 0
.globl _class_json_pname_60_6
_class_json_pname_60_6:
    .ascii "name"
    .p2align 3
.globl _class_json_desc_60
_class_json_desc_60:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_60_6
    .quad 4
    .quad 6
    .quad 1
    .p2align 3
.globl _class_gc_desc_60
_class_gc_desc_60:
    .byte 1, 0, 3, 3, 3, 7, 1
    .p2align 3
.globl _class_vtable_60
_class_vtable_60:
    .quad _method_ReflectionParameter_getname
    .quad _method_ReflectionParameter_getposition
    .quad _method_ReflectionParameter_isoptional
    .quad _method_ReflectionParameter_isvariadic
    .quad _method_ReflectionParameter_hastype
    .quad _method_ReflectionParameter_gettype
    .quad _method_ReflectionParameter_getdeclaringfunction
    .quad _method_ReflectionParameter_isdefaultvalueavailable
    .quad _method_ReflectionParameter_hasdefaultvalue
    .quad _method_ReflectionParameter_getdefaultvalue
    .quad _method_ReflectionParameter_getdeclaringclass
    .p2align 3
.globl _class_static_vtable_60
_class_static_vtable_60:
    .quad 0
.globl _class_callable_method_name_60_getdeclaringclass
_class_callable_method_name_60_getdeclaringclass:
    .ascii "getdeclaringclass"
.globl _class_callable_method_name_60_getdeclaringfunction
_class_callable_method_name_60_getdeclaringfunction:
    .ascii "getdeclaringfunction"
.globl _class_callable_method_name_60_getdefaultvalue
_class_callable_method_name_60_getdefaultvalue:
    .ascii "getdefaultvalue"
.globl _class_callable_method_name_60_getname
_class_callable_method_name_60_getname:
    .ascii "getname"
.globl _class_callable_method_name_60_getposition
_class_callable_method_name_60_getposition:
    .ascii "getposition"
.globl _class_callable_method_name_60_gettype
_class_callable_method_name_60_gettype:
    .ascii "gettype"
.globl _class_callable_method_name_60_hasdefaultvalue
_class_callable_method_name_60_hasdefaultvalue:
    .ascii "hasdefaultvalue"
.globl _class_callable_method_name_60_hastype
_class_callable_method_name_60_hastype:
    .ascii "hastype"
.globl _class_callable_method_name_60_isdefaultvalueavailable
_class_callable_method_name_60_isdefaultvalueavailable:
    .ascii "isdefaultvalueavailable"
.globl _class_callable_method_name_60_isoptional
_class_callable_method_name_60_isoptional:
    .ascii "isoptional"
.globl _class_callable_method_name_60_isvariadic
_class_callable_method_name_60_isvariadic:
    .ascii "isvariadic"
.p2align 3
.globl _class_callable_methods_60
_class_callable_methods_60:
    .quad 11
    .quad _class_callable_method_name_60_getdeclaringclass
    .quad 17
    .quad _class_callable_method_name_60_getdeclaringfunction
    .quad 20
    .quad _class_callable_method_name_60_getdefaultvalue
    .quad 15
    .quad _class_callable_method_name_60_getname
    .quad 7
    .quad _class_callable_method_name_60_getposition
    .quad 11
    .quad _class_callable_method_name_60_gettype
    .quad 7
    .quad _class_callable_method_name_60_hasdefaultvalue
    .quad 15
    .quad _class_callable_method_name_60_hastype
    .quad 7
    .quad _class_callable_method_name_60_isdefaultvalueavailable
    .quad 23
    .quad _class_callable_method_name_60_isoptional
    .quad 10
    .quad _class_callable_method_name_60_isvariadic
    .quad 10
.globl _class_interfaces_62
_class_interfaces_62:
    .quad 1
    .quad 14
    .quad _class_interface_impl_62_14
.globl _class_interface_impl_62_14
_class_interface_impl_62_14:
    .quad 0
    .p2align 3
.globl _class_json_desc_62
_class_json_desc_62:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_62
_class_gc_desc_62:
    .byte 0
    .p2align 3
.globl _class_vtable_62
_class_vtable_62:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_static_vtable_62
_class_static_vtable_62:
    .quad 0
.globl _class_callable_method_name_62__u__u_tostring
_class_callable_method_name_62__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_62_allowsnull
_class_callable_method_name_62_allowsnull:
    .ascii "allowsnull"
.globl _class_callable_method_name_62_getname
_class_callable_method_name_62_getname:
    .ascii "getname"
.p2align 3
.globl _class_callable_methods_62
_class_callable_methods_62:
    .quad 3
    .quad _class_callable_method_name_62__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_62_allowsnull
    .quad 10
    .quad _class_callable_method_name_62_getname
    .quad 7
.globl _class_interfaces_77
_class_interfaces_77:
    .quad 2
    .quad 10
    .quad _class_interface_impl_77_10
    .quad 14
    .quad _class_interface_impl_77_14
.globl _class_interface_impl_77_10
_class_interface_impl_77_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_77_14
_class_interface_impl_77_14:
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
    .byte 1, 0
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
    .quad 1
    .quad 14
    .quad _class_interface_impl_89_14
.globl _class_interface_impl_89_14
_class_interface_impl_89_14:
    .quad 0
    .p2align 3
.globl _class_json_desc_89
_class_json_desc_89:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_89
_class_gc_desc_89:
    .byte 1, 3, 3
    .p2align 3
.globl _class_vtable_89
_class_vtable_89:
    .quad _method_ReflectionNamedType_allowsnull
    .quad 0
    .quad _method_ReflectionNamedType_getname
    .quad _method_ReflectionNamedType_isbuiltin
    .p2align 3
.globl _class_static_vtable_89
_class_static_vtable_89:
    .quad 0
.globl _class_callable_method_name_89__u__u_tostring
_class_callable_method_name_89__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_89_allowsnull
_class_callable_method_name_89_allowsnull:
    .ascii "allowsnull"
.globl _class_callable_method_name_89_getname
_class_callable_method_name_89_getname:
    .ascii "getname"
.globl _class_callable_method_name_89_isbuiltin
_class_callable_method_name_89_isbuiltin:
    .ascii "isbuiltin"
.p2align 3
.globl _class_callable_methods_89
_class_callable_methods_89:
    .quad 4
    .quad _class_callable_method_name_89__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_89_allowsnull
    .quad 10
    .quad _class_callable_method_name_89_getname
    .quad 7
    .quad _class_callable_method_name_89_isbuiltin
    .quad 9
.globl _class_interfaces_92
_class_interfaces_92:
    .quad 2
    .quad 10
    .quad _class_interface_impl_92_10
    .quad 14
    .quad _class_interface_impl_92_14
.globl _class_interface_impl_92_10
_class_interface_impl_92_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_92_14
_class_interface_impl_92_14:
    .quad 0
.globl _class_json_pname_92_0
_class_json_pname_92_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_92
_class_json_desc_92:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_92_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_92
_class_gc_desc_92:
    .byte 1, 0
    .p2align 3
.globl _class_vtable_92
_class_vtable_92:
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
.globl _class_static_vtable_92
_class_static_vtable_92:
    .quad 0
.globl _class_callable_method_name_92__u__u_construct
_class_callable_method_name_92__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_92__u__u_tostring
_class_callable_method_name_92__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_92_getcode
_class_callable_method_name_92_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_92_getfile
_class_callable_method_name_92_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_92_getline
_class_callable_method_name_92_getline:
    .ascii "getline"
.globl _class_callable_method_name_92_getmessage
_class_callable_method_name_92_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_92_getprevious
_class_callable_method_name_92_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_92_gettrace
_class_callable_method_name_92_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_92_gettraceasstring
_class_callable_method_name_92_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_92
_class_callable_methods_92:
    .quad 9
    .quad _class_callable_method_name_92__u__u_construct
    .quad 11
    .quad _class_callable_method_name_92__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_92_getcode
    .quad 7
    .quad _class_callable_method_name_92_getfile
    .quad 7
    .quad _class_callable_method_name_92_getline
    .quad 7
    .quad _class_callable_method_name_92_getmessage
    .quad 10
    .quad _class_callable_method_name_92_getprevious
    .quad 11
    .quad _class_callable_method_name_92_gettrace
    .quad 8
    .quad _class_callable_method_name_92_gettraceasstring
    .quad 16
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 39

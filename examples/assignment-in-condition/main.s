    ; @fn name=shortName symbol=_fn_shortName
.align 2

.globl _fn_shortName
_fn_shortName:
    ; prologue
    sub sp, sp, #224
    stp x29, x30, [sp, #208]
    add x29, sp, #208
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-200]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-192]
    ; param $class from x0,x1
    stur x0, [x29, #-176]
    stur x1, [x29, #-168]
    stur xzr, [x29, #-184]
    ; @block name=entry
_eir_shortName_entry_0:
    ; @src line=12 col=15 end=12:47 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=9 end=12:14 op=const_bool
    mov x0, #0
    mov x21, x0
    ; @src line=12 col=34 end=12:40 op=load_local
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ; @src line=12 col=42 end=12:46 op=const_str
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=12 col=26 end=12:47 op=runtime_call
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    stp x1, x2, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    mov x3, x1
    mov x4, x2
    ldp x1, x2, [sp], #16
    bl __rt_strrpos
    cmp x0, #0
    b.ge _eir_shortName_strrpos_found_0
    mov x1, #0
    mov x2, #0
    mov x0, #3
    bl __rt_mixed_from_value
    b _eir_shortName_strrpos_done_1
_eir_shortName_strrpos_found_0:
    mov x1, x0
    mov x2, #0
    mov x0, #0
    bl __rt_mixed_from_value
_eir_shortName_strrpos_done_1:
    stur x0, [x29, #-48]
    ; @src line=12 col=24 end=12:47 op=acquire
    ldur x0, [x29, #-48]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-56]
    ; @src line=12 col=24 end=12:47 op=store_local
    ldur x0, [x29, #-56]
    stur x0, [x29, #-184]
    ; @src line=12 col=24 end=12:47 op=release
    ldur x0, [x29, #-48]
    bl __rt_decref_mixed
    ; @src line=12 col=24 end=12:47 op=load_local
    ldur x0, [x29, #-184]
    stur x0, [x29, #-64]
    ; @src line=12 col=15 end=12:47 op=strict_not_eq
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldur x0, [x29, #-64]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_strict_eq
    eor x0, x0, #1
    str x0, [sp, #-16]!
    ldr x0, [sp, #32]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_shortName_if_then_2
    b _eir_shortName_if_else_3
    ; @block name=if.merge
_eir_shortName_if_merge_1:
    udf #0
    ; @block name=if.then
_eir_shortName_if_then_2:
    ; @src line=13 col=9 end=13:15 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=23 end=13:29 op=load_local
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    stur x1, [x29, #-88]
    stur x2, [x29, #-80]
    ; @src line=13 col=31 end=13:35 op=load_local
    ldur x0, [x29, #-184]
    stur x0, [x29, #-96]
    ; @src line=13 col=38 end=13:39 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=13 col=36 end=13:39 op=mixed_numeric_binop
    ldur x0, [x29, #-96]
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
    stur x0, [x29, #-112]
    ; @src line=13 col=16 end=13:40 op=runtime_call
    ldur x1, [x29, #-88]
    ldur x2, [x29, #-80]
    stp x1, x2, [sp, #-16]!
    ldur x0, [x29, #-112]
    ldur x0, [x29, #-112]
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    mov x3, #-1
    ldr x0, [sp], #16
    ldp x1, x2, [sp], #16
    cmp x0, #0
    b.ge _eir_shortName_substr_neg_done_2
    add x0, x2, x0
    cmp x0, #0
    csel x0, xzr, x0, lt
_eir_shortName_substr_neg_done_2:
    cmp x0, x2
    csel x0, x2, x0, gt
    add x1, x1, x0
    sub x2, x2, x0
    cmn x3, #1
    b.eq _eir_shortName_substr_len_done_3
    cmp x3, #0
    csel x3, xzr, x3, lt
    cmp x3, x2
    csel x2, x3, x2, lt
_eir_shortName_substr_len_done_3:
    stur x1, [x29, #-128]
    stur x2, [x29, #-120]
    ; @src line=13 col=16 end=13:40 op=release
    ldur x0, [x29, #-112]
    bl __rt_decref_mixed
    ; @src line=13 col=9 end=13:15 op=str_persist
    ldur x1, [x29, #-128]
    ldur x2, [x29, #-120]
    bl __rt_str_persist
    stur x1, [x29, #-144]
    stur x2, [x29, #-136]
    ldur x1, [x29, #-144]
    ldur x2, [x29, #-136]
    stp x1, x2, [sp, #-16]!
    ; epilogue cleanup $pos
    ldur x0, [x29, #-184]
    cbz x0, _eir_shortName_main_refcounted_cleanup_done_4
    bl __rt_decref_mixed
_eir_shortName_main_refcounted_cleanup_done_4:
    ldp x1, x2, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
    ; @block name=if.else
_eir_shortName_if_else_3:
    ; @src line=15 col=5 end=15:11 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=15 col=12 end=15:18 op=load_local
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    stur x1, [x29, #-160]
    stur x2, [x29, #-152]
    ldur x1, [x29, #-160]
    ldur x2, [x29, #-152]
    stp x1, x2, [sp, #-16]!
    ; epilogue cleanup $pos
    ldur x0, [x29, #-184]
    cbz x0, _eir_shortName_main_refcounted_cleanup_done_5
    bl __rt_decref_mixed
_eir_shortName_main_refcounted_cleanup_done_5:
    ldp x1, x2, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
_fn_shortName_epilogue:
    stp x1, x2, [sp, #-16]!
    ; epilogue cleanup $pos
    ldur x0, [x29, #-184]
    cbz x0, _eir_shortName_main_refcounted_cleanup_done_6
    bl __rt_decref_mixed
_eir_shortName_main_refcounted_cleanup_done_6:
    ldp x1, x2, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
    ; @endfn name=shortName
    ; @fn name=_class_propinit_0 symbol=_class_propinit_0 synthetic=1
.align 2

.globl _class_propinit_0
_class_propinit_0:
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
_eir__class_propinit_0_entry_0:
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
_class_propinit_0_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_0
    ; @fn name=_class_propinit_1 symbol=_class_propinit_1 synthetic=1
.align 2

.globl _class_propinit_1
_class_propinit_1:
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
_eir__class_propinit_1_entry_0:
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
_class_propinit_1_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_1
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
_eir__class_propinit_4_entry_0:
    ; @src line=36 col=28 end=36:35 op=nop
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    ; @src line=36 col=28 end=36:35 op=const_str
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_4_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_8_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
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
_class_propinit_8_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-120]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-104]
    ; @block name=entry
_eir__class_propinit_9_entry_0:
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
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
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
_class_propinit_9_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_9
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_26 symbol=_class_propinit_26 synthetic=1
.align 2

.globl _class_propinit_26
_class_propinit_26:
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
_eir__class_propinit_26_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_26_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_26
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_42_entry_0:
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
_class_propinit_42_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_45 symbol=_class_propinit_45 synthetic=1
.align 2

.globl _class_propinit_45
_class_propinit_45:
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
_eir__class_propinit_45_entry_0:
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
_class_propinit_45_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_45
    ; @fn name=_class_propinit_51 symbol=_class_propinit_51 synthetic=1
.align 2

.globl _class_propinit_51
_class_propinit_51:
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
_eir__class_propinit_51_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_51_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_51
    ; @fn name=_class_propinit_52 symbol=_class_propinit_52 synthetic=1
.align 2

.globl _class_propinit_52
_class_propinit_52:
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
_eir__class_propinit_52_entry_0:
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
_class_propinit_52_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_52
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_59 symbol=_class_propinit_59 synthetic=1
.align 2

.globl _class_propinit_59
_class_propinit_59:
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
_eir__class_propinit_59_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_59_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_59
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
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_71_entry_0:
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
_class_propinit_71_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_71
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_81_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_81_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_83 symbol=_class_propinit_83 synthetic=1
.align 2

.globl _class_propinit_83
_class_propinit_83:
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
_eir__class_propinit_83_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_83_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_83
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_90 symbol=_class_propinit_90 synthetic=1
.align 2

.globl _class_propinit_90
_class_propinit_90:
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
_eir__class_propinit_90_entry_0:
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
    adrp x9, _float_4@PAGE
    add x9, x9, _float_4@PAGEOFF
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
_class_propinit_90_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_103 symbol=_class_propinit_103 synthetic=1
.align 2

.globl _class_propinit_103
_class_propinit_103:
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
_eir__class_propinit_103_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_103_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_103
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #1136
    mov x9, sp
    add x9, x9, #1120
    stp x29, x30, [x9]
    add x29, sp, #1120
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #1120
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #1104
    str x21, [x9]
    sub x9, x29, #1112
    str x22, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #1048
    str xzr, [x9]
    sub x9, x29, #1040
    str xzr, [x9]
    sub x9, x29, #1064
    str xzr, [x9]
    sub x9, x29, #1056
    str xzr, [x9]
    sub x9, x29, #1072
    str xzr, [x9]
    sub x9, x29, #1080
    str xzr, [x9]
    sub x9, x29, #1096
    str xzr, [x9]
    sub x9, x29, #1088
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=8 col=1 end=8:9 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=1 end=8:9 op=nop
    ; @src line=18 col=1 end=18:5 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=16 end=18:38 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #18
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=18 col=6 end=18:39 op=call
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    bl _fn_shortName
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=18 col=42 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=18 col=40 end=18:42 op=str_concat
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldur x3, [x29, #-48]
    ldur x4, [x29, #-40]
    bl __rt_concat
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=18 col=40 end=18:42 op=release
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=18 col=1 end=18:5 op=echo_value
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=18 col=1 end=18:5 op=release
    ; @src line=19 col=1 end=19:5 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=16 end=19:28 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #10
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ; @src line=19 col=6 end=19:29 op=call
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    bl _fn_shortName
    stur x1, [x29, #-96]
    stur x2, [x29, #-88]
    ; @src line=19 col=32 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-112]
    stur x2, [x29, #-104]
    ; @src line=19 col=30 end=19:32 op=str_concat
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    ldur x3, [x29, #-112]
    ldur x4, [x29, #-104]
    bl __rt_concat
    stur x1, [x29, #-128]
    stur x2, [x29, #-120]
    ; @src line=19 col=30 end=19:32 op=release
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=19 col=1 end=19:5 op=echo_value
    ldur x1, [x29, #-128]
    ldur x2, [x29, #-120]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=1 end=19:5 op=release
    ; @src line=22 col=1 end=22:3 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=6 end=22:7 op=nop
    ; @src line=22 col=1 end=22:3 op=nop
    ; @src line=23 col=1 end=23:5 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=23 col=6 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #15
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=23 col=28 end=23:29 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=23 col=37 end=23:38 op=const_i64
    mov x0, #5
    mov x22, x0
    ; @src line=23 col=35 end=23:38 op=store_local
    mov x0, x22
    sub x9, x29, #1016
    str x0, [x9]
    ; @src line=23 col=35 end=23:38 op=load_local
    sub x9, x29, #1016
    ldr x0, [x9]
    mov x22, x0
    ; @src line=23 col=30 end=23:38 op=ichecked_add
    mov x0, x21
    mov x10, x22
    mov x1, x10
    bl __rt_int_add_checked
    stur x0, [x29, #-184]
    ; @src line=23 col=25 end=23:38 op=cast
    ldur x0, [x29, #-184]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_0
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_object_0:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_3
    mov x10, #5
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_4
    mov x10, #6
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_5
    mov x10, #7
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_6
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_7
    mov x10, #16
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_8
    mov x10, #17
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_9
    mov x10, #18
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_10
    mov x10, #19
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_11
    mov x10, #20
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_12
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_13
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_14
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_15
    mov x10, #26
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_16
    mov x10, #27
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_17
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_18
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_19
    mov x10, #30
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_20
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_21
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_22
    mov x10, #34
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_23
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_24
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_25
    mov x10, #37
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_26
    mov x10, #38
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_27
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_28
    mov x10, #43
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_29
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_30
    mov x10, #47
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_31
    mov x10, #50
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_32
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_33
    mov x10, #55
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_34
    mov x10, #56
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_35
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_36
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_37
    mov x10, #59
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_38
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_39
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_40
    mov x10, #69
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_41
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_42
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_43
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_44
    mov x10, #74
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_45
    mov x10, #75
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_46
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_47
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_48
    mov x10, #80
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_49
    mov x10, #82
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_50
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_51
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_52
    mov x10, #91
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_53
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_54
    mov x10, #96
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_55
    mov x10, #98
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_56
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_57
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_58
    mov x10, #102
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_59
    mov x10, #103
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_60
    b _eir_main_mixed_string_no_match_1
_eir_main_mixed_string_ReflectionEnumUnitCase_3:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_Exception_4:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_RuntimeException_5:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_OverflowException_6:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_LogicException_7:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_SplFileInfo_8:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DirectoryIterator_9:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_FilesystemIterator_10:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_GlobIterator_11:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_PharFileInfo_12:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_PharData_13:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionFunctionAbstract_14:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionNamedType_15:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_UnexpectedValueException_16:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_Error_17:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateError_18:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateRangeError_19:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateException_20:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateMalformedStringException_21:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_CachingIterator_23:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_RecursiveCachingIterator_24:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateObjectError_25:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_SplFileObject_26:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_SplTempFileObject_27:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_UnderflowException_28:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DomainException_29:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_RecursiveDirectoryIterator_30:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionIntersectionType_31:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionMethod_32:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_OutOfBoundsException_33:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_Phar_34:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateMalformedPeriodStringException_35:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionUnionType_36:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_RangeException_37:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_JsonException_38:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_TypeError_39:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateInvalidTimeZoneException_40:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_BadFunctionCallException_41:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_BadMethodCallException_42:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionClass_43:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionProperty_44:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionObject_45:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_UnhandledMatchError_46:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_FiberError_47:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateUnknownException_48:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_OutOfRangeException_49:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ArithmeticError_50:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_LengthException_51:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateMalformedIntervalStringException_53:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionEnum_54:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionClassConstant_55:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionParameter_56:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ReflectionEnumBackedCase_57:
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
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_DateInvalidOperationException_58:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_InvalidArgumentException_59:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_ValueError_60:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_2
_eir_main_mixed_string_no_match_1:
    mov x0, #2
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_2:
    stur x1, [x29, #-200]
    stur x2, [x29, #-192]
    ; @src line=23 col=25 end=23:38 op=release
    ldur x0, [x29, #-184]
    bl __rt_decref_mixed
    ; @src line=23 col=25 end=23:38 op=str_concat
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]
    ldur x3, [x29, #-200]
    ldur x4, [x29, #-192]
    bl __rt_concat
    stur x1, [x29, #-216]
    stur x2, [x29, #-208]
    ; @src line=23 col=25 end=23:38 op=release
    ldur x1, [x29, #-200]
    ldur x2, [x29, #-192]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=23 col=42 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #12
    stur x1, [x29, #-232]
    stur x2, [x29, #-224]
    ; @src line=23 col=40 end=23:42 op=str_concat
    ldur x1, [x29, #-216]
    ldur x2, [x29, #-208]
    ldur x3, [x29, #-232]
    ldur x4, [x29, #-224]
    bl __rt_concat
    stur x1, [x29, #-248]
    stur x2, [x29, #-240]
    ; @src line=23 col=40 end=23:42 op=release
    ; @src line=23 col=60 end=23:62 op=load_local
    sub x9, x29, #1016
    ldr x0, [x9]
    mov x22, x0
    ; @src line=23 col=58 end=23:62 op=i_to_str
    mov x0, x22
    bl __rt_itoa
    sub x9, x29, #272
    str x1, [x9]
    sub x9, x29, #264
    str x2, [x9]
    ; @src line=23 col=58 end=23:62 op=str_concat
    ldur x1, [x29, #-248]
    ldur x2, [x29, #-240]
    sub x9, x29, #272
    ldr x3, [x9]
    sub x9, x29, #264
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #288
    str x1, [x9]
    sub x9, x29, #280
    str x2, [x9]
    ; @src line=23 col=58 end=23:62 op=release
    ; @src line=23 col=58 end=23:62 op=release
    ; @src line=23 col=65 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #1
    sub x9, x29, #304
    str x1, [x9]
    sub x9, x29, #296
    str x2, [x9]
    ; @src line=23 col=63 end=23:65 op=str_concat
    sub x9, x29, #288
    ldr x1, [x9]
    sub x9, x29, #280
    ldr x2, [x9]
    sub x9, x29, #304
    ldr x3, [x9]
    sub x9, x29, #296
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #320
    str x1, [x9]
    sub x9, x29, #312
    str x2, [x9]
    ; @src line=23 col=63 end=23:65 op=release
    ; @src line=23 col=1 end=23:5 op=echo_value
    sub x9, x29, #320
    ldr x1, [x9]
    sub x9, x29, #312
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=23 col=1 end=23:5 op=release
    ; @src line=25 col=1 end=25:6 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=9 end=25:13 op=nop
    ; @src line=25 col=1 end=25:6 op=nop
    ; @src line=26 col=1 end=26:8 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=20 end=26:25 op=const_bool
    mov x0, #0
    mov x22, x0
    ; @src line=26 col=18 end=26:25 op=store_local
    mov x0, x22
    sub x9, x29, #1024
    str x0, [x9]
    ; @src line=26 col=18 end=26:25 op=nop
    ; @src line=26 col=11 end=26:12 op=nop
    ; @src line=26 col=11 end=26:12 op=const_bool origin=const_fold
    mov x0, #1
    mov x22, x0
    ; @src line=26 col=1 end=26:8 op=store_local
    mov x0, x22
    sub x9, x29, #1032
    str x0, [x9]
    ; @src line=27 col=1 end=27:5 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=6 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #19
    sub x9, x29, #376
    str x1, [x9]
    sub x9, x29, #368
    str x2, [x9]
    ; @src line=27 col=32 end=27:39 op=load_local
    sub x9, x29, #1032
    ldr x0, [x9]
    mov x22, x0
    mov x0, x22
    cbnz x0, _eir_main_ternary_then_1
    b _eir_main_ternary_else_2
    ; @block name=ternary.then
_eir_main_ternary_then_1:
    ; @src line=27 col=42 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #4
    sub x9, x29, #400
    str x1, [x9]
    sub x9, x29, #392
    str x2, [x9]
    ; @src line=27 col=40 end=27:41 op=acquire
    sub x9, x29, #400
    ldr x1, [x9]
    sub x9, x29, #392
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #416
    str x1, [x9]
    sub x9, x29, #408
    str x2, [x9]
    ; @src line=27 col=40 end=27:41 op=store_local
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    sub x9, x29, #1048
    str x1, [x9]
    sub x9, x29, #1040
    str x2, [x9]
    b _eir_main_ternary_merge_3
    ; @block name=ternary.else
_eir_main_ternary_else_2:
    ; @src line=27 col=51 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #5
    sub x9, x29, #432
    str x1, [x9]
    sub x9, x29, #424
    str x2, [x9]
    ; @src line=27 col=40 end=27:41 op=acquire
    sub x9, x29, #432
    ldr x1, [x9]
    sub x9, x29, #424
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #448
    str x1, [x9]
    sub x9, x29, #440
    str x2, [x9]
    ; @src line=27 col=40 end=27:41 op=store_local
    sub x9, x29, #448
    ldr x1, [x9]
    sub x9, x29, #440
    ldr x2, [x9]
    sub x9, x29, #1048
    str x1, [x9]
    sub x9, x29, #1040
    str x2, [x9]
    b _eir_main_ternary_merge_3
    ; @block name=ternary.merge
_eir_main_ternary_merge_3:
    ; @src line=27 col=40 end=27:41 op=load_local
    sub x9, x29, #1048
    ldr x1, [x9]
    sub x9, x29, #1040
    ldr x2, [x9]
    sub x9, x29, #464
    str x1, [x9]
    sub x9, x29, #456
    str x2, [x9]
    ; @src line=27 col=40 end=27:41 op=unset_local
    sub x9, x29, #1048
    str xzr, [x9]
    sub x9, x29, #1040
    str xzr, [x9]
    ; @src line=27 col=29 end=27:41 op=str_concat
    sub x9, x29, #376
    ldr x1, [x9]
    sub x9, x29, #368
    ldr x2, [x9]
    sub x9, x29, #464
    ldr x3, [x9]
    sub x9, x29, #456
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #480
    str x1, [x9]
    sub x9, x29, #472
    str x2, [x9]
    ; @src line=27 col=29 end=27:41 op=release
    sub x9, x29, #464
    ldr x1, [x9]
    sub x9, x29, #456
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=28 col=7 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #15
    sub x9, x29, #496
    str x1, [x9]
    sub x9, x29, #488
    str x2, [x9]
    ; @src line=28 col=5 end=28:7 op=str_concat
    sub x9, x29, #480
    ldr x1, [x9]
    sub x9, x29, #472
    ldr x2, [x9]
    sub x9, x29, #496
    ldr x3, [x9]
    sub x9, x29, #488
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #512
    str x1, [x9]
    sub x9, x29, #504
    str x2, [x9]
    ; @src line=28 col=5 end=28:7 op=release
    ; @src line=28 col=29 end=28:34 op=load_local
    sub x9, x29, #1024
    ldr x0, [x9]
    mov x22, x0
    mov x0, x22
    cbnz x0, _eir_main_ternary_then_4
    b _eir_main_ternary_else_5
    ; @block name=ternary.then
_eir_main_ternary_then_4:
    ; @src line=28 col=37 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #4
    sub x9, x29, #536
    str x1, [x9]
    sub x9, x29, #528
    str x2, [x9]
    ; @src line=28 col=35 end=28:36 op=acquire
    sub x9, x29, #536
    ldr x1, [x9]
    sub x9, x29, #528
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #552
    str x1, [x9]
    sub x9, x29, #544
    str x2, [x9]
    ; @src line=28 col=35 end=28:36 op=store_local
    sub x9, x29, #552
    ldr x1, [x9]
    sub x9, x29, #544
    ldr x2, [x9]
    sub x9, x29, #1064
    str x1, [x9]
    sub x9, x29, #1056
    str x2, [x9]
    b _eir_main_ternary_merge_6
    ; @block name=ternary.else
_eir_main_ternary_else_5:
    ; @src line=28 col=46 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #5
    sub x9, x29, #568
    str x1, [x9]
    sub x9, x29, #560
    str x2, [x9]
    ; @src line=28 col=35 end=28:36 op=acquire
    sub x9, x29, #568
    ldr x1, [x9]
    sub x9, x29, #560
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #584
    str x1, [x9]
    sub x9, x29, #576
    str x2, [x9]
    ; @src line=28 col=35 end=28:36 op=store_local
    sub x9, x29, #584
    ldr x1, [x9]
    sub x9, x29, #576
    ldr x2, [x9]
    sub x9, x29, #1064
    str x1, [x9]
    sub x9, x29, #1056
    str x2, [x9]
    b _eir_main_ternary_merge_6
    ; @block name=ternary.merge
_eir_main_ternary_merge_6:
    ; @src line=28 col=35 end=28:36 op=load_local
    sub x9, x29, #1064
    ldr x1, [x9]
    sub x9, x29, #1056
    ldr x2, [x9]
    sub x9, x29, #600
    str x1, [x9]
    sub x9, x29, #592
    str x2, [x9]
    ; @src line=28 col=35 end=28:36 op=unset_local
    sub x9, x29, #1064
    str xzr, [x9]
    sub x9, x29, #1056
    str xzr, [x9]
    ; @src line=28 col=26 end=28:36 op=str_concat
    sub x9, x29, #512
    ldr x1, [x9]
    sub x9, x29, #504
    ldr x2, [x9]
    sub x9, x29, #600
    ldr x3, [x9]
    sub x9, x29, #592
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #616
    str x1, [x9]
    sub x9, x29, #608
    str x2, [x9]
    ; @src line=28 col=26 end=28:36 op=release
    ; @src line=28 col=26 end=28:36 op=release
    sub x9, x29, #600
    ldr x1, [x9]
    sub x9, x29, #592
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=28 col=57 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #1
    sub x9, x29, #632
    str x1, [x9]
    sub x9, x29, #624
    str x2, [x9]
    ; @src line=28 col=55 end=28:57 op=str_concat
    sub x9, x29, #616
    ldr x1, [x9]
    sub x9, x29, #608
    ldr x2, [x9]
    sub x9, x29, #632
    ldr x3, [x9]
    sub x9, x29, #624
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #648
    str x1, [x9]
    sub x9, x29, #640
    str x2, [x9]
    ; @src line=28 col=55 end=28:57 op=release
    ; @src line=27 col=1 end=27:5 op=echo_value
    sub x9, x29, #648
    ldr x1, [x9]
    sub x9, x29, #640
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=27 col=1 end=27:5 op=release
    ; @src line=34 col=1 end=34:6 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=34 col=1 end=34:6 op=nop
    ; @src line=39 col=1 end=39:3 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=39 col=6 end=39:16 op=object_new
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    mov x10, #4
    str x10, [x0]
    str xzr, [x0, #8]
    str xzr, [x0, #16]
    sub x9, x29, #656
    str x0, [x9]
    sub x9, x29, #656
    ldr x10, [x9]
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
    mov x2, #5
    str x1, [x10, #8]
    str x2, [x10, #16]
    ; @src line=39 col=1 end=39:3 op=acquire
    sub x9, x29, #656
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #664
    str x0, [x9]
    ; @src line=39 col=1 end=39:3 op=store_local
    sub x9, x29, #664
    ldr x0, [x9]
    sub x9, x29, #1072
    str x0, [x9]
    ; @src line=39 col=1 end=39:3 op=release
    sub x9, x29, #656
    ldr x0, [x9]
    bl __rt_decref_object
    ; @src line=40 col=1 end=40:7 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=40 col=8 end=40:10 op=load_local
    sub x9, x29, #1072
    ldr x0, [x9]
    sub x9, x29, #672
    str x0, [x9]
    ; @src line=40 col=10 end=40:12 op=prop_get
    sub x9, x29, #672
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_61
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_61
    sub x9, x29, #672
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_63
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_64
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #84
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_64:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_16@PAGE
    add x9, x9, _str_16@PAGEOFF
    str x9, [x0, #8]
    mov x9, #70
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_63:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    sub x9, x29, #688
    str x1, [x9]
    sub x9, x29, #680
    str x2, [x9]
    b _eir_main_prop_get_done_62
_eir_main_prop_get_null_receiver_61:
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x1, #0xfffe
    movk x1, #0xffff, lsl #16
    movk x1, #0xffff, lsl #32
    movk x1, #0x7fff, lsl #48
    mov x2, #0
    sub x9, x29, #688
    str x1, [x9]
    sub x9, x29, #680
    str x2, [x9]
_eir_main_prop_get_done_62:
    ; @src line=40 col=10 end=40:12 op=acquire
    sub x9, x29, #688
    ldr x1, [x9]
    sub x9, x29, #680
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #704
    str x1, [x9]
    sub x9, x29, #696
    str x2, [x9]
    ; @src line=40 col=10 end=40:12 op=nop
    ; @src line=40 col=1 end=40:18 op=str_len
    sub x9, x29, #704
    ldr x1, [x9]
    sub x9, x29, #696
    ldr x2, [x9]
    mov x0, x2
    mov x22, x0
    ; @src line=40 col=1 end=40:18 op=release
    sub x9, x29, #704
    ldr x1, [x9]
    sub x9, x29, #696
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=40 col=21 end=40:24 op=const_i64
    mov x0, #100
    mov x12, x0
    ; @src line=40 col=19 end=40:24 op=icmp
    mov x0, x22
    mov x10, x12
    cmp x0, x10
    cset x0, gt
    mov x22, x0
    mov x0, x22
    cbnz x0, _eir_main_short_ternary_value_7
    b _eir_main_short_ternary_default_8
    ; @block name=short_ternary.value
_eir_main_short_ternary_value_7:
    ; @src line=40 col=25 end=40:26 op=mixed_box
    mov x0, x22
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    sub x9, x29, #736
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=acquire
    sub x9, x29, #736
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #744
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=store_local
    sub x9, x29, #744
    ldr x0, [x9]
    sub x9, x29, #1080
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=release
    sub x9, x29, #736
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_short_ternary_merge_9
    ; @block name=short_ternary.default
_eir_main_short_ternary_default_8:
    ; @src line=40 col=38 end=40:61 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=40 col=51 end=40:53 op=load_local
    sub x9, x29, #1072
    ldr x0, [x9]
    sub x9, x29, #752
    str x0, [x9]
    ; @src line=40 col=53 end=40:55 op=prop_get
    sub x9, x29, #752
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_65
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_65
    sub x9, x29, #752
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_67
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_68
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #84
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_68:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_16@PAGE
    add x9, x9, _str_16@PAGEOFF
    str x9, [x0, #8]
    mov x9, #70
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_67:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    sub x9, x29, #768
    str x1, [x9]
    sub x9, x29, #760
    str x2, [x9]
    b _eir_main_prop_get_done_66
_eir_main_prop_get_null_receiver_65:
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x1, #0xfffe
    movk x1, #0xffff, lsl #16
    movk x1, #0xffff, lsl #32
    movk x1, #0x7fff, lsl #48
    mov x2, #0
    sub x9, x29, #768
    str x1, [x9]
    sub x9, x29, #760
    str x2, [x9]
_eir_main_prop_get_done_66:
    ; @src line=40 col=53 end=40:55 op=acquire
    sub x9, x29, #768
    ldr x1, [x9]
    sub x9, x29, #760
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #784
    str x1, [x9]
    sub x9, x29, #776
    str x2, [x9]
    ; @src line=40 col=53 end=40:55 op=nop
    ; @src line=40 col=40 end=40:61 op=runtime_call
    sub x9, x29, #784
    ldr x1, [x9]
    sub x9, x29, #776
    ldr x2, [x9]
    bl __rt_strtoupper
    sub x9, x29, #800
    str x1, [x9]
    sub x9, x29, #792
    str x2, [x9]
    ; @src line=40 col=40 end=40:61 op=release
    sub x9, x29, #784
    ldr x1, [x9]
    sub x9, x29, #776
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=40 col=38 end=40:61 op=acquire
    sub x9, x29, #800
    ldr x1, [x9]
    sub x9, x29, #792
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #816
    str x1, [x9]
    sub x9, x29, #808
    str x2, [x9]
    ; @src line=40 col=38 end=40:61 op=store_local
    sub x9, x29, #816
    ldr x1, [x9]
    sub x9, x29, #808
    ldr x2, [x9]
    sub x9, x29, #1096
    str x1, [x9]
    sub x9, x29, #1088
    str x2, [x9]
    ; @src line=40 col=38 end=40:61 op=release
    sub x9, x29, #800
    ldr x1, [x9]
    sub x9, x29, #792
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=40 col=38 end=40:61 op=load_local
    sub x9, x29, #1096
    ldr x1, [x9]
    sub x9, x29, #1088
    ldr x2, [x9]
    sub x9, x29, #832
    str x1, [x9]
    sub x9, x29, #824
    str x2, [x9]
    ; @src line=40 col=38 end=40:61 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=40 col=28 end=40:30 op=load_local
    sub x9, x29, #1072
    ldr x0, [x9]
    sub x9, x29, #840
    str x0, [x9]
    ; @src line=40 col=38 end=40:61 op=load_local
    sub x9, x29, #1096
    ldr x1, [x9]
    sub x9, x29, #1088
    ldr x2, [x9]
    sub x9, x29, #856
    str x1, [x9]
    sub x9, x29, #848
    str x2, [x9]
    ; @src line=40 col=38 end=40:61 op=prop_set
    sub x9, x29, #840
    ldr x9, [x9]
    str x9, [sp, #-16]!
    sub x9, x29, #856
    ldr x1, [x9]
    sub x9, x29, #848
    ldr x2, [x9]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ; @src line=40 col=38 end=40:61 op=load_local
    sub x9, x29, #1096
    ldr x1, [x9]
    sub x9, x29, #1088
    ldr x2, [x9]
    sub x9, x29, #872
    str x1, [x9]
    sub x9, x29, #864
    str x2, [x9]
    ; @src line=40 col=25 end=40:26 op=mixed_box
    sub x9, x29, #872
    ldr x1, [x9]
    sub x9, x29, #864
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    sub x9, x29, #880
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=acquire
    sub x9, x29, #880
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #888
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=store_local
    sub x9, x29, #888
    ldr x0, [x9]
    sub x9, x29, #1080
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=release
    sub x9, x29, #880
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_short_ternary_merge_9
    ; @block name=short_ternary.merge
_eir_main_short_ternary_merge_9:
    ; @src line=40 col=25 end=40:26 op=load_local
    sub x9, x29, #1080
    ldr x0, [x9]
    sub x9, x29, #896
    str x0, [x9]
    ; @src line=40 col=25 end=40:26 op=unset_local
    sub x9, x29, #1080
    str xzr, [x9]
    ; @src line=40 col=25 end=40:26 op=release
    sub x9, x29, #896
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=41 col=1 end=41:5 op=concat_reset
    sub x9, x29, #1120
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=41 col=6 op=const_str
    adrp x1, _str_18@PAGE
    add x1, x1, _str_18@PAGEOFF
    mov x2, #19
    sub x9, x29, #912
    str x1, [x9]
    sub x9, x29, #904
    str x2, [x9]
    ; @src line=41 col=30 end=41:32 op=load_local
    sub x9, x29, #1072
    ldr x0, [x9]
    sub x9, x29, #920
    str x0, [x9]
    ; @src line=41 col=32 end=41:34 op=prop_get
    sub x9, x29, #920
    ldr x9, [x9]
    cbz x9, _eir_main_prop_get_null_receiver_69
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir_main_prop_get_null_receiver_69
    sub x9, x29, #920
    ldr x9, [x9]
    ldr x10, [x9, #16]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_main_typed_prop_initialized_71
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_main_typed_property_throw_72
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #84
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_typed_property_throw_72:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_16@PAGE
    add x9, x9, _str_16@PAGEOFF
    str x9, [x0, #8]
    mov x9, #70
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_main_typed_prop_initialized_71:
    ldr x1, [x9, #8]
    ldr x2, [x9, #16]
    sub x9, x29, #936
    str x1, [x9]
    sub x9, x29, #928
    str x2, [x9]
    b _eir_main_prop_get_done_70
_eir_main_prop_get_null_receiver_69:
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #50
    bl __rt_diag_warning
    movz x1, #0xfffe
    movk x1, #0xffff, lsl #16
    movk x1, #0xffff, lsl #32
    movk x1, #0x7fff, lsl #48
    mov x2, #0
    sub x9, x29, #936
    str x1, [x9]
    sub x9, x29, #928
    str x2, [x9]
_eir_main_prop_get_done_70:
    ; @src line=41 col=32 end=41:34 op=acquire
    sub x9, x29, #936
    ldr x1, [x9]
    sub x9, x29, #928
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #952
    str x1, [x9]
    sub x9, x29, #944
    str x2, [x9]
    ; @src line=41 col=32 end=41:34 op=nop
    ; @src line=41 col=28 end=41:34 op=str_concat
    sub x9, x29, #912
    ldr x1, [x9]
    sub x9, x29, #904
    ldr x2, [x9]
    sub x9, x29, #952
    ldr x3, [x9]
    sub x9, x29, #944
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #968
    str x1, [x9]
    sub x9, x29, #960
    str x2, [x9]
    ; @src line=41 col=28 end=41:34 op=release
    sub x9, x29, #952
    ldr x1, [x9]
    sub x9, x29, #944
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=41 col=42 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #1
    sub x9, x29, #984
    str x1, [x9]
    sub x9, x29, #976
    str x2, [x9]
    ; @src line=41 col=40 end=41:42 op=str_concat
    sub x9, x29, #968
    ldr x1, [x9]
    sub x9, x29, #960
    ldr x2, [x9]
    sub x9, x29, #984
    ldr x3, [x9]
    sub x9, x29, #976
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1000
    str x1, [x9]
    sub x9, x29, #992
    str x2, [x9]
    ; @src line=41 col=40 end=41:42 op=release
    ; @src line=41 col=1 end=41:5 op=echo_value
    sub x9, x29, #1000
    ldr x1, [x9]
    sub x9, x29, #992
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=41 col=1 end=41:5 op=release

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $__eir_tmp0
    sub x9, x29, #1048
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $__eir_tmp1
    sub x9, x29, #1064
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $t
    sub x9, x29, #1072
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_73
    bl __rt_decref_object
_eir_main_main_refcounted_cleanup_done_73:
    ; epilogue cleanup $__eir_tmp2
    sub x9, x29, #1080
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_74
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_74:
    ; epilogue cleanup $__elephc_assign_expr_40_38_0
    sub x9, x29, #1096
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #1104
    ldr x21, [x9]
    sub x9, x29, #1112
    ldr x22, [x9]
    mov x9, sp
    add x9, x9, #1120
    ldp x29, x30, [x9]
    add sp, sp, #1136
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_75
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_75:
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
    .ascii "\\"
.globl _str_1
_str_1:
    .ascii "hello"
.globl _str_2
_str_2:
    .ascii ""
.globl _str_3
_str_3:
    .ascii "UTC"
.globl _str_5
_str_5:
    .ascii "App\\Service\\Mailer"
.globl _str_6
_str_6:
    .ascii "\n"
.globl _str_7
_str_7:
    .ascii "Stringable"
.globl _str_8
_str_8:
    .ascii "1 + ($n = 5) = "
.globl _str_9
_str_9:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_10
_str_10:
    .ascii ", $n is now "
.globl _str_11
_str_11:
    .ascii "!($flag = false) = "
.globl _str_12
_str_12:
    .ascii "true"
.globl _str_13
_str_13:
    .ascii "false"
.globl _str_14
_str_14:
    .ascii ", $flag is now "
.globl _str_15
_str_15:
    .ascii "Fatal error: Typed property Text::$value must not be accessed before initialization\n"
.globl _str_16
_str_16:
    .ascii "Typed property Text::$value must not be accessed before initialization"
.globl _str_17
_str_17:
    .ascii "Warning: Attempt to read property \"value\" on null\n"
.globl _str_18
_str_18:
    .ascii "normalised value = "
.p2align 3
.globl _float_4
_float_4:
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
.globl _callable_user_fn_name_1
_callable_user_fn_name_1:
    .ascii "shortName"
.p2align 3
.globl _callable_user_function_count
_callable_user_function_count:
    .quad 2
.globl _callable_user_function_table
_callable_user_function_table:
    .quad _callable_user_fn_name_0
    .quad 4
    .quad 0
    .quad _callable_user_fn_name_1
    .quad 9
    .quad 0
.p2align 3
.globl _instanceof_target_count
_instanceof_target_count:
    .quad 34
.globl _instanceof_target_entries
_instanceof_target_entries:
    .quad _instanceof_name_class_4
    .quad 4
    .quad 4
    .quad 0
    .quad _instanceof_name_class_abs_4
    .quad 5
    .quad 4
    .quad 0
    .quad _instanceof_name_class_5
    .quad 9
    .quad 5
    .quad 0
    .quad _instanceof_name_class_abs_5
    .quad 10
    .quad 5
    .quad 0
    .quad _instanceof_name_class_6
    .quad 16
    .quad 6
    .quad 0
    .quad _instanceof_name_class_abs_6
    .quad 17
    .quad 6
    .quad 0
    .quad _instanceof_name_class_12
    .quad 14
    .quad 12
    .quad 0
    .quad _instanceof_name_class_abs_12
    .quad 15
    .quad 12
    .quad 0
    .quad _instanceof_name_class_27
    .quad 5
    .quad 27
    .quad 0
    .quad _instanceof_name_class_abs_27
    .quad 6
    .quad 27
    .quad 0
    .quad _instanceof_name_class_54
    .quad 20
    .quad 54
    .quad 0
    .quad _instanceof_name_class_abs_54
    .quad 21
    .quad 54
    .quad 0
    .quad _instanceof_name_class_59
    .quad 13
    .quad 59
    .quad 0
    .quad _instanceof_name_class_abs_59
    .quad 14
    .quad 59
    .quad 0
    .quad _instanceof_name_class_66
    .quad 9
    .quad 66
    .quad 0
    .quad _instanceof_name_class_abs_66
    .quad 10
    .quad 66
    .quad 0
    .quad _instanceof_name_class_75
    .quad 19
    .quad 75
    .quad 0
    .quad _instanceof_name_class_abs_75
    .quad 20
    .quad 75
    .quad 0
    .quad _instanceof_name_class_80
    .quad 19
    .quad 80
    .quad 0
    .quad _instanceof_name_class_abs_80
    .quad 20
    .quad 80
    .quad 0
    .quad _instanceof_name_class_82
    .quad 15
    .quad 82
    .quad 0
    .quad _instanceof_name_class_abs_82
    .quad 16
    .quad 82
    .quad 0
    .quad _instanceof_name_class_85
    .quad 19
    .quad 85
    .quad 0
    .quad _instanceof_name_class_abs_85
    .quad 20
    .quad 85
    .quad 0
    .quad _instanceof_name_class_93
    .quad 8
    .quad 93
    .quad 0
    .quad _instanceof_name_class_abs_93
    .quad 9
    .quad 93
    .quad 0
    .quad _instanceof_name_class_102
    .quad 24
    .quad 102
    .quad 0
    .quad _instanceof_name_class_abs_102
    .quad 25
    .quad 102
    .quad 0
    .quad _instanceof_name_class_103
    .quad 10
    .quad 103
    .quad 0
    .quad _instanceof_name_class_abs_103
    .quad 11
    .quad 103
    .quad 0
    .quad _instanceof_name_interface_8
    .quad 10
    .quad 8
    .quad 1
    .quad _instanceof_name_interface_abs_8
    .quad 11
    .quad 8
    .quad 1
    .quad _instanceof_name_interface_10
    .quad 9
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_abs_10
    .quad 10
    .quad 10
    .quad 1
.globl _instanceof_name_class_4
_instanceof_name_class_4:
    .ascii "Text"
.globl _instanceof_name_class_abs_4
_instanceof_name_class_abs_4:
    .ascii "\\Text"
.globl _instanceof_name_class_5
_instanceof_name_class_5:
    .ascii "Exception"
.globl _instanceof_name_class_abs_5
_instanceof_name_class_abs_5:
    .ascii "\\Exception"
.globl _instanceof_name_class_6
_instanceof_name_class_6:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_6
_instanceof_name_class_abs_6:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_12
_instanceof_name_class_12:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_12
_instanceof_name_class_abs_12:
    .ascii "\\LogicException"
.globl _instanceof_name_class_27
_instanceof_name_class_27:
    .ascii "Error"
.globl _instanceof_name_class_abs_27
_instanceof_name_class_abs_27:
    .ascii "\\Error"
.globl _instanceof_name_class_54
_instanceof_name_class_54:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_54
_instanceof_name_class_abs_54:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_59
_instanceof_name_class_59:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_59
_instanceof_name_class_abs_59:
    .ascii "\\JsonException"
.globl _instanceof_name_class_66
_instanceof_name_class_66:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_66
_instanceof_name_class_abs_66:
    .ascii "\\TypeError"
.globl _instanceof_name_class_75
_instanceof_name_class_75:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_75
_instanceof_name_class_abs_75:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_80
_instanceof_name_class_80:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_80
_instanceof_name_class_abs_80:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_82
_instanceof_name_class_82:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_82
_instanceof_name_class_abs_82:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_85
_instanceof_name_class_85:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_85
_instanceof_name_class_abs_85:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_93
_instanceof_name_class_93:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_93
_instanceof_name_class_abs_93:
    .ascii "\\stdClass"
.globl _instanceof_name_class_102
_instanceof_name_class_102:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_102
_instanceof_name_class_abs_102:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_103
_instanceof_name_class_103:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_103
_instanceof_name_class_abs_103:
    .ascii "\\ValueError"
.globl _instanceof_name_interface_8
_instanceof_name_interface_8:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_8
_instanceof_name_interface_abs_8:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_10
_instanceof_name_interface_10:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_10
_instanceof_name_interface_abs_10:
    .ascii "\\Throwable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 104
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
    .quad _class_name_4
    .quad 4
    .quad _class_name_5
    .quad 9
    .quad _class_name_6
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
    .quad _class_name_12
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
    .quad _class_name_27
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
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
    .quad 20
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_59
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
    .quad _class_name_66
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
    .quad _class_name_75
    .quad 19
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_82
    .quad 15
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_85
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
    .quad _class_name_93
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
    .quad _class_name_102
    .quad 24
    .quad _class_name_103
    .quad 10
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_4
_class_name_4:
    .ascii "Text"
.globl _class_name_5
_class_name_5:
    .ascii "Exception"
.globl _class_name_6
_class_name_6:
    .ascii "RuntimeException"
.globl _class_name_12
_class_name_12:
    .ascii "LogicException"
.globl _class_name_27
_class_name_27:
    .ascii "Error"
.globl _class_name_54
_class_name_54:
    .ascii "OutOfBoundsException"
.globl _class_name_59
_class_name_59:
    .ascii "JsonException"
.globl _class_name_66
_class_name_66:
    .ascii "TypeError"
.globl _class_name_75
_class_name_75:
    .ascii "UnhandledMatchError"
.globl _class_name_80
_class_name_80:
    .ascii "OutOfRangeException"
.globl _class_name_82
_class_name_82:
    .ascii "ArithmeticError"
.globl _class_name_85
_class_name_85:
    .ascii "ReflectionException"
.globl _class_name_93
_class_name_93:
    .ascii "stdClass"
.globl _class_name_102
_class_name_102:
    .ascii "InvalidArgumentException"
.globl _class_name_103
_class_name_103:
    .ascii "ValueError"
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
    .quad 32
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 76
.globl _generator_class_id
_generator_class_id:
    .quad 94
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 10
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 11
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 23
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 101
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 27
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 12
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 6
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 80
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 54
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 102
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 66
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 103
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 85
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 82
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_8
    .quad _interface_methods_10
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_4
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_27
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
    .quad _class_interfaces_54
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_59
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
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
    .quad _class_interfaces_75
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_80
    .quad _class_interfaces_missing
    .quad _class_interfaces_82
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_85
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_93
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_102
    .quad _class_interfaces_103
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_4
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_27
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
    .quad _class_json_desc_54
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_59
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
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
    .quad _class_json_desc_75
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_80
    .quad _class_json_desc_missing
    .quad _class_json_desc_82
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_85
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_93
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_102
    .quad _class_json_desc_103
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 59
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
    .quad -1
    .quad -1
    .quad -1
    .quad 6
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
    .quad 27
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 27
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 12
    .quad -1
    .quad 27
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 12
    .quad 27
.globl _class_object_payload_sizes
_class_object_payload_sizes:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 24
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
    .quad 72
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
    .quad 16
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
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 104
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_4
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_27
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
    .quad _class_gc_desc_54
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_59
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
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
    .quad _class_gc_desc_75
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_80
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_82
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_85
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_93
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_102
    .quad _class_gc_desc_103
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_4
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
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_27
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
    .quad _class_vtable_54
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_59
    .quad _class_vtable_missing
    .quad _class_vtable_missing
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
    .quad _class_vtable_75
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_80
    .quad _class_vtable_missing
    .quad _class_vtable_82
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_85
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_93
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_102
    .quad _class_vtable_103
.globl _class_destruct_count
_class_destruct_count:
    .quad 104
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
    .quad 0
.globl _class_clone_count
_class_clone_count:
    .quad 104
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
    .quad 0
.globl _class_propinit_ptrs
_class_propinit_ptrs:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_4
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_27
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_propinit_59
    .quad 0
    .quad 0
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
    .quad _class_propinit_75
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_80
    .quad 0
    .quad _class_propinit_82
    .quad 0
    .quad 0
    .quad _class_propinit_85
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_propinit_103
.globl _class_serprop_ptrs
_class_serprop_ptrs:
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_4
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
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_27
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
    .quad _class_serprop_54
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_59
    .quad _class_serprop_missing
    .quad _class_serprop_missing
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
    .quad _class_serprop_75
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_80
    .quad _class_serprop_missing
    .quad _class_serprop_82
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_85
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_93
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_102
    .quad _class_serprop_103
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_4
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_27
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
    .quad _class_static_vtable_54
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_59
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
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
    .quad _class_static_vtable_75
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_80
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_82
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_85
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_93
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_102
    .quad _class_static_vtable_103
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_4
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_27
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
    .quad _class_callable_methods_54
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_59
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
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
    .quad _class_callable_methods_75
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_80
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_82
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_85
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_93
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_102
    .quad _class_callable_methods_103
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
.globl _class_by_name_str_4
_class_by_name_str_4:
    .ascii "Text"
.globl _class_by_name_str_5
_class_by_name_str_5:
    .ascii "Exception"
.globl _class_by_name_str_6
_class_by_name_str_6:
    .ascii "RuntimeException"
.globl _class_by_name_str_12
_class_by_name_str_12:
    .ascii "LogicException"
.globl _class_by_name_str_27
_class_by_name_str_27:
    .ascii "Error"
.globl _class_by_name_str_54
_class_by_name_str_54:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_59
_class_by_name_str_59:
    .ascii "JsonException"
.globl _class_by_name_str_66
_class_by_name_str_66:
    .ascii "TypeError"
.globl _class_by_name_str_75
_class_by_name_str_75:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_80
_class_by_name_str_80:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_82
_class_by_name_str_82:
    .ascii "ArithmeticError"
.globl _class_by_name_str_85
_class_by_name_str_85:
    .ascii "ReflectionException"
.globl _class_by_name_str_93
_class_by_name_str_93:
    .ascii "stdClass"
.globl _class_by_name_str_102
_class_by_name_str_102:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_103
_class_by_name_str_103:
    .ascii "ValueError"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 15
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_4
    .quad 4
    .quad 4
    .quad 24
    .quad _class_by_name_str_5
    .quad 9
    .quad 5
    .quad 72
    .quad _class_by_name_str_6
    .quad 16
    .quad 6
    .quad 72
    .quad _class_by_name_str_12
    .quad 14
    .quad 12
    .quad 72
    .quad _class_by_name_str_27
    .quad 5
    .quad 27
    .quad 72
    .quad _class_by_name_str_54
    .quad 20
    .quad 54
    .quad 72
    .quad _class_by_name_str_59
    .quad 13
    .quad 59
    .quad 72
    .quad _class_by_name_str_66
    .quad 9
    .quad 66
    .quad 72
    .quad _class_by_name_str_75
    .quad 19
    .quad 75
    .quad 72
    .quad _class_by_name_str_80
    .quad 19
    .quad 80
    .quad 72
    .quad _class_by_name_str_82
    .quad 15
    .quad 82
    .quad 72
    .quad _class_by_name_str_85
    .quad 19
    .quad 85
    .quad 72
    .quad _class_by_name_str_93
    .quad 8
    .quad 93
    .quad 16
    .quad _class_by_name_str_102
    .quad 24
    .quad 102
    .quad 72
    .quad _class_by_name_str_103
    .quad 10
    .quad 103
    .quad 72
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 104
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
    .quad _class_attributes_missing
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_8
_interface_methods_8:
    .quad 1
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
.globl _class_interfaces_4
_class_interfaces_4:
    .quad 0
.globl _class_json_pname_4_0
_class_json_pname_4_0:
    .ascii "value"
    .p2align 3
.globl _class_json_desc_4
_class_json_desc_4:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_4_0
    .quad 5
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_4
_class_gc_desc_4:
    .byte 1
.globl _class_serpname_4_0
_class_serpname_4_0:
    .byte 118, 97, 108, 117, 101
    .p2align 3
.globl _class_serprop_4
_class_serprop_4:
    .quad 1
    .quad _class_serpname_4_0
    .quad 5
    .quad 8
    .quad 1
    .p2align 3
.globl _class_vtable_4
_class_vtable_4:
    .quad 0
    .p2align 3
.globl _class_static_vtable_4
_class_static_vtable_4:
    .quad 0
.p2align 3
.globl _class_callable_methods_4
_class_callable_methods_4:
    .quad 0
.globl _class_interfaces_5
_class_interfaces_5:
    .quad 2
    .quad 10
    .quad _class_interface_impl_5_10
    .quad 8
    .quad _class_interface_impl_5_8
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
.globl _class_interface_impl_5_8
_class_interface_impl_5_8:
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
    .quad 10
    .quad _class_interface_impl_6_10
    .quad 8
    .quad _class_interface_impl_6_8
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
.globl _class_interface_impl_6_8
_class_interface_impl_6_8:
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
    .quad 10
    .quad _class_interface_impl_12_10
    .quad 8
    .quad _class_interface_impl_12_8
.globl _class_interface_impl_12_10
_class_interface_impl_12_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_12_8
_class_interface_impl_12_8:
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
.globl _class_interfaces_27
_class_interfaces_27:
    .quad 2
    .quad 10
    .quad _class_interface_impl_27_10
    .quad 8
    .quad _class_interface_impl_27_8
.globl _class_interface_impl_27_10
_class_interface_impl_27_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_27_8
_class_interface_impl_27_8:
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
.globl _class_interfaces_54
_class_interfaces_54:
    .quad 2
    .quad 10
    .quad _class_interface_impl_54_10
    .quad 8
    .quad _class_interface_impl_54_8
.globl _class_interface_impl_54_10
_class_interface_impl_54_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_54_8
_class_interface_impl_54_8:
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
.globl _class_interfaces_59
_class_interfaces_59:
    .quad 2
    .quad 10
    .quad _class_interface_impl_59_10
    .quad 8
    .quad _class_interface_impl_59_8
.globl _class_interface_impl_59_10
_class_interface_impl_59_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_59_8
_class_interface_impl_59_8:
    .quad 0
.globl _class_json_pname_59_0
_class_json_pname_59_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_59
_class_json_desc_59:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_59_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_59
_class_gc_desc_59:
    .byte 1, 0, 7, 4
.globl _class_serpname_59_0
_class_serpname_59_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_59_1
_class_serpname_59_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_59_2
_class_serpname_59_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_59_3
_class_serpname_59_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_59
_class_serprop_59:
    .quad 4
    .quad _class_serpname_59_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_59_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_59_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_59_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_59
_class_vtable_59:
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
.globl _class_static_vtable_59
_class_static_vtable_59:
    .quad 0
.globl _class_callable_method_name_59__u__u_construct
_class_callable_method_name_59__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_59__u__u_tostring
_class_callable_method_name_59__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_59_getcode
_class_callable_method_name_59_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_59_getfile
_class_callable_method_name_59_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_59_getline
_class_callable_method_name_59_getline:
    .ascii "getline"
.globl _class_callable_method_name_59_getmessage
_class_callable_method_name_59_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_59_getprevious
_class_callable_method_name_59_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_59_gettrace
_class_callable_method_name_59_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_59_gettraceasstring
_class_callable_method_name_59_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_59
_class_callable_methods_59:
    .quad 9
    .quad _class_callable_method_name_59__u__u_construct
    .quad 11
    .quad _class_callable_method_name_59__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_59_getcode
    .quad 7
    .quad _class_callable_method_name_59_getfile
    .quad 7
    .quad _class_callable_method_name_59_getline
    .quad 7
    .quad _class_callable_method_name_59_getmessage
    .quad 10
    .quad _class_callable_method_name_59_getprevious
    .quad 11
    .quad _class_callable_method_name_59_gettrace
    .quad 8
    .quad _class_callable_method_name_59_gettraceasstring
    .quad 16
.globl _class_interfaces_66
_class_interfaces_66:
    .quad 2
    .quad 10
    .quad _class_interface_impl_66_10
    .quad 8
    .quad _class_interface_impl_66_8
.globl _class_interface_impl_66_10
_class_interface_impl_66_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_66_8
_class_interface_impl_66_8:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_66
_class_vtable_66:
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
.globl _class_interfaces_75
_class_interfaces_75:
    .quad 2
    .quad 10
    .quad _class_interface_impl_75_10
    .quad 8
    .quad _class_interface_impl_75_8
.globl _class_interface_impl_75_10
_class_interface_impl_75_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_75_8
_class_interface_impl_75_8:
    .quad 0
.globl _class_json_pname_75_0
_class_json_pname_75_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_75
_class_json_desc_75:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_75_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_75
_class_gc_desc_75:
    .byte 1, 0, 7, 4
.globl _class_serpname_75_0
_class_serpname_75_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_75_1
_class_serpname_75_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_75_2
_class_serpname_75_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_75_3
_class_serpname_75_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_75
_class_serprop_75:
    .quad 4
    .quad _class_serpname_75_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_75_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_75_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_75_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_75
_class_vtable_75:
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
.globl _class_static_vtable_75
_class_static_vtable_75:
    .quad 0
.globl _class_callable_method_name_75__u__u_construct
_class_callable_method_name_75__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_75__u__u_tostring
_class_callable_method_name_75__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_75_getcode
_class_callable_method_name_75_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_75_getfile
_class_callable_method_name_75_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_75_getline
_class_callable_method_name_75_getline:
    .ascii "getline"
.globl _class_callable_method_name_75_getmessage
_class_callable_method_name_75_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_75_getprevious
_class_callable_method_name_75_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_75_gettrace
_class_callable_method_name_75_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_75_gettraceasstring
_class_callable_method_name_75_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_75
_class_callable_methods_75:
    .quad 9
    .quad _class_callable_method_name_75__u__u_construct
    .quad 11
    .quad _class_callable_method_name_75__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_75_getcode
    .quad 7
    .quad _class_callable_method_name_75_getfile
    .quad 7
    .quad _class_callable_method_name_75_getline
    .quad 7
    .quad _class_callable_method_name_75_getmessage
    .quad 10
    .quad _class_callable_method_name_75_getprevious
    .quad 11
    .quad _class_callable_method_name_75_gettrace
    .quad 8
    .quad _class_callable_method_name_75_gettraceasstring
    .quad 16
.globl _class_interfaces_80
_class_interfaces_80:
    .quad 2
    .quad 10
    .quad _class_interface_impl_80_10
    .quad 8
    .quad _class_interface_impl_80_8
.globl _class_interface_impl_80_10
_class_interface_impl_80_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_80_8
_class_interface_impl_80_8:
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
.globl _class_interfaces_82
_class_interfaces_82:
    .quad 2
    .quad 10
    .quad _class_interface_impl_82_10
    .quad 8
    .quad _class_interface_impl_82_8
.globl _class_interface_impl_82_10
_class_interface_impl_82_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_82_8
_class_interface_impl_82_8:
    .quad 0
.globl _class_json_pname_82_0
_class_json_pname_82_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_82
_class_json_desc_82:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_82_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_82
_class_gc_desc_82:
    .byte 1, 0, 7, 4
.globl _class_serpname_82_0
_class_serpname_82_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_82_1
_class_serpname_82_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_82_2
_class_serpname_82_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_82_3
_class_serpname_82_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_82
_class_serprop_82:
    .quad 4
    .quad _class_serpname_82_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_82_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_82_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_82_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_82
_class_vtable_82:
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
.globl _class_static_vtable_82
_class_static_vtable_82:
    .quad 0
.globl _class_callable_method_name_82__u__u_construct
_class_callable_method_name_82__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_82__u__u_tostring
_class_callable_method_name_82__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_82_getcode
_class_callable_method_name_82_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_82_getfile
_class_callable_method_name_82_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_82_getline
_class_callable_method_name_82_getline:
    .ascii "getline"
.globl _class_callable_method_name_82_getmessage
_class_callable_method_name_82_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_82_getprevious
_class_callable_method_name_82_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_82_gettrace
_class_callable_method_name_82_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_82_gettraceasstring
_class_callable_method_name_82_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_82
_class_callable_methods_82:
    .quad 9
    .quad _class_callable_method_name_82__u__u_construct
    .quad 11
    .quad _class_callable_method_name_82__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_82_getcode
    .quad 7
    .quad _class_callable_method_name_82_getfile
    .quad 7
    .quad _class_callable_method_name_82_getline
    .quad 7
    .quad _class_callable_method_name_82_getmessage
    .quad 10
    .quad _class_callable_method_name_82_getprevious
    .quad 11
    .quad _class_callable_method_name_82_gettrace
    .quad 8
    .quad _class_callable_method_name_82_gettraceasstring
    .quad 16
.globl _class_interfaces_85
_class_interfaces_85:
    .quad 2
    .quad 10
    .quad _class_interface_impl_85_10
    .quad 8
    .quad _class_interface_impl_85_8
.globl _class_interface_impl_85_10
_class_interface_impl_85_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_85_8
_class_interface_impl_85_8:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_85
_class_vtable_85:
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
.globl _class_interfaces_93
_class_interfaces_93:
    .quad 0
    .p2align 3
.globl _class_json_desc_93
_class_json_desc_93:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_93
_class_gc_desc_93:
    .byte 0
    .p2align 3
.globl _class_serprop_93
_class_serprop_93:
    .quad 0
    .p2align 3
.globl _class_vtable_93
_class_vtable_93:
    .quad 0
    .p2align 3
.globl _class_static_vtable_93
_class_static_vtable_93:
    .quad 0
.p2align 3
.globl _class_callable_methods_93
_class_callable_methods_93:
    .quad 0
.globl _class_interfaces_102
_class_interfaces_102:
    .quad 2
    .quad 10
    .quad _class_interface_impl_102_10
    .quad 8
    .quad _class_interface_impl_102_8
.globl _class_interface_impl_102_10
_class_interface_impl_102_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_102_8
_class_interface_impl_102_8:
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
.globl _class_interfaces_103
_class_interfaces_103:
    .quad 2
    .quad 10
    .quad _class_interface_impl_103_10
    .quad 8
    .quad _class_interface_impl_103_8
.globl _class_interface_impl_103_10
_class_interface_impl_103_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_103_8
_class_interface_impl_103_8:
    .quad 0
.globl _class_json_pname_103_0
_class_json_pname_103_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_103
_class_json_desc_103:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_103_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_103
_class_gc_desc_103:
    .byte 1, 0, 7, 4
.globl _class_serpname_103_0
_class_serpname_103_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_103_1
_class_serpname_103_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_103_2
_class_serpname_103_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_103_3
_class_serpname_103_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_103
_class_serprop_103:
    .quad 4
    .quad _class_serpname_103_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_103_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_103_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_103_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_103
_class_vtable_103:
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
.globl _class_static_vtable_103
_class_static_vtable_103:
    .quad 0
.globl _class_callable_method_name_103__u__u_construct
_class_callable_method_name_103__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_103__u__u_tostring
_class_callable_method_name_103__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_103_getcode
_class_callable_method_name_103_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_103_getfile
_class_callable_method_name_103_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_103_getline
_class_callable_method_name_103_getline:
    .ascii "getline"
.globl _class_callable_method_name_103_getmessage
_class_callable_method_name_103_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_103_getprevious
_class_callable_method_name_103_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_103_gettrace
_class_callable_method_name_103_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_103_gettraceasstring
_class_callable_method_name_103_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_103
_class_callable_methods_103:
    .quad 9
    .quad _class_callable_method_name_103__u__u_construct
    .quad 11
    .quad _class_callable_method_name_103__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_103_getcode
    .quad 7
    .quad _class_callable_method_name_103_getfile
    .quad 7
    .quad _class_callable_method_name_103_getline
    .quad 7
    .quad _class_callable_method_name_103_getmessage
    .quad 10
    .quad _class_callable_method_name_103_getprevious
    .quad 11
    .quad _class_callable_method_name_103_gettrace
    .quad 8
    .quad _class_callable_method_name_103_gettraceasstring
    .quad 16
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 93

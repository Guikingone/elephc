    ; @fn name=wrap symbol=_fn_wrap
.align 2

.globl _fn_wrap
_fn_wrap:
    ; prologue
    sub sp, sp, #176
    stp x29, x30, [sp, #160]
    add x29, sp, #160
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-152]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-136]
    stur x22, [x29, #-144]
    ; param $t from x0
    stur x0, [x29, #-120]
    ldur x0, [x29, #-120]
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    stur x0, [x29, #-120]
    ; @block name=entry
_eir_wrap_entry_0:
    ; @src line=12 col=5 end=12:6 op=concat_reset
    ldur x10, [x29, #-152]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=5 end=12:6 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=12 col=9 end=12:11 op=load_local
    ldur x0, [x29, #-120]
    bl __rt_mixed_cast_int
    mov x22, x0
    ; @src line=12 col=7 end=12:11 op=icmp
    mov x0, x21
    mov x10, x22
    cmp x0, x10
    cset x0, gt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir_wrap_logical_rhs_1
    b _eir_wrap_logical_const_2
    ; @block name=logical.rhs
_eir_wrap_logical_rhs_1:
    ; @src line=12 col=15 end=12:17 op=load_local
    ldur x0, [x29, #-120]
    bl __rt_mixed_cast_int
    mov x22, x0
    ; @src line=12 col=21 end=12:25 op=const_i64
    mov x0, #64
    mov x21, x0
    ; @src line=12 col=18 end=12:25 op=ichecked_add
    mov x0, x22
    mov x10, x21
    mov x1, x10
    bl __rt_int_add_checked
    stur x0, [x29, #-48]
    ; @src line=12 col=18 end=12:25 op=acquire
    ldur x0, [x29, #-48]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-56]
    ; @src line=12 col=18 end=12:25 op=load_local
    ldur x0, [x29, #-120]
    stur x0, [x29, #-64]
    ; @src line=12 col=18 end=12:25 op=release
    ldur x0, [x29, #-64]
    bl __rt_decref_mixed
    ; @src line=12 col=18 end=12:25 op=store_local
    ldur x0, [x29, #-56]
    stur x0, [x29, #-120]
    ; @src line=12 col=18 end=12:25 op=release
    ldur x0, [x29, #-48]
    bl __rt_decref_mixed
    ; @src line=12 col=18 end=12:25 op=load_local
    ldur x0, [x29, #-120]
    stur x0, [x29, #-72]
    ; @src line=12 col=18 end=12:25 op=is_truthy
    ldur x0, [x29, #-72]
    bl __rt_mixed_cast_bool
    mov x21, x0
    ; @src line=12 col=12 end=12:25 op=store_local
    mov x0, x21
    stur x0, [x29, #-128]
    b _eir_wrap_logical_merge_3
    ; @block name=logical.const
_eir_wrap_logical_const_2:
    ; @src line=12 col=12 end=12:25 op=const_bool
    mov x0, #0
    mov x21, x0
    ; @src line=12 col=12 end=12:25 op=store_local
    mov x0, x21
    stur x0, [x29, #-128]
    b _eir_wrap_logical_merge_3
    ; @block name=logical.merge
_eir_wrap_logical_merge_3:
    ; @src line=12 col=12 end=12:25 op=load_local
    ldur x0, [x29, #-128]
    mov x21, x0
    ; @src line=13 col=5 end=13:11 op=concat_reset
    ldur x10, [x29, #-152]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=12 end=13:14 op=load_local
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    ; @src line=13 col=5 end=13:11 op=cast
    ldur x0, [x29, #-104]
    bl __rt_mixed_cast_int
    mov x21, x0
    mov x0, x21
    str x0, [sp, #-16]!
    ; epilogue cleanup $t
    ldur x0, [x29, #-120]
    cbz x0, _eir_wrap_main_refcounted_cleanup_done_0
    bl __rt_decref_mixed
_eir_wrap_main_refcounted_cleanup_done_0:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-136]
    ldur x22, [x29, #-144]
    ldp x29, x30, [sp, #160]
    add sp, sp, #176
    ret
_fn_wrap_epilogue:
    str x0, [sp, #-16]!
    ; epilogue cleanup $t
    ldur x0, [x29, #-120]
    cbz x0, _eir_wrap_main_refcounted_cleanup_done_1
    bl __rt_decref_mixed
_eir_wrap_main_refcounted_cleanup_done_1:
    ldr x0, [sp], #16
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-136]
    ldur x22, [x29, #-144]
    ldp x29, x30, [sp, #160]
    add sp, sp, #176
    ret
    ; @endfn name=wrap
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
    ; @fn name=_class_propinit_13 symbol=_class_propinit_13 synthetic=1
.align 2

.globl _class_propinit_13
_class_propinit_13:
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
_eir__class_propinit_13_entry_0:
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
_class_propinit_13_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_13
    ; @fn name=_class_propinit_14 symbol=_class_propinit_14 synthetic=1
.align 2

.globl _class_propinit_14
_class_propinit_14:
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
_eir__class_propinit_14_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_14_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_14
    ; @fn name=_class_propinit_15 symbol=_class_propinit_15 synthetic=1
.align 2

.globl _class_propinit_15
_class_propinit_15:
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
_eir__class_propinit_15_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_15_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_15
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
    ; @fn name=_class_propinit_32 symbol=_class_propinit_32 synthetic=1
.align 2

.globl _class_propinit_32
_class_propinit_32:
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
_eir__class_propinit_32_entry_0:
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
_class_propinit_32_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    ret
    ; @endfn name=_class_propinit_32
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
    ; @fn name=_class_propinit_37 symbol=_class_propinit_37 synthetic=1
.align 2

.globl _class_propinit_37
_class_propinit_37:
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
_eir__class_propinit_37_entry_0:
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
_class_propinit_37_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_37
    ; @fn name=_class_propinit_38 symbol=_class_propinit_38 synthetic=1
.align 2

.globl _class_propinit_38
_class_propinit_38:
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
_eir__class_propinit_38_entry_0:
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
_class_propinit_38_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
_class_propinit_45_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_45
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
_class_propinit_46_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_46
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
    ; @fn name=_class_propinit_52 symbol=_class_propinit_52 synthetic=1
.align 2

.globl _class_propinit_52
_class_propinit_52:
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
_eir__class_propinit_52_entry_0:
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
_class_propinit_52_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
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
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_66_entry_0:
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
_class_propinit_66_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_66
    ; @fn name=_class_propinit_67 symbol=_class_propinit_67 synthetic=1
.align 2

.globl _class_propinit_67
_class_propinit_67:
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
_eir__class_propinit_67_entry_0:
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
_class_propinit_67_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_67
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
    ; @fn name=_class_propinit_74 symbol=_class_propinit_74 synthetic=1
.align 2

.globl _class_propinit_74
_class_propinit_74:
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
_eir__class_propinit_74_entry_0:
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
_class_propinit_74_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_74
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
_eir__class_propinit_83_entry_0:
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
_class_propinit_83_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_83
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
_eir__class_propinit_96_entry_0:
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
_class_propinit_96_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_96
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
_class_propinit_99_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    ; @fn name=_class_propinit_103 symbol=_class_propinit_103 synthetic=1
.align 2

.globl _class_propinit_103
_class_propinit_103:
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
_eir__class_propinit_103_entry_0:
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
_class_propinit_103_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_103
    ; @fn name=Banner::__construct symbol=_method_Banner__u__u_construct
.align 2

.globl _method_Banner__u__u_construct
_method_Banner__u__u_construct:
    ; prologue
    sub sp, sp, #128
    stp x29, x30, [sp, #112]
    add x29, sp, #112
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-112]
    ; param $this from x0
    stur x0, [x29, #-88]
    ; param $text from x1,x2
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @block name=entry
_eir_Banner____construct_entry_0:
    ; @src line=29 col=9 end=29:13 op=concat_reset
    ldur x10, [x29, #-112]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=29 col=14 op=const_str
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=29 col=14 op=load_local
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=29 col=14 op=str_concat
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    bl __rt_concat
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=29 col=14 op=const_str
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=29 col=14 op=str_concat
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldur x3, [x29, #-64]
    ldur x4, [x29, #-56]
    bl __rt_concat
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ; @src line=29 col=14 op=release
    ; @src line=29 col=9 end=29:13 op=echo_value
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=29 col=9 end=29:13 op=release
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
_method_Banner__u__u_construct_epilogue:
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
    ; @endfn name=Banner::__construct
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
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
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #384
    str xzr, [x9]
    sub x9, x29, #376
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=10 col=1 end=10:9 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=1 end=10:9 op=nop
    ; @src line=16 col=1 end=16:5 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=16 col=11 end=16:12 op=const_i64
    mov x0, #-5
    mov x21, x0
    ; @src line=16 col=6 end=16:14 op=call
    mov x0, x21
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    bl _fn_wrap
    mov x21, x0
    ; @src line=16 col=15 end=16:17 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=16 col=17 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=16 col=15 end=16:17 op=str_concat
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldur x3, [x29, #-48]
    ldur x4, [x29, #-40]
    bl __rt_concat
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=16 col=15 end=16:17 op=release
    ; @src line=16 col=1 end=16:5 op=echo_value
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=16 col=1 end=16:5 op=release
    ; @src line=17 col=1 end=17:5 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=11 end=17:13 op=const_i64
    mov x0, #10
    mov x21, x0
    ; @src line=17 col=6 end=17:14 op=call
    mov x0, x21
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    bl _fn_wrap
    mov x21, x0
    ; @src line=17 col=15 end=17:17 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    stur x1, [x29, #-96]
    stur x2, [x29, #-88]
    ; @src line=17 col=17 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #1
    stur x1, [x29, #-112]
    stur x2, [x29, #-104]
    ; @src line=17 col=15 end=17:17 op=str_concat
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    ldur x3, [x29, #-112]
    ldur x4, [x29, #-104]
    bl __rt_concat
    stur x1, [x29, #-128]
    stur x2, [x29, #-120]
    ; @src line=17 col=15 end=17:17 op=release
    ; @src line=17 col=1 end=17:5 op=echo_value
    ldur x1, [x29, #-128]
    ldur x2, [x29, #-120]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=17 col=1 end=17:5 op=release
    ; @src line=20 col=1 end=20:6 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=20 col=9 end=20:11 op=const_str
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-144]
    stur x2, [x29, #-136]
    ; @src line=20 col=1 end=20:6 op=acquire
    ldur x1, [x29, #-144]
    ldur x2, [x29, #-136]
    bl __rt_str_persist
    stur x1, [x29, #-160]
    stur x2, [x29, #-152]
    ; @src line=20 col=1 end=20:6 op=store_local
    ldur x1, [x29, #-160]
    ldur x2, [x29, #-152]
    sub x9, x29, #384
    str x1, [x9]
    sub x9, x29, #376
    str x2, [x9]
    ; @src line=21 col=1 end=21:3 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1 end=21:3 op=const_str
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=21 col=8 end=21:13 op=load_local
    sub x9, x29, #384
    ldr x1, [x9]
    sub x9, x29, #376
    ldr x2, [x9]
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ; @src line=21 col=4 end=21:13 op=strict_eq
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    ldur x3, [x29, #-192]
    ldur x4, [x29, #-184]
    bl __rt_str_eq
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_logical_rhs_1
    b _eir_main_logical_const_2
    ; @block name=logical.rhs
_eir_main_logical_rhs_1:
    ; @src line=21 col=25 end=21:36 op=const_str
    adrp x1, _str_6@PAGE
    add x1, x1, _str_6@PAGEOFF
    mov x2, #9
    stur x1, [x29, #-216]
    stur x2, [x29, #-208]
    ; @src line=21 col=23 end=21:36 op=acquire
    ldur x1, [x29, #-216]
    ldur x2, [x29, #-208]
    bl __rt_str_persist
    stur x1, [x29, #-232]
    stur x2, [x29, #-224]
    ; @src line=21 col=23 end=21:36 op=load_local
    sub x9, x29, #384
    ldr x1, [x9]
    sub x9, x29, #376
    ldr x2, [x9]
    stur x1, [x29, #-248]
    stur x2, [x29, #-240]
    ; @src line=21 col=23 end=21:36 op=release
    ldur x1, [x29, #-248]
    ldur x2, [x29, #-240]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=21 col=23 end=21:36 op=store_local
    ldur x1, [x29, #-232]
    ldur x2, [x29, #-224]
    sub x9, x29, #384
    str x1, [x9]
    sub x9, x29, #376
    str x2, [x9]
    ; @src line=21 col=23 end=21:36 op=load_local
    sub x9, x29, #384
    ldr x1, [x9]
    sub x9, x29, #376
    ldr x2, [x9]
    sub x9, x29, #264
    str x1, [x9]
    sub x9, x29, #256
    str x2, [x9]
    ; @src line=21 col=23 end=21:36 op=is_truthy
    sub x9, x29, #264
    ldr x1, [x9]
    sub x9, x29, #256
    ldr x2, [x9]
    mov x0, #0
    cbz x2, _eir_main_str_truthy_done_1
    cmp x2, #1
    b.ne _eir_main_str_truthy_true_0
    ldrb w10, [x1]
    cmp w10, #48
    cset x0, ne
    b _eir_main_str_truthy_done_1
_eir_main_str_truthy_true_0:
    mov x0, #1
_eir_main_str_truthy_done_1:
    mov x21, x0
    ; @src line=21 col=14 end=21:36 op=store_local
    mov x0, x21
    sub x9, x29, #392
    str x0, [x9]
    b _eir_main_logical_merge_3
    ; @block name=logical.const
_eir_main_logical_const_2:
    ; @src line=21 col=14 end=21:36 op=const_bool
    mov x0, #0
    mov x21, x0
    ; @src line=21 col=14 end=21:36 op=store_local
    mov x0, x21
    sub x9, x29, #392
    str x0, [x9]
    b _eir_main_logical_merge_3
    ; @block name=logical.merge
_eir_main_logical_merge_3:
    ; @src line=21 col=14 end=21:36 op=load_local
    sub x9, x29, #392
    ldr x0, [x9]
    mov x21, x0
    ; @src line=22 col=1 end=22:5 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=6 end=22:11 op=load_local
    sub x9, x29, #384
    ldr x1, [x9]
    sub x9, x29, #376
    ldr x2, [x9]
    sub x9, x29, #304
    str x1, [x9]
    sub x9, x29, #296
    str x2, [x9]
    ; @src line=22 col=14 op=const_str
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #1
    sub x9, x29, #320
    str x1, [x9]
    sub x9, x29, #312
    str x2, [x9]
    ; @src line=22 col=12 end=22:14 op=str_concat
    sub x9, x29, #304
    ldr x1, [x9]
    sub x9, x29, #296
    ldr x2, [x9]
    sub x9, x29, #320
    ldr x3, [x9]
    sub x9, x29, #312
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #336
    str x1, [x9]
    sub x9, x29, #328
    str x2, [x9]
    ; @src line=22 col=1 end=22:5 op=echo_value
    sub x9, x29, #336
    ldr x1, [x9]
    sub x9, x29, #328
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=22 col=1 end=22:5 op=release
    ; @src line=25 col=1 end=25:6 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=1 end=25:6 op=nop
    ; @src line=33 col=1 end=33:4 op=concat_reset
    sub x9, x29, #408
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=33 col=12 end=33:19 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #5
    sub x9, x29, #352
    str x1, [x9]
    sub x9, x29, #344
    str x2, [x9]
    ; @src line=33 col=1 end=33:20 op=object_new
    mov x0, #8
    bl __rt_heap_alloc
    mov x9, #4
    str x9, [x0, #-8]
    mov x10, #102
    str x10, [x0]
    sub x9, x29, #360
    str x0, [x9]
    sub x9, x29, #360
    ldr x0, [x9]
    str x0, [sp, #-16]!
    sub x9, x29, #352
    ldr x1, [x9]
    sub x9, x29, #344
    ldr x2, [x9]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    ldr x2, [sp, #8]
    add sp, sp, #32
    bl _method_Banner__u__u_construct
    ; @src line=33 col=1 end=33:20 op=release
    sub x9, x29, #360
    ldr x0, [x9]
    bl __rt_decref_object

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $name
    sub x9, x29, #384
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_2
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_2:
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
.globl _str_2
_str_2:
    .ascii "UTC"
.globl _str_3
_str_3:
    .ascii "["
.globl _str_4
_str_4:
    .ascii "]\n"
.globl _str_5
_str_5:
    .ascii "\n"
.globl _str_6
_str_6:
    .ascii "anonymous"
.globl _str_7
_str_7:
    .ascii "ready"
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
.globl _callable_user_fn_name_1
_callable_user_fn_name_1:
    .ascii "wrap"
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
    .quad 4
    .quad 0
.p2align 3
.globl _instanceof_target_count
_instanceof_target_count:
    .quad 34
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
    .quad 15
    .quad 1
    .quad 0
    .quad _instanceof_name_class_abs_1
    .quad 16
    .quad 1
    .quad 0
    .quad _instanceof_name_class_23
    .quad 9
    .quad 23
    .quad 0
    .quad _instanceof_name_class_abs_23
    .quad 10
    .quad 23
    .quad 0
    .quad _instanceof_name_class_28
    .quad 14
    .quad 28
    .quad 0
    .quad _instanceof_name_class_abs_28
    .quad 15
    .quad 28
    .quad 0
    .quad _instanceof_name_class_35
    .quad 19
    .quad 35
    .quad 0
    .quad _instanceof_name_class_abs_35
    .quad 20
    .quad 35
    .quad 0
    .quad _instanceof_name_class_37
    .quad 16
    .quad 37
    .quad 0
    .quad _instanceof_name_class_abs_37
    .quad 17
    .quad 37
    .quad 0
    .quad _instanceof_name_class_38
    .quad 20
    .quad 38
    .quad 0
    .quad _instanceof_name_class_abs_38
    .quad 21
    .quad 38
    .quad 0
    .quad _instanceof_name_class_54
    .quad 19
    .quad 54
    .quad 0
    .quad _instanceof_name_class_abs_54
    .quad 20
    .quad 54
    .quad 0
    .quad _instanceof_name_class_55
    .quad 9
    .quad 55
    .quad 0
    .quad _instanceof_name_class_abs_55
    .quad 10
    .quad 55
    .quad 0
    .quad _instanceof_name_class_64
    .quad 19
    .quad 64
    .quad 0
    .quad _instanceof_name_class_abs_64
    .quad 20
    .quad 64
    .quad 0
    .quad _instanceof_name_class_75
    .quad 10
    .quad 75
    .quad 0
    .quad _instanceof_name_class_abs_75
    .quad 11
    .quad 75
    .quad 0
    .quad _instanceof_name_class_86
    .quad 13
    .quad 86
    .quad 0
    .quad _instanceof_name_class_abs_86
    .quad 14
    .quad 86
    .quad 0
    .quad _instanceof_name_class_88
    .quad 8
    .quad 88
    .quad 0
    .quad _instanceof_name_class_abs_88
    .quad 9
    .quad 88
    .quad 0
    .quad _instanceof_name_class_89
    .quad 24
    .quad 89
    .quad 0
    .quad _instanceof_name_class_abs_89
    .quad 25
    .quad 89
    .quad 0
    .quad _instanceof_name_class_102
    .quad 6
    .quad 102
    .quad 0
    .quad _instanceof_name_class_abs_102
    .quad 7
    .quad 102
    .quad 0
    .quad _instanceof_name_interface_9
    .quad 10
    .quad 9
    .quad 1
    .quad _instanceof_name_interface_abs_9
    .quad 11
    .quad 9
    .quad 1
    .quad _instanceof_name_interface_10
    .quad 9
    .quad 10
    .quad 1
    .quad _instanceof_name_interface_abs_10
    .quad 10
    .quad 10
    .quad 1
.globl _instanceof_name_class_0
_instanceof_name_class_0:
    .ascii "Error"
.globl _instanceof_name_class_abs_0
_instanceof_name_class_abs_0:
    .ascii "\\Error"
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_23
_instanceof_name_class_23:
    .ascii "Exception"
.globl _instanceof_name_class_abs_23
_instanceof_name_class_abs_23:
    .ascii "\\Exception"
.globl _instanceof_name_class_28
_instanceof_name_class_28:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_28
_instanceof_name_class_abs_28:
    .ascii "\\LogicException"
.globl _instanceof_name_class_35
_instanceof_name_class_35:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_35
_instanceof_name_class_abs_35:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_37
_instanceof_name_class_37:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_37
_instanceof_name_class_abs_37:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_38
_instanceof_name_class_38:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_38
_instanceof_name_class_abs_38:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_54
_instanceof_name_class_54:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_54
_instanceof_name_class_abs_54:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_55
_instanceof_name_class_55:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_55
_instanceof_name_class_abs_55:
    .ascii "\\TypeError"
.globl _instanceof_name_class_64
_instanceof_name_class_64:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_64
_instanceof_name_class_abs_64:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_75
_instanceof_name_class_75:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_75
_instanceof_name_class_abs_75:
    .ascii "\\ValueError"
.globl _instanceof_name_class_86
_instanceof_name_class_86:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_86
_instanceof_name_class_abs_86:
    .ascii "\\JsonException"
.globl _instanceof_name_class_88
_instanceof_name_class_88:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_88
_instanceof_name_class_abs_88:
    .ascii "\\stdClass"
.globl _instanceof_name_class_89
_instanceof_name_class_89:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_89
_instanceof_name_class_abs_89:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_102
_instanceof_name_class_102:
    .ascii "Banner"
.globl _instanceof_name_class_abs_102
_instanceof_name_class_abs_102:
    .ascii "\\Banner"
.globl _instanceof_name_interface_9
_instanceof_name_interface_9:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_9
_instanceof_name_interface_abs_9:
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
    .quad 103
.globl _class_name_entries
_class_name_entries:
    .quad _class_name_0
    .quad 5
    .quad _class_name_1
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_23
    .quad 9
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_28
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
    .quad _class_name_35
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_37
    .quad 16
    .quad _class_name_38
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_54
    .quad 19
    .quad _class_name_55
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
    .quad _class_name_64
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
    .quad _class_name_75
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
    .quad _class_name_86
    .quad 13
    .quad _class_name_missing
    .quad 0
    .quad _class_name_88
    .quad 8
    .quad _class_name_89
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
    .quad _class_name_102
    .quad 6
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_0
_class_name_0:
    .ascii "Error"
.globl _class_name_1
_class_name_1:
    .ascii "ArithmeticError"
.globl _class_name_23
_class_name_23:
    .ascii "Exception"
.globl _class_name_28
_class_name_28:
    .ascii "LogicException"
.globl _class_name_35
_class_name_35:
    .ascii "ReflectionException"
.globl _class_name_37
_class_name_37:
    .ascii "RuntimeException"
.globl _class_name_38
_class_name_38:
    .ascii "OutOfBoundsException"
.globl _class_name_54
_class_name_54:
    .ascii "UnhandledMatchError"
.globl _class_name_55
_class_name_55:
    .ascii "TypeError"
.globl _class_name_64
_class_name_64:
    .ascii "OutOfRangeException"
.globl _class_name_75
_class_name_75:
    .ascii "ValueError"
.globl _class_name_86
_class_name_86:
    .ascii "JsonException"
.globl _class_name_88
_class_name_88:
    .ascii "stdClass"
.globl _class_name_89
_class_name_89:
    .ascii "InvalidArgumentException"
.globl _class_name_102
_class_name_102:
    .ascii "Banner"
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
    .quad 41
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 61
.globl _generator_class_id
_generator_class_id:
    .quad 33
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 21
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 22
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 44
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 50
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 0
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 28
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 37
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 64
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 38
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 89
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 55
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 75
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 35
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 1
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_9
    .quad _interface_methods_10
.globl _class_interface_ptrs
_class_interface_ptrs:
    .quad _class_interfaces_0
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_28
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_35
    .quad _class_interfaces_missing
    .quad _class_interfaces_37
    .quad _class_interfaces_38
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
    .quad _class_interfaces_55
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_64
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
    .quad _class_interfaces_75
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
    .quad _class_interfaces_86
    .quad _class_interfaces_missing
    .quad _class_interfaces_88
    .quad _class_interfaces_89
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
    .quad _class_interfaces_102
.globl _class_json_desc_ptrs
_class_json_desc_ptrs:
    .quad _class_json_desc_0
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_28
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_35
    .quad _class_json_desc_missing
    .quad _class_json_desc_37
    .quad _class_json_desc_38
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
    .quad _class_json_desc_55
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_64
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
    .quad _class_json_desc_75
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
    .quad _class_json_desc_86
    .quad _class_json_desc_missing
    .quad _class_json_desc_88
    .quad _class_json_desc_89
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
    .quad _class_json_desc_102
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 86
.globl _class_parent_ids
_class_parent_ids:
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 23
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 23
    .quad -1
    .quad 23
    .quad 37
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 0
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 28
    .quad -1
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
    .quad 37
    .quad -1
    .quad -1
    .quad 28
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 72
    .quad 0
    .quad 16
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
    .quad 8
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
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 103
.globl _class_gc_desc_ptrs
_class_gc_desc_ptrs:
    .quad _class_gc_desc_0
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_28
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_35
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_37
    .quad _class_gc_desc_38
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
    .quad _class_gc_desc_55
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_64
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
    .quad _class_gc_desc_75
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
    .quad _class_gc_desc_86
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_88
    .quad _class_gc_desc_89
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
    .quad _class_gc_desc_102
.globl _class_vtable_ptrs
_class_vtable_ptrs:
    .quad _class_vtable_0
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
    .quad _class_vtable_missing
    .quad _class_vtable_28
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_35
    .quad _class_vtable_missing
    .quad _class_vtable_37
    .quad _class_vtable_38
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
    .quad _class_vtable_55
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_64
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
    .quad _class_vtable_75
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
    .quad _class_vtable_86
    .quad _class_vtable_missing
    .quad _class_vtable_88
    .quad _class_vtable_89
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad 0
    .quad _class_propinit_28
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_35
    .quad 0
    .quad _class_propinit_37
    .quad _class_propinit_38
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_propinit_55
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_64
    .quad 0
    .quad 0
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_86
    .quad 0
    .quad 0
    .quad _class_propinit_89
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_serprop_missing
    .quad _class_serprop_28
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_35
    .quad _class_serprop_missing
    .quad _class_serprop_37
    .quad _class_serprop_38
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
    .quad _class_serprop_55
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_64
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
    .quad _class_serprop_75
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
    .quad _class_serprop_86
    .quad _class_serprop_missing
    .quad _class_serprop_88
    .quad _class_serprop_89
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
    .quad _class_serprop_102
.globl _class_static_vtable_ptrs
_class_static_vtable_ptrs:
    .quad _class_static_vtable_0
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_28
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_35
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_37
    .quad _class_static_vtable_38
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
    .quad _class_static_vtable_55
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_64
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
    .quad _class_static_vtable_75
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
    .quad _class_static_vtable_86
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_88
    .quad _class_static_vtable_89
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
    .quad _class_static_vtable_102
.globl _class_callable_method_ptrs
_class_callable_method_ptrs:
    .quad _class_callable_methods_0
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_28
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_35
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_37
    .quad _class_callable_methods_38
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
    .quad _class_callable_methods_55
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_64
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
    .quad _class_callable_methods_75
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
    .quad _class_callable_methods_86
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_88
    .quad _class_callable_methods_89
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
    .ascii "Error"
.globl _class_by_name_str_1
_class_by_name_str_1:
    .ascii "ArithmeticError"
.globl _class_by_name_str_23
_class_by_name_str_23:
    .ascii "Exception"
.globl _class_by_name_str_28
_class_by_name_str_28:
    .ascii "LogicException"
.globl _class_by_name_str_35
_class_by_name_str_35:
    .ascii "ReflectionException"
.globl _class_by_name_str_37
_class_by_name_str_37:
    .ascii "RuntimeException"
.globl _class_by_name_str_38
_class_by_name_str_38:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_54
_class_by_name_str_54:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_55
_class_by_name_str_55:
    .ascii "TypeError"
.globl _class_by_name_str_64
_class_by_name_str_64:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_75
_class_by_name_str_75:
    .ascii "ValueError"
.globl _class_by_name_str_86
_class_by_name_str_86:
    .ascii "JsonException"
.globl _class_by_name_str_88
_class_by_name_str_88:
    .ascii "stdClass"
.globl _class_by_name_str_89
_class_by_name_str_89:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_102
_class_by_name_str_102:
    .ascii "Banner"
.p2align 3
.globl _classes_by_name_count
_classes_by_name_count:
    .quad 15
.globl _classes_by_name
_classes_by_name:
    .quad _class_by_name_str_0
    .quad 5
    .quad 0
    .quad 72
    .quad _class_by_name_str_1
    .quad 15
    .quad 1
    .quad 72
    .quad _class_by_name_str_23
    .quad 9
    .quad 23
    .quad 72
    .quad _class_by_name_str_28
    .quad 14
    .quad 28
    .quad 72
    .quad _class_by_name_str_35
    .quad 19
    .quad 35
    .quad 72
    .quad _class_by_name_str_37
    .quad 16
    .quad 37
    .quad 72
    .quad _class_by_name_str_38
    .quad 20
    .quad 38
    .quad 72
    .quad _class_by_name_str_54
    .quad 19
    .quad 54
    .quad 72
    .quad _class_by_name_str_55
    .quad 9
    .quad 55
    .quad 72
    .quad _class_by_name_str_64
    .quad 19
    .quad 64
    .quad 72
    .quad _class_by_name_str_75
    .quad 10
    .quad 75
    .quad 72
    .quad _class_by_name_str_86
    .quad 13
    .quad 86
    .quad 72
    .quad _class_by_name_str_88
    .quad 8
    .quad 88
    .quad 16
    .quad _class_by_name_str_89
    .quad 24
    .quad 89
    .quad 72
    .quad _class_by_name_str_102
    .quad 6
    .quad 102
    .quad 8
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
.globl _interface_methods_9
_interface_methods_9:
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
.globl _class_interfaces_0
_class_interfaces_0:
    .quad 2
    .quad 10
    .quad _class_interface_impl_0_10
    .quad 9
    .quad _class_interface_impl_0_9
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
.globl _class_interface_impl_0_9
_class_interface_impl_0_9:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
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
    .quad 9
    .quad _class_interface_impl_1_9
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
.globl _class_interface_impl_1_9
_class_interface_impl_1_9:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
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
.globl _class_interfaces_23
_class_interfaces_23:
    .quad 2
    .quad 10
    .quad _class_interface_impl_23_10
    .quad 9
    .quad _class_interface_impl_23_9
.globl _class_interface_impl_23_10
_class_interface_impl_23_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_23_9
_class_interface_impl_23_9:
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
.globl _class_interfaces_28
_class_interfaces_28:
    .quad 2
    .quad 10
    .quad _class_interface_impl_28_10
    .quad 9
    .quad _class_interface_impl_28_9
.globl _class_interface_impl_28_10
_class_interface_impl_28_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_28_9
_class_interface_impl_28_9:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_28
_class_vtable_28:
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
.globl _class_interfaces_35
_class_interfaces_35:
    .quad 2
    .quad 10
    .quad _class_interface_impl_35_10
    .quad 9
    .quad _class_interface_impl_35_9
.globl _class_interface_impl_35_10
_class_interface_impl_35_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_35_9
_class_interface_impl_35_9:
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
.globl _class_interfaces_37
_class_interfaces_37:
    .quad 2
    .quad 10
    .quad _class_interface_impl_37_10
    .quad 9
    .quad _class_interface_impl_37_9
.globl _class_interface_impl_37_10
_class_interface_impl_37_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_37_9
_class_interface_impl_37_9:
    .quad 0
.globl _class_json_pname_37_0
_class_json_pname_37_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_37
_class_json_desc_37:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_37_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_37
_class_gc_desc_37:
    .byte 1, 0, 7, 4
.globl _class_serpname_37_0
_class_serpname_37_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_37_1
_class_serpname_37_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_37_2
_class_serpname_37_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_37_3
_class_serpname_37_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_37
_class_serprop_37:
    .quad 4
    .quad _class_serpname_37_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_37_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_37_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_37_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_37
_class_vtable_37:
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
.globl _class_static_vtable_37
_class_static_vtable_37:
    .quad 0
.globl _class_callable_method_name_37__u__u_construct
_class_callable_method_name_37__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_37__u__u_tostring
_class_callable_method_name_37__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_37_getcode
_class_callable_method_name_37_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_37_getfile
_class_callable_method_name_37_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_37_getline
_class_callable_method_name_37_getline:
    .ascii "getline"
.globl _class_callable_method_name_37_getmessage
_class_callable_method_name_37_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_37_getprevious
_class_callable_method_name_37_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_37_gettrace
_class_callable_method_name_37_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_37_gettraceasstring
_class_callable_method_name_37_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_37
_class_callable_methods_37:
    .quad 9
    .quad _class_callable_method_name_37__u__u_construct
    .quad 11
    .quad _class_callable_method_name_37__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_37_getcode
    .quad 7
    .quad _class_callable_method_name_37_getfile
    .quad 7
    .quad _class_callable_method_name_37_getline
    .quad 7
    .quad _class_callable_method_name_37_getmessage
    .quad 10
    .quad _class_callable_method_name_37_getprevious
    .quad 11
    .quad _class_callable_method_name_37_gettrace
    .quad 8
    .quad _class_callable_method_name_37_gettraceasstring
    .quad 16
.globl _class_interfaces_38
_class_interfaces_38:
    .quad 2
    .quad 10
    .quad _class_interface_impl_38_10
    .quad 9
    .quad _class_interface_impl_38_9
.globl _class_interface_impl_38_10
_class_interface_impl_38_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_38_9
_class_interface_impl_38_9:
    .quad 0
.globl _class_json_pname_38_0
_class_json_pname_38_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_38
_class_json_desc_38:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_38_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_38
_class_gc_desc_38:
    .byte 1, 0, 7, 4
.globl _class_serpname_38_0
_class_serpname_38_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_38_1
_class_serpname_38_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_38_2
_class_serpname_38_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_38_3
_class_serpname_38_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_38
_class_serprop_38:
    .quad 4
    .quad _class_serpname_38_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_38_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_38_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_38_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_38
_class_vtable_38:
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
.globl _class_static_vtable_38
_class_static_vtable_38:
    .quad 0
.globl _class_callable_method_name_38__u__u_construct
_class_callable_method_name_38__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_38__u__u_tostring
_class_callable_method_name_38__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_38_getcode
_class_callable_method_name_38_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_38_getfile
_class_callable_method_name_38_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_38_getline
_class_callable_method_name_38_getline:
    .ascii "getline"
.globl _class_callable_method_name_38_getmessage
_class_callable_method_name_38_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_38_getprevious
_class_callable_method_name_38_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_38_gettrace
_class_callable_method_name_38_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_38_gettraceasstring
_class_callable_method_name_38_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_38
_class_callable_methods_38:
    .quad 9
    .quad _class_callable_method_name_38__u__u_construct
    .quad 11
    .quad _class_callable_method_name_38__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_38_getcode
    .quad 7
    .quad _class_callable_method_name_38_getfile
    .quad 7
    .quad _class_callable_method_name_38_getline
    .quad 7
    .quad _class_callable_method_name_38_getmessage
    .quad 10
    .quad _class_callable_method_name_38_getprevious
    .quad 11
    .quad _class_callable_method_name_38_gettrace
    .quad 8
    .quad _class_callable_method_name_38_gettraceasstring
    .quad 16
.globl _class_interfaces_54
_class_interfaces_54:
    .quad 2
    .quad 10
    .quad _class_interface_impl_54_10
    .quad 9
    .quad _class_interface_impl_54_9
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
.globl _class_interface_impl_54_9
_class_interface_impl_54_9:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_54
_class_vtable_54:
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
.globl _class_interfaces_55
_class_interfaces_55:
    .quad 2
    .quad 10
    .quad _class_interface_impl_55_10
    .quad 9
    .quad _class_interface_impl_55_9
.globl _class_interface_impl_55_10
_class_interface_impl_55_10:
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
.globl _class_interfaces_64
_class_interfaces_64:
    .quad 2
    .quad 10
    .quad _class_interface_impl_64_10
    .quad 9
    .quad _class_interface_impl_64_9
.globl _class_interface_impl_64_10
_class_interface_impl_64_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_64_9
_class_interface_impl_64_9:
    .quad 0
.globl _class_json_pname_64_0
_class_json_pname_64_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_64
_class_json_desc_64:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_64_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_64
_class_gc_desc_64:
    .byte 1, 0, 7, 4
.globl _class_serpname_64_0
_class_serpname_64_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_64_1
_class_serpname_64_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_64_2
_class_serpname_64_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_64_3
_class_serpname_64_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_64
_class_serprop_64:
    .quad 4
    .quad _class_serpname_64_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_64_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_64_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_64_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_64
_class_vtable_64:
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
.globl _class_static_vtable_64
_class_static_vtable_64:
    .quad 0
.globl _class_callable_method_name_64__u__u_construct
_class_callable_method_name_64__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_64__u__u_tostring
_class_callable_method_name_64__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_64_getcode
_class_callable_method_name_64_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_64_getfile
_class_callable_method_name_64_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_64_getline
_class_callable_method_name_64_getline:
    .ascii "getline"
.globl _class_callable_method_name_64_getmessage
_class_callable_method_name_64_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_64_getprevious
_class_callable_method_name_64_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_64_gettrace
_class_callable_method_name_64_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_64_gettraceasstring
_class_callable_method_name_64_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_64
_class_callable_methods_64:
    .quad 9
    .quad _class_callable_method_name_64__u__u_construct
    .quad 11
    .quad _class_callable_method_name_64__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_64_getcode
    .quad 7
    .quad _class_callable_method_name_64_getfile
    .quad 7
    .quad _class_callable_method_name_64_getline
    .quad 7
    .quad _class_callable_method_name_64_getmessage
    .quad 10
    .quad _class_callable_method_name_64_getprevious
    .quad 11
    .quad _class_callable_method_name_64_gettrace
    .quad 8
    .quad _class_callable_method_name_64_gettraceasstring
    .quad 16
.globl _class_interfaces_75
_class_interfaces_75:
    .quad 2
    .quad 10
    .quad _class_interface_impl_75_10
    .quad 9
    .quad _class_interface_impl_75_9
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
.globl _class_interface_impl_75_9
_class_interface_impl_75_9:
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
.globl _class_interfaces_86
_class_interfaces_86:
    .quad 2
    .quad 10
    .quad _class_interface_impl_86_10
    .quad 9
    .quad _class_interface_impl_86_9
.globl _class_interface_impl_86_10
_class_interface_impl_86_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_86_9
_class_interface_impl_86_9:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_86
_class_vtable_86:
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
.globl _class_interfaces_88
_class_interfaces_88:
    .quad 0
    .p2align 3
.globl _class_json_desc_88
_class_json_desc_88:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_88
_class_gc_desc_88:
    .byte 0
    .p2align 3
.globl _class_serprop_88
_class_serprop_88:
    .quad 0
    .p2align 3
.globl _class_vtable_88
_class_vtable_88:
    .quad 0
    .p2align 3
.globl _class_static_vtable_88
_class_static_vtable_88:
    .quad 0
.p2align 3
.globl _class_callable_methods_88
_class_callable_methods_88:
    .quad 0
.globl _class_interfaces_89
_class_interfaces_89:
    .quad 2
    .quad 10
    .quad _class_interface_impl_89_10
    .quad 9
    .quad _class_interface_impl_89_9
.globl _class_interface_impl_89_10
_class_interface_impl_89_10:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_89_9
_class_interface_impl_89_9:
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
.globl _class_interfaces_102
_class_interfaces_102:
    .quad 0
    .p2align 3
.globl _class_json_desc_102
_class_json_desc_102:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_102
_class_gc_desc_102:
    .byte 0
    .p2align 3
.globl _class_serprop_102
_class_serprop_102:
    .quad 0
    .p2align 3
.globl _class_vtable_102
_class_vtable_102:
    .quad _method_Banner__u__u_construct
    .p2align 3
.globl _class_static_vtable_102
_class_static_vtable_102:
    .quad 0
.globl _class_callable_method_name_102__u__u_construct
_class_callable_method_name_102__u__u_construct:
    .ascii "__construct"
.p2align 3
.globl _class_callable_methods_102
_class_callable_methods_102:
    .quad 1
    .quad _class_callable_method_name_102__u__u_construct
    .quad 11
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 88

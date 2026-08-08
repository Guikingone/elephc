    ; @fn name=describe symbol=_fn_describe
.align 2

.globl _fn_describe
_fn_describe:
    ; prologue
    sub sp, sp, #480
    stp x29, x30, [sp, #464]
    add x29, sp, #464
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #456
    str x10, [x9]
    ; param $thing from x0
    stur x0, [x29, #-216]
    ldur x0, [x29, #-216]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-216]
    stur xzr, [x29, #-224]
    ; @block name=entry
_eir_describe_entry_0:
    ; @src line=30 col=16 end=30:24 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=30 col=9 end=30:15 op=load_local
    ldur x0, [x29, #-216]
    stur x0, [x29, #-8]
    ; @src line=30 col=20 end=30:24 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    stur x0, [x29, #-16]
    ; @src line=30 col=16 end=30:24 op=strict_eq
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    bl __rt_mixed_strict_eq
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    cbnz x0, _eir_describe_if_then_2
    b _eir_describe_if_else_3
    ; @block name=if.merge
_eir_describe_if_merge_1:
    ; @src line=40 col=5 end=40:17 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_describe_goto_label_8
    ; @block name=if.then
_eir_describe_if_then_2:
    ; @src line=31 col=9 end=31:12 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=31 col=9 end=31:12 op=try_push_handler
    ; push EIR exception handler
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #448
    str x10, [x9]
    mov x10, #0
    sub x9, x29, #440
    str x10, [x9]
    adrp x9, _rt_diag_suppression@PAGE
    add x9, x9, _rt_diag_suppression@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #432
    str x10, [x9]
    sub x10, x29, #448
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    str x10, [x9]
    sub x0, x29, #424
    bl _setjmp
    cbnz x0, _eir_describe_try_catch_dispatch_4
    ; @src line=32 col=13 end=32:18 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=32 col=48 op=const_str
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #13
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    mov x0, #0
    stur x0, [x29, #-48]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    stur x0, [x29, #-56]
    ; @src line=32 col=19 end=32:64 op=object_new
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    mov x9, #24
    str x9, [x0]
    str xzr, [x0, #40]
    str x0, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp]
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x1, [x29, #-48]
    ldr x9, [sp]
    str x1, [x9, #24]
    ldur x0, [x29, #-56]
    cbz x0, _eir_describe_throwable_previous_null_1
    movz x9, #0xfffe
    movk x9, #0xffff, lsl #16
    movk x9, #0xffff, lsl #32
    movk x9, #0x7fff, lsl #48
    cmp x0, x9
    b.eq _eir_describe_throwable_previous_null_1
    bl __rt_incref
    b _eir_describe_throwable_previous_store_0
_eir_describe_throwable_previous_null_1:
    mov x0, xzr
_eir_describe_throwable_previous_store_0:
    ldr x9, [sp]
    str x0, [x9, #40]
    ldr x0, [sp], #16
    stur x0, [x29, #-64]
    ldur x0, [x29, #-64]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    bl __rt_throw_current
    ; @block name=if.else
_eir_describe_if_else_3:
    ; @src line=30 col=16 end=30:24 op=nop
    b _eir_describe_if_merge_1
    ; @block name=try.catch_dispatch
_eir_describe_try_catch_dispatch_4:
    ; @src line=31 col=9 end=31:12 op=try_pop_handler
    ; pop EIR exception handler
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    str x10, [x9]
    sub x9, x29, #432
    ldr x10, [x9]
    adrp x9, _rt_diag_suppression@PAGE
    add x9, x9, _rt_diag_suppression@PAGEOFF
    str x10, [x9]
    ; @src line=31 col=9 end=31:12 op=catch_current
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-72]
    ; @src line=31 col=9 end=31:12 op=instance_of
    ldur x0, [x29, #-72]
    mov x1, #24
    mov x2, #0
    bl __rt_exception_matches
    stur x0, [x29, #-80]
    ldur x0, [x29, #-80]
    cbnz x0, _eir_describe_try_catch_body_6
    b _eir_describe_try_catch_next_7
    ; @block name=try.after
_eir_describe_try_after_5:
    udf #0
    ; @block name=try.catch_body
_eir_describe_try_catch_body_6:
    ; @src line=31 col=9 end=31:12 op=catch_bind
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-88]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str xzr, [x9]
    ; @src line=31 col=9 end=31:12 op=store_local
    ldur x0, [x29, #-88]
    stur x0, [x29, #-224]
    ; @src line=35 col=13 end=35:19 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=35 col=22 op=const_str
    adrp x1, _str_1@PAGE
    add x1, x1, _str_1@PAGEOFF
    mov x2, #7
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=35 col=13 end=35:19 op=acquire
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]
    bl __rt_str_persist
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=35 col=13 end=35:19 op=load_local
    ldur x0, [x29, #-216]
    stur x0, [x29, #-128]
    ; @src line=35 col=13 end=35:19 op=release
    ldur x0, [x29, #-128]
    bl __rt_decref_mixed
    ; @src line=35 col=13 end=35:19 op=store_local
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    stp x1, x2, [sp, #-16]!
    mov x0, #24
    bl __rt_heap_alloc
    mov x9, #5
    str x9, [x0, #-8]
    mov x10, #1
    str x10, [x0]
    ldp x11, x12, [sp], #16
    stp x11, x12, [x0, #8]
    stur x0, [x29, #-216]
    ; @src line=36 col=13 end=36:17 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_describe_goto_label_8
    ; @block name=try.catch_next
_eir_describe_try_catch_next_7:
    ; @src line=31 col=9 end=31:12 op=catch_current
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-136]
    ldur x0, [x29, #-136]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    bl __rt_throw_current
    ; @block name=goto.label
_eir_describe_goto_label_8:
    ; @src line=41 col=5 end=41:11 op=concat_reset
    sub x9, x29, #456
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=41 col=12 op=const_str
    adrp x1, _str_2@PAGE
    add x1, x1, _str_2@PAGEOFF
    mov x2, #12
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=41 col=29 end=41:35 op=load_local
    ldur x0, [x29, #-216]
    stur x0, [x29, #-160]
    ; @src line=41 col=27 end=41:35 op=cast
    ldur x0, [x29, #-160]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_describe_mixed_string_object_2
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_object_2:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_describe_mixed_string_Exception_5
    mov x10, #2
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateException_6
    mov x10, #3
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateMalformedPeriodStringException_7
    mov x10, #9
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionFunctionAbstract_8
    mov x10, #10
    cmp x9, x10
    b.eq _eir_describe_mixed_string_Error_9
    mov x10, #11
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateError_10
    mov x10, #12
    cmp x9, x10
    b.eq _eir_describe_mixed_string_SplFileInfo_11
    mov x10, #13
    cmp x9, x10
    b.eq _eir_describe_mixed_string_PharFileInfo_12
    mov x10, #14
    cmp x9, x10
    b.eq _eir_describe_mixed_string_Phar_13
    mov x10, #21
    cmp x9, x10
    b.eq _eir_describe_mixed_string_TypeError_14
    mov x10, #22
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionClassConstant_15
    mov x10, #23
    cmp x9, x10
    b.eq _eir_describe_mixed_string_LogicException_16
    mov x10, #24
    cmp x9, x10
    b.eq _eir_describe_mixed_string_InvalidArgumentException_17
    mov x10, #28
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionUnionType_18
    mov x10, #29
    cmp x9, x10
    b.eq _eir_describe_mixed_string_RuntimeException_19
    mov x10, #31
    cmp x9, x10
    b.eq _eir_describe_mixed_string_FiberError_20
    mov x10, #33
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionEnumBackedCase_21
    mov x10, #35
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionEnum_22
    mov x10, #36
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateInvalidOperationException_23
    mov x10, #40
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DirectoryIterator_24
    mov x10, #41
    cmp x9, x10
    b.eq _eir_describe_mixed_string_FilesystemIterator_25
    mov x10, #42
    cmp x9, x10
    b.eq _eir_describe_mixed_string_RecursiveDirectoryIterator_26
    mov x10, #44
    cmp x9, x10
    b.eq _eir_describe_mixed_string_SplFileObject_27
    mov x10, #45
    cmp x9, x10
    b.eq _eir_describe_mixed_string_BadFunctionCallException_28
    mov x10, #49
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateUnknownException_29
    mov x10, #51
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionClass_30
    mov x10, #52
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionMethod_31
    mov x10, #53
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionProperty_32
    mov x10, #54
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionObject_33
    mov x10, #57
    cmp x9, x10
    b.eq _eir_describe_mixed_string_UnexpectedValueException_34
    mov x10, #58
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateMalformedStringException_35
    mov x10, #61
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateMalformedIntervalStringException_36
    mov x10, #62
    cmp x9, x10
    b.eq _eir_describe_mixed_string_UnhandledMatchError_37
    mov x10, #63
    cmp x9, x10
    b.eq _eir_describe_mixed_string_SplTempFileObject_38
    mov x10, #64
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ArithmeticError_39
    mov x10, #65
    cmp x9, x10
    b.eq _eir_describe_mixed_string_UnderflowException_40
    mov x10, #66
    cmp x9, x10
    b.eq _eir_describe_mixed_string_CachingIterator_41
    mov x10, #67
    cmp x9, x10
    b.eq _eir_describe_mixed_string_GlobIterator_42
    mov x10, #68
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionParameter_43
    mov x10, #70
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionException_44
    mov x10, #72
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionIntersectionType_45
    mov x10, #73
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateInvalidTimeZoneException_46
    mov x10, #76
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionFunction_47
    mov x10, #77
    cmp x9, x10
    b.eq _eir_describe_mixed_string_PharData_48
    mov x10, #81
    cmp x9, x10
    b.eq _eir_describe_mixed_string_JsonException_49
    mov x10, #83
    cmp x9, x10
    b.eq _eir_describe_mixed_string_OverflowException_50
    mov x10, #85
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateObjectError_51
    mov x10, #86
    cmp x9, x10
    b.eq _eir_describe_mixed_string_RecursiveCachingIterator_52
    mov x10, #88
    cmp x9, x10
    b.eq _eir_describe_mixed_string_BadMethodCallException_53
    mov x10, #89
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionEnumUnitCase_54
    mov x10, #92
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DateRangeError_55
    mov x10, #94
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ValueError_56
    mov x10, #95
    cmp x9, x10
    b.eq _eir_describe_mixed_string_OutOfRangeException_57
    mov x10, #96
    cmp x9, x10
    b.eq _eir_describe_mixed_string_ReflectionNamedType_58
    mov x10, #97
    cmp x9, x10
    b.eq _eir_describe_mixed_string_OutOfBoundsException_59
    mov x10, #99
    cmp x9, x10
    b.eq _eir_describe_mixed_string_LengthException_60
    mov x10, #100
    cmp x9, x10
    b.eq _eir_describe_mixed_string_RangeException_61
    mov x10, #101
    cmp x9, x10
    b.eq _eir_describe_mixed_string_DomainException_62
    b _eir_describe_mixed_string_no_match_3
_eir_describe_mixed_string_Exception_5:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateException_6:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateMalformedPeriodStringException_7:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionFunctionAbstract_8:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_Error_9:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateError_10:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_SplFileInfo_11:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_PharFileInfo_12:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_Phar_13:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_TypeError_14:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionClassConstant_15:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_LogicException_16:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_InvalidArgumentException_17:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionUnionType_18:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_RuntimeException_19:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_FiberError_20:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionEnumBackedCase_21:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionEnum_22:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateInvalidOperationException_23:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DirectoryIterator_24:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_FilesystemIterator_25:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_RecursiveDirectoryIterator_26:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_SplFileObject_27:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_BadFunctionCallException_28:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateUnknownException_29:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionClass_30:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionMethod_31:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionProperty_32:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionObject_33:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_UnexpectedValueException_34:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateMalformedStringException_35:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateMalformedIntervalStringException_36:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_UnhandledMatchError_37:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_SplTempFileObject_38:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ArithmeticError_39:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_UnderflowException_40:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_CachingIterator_41:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_GlobIterator_42:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionParameter_43:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionException_44:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionIntersectionType_45:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateInvalidTimeZoneException_46:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionFunction_47:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_PharData_48:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_JsonException_49:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_OverflowException_50:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateObjectError_51:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_RecursiveCachingIterator_52:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_BadMethodCallException_53:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionEnumUnitCase_54:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DateRangeError_55:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ValueError_56:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_OutOfRangeException_57:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_ReflectionNamedType_58:
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
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_OutOfBoundsException_59:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_LengthException_60:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_RangeException_61:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_DomainException_62:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_describe_mixed_string_done_4
_eir_describe_mixed_string_no_match_3:
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_describe_mixed_string_done_4:
    stur x1, [x29, #-176]
    stur x2, [x29, #-168]
    ; @src line=41 col=27 end=41:35 op=str_concat
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]
    ldur x3, [x29, #-176]
    ldur x4, [x29, #-168]
    bl __rt_concat
    stur x1, [x29, #-192]
    stur x2, [x29, #-184]
    ; @src line=41 col=27 end=41:35 op=release
    ldur x1, [x29, #-176]
    ldur x2, [x29, #-168]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=41 col=5 end=41:11 op=str_persist
    ldur x1, [x29, #-192]
    ldur x2, [x29, #-184]
    bl __rt_str_persist
    stur x1, [x29, #-208]
    stur x2, [x29, #-200]
    ldur x1, [x29, #-208]
    ldur x2, [x29, #-200]
    stp x1, x2, [sp, #-16]!
    ; epilogue cleanup $thing
    ldur x0, [x29, #-216]
    cbz x0, _eir_describe_main_refcounted_cleanup_done_63
    bl __rt_decref_mixed
_eir_describe_main_refcounted_cleanup_done_63:
    ; epilogue cleanup $e
    ldur x0, [x29, #-224]
    cbz x0, _eir_describe_main_refcounted_cleanup_done_64
    bl __rt_decref_object
_eir_describe_main_refcounted_cleanup_done_64:
    ldp x1, x2, [sp], #16
    ldp x29, x30, [sp, #464]
    add sp, sp, #480
    ret
_fn_describe_epilogue:
    stp x1, x2, [sp, #-16]!
    ; epilogue cleanup $thing
    ldur x0, [x29, #-216]
    cbz x0, _eir_describe_main_refcounted_cleanup_done_65
    bl __rt_decref_mixed
_eir_describe_main_refcounted_cleanup_done_65:
    ; epilogue cleanup $e
    ldur x0, [x29, #-224]
    cbz x0, _eir_describe_main_refcounted_cleanup_done_66
    bl __rt_decref_object
_eir_describe_main_refcounted_cleanup_done_66:
    ldp x1, x2, [sp], #16
    ldp x29, x30, [sp, #464]
    add sp, sp, #480
    ret
    ; @endfn name=describe
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
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_0_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
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
_class_propinit_0_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_6 symbol=_class_propinit_6 synthetic=1
.align 2

.globl _class_propinit_6
_class_propinit_6:
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
_eir__class_propinit_6_entry_0:
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
_class_propinit_6_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_6
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_18 symbol=_class_propinit_18 synthetic=1
.align 2

.globl _class_propinit_18
_class_propinit_18:
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
_eir__class_propinit_18_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_18_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_18
    ; @fn name=_class_propinit_19 symbol=_class_propinit_19 synthetic=1
.align 2

.globl _class_propinit_19
_class_propinit_19:
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
_eir__class_propinit_19_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_19_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_19
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_25_entry_0:
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
_class_propinit_25_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_25
    ; @fn name=_class_propinit_26 symbol=_class_propinit_26 synthetic=1
.align 2

.globl _class_propinit_26
_class_propinit_26:
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
_eir__class_propinit_26_entry_0:
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
_class_propinit_26_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_26
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_30_entry_0:
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
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
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
_class_propinit_30_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_48 symbol=_class_propinit_48 synthetic=1
.align 2

.globl _class_propinit_48
_class_propinit_48:
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
_eir__class_propinit_48_entry_0:
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
    adrp x9, _float_6@PAGE
    add x9, x9, _float_6@PAGEOFF
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
_class_propinit_48_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_48
    ; @fn name=_class_propinit_49 symbol=_class_propinit_49 synthetic=1
.align 2

.globl _class_propinit_49
_class_propinit_49:
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
_eir__class_propinit_49_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_49_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_49
    ; @fn name=_class_propinit_50 symbol=_class_propinit_50 synthetic=1
.align 2

.globl _class_propinit_50
_class_propinit_50:
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
_eir__class_propinit_50_entry_0:
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
_class_propinit_50_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_50
    ; @fn name=_class_propinit_56 symbol=_class_propinit_56 synthetic=1
.align 2

.globl _class_propinit_56
_class_propinit_56:
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
_eir__class_propinit_56_entry_0:
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
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
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
_class_propinit_56_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_62_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_62_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_62
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
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
_eir__class_propinit_65_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_class_propinit_65_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_65
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    ; @fn name=_class_propinit_80 symbol=_class_propinit_80 synthetic=1
.align 2

.globl _class_propinit_80
_class_propinit_80:
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
_eir__class_propinit_80_entry_0:
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
_class_propinit_80_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_90_entry_0:
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
_class_propinit_90_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_90
    ; @fn name=_class_propinit_91 symbol=_class_propinit_91 synthetic=1
.align 2

.globl _class_propinit_91
_class_propinit_91:
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
_eir__class_propinit_91_entry_0:
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
_class_propinit_91_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
    adrp x1, _str_5@PAGE
    add x1, x1, _str_5@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
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
_eir__class_propinit_102_entry_0:
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
_class_propinit_102_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_102
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #1280
    mov x9, sp
    add x9, x9, #1264
    stp x29, x30, [x9]
    add x29, sp, #1264
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #1264
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #1248
    str x21, [x9]
    sub x9, x29, #1256
    str x22, [x9]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    sub x9, x29, #1184
    str xzr, [x9]
    sub x9, x29, #1200
    str xzr, [x9]
    sub x9, x29, #1192
    str xzr, [x9]
    sub x9, x29, #1208
    str xzr, [x9]
    sub x9, x29, #1216
    str xzr, [x9]
    sub x9, x29, #1224
    str xzr, [x9]
    sub x9, x29, #1232
    str xzr, [x9]
    sub x9, x29, #1240
    str xzr, [x9]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=9 col=1 end=9:8 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=11 end=9:13 op=nop
    ; @src line=9 col=1 end=9:8 op=nop
    ; @src line=10 col=1 end=10:6 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=9 end=10:10 op=array_new
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
    stur x0, [x29, #-16]
    ; @src line=10 col=10 end=10:11 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    stur x0, [x29, #-24]
    ; @src line=10 col=11 end=10:12 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=10 col=11 end=10:12 op=array_push
    mov x1, x21
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-24]
    ; @src line=10 col=14 end=10:15 op=const_i64
    mov x0, #2
    mov x21, x0
    ; @src line=10 col=14 end=10:15 op=array_push
    mov x1, x21
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-24]
    ; @src line=10 col=17 end=10:18 op=const_i64
    mov x0, #3
    mov x21, x0
    ; @src line=10 col=17 end=10:18 op=array_push
    mov x1, x21
    ldur x9, [x29, #-24]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-24]
    ; @src line=10 col=10 end=10:11 op=array_push
    ldur x1, [x29, #-24]
    ldur x9, [x29, #-16]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-16]
    ; @src line=10 col=10 end=10:11 op=release
    ldur x0, [x29, #-24]
    bl __rt_decref_any
    ; @src line=10 col=21 end=10:22 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    stur x0, [x29, #-56]
    ; @src line=10 col=22 end=10:23 op=const_i64
    mov x0, #4
    mov x21, x0
    ; @src line=10 col=22 end=10:23 op=array_push
    mov x1, x21
    ldur x9, [x29, #-56]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-56]
    ; @src line=10 col=25 end=10:27 op=const_i64
    mov x0, #42
    mov x21, x0
    ; @src line=10 col=25 end=10:27 op=array_push
    mov x1, x21
    ldur x9, [x29, #-56]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-56]
    ; @src line=10 col=29 end=10:30 op=const_i64
    mov x0, #6
    mov x21, x0
    ; @src line=10 col=29 end=10:30 op=array_push
    mov x1, x21
    ldur x9, [x29, #-56]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-56]
    ; @src line=10 col=21 end=10:22 op=array_push
    ldur x1, [x29, #-56]
    ldur x9, [x29, #-16]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-16]
    ; @src line=10 col=21 end=10:22 op=release
    ldur x0, [x29, #-56]
    bl __rt_decref_any
    ; @src line=10 col=33 end=10:34 op=array_new
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    stur x0, [x29, #-88]
    ; @src line=10 col=34 end=10:35 op=const_i64
    mov x0, #7
    mov x21, x0
    ; @src line=10 col=34 end=10:35 op=array_push
    mov x1, x21
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-88]
    ; @src line=10 col=37 end=10:38 op=const_i64
    mov x0, #8
    mov x21, x0
    ; @src line=10 col=37 end=10:38 op=array_push
    mov x1, x21
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-88]
    ; @src line=10 col=40 end=10:41 op=const_i64
    mov x0, #9
    mov x21, x0
    ; @src line=10 col=40 end=10:41 op=array_push
    mov x1, x21
    ldur x9, [x29, #-88]
    mov x0, x9
    bl __rt_array_push_int
    stur x0, [x29, #-88]
    ; @src line=10 col=33 end=10:34 op=array_push
    ldur x1, [x29, #-88]
    ldur x9, [x29, #-16]
    mov x0, x9
    bl __rt_array_push_refcounted
    stur x0, [x29, #-16]
    ; @src line=10 col=33 end=10:34 op=release
    ldur x0, [x29, #-88]
    bl __rt_decref_any
    ; @src line=10 col=1 end=10:6 op=acquire
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-120]
    ; @src line=10 col=1 end=10:6 op=store_local
    ldur x0, [x29, #-120]
    sub x9, x29, #1184
    str x0, [x9]
    ; @src line=10 col=1 end=10:6 op=release
    ldur x0, [x29, #-16]
    bl __rt_decref_any
    ; @src line=11 col=1 end=11:7 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=11 col=10 op=const_str
    adrp x1, _str_7@PAGE
    add x1, x1, _str_7@PAGEOFF
    mov x2, #9
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=11 col=1 end=11:7 op=acquire
    ldur x1, [x29, #-136]
    ldur x2, [x29, #-128]
    bl __rt_str_persist
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=11 col=1 end=11:7 op=store_local
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]
    sub x9, x29, #1200
    str x1, [x9]
    sub x9, x29, #1192
    str x2, [x9]
    ; @src line=12 col=1 end=12:8 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=12 col=10 end=12:15 op=load_local
    sub x9, x29, #1184
    ldr x0, [x9]
    stur x0, [x29, #-160]
    ; @src line=12 col=10 end=12:15 op=iter_start
    ldur x0, [x29, #-160]
    stur x0, [x29, #-232]
    mov x0, #-1
    stur x0, [x29, #-224]
    ldur x0, [x29, #-232]
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
    stur x10, [x29, #-168]
    ; @src line=12 col=10 end=12:15 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=12 col=10 end=12:15 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    stur x0, [x29, #-248]
    ; @src line=12 col=10 end=12:15 op=acquire
    ldur x0, [x29, #-248]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #256
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=store_local
    sub x9, x29, #256
    ldr x0, [x9]
    sub x9, x29, #1208
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    ldur x0, [x29, #-248]
    bl __rt_decref_mixed
    ; @src line=12 col=10 end=12:15 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=12 col=10 end=12:15 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #272
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=acquire
    sub x9, x29, #272
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #280
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=store_local
    sub x9, x29, #280
    ldr x0, [x9]
    sub x9, x29, #1216
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    sub x9, x29, #272
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_1
    ; @block name=foreach.next
_eir_main_foreach_next_1:
    ; @src line=12 col=10 end=12:15 op=iter_next
    ldur x10, [x29, #-224]
    add x10, x10, #1
    ldur x11, [x29, #-168]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_2
    stur x10, [x29, #-224]
_eir_main_iter_next_done_2:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_2
    b _eir_main_foreach_exit_3
    ; @block name=foreach.body
_eir_main_foreach_body_2:
    ; @src line=12 col=10 end=12:15 op=iter_current_key
    ldur x0, [x29, #-224]
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    sub x9, x29, #296
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=acquire
    sub x9, x29, #296
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #304
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=load_local
    sub x9, x29, #1208
    ldr x0, [x9]
    sub x9, x29, #312
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    sub x9, x29, #312
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=12 col=10 end=12:15 op=store_local
    sub x9, x29, #304
    ldr x0, [x9]
    sub x9, x29, #1208
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    sub x9, x29, #296
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=12 col=10 end=12:15 op=iter_current_value
    ldur x12, [x29, #-232]
    ldur x10, [x29, #-224]
    add x12, x12, #24
    ldr x0, [x12, x10, lsl #3]
    mov x1, x0
    mov x2, xzr
    mov x0, #4
    bl __rt_mixed_from_value
    sub x9, x29, #320
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=acquire
    sub x9, x29, #320
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #328
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=load_local
    sub x9, x29, #1216
    ldr x0, [x9]
    sub x9, x29, #336
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    sub x9, x29, #336
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=12 col=10 end=12:15 op=store_local
    sub x9, x29, #328
    ldr x0, [x9]
    sub x9, x29, #1216
    str x0, [x9]
    ; @src line=12 col=10 end=12:15 op=release
    sub x9, x29, #320
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=5 end=13:12 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=14 end=13:20 op=load_local
    sub x9, x29, #1216
    ldr x0, [x9]
    sub x9, x29, #344
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=iter_start
    sub x9, x29, #344
    ldr x0, [x9]
    bl __rt_mixed_unbox
    cmp x0, #4
    b.eq _eir_main_iter_start_mixed_indexed_3
    cmp x0, #5
    b.eq _eir_main_iter_start_mixed_hash_4
    cmp x0, #6
    b.eq _eir_main_iter_start_mixed_object_5
    bl __rt_iterable_unsupported_kind
_eir_main_iter_start_mixed_indexed_3:
    sub x9, x29, #416
    str x1, [x9]
    mov x0, #-1
    sub x9, x29, #408
    str x0, [x9]
    sub x9, x29, #416
    ldr x0, [x9]
    cbz x0, _eir_main_iter_len_null_source_7
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir_main_iter_len_null_source_7
    ldr x10, [x0]
    b _eir_main_iter_len_done_8
_eir_main_iter_len_null_source_7:
    mov x10, #0
_eir_main_iter_len_done_8:
    sub x9, x29, #352
    str x10, [x9]
    b _eir_main_iter_start_mixed_done_6
_eir_main_iter_start_mixed_hash_4:
    sub x9, x29, #416
    str x1, [x9]
    mov x0, #0
    sub x9, x29, #408
    str x0, [x9]
    b _eir_main_iter_start_mixed_done_6
_eir_main_iter_start_mixed_object_5:
    sub x9, x29, #416
    str x1, [x9]
    mov x0, #0
    sub x9, x29, #408
    str x0, [x9]
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #15
_eir_main_interface_dispatch_scan_10:
    cbz x10, _eir_main_interface_dispatch_missing_12
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_11
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_10
_eir_main_interface_dispatch_found_11:
    ldr x11, [x11, #8]
    ldr x11, [x11]
    blr x11
    b _eir_main_interface_dispatch_done_13
_eir_main_interface_dispatch_missing_12:
    mov x0, #0
_eir_main_interface_dispatch_done_13:
    cbz x0, _eir_main_iter_dynamic_keep_original_object_9
    sub x9, x29, #416
    str x0, [x9]
_eir_main_iter_dynamic_keep_original_object_9:
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x9, _generator_class_id@PAGE
    add x9, x9, _generator_class_id@PAGEOFF
    ldr x11, [x9]
    cmp x10, x11
    b.ne _eir_main_interface_dispatch_not_generator_15
    bl __rt_gen_rewind
    b _eir_main_interface_dispatch_done_14
_eir_main_interface_dispatch_not_generator_15:
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #2
_eir_main_interface_dispatch_scan_16:
    cbz x10, _eir_main_interface_dispatch_missing_18
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_17
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_16
_eir_main_interface_dispatch_found_17:
    ldr x11, [x11, #8]
    ldr x11, [x11, #32]
    blr x11
    b _eir_main_interface_dispatch_done_14
_eir_main_interface_dispatch_missing_18:
    mov x0, #0
_eir_main_interface_dispatch_done_14:
_eir_main_iter_start_mixed_done_6:
    ; @src line=13 col=14 end=13:20 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=13 col=14 end=13:20 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #432
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=acquire
    sub x9, x29, #432
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #440
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=load_local
    sub x9, x29, #1224
    ldr x0, [x9]
    sub x9, x29, #448
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #448
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=store_local
    sub x9, x29, #440
    ldr x0, [x9]
    sub x9, x29, #1224
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #432
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=13 col=14 end=13:20 op=mixed_box
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    sub x9, x29, #464
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=acquire
    sub x9, x29, #464
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #472
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=load_local
    sub x9, x29, #1232
    ldr x0, [x9]
    sub x9, x29, #480
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #480
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=store_local
    sub x9, x29, #472
    ldr x0, [x9]
    sub x9, x29, #1232
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #464
    ldr x0, [x9]
    bl __rt_decref_mixed
    b _eir_main_foreach_next_4
    ; @block name=foreach.exit
_eir_main_foreach_exit_3:
    ; @src line=12 col=10 end=12:15 op=nop
    ; @src line=20 col=1 end=20:15 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_goto_label_10
    ; @block name=foreach.next
_eir_main_foreach_next_4:
    ; @src line=13 col=14 end=13:20 op=iter_next
    sub x9, x29, #416
    ldr x0, [x9]
    bl __rt_heap_kind
    cmp x0, #2
    b.eq _eir_main_iter_next_dyn_indexed_19
    cmp x0, #3
    b.eq _eir_main_iter_next_dyn_hash_20
    cmp x0, #4
    b.eq _eir_main_iter_next_dyn_object_21
    bl __rt_iterable_unsupported_kind
_eir_main_iter_next_dyn_indexed_19:
    sub x9, x29, #408
    ldr x10, [x9]
    add x10, x10, #1
    sub x9, x29, #352
    ldr x11, [x9]
    cmp x10, x11
    cset x0, lt
    b.ge _eir_main_iter_next_done_23
    sub x9, x29, #408
    str x10, [x9]
_eir_main_iter_next_done_23:
    b _eir_main_iter_next_dyn_done_22
_eir_main_iter_next_dyn_hash_20:
    sub x9, x29, #416
    ldr x0, [x9]
    sub x9, x29, #408
    ldr x1, [x9]
    bl __rt_hash_iter_next
    cmn x0, #1
    sub x9, x29, #408
    str x0, [x9]
    sub x9, x29, #400
    str x1, [x9]
    sub x9, x29, #392
    str x2, [x9]
    sub x9, x29, #384
    str x3, [x9]
    sub x9, x29, #376
    str x4, [x9]
    sub x9, x29, #368
    str x5, [x9]
    sub x9, x29, #360
    str x6, [x9]
    cset x0, ne
    b _eir_main_iter_next_dyn_done_22
_eir_main_iter_next_dyn_object_21:
    sub x9, x29, #408
    ldr x0, [x9]
    cmp x0, #0
    b.eq _eir_main_interface_iter_first_24
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x9, _generator_class_id@PAGE
    add x9, x9, _generator_class_id@PAGEOFF
    ldr x11, [x9]
    cmp x10, x11
    b.ne _eir_main_interface_dispatch_not_generator_27
    bl __rt_gen_next
    b _eir_main_interface_dispatch_done_26
_eir_main_interface_dispatch_not_generator_27:
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #2
_eir_main_interface_dispatch_scan_28:
    cbz x10, _eir_main_interface_dispatch_missing_30
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_29
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_28
_eir_main_interface_dispatch_found_29:
    ldr x11, [x11, #8]
    ldr x11, [x11, #16]
    blr x11
    b _eir_main_interface_dispatch_done_26
_eir_main_interface_dispatch_missing_30:
    mov x0, #0
_eir_main_interface_dispatch_done_26:
    b _eir_main_interface_iter_valid_25
_eir_main_interface_iter_first_24:
    mov x0, #1
    sub x9, x29, #408
    str x0, [x9]
_eir_main_interface_iter_valid_25:
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x9, _generator_class_id@PAGE
    add x9, x9, _generator_class_id@PAGEOFF
    ldr x11, [x9]
    cmp x10, x11
    b.ne _eir_main_interface_dispatch_not_generator_32
    bl __rt_gen_valid
    b _eir_main_interface_dispatch_done_31
_eir_main_interface_dispatch_not_generator_32:
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #2
_eir_main_interface_dispatch_scan_33:
    cbz x10, _eir_main_interface_dispatch_missing_35
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_34
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_33
_eir_main_interface_dispatch_found_34:
    ldr x11, [x11, #8]
    ldr x11, [x11, #24]
    blr x11
    b _eir_main_interface_dispatch_done_31
_eir_main_interface_dispatch_missing_35:
    mov x0, #0
_eir_main_interface_dispatch_done_31:
_eir_main_iter_next_dyn_done_22:
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_foreach_body_5
    b _eir_main_foreach_next_1
    ; @block name=foreach.body
_eir_main_foreach_body_5:
    ; @src line=13 col=14 end=13:20 op=iter_current_key
    sub x9, x29, #416
    ldr x0, [x9]
    bl __rt_heap_kind
    cmp x0, #2
    b.eq _eir_main_iter_key_dyn_indexed_36
    cmp x0, #3
    b.eq _eir_main_iter_key_dyn_hash_37
    cmp x0, #4
    b.eq _eir_main_iter_key_dyn_object_38
    bl __rt_iterable_unsupported_kind
_eir_main_iter_key_dyn_indexed_36:
    sub x9, x29, #408
    ldr x0, [x9]
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    b _eir_main_iter_key_dyn_done_39
_eir_main_iter_key_dyn_hash_37:
    sub x9, x29, #400
    ldr x1, [x9]
    sub x9, x29, #392
    ldr x2, [x9]
    cmn x2, #1
    b.ne _eir_main_iter_hash_key_string_40
    mov x0, #0
    mov x2, xzr
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_key_done_41
_eir_main_iter_hash_key_string_40:
    mov x0, #1
    bl __rt_mixed_from_value
_eir_main_iter_hash_key_done_41:
    b _eir_main_iter_key_dyn_done_39
_eir_main_iter_key_dyn_object_38:
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x9, _generator_class_id@PAGE
    add x9, x9, _generator_class_id@PAGEOFF
    ldr x11, [x9]
    cmp x10, x11
    b.ne _eir_main_interface_dispatch_not_generator_43
    bl __rt_gen_key
    b _eir_main_interface_dispatch_done_42
_eir_main_interface_dispatch_not_generator_43:
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #2
_eir_main_interface_dispatch_scan_44:
    cbz x10, _eir_main_interface_dispatch_missing_46
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_45
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_44
_eir_main_interface_dispatch_found_45:
    ldr x11, [x11, #8]
    ldr x11, [x11, #8]
    blr x11
    b _eir_main_interface_dispatch_done_42
_eir_main_interface_dispatch_missing_46:
    mov x0, #0
_eir_main_interface_dispatch_done_42:
_eir_main_iter_key_dyn_done_39:
    sub x9, x29, #496
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=acquire
    sub x9, x29, #496
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #504
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=load_local
    sub x9, x29, #1224
    ldr x0, [x9]
    sub x9, x29, #512
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #512
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=store_local
    sub x9, x29, #504
    ldr x0, [x9]
    sub x9, x29, #1224
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #496
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=iter_current_value
    sub x9, x29, #416
    ldr x0, [x9]
    bl __rt_heap_kind
    cmp x0, #2
    b.eq _eir_main_iter_value_dyn_indexed_47
    cmp x0, #3
    b.eq _eir_main_iter_value_dyn_hash_48
    cmp x0, #4
    b.eq _eir_main_iter_value_dyn_object_49
    bl __rt_iterable_unsupported_kind
_eir_main_iter_value_dyn_indexed_47:
    sub x9, x29, #416
    ldr x11, [x9]
    sub x9, x29, #408
    ldr x0, [x9]
    ldr x5, [x11, #-8]
    lsr x5, x5, #8
    and x5, x5, #0x7f
    cmp x5, #1
    b.eq _eir_main_iter_dynamic_indexed_string_51
    add x11, x11, #24
    ldr x3, [x11, x0, lsl #3]
    mov x4, xzr
    b _eir_main_iter_dynamic_indexed_loaded_52
_eir_main_iter_dynamic_indexed_string_51:
    lsl x10, x0, #4
    add x11, x11, x10
    add x11, x11, #24
    ldr x3, [x11]
    ldr x4, [x11, #8]
_eir_main_iter_dynamic_indexed_loaded_52:
    cmp x5, #7
    b.eq _eir_main_iter_dynamic_indexed_reuse_box_53
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_iter_dynamic_indexed_done_54
_eir_main_iter_dynamic_indexed_reuse_box_53:
    mov x0, x3
    bl __rt_incref
_eir_main_iter_dynamic_indexed_done_54:
    b _eir_main_iter_value_dyn_done_50
_eir_main_iter_value_dyn_hash_48:
    sub x9, x29, #368
    ldr x5, [x9]
    sub x9, x29, #384
    ldr x3, [x9]
    sub x9, x29, #376
    ldr x4, [x9]
    mov x1, x3
    mov x3, x5
    bl __rt_deref_if_reference
    mov x5, x3
    mov x3, x1
    cmp x5, #7
    b.eq _eir_main_iter_hash_value_inspect_box_55
    mov x0, x5
    mov x1, x3
    mov x2, x4
    bl __rt_mixed_from_value
    b _eir_main_iter_hash_value_boxed_56
_eir_main_iter_hash_value_inspect_box_55:
    str x3, [sp, #-16]!
    mov x0, x3
    bl __rt_heap_kind
    cmp x0, #5
    b.eq _eir_main_iter_hash_value_reuse_box_57
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
    b _eir_main_iter_hash_value_tagged_done_58
_eir_main_iter_hash_value_reuse_box_57:
    ldr x0, [sp], #16
    bl __rt_incref
_eir_main_iter_hash_value_tagged_done_58:
_eir_main_iter_hash_value_boxed_56:
    b _eir_main_iter_value_dyn_done_50
_eir_main_iter_value_dyn_object_49:
    sub x9, x29, #416
    ldr x0, [x9]
    ldr x10, [x0]
    adrp x9, _generator_class_id@PAGE
    add x9, x9, _generator_class_id@PAGEOFF
    ldr x11, [x9]
    cmp x10, x11
    b.ne _eir_main_interface_dispatch_not_generator_60
    bl __rt_gen_current
    b _eir_main_interface_dispatch_done_59
_eir_main_interface_dispatch_not_generator_60:
    ldr x10, [x0]
    adrp x11, _class_interface_ptrs@PAGE
    add x11, x11, _class_interface_ptrs@PAGEOFF
    ldr x11, [x11, x10, lsl #3]
    ldr x10, [x11]
    add x11, x11, #8
    mov x13, #2
_eir_main_interface_dispatch_scan_61:
    cbz x10, _eir_main_interface_dispatch_missing_63
    ldr x12, [x11]
    cmp x12, x13
    b.eq _eir_main_interface_dispatch_found_62
    add x11, x11, #16
    sub x10, x10, #1
    b _eir_main_interface_dispatch_scan_61
_eir_main_interface_dispatch_found_62:
    ldr x11, [x11, #8]
    ldr x11, [x11]
    blr x11
    b _eir_main_interface_dispatch_done_59
_eir_main_interface_dispatch_missing_63:
    mov x0, #0
_eir_main_interface_dispatch_done_59:
_eir_main_iter_value_dyn_done_50:
    sub x9, x29, #520
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=acquire
    sub x9, x29, #520
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #528
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=load_local
    sub x9, x29, #1232
    ldr x0, [x9]
    sub x9, x29, #536
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #536
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=13 col=14 end=13:20 op=store_local
    sub x9, x29, #528
    ldr x0, [x9]
    sub x9, x29, #1232
    str x0, [x9]
    ; @src line=13 col=14 end=13:20 op=release
    sub x9, x29, #520
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=14 col=20 end=14:31 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=13 end=14:19 op=load_local
    sub x9, x29, #1232
    ldr x0, [x9]
    sub x9, x29, #544
    str x0, [x9]
    ; @src line=14 col=24 end=14:31 op=const_i64
    mov x0, #42
    mov x21, x0
    ; @src line=14 col=20 end=14:31 op=strict_eq
    sub x9, x29, #544
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
    bl __rt_mixed_strict_eq
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir_main_if_then_8
    b _eir_main_foreach_next_4
    ; @block name=foreach.exit
_eir_main_foreach_exit_6:
    udf #0
    ; @block name=if.merge
_eir_main_if_merge_7:
    udf #0
    ; @block name=if.then
_eir_main_if_then_8:
    ; @src line=15 col=13 end=15:19 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=15 col=22 op=const_str
    adrp x1, _str_8@PAGE
    add x1, x1, _str_8@PAGEOFF
    mov x2, #6
    sub x9, x29, #576
    str x1, [x9]
    sub x9, x29, #568
    str x2, [x9]
    ; @src line=15 col=22 op=const_i64
    mov x0, #42
    mov x21, x0
    ; @src line=15 col=22 op=i_to_str
    mov x0, x21
    bl __rt_itoa
    sub x9, x29, #600
    str x1, [x9]
    sub x9, x29, #592
    str x2, [x9]
    ; @src line=15 col=22 op=str_concat
    sub x9, x29, #576
    ldr x1, [x9]
    sub x9, x29, #568
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
    ; @src line=15 col=22 op=release
    ; @src line=15 col=22 op=const_str
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #8
    sub x9, x29, #632
    str x1, [x9]
    sub x9, x29, #624
    str x2, [x9]
    ; @src line=15 col=22 op=str_concat
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
    ; @src line=15 col=22 op=release
    ; @src line=15 col=22 op=load_local
    sub x9, x29, #1208
    ldr x0, [x9]
    sub x9, x29, #656
    str x0, [x9]
    ; @src line=15 col=22 op=cast
    sub x9, x29, #656
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_64
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_object_64:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_67
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_68
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_69
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_70
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_71
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_72
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_73
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_74
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_75
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_76
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_77
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_78
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_79
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_80
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_81
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_82
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_83
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_84
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_85
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_86
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_87
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_88
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_89
    mov x10, #45
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_90
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_91
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_92
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_93
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_94
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_95
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_96
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_97
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_98
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_99
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_100
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_101
    mov x10, #65
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_102
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_103
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_104
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_105
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_106
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_107
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_108
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_109
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_110
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_111
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_112
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_113
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_114
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_115
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_116
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_117
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_118
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_119
    mov x10, #96
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_120
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_121
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_122
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_123
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_124
    b _eir_main_mixed_string_no_match_65
_eir_main_mixed_string_Exception_67:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateException_68:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateMalformedPeriodStringException_69:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionFunctionAbstract_70:
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
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateError_72:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_SplFileInfo_73:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_PharFileInfo_74:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_Phar_75:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_TypeError_76:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionClassConstant_77:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_LogicException_78:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_InvalidArgumentException_79:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionUnionType_80:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_RuntimeException_81:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_FiberError_82:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionEnumBackedCase_83:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionEnum_84:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateInvalidOperationException_85:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DirectoryIterator_86:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_FilesystemIterator_87:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_RecursiveDirectoryIterator_88:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_SplFileObject_89:
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
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateUnknownException_91:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionClass_92:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionMethod_93:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionProperty_94:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionObject_95:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_UnexpectedValueException_96:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateMalformedStringException_97:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateMalformedIntervalStringException_98:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_UnhandledMatchError_99:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_SplTempFileObject_100:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ArithmeticError_101:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_UnderflowException_102:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_CachingIterator_103:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_GlobIterator_104:
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
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionException_106:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionIntersectionType_107:
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
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionFunction_109:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_PharData_110:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_JsonException_111:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_OverflowException_112:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateObjectError_113:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_RecursiveCachingIterator_114:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionEnumUnitCase_116:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DateRangeError_117:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ValueError_118:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_OutOfRangeException_119:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_ReflectionNamedType_120:
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_OutOfBoundsException_121:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_LengthException_122:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
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
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_DomainException_124:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_66
_eir_main_mixed_string_no_match_65:
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_66:
    sub x9, x29, #672
    str x1, [x9]
    sub x9, x29, #664
    str x2, [x9]
    ; @src line=15 col=22 op=str_concat
    sub x9, x29, #648
    ldr x1, [x9]
    sub x9, x29, #640
    ldr x2, [x9]
    sub x9, x29, #672
    ldr x3, [x9]
    sub x9, x29, #664
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #688
    str x1, [x9]
    sub x9, x29, #680
    str x2, [x9]
    ; @src line=15 col=22 op=release
    ; @src line=15 col=22 op=release
    sub x9, x29, #672
    ldr x1, [x9]
    sub x9, x29, #664
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=15 col=22 op=const_str
    adrp x1, _str_10@PAGE
    add x1, x1, _str_10@PAGEOFF
    mov x2, #6
    sub x9, x29, #704
    str x1, [x9]
    sub x9, x29, #696
    str x2, [x9]
    ; @src line=15 col=22 op=str_concat
    sub x9, x29, #688
    ldr x1, [x9]
    sub x9, x29, #680
    ldr x2, [x9]
    sub x9, x29, #704
    ldr x3, [x9]
    sub x9, x29, #696
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #720
    str x1, [x9]
    sub x9, x29, #712
    str x2, [x9]
    ; @src line=15 col=22 op=release
    ; @src line=15 col=22 op=load_local
    sub x9, x29, #1224
    ldr x0, [x9]
    sub x9, x29, #728
    str x0, [x9]
    ; @src line=15 col=22 op=cast
    sub x9, x29, #728
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_125
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_object_125:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_128
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_129
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_130
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_131
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_132
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_133
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_134
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_135
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_136
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_137
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_138
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_139
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_140
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_141
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_142
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_143
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_144
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_145
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_146
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_147
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_148
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_149
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_150
    mov x10, #45
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_151
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_152
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_153
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_154
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_155
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_156
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_157
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_158
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_159
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_160
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_161
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_162
    mov x10, #65
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_163
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_164
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_165
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_166
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_167
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_168
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_169
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_170
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_171
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_172
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_173
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_174
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_175
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_176
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_177
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_178
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_179
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_180
    mov x10, #96
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_181
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_182
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_183
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_184
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_185
    b _eir_main_mixed_string_no_match_126
_eir_main_mixed_string_Exception_128:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateException_129:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateMalformedPeriodStringException_130:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionFunctionAbstract_131:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_Error_132:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateError_133:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_SplFileInfo_134:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_PharFileInfo_135:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_Phar_136:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_TypeError_137:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionClassConstant_138:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_LogicException_139:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_InvalidArgumentException_140:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionUnionType_141:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_RuntimeException_142:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_FiberError_143:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionEnumBackedCase_144:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionEnum_145:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateInvalidOperationException_146:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DirectoryIterator_147:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_FilesystemIterator_148:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_RecursiveDirectoryIterator_149:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_SplFileObject_150:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_BadFunctionCallException_151:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateUnknownException_152:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionClass_153:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionMethod_154:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionProperty_155:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionObject_156:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_UnexpectedValueException_157:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateMalformedStringException_158:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateMalformedIntervalStringException_159:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_UnhandledMatchError_160:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_SplTempFileObject_161:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ArithmeticError_162:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_UnderflowException_163:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_GlobIterator_165:
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
    b _eir_main_mixed_string_done_127
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionException_167:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionIntersectionType_168:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateInvalidTimeZoneException_169:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionFunction_170:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_PharData_171:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_JsonException_172:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_OverflowException_173:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateObjectError_174:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_RecursiveCachingIterator_175:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_BadMethodCallException_176:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionEnumUnitCase_177:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DateRangeError_178:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ValueError_179:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_OutOfRangeException_180:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_ReflectionNamedType_181:
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_OutOfBoundsException_182:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
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
    b _eir_main_mixed_string_done_127
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
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_DomainException_185:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_127
_eir_main_mixed_string_no_match_126:
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_127:
    sub x9, x29, #744
    str x1, [x9]
    sub x9, x29, #736
    str x2, [x9]
    ; @src line=15 col=22 op=str_concat
    sub x9, x29, #720
    ldr x1, [x9]
    sub x9, x29, #712
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
    ; @src line=15 col=22 op=release
    ; @src line=15 col=22 op=release
    sub x9, x29, #744
    ldr x1, [x9]
    sub x9, x29, #736
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=15 col=13 end=15:19 op=acquire
    sub x9, x29, #760
    ldr x1, [x9]
    sub x9, x29, #752
    ldr x2, [x9]
    bl __rt_str_persist
    sub x9, x29, #776
    str x1, [x9]
    sub x9, x29, #768
    str x2, [x9]
    ; @src line=15 col=13 end=15:19 op=load_local
    sub x9, x29, #1200
    ldr x1, [x9]
    sub x9, x29, #1192
    ldr x2, [x9]
    sub x9, x29, #792
    str x1, [x9]
    sub x9, x29, #784
    str x2, [x9]
    ; @src line=15 col=13 end=15:19 op=release
    sub x9, x29, #792
    ldr x1, [x9]
    sub x9, x29, #784
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=15 col=13 end=15:19 op=store_local
    sub x9, x29, #776
    ldr x1, [x9]
    sub x9, x29, #768
    ldr x2, [x9]
    sub x9, x29, #1200
    str x1, [x9]
    sub x9, x29, #1192
    str x2, [x9]
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=15 col=13 end=15:19 op=release
    ; @src line=16 col=13 end=16:17 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_goto_label_10
    ; @block name=if.else
_eir_main_if_else_9:
    ; @src line=14 col=20 end=14:31 op=nop
    udf #0
    ; @block name=goto.label
_eir_main_goto_label_10:
    ; @src line=21 col=1 end=21:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=1 end=21:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=6 end=21:12 op=load_local
    sub x9, x29, #1200
    ldr x1, [x9]
    sub x9, x29, #1192
    ldr x2, [x9]
    sub x9, x29, #808
    str x1, [x9]
    sub x9, x29, #800
    str x2, [x9]
    ; @src line=21 col=1 end=21:5 op=echo_value
    sub x9, x29, #808
    ldr x1, [x9]
    sub x9, x29, #800
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=21 col=1 end=21:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=14 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #1
    sub x9, x29, #824
    str x1, [x9]
    sub x9, x29, #816
    str x2, [x9]
    ; @src line=21 col=1 end=21:5 op=echo_value
    sub x9, x29, #824
    ldr x1, [x9]
    sub x9, x29, #816
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=25 col=1 end=25:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=1 end=25:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=15 end=25:19 op=const_null
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=25 col=6 end=25:20 op=call
    sub sp, sp, #16
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    mov x9, sp
    str x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    bl _fn_describe
    sub x9, x29, #848
    str x1, [x9]
    sub x9, x29, #840
    str x2, [x9]
    ldr x0, [sp]
    bl __rt_decref_mixed
    add sp, sp, #16
    ; @src line=25 col=1 end=25:5 op=echo_value
    sub x9, x29, #848
    ldr x1, [x9]
    sub x9, x29, #840
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=25 col=1 end=25:5 op=release
    sub x9, x29, #848
    ldr x1, [x9]
    sub x9, x29, #840
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=25 col=1 end=25:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=22 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #1
    sub x9, x29, #864
    str x1, [x9]
    sub x9, x29, #856
    str x2, [x9]
    ; @src line=25 col=1 end=25:5 op=echo_value
    sub x9, x29, #864
    ldr x1, [x9]
    sub x9, x29, #856
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=26 col=1 end=26:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=1 end=26:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=15 op=const_str
    adrp x1, _str_12@PAGE
    add x1, x1, _str_12@PAGEOFF
    mov x2, #6
    sub x9, x29, #880
    str x1, [x9]
    sub x9, x29, #872
    str x2, [x9]
    ; @src line=26 col=6 end=26:24 op=call
    sub sp, sp, #16
    sub x9, x29, #880
    ldr x1, [x9]
    sub x9, x29, #872
    ldr x2, [x9]
    mov x0, #1
    bl __rt_mixed_from_value
    mov x9, sp
    str x0, [x9]
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    bl _fn_describe
    sub x9, x29, #896
    str x1, [x9]
    sub x9, x29, #888
    str x2, [x9]
    ldr x0, [sp]
    bl __rt_decref_mixed
    add sp, sp, #16
    ; @src line=26 col=1 end=26:5 op=echo_value
    sub x9, x29, #896
    ldr x1, [x9]
    sub x9, x29, #888
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=26 col=1 end=26:5 op=release
    sub x9, x29, #896
    ldr x1, [x9]
    sub x9, x29, #888
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=26 col=1 end=26:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=26 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #1
    sub x9, x29, #912
    str x1, [x9]
    sub x9, x29, #904
    str x2, [x9]
    ; @src line=26 col=1 end=26:5 op=echo_value
    sub x9, x29, #912
    ldr x1, [x9]
    sub x9, x29, #904
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=28 col=1 end=28:9 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=28 col=1 end=28:9 op=nop
    ; @src line=45 col=1 end=45:9 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=45 col=12 end=45:13 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=45 col=1 end=45:9 op=store_local
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    sub x9, x29, #1240
    str x0, [x9]
    ; @src line=46 col=1 end=46:6 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_goto_label_11
    ; @block name=goto.label
_eir_main_goto_label_11:
    ; @src line=47 col=1 end=47:9 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=47 col=1 end=47:9 op=load_local
    sub x9, x29, #1240
    ldr x0, [x9]
    bl __rt_mixed_cast_int
    mov x21, x0
    ; @src line=47 col=1 end=47:9 op=const_i64
    mov x0, #1
    mov x22, x0
    ; @src line=47 col=1 end=47:9 op=ichecked_add
    mov x0, x21
    mov x10, x22
    mov x1, x10
    bl __rt_int_add_checked
    sub x9, x29, #944
    str x0, [x9]
    ; @src line=47 col=1 end=47:9 op=acquire
    sub x9, x29, #944
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    sub x9, x29, #952
    str x0, [x9]
    ; @src line=47 col=1 end=47:9 op=load_local
    sub x9, x29, #1240
    ldr x0, [x9]
    sub x9, x29, #960
    str x0, [x9]
    ; @src line=47 col=1 end=47:9 op=release
    sub x9, x29, #960
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=47 col=1 end=47:9 op=store_local
    sub x9, x29, #952
    ldr x0, [x9]
    sub x9, x29, #1240
    str x0, [x9]
    ; @src line=47 col=1 end=47:9 op=release
    sub x9, x29, #944
    ldr x0, [x9]
    bl __rt_decref_mixed
    ; @src line=48 col=1 end=48:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=48 col=6 op=const_str
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #8
    sub x9, x29, #976
    str x1, [x9]
    sub x9, x29, #968
    str x2, [x9]
    ; @src line=48 col=6 op=load_local
    sub x9, x29, #1240
    ldr x0, [x9]
    sub x9, x29, #984
    str x0, [x9]
    ; @src line=48 col=6 op=cast
    sub x9, x29, #984
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_186
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_object_186:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_189
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_190
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_191
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_192
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_193
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_194
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_195
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_196
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_197
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_198
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_199
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_200
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_201
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_202
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_203
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_204
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_205
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_206
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_207
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_208
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_209
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_210
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_211
    mov x10, #45
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_212
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_213
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_214
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_215
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_216
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_217
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_218
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_219
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_220
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_221
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_222
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_223
    mov x10, #65
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_224
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_225
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_226
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_227
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_228
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_229
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_230
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_231
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_232
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_233
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_234
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_235
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_236
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_237
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_238
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_239
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_240
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_241
    mov x10, #96
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_242
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_243
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_244
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_245
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_246
    b _eir_main_mixed_string_no_match_187
_eir_main_mixed_string_Exception_189:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateException_190:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateMalformedPeriodStringException_191:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionFunctionAbstract_192:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_Error_193:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateError_194:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_SplFileInfo_195:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_PharFileInfo_196:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_Phar_197:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_TypeError_198:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionClassConstant_199:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_LogicException_200:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_InvalidArgumentException_201:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionUnionType_202:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_RuntimeException_203:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_FiberError_204:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionEnumBackedCase_205:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionEnum_206:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateInvalidOperationException_207:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
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
    b _eir_main_mixed_string_done_188
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_RecursiveDirectoryIterator_210:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_SplFileObject_211:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_BadFunctionCallException_212:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateUnknownException_213:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionClass_214:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionMethod_215:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionProperty_216:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionObject_217:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_UnexpectedValueException_218:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateMalformedStringException_219:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateMalformedIntervalStringException_220:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_UnhandledMatchError_221:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_SplTempFileObject_222:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ArithmeticError_223:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_UnderflowException_224:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_CachingIterator_225:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_GlobIterator_226:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionParameter_227:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionException_228:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionIntersectionType_229:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateInvalidTimeZoneException_230:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionFunction_231:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_PharData_232:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_JsonException_233:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_OverflowException_234:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateObjectError_235:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_RecursiveCachingIterator_236:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_BadMethodCallException_237:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionEnumUnitCase_238:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DateRangeError_239:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ValueError_240:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_OutOfRangeException_241:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_ReflectionNamedType_242:
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
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_OutOfBoundsException_243:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_LengthException_244:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_RangeException_245:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_DomainException_246:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_188
_eir_main_mixed_string_no_match_187:
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_188:
    sub x9, x29, #1000
    str x1, [x9]
    sub x9, x29, #992
    str x2, [x9]
    ; @src line=48 col=6 op=str_concat
    sub x9, x29, #976
    ldr x1, [x9]
    sub x9, x29, #968
    ldr x2, [x9]
    sub x9, x29, #1000
    ldr x3, [x9]
    sub x9, x29, #992
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1016
    str x1, [x9]
    sub x9, x29, #1008
    str x2, [x9]
    ; @src line=48 col=6 op=release
    sub x9, x29, #1000
    ldr x1, [x9]
    sub x9, x29, #992
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=48 col=6 op=const_str
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #1
    sub x9, x29, #1032
    str x1, [x9]
    sub x9, x29, #1024
    str x2, [x9]
    ; @src line=48 col=6 op=str_concat
    sub x9, x29, #1016
    ldr x1, [x9]
    sub x9, x29, #1008
    ldr x2, [x9]
    sub x9, x29, #1032
    ldr x3, [x9]
    sub x9, x29, #1024
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #1048
    str x1, [x9]
    sub x9, x29, #1040
    str x2, [x9]
    ; @src line=48 col=6 op=release
    ; @src line=48 col=1 end=48:5 op=echo_value
    sub x9, x29, #1048
    ldr x1, [x9]
    sub x9, x29, #1040
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=48 col=1 end=48:5 op=release
    ; @src line=49 col=14 end=49:17 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=49 col=5 end=49:13 op=load_local
    sub x9, x29, #1240
    ldr x0, [x9]
    sub x9, x29, #1056
    str x0, [x9]
    ; @src line=49 col=16 end=49:17 op=const_i64
    mov x0, #3
    mov x22, x0
    ; @src line=49 col=14 end=49:17 op=str_cmp
    sub x9, x29, #1056
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
    bl __rt_php_compare
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #32
    cmp x0, #0
    cset x0, lt
    mov x22, x0
    mov x0, x22
    cbnz x0, _eir_main_if_then_13
    b _eir_main_if_merge_12
    ; @block name=if.merge
_eir_main_if_merge_12:
    ; @src line=52 col=1 end=52:5 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=52 col=6 op=const_str
    adrp x1, _str_14@PAGE
    add x1, x1, _str_14@PAGEOFF
    mov x2, #16
    sub x9, x29, #1088
    str x1, [x9]
    sub x9, x29, #1080
    str x2, [x9]
    ; @src line=52 col=6 op=load_local
    sub x9, x29, #1240
    ldr x0, [x9]
    sub x9, x29, #1096
    str x0, [x9]
    ; @src line=52 col=6 op=cast
    sub x9, x29, #1096
    ldr x0, [x9]
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir_main_mixed_string_object_247
    ldr x0, [sp], #16
    bl __rt_mixed_cast_string
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_object_247:
    ldr x10, [sp], #16
    mov x19, x1
    ldr x9, [x19]
    mov x10, #1
    cmp x9, x10
    b.eq _eir_main_mixed_string_Exception_250
    mov x10, #2
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateException_251
    mov x10, #3
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedPeriodStringException_252
    mov x10, #9
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunctionAbstract_253
    mov x10, #10
    cmp x9, x10
    b.eq _eir_main_mixed_string_Error_254
    mov x10, #11
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateError_255
    mov x10, #12
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileInfo_256
    mov x10, #13
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharFileInfo_257
    mov x10, #14
    cmp x9, x10
    b.eq _eir_main_mixed_string_Phar_258
    mov x10, #21
    cmp x9, x10
    b.eq _eir_main_mixed_string_TypeError_259
    mov x10, #22
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClassConstant_260
    mov x10, #23
    cmp x9, x10
    b.eq _eir_main_mixed_string_LogicException_261
    mov x10, #24
    cmp x9, x10
    b.eq _eir_main_mixed_string_InvalidArgumentException_262
    mov x10, #28
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionUnionType_263
    mov x10, #29
    cmp x9, x10
    b.eq _eir_main_mixed_string_RuntimeException_264
    mov x10, #31
    cmp x9, x10
    b.eq _eir_main_mixed_string_FiberError_265
    mov x10, #33
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumBackedCase_266
    mov x10, #35
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnum_267
    mov x10, #36
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidOperationException_268
    mov x10, #40
    cmp x9, x10
    b.eq _eir_main_mixed_string_DirectoryIterator_269
    mov x10, #41
    cmp x9, x10
    b.eq _eir_main_mixed_string_FilesystemIterator_270
    mov x10, #42
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveDirectoryIterator_271
    mov x10, #44
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplFileObject_272
    mov x10, #45
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadFunctionCallException_273
    mov x10, #49
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateUnknownException_274
    mov x10, #51
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionClass_275
    mov x10, #52
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionMethod_276
    mov x10, #53
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionProperty_277
    mov x10, #54
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionObject_278
    mov x10, #57
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnexpectedValueException_279
    mov x10, #58
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedStringException_280
    mov x10, #61
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateMalformedIntervalStringException_281
    mov x10, #62
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnhandledMatchError_282
    mov x10, #63
    cmp x9, x10
    b.eq _eir_main_mixed_string_SplTempFileObject_283
    mov x10, #64
    cmp x9, x10
    b.eq _eir_main_mixed_string_ArithmeticError_284
    mov x10, #65
    cmp x9, x10
    b.eq _eir_main_mixed_string_UnderflowException_285
    mov x10, #66
    cmp x9, x10
    b.eq _eir_main_mixed_string_CachingIterator_286
    mov x10, #67
    cmp x9, x10
    b.eq _eir_main_mixed_string_GlobIterator_287
    mov x10, #68
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionParameter_288
    mov x10, #70
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionException_289
    mov x10, #72
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionIntersectionType_290
    mov x10, #73
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateInvalidTimeZoneException_291
    mov x10, #76
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionFunction_292
    mov x10, #77
    cmp x9, x10
    b.eq _eir_main_mixed_string_PharData_293
    mov x10, #81
    cmp x9, x10
    b.eq _eir_main_mixed_string_JsonException_294
    mov x10, #83
    cmp x9, x10
    b.eq _eir_main_mixed_string_OverflowException_295
    mov x10, #85
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateObjectError_296
    mov x10, #86
    cmp x9, x10
    b.eq _eir_main_mixed_string_RecursiveCachingIterator_297
    mov x10, #88
    cmp x9, x10
    b.eq _eir_main_mixed_string_BadMethodCallException_298
    mov x10, #89
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionEnumUnitCase_299
    mov x10, #92
    cmp x9, x10
    b.eq _eir_main_mixed_string_DateRangeError_300
    mov x10, #94
    cmp x9, x10
    b.eq _eir_main_mixed_string_ValueError_301
    mov x10, #95
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfRangeException_302
    mov x10, #96
    cmp x9, x10
    b.eq _eir_main_mixed_string_ReflectionNamedType_303
    mov x10, #97
    cmp x9, x10
    b.eq _eir_main_mixed_string_OutOfBoundsException_304
    mov x10, #99
    cmp x9, x10
    b.eq _eir_main_mixed_string_LengthException_305
    mov x10, #100
    cmp x9, x10
    b.eq _eir_main_mixed_string_RangeException_306
    mov x10, #101
    cmp x9, x10
    b.eq _eir_main_mixed_string_DomainException_307
    b _eir_main_mixed_string_no_match_248
_eir_main_mixed_string_Exception_250:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateException_251:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateMalformedPeriodStringException_252:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionFunctionAbstract_253:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_Error_254:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateError_255:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_SplFileInfo_256:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_PharFileInfo_257:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_Phar_258:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_TypeError_259:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionClassConstant_260:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_LogicException_261:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_InvalidArgumentException_262:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionUnionType_263:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_RuntimeException_264:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_FiberError_265:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionEnumBackedCase_266:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionEnum_267:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateInvalidOperationException_268:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DirectoryIterator_269:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_FilesystemIterator_270:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_RecursiveDirectoryIterator_271:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_SplFileObject_272:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_BadFunctionCallException_273:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateUnknownException_274:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionClass_275:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionMethod_276:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionProperty_277:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionObject_278:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_UnexpectedValueException_279:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateMalformedStringException_280:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateMalformedIntervalStringException_281:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_UnhandledMatchError_282:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_SplTempFileObject_283:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ArithmeticError_284:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_UnderflowException_285:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_GlobIterator_287:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionParameter_288:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionException_289:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionIntersectionType_290:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateInvalidTimeZoneException_291:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionFunction_292:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_PharData_293:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_JsonException_294:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_OverflowException_295:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateObjectError_296:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_RecursiveCachingIterator_297:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_BadMethodCallException_298:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionEnumUnitCase_299:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DateRangeError_300:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ValueError_301:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_OutOfRangeException_302:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_ReflectionNamedType_303:
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
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_OutOfBoundsException_304:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_LengthException_305:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_RangeException_306:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_DomainException_307:
    mov x0, x19
    str x0, [sp, #-16]!
    ldr x0, [sp]
    add sp, sp, #16
    ldr x10, [x0]
    adrp x9, _class_vtable_ptrs@PAGE
    add x9, x9, _class_vtable_ptrs@PAGEOFF
    ldr x9, [x9, x10, lsl #3]
    ldr x9, [x9, #64]
    blr x9
    b _eir_main_mixed_string_done_249
_eir_main_mixed_string_no_match_248:
    mov x0, #2
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #53
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_mixed_string_done_249:
    sub x9, x29, #1112
    str x1, [x9]
    sub x9, x29, #1104
    str x2, [x9]
    ; @src line=52 col=6 op=str_concat
    sub x9, x29, #1088
    ldr x1, [x9]
    sub x9, x29, #1080
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
    ; @src line=52 col=6 op=release
    sub x9, x29, #1112
    ldr x1, [x9]
    sub x9, x29, #1104
    ldr x2, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; @src line=52 col=6 op=const_str
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #10
    sub x9, x29, #1144
    str x1, [x9]
    sub x9, x29, #1136
    str x2, [x9]
    ; @src line=52 col=6 op=str_concat
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
    ; @src line=52 col=6 op=release
    ; @src line=52 col=1 end=52:5 op=echo_value
    sub x9, x29, #1160
    ldr x1, [x9]
    sub x9, x29, #1152
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=52 col=1 end=52:5 op=release

    ; epilogue + exit(0)
    bl __rt_ob_flush_all
    ; epilogue cleanup $grid
    sub x9, x29, #1184
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_308
    bl __rt_decref_array
_eir_main_main_refcounted_cleanup_done_308:
    ; epilogue cleanup $found
    sub x9, x29, #1200
    ldr x1, [x9]
    mov x0, x1
    bl __rt_heap_free_safe
    ; epilogue cleanup $row
    sub x9, x29, #1208
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_309
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_309:
    ; epilogue cleanup $cells
    sub x9, x29, #1216
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_310
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_310:
    ; epilogue cleanup $col
    sub x9, x29, #1224
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_311
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_311:
    ; epilogue cleanup $value
    sub x9, x29, #1232
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_312
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_312:
    ; epilogue cleanup $attempt
    sub x9, x29, #1240
    ldr x0, [x9]
    cbz x0, _eir_main_main_refcounted_cleanup_done_313
    bl __rt_decref_mixed
_eir_main_main_refcounted_cleanup_done_313:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #1248
    ldr x21, [x9]
    sub x9, x29, #1256
    ldr x22, [x9]
    mov x9, sp
    add x9, x9, #1264
    ldp x29, x30, [x9]
    add sp, sp, #1280
    ; teardown: deep-free the persistent ini directive table (guarded)
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    ldr x9, [x9]
    cbz x9, _eir_main_ini_teardown_skip_314
    adrp x9, _rt_ini_table@PAGE
    add x9, x9, _rt_ini_table@PAGEOFF
    ldr x0, [x9]
    bl __rt_hash_free_deep
    mov x9, #0
    adrp x9, _rt_ini_table_init@PAGE
    add x9, x9, _rt_ini_table_init@PAGEOFF
    str x9, [x9]
_eir_main_ini_teardown_skip_314:
    bl __rt_ob_flush_all
    mov x0, #0
    mov x16, #1
    svc #0x80
    ; @block name=if.then
_eir_main_if_then_13:
    ; @src line=50 col=5 end=50:9 op=concat_reset
    sub x9, x29, #1264
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_goto_label_11
    ; @block name=if.else
_eir_main_if_else_14:
    ; @src line=49 col=14 end=49:17 op=nop
    udf #0
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
    .ascii "missing value"
.globl _str_1
_str_1:
    .ascii "default"
.globl _str_2
_str_2:
    .ascii "describing: "
.globl _str_3
_str_3:
    .ascii "Fatal error: Object could not be converted to string\n"
.globl _str_4
_str_4:
    .ascii "UTC"
.globl _str_5
_str_5:
    .ascii ""
.globl _str_7
_str_7:
    .ascii "not found"
.globl _str_8
_str_8:
    .ascii "found "
.globl _str_9
_str_9:
    .ascii " at row "
.globl _str_10
_str_10:
    .ascii ", col "
.globl _str_11
_str_11:
    .ascii "\n"
.globl _str_12
_str_12:
    .ascii "widget"
.globl _str_13
_str_13:
    .ascii "attempt "
.globl _str_14
_str_14:
    .ascii "succeeded after "
.globl _str_15
_str_15:
    .ascii " attempts\n"
.p2align 3
.globl _float_6
_float_6:
    .quad 0x0000000000000000

.comm _enum_case_PropertyHookType_Get, 8, 3
.comm _enum_case_PropertyHookType_Set, 8, 3
.comm _enum_case_SortDirection_Ascending, 8, 3
.comm _enum_case_SortDirection_Descending, 8, 3
.data
.p2align 3
.globl _callable_user_fn_name_0
_callable_user_fn_name_0:
    .ascii "describe"
.globl _callable_user_fn_name_1
_callable_user_fn_name_1:
    .ascii "main"
.p2align 3
.globl _callable_user_function_count
_callable_user_function_count:
    .quad 2
.globl _callable_user_function_table
_callable_user_function_table:
    .quad _callable_user_fn_name_0
    .quad 8
    .quad 0
    .quad _callable_user_fn_name_1
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
    .quad _instanceof_name_class_10
    .quad 5
    .quad 10
    .quad 0
    .quad _instanceof_name_class_abs_10
    .quad 6
    .quad 10
    .quad 0
    .quad _instanceof_name_class_21
    .quad 9
    .quad 21
    .quad 0
    .quad _instanceof_name_class_abs_21
    .quad 10
    .quad 21
    .quad 0
    .quad _instanceof_name_class_23
    .quad 14
    .quad 23
    .quad 0
    .quad _instanceof_name_class_abs_23
    .quad 15
    .quad 23
    .quad 0
    .quad _instanceof_name_class_24
    .quad 24
    .quad 24
    .quad 0
    .quad _instanceof_name_class_abs_24
    .quad 25
    .quad 24
    .quad 0
    .quad _instanceof_name_class_29
    .quad 16
    .quad 29
    .quad 0
    .quad _instanceof_name_class_abs_29
    .quad 17
    .quad 29
    .quad 0
    .quad _instanceof_name_class_43
    .quad 8
    .quad 43
    .quad 0
    .quad _instanceof_name_class_abs_43
    .quad 9
    .quad 43
    .quad 0
    .quad _instanceof_name_class_62
    .quad 19
    .quad 62
    .quad 0
    .quad _instanceof_name_class_abs_62
    .quad 20
    .quad 62
    .quad 0
    .quad _instanceof_name_class_64
    .quad 15
    .quad 64
    .quad 0
    .quad _instanceof_name_class_abs_64
    .quad 16
    .quad 64
    .quad 0
    .quad _instanceof_name_class_70
    .quad 19
    .quad 70
    .quad 0
    .quad _instanceof_name_class_abs_70
    .quad 20
    .quad 70
    .quad 0
    .quad _instanceof_name_class_81
    .quad 13
    .quad 81
    .quad 0
    .quad _instanceof_name_class_abs_81
    .quad 14
    .quad 81
    .quad 0
    .quad _instanceof_name_class_94
    .quad 10
    .quad 94
    .quad 0
    .quad _instanceof_name_class_abs_94
    .quad 11
    .quad 94
    .quad 0
    .quad _instanceof_name_class_95
    .quad 19
    .quad 95
    .quad 0
    .quad _instanceof_name_class_abs_95
    .quad 20
    .quad 95
    .quad 0
    .quad _instanceof_name_class_97
    .quad 20
    .quad 97
    .quad 0
    .quad _instanceof_name_class_abs_97
    .quad 21
    .quad 97
    .quad 0
    .quad _instanceof_name_interface_8
    .quad 10
    .quad 8
    .quad 1
    .quad _instanceof_name_interface_abs_8
    .quad 11
    .quad 8
    .quad 1
    .quad _instanceof_name_interface_11
    .quad 9
    .quad 11
    .quad 1
    .quad _instanceof_name_interface_abs_11
    .quad 10
    .quad 11
    .quad 1
.globl _instanceof_name_class_1
_instanceof_name_class_1:
    .ascii "Exception"
.globl _instanceof_name_class_abs_1
_instanceof_name_class_abs_1:
    .ascii "\\Exception"
.globl _instanceof_name_class_10
_instanceof_name_class_10:
    .ascii "Error"
.globl _instanceof_name_class_abs_10
_instanceof_name_class_abs_10:
    .ascii "\\Error"
.globl _instanceof_name_class_21
_instanceof_name_class_21:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_21
_instanceof_name_class_abs_21:
    .ascii "\\TypeError"
.globl _instanceof_name_class_23
_instanceof_name_class_23:
    .ascii "LogicException"
.globl _instanceof_name_class_abs_23
_instanceof_name_class_abs_23:
    .ascii "\\LogicException"
.globl _instanceof_name_class_24
_instanceof_name_class_24:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_24
_instanceof_name_class_abs_24:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_29
_instanceof_name_class_29:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_29
_instanceof_name_class_abs_29:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_43
_instanceof_name_class_43:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_43
_instanceof_name_class_abs_43:
    .ascii "\\stdClass"
.globl _instanceof_name_class_62
_instanceof_name_class_62:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_62
_instanceof_name_class_abs_62:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_class_64
_instanceof_name_class_64:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_64
_instanceof_name_class_abs_64:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_70
_instanceof_name_class_70:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_70
_instanceof_name_class_abs_70:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_81
_instanceof_name_class_81:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_81
_instanceof_name_class_abs_81:
    .ascii "\\JsonException"
.globl _instanceof_name_class_94
_instanceof_name_class_94:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_94
_instanceof_name_class_abs_94:
    .ascii "\\ValueError"
.globl _instanceof_name_class_95
_instanceof_name_class_95:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_95
_instanceof_name_class_abs_95:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_97
_instanceof_name_class_97:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_97
_instanceof_name_class_abs_97:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_interface_8
_instanceof_name_interface_8:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_8
_instanceof_name_interface_abs_8:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_11
_instanceof_name_interface_11:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_11
_instanceof_name_interface_abs_11:
    .ascii "\\Throwable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 98
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
    .quad _class_name_10
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
    .quad _class_name_21
    .quad 9
    .quad _class_name_missing
    .quad 0
    .quad _class_name_23
    .quad 14
    .quad _class_name_24
    .quad 24
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_29
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
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
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_64
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
    .quad _class_name_70
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
    .quad _class_name_81
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
    .quad _class_name_94
    .quad 10
    .quad _class_name_95
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_97
    .quad 20
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_1
_class_name_1:
    .ascii "Exception"
.globl _class_name_10
_class_name_10:
    .ascii "Error"
.globl _class_name_21
_class_name_21:
    .ascii "TypeError"
.globl _class_name_23
_class_name_23:
    .ascii "LogicException"
.globl _class_name_24
_class_name_24:
    .ascii "InvalidArgumentException"
.globl _class_name_29
_class_name_29:
    .ascii "RuntimeException"
.globl _class_name_43
_class_name_43:
    .ascii "stdClass"
.globl _class_name_62
_class_name_62:
    .ascii "UnhandledMatchError"
.globl _class_name_64
_class_name_64:
    .ascii "ArithmeticError"
.globl _class_name_70
_class_name_70:
    .ascii "ReflectionException"
.globl _class_name_81
_class_name_81:
    .ascii "JsonException"
.globl _class_name_94
_class_name_94:
    .ascii "ValueError"
.globl _class_name_95
_class_name_95:
    .ascii "OutOfRangeException"
.globl _class_name_97
_class_name_97:
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
    .quad 55
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 31
.globl _generator_class_id
_generator_class_id:
    .quad 78
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 16
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 69
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 17
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 79
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 10
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 23
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 29
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 95
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 97
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 24
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 21
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 94
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 70
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 64
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_8
    .quad _interface_methods_11
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
    .quad _class_interfaces_21
    .quad _class_interfaces_missing
    .quad _class_interfaces_23
    .quad _class_interfaces_24
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
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_62
    .quad _class_interfaces_missing
    .quad _class_interfaces_64
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_70
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
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
    .quad _class_interfaces_94
    .quad _class_interfaces_95
    .quad _class_interfaces_missing
    .quad _class_interfaces_97
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
    .quad _class_json_desc_21
    .quad _class_json_desc_missing
    .quad _class_json_desc_23
    .quad _class_json_desc_24
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
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_62
    .quad _class_json_desc_missing
    .quad _class_json_desc_64
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_70
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
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
    .quad _class_json_desc_94
    .quad _class_json_desc_95
    .quad _class_json_desc_missing
    .quad _class_json_desc_97
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 81
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 10
    .quad -1
    .quad 1
    .quad 23
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
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad -1
    .quad 10
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
    .quad 29
    .quad -1
    .quad -1
    .quad -1
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
    .quad 23
    .quad -1
    .quad 29
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
    .quad 72
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
    .quad 72
    .quad 0
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 98
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
    .quad _class_gc_desc_21
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_23
    .quad _class_gc_desc_24
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
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_62
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_64
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_70
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
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
    .quad _class_gc_desc_94
    .quad _class_gc_desc_95
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_97
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
    .quad _class_vtable_21
    .quad _class_vtable_missing
    .quad _class_vtable_23
    .quad _class_vtable_24
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
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_62
    .quad _class_vtable_missing
    .quad _class_vtable_64
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_70
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
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
    .quad _class_vtable_94
    .quad _class_vtable_95
    .quad _class_vtable_missing
    .quad _class_vtable_97
.globl _class_destruct_count
_class_destruct_count:
    .quad 98
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
.globl _class_clone_count
_class_clone_count:
    .quad 98
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
    .quad _class_propinit_21
    .quad 0
    .quad _class_propinit_23
    .quad _class_propinit_24
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_29
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_62
    .quad 0
    .quad _class_propinit_64
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_70
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_propinit_94
    .quad _class_propinit_95
    .quad 0
    .quad _class_propinit_97
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
    .quad _class_serprop_21
    .quad _class_serprop_missing
    .quad _class_serprop_23
    .quad _class_serprop_24
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
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_62
    .quad _class_serprop_missing
    .quad _class_serprop_64
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_70
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
    .quad _class_serprop_94
    .quad _class_serprop_95
    .quad _class_serprop_missing
    .quad _class_serprop_97
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
    .quad _class_static_vtable_21
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_23
    .quad _class_static_vtable_24
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
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_62
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_64
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_70
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
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
    .quad _class_static_vtable_94
    .quad _class_static_vtable_95
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_97
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
    .quad _class_callable_methods_21
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_23
    .quad _class_callable_methods_24
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
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_62
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_64
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_70
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
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
    .quad _class_callable_methods_94
    .quad _class_callable_methods_95
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_97
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
.globl _class_by_name_str_10
_class_by_name_str_10:
    .ascii "Error"
.globl _class_by_name_str_21
_class_by_name_str_21:
    .ascii "TypeError"
.globl _class_by_name_str_23
_class_by_name_str_23:
    .ascii "LogicException"
.globl _class_by_name_str_24
_class_by_name_str_24:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_29
_class_by_name_str_29:
    .ascii "RuntimeException"
.globl _class_by_name_str_43
_class_by_name_str_43:
    .ascii "stdClass"
.globl _class_by_name_str_62
_class_by_name_str_62:
    .ascii "UnhandledMatchError"
.globl _class_by_name_str_64
_class_by_name_str_64:
    .ascii "ArithmeticError"
.globl _class_by_name_str_70
_class_by_name_str_70:
    .ascii "ReflectionException"
.globl _class_by_name_str_81
_class_by_name_str_81:
    .ascii "JsonException"
.globl _class_by_name_str_94
_class_by_name_str_94:
    .ascii "ValueError"
.globl _class_by_name_str_95
_class_by_name_str_95:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_97
_class_by_name_str_97:
    .ascii "OutOfBoundsException"
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
    .quad _class_by_name_str_10
    .quad 5
    .quad 10
    .quad 72
    .quad _class_by_name_str_21
    .quad 9
    .quad 21
    .quad 72
    .quad _class_by_name_str_23
    .quad 14
    .quad 23
    .quad 72
    .quad _class_by_name_str_24
    .quad 24
    .quad 24
    .quad 72
    .quad _class_by_name_str_29
    .quad 16
    .quad 29
    .quad 72
    .quad _class_by_name_str_43
    .quad 8
    .quad 43
    .quad 16
    .quad _class_by_name_str_62
    .quad 19
    .quad 62
    .quad 72
    .quad _class_by_name_str_64
    .quad 15
    .quad 64
    .quad 72
    .quad _class_by_name_str_70
    .quad 19
    .quad 70
    .quad 72
    .quad _class_by_name_str_81
    .quad 13
    .quad 81
    .quad 72
    .quad _class_by_name_str_94
    .quad 10
    .quad 94
    .quad 72
    .quad _class_by_name_str_95
    .quad 19
    .quad 95
    .quad 72
    .quad _class_by_name_str_97
    .quad 20
    .quad 97
    .quad 72
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 98
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
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_8
_interface_methods_8:
    .quad 1
    .quad 0
.globl _interface_methods_11
_interface_methods_11:
    .quad 8
    .quad 0
    .quad 1
    .quad 2
    .quad 3
    .quad 4
    .quad 5
    .quad 6
    .quad 7
.globl _class_interfaces_1
_class_interfaces_1:
    .quad 2
    .quad 11
    .quad _class_interface_impl_1_11
    .quad 8
    .quad _class_interface_impl_1_8
.globl _class_interface_impl_1_11
_class_interface_impl_1_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_8
_class_interface_impl_1_8:
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
.globl _class_interfaces_10
_class_interfaces_10:
    .quad 2
    .quad 11
    .quad _class_interface_impl_10_11
    .quad 8
    .quad _class_interface_impl_10_8
.globl _class_interface_impl_10_11
_class_interface_impl_10_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_10_8
_class_interface_impl_10_8:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_10_0
_class_serpname_10_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_10_1
_class_serpname_10_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_10_2
_class_serpname_10_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_10_3
_class_serpname_10_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_10
_class_serprop_10:
    .quad 4
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
    .quad _class_serpname_10_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_10
_class_vtable_10:
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
.globl _class_interfaces_21
_class_interfaces_21:
    .quad 2
    .quad 11
    .quad _class_interface_impl_21_11
    .quad 8
    .quad _class_interface_impl_21_8
.globl _class_interface_impl_21_11
_class_interface_impl_21_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_21_8
_class_interface_impl_21_8:
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
.globl _class_interfaces_23
_class_interfaces_23:
    .quad 2
    .quad 11
    .quad _class_interface_impl_23_11
    .quad 8
    .quad _class_interface_impl_23_8
.globl _class_interface_impl_23_11
_class_interface_impl_23_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_23_8
_class_interface_impl_23_8:
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
.globl _class_interfaces_24
_class_interfaces_24:
    .quad 2
    .quad 11
    .quad _class_interface_impl_24_11
    .quad 8
    .quad _class_interface_impl_24_8
.globl _class_interface_impl_24_11
_class_interface_impl_24_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_24_8
_class_interface_impl_24_8:
    .quad 0
.globl _class_json_pname_24_0
_class_json_pname_24_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_24
_class_json_desc_24:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_24_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_24
_class_gc_desc_24:
    .byte 1, 0, 7, 4
.globl _class_serpname_24_0
_class_serpname_24_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_24_1
_class_serpname_24_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_24_2
_class_serpname_24_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_24_3
_class_serpname_24_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_24
_class_serprop_24:
    .quad 4
    .quad _class_serpname_24_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_24_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_24_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_24_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_24
_class_vtable_24:
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
.globl _class_static_vtable_24
_class_static_vtable_24:
    .quad 0
.globl _class_callable_method_name_24__u__u_construct
_class_callable_method_name_24__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_24__u__u_tostring
_class_callable_method_name_24__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_24_getcode
_class_callable_method_name_24_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_24_getfile
_class_callable_method_name_24_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_24_getline
_class_callable_method_name_24_getline:
    .ascii "getline"
.globl _class_callable_method_name_24_getmessage
_class_callable_method_name_24_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_24_getprevious
_class_callable_method_name_24_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_24_gettrace
_class_callable_method_name_24_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_24_gettraceasstring
_class_callable_method_name_24_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_24
_class_callable_methods_24:
    .quad 9
    .quad _class_callable_method_name_24__u__u_construct
    .quad 11
    .quad _class_callable_method_name_24__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_24_getcode
    .quad 7
    .quad _class_callable_method_name_24_getfile
    .quad 7
    .quad _class_callable_method_name_24_getline
    .quad 7
    .quad _class_callable_method_name_24_getmessage
    .quad 10
    .quad _class_callable_method_name_24_getprevious
    .quad 11
    .quad _class_callable_method_name_24_gettrace
    .quad 8
    .quad _class_callable_method_name_24_gettraceasstring
    .quad 16
.globl _class_interfaces_29
_class_interfaces_29:
    .quad 2
    .quad 11
    .quad _class_interface_impl_29_11
    .quad 8
    .quad _class_interface_impl_29_8
.globl _class_interface_impl_29_11
_class_interface_impl_29_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_29_8
_class_interface_impl_29_8:
    .quad 0
.globl _class_json_pname_29_0
_class_json_pname_29_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_29
_class_json_desc_29:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_29_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_29
_class_gc_desc_29:
    .byte 1, 0, 7, 4
.globl _class_serpname_29_0
_class_serpname_29_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_29_1
_class_serpname_29_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_29_2
_class_serpname_29_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_29_3
_class_serpname_29_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_29
_class_serprop_29:
    .quad 4
    .quad _class_serpname_29_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_29_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_29_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_29_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_29
_class_vtable_29:
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
.globl _class_static_vtable_29
_class_static_vtable_29:
    .quad 0
.globl _class_callable_method_name_29__u__u_construct
_class_callable_method_name_29__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_29__u__u_tostring
_class_callable_method_name_29__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_29_getcode
_class_callable_method_name_29_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_29_getfile
_class_callable_method_name_29_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_29_getline
_class_callable_method_name_29_getline:
    .ascii "getline"
.globl _class_callable_method_name_29_getmessage
_class_callable_method_name_29_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_29_getprevious
_class_callable_method_name_29_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_29_gettrace
_class_callable_method_name_29_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_29_gettraceasstring
_class_callable_method_name_29_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_29
_class_callable_methods_29:
    .quad 9
    .quad _class_callable_method_name_29__u__u_construct
    .quad 11
    .quad _class_callable_method_name_29__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_29_getcode
    .quad 7
    .quad _class_callable_method_name_29_getfile
    .quad 7
    .quad _class_callable_method_name_29_getline
    .quad 7
    .quad _class_callable_method_name_29_getmessage
    .quad 10
    .quad _class_callable_method_name_29_getprevious
    .quad 11
    .quad _class_callable_method_name_29_gettrace
    .quad 8
    .quad _class_callable_method_name_29_gettraceasstring
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
.globl _class_interfaces_62
_class_interfaces_62:
    .quad 2
    .quad 11
    .quad _class_interface_impl_62_11
    .quad 8
    .quad _class_interface_impl_62_8
.globl _class_interface_impl_62_11
_class_interface_impl_62_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_62_8
_class_interface_impl_62_8:
    .quad 0
.globl _class_json_pname_62_0
_class_json_pname_62_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_62
_class_json_desc_62:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_62_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_62
_class_gc_desc_62:
    .byte 1, 0, 7, 4
.globl _class_serpname_62_0
_class_serpname_62_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_62_1
_class_serpname_62_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_62_2
_class_serpname_62_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_62_3
_class_serpname_62_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_62
_class_serprop_62:
    .quad 4
    .quad _class_serpname_62_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_62_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_62_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_62_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_62
_class_vtable_62:
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
.globl _class_static_vtable_62
_class_static_vtable_62:
    .quad 0
.globl _class_callable_method_name_62__u__u_construct
_class_callable_method_name_62__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_62__u__u_tostring
_class_callable_method_name_62__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_62_getcode
_class_callable_method_name_62_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_62_getfile
_class_callable_method_name_62_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_62_getline
_class_callable_method_name_62_getline:
    .ascii "getline"
.globl _class_callable_method_name_62_getmessage
_class_callable_method_name_62_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_62_getprevious
_class_callable_method_name_62_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_62_gettrace
_class_callable_method_name_62_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_62_gettraceasstring
_class_callable_method_name_62_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_62
_class_callable_methods_62:
    .quad 9
    .quad _class_callable_method_name_62__u__u_construct
    .quad 11
    .quad _class_callable_method_name_62__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_62_getcode
    .quad 7
    .quad _class_callable_method_name_62_getfile
    .quad 7
    .quad _class_callable_method_name_62_getline
    .quad 7
    .quad _class_callable_method_name_62_getmessage
    .quad 10
    .quad _class_callable_method_name_62_getprevious
    .quad 11
    .quad _class_callable_method_name_62_gettrace
    .quad 8
    .quad _class_callable_method_name_62_gettraceasstring
    .quad 16
.globl _class_interfaces_64
_class_interfaces_64:
    .quad 2
    .quad 11
    .quad _class_interface_impl_64_11
    .quad 8
    .quad _class_interface_impl_64_8
.globl _class_interface_impl_64_11
_class_interface_impl_64_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_64_8
_class_interface_impl_64_8:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_64
_class_vtable_64:
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
.globl _class_interfaces_70
_class_interfaces_70:
    .quad 2
    .quad 11
    .quad _class_interface_impl_70_11
    .quad 8
    .quad _class_interface_impl_70_8
.globl _class_interface_impl_70_11
_class_interface_impl_70_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_70_8
_class_interface_impl_70_8:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_70
_class_vtable_70:
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
.globl _class_interfaces_81
_class_interfaces_81:
    .quad 2
    .quad 11
    .quad _class_interface_impl_81_11
    .quad 8
    .quad _class_interface_impl_81_8
.globl _class_interface_impl_81_11
_class_interface_impl_81_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_81_8
_class_interface_impl_81_8:
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
.globl _class_interfaces_94
_class_interfaces_94:
    .quad 2
    .quad 11
    .quad _class_interface_impl_94_11
    .quad 8
    .quad _class_interface_impl_94_8
.globl _class_interface_impl_94_11
_class_interface_impl_94_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_94_8
_class_interface_impl_94_8:
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
    .byte 1, 0, 7, 4
.globl _class_serpname_94_0
_class_serpname_94_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_94_1
_class_serpname_94_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_94_2
_class_serpname_94_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_94_3
_class_serpname_94_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_94
_class_serprop_94:
    .quad 4
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
    .quad _class_serpname_94_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_94
_class_vtable_94:
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
    .quad 11
    .quad _class_interface_impl_95_11
    .quad 8
    .quad _class_interface_impl_95_8
.globl _class_interface_impl_95_11
_class_interface_impl_95_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_95_8
_class_interface_impl_95_8:
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
.globl _class_interfaces_97
_class_interfaces_97:
    .quad 2
    .quad 11
    .quad _class_interface_impl_97_11
    .quad 8
    .quad _class_interface_impl_97_8
.globl _class_interface_impl_97_11
_class_interface_impl_97_11:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_97_8
_class_interface_impl_97_8:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_97
_class_vtable_97:
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
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 43

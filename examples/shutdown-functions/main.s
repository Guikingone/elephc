    ; @fn name=__elephc_shutdown_wrap symbol=_fn__u__u_elephc_u_shutdown_u_wrap
.align 2

.globl _fn__u__u_elephc_u_shutdown_u_wrap
_fn__u__u_elephc_u_shutdown_u_wrap:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $cb from x0
    stur x0, [x29, #-40]
    ; param $args from x1
    stur x1, [x29, #-48]
    ; @block name=entry
_eir___elephc_shutdown_wrap_entry_0:
    ; @src line=8 col=5 end=8:11 op=concat_reset
    ldur x10, [x29, #-56]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=12 end=8:20 op=load_local
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ; @src line=8 col=12 end=8:20 op=closure_capture
    ; @src line=8 col=12 end=8:20 op=load_local
    ldur x0, [x29, #-48]
    stur x0, [x29, #-16]
    ; @src line=8 col=12 end=8:20 op=closure_capture
    ; @src line=8 col=12 end=8:20 op=closure_new
    b _eir___elephc_shutdown_wrap_callable_invoker_done_1

    ; runtime callable invoker _eir___elephc_shutdown_wrap_callable_invoker_0
.align 2
.globl _eir___elephc_shutdown_wrap_callable_invoker_0
_eir___elephc_shutdown_wrap_callable_invoker_0:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    ldur x9, [x29, #-8]
    ldr x0, [x9, #64]
    str x0, [sp, #-16]!
    ldur x9, [x29, #-8]
    ldr x0, [x9, #80]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_done_2
_eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_assoc_1:
    ldr x20, [sp]
    ldur x9, [x29, #-8]
    ldr x0, [x9, #64]
    str x0, [sp, #-16]!
    ldur x9, [x29, #-8]
    ldr x0, [x9, #80]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_done_2
_eir___elephc_shutdown_wrap_callable_invoker_0_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_shutdown_wrap_callable_invoker_done_1:
    mov x0, #96
    bl __rt_heap_alloc
    mov x19, x0
    adrp x9, _data_8@PAGE
    add x9, x9, _data_8@PAGEOFF
    ldr x10, [x9]
    str x10, [x19]
    ldr x10, [x9, #8]
    str x10, [x19, #8]
    ldr x10, [x9, #16]
    str x10, [x19, #16]
    ldr x10, [x9, #24]
    str x10, [x19, #24]
    ldr x10, [x9, #32]
    str x10, [x19, #32]
    ldr x10, [x9, #40]
    str x10, [x19, #40]
    ldr x10, [x9, #48]
    str x10, [x19, #48]
    ldr x10, [x9, #56]
    str x10, [x19, #56]
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [x19, #64]
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [x19, #80]
    mov x0, x19
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_fn__u__u_elephc_u_shutdown_u_wrap_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=__elephc_shutdown_wrap
    ; @fn name=register_shutdown_function symbol=_fn_register_u_shutdown_u_function
.align 2

.globl _fn_register_u_shutdown_u_function
_fn_register_u_shutdown_u_function:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $callback from x0
    stur x0, [x29, #-40]
    ; param $args from x1
    stur x1, [x29, #-48]
    ; @block name=entry
_eir_register_shutdown_function_entry_0:
    ; @src line=14 col=5 end=14:29 op=concat_reset
    ldur x10, [x29, #-56]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=5 end=14:29 op=load_static_property
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x10, [x9, #8]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir_register_shutdown_function_static_prop_initialized_0
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir_register_shutdown_function_typed_static_property_throw_1
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #115
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_register_shutdown_function_typed_static_property_throw_1:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_10@PAGE
    add x9, x9, _str_10@PAGEOFF
    str x9, [x0, #8]
    mov x9, #101
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir_register_shutdown_function_static_prop_initialized_0:
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-8]
    ; @src line=14 col=69 end=14:78 op=load_local
    ldur x0, [x29, #-40]
    stur x0, [x29, #-16]
    ; @src line=14 col=80 end=14:85 op=load_local
    ldur x0, [x29, #-48]
    stur x0, [x29, #-24]
    ; @src line=14 col=46 end=14:86 op=call
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    ldur x0, [x29, #-24]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    bl _fn__u__u_elephc_u_shutdown_u_wrap
    stur x0, [x29, #-32]
    ; @src line=14 col=46 end=14:86 op=nop
    ; @src line=14 col=5 end=14:29 op=array_push
    ldur x0, [x29, #-32]
    str x0, [sp, #-16]!
    mov x1, x0
    mov x2, xzr
    mov x0, #10
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_callable_descriptor_release
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    ldur x9, [x29, #-8]
    mov x1, x0
    mov x0, x9
    bl __rt_array_push_refcounted
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    stur x0, [x29, #-8]
    ; @src line=14 col=5 end=14:29 op=store_static_property
    ldur x0, [x29, #-8]
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    str xzr, [x9, #8]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_fn_register_u_shutdown_u_function_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=register_shutdown_function
    ; @fn name=__elephc_run_shutdown_functions symbol=_fn__u__u_elephc_u_run_u_shutdown_u_functions
.align 2

.globl _fn__u__u_elephc_u_run_u_shutdown_u_functions
_fn__u__u_elephc_u_run_u_shutdown_u_functions:
    ; prologue
    sub sp, sp, #224
    stp x29, x30, [sp, #208]
    add x29, sp, #208
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-200]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-184]
    stur x22, [x29, #-192]
    stur xzr, [x29, #-168]
    stur xzr, [x29, #-176]
    ; @block name=entry
_eir___elephc_run_shutdown_functions_entry_0:
    ; @src line=18 col=9 end=18:33 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=18 col=9 end=18:33 op=load_static_property
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    ldr x10, [x9, #8]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir___elephc_run_shutdown_functions_static_prop_initialized_0
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir___elephc_run_shutdown_functions_typed_static_property_throw_1
    adrp x1, _str_11@PAGE
    add x1, x1, _str_11@PAGEOFF
    mov x2, #113
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_typed_static_property_throw_1:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_12@PAGE
    add x9, x9, _str_12@PAGEOFF
    str x9, [x0, #8]
    mov x9, #99
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir___elephc_run_shutdown_functions_static_prop_initialized_0:
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    ldr x0, [x9]
    mov x21, x0
    mov x0, x21
    cbnz x0, _eir___elephc_run_shutdown_functions_if_then_2
    b _eir___elephc_run_shutdown_functions_if_merge_1
    ; @block name=if.merge
_eir___elephc_run_shutdown_functions_if_merge_1:
    ; @src line=21 col=5 end=21:29 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=21 col=42 end=21:46 op=const_bool
    mov x0, #1
    mov x21, x0
    ; @src line=21 col=5 end=21:29 op=store_static_property
    mov x0, x21
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    str xzr, [x9, #8]
    ; @src line=22 col=5 end=22:7 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=10 end=22:11 op=const_i64
    mov x0, #0
    mov x21, x0
    ; @src line=22 col=5 end=22:7 op=store_local
    mov x0, x21
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    stur x0, [x29, #-168]
    ; @src line=23 col=5 end=23:10 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_while_cond_4
    ; @block name=if.then
_eir___elephc_run_shutdown_functions_if_then_2:
    ; @src line=19 col=9 end=19:15 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; epilogue cleanup $i
    ldur x0, [x29, #-168]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_2
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_2:
    ; epilogue cleanup $c
    ldur x0, [x29, #-176]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_3
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_3:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-184]
    ldur x22, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
    ; @block name=if.else
_eir___elephc_run_shutdown_functions_if_else_3:
    ; @src line=18 col=9 end=18:33 op=nop
    udf #0
    ; @block name=while.cond
_eir___elephc_run_shutdown_functions_while_cond_4:
    ; @src line=23 col=12 end=23:14 op=load_local
    ldur x0, [x29, #-168]
    bl __rt_mixed_cast_int
    mov x21, x0
    ; @src line=23 col=23 end=23:47 op=load_static_property
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x10, [x9, #8]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir___elephc_run_shutdown_functions_static_prop_initialized_4
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir___elephc_run_shutdown_functions_typed_static_property_throw_5
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #115
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_typed_static_property_throw_5:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_10@PAGE
    add x9, x9, _str_10@PAGEOFF
    str x9, [x0, #8]
    mov x9, #101
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir___elephc_run_shutdown_functions_static_prop_initialized_4:
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-40]
    ; @src line=23 col=17 end=23:60 op=runtime_call
    ldur x0, [x29, #-40]
    cbz x0, _eir___elephc_run_shutdown_functions_count_null_container_6
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x0, x10
    b.eq _eir___elephc_run_shutdown_functions_count_null_container_6
    ldr x0, [x0]
    b _eir___elephc_run_shutdown_functions_count_done_7
_eir___elephc_run_shutdown_functions_count_null_container_6:
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir___elephc_run_shutdown_functions_static_exception_throw_8
    adrp x1, _str_13@PAGE
    add x1, x1, _str_13@PAGEOFF
    mov x2, #107
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_static_exception_throw_8:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_type_error_class_id@PAGE
    add x9, x9, _spl_type_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_14@PAGE
    add x9, x9, _str_14@PAGEOFF
    str x9, [x0, #8]
    mov x9, #73
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir___elephc_run_shutdown_functions_count_done_7:
    mov x22, x0
    ; @src line=23 col=15 end=23:60 op=icmp
    mov x0, x21
    mov x10, x22
    cmp x0, x10
    cset x0, lt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir___elephc_run_shutdown_functions_while_body_5
    b _eir___elephc_run_shutdown_functions_while_exit_6
    ; @block name=while.body
_eir___elephc_run_shutdown_functions_while_body_5:
    ; @src line=24 col=9 end=24:11 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=24 col=14 end=24:38 op=load_static_property
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x10, [x9, #8]
    movz x11, #0xfffd
    movk x11, #0xffff, lsl #16
    movk x11, #0xffff, lsl #32
    movk x11, #0x7fff, lsl #48
    cmp x10, x11
    b.ne _eir___elephc_run_shutdown_functions_static_prop_initialized_9
    adrp x9, _exc_handler_top@PAGE
    add x9, x9, _exc_handler_top@PAGEOFF
    ldr x9, [x9]
    cbnz x9, _eir___elephc_run_shutdown_functions_typed_static_property_throw_10
    adrp x1, _str_9@PAGE
    add x1, x1, _str_9@PAGEOFF
    mov x2, #115
    mov x0, #2
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_typed_static_property_throw_10:
    mov x0, #56
    bl __rt_heap_alloc
    mov x9, #6
    str x9, [x0, #-8]
    adrp x9, _spl_error_class_id@PAGE
    add x9, x9, _spl_error_class_id@PAGEOFF
    ldr x9, [x9]
    str x9, [x0]
    adrp x9, _str_10@PAGE
    add x9, x9, _str_10@PAGEOFF
    str x9, [x0, #8]
    mov x9, #101
    str x9, [x0, #16]
    str xzr, [x0, #24]
    str xzr, [x0, #40]
    adrp x9, _exc_value@PAGE
    add x9, x9, _exc_value@PAGEOFF
    str x0, [x9]
    b __rt_throw_current
_eir___elephc_run_shutdown_functions_static_prop_initialized_9:
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    ldr x0, [x9]
    stur x0, [x29, #-64]
    ; @src line=24 col=51 end=24:53 op=load_local
    ldur x0, [x29, #-168]
    bl __rt_mixed_cast_int
    mov x22, x0
    ; @src line=24 col=50 end=24:51 op=array_get
    mov x0, x22
    ldur x9, [x29, #-64]
    cbz x9, _eir___elephc_run_shutdown_functions_array_get_null_recv_12
    movz x10, #0xfffe
    movk x10, #0xffff, lsl #16
    movk x10, #0xffff, lsl #32
    movk x10, #0x7fff, lsl #48
    cmp x9, x10
    b.eq _eir___elephc_run_shutdown_functions_array_get_null_recv_12
    cmp x0, #0
    b.lt _eir___elephc_run_shutdown_functions_array_get_null_11
    ldr x10, [x9]
    cmp x0, x10
    b.ge _eir___elephc_run_shutdown_functions_array_get_null_11
    add x9, x9, #24
    ldr x0, [x9, x0, lsl #3]
    cbz x0, _eir___elephc_run_shutdown_functions_array_get_mixed_done_16
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_array_get_mixed_ref_cell_15
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    b _eir___elephc_run_shutdown_functions_array_get_mixed_done_16
_eir___elephc_run_shutdown_functions_array_get_mixed_ref_cell_15:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #7
    b.eq _eir___elephc_run_shutdown_functions_array_get_mixed_ref_cell_18
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_array_get_mixed_ref_string_hi_17
    b _eir___elephc_run_shutdown_functions_array_get_mixed_ref_box_19
_eir___elephc_run_shutdown_functions_array_get_mixed_ref_string_hi_17:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_array_get_mixed_ref_box_19:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    b _eir___elephc_run_shutdown_functions_array_get_mixed_ref_done_20
_eir___elephc_run_shutdown_functions_array_get_mixed_ref_cell_18:
    mov x0, x11
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
_eir___elephc_run_shutdown_functions_array_get_mixed_ref_done_20:
_eir___elephc_run_shutdown_functions_array_get_mixed_done_16:
    b _eir___elephc_run_shutdown_functions_array_get_done_14
_eir___elephc_run_shutdown_functions_array_get_null_11:
    bl __rt_warn_undefined_array_key_int
    b _eir___elephc_run_shutdown_functions_array_get_fallback_13
_eir___elephc_run_shutdown_functions_array_get_null_recv_12:
    bl __rt_warn_array_offset_on_null
_eir___elephc_run_shutdown_functions_array_get_fallback_13:
    mov x0, #8
    mov x1, #0
    mov x2, #0
    bl __rt_mixed_from_value
_eir___elephc_run_shutdown_functions_array_get_done_14:
    stur x0, [x29, #-80]
    ; @src line=24 col=9 end=24:11 op=acquire
    ldur x0, [x29, #-80]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-88]
    ; @src line=24 col=9 end=24:11 op=load_local
    ldur x0, [x29, #-176]
    stur x0, [x29, #-96]
    ; @src line=24 col=9 end=24:11 op=release
    ldur x0, [x29, #-96]
    bl __rt_decref_mixed
    ; @src line=24 col=9 end=24:11 op=store_local
    ldur x0, [x29, #-88]
    stur x0, [x29, #-176]
    ; @src line=24 col=9 end=24:11 op=release
    ldur x0, [x29, #-80]
    bl __rt_decref_mixed
    ; @src line=25 col=9 end=25:11 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=25 col=9 end=25:13 op=load_local
    ldur x0, [x29, #-176]
    stur x0, [x29, #-104]
    ; @src line=25 col=9 end=25:13 op=array_new
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
    stur x0, [x29, #-112]
    ; @src line=25 col=9 end=25:13 op=callable_descriptor_invoke
    ldur x0, [x29, #-104]
    bl __rt_mixed_unbox
    cmp x0, #1
    b.eq _eir___elephc_run_shutdown_functions_mixed_callable_string_name_21
    cmp x0, #10
    b.eq _eir___elephc_run_shutdown_functions_mixed_callable_closure_22
    b _eir___elephc_run_shutdown_functions_mixed_callable_not_callable_23
_eir___elephc_run_shutdown_functions_mixed_callable_closure_22:
    mov x19, x1
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_25
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_25:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_mixed_callable_done_24
_eir___elephc_run_shutdown_functions_mixed_callable_string_name_21:
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_27
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_26
_eir___elephc_run_shutdown_functions_callable_builtin_26:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $num from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_26_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_abs_mixed
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_26_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_27:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_29

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_28
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_28
_eir___elephc_run_shutdown_functions_callable_invoker_28:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_value_done_5:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_17@PAGE
    add x1, x1, _str_17@PAGEOFF
    mov x2, #3
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_key_found_8
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_key_found_8:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_missing_9
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_direct_11
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_not_boxed_17
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_boxed_12
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_not_boxed_17:
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_13
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_13:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_boxed_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_raw_15
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_boxed_14:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_ordinary_raw_15:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_direct_11:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_string_hi_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_box_19
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_string_hi_18:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_box_19:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_boxed_12:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_string_hi_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_box_21
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_string_hi_20:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_box_21:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_28_hash_invoker_ref_value_done_16:
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_done_10
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_missing_9:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_28_invoker_assoc_done_10:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_28_cufa_mixed_done_2:
    add sp, sp, #16
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_29:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_31
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_30
_eir___elephc_run_shutdown_functions_callable_builtin_30:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_30_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_addslashes
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_30_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_31:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_33

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_32
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_32
_eir___elephc_run_shutdown_functions_callable_invoker_32:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_value_4
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_value_done_5:
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #6
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_key_found_8
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_key_found_8:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_missing_9
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_direct_11
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_not_boxed_17
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_boxed_12
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_not_boxed_17:
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_13
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_13:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_boxed_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_raw_15
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_boxed_14:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_ordinary_raw_15:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_direct_11:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_string_hi_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_box_19
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_string_hi_18:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_box_19:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_boxed_12:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_string_hi_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_box_21
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_string_hi_20:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_box_21:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_32_hash_invoker_ref_value_done_16:
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_done_10
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_missing_9:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_32_invoker_assoc_done_10:
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_32_cufa_mixed_done_2:
    add sp, sp, #16
    mov x0, #1
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_33:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_35
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_34
_eir___elephc_run_shutdown_functions_callable_builtin_34:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_34_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_base64_decode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_34_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_35:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_37
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_36
_eir___elephc_run_shutdown_functions_callable_builtin_36:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_36_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_base64_encode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_36_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_37:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_39
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_38
_eir___elephc_run_shutdown_functions_callable_builtin_38:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_38_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_bin2hex
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_38_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_39:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_41
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_40
_eir___elephc_run_shutdown_functions_callable_builtin_40:
    ; prologue
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-48]
    ; param $value from x0
    stur x0, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #0
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_integer_0
    cmp x0, #1
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_string_2
    cmp x0, #2
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_double_1
    cmp x0, #3
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_boolean_3
    cmp x0, #4
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_array_5
    cmp x0, #5
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_array_5
    cmp x0, #6
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_object_6
    cmp x0, #9
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_resource_7
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_null_4
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_integer_0:
    adrp x1, _str_43@PAGE
    add x1, x1, _str_43@PAGEOFF
    mov x2, #7
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_double_1:
    adrp x1, _str_44@PAGE
    add x1, x1, _str_44@PAGEOFF
    mov x2, #6
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_string_2:
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #6
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_boolean_3:
    adrp x1, _str_45@PAGE
    add x1, x1, _str_45@PAGEOFF
    mov x2, #7
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_null_4:
    adrp x1, _str_46@PAGE
    add x1, x1, _str_46@PAGEOFF
    mov x2, #4
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_array_5:
    adrp x1, _str_47@PAGE
    add x1, x1, _str_47@PAGEOFF
    mov x2, #5
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_object_6:
    adrp x1, _str_48@PAGE
    add x1, x1, _str_48@PAGEOFF
    mov x2, #6
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_resource_7:
    adrp x1, _str_49@PAGE
    add x1, x1, _str_49@PAGEOFF
    mov x2, #8
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8
_eir__eir___elephc_run_shutdown_functions_callable_builtin_40_gettype_mixed_done_8:
    stur x1, [x29, #-24]
    stur x2, [x29, #-16]
    ldur x1, [x29, #-24]
    ldur x2, [x29, #-16]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_40_epilogue:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_41:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_43

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_42
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_42
_eir___elephc_run_shutdown_functions_callable_invoker_42:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_value_done_5:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #5
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_key_found_8
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_key_found_8:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_missing_9
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_direct_11
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_not_boxed_17
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_boxed_12
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_not_boxed_17:
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_13
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_13:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_boxed_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_raw_15
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_boxed_14:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_ordinary_raw_15:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_direct_11:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_string_hi_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_box_19
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_string_hi_18:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_box_19:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_boxed_12:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_string_hi_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_box_21
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_string_hi_20:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_box_21:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_42_hash_invoker_ref_value_done_16:
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_done_10
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_missing_9:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_42_invoker_assoc_done_10:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_42_cufa_mixed_done_2:
    add sp, sp, #16
    mov x0, #1
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_43:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_45
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_44
_eir___elephc_run_shutdown_functions_callable_builtin_44:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_44_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_hex2bin
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_44_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_45:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_47
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_46
_eir___elephc_run_shutdown_functions_callable_builtin_46:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_46_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_html_entity_decode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_46_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_47:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_49
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_48
_eir___elephc_run_shutdown_functions_callable_builtin_48:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $value from x0
    stur x0, [x29, #-40]
    ; param $base from x1
    stur x1, [x29, #-48]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_48_entry_0:
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-48]
    stur x0, [x29, #-16]
    ldur x0, [x29, #-8]
    bl __rt_mixed_cast_int
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_48_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_49:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_51

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_50
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_50
_eir___elephc_run_shutdown_functions_callable_invoker_50:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_value_done_5:
    cmp x21, #2
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_load_arg_8
    mov x0, #10
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_arg_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_load_arg_8:
    ldr x0, [x20, #32]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_value_10
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_value_done_11
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_value_10:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_string_hi_12
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_box_13
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_string_hi_12:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_ref_box_13:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_value_done_11:
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_arg_done_9:
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #5
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_key_found_14
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_key_found_14:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_missing_15
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_direct_17
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_not_boxed_23
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_boxed_18
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_not_boxed_23:
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_19
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_19:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_boxed_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_raw_21
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_boxed_20:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_raw_21:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_direct_17:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_24
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_25
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_24:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_25:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_boxed_18:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_26
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_27
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_26:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_27:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_22:
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_missing_15:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_done_16:
    mov x0, x20
    adrp x1, _str_62@PAGE
    add x1, x1, _str_62@PAGEOFF
    mov x2, #4
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_key_found_28
    mov x0, x20
    mov x1, #1
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_key_found_28:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_default_29
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_direct_31
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_not_boxed_37
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_boxed_32
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_not_boxed_37:
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_33
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_33:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_boxed_34
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_raw_35
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_boxed_34:
    mov x0, x1
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_ordinary_raw_35:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_direct_31:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_38
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_39
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_38:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_39:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_boxed_32:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_40
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_41
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_string_hi_40:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_box_41:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_50_hash_invoker_ref_value_done_36:
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_done_30
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_default_29:
    mov x0, #10
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_50_invoker_assoc_done_30:
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_50_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_51:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_53
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_52
_eir___elephc_run_shutdown_functions_callable_builtin_52:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_52_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #4
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_52_mixed_kind_true_0
    cmp x0, #5
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_52_mixed_kind_true_0
    mov x0, #0
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_52_mixed_kind_done_1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_52_mixed_kind_true_0:
    mov x0, #1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_52_mixed_kind_done_1:
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_52_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_53:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_55

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_54
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_54
_eir___elephc_run_shutdown_functions_callable_invoker_54:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_value_done_5:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_50@PAGE
    add x1, x1, _str_50@PAGEOFF
    mov x2, #5
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_key_found_8
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_key_found_8:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_missing_9
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_direct_11
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_not_boxed_17
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_boxed_12
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_not_boxed_17:
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_13
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_13:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_boxed_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_raw_15
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_boxed_14:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_ordinary_raw_15:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_direct_11:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_string_hi_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_box_19
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_string_hi_18:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_box_19:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_boxed_12:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_string_hi_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_box_21
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_string_hi_20:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_box_21:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_54_hash_invoker_ref_value_done_16:
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_done_10
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_missing_9:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_54_invoker_assoc_done_10:
    ldr x0, [sp]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_54_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #3
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_55:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_57
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_56
_eir___elephc_run_shutdown_functions_callable_builtin_56:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_56_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #3
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_56_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_57:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_59
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_58
_eir___elephc_run_shutdown_functions_callable_builtin_58:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_58_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #2
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_58_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_59:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_61
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_60
_eir___elephc_run_shutdown_functions_callable_builtin_60:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_60_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #2
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_60_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_61:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_63
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_62
_eir___elephc_run_shutdown_functions_callable_builtin_62:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_62_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #0
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_62_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_63:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_65
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_64
_eir___elephc_run_shutdown_functions_callable_builtin_64:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_64_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #0
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_64_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_65:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_67
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_66
_eir___elephc_run_shutdown_functions_callable_builtin_66:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_66_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #4
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_true_0
    cmp x0, #5
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_true_0
    cmp x0, #6
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_object_1
    mov x0, #0
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_done_2
_eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_object_1:
    str x1, [sp, #-16]!
    ldr x0, [sp]
    mov x1, #4
    mov x2, #1
    bl __rt_exception_matches
    cmp x0, #0
    b.ne _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_object_true_3
    ldr x0, [sp]
    mov x1, #10
    mov x2, #1
    bl __rt_exception_matches
    cmp x0, #0
    b.ne _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_object_true_3
    add sp, sp, #16
    mov x0, #0
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_done_2
_eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_object_true_3:
    add sp, sp, #16
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_true_0
_eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_true_0:
    mov x0, #1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_66_is_iterable_mixed_done_2:
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_66_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_67:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_69
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_68
_eir___elephc_run_shutdown_functions_callable_builtin_68:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_68_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #0
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_68_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_69:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_71
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_70
_eir___elephc_run_shutdown_functions_callable_builtin_70:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_70_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #6
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_70_mixed_kind_true_0
    mov x0, #0
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_70_mixed_kind_done_1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_70_mixed_kind_true_0:
    mov x0, #1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_70_mixed_kind_done_1:
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_70_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_71:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_73
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_72
_eir___elephc_run_shutdown_functions_callable_builtin_72:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_72_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #2
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_72_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_73:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_75
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_74
_eir___elephc_run_shutdown_functions_callable_builtin_74:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_74_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #9
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_74_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_75:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_77
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_76
_eir___elephc_run_shutdown_functions_callable_builtin_76:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_76_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #0
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_true_0
    cmp x0, #1
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_true_0
    cmp x0, #2
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_true_0
    cmp x0, #3
    b.eq _eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_true_0
    mov x0, #0
    b _eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_done_1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_true_0:
    mov x0, #1
_eir__eir___elephc_run_shutdown_functions_callable_builtin_76_mixed_kind_done_1:
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_76_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_77:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_79
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_78
_eir___elephc_run_shutdown_functions_callable_builtin_78:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-32]
    ; param $value from x0
    stur x0, [x29, #-24]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_78_entry_0:
    ldur x0, [x29, #-24]
    stur x0, [x29, #-8]
    ldur x0, [x29, #-8]
    bl __rt_mixed_unbox
    cmp x0, #1
    cset x0, eq
    stur x0, [x29, #-16]
    ldur x0, [x29, #-16]
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_78_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_79:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_81
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_80
_eir___elephc_run_shutdown_functions_callable_builtin_80:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_80_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_nl2br
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_80_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_81:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_83
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_82
_eir___elephc_run_shutdown_functions_callable_builtin_82:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-128]
    ; param $pattern from x0,x1
    stur x0, [x29, #-80]
    stur x1, [x29, #-72]
    ; param $subject from x2,x3
    stur x2, [x29, #-96]
    stur x3, [x29, #-88]
    ; param &$matches from x4 (ref)
    stur x4, [x29, #-104]
    ; param $flags from x5
    stur x5, [x29, #-112]
    ; param $offset from x6
    stur x6, [x29, #-120]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_82_entry_0:
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x9, [x29, #-104]
    ldr x0, [x9]
    stur x0, [x29, #-40]
    ldur x0, [x29, #-112]
    stur x0, [x29, #-48]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    mov x5, #1
    bl __rt_preg_match_capture
    sub sp, sp, #32
    str x0, [sp]
    str x1, [sp, #8]
    ldur x9, [x29, #-104]
    ldr x0, [x9]
    bl __rt_decref_mixed
    ldr x0, [sp, #8]
    bl __rt_mixed_from_array_kind
    str x0, [sp, #16]
    ldr x0, [sp, #8]
    bl __rt_decref_any
    ldr x0, [sp, #16]
    ldur x9, [x29, #-104]
    str x0, [x9]
    ldr x0, [sp]
    add sp, sp, #32
    stur x0, [x29, #-64]
    ldur x0, [x29, #-64]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_82_epilogue:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_83:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_85

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_84
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_84
_eir___elephc_run_shutdown_functions_callable_invoker_84:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #2
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_4
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_5:
    ldr x0, [x20, #32]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_8
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_8:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_10
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_11
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_10:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_11:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_9:
    cmp x21, #3
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_load_arg_12
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
    mov x1, x0
    mov x2, xzr
    mov x0, #4
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x0, #16
    bl __rt_heap_alloc
    mov x9, x0
    ldr x10, [sp], #16
    str x10, [x9]
    mov x10, #7
    str x10, [x9, #8]
    str x9, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_arg_done_13
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_load_arg_12:
    ldr x0, [x20, #40]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_cell_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_temp_15
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_cell_14:
    ldr x0, [x0, #8]
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_temp_15:
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    mov x0, #16
    bl __rt_heap_alloc
    mov x9, x0
    ldr x10, [sp], #16
    str x10, [x9]
    mov x10, #7
    str x10, [x9, #8]
    str x9, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_done_16:
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_arg_done_13:
    cmp x21, #4
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_load_arg_17
    mov x0, #0
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_arg_done_18
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_load_arg_17:
    ldr x0, [x20, #48]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_19
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_19:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_21
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_22
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_21:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_22:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_20:
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_arg_done_18:
    cmp x21, #5
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_load_arg_23
    mov x0, #0
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_arg_done_24
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_load_arg_23:
    ldr x0, [x20, #56]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_25
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_26
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_value_25:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_27
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_28
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_string_hi_27:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_ref_box_28:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_value_done_26:
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_arg_done_24:
    ldr x0, [sp, #64]
    ldr x1, [sp, #72]
    ldr x2, [sp, #48]
    ldr x3, [sp, #56]
    ldr x4, [sp, #32]
    ldr x5, [sp, #16]
    ldr x6, [sp]
    add sp, sp, #80
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_115@PAGE
    add x1, x1, _str_115@PAGEOFF
    mov x2, #7
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_29
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_29:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_missing_30
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_32
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_38
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_33
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_38:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_34
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_34:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_35
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_36
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_35:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_37
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_36:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_37
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_32:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_39
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_40
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_39:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_40:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_37
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_33:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_41
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_42
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_41:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_42:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_37:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_31
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_missing_30:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_31:
    mov x0, x20
    adrp x1, _str_116@PAGE
    add x1, x1, _str_116@PAGEOFF
    mov x2, #7
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_43
    mov x0, x20
    mov x1, #1
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_43:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_missing_44
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_46
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_52
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_47
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_52:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_48
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_48:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_49
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_50
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_49:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_51
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_50:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_51
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_46:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_53
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_54
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_53:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_54:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_51
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_47:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_55
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_56
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_55:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_56:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_51:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_45
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_missing_44:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_45:
    mov x0, x20
    adrp x1, _str_117@PAGE
    add x1, x1, _str_117@PAGEOFF
    mov x2, #7
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_57
    mov x0, x20
    mov x1, #2
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_57:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_ref_default_58
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_direct_60
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_66
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_boxed_61
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_66:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_62
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_direct_60:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_done_65
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_boxed_61:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_done_65
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_62:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_boxed_63
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_raw_64
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_boxed_63:
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    mov x0, #16
    bl __rt_heap_alloc
    mov x9, x0
    ldr x10, [sp], #16
    str x10, [x9]
    mov x10, #7
    str x10, [x9, #8]
    str x9, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_done_65
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_ordinary_raw_64:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    mov x0, #16
    bl __rt_heap_alloc
    mov x9, x0
    ldr x10, [sp], #16
    str x10, [x9]
    mov x10, #7
    str x10, [x9, #8]
    str x9, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_done_65:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_ref_done_59
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_ref_default_58:
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
    mov x1, x0
    mov x2, xzr
    mov x0, #4
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    mov x0, #16
    bl __rt_heap_alloc
    mov x9, x0
    ldr x10, [sp], #16
    str x10, [x9]
    mov x10, #7
    str x10, [x9, #8]
    str x9, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_ref_done_59:
    mov x0, x20
    adrp x1, _str_118@PAGE
    add x1, x1, _str_118@PAGEOFF
    mov x2, #5
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_67
    mov x0, x20
    mov x1, #3
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_67:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_default_68
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_70
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_76
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_71
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_76:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_72
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_72:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_73
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_74
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_73:
    mov x0, x1
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_75
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_74:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_75
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_70:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_77
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_78
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_77:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_78:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_75
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_71:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_79
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_80
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_79:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_80:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_75:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_69
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_default_68:
    mov x0, #0
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_69:
    mov x0, x20
    adrp x1, _str_119@PAGE
    add x1, x1, _str_119@PAGEOFF
    mov x2, #6
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_81
    mov x0, x20
    mov x1, #4
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_key_found_81:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_default_82
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_84
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_90
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_85
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_not_boxed_90:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_86
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_86:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_87
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_88
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_boxed_87:
    mov x0, x1
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_89
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_ordinary_raw_88:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_89
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_direct_84:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_91
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_92
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_91:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_92:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_89
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_boxed_85:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_93
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_94
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_string_hi_93:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_box_94:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_int
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_hash_invoker_ref_value_done_89:
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_83
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_default_82:
    mov x0, #0
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_84_invoker_assoc_done_83:
    ldr x0, [sp, #64]
    ldr x1, [sp, #72]
    ldr x2, [sp, #48]
    ldr x3, [sp, #56]
    ldr x4, [sp, #32]
    ldr x5, [sp, #16]
    ldr x6, [sp]
    add sp, sp, #80
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_84_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_85:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_87
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_86
_eir___elephc_run_shutdown_functions_callable_builtin_86:
    ; prologue
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-128]
    ; param $pattern from x0,x1
    stur x0, [x29, #-80]
    stur x1, [x29, #-72]
    ; param $subject from x2,x3
    stur x2, [x29, #-96]
    stur x3, [x29, #-88]
    ; param &$matches from x4 (ref)
    stur x4, [x29, #-104]
    ; param $flags from x5
    stur x5, [x29, #-112]
    ; param $offset from x6
    stur x6, [x29, #-120]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_86_entry_0:
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x9, [x29, #-104]
    ldr x0, [x9]
    stur x0, [x29, #-40]
    ldur x0, [x29, #-112]
    stur x0, [x29, #-48]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    mov x5, #1
    bl __rt_preg_match_capture
    sub sp, sp, #32
    str x0, [sp]
    str x1, [sp, #8]
    ldur x9, [x29, #-104]
    ldr x0, [x9]
    bl __rt_decref_mixed
    ldr x0, [sp, #8]
    bl __rt_mixed_from_array_kind
    str x0, [sp, #16]
    ldr x0, [sp, #8]
    bl __rt_decref_any
    ldr x0, [sp, #16]
    ldur x9, [x29, #-104]
    str x0, [x9]
    ldr x0, [sp]
    add sp, sp, #32
    stur x0, [x29, #-64]
    ldur x0, [x29, #-64]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_86_epilogue:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_87:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_89
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_88
_eir___elephc_run_shutdown_functions_callable_builtin_88:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_88_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_urldecode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_88_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_89:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_91
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_90
_eir___elephc_run_shutdown_functions_callable_builtin_90:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_90_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_rawurlencode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_90_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_91:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_93
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_92
_eir___elephc_run_shutdown_functions_callable_builtin_92:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_92_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_stripslashes
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_92_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_93:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_95
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_94
_eir___elephc_run_shutdown_functions_callable_builtin_94:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_94_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    mov x0, x2
    stur x0, [x29, #-24]
    ldur x0, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_94_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_95:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_97

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_96
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_96
_eir___elephc_run_shutdown_functions_callable_invoker_96:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_value_4
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_value_done_5:
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #6
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_key_found_8
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_key_found_8:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_missing_9
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_direct_11
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_not_boxed_17
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_boxed_12
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_not_boxed_17:
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_13
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_13:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_boxed_14
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_raw_15
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_boxed_14:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_ordinary_raw_15:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_direct_11:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_string_hi_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_box_19
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_string_hi_18:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_box_19:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_boxed_12:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_string_hi_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_box_21
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_string_hi_20:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_box_21:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_96_hash_invoker_ref_value_done_16:
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_done_10
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_missing_9:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_96_invoker_assoc_done_10:
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_96_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #0
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_97:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_99
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_98
_eir___elephc_run_shutdown_functions_callable_builtin_98:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_98_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_strrev
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_98_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_99:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_101
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_100
_eir___elephc_run_shutdown_functions_callable_builtin_100:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_100_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_strtolower
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_100_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_101:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_103
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_102
_eir___elephc_run_shutdown_functions_callable_builtin_102:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_102_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_strtoupper
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_102_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_103:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_105
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_104
_eir___elephc_run_shutdown_functions_callable_builtin_104:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-88]
    ; param $string from x0,x1
    stur x0, [x29, #-64]
    stur x1, [x29, #-56]
    ; param $characters from x2,x3
    stur x2, [x29, #-80]
    stur x3, [x29, #-72]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_104_entry_0:
    ldur x1, [x29, #-64]
    ldur x2, [x29, #-56]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    str x1, [sp, #-16]!
    str x2, [sp, #-16]!
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    mov x3, x1
    mov x4, x2
    ldr x2, [sp], #16
    ldr x1, [sp], #16
    bl __rt_trim_mask
    bl __rt_str_persist
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_104_epilogue:
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_105:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_107

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_106
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_106
_eir___elephc_run_shutdown_functions_callable_invoker_106:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_value_4
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_value_done_5:
    cmp x21, #2
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_load_arg_8
    adrp x1, _str_154@PAGE
    add x1, x1, _str_154@PAGEOFF
    mov x2, #7
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_arg_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_load_arg_8:
    ldr x0, [x20, #32]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_value_10
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_value_done_11
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_value_10:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_string_hi_12
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_box_13
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_string_hi_12:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_ref_box_13:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_value_done_11:
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_arg_done_9:
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr x2, [sp]
    ldr x3, [sp, #8]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_27@PAGE
    add x1, x1, _str_27@PAGEOFF
    mov x2, #6
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_key_found_14
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_key_found_14:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_missing_15
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_direct_17
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_not_boxed_23
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_boxed_18
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_not_boxed_23:
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_19
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_19:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_boxed_20
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_raw_21
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_boxed_20:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_raw_21:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_direct_17:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_24
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_25
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_24:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_25:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_22
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_boxed_18:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_26
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_27
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_26:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_27:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_22:
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_done_16
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_missing_15:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_done_16:
    mov x0, x20
    adrp x1, _str_155@PAGE
    add x1, x1, _str_155@PAGEOFF
    mov x2, #10
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_key_found_28
    mov x0, x20
    mov x1, #1
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_key_found_28:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_default_29
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_direct_31
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_not_boxed_37
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_boxed_32
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_not_boxed_37:
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_33
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_33:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_boxed_34
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_raw_35
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_boxed_34:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_ordinary_raw_35:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_direct_31:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_38
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_39
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_38:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_39:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_36
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_boxed_32:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_40
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_41
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_string_hi_40:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_box_41:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_106_hash_invoker_ref_value_done_36:
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_done_30
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_default_29:
    adrp x1, _str_154@PAGE
    add x1, x1, _str_154@PAGEOFF
    mov x2, #7
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_106_invoker_assoc_done_30:
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr x2, [sp]
    ldr x3, [sp, #8]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    bl __rt_str_persist
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_106_cufa_mixed_done_2:
    add sp, sp, #16
    mov x0, #1
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_107:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_109
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_108
_eir___elephc_run_shutdown_functions_callable_builtin_108:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_108_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_urldecode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_108_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_109:
    b _eir___elephc_run_shutdown_functions_callable_builtin_done_111
.align 2

.globl _eir___elephc_run_shutdown_functions_callable_builtin_110
_eir___elephc_run_shutdown_functions_callable_builtin_110:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $string from x0,x1
    stur x0, [x29, #-48]
    stur x1, [x29, #-40]
    ; @block name=entry
_eir__eir___elephc_run_shutdown_functions_callable_builtin_110_entry_0:
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    bl __rt_urlencode
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ldur x1, [x29, #-32]
    ldur x2, [x29, #-24]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_110_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_eir___elephc_run_shutdown_functions_callable_builtin_done_111:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_113

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_112
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_112
_eir___elephc_run_shutdown_functions_callable_invoker_112:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_assoc_1:
    ldr x20, [sp]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_112_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_113:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_115

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_114
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_114
_eir___elephc_run_shutdown_functions_callable_invoker_114:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #2
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_value_done_5:
    ldr x0, [x20, #32]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_value_8
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_value_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_value_8:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_string_hi_10
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_box_11
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_string_hi_10:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_ref_box_11:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_value_done_9:
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_3@PAGE
    add x1, x1, _str_3@PAGEOFF
    mov x2, #2
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_key_found_12
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_key_found_12:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_missing_13
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_direct_15
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_not_boxed_21
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_boxed_16
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_not_boxed_21:
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_17
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_17:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_boxed_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_raw_19
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_boxed_18:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_raw_19:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_direct_15:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_22
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_23
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_22:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_23:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_boxed_16:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_24
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_25
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_24:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_25:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_20:
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_done_14
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_missing_13:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_done_14:
    mov x0, x20
    adrp x1, _str_4@PAGE
    add x1, x1, _str_4@PAGEOFF
    mov x2, #4
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_key_found_26
    mov x0, x20
    mov x1, #1
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_key_found_26:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_missing_27
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_direct_29
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_not_boxed_35
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_boxed_30
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_not_boxed_35:
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_31
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_31:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_boxed_32
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_raw_33
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_boxed_32:
    mov x0, x1
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_ordinary_raw_33:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_direct_29:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_36
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_37
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_36:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_37:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_boxed_30:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_38
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_39
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_string_hi_38:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_box_39:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_unbox
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_114_hash_invoker_ref_value_done_34:
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_done_28
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_missing_27:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_114_invoker_assoc_done_28:
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_114_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #10
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_115:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_117

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_116
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_116
_eir___elephc_run_shutdown_functions_callable_invoker_116:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #2
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_value_4
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_value_done_5:
    ldr x0, [x20, #32]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_value_8
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_value_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_value_8:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_string_hi_10
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_box_11
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_string_hi_10:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_ref_box_11:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr d0, [sp], #16
    add sp, sp, #16
    str d0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_value_done_9:
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr d0, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_179@PAGE
    add x1, x1, _str_179@PAGEOFF
    mov x2, #2
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_key_found_12
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_key_found_12:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_missing_13
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_direct_15
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_not_boxed_21
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_boxed_16
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_not_boxed_21:
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_17
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_17:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_boxed_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_raw_19
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_boxed_18:
    mov x0, x1
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_raw_19:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_direct_15:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_22
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_23
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_22:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_23:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_boxed_16:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_24
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_25
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_24:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_25:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_string
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldp x1, x2, [sp], #16
    add sp, sp, #16
    stp x1, x2, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_20:
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_done_14
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_missing_13:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_done_14:
    mov x0, x20
    adrp x1, _str_180@PAGE
    add x1, x1, _str_180@PAGEOFF
    mov x2, #5
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_key_found_26
    mov x0, x20
    mov x1, #1
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_key_found_26:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_missing_27
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_direct_29
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_not_boxed_35
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_boxed_30
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_not_boxed_35:
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_31
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_31:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_boxed_32
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_raw_33
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_boxed_32:
    mov x0, x1
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_ordinary_raw_33:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr d0, [sp], #16
    add sp, sp, #16
    str d0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_direct_29:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_36
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_37
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_36:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_37:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr d0, [sp], #16
    add sp, sp, #16
    str d0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_34
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_boxed_30:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_38
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_39
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_string_hi_38:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_box_39:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    bl __rt_mixed_cast_float
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr d0, [sp], #16
    add sp, sp, #16
    str d0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_116_hash_invoker_ref_value_done_34:
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_done_28
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_missing_27:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_116_invoker_assoc_done_28:
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr d0, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_116_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_117:
    b _eir___elephc_run_shutdown_functions_callable_invoker_done_119

    ; runtime callable invoker _eir___elephc_run_shutdown_functions_callable_invoker_118
.align 2
.globl _eir___elephc_run_shutdown_functions_callable_invoker_118
_eir___elephc_run_shutdown_functions_callable_invoker_118:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    cmp x21, #1
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_indexed_required_ok_3
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_indexed_required_ok_3:
    ldr x0, [x20, #24]
    ldr x10, [x0]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_value_4
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_value_done_5
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_value_4:
    ldr x9, [x0, #8]
    ldr x10, [x0, #16]
    ldr x11, [x9]
    mov x12, #0
    cmp x10, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_string_hi_6
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_box_7
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_string_hi_6:
    ldr x12, [x9, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_ref_box_7:
    mov x0, x10
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_value_done_5:
    cmp x21, #1
    b.gt _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_build_variadic_8
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
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_done_9
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_build_variadic_8:
    sub x22, x21, #1
    mov x0, x22
    mov x1, #8
    bl __rt_array_new
    str x0, [sp, #-16]!
    mov x9, x0
    ldr x10, [x9, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #7
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x9, #-8]
    mov x23, #0
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_loop_10:
    cmp x23, x22
    b.ge _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_loop_done_11
    mov x24, x23
    add x24, x24, #1
    mov x26, x20
    add x26, x26, #24
    lsl x25, x24, #3
    add x26, x26, x25
    ldr x0, [x26]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp]
    mov x10, x9
    add x10, x10, #24
    lsl x25, x23, #3
    add x10, x10, x25
    str x0, [x10]
    add x23, x23, #1
    str x23, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_loop_10
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_loop_done_11:
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_variadic_done_9:
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_assoc_1:
    ldr x20, [sp]
    mov x0, x20
    adrp x1, _str_187@PAGE
    add x1, x1, _str_187@PAGEOFF
    mov x2, #8
    bl __rt_hash_get
    cbnz x0, _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_key_found_12
    mov x0, x20
    mov x1, #0
    mov x2, #-1
    bl __rt_hash_get
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_key_found_12:
    cbz x0, _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_missing_13
    cmp x3, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_direct_15
    cmp x3, #7
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_not_boxed_21
    ldr x10, [x1]
    cmp x10, #11
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_boxed_16
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_not_boxed_21:
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_17
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_17:
    cmp x3, #7
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_boxed_18
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_raw_19
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_boxed_18:
    mov x0, x1
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_ordinary_raw_19:
    mov x0, x3
    mov x1, x1
    mov x2, x2
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_direct_15:
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_string_hi_22
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_box_23
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_string_hi_22:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_box_23:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_done_20
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_boxed_16:
    mov x10, x1
    ldr x1, [x10, #8]
    ldr x2, [x10, #16]
    ldr x11, [x1]
    mov x12, #0
    cmp x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_string_hi_24
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_box_25
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_string_hi_24:
    ldr x12, [x1, #8]
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_box_25:
    mov x0, x2
    mov x1, x11
    mov x2, x12
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
_eir___elephc_run_shutdown_functions_callable_invoker_118_hash_invoker_ref_value_done_20:
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_done_14
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_missing_13:
    mov x0, #2
    adrp x1, _str_16@PAGE
    add x1, x1, _str_16@PAGEOFF
    mov x2, #63
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_invoker_118_invoker_assoc_done_14:
    mov x0, #16
    mov x1, #7
    bl __rt_hash_new
    str x0, [sp, #-16]!
    sub sp, sp, #96
    str x20, [sp, #8]
    str xzr, [sp]
    str xzr, [sp, #56]
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_loop_26:
    ldr x0, [sp, #8]
    ldr x1, [sp]
    bl __rt_hash_iter_next
    cmn x0, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_done_27
    str x0, [sp]
    str x1, [sp, #16]
    str x2, [sp, #24]
    str x3, [sp, #32]
    str x4, [sp, #40]
    str x5, [sp, #48]
    cmn x2, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_numeric_key_29
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_string_key_30
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_numeric_key_29:
    ldr x8, [sp, #16]
    mov x9, #1
    cmp x8, x9
    b.lt _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_skip_28
    ldr x8, [sp, #56]
    str x8, [sp, #16]
    mov x9, #-1
    str x9, [sp, #24]
    add x8, x8, #1
    str x8, [sp, #56]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_insert_31
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_string_key_30:
    ldr x1, [sp, #16]
    ldr x2, [sp, #24]
    adrp x3, _str_187@PAGE
    add x3, x3, _str_187@PAGEOFF
    mov x4, #8
    bl __rt_hash_key_eq
    cmp x0, #0
    b.ne _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_skip_28
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_insert_31:
    ldr x5, [sp, #48]
    cmp x5, #1
    b.eq _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_string_32
    cmp x5, #4
    b.lo _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_scalar_34
    cmp x5, #7
    b.hi _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_scalar_34
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_ref_33
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_string_32:
    ldr x1, [sp, #32]
    ldr x2, [sp, #40]
    bl __rt_str_persist
    mov x3, x1
    mov x4, x2
    ldr x5, [sp, #48]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_insert_call_35
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_ref_33:
    ldr x0, [sp, #32]
    bl __rt_incref
    ldr x3, [sp, #32]
    ldr x4, [sp, #40]
    ldr x5, [sp, #48]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_insert_call_35
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_value_scalar_34:
    ldr x3, [sp, #32]
    ldr x4, [sp, #40]
    ldr x5, [sp, #48]
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_insert_call_35:
    ldr x0, [sp, #96]
    ldr x1, [sp, #16]
    ldr x2, [sp, #24]
    bl __rt_hash_set
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_loop_26
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_skip_28:
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_loop_26
_eir___elephc_run_shutdown_functions_callable_invoker_118_assoc_variadic_done_27:
    add sp, sp, #96
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_done_2
_eir___elephc_run_shutdown_functions_callable_invoker_118_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir___elephc_run_shutdown_functions_callable_invoker_done_119:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_148@PAGE
    add x3, x3, _str_148@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_123
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_193@PAGE
    add x3, x3, _str_193@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_123
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_122
_eir___elephc_run_shutdown_functions_callable_string_match_123:
    adrp x19, _data_150@PAGE
    add x19, x19, _data_150@PAGEOFF
    adrp x19, _data_150@PAGE
    add x19, x19, _data_150@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_124
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_124:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_122:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_151@PAGE
    add x3, x3, _str_151@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_126
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_194@PAGE
    add x3, x3, _str_194@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_126
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_125
_eir___elephc_run_shutdown_functions_callable_string_match_126:
    adrp x19, _data_153@PAGE
    add x19, x19, _data_153@PAGEOFF
    adrp x19, _data_153@PAGE
    add x19, x19, _data_153@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_127
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_127:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_125:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_156@PAGE
    add x3, x3, _str_156@PAGEOFF
    mov x4, #4
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_129
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_195@PAGE
    add x3, x3, _str_195@PAGEOFF
    mov x4, #5
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_129
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_128
_eir___elephc_run_shutdown_functions_callable_string_match_129:
    adrp x19, _data_162@PAGE
    add x19, x19, _data_162@PAGEOFF
    adrp x19, _data_162@PAGE
    add x19, x19, _data_162@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_130
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_130:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_128:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_163@PAGE
    add x3, x3, _str_163@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_132
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_196@PAGE
    add x3, x3, _str_196@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_132
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_131
_eir___elephc_run_shutdown_functions_callable_string_match_132:
    adrp x19, _data_165@PAGE
    add x19, x19, _data_165@PAGEOFF
    adrp x19, _data_165@PAGE
    add x19, x19, _data_165@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_133
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_133:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_131:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_166@PAGE
    add x3, x3, _str_166@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_135
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_197@PAGE
    add x3, x3, _str_197@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_135
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_134
_eir___elephc_run_shutdown_functions_callable_string_match_135:
    adrp x19, _data_168@PAGE
    add x19, x19, _data_168@PAGEOFF
    adrp x19, _data_168@PAGE
    add x19, x19, _data_168@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_136
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_136:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_134:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_18@PAGE
    add x3, x3, _str_18@PAGEOFF
    mov x4, #3
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_138
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_198@PAGE
    add x3, x3, _str_198@PAGEOFF
    mov x4, #4
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_138
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_137
_eir___elephc_run_shutdown_functions_callable_string_match_138:
    adrp x19, _data_26@PAGE
    add x19, x19, _data_26@PAGEOFF
    adrp x19, _data_26@PAGE
    add x19, x19, _data_26@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_139
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_139:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_137:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_28@PAGE
    add x3, x3, _str_28@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_141
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_199@PAGE
    add x3, x3, _str_199@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_141
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_140
_eir___elephc_run_shutdown_functions_callable_string_match_141:
    adrp x19, _data_33@PAGE
    add x19, x19, _data_33@PAGEOFF
    adrp x19, _data_33@PAGE
    add x19, x19, _data_33@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_142
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_142:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_140:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_34@PAGE
    add x3, x3, _str_34@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_144
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_200@PAGE
    add x3, x3, _str_200@PAGEOFF
    mov x4, #14
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_144
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_143
_eir___elephc_run_shutdown_functions_callable_string_match_144:
    adrp x19, _data_36@PAGE
    add x19, x19, _data_36@PAGEOFF
    adrp x19, _data_36@PAGE
    add x19, x19, _data_36@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_145
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_145:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_143:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_37@PAGE
    add x3, x3, _str_37@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_147
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_201@PAGE
    add x3, x3, _str_201@PAGEOFF
    mov x4, #14
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_147
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_146
_eir___elephc_run_shutdown_functions_callable_string_match_147:
    adrp x19, _data_39@PAGE
    add x19, x19, _data_39@PAGEOFF
    adrp x19, _data_39@PAGE
    add x19, x19, _data_39@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_148
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_148:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_146:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_40@PAGE
    add x3, x3, _str_40@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_150
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_202@PAGE
    add x3, x3, _str_202@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_150
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_149
_eir___elephc_run_shutdown_functions_callable_string_match_150:
    adrp x19, _data_42@PAGE
    add x19, x19, _data_42@PAGEOFF
    adrp x19, _data_42@PAGE
    add x19, x19, _data_42@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_151
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_151:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_149:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_51@PAGE
    add x3, x3, _str_51@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_153
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_203@PAGE
    add x3, x3, _str_203@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_153
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_152
_eir___elephc_run_shutdown_functions_callable_string_match_153:
    adrp x19, _data_55@PAGE
    add x19, x19, _data_55@PAGEOFF
    adrp x19, _data_55@PAGE
    add x19, x19, _data_55@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_154
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_154:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_152:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_56@PAGE
    add x3, x3, _str_56@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_156
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_204@PAGE
    add x3, x3, _str_204@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_156
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_155
_eir___elephc_run_shutdown_functions_callable_string_match_156:
    adrp x19, _data_58@PAGE
    add x19, x19, _data_58@PAGEOFF
    adrp x19, _data_58@PAGE
    add x19, x19, _data_58@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_157
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_157:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_155:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_59@PAGE
    add x3, x3, _str_59@PAGEOFF
    mov x4, #18
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_159
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_205@PAGE
    add x3, x3, _str_205@PAGEOFF
    mov x4, #19
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_159
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_158
_eir___elephc_run_shutdown_functions_callable_string_match_159:
    adrp x19, _data_61@PAGE
    add x19, x19, _data_61@PAGEOFF
    adrp x19, _data_61@PAGE
    add x19, x19, _data_61@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_160
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_160:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_158:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_63@PAGE
    add x3, x3, _str_63@PAGEOFF
    mov x4, #6
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_162
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_206@PAGE
    add x3, x3, _str_206@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_162
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_161
_eir___elephc_run_shutdown_functions_callable_string_match_162:
    adrp x19, _data_71@PAGE
    add x19, x19, _data_71@PAGEOFF
    adrp x19, _data_71@PAGE
    add x19, x19, _data_71@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_163
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_163:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_161:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_72@PAGE
    add x3, x3, _str_72@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_165
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_207@PAGE
    add x3, x3, _str_207@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_165
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_164
_eir___elephc_run_shutdown_functions_callable_string_match_165:
    adrp x19, _data_75@PAGE
    add x19, x19, _data_75@PAGEOFF
    adrp x19, _data_75@PAGE
    add x19, x19, _data_75@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_166
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_166:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_164:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_76@PAGE
    add x3, x3, _str_76@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_168
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_208@PAGE
    add x3, x3, _str_208@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_168
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_167
_eir___elephc_run_shutdown_functions_callable_string_match_168:
    adrp x19, _data_78@PAGE
    add x19, x19, _data_78@PAGEOFF
    adrp x19, _data_78@PAGE
    add x19, x19, _data_78@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_169
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_169:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_167:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_79@PAGE
    add x3, x3, _str_79@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_171
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_209@PAGE
    add x3, x3, _str_209@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_171
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_170
_eir___elephc_run_shutdown_functions_callable_string_match_171:
    adrp x19, _data_81@PAGE
    add x19, x19, _data_81@PAGEOFF
    adrp x19, _data_81@PAGE
    add x19, x19, _data_81@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_172
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_172:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_170:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_82@PAGE
    add x3, x3, _str_82@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_174
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_210@PAGE
    add x3, x3, _str_210@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_174
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_173
_eir___elephc_run_shutdown_functions_callable_string_match_174:
    adrp x19, _data_84@PAGE
    add x19, x19, _data_84@PAGEOFF
    adrp x19, _data_84@PAGE
    add x19, x19, _data_84@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_175
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_175:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_173:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_85@PAGE
    add x3, x3, _str_85@PAGEOFF
    mov x4, #6
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_177
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_211@PAGE
    add x3, x3, _str_211@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_177
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_176
_eir___elephc_run_shutdown_functions_callable_string_match_177:
    adrp x19, _data_87@PAGE
    add x19, x19, _data_87@PAGEOFF
    adrp x19, _data_87@PAGE
    add x19, x19, _data_87@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_178
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_178:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_176:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_88@PAGE
    add x3, x3, _str_88@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_180
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_212@PAGE
    add x3, x3, _str_212@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_180
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_179
_eir___elephc_run_shutdown_functions_callable_string_match_180:
    adrp x19, _data_90@PAGE
    add x19, x19, _data_90@PAGEOFF
    adrp x19, _data_90@PAGE
    add x19, x19, _data_90@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_181
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_181:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_179:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_91@PAGE
    add x3, x3, _str_91@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_183
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_213@PAGE
    add x3, x3, _str_213@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_183
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_182
_eir___elephc_run_shutdown_functions_callable_string_match_183:
    adrp x19, _data_93@PAGE
    add x19, x19, _data_93@PAGEOFF
    adrp x19, _data_93@PAGE
    add x19, x19, _data_93@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_184
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_184:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_182:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_94@PAGE
    add x3, x3, _str_94@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_186
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_214@PAGE
    add x3, x3, _str_214@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_186
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_185
_eir___elephc_run_shutdown_functions_callable_string_match_186:
    adrp x19, _data_96@PAGE
    add x19, x19, _data_96@PAGEOFF
    adrp x19, _data_96@PAGE
    add x19, x19, _data_96@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_187
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_187:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_185:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_97@PAGE
    add x3, x3, _str_97@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_189
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_215@PAGE
    add x3, x3, _str_215@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_189
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_188
_eir___elephc_run_shutdown_functions_callable_string_match_189:
    adrp x19, _data_99@PAGE
    add x19, x19, _data_99@PAGEOFF
    adrp x19, _data_99@PAGE
    add x19, x19, _data_99@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_190
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_190:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_188:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_100@PAGE
    add x3, x3, _str_100@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_192
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_216@PAGE
    add x3, x3, _str_216@PAGEOFF
    mov x4, #8
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_192
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_191
_eir___elephc_run_shutdown_functions_callable_string_match_192:
    adrp x19, _data_102@PAGE
    add x19, x19, _data_102@PAGEOFF
    adrp x19, _data_102@PAGE
    add x19, x19, _data_102@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_193
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_193:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_191:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_103@PAGE
    add x3, x3, _str_103@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_195
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_217@PAGE
    add x3, x3, _str_217@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_195
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_194
_eir___elephc_run_shutdown_functions_callable_string_match_195:
    adrp x19, _data_105@PAGE
    add x19, x19, _data_105@PAGEOFF
    adrp x19, _data_105@PAGE
    add x19, x19, _data_105@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_196
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_196:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_194:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_106@PAGE
    add x3, x3, _str_106@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_198
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_218@PAGE
    add x3, x3, _str_218@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_198
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_197
_eir___elephc_run_shutdown_functions_callable_string_match_198:
    adrp x19, _data_108@PAGE
    add x19, x19, _data_108@PAGEOFF
    adrp x19, _data_108@PAGE
    add x19, x19, _data_108@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_199
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_199:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_197:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_109@PAGE
    add x3, x3, _str_109@PAGEOFF
    mov x4, #9
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_201
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_219@PAGE
    add x3, x3, _str_219@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_201
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_200
_eir___elephc_run_shutdown_functions_callable_string_match_201:
    adrp x19, _data_111@PAGE
    add x19, x19, _data_111@PAGEOFF
    adrp x19, _data_111@PAGE
    add x19, x19, _data_111@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_202
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_202:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_200:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_112@PAGE
    add x3, x3, _str_112@PAGEOFF
    mov x4, #5
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_204
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_220@PAGE
    add x3, x3, _str_220@PAGEOFF
    mov x4, #6
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_204
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_203
_eir___elephc_run_shutdown_functions_callable_string_match_204:
    adrp x19, _data_114@PAGE
    add x19, x19, _data_114@PAGEOFF
    adrp x19, _data_114@PAGE
    add x19, x19, _data_114@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_205
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_205:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_203:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_120@PAGE
    add x3, x3, _str_120@PAGEOFF
    mov x4, #10
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_207
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_221@PAGE
    add x3, x3, _str_221@PAGEOFF
    mov x4, #11
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_207
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_206
_eir___elephc_run_shutdown_functions_callable_string_match_207:
    adrp x19, _data_128@PAGE
    add x19, x19, _data_128@PAGEOFF
    adrp x19, _data_128@PAGE
    add x19, x19, _data_128@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_208
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_208:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_206:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_129@PAGE
    add x3, x3, _str_129@PAGEOFF
    mov x4, #14
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_210
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_222@PAGE
    add x3, x3, _str_222@PAGEOFF
    mov x4, #15
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_210
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_209
_eir___elephc_run_shutdown_functions_callable_string_match_210:
    adrp x19, _data_131@PAGE
    add x19, x19, _data_131@PAGEOFF
    adrp x19, _data_131@PAGE
    add x19, x19, _data_131@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_211
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_211:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_209:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_132@PAGE
    add x3, x3, _str_132@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_213
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_223@PAGE
    add x3, x3, _str_223@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_213
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_212
_eir___elephc_run_shutdown_functions_callable_string_match_213:
    adrp x19, _data_134@PAGE
    add x19, x19, _data_134@PAGEOFF
    adrp x19, _data_134@PAGE
    add x19, x19, _data_134@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_214
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_214:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_212:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_135@PAGE
    add x3, x3, _str_135@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_216
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_224@PAGE
    add x3, x3, _str_224@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_216
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_215
_eir___elephc_run_shutdown_functions_callable_string_match_216:
    adrp x19, _data_137@PAGE
    add x19, x19, _data_137@PAGEOFF
    adrp x19, _data_137@PAGE
    add x19, x19, _data_137@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_217
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_217:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_215:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_138@PAGE
    add x3, x3, _str_138@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_219
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_225@PAGE
    add x3, x3, _str_225@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_219
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_218
_eir___elephc_run_shutdown_functions_callable_string_match_219:
    adrp x19, _data_140@PAGE
    add x19, x19, _data_140@PAGEOFF
    adrp x19, _data_140@PAGE
    add x19, x19, _data_140@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_220
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_220:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_218:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_141@PAGE
    add x3, x3, _str_141@PAGEOFF
    mov x4, #6
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_222
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_226@PAGE
    add x3, x3, _str_226@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_222
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_221
_eir___elephc_run_shutdown_functions_callable_string_match_222:
    adrp x19, _data_144@PAGE
    add x19, x19, _data_144@PAGEOFF
    adrp x19, _data_144@PAGE
    add x19, x19, _data_144@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_223
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_223:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_221:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_145@PAGE
    add x3, x3, _str_145@PAGEOFF
    mov x4, #6
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_225
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_227@PAGE
    add x3, x3, _str_227@PAGEOFF
    mov x4, #7
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_225
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_224
_eir___elephc_run_shutdown_functions_callable_string_match_225:
    adrp x19, _data_147@PAGE
    add x19, x19, _data_147@PAGEOFF
    adrp x19, _data_147@PAGE
    add x19, x19, _data_147@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_226
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_226:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_224:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_169@PAGE
    add x3, x3, _str_169@PAGEOFF
    mov x4, #31
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_228
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_228@PAGE
    add x3, x3, _str_228@PAGEOFF
    mov x4, #32
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_228
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_227
_eir___elephc_run_shutdown_functions_callable_string_match_228:
    adrp x19, _data_171@PAGE
    add x19, x19, _data_171@PAGEOFF
    adrp x19, _data_171@PAGE
    add x19, x19, _data_171@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_229
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_229:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_227:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_172@PAGE
    add x3, x3, _str_172@PAGEOFF
    mov x4, #22
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_231
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_229@PAGE
    add x3, x3, _str_229@PAGEOFF
    mov x4, #23
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_231
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_230
_eir___elephc_run_shutdown_functions_callable_string_match_231:
    adrp x19, _data_178@PAGE
    add x19, x19, _data_178@PAGEOFF
    adrp x19, _data_178@PAGE
    add x19, x19, _data_178@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_232
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_232:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_230:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_181@PAGE
    add x3, x3, _str_181@PAGEOFF
    mov x4, #12
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_234
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_230@PAGE
    add x3, x3, _str_230@PAGEOFF
    mov x4, #13
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_234
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_233
_eir___elephc_run_shutdown_functions_callable_string_match_234:
    adrp x19, _data_186@PAGE
    add x19, x19, _data_186@PAGEOFF
    adrp x19, _data_186@PAGE
    add x19, x19, _data_186@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_235
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_235:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_233:
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_188@PAGE
    add x3, x3, _str_188@PAGEOFF
    mov x4, #26
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_237
    ldr x1, [sp]
    ldr x2, [sp, #8]
    adrp x3, _str_231@PAGE
    add x3, x3, _str_231@PAGEOFF
    mov x4, #27
    bl __rt_strcasecmp
    cmp x0, #0
    b.eq _eir___elephc_run_shutdown_functions_callable_string_match_237
    b _eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_236
_eir___elephc_run_shutdown_functions_callable_string_match_237:
    adrp x19, _data_192@PAGE
    add x19, x19, _data_192@PAGEOFF
    adrp x19, _data_192@PAGE
    add x19, x19, _data_192@PAGEOFF
    ldr x9, [x19, #56]
    cbnz x9, _eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_238
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_cufa_descriptor_invoker_ready_238:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-112]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-120]
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120
_eir___elephc_run_shutdown_functions_runtime_string_descriptor_next_236:
    b _eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_missing_121
_eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_missing_121:
    mov x0, #2
    adrp x1, _str_232@PAGE
    add x1, x1, _str_232@PAGEOFF
    mov x2, #52
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_callable_descriptor_invoke_runtime_string_done_120:
    add sp, sp, #16
    b _eir___elephc_run_shutdown_functions_mixed_callable_done_24
_eir___elephc_run_shutdown_functions_mixed_callable_not_callable_23:
    mov x0, #2
    adrp x1, _str_233@PAGE
    add x1, x1, _str_233@PAGEOFF
    mov x2, #84
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___elephc_run_shutdown_functions_mixed_callable_done_24:
    ; @src line=25 col=9 end=25:13 op=release
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ; @src line=25 col=9 end=25:13 op=release
    ldur x0, [x29, #-120]
    bl __rt_decref_mixed
    ; @src line=26 col=9 end=26:11 op=concat_reset
    ldur x10, [x29, #-200]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=9 end=26:11 op=load_local
    ldur x0, [x29, #-168]
    bl __rt_mixed_cast_int
    mov x22, x0
    ; @src line=26 col=9 end=26:11 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=26 col=9 end=26:11 op=ichecked_add
    mov x0, x22
    mov x10, x21
    mov x1, x10
    bl __rt_int_add_checked
    stur x0, [x29, #-144]
    ; @src line=26 col=9 end=26:11 op=acquire
    ldur x0, [x29, #-144]
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    stur x0, [x29, #-152]
    ; @src line=26 col=9 end=26:11 op=load_local
    ldur x0, [x29, #-168]
    stur x0, [x29, #-160]
    ; @src line=26 col=9 end=26:11 op=release
    ldur x0, [x29, #-160]
    bl __rt_decref_mixed
    ; @src line=26 col=9 end=26:11 op=store_local
    ldur x0, [x29, #-152]
    stur x0, [x29, #-168]
    ; @src line=26 col=9 end=26:11 op=release
    ldur x0, [x29, #-144]
    bl __rt_decref_mixed
    b _eir___elephc_run_shutdown_functions_while_cond_4
    ; @block name=while.exit
_eir___elephc_run_shutdown_functions_while_exit_6:
    ; epilogue cleanup $i
    ldur x0, [x29, #-168]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_239
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_239:
    ; epilogue cleanup $c
    ldur x0, [x29, #-176]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_240
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_240:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-184]
    ldur x22, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
_fn__u__u_elephc_u_run_u_shutdown_u_functions_epilogue:
    ; epilogue cleanup $i
    ldur x0, [x29, #-168]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_241
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_241:
    ; epilogue cleanup $c
    ldur x0, [x29, #-176]
    cbz x0, _eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_242
    bl __rt_decref_mixed
_eir___elephc_run_shutdown_functions_main_refcounted_cleanup_done_242:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-184]
    ldur x22, [x29, #-192]
    ldp x29, x30, [sp, #208]
    add sp, sp, #224
    ret
    ; @endfn name=__elephc_run_shutdown_functions
    ; @fn name=processOrder symbol=_fn_processOrder
.align 2

.globl _fn_processOrder
_fn_processOrder:
    ; prologue
    sub sp, sp, #464
    stp x29, x30, [sp, #448]
    add x29, sp, #448
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    sub x9, x29, #448
    str x10, [x9]
    ; save callee-saved registers used by the register allocator
    sub x9, x29, #432
    str d8, [x9]
    sub x9, x29, #440
    str x21, [x9]
    ; param $id from x0,x1
    sub x9, x29, #416
    str x0, [x9]
    sub x9, x29, #408
    str x1, [x9]
    ; param $total from d0
    sub x9, x29, #424
    str d0, [x9]
    ; @block name=entry
_eir_processOrder_entry_0:
    ; @src line=8 col=5 end=8:9 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=8 col=10 op=const_str
    adrp x1, _str_234@PAGE
    add x1, x1, _str_234@PAGEOFF
    mov x2, #17
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=8 col=10 op=load_local
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=8 col=10 op=str_concat
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    bl __rt_concat
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=8 col=10 op=const_str
    adrp x1, _str_235@PAGE
    add x1, x1, _str_235@PAGEOFF
    mov x2, #3
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=8 col=10 op=str_concat
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldur x3, [x29, #-64]
    ldur x4, [x29, #-56]
    bl __rt_concat
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ; @src line=8 col=10 op=release
    ; @src line=8 col=10 op=load_local
    sub x9, x29, #424
    ldr d0, [x9]
    fmov d8, d0
    ; @src line=8 col=10 op=f_to_str
    fmov d0, d8
    bl __rt_ftoa
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=8 col=10 op=str_concat
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]
    ldur x3, [x29, #-104]
    ldur x4, [x29, #-96]
    bl __rt_concat
    stur x1, [x29, #-120]
    stur x2, [x29, #-112]
    ; @src line=8 col=10 op=release
    ; @src line=8 col=10 op=release
    ; @src line=8 col=10 op=const_str
    adrp x1, _str_236@PAGE
    add x1, x1, _str_236@PAGEOFF
    mov x2, #2
    stur x1, [x29, #-136]
    stur x2, [x29, #-128]
    ; @src line=8 col=10 op=str_concat
    ldur x1, [x29, #-120]
    ldur x2, [x29, #-112]
    ldur x3, [x29, #-136]
    ldur x4, [x29, #-128]
    bl __rt_concat
    stur x1, [x29, #-152]
    stur x2, [x29, #-144]
    ; @src line=8 col=10 op=release
    ; @src line=8 col=5 end=8:9 op=echo_value
    ldur x1, [x29, #-152]
    ldur x2, [x29, #-144]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=8 col=5 end=8:9 op=release
    ; @src line=10 col=5 end=10:31 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=10 col=32 end=10:40 op=load_local
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    stur x1, [x29, #-168]
    stur x2, [x29, #-160]
    ; @src line=10 col=32 end=10:40 op=closure_capture
    ; @src line=10 col=32 end=10:40 op=closure_new
    b _eir_processOrder_callable_invoker_done_1

    ; runtime callable invoker _eir_processOrder_callable_invoker_0
.align 2
.globl _eir_processOrder_callable_invoker_0
_eir_processOrder_callable_invoker_0:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir_processOrder_callable_invoker_0_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir_processOrder_callable_invoker_0_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_processOrder_callable_invoker_0_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    ldur x9, [x29, #-8]
    ldr x1, [x9, #64]
    ldr x2, [x9, #72]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_processOrder_callable_invoker_0_cufa_mixed_done_2
_eir_processOrder_callable_invoker_0_cufa_mixed_assoc_1:
    ldr x20, [sp]
    ldur x9, [x29, #-8]
    ldr x1, [x9, #64]
    ldr x2, [x9, #72]
    stp x1, x2, [sp, #-16]!
    ldr x0, [sp]
    ldr x1, [sp, #8]
    add sp, sp, #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_processOrder_callable_invoker_0_cufa_mixed_done_2
_eir_processOrder_callable_invoker_0_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir_processOrder_callable_invoker_done_1:
    mov x0, #80
    bl __rt_heap_alloc
    mov x19, x0
    adrp x9, _data_241@PAGE
    add x9, x9, _data_241@PAGEOFF
    ldr x10, [x9]
    str x10, [x19]
    ldr x10, [x9, #8]
    str x10, [x19, #8]
    ldr x10, [x9, #16]
    str x10, [x19, #16]
    ldr x10, [x9, #24]
    str x10, [x19, #24]
    ldr x10, [x9, #32]
    str x10, [x19, #32]
    ldr x10, [x9, #40]
    str x10, [x19, #40]
    ldr x10, [x9, #48]
    str x10, [x19, #48]
    ldr x10, [x9, #56]
    str x10, [x19, #56]
    ldur x1, [x29, #-168]
    ldur x2, [x29, #-160]
    bl __rt_str_persist
    str x1, [x19, #64]
    str x2, [x19, #72]
    mov x0, x19
    stur x0, [x29, #-176]
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
    stur x0, [x29, #-184]
    ; @src line=10 col=5 end=12:7 op=call
    ldur x0, [x29, #-176]
    str x0, [sp, #-16]!
    ldur x0, [x29, #-184]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    bl _fn_register_u_shutdown_u_function
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=10 col=5 end=12:7 op=release
    ldur x0, [x29, #-176]
    bl __rt_callable_descriptor_release
    ; @src line=10 col=5 end=12:7 op=release
    ldur x0, [x29, #-184]
    bl __rt_decref_any
    ; @src line=14 col=16 end=14:24 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=14 col=9 end=14:15 op=load_local
    sub x9, x29, #424
    ldr d0, [x9]
    fmov d8, d0
    ; @src line=14 col=18 end=14:24 op=const_f64
    adrp x9, _float_242@PAGE
    add x9, x9, _float_242@PAGEOFF
    ldr d0, [x9]
    fmov d16, d0
    ; @src line=14 col=16 end=14:24 op=fcmp
    fmov d1, d8
    fmov d0, d16
    fcmp d1, d0
    cset x0, gt
    mov x12, x0
    mov x0, x12
    cbnz x0, _eir_processOrder_if_then_2
    b _eir_processOrder_if_else_3
    ; @block name=if.merge
_eir_processOrder_if_merge_1:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #432
    ldr d8, [x9]
    sub x9, x29, #440
    ldr x21, [x9]
    ldp x29, x30, [sp, #448]
    add sp, sp, #464
    ret
    ; @block name=if.then
_eir_processOrder_if_then_2:
    ; @src line=15 col=9 end=15:13 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=15 col=14 op=const_str
    adrp x1, _str_243@PAGE
    add x1, x1, _str_243@PAGEOFF
    mov x2, #6
    stur x1, [x29, #-232]
    stur x2, [x29, #-224]
    ; @src line=15 col=14 op=load_local
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    stur x1, [x29, #-248]
    stur x2, [x29, #-240]
    ; @src line=15 col=14 op=str_concat
    ldur x1, [x29, #-232]
    ldur x2, [x29, #-224]
    ldur x3, [x29, #-248]
    ldur x4, [x29, #-240]
    bl __rt_concat
    sub x9, x29, #264
    str x1, [x9]
    sub x9, x29, #256
    str x2, [x9]
    ; @src line=15 col=14 op=const_str
    adrp x1, _str_244@PAGE
    add x1, x1, _str_244@PAGEOFF
    mov x2, #28
    sub x9, x29, #280
    str x1, [x9]
    sub x9, x29, #272
    str x2, [x9]
    ; @src line=15 col=14 op=str_concat
    sub x9, x29, #264
    ldr x1, [x9]
    sub x9, x29, #256
    ldr x2, [x9]
    sub x9, x29, #280
    ldr x3, [x9]
    sub x9, x29, #272
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #296
    str x1, [x9]
    sub x9, x29, #288
    str x2, [x9]
    ; @src line=15 col=14 op=release
    ; @src line=15 col=9 end=15:13 op=echo_value
    sub x9, x29, #296
    ldr x1, [x9]
    sub x9, x29, #288
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=15 col=9 end=15:13 op=release
    ; @src line=16 col=9 end=16:13 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=16 col=14 end=16:15 op=const_i64
    mov x0, #1
    mov x21, x0
    ; @src line=16 col=9 end=16:16 op=builtin_call
    ; run registered shutdown functions (registration order)
    bl _fn__u__u_elephc_u_run_u_shutdown_u_functions
    mov x0, x21
    mov x19, x0
    bl __rt_ob_flush_all
    mov x0, x19
    mov x16, #1
    svc #0x80
    b _eir_processOrder_if_merge_1
    ; @block name=if.else
_eir_processOrder_if_else_3:
    ; @src line=19 col=5 end=19:9 op=concat_reset
    sub x9, x29, #448
    ldr x10, [x9]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=19 col=10 op=const_str
    adrp x1, _str_243@PAGE
    add x1, x1, _str_243@PAGEOFF
    mov x2, #6
    sub x9, x29, #328
    str x1, [x9]
    sub x9, x29, #320
    str x2, [x9]
    ; @src line=19 col=10 op=load_local
    sub x9, x29, #416
    ldr x1, [x9]
    sub x9, x29, #408
    ldr x2, [x9]
    sub x9, x29, #344
    str x1, [x9]
    sub x9, x29, #336
    str x2, [x9]
    ; @src line=19 col=10 op=str_concat
    sub x9, x29, #328
    ldr x1, [x9]
    sub x9, x29, #320
    ldr x2, [x9]
    sub x9, x29, #344
    ldr x3, [x9]
    sub x9, x29, #336
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #360
    str x1, [x9]
    sub x9, x29, #352
    str x2, [x9]
    ; @src line=19 col=10 op=const_str
    adrp x1, _str_245@PAGE
    add x1, x1, _str_245@PAGEOFF
    mov x2, #12
    sub x9, x29, #376
    str x1, [x9]
    sub x9, x29, #368
    str x2, [x9]
    ; @src line=19 col=10 op=str_concat
    sub x9, x29, #360
    ldr x1, [x9]
    sub x9, x29, #352
    ldr x2, [x9]
    sub x9, x29, #376
    ldr x3, [x9]
    sub x9, x29, #368
    ldr x4, [x9]
    bl __rt_concat
    sub x9, x29, #392
    str x1, [x9]
    sub x9, x29, #384
    str x2, [x9]
    ; @src line=19 col=10 op=release
    ; @src line=19 col=5 end=19:9 op=echo_value
    sub x9, x29, #392
    ldr x1, [x9]
    sub x9, x29, #384
    ldr x2, [x9]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=19 col=5 end=19:9 op=release
    b _eir_processOrder_if_merge_1
_fn_processOrder_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #432
    ldr d8, [x9]
    sub x9, x29, #440
    ldr x21, [x9]
    ldp x29, x30, [sp, #448]
    add sp, sp, #464
    ret
    ; @endfn name=processOrder
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_11 symbol=_class_propinit_11 synthetic=1
.align 2

.globl _class_propinit_11
_class_propinit_11:
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
_eir__class_propinit_11_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_11_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
    ; @endfn name=_class_propinit_11
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_29 symbol=_class_propinit_29 synthetic=1
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
    ; @block name=entry
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
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_class_propinit_29_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_29
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
_eir__class_propinit_33_entry_0:
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
_class_propinit_33_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_33
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_42 symbol=_class_propinit_42 synthetic=1
.align 2

.globl _class_propinit_42
_class_propinit_42:
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
_eir__class_propinit_42_entry_0:
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
_class_propinit_42_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_42
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
    stur x10, [x29, #-64]
    ; param $this from x0
    stur x0, [x29, #-56]
    ; @block name=entry
_eir__class_propinit_44_entry_0:
    ldur x0, [x29, #-56]
    stur x0, [x29, #-24]
    adrp x1, _str_247@PAGE
    add x1, x1, _str_247@PAGEOFF
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
_class_propinit_44_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_44
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
_eir__class_propinit_50_entry_0:
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
_class_propinit_50_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=_class_propinit_50
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
_class_propinit_54_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-48]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
_eir__class_propinit_59_entry_0:
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
_class_propinit_59_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-64]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_63_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_63_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_63
    ; @fn name=_class_propinit_64 symbol=_class_propinit_64 synthetic=1
.align 2

.globl _class_propinit_64
_class_propinit_64:
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
_eir__class_propinit_64_entry_0:
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
_class_propinit_64_epilogue:
    ; restore callee-saved registers used by the register allocator
    sub x9, x29, #400
    ldr x21, [x9]
    ldp x29, x30, [sp, #416]
    add sp, sp, #432
    ret
    ; @endfn name=_class_propinit_64
    ; @fn name=_class_propinit_65 symbol=_class_propinit_65 synthetic=1
.align 2

.globl _class_propinit_65
_class_propinit_65:
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
_eir__class_propinit_65_entry_0:
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
    adrp x9, _float_248@PAGE
    add x9, x9, _float_248@PAGEOFF
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
_class_propinit_65_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-240]
    ldur x21, [x29, #-248]
    ldp x29, x30, [sp, #256]
    add sp, sp, #272
    ret
    ; @endfn name=_class_propinit_65
    ; @fn name=_class_propinit_68 symbol=_class_propinit_68 synthetic=1
.align 2

.globl _class_propinit_68
_class_propinit_68:
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
_eir__class_propinit_68_entry_0:
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
    adrp x1, _str_247@PAGE
    add x1, x1, _str_247@PAGEOFF
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
_class_propinit_68_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    ret
    ; @endfn name=_class_propinit_68
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_74 symbol=_class_propinit_74 synthetic=1
.align 2

.globl _class_propinit_74
_class_propinit_74:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur x21, [x29, #-128]
    ; param $this from x0
    stur x0, [x29, #-120]
    ; @block name=entry
_eir__class_propinit_74_entry_0:
    ldur x0, [x29, #-120]
    stur x0, [x29, #-24]
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
    ldur x0, [x29, #-112]
    bl __rt_decref_any
    ldur x0, [x29, #-96]
    bl __rt_decref_any
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
_class_propinit_74_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    ret
    ; @endfn name=_class_propinit_74
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_86 symbol=_class_propinit_86 synthetic=1
.align 2

.globl _class_propinit_86
_class_propinit_86:
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
_eir__class_propinit_86_entry_0:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret
_class_propinit_86_epilogue:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=_class_propinit_100 symbol=_class_propinit_100 synthetic=1
.align 2

.globl _class_propinit_100
_class_propinit_100:
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
_eir__class_propinit_100_entry_0:
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
    adrp x1, _str_247@PAGE
    add x1, x1, _str_247@PAGEOFF
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
_class_propinit_100_epilogue:
    ; restore callee-saved registers used by the register allocator
    ldur x21, [x29, #-112]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
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
    adrp x1, _str_246@PAGE
    add x1, x1, _str_246@PAGEOFF
    mov x2, #0
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ldur x9, [x29, #-24]
    str x9, [sp, #-16]!
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    bl __rt_str_persist
    ldr x9, [sp], #16
    stp x1, x2, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #8]
    bl __rt_heap_free_safe
    ldr x9, [sp], #16
    ldp x1, x2, [sp], #16
    str x1, [x9, #8]
    str x2, [x9, #16]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-56]
    mov x0, #0
    mov x21, x0
    ldur x9, [x29, #-56]
    str x9, [sp, #-16]!
    mov x0, x21
    ldr x9, [sp], #16
    str x0, [x9, #24]
    str xzr, [x9, #32]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-80]
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ldur x9, [x29, #-80]
    str x9, [sp, #-16]!
    ldr x0, [x9, #40]
    bl __rt_decref_mixed
    ldr x9, [sp], #16
    str xzr, [x9, #40]
    str xzr, [x9, #48]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-96]
    ldur x0, [x29, #-120]
    stur x0, [x29, #-104]
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    ldr x10, [x0, #-8]
    mov x12, #0x80ff
    and x10, x10, x12
    mov x11, #8
    lsl x11, x11, #8
    orr x10, x10, x11
    str x10, [x0, #-8]
    stur x0, [x29, #-112]
    ldur x9, [x29, #-104]
    str x9, [sp, #-16]!
    ldur x0, [x29, #-112]
    mov x1, #8
    bl __rt_array_to_mixed
    str x0, [sp, #-16]!
    bl __rt_incref
    ldr x0, [sp], #16
    ldr x9, [sp], #16
    str x0, [sp, #-16]!
    str x9, [sp, #-16]!
    ldr x0, [x9, #56]
    bl __rt_decref_array
    ldr x9, [sp], #16
    ldr x0, [sp], #16
    str x0, [x9, #56]
    str xzr, [x9, #64]
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
    ; @fn name=__eir_closure___elephc_shutdown_wrap_0 symbol=_fn__u__u_eir_u_closure_u__u__u_elephc_u_shutdown_u_wrap_u_0
.align 2

.globl _fn__u__u_eir_u_closure_u__u__u_elephc_u_shutdown_u_wrap_u_0
_fn__u__u_eir_u_closure_u__u__u_elephc_u_shutdown_u_wrap_u_0:
    ; prologue
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-56]
    ; param $cb from x0
    stur x0, [x29, #-40]
    ; param $args from x1
    stur x1, [x29, #-48]
    ; @block name=entry
_eir___eir_closure___elephc_shutdown_wrap_0_entry_0:
    ; @src line=9 col=9 end=9:29 op=concat_reset
    ldur x10, [x29, #-56]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=9 col=30 end=9:33 op=load_local
    ldur x0, [x29, #-40]
    stur x0, [x29, #-8]
    ; @src line=9 col=35 end=9:40 op=load_local
    ldur x0, [x29, #-48]
    stur x0, [x29, #-16]
    ; @src line=9 col=9 end=9:41 op=callable_descriptor_invoke
    ldur x19, [x29, #-8]
    ldr x9, [x19, #56]
    cbnz x9, _eir___eir_closure___elephc_shutdown_wrap_0_cufa_descriptor_invoker_ready_0
    mov x0, #2
    adrp x1, _str_15@PAGE
    add x1, x1, _str_15@PAGEOFF
    mov x2, #92
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir___eir_closure___elephc_shutdown_wrap_0_cufa_descriptor_invoker_ready_0:
    str x19, [sp, #-16]!
    ldur x1, [x29, #-16]
    mov x0, x1
    bl __rt_array_clone_shallow
    mov x1, #7
    bl __rt_array_to_mixed
    mov x1, x0
    str x1, [sp, #-16]!
    mov x10, #4
    mov x11, #0
    mov x0, x10
    mov x1, x1
    mov x2, x11
    bl __rt_mixed_from_value
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_array
    ldr x1, [sp], #16
    add sp, sp, #16
    mov x0, x1
    str x0, [sp, #-16]!
    ldr x19, [sp, #16]
    mov x0, x19
    ldr x1, [sp]
    ldr x9, [x19, #56]
    blr x9
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    bl __rt_decref_mixed
    ldr x0, [sp], #16
    add sp, sp, #16
    add sp, sp, #16
    stur x0, [x29, #-24]
    ; @src line=9 col=9 end=9:41 op=nop
    ; @src line=9 col=9 end=9:41 op=release
    ldur x0, [x29, #-24]
    bl __rt_decref_mixed
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
_fn__u__u_eir_u_closure_u__u__u_elephc_u_shutdown_u_wrap_u_0_epilogue:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    ret
    ; @endfn name=__eir_closure___elephc_shutdown_wrap_0
    ; @fn name=__eir_closure_processOrder_0 symbol=_fn__u__u_eir_u_closure_u_processOrder_u_0
.align 2

.globl _fn__u__u_eir_u_closure_u_processOrder_u_0
_fn__u__u_eir_u_closure_u_processOrder_u_0:
    ; prologue
    sub sp, sp, #128
    stp x29, x30, [sp, #112]
    add x29, sp, #112
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-104]
    ; param $id from x0,x1
    stur x0, [x29, #-96]
    stur x1, [x29, #-88]
    ; @block name=entry
_eir___eir_closure_processOrder_0_entry_0:
    ; @src line=11 col=9 end=11:13 op=concat_reset
    ldur x10, [x29, #-104]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=11 col=14 op=const_str
    adrp x1, _str_243@PAGE
    add x1, x1, _str_243@PAGEOFF
    mov x2, #6
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=11 col=14 op=load_local
    ldur x1, [x29, #-96]
    ldur x2, [x29, #-88]
    stur x1, [x29, #-32]
    stur x2, [x29, #-24]
    ; @src line=11 col=14 op=str_concat
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]
    ldur x3, [x29, #-32]
    ldur x4, [x29, #-24]
    bl __rt_concat
    stur x1, [x29, #-48]
    stur x2, [x29, #-40]
    ; @src line=11 col=14 op=const_str
    adrp x1, _str_249@PAGE
    add x1, x1, _str_249@PAGEOFF
    mov x2, #17
    stur x1, [x29, #-64]
    stur x2, [x29, #-56]
    ; @src line=11 col=14 op=str_concat
    ldur x1, [x29, #-48]
    ldur x2, [x29, #-40]
    ldur x3, [x29, #-64]
    ldur x4, [x29, #-56]
    bl __rt_concat
    stur x1, [x29, #-80]
    stur x2, [x29, #-72]
    ; @src line=11 col=14 op=release
    ; @src line=11 col=9 end=11:13 op=echo_value
    ldur x1, [x29, #-80]
    ldur x2, [x29, #-72]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ; @src line=11 col=9 end=11:13 op=release
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
_fn__u__u_eir_u_closure_u_processOrder_u_0_epilogue:
    ldp x29, x30, [sp, #112]
    add sp, sp, #128
    ret
    ; @endfn name=__eir_closure_processOrder_0
    ; @fn name=__eir_closure_main_0 symbol=_fn__u__u_eir_u_closure_u_main_u_0
.align 2

.globl _fn__u__u_eir_u_closure_u_main_u_0
_fn__u__u_eir_u_closure_u_main_u_0:
    ; prologue
    sub sp, sp, #48
    stp x29, x30, [sp, #32]
    add x29, sp, #32
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-24]
    ; @block name=entry
_eir___eir_closure_main_0_entry_0:
    ; @src line=23 col=5 end=23:9 op=concat_reset
    ldur x10, [x29, #-24]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=23 col=10 op=const_str
    adrp x1, _str_250@PAGE
    add x1, x1, _str_250@PAGEOFF
    mov x2, #43
    stur x1, [x29, #-16]
    stur x2, [x29, #-8]
    ; @src line=23 col=5 end=23:9 op=echo_value
    ldur x1, [x29, #-16]
    ldur x2, [x29, #-8]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
_fn__u__u_eir_u_closure_u_main_u_0_epilogue:
    ldp x29, x30, [sp, #32]
    add sp, sp, #48
    ret
    ; @endfn name=__eir_closure_main_0
    ; @fn name=main symbol=_main
.align 2

.globl _main
_main:
    ; prologue
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    stur x10, [x29, #-136]
    ; save callee-saved registers used by the register allocator
    stur d8, [x29, #-120]
    stur x21, [x29, #-128]
    ; save argc/argv to globals
    adrp x9, _global_argc@PAGE
    add x9, x9, _global_argc@PAGEOFF
    str x0, [x9]
    adrp x9, _global_argv@PAGE
    add x9, x9, _global_argv@PAGEOFF
    str x1, [x9]
    ; initialize static property __ElephcShutdownRegistry::$callbacks
    mov x0, #4
    mov x1, #8
    bl __rt_array_new
    str x0, [sp, #-16]!
    ldr x0, [sp], #16
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_callbacks@PAGEOFF
    str xzr, [x9, #8]
    ; initialize static property __ElephcShutdownRegistry::$running
    mov x0, #0
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    str x0, [x9]
    adrp x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGE
    add x9, x9, _static_prop__u__u_ElephcShutdownRegistry_running@PAGEOFF
    str xzr, [x9, #8]
    ; @block name=entry
_eir_main_entry_0:
    ; @src line=2 col=1 end=2:6 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=2 col=1 end=2:6 op=nop
    ; @src line=7 col=1 end=7:9 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=7 col=1 end=7:9 op=nop
    ; @src line=13 col=1 end=13:9 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=13 col=1 end=13:9 op=nop
    ; @src line=17 col=1 end=17:9 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=17 col=1 end=17:9 op=nop
    ; @src line=6 col=1 end=6:9 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=6 col=1 end=6:9 op=nop
    ; @src line=22 col=1 end=22:27 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=22 col=28 end=22:36 op=closure_new
    b _eir_main_callable_invoker_done_1

    ; runtime callable invoker _eir_main_callable_invoker_0
.align 2
.globl _eir_main_callable_invoker_0
_eir_main_callable_invoker_0:
    ; prologue
    sub sp, sp, #112
    stp x29, x30, [sp, #96]
    add x29, sp, #96
    stur x19, [x29, #-32]
    stur x20, [x29, #-40]
    stur x21, [x29, #-48]
    stur x22, [x29, #-56]
    stur x23, [x29, #-64]
    stur x24, [x29, #-72]
    stur x25, [x29, #-80]
    stur x26, [x29, #-88]
    stur x0, [x29, #-8]
    mov x19, x0
    ldr x19, [x19, #8]
    mov x20, x1
    ldr x21, [x20]
    ldr x22, [x20, #8]
    str x22, [sp, #-16]!
    cmp x21, #4
    b.eq _eir_main_callable_invoker_0_cufa_mixed_indexed_0
    cmp x21, #5
    b.eq _eir_main_callable_invoker_0_cufa_mixed_assoc_1
    mov x0, #2
    adrp x1, _str_0@PAGE
    add x1, x1, _str_0@PAGEOFF
    mov x2, #91
    mov x16, #4
    svc #0x80
    bl __rt_ob_flush_all
    mov x0, #1
    mov x16, #1
    svc #0x80
_eir_main_callable_invoker_0_cufa_mixed_indexed_0:
    ldr x20, [sp]
    ldr x21, [x20]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_callable_invoker_0_cufa_mixed_done_2
_eir_main_callable_invoker_0_cufa_mixed_assoc_1:
    ldr x20, [sp]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    ldr x10, [x9]
    str x10, [sp, #-16]!
    blr x19
    ldr x10, [sp], #16
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    b _eir_main_callable_invoker_0_cufa_mixed_done_2
_eir_main_callable_invoker_0_cufa_mixed_done_2:
    add sp, sp, #16
    mov x1, x0
    mov x2, xzr
    mov x0, #8
    bl __rt_mixed_from_value
    ldur x19, [x29, #-32]
    ldur x20, [x29, #-40]
    ldur x21, [x29, #-48]
    ldur x22, [x29, #-56]
    ldur x23, [x29, #-64]
    ldur x24, [x29, #-72]
    ldur x25, [x29, #-80]
    ldur x26, [x29, #-88]
    ldp x29, x30, [sp, #96]
    add sp, sp, #112
    ret
_eir_main_callable_invoker_done_1:
    adrp x0, _data_252@PAGE
    add x0, x0, _data_252@PAGEOFF
    stur x0, [x29, #-8]
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
    stur x0, [x29, #-16]
    ; @src line=22 col=1 end=24:3 op=call
    ldur x0, [x29, #-8]
    str x0, [sp, #-16]!
    ldur x0, [x29, #-16]
    str x0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp]
    add sp, sp, #32
    bl _fn_register_u_shutdown_u_function
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=22 col=1 end=24:3 op=release
    ldur x0, [x29, #-8]
    bl __rt_callable_descriptor_release
    ; @src line=22 col=1 end=24:3 op=release
    ldur x0, [x29, #-16]
    bl __rt_decref_any
    ; @src line=26 col=1 end=26:13 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=26 col=14 op=const_str
    adrp x1, _str_253@PAGE
    add x1, x1, _str_253@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-40]
    stur x2, [x29, #-32]
    ; @src line=26 col=23 end=26:28 op=const_f64
    adrp x9, _float_254@PAGE
    add x9, x9, _float_254@PAGEOFF
    ldr d0, [x9]
    fmov d8, d0
    ; @src line=26 col=1 end=26:29 op=call
    ldur x1, [x29, #-40]
    ldur x2, [x29, #-32]
    stp x1, x2, [sp, #-16]!
    fmov d0, d8
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr d0, [sp]
    add sp, sp, #32
    bl _fn_processOrder
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=27 col=1 end=27:13 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=27 col=14 op=const_str
    adrp x1, _str_255@PAGE
    add x1, x1, _str_255@PAGEOFF
    mov x2, #5
    stur x1, [x29, #-72]
    stur x2, [x29, #-64]
    ; @src line=27 col=23 end=27:29 op=const_f64
    adrp x9, _float_256@PAGE
    add x9, x9, _float_256@PAGEOFF
    ldr d0, [x9]
    fmov d8, d0
    ; @src line=27 col=1 end=27:30 op=call
    ldur x1, [x29, #-72]
    ldur x2, [x29, #-64]
    stp x1, x2, [sp, #-16]!
    fmov d0, d8
    str d0, [sp, #-16]!
    ldr x0, [sp, #16]
    ldr x1, [sp, #24]
    ldr d0, [sp]
    add sp, sp, #32
    bl _fn_processOrder
    movz x0, #0xfffe
    movk x0, #0xffff, lsl #16
    movk x0, #0xffff, lsl #32
    movk x0, #0x7fff, lsl #48
    mov x21, x0
    ; @src line=29 col=1 end=29:5 op=concat_reset
    ldur x10, [x29, #-136]
    adrp x9, _concat_off@PAGE
    add x9, x9, _concat_off@PAGEOFF
    str x10, [x9]
    ; @src line=29 col=6 op=const_str
    adrp x1, _str_257@PAGE
    add x1, x1, _str_257@PAGEOFF
    mov x2, #12
    stur x1, [x29, #-104]
    stur x2, [x29, #-96]
    ; @src line=29 col=1 end=29:5 op=echo_value
    ldur x1, [x29, #-104]
    ldur x2, [x29, #-96]

    ; echo
    mov x0, x1
    mov x1, x2
    bl __rt_stdout_write

    ; epilogue + exit(0)
    ; run registered shutdown functions (registration order)
    bl _fn__u__u_elephc_u_run_u_shutdown_u_functions
    bl __rt_ob_flush_all
    ; restore callee-saved registers used by the register allocator
    ldur d8, [x29, #-120]
    ldur x21, [x29, #-128]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
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
    .ascii "Fatal error: callable descriptor invoker expected an indexed or associative argument array\n"
.globl _str_1
_str_1:
    .ascii "__eir_closure___elephc_shutdown_wrap_0"
.globl _str_3
_str_3:
    .ascii "cb"
.globl _str_4
_str_4:
    .ascii "args"
.globl _str_9
_str_9:
    .ascii "Fatal error: Typed static property __ElephcShutdownRegistry::$callbacks must not be accessed before initialization\n"
.globl _str_10
_str_10:
    .ascii "Typed static property __ElephcShutdownRegistry::$callbacks must not be accessed before initialization"
.globl _str_11
_str_11:
    .ascii "Fatal error: Typed static property __ElephcShutdownRegistry::$running must not be accessed before initialization\n"
.globl _str_12
_str_12:
    .ascii "Typed static property __ElephcShutdownRegistry::$running must not be accessed before initialization"
.globl _str_13
_str_13:
    .ascii "Fatal error: Uncaught TypeError: count(): Argument #1 ($value) must be of type Countable|array, null given\n"
.globl _str_14
_str_14:
    .ascii "count(): Argument #1 ($value) must be of type Countable|array, null given"
.globl _str_15
_str_15:
    .ascii "Fatal error: Unsupported EIR callable_descriptor_invoke callable descriptor without invoker\n"
.globl _str_16
_str_16:
    .ascii "Fatal error: call_user_func_array(): missing required argument\n"
.globl _str_17
_str_17:
    .ascii "num"
.globl _str_18
_str_18:
    .ascii "abs"
.globl _str_27
_str_27:
    .ascii "string"
.globl _str_28
_str_28:
    .ascii "addslashes"
.globl _str_34
_str_34:
    .ascii "base64_decode"
.globl _str_37
_str_37:
    .ascii "base64_encode"
.globl _str_40
_str_40:
    .ascii "bin2hex"
.globl _str_43
_str_43:
    .ascii "integer"
.globl _str_44
_str_44:
    .ascii "double"
.globl _str_45
_str_45:
    .ascii "boolean"
.globl _str_46
_str_46:
    .ascii "NULL"
.globl _str_47
_str_47:
    .ascii "array"
.globl _str_48
_str_48:
    .ascii "object"
.globl _str_49
_str_49:
    .ascii "resource"
.globl _str_50
_str_50:
    .ascii "value"
.globl _str_51
_str_51:
    .ascii "gettype"
.globl _str_56
_str_56:
    .ascii "hex2bin"
.globl _str_59
_str_59:
    .ascii "html_entity_decode"
.globl _str_62
_str_62:
    .ascii "base"
.globl _str_63
_str_63:
    .ascii "intval"
.globl _str_72
_str_72:
    .ascii "is_array"
.globl _str_76
_str_76:
    .ascii "is_bool"
.globl _str_79
_str_79:
    .ascii "is_double"
.globl _str_82
_str_82:
    .ascii "is_float"
.globl _str_85
_str_85:
    .ascii "is_int"
.globl _str_88
_str_88:
    .ascii "is_integer"
.globl _str_91
_str_91:
    .ascii "is_iterable"
.globl _str_94
_str_94:
    .ascii "is_long"
.globl _str_97
_str_97:
    .ascii "is_object"
.globl _str_100
_str_100:
    .ascii "is_real"
.globl _str_103
_str_103:
    .ascii "is_resource"
.globl _str_106
_str_106:
    .ascii "is_scalar"
.globl _str_109
_str_109:
    .ascii "is_string"
.globl _str_112
_str_112:
    .ascii "nl2br"
.globl _str_115
_str_115:
    .ascii "pattern"
.globl _str_116
_str_116:
    .ascii "subject"
.globl _str_117
_str_117:
    .ascii "matches"
.globl _str_118
_str_118:
    .ascii "flags"
.globl _str_119
_str_119:
    .ascii "offset"
.globl _str_120
_str_120:
    .ascii "preg_match"
.globl _str_129
_str_129:
    .ascii "preg_match_all"
.globl _str_132
_str_132:
    .ascii "rawurldecode"
.globl _str_135
_str_135:
    .ascii "rawurlencode"
.globl _str_138
_str_138:
    .ascii "stripslashes"
.globl _str_141
_str_141:
    .ascii "strlen"
.globl _str_145
_str_145:
    .ascii "strrev"
.globl _str_148
_str_148:
    .ascii "strtolower"
.globl _str_151
_str_151:
    .ascii "strtoupper"
.globl _str_154
_str_154:
    .ascii " \n\015\t\013\014\000"
.globl _str_155
_str_155:
    .ascii "characters"
.globl _str_156
_str_156:
    .ascii "trim"
.globl _str_163
_str_163:
    .ascii "urldecode"
.globl _str_166
_str_166:
    .ascii "urlencode"
.globl _str_169
_str_169:
    .ascii "__elephc_run_shutdown_functions"
.globl _str_172
_str_172:
    .ascii "__elephc_shutdown_wrap"
.globl _str_179
_str_179:
    .ascii "id"
.globl _str_180
_str_180:
    .ascii "total"
.globl _str_181
_str_181:
    .ascii "processOrder"
.globl _str_187
_str_187:
    .ascii "callback"
.globl _str_188
_str_188:
    .ascii "register_shutdown_function"
.globl _str_193
_str_193:
    .ascii "\\strtolower"
.globl _str_194
_str_194:
    .ascii "\\strtoupper"
.globl _str_195
_str_195:
    .ascii "\\trim"
.globl _str_196
_str_196:
    .ascii "\\urldecode"
.globl _str_197
_str_197:
    .ascii "\\urlencode"
.globl _str_198
_str_198:
    .ascii "\\abs"
.globl _str_199
_str_199:
    .ascii "\\addslashes"
.globl _str_200
_str_200:
    .ascii "\\base64_decode"
.globl _str_201
_str_201:
    .ascii "\\base64_encode"
.globl _str_202
_str_202:
    .ascii "\\bin2hex"
.globl _str_203
_str_203:
    .ascii "\\gettype"
.globl _str_204
_str_204:
    .ascii "\\hex2bin"
.globl _str_205
_str_205:
    .ascii "\\html_entity_decode"
.globl _str_206
_str_206:
    .ascii "\\intval"
.globl _str_207
_str_207:
    .ascii "\\is_array"
.globl _str_208
_str_208:
    .ascii "\\is_bool"
.globl _str_209
_str_209:
    .ascii "\\is_double"
.globl _str_210
_str_210:
    .ascii "\\is_float"
.globl _str_211
_str_211:
    .ascii "\\is_int"
.globl _str_212
_str_212:
    .ascii "\\is_integer"
.globl _str_213
_str_213:
    .ascii "\\is_iterable"
.globl _str_214
_str_214:
    .ascii "\\is_long"
.globl _str_215
_str_215:
    .ascii "\\is_object"
.globl _str_216
_str_216:
    .ascii "\\is_real"
.globl _str_217
_str_217:
    .ascii "\\is_resource"
.globl _str_218
_str_218:
    .ascii "\\is_scalar"
.globl _str_219
_str_219:
    .ascii "\\is_string"
.globl _str_220
_str_220:
    .ascii "\\nl2br"
.globl _str_221
_str_221:
    .ascii "\\preg_match"
.globl _str_222
_str_222:
    .ascii "\\preg_match_all"
.globl _str_223
_str_223:
    .ascii "\\rawurldecode"
.globl _str_224
_str_224:
    .ascii "\\rawurlencode"
.globl _str_225
_str_225:
    .ascii "\\stripslashes"
.globl _str_226
_str_226:
    .ascii "\\strlen"
.globl _str_227
_str_227:
    .ascii "\\strrev"
.globl _str_228
_str_228:
    .ascii "\\__elephc_run_shutdown_functions"
.globl _str_229
_str_229:
    .ascii "\\__elephc_shutdown_wrap"
.globl _str_230
_str_230:
    .ascii "\\processOrder"
.globl _str_231
_str_231:
    .ascii "\\register_shutdown_function"
.globl _str_232
_str_232:
    .ascii "Fatal error: Call to undefined function <dynamic>()\n"
.globl _str_233
_str_233:
    .ascii "Fatal error: Unsupported EIR callable_descriptor_invoke mixed value is not callable\n"
.globl _str_234
_str_234:
    .ascii "Processing order "
.globl _str_235
_str_235:
    .ascii " ($"
.globl _str_236
_str_236:
    .ascii ")\n"
.globl _str_237
_str_237:
    .ascii "__eir_closure_processOrder_0"
.globl _str_243
_str_243:
    .ascii "Order "
.globl _str_244
_str_244:
    .ascii ": flagged for manual review\n"
.globl _str_245
_str_245:
    .ascii ": confirmed\n"
.globl _str_246
_str_246:
    .ascii ""
.globl _str_247
_str_247:
    .ascii "UTC"
.globl _str_249
_str_249:
    .ascii ": releasing lock\n"
.globl _str_250
_str_250:
    .ascii "Shutting down: closing database connection\n"
.globl _str_251
_str_251:
    .ascii "__eir_closure_main_0"
.globl _str_253
_str_253:
    .ascii "A-100"
.globl _str_255
_str_255:
    .ascii "B-200"
.globl _str_257
_str_257:
    .ascii "unreachable\n"
.p2align 3
.globl _float_242
_float_242:
    .quad 0x408f400000000000
.p2align 3
.globl _float_248
_float_248:
    .quad 0x0000000000000000
.p2align 3
.globl _float_254
_float_254:
    .quad 0x406f400000000000
.p2align 3
.globl _float_256
_float_256:
    .quad 0x4097700000000000
.p2align 3
.globl _data_2
_data_2:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0xffffffffffffffff
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_5
_data_5:
    .quad _str_3
    .quad 0x0000000000000002
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad _str_4
    .quad 0x0000000000000004
    .quad 0x0000000000000004
    .quad 0x0000000000000000
.p2align 3
.globl _data_6
_data_6:
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad _data_5
    .quad _data_5
    .quad 0x0000000000000000
.p2align 3
.globl _data_7
_data_7:
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_8
_data_8:
    .quad 0x0000000000000001
    .quad _fn__u__u_eir_u_closure_u__u__u_elephc_u_shutdown_u_wrap_u_0
    .quad _str_1
    .quad 0x0000000000000026
    .quad _data_2
    .quad _data_6
    .quad _data_7
    .quad _eir___elephc_shutdown_wrap_callable_invoker_0
.p2align 3
.globl _data_19
_data_19:
    .quad _str_17
    .quad 0x0000000000000003
.p2align 3
.globl _data_20
_data_20:
    .quad 0x0000000000000007
    .quad 0x0000000000000008
    .quad 0x0000000000000001
.p2align 3
.globl _data_21
_data_21:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_22
_data_22:
    .quad 0x0000000000000000
.p2align 3
.globl _data_23
_data_23:
    .quad 0x0000000000000001
.p2align 3
.globl _data_24
_data_24:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0xffffffffffffffff
    .quad 0x0000000000000007
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_19
    .quad _data_20
    .quad _data_21
    .quad _data_22
    .quad _data_23
.p2align 3
.globl _data_25
_data_25:
    .quad 0x0000000000000009
    .quad _str_18
    .quad 0x0000000000000003
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_26
_data_26:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_26
    .quad _str_18
    .quad 0x0000000000000003
    .quad _data_24
    .quad 0x0000000000000000
    .quad _data_25
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_28
.p2align 3
.globl _data_29
_data_29:
    .quad _str_27
    .quad 0x0000000000000006
.p2align 3
.globl _data_30
_data_30:
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
.p2align 3
.globl _data_31
_data_31:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0xffffffffffffffff
    .quad 0x0000000000000001
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad _data_29
    .quad _data_30
    .quad _data_21
    .quad _data_22
    .quad _data_23
.p2align 3
.globl _data_32
_data_32:
    .quad 0x0000000000000009
    .quad _str_28
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_33
_data_33:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_30
    .quad _str_28
    .quad 0x000000000000000a
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_32
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_35
_data_35:
    .quad 0x0000000000000009
    .quad _str_34
    .quad 0x000000000000000d
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_36
_data_36:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_34
    .quad _str_34
    .quad 0x000000000000000d
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_35
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_38
_data_38:
    .quad 0x0000000000000009
    .quad _str_37
    .quad 0x000000000000000d
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_39
_data_39:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_36
    .quad _str_37
    .quad 0x000000000000000d
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_38
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_41
_data_41:
    .quad 0x0000000000000009
    .quad _str_40
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_42
_data_42:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_38
    .quad _str_40
    .quad 0x0000000000000007
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_41
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_52
_data_52:
    .quad _str_50
    .quad 0x0000000000000005
.p2align 3
.globl _data_53
_data_53:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0xffffffffffffffff
    .quad 0x0000000000000001
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad _data_52
    .quad _data_20
    .quad _data_21
    .quad _data_22
    .quad _data_23
.p2align 3
.globl _data_54
_data_54:
    .quad 0x0000000000000009
    .quad _str_51
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_55
_data_55:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_40
    .quad _str_51
    .quad 0x0000000000000007
    .quad _data_53
    .quad 0x0000000000000000
    .quad _data_54
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_42
.p2align 3
.globl _data_57
_data_57:
    .quad 0x0000000000000009
    .quad _str_56
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_58
_data_58:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_44
    .quad _str_56
    .quad 0x0000000000000007
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_57
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_60
_data_60:
    .quad 0x0000000000000009
    .quad _str_59
    .quad 0x0000000000000012
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_61
_data_61:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_46
    .quad _str_59
    .quad 0x0000000000000012
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_60
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_64
_data_64:
    .quad _str_50
    .quad 0x0000000000000005
    .quad _str_62
    .quad 0x0000000000000004
.p2align 3
.globl _data_65
_data_65:
    .quad 0x0000000000000007
    .quad 0x0000000000000008
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000008
    .quad 0x0000000000000001
.p2align 3
.globl _data_66
_data_66:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x000000000000000a
    .quad 0x0000000000000000
.p2align 3
.globl _data_67
_data_67:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_68
_data_68:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
.p2align 3
.globl _data_69
_data_69:
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000002
    .quad 0xffffffffffffffff
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_64
    .quad _data_65
    .quad _data_66
    .quad _data_67
    .quad _data_68
.p2align 3
.globl _data_70
_data_70:
    .quad 0x0000000000000009
    .quad _str_63
    .quad 0x0000000000000006
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_71
_data_71:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_48
    .quad _str_63
    .quad 0x0000000000000006
    .quad _data_69
    .quad 0x0000000000000000
    .quad _data_70
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_50
.p2align 3
.globl _data_73
_data_73:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0xffffffffffffffff
    .quad 0x0000000000000003
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_52
    .quad _data_20
    .quad _data_21
    .quad _data_22
    .quad _data_23
.p2align 3
.globl _data_74
_data_74:
    .quad 0x0000000000000009
    .quad _str_72
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_75
_data_75:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_52
    .quad _str_72
    .quad 0x0000000000000008
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_74
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_77
_data_77:
    .quad 0x0000000000000009
    .quad _str_76
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_78
_data_78:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_56
    .quad _str_76
    .quad 0x0000000000000007
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_77
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_80
_data_80:
    .quad 0x0000000000000009
    .quad _str_79
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_81
_data_81:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_58
    .quad _str_79
    .quad 0x0000000000000009
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_80
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_83
_data_83:
    .quad 0x0000000000000009
    .quad _str_82
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_84
_data_84:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_60
    .quad _str_82
    .quad 0x0000000000000008
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_83
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_86
_data_86:
    .quad 0x0000000000000009
    .quad _str_85
    .quad 0x0000000000000006
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_87
_data_87:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_62
    .quad _str_85
    .quad 0x0000000000000006
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_86
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_89
_data_89:
    .quad 0x0000000000000009
    .quad _str_88
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_90
_data_90:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_64
    .quad _str_88
    .quad 0x000000000000000a
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_89
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_92
_data_92:
    .quad 0x0000000000000009
    .quad _str_91
    .quad 0x000000000000000b
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_93
_data_93:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_66
    .quad _str_91
    .quad 0x000000000000000b
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_92
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_95
_data_95:
    .quad 0x0000000000000009
    .quad _str_94
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_96
_data_96:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_68
    .quad _str_94
    .quad 0x0000000000000007
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_95
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_98
_data_98:
    .quad 0x0000000000000009
    .quad _str_97
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_99
_data_99:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_70
    .quad _str_97
    .quad 0x0000000000000009
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_98
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_101
_data_101:
    .quad 0x0000000000000009
    .quad _str_100
    .quad 0x0000000000000007
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_102
_data_102:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_72
    .quad _str_100
    .quad 0x0000000000000007
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_101
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_104
_data_104:
    .quad 0x0000000000000009
    .quad _str_103
    .quad 0x000000000000000b
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_105
_data_105:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_74
    .quad _str_103
    .quad 0x000000000000000b
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_104
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_107
_data_107:
    .quad 0x0000000000000009
    .quad _str_106
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_108
_data_108:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_76
    .quad _str_106
    .quad 0x0000000000000009
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_107
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_110
_data_110:
    .quad 0x0000000000000009
    .quad _str_109
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_111
_data_111:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_78
    .quad _str_109
    .quad 0x0000000000000009
    .quad _data_73
    .quad 0x0000000000000000
    .quad _data_110
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_54
.p2align 3
.globl _data_113
_data_113:
    .quad 0x0000000000000009
    .quad _str_112
    .quad 0x0000000000000005
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_114
_data_114:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_80
    .quad _str_112
    .quad 0x0000000000000005
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_113
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_121
_data_121:
    .quad _str_115
    .quad 0x0000000000000007
    .quad _str_116
    .quad 0x0000000000000007
    .quad _str_117
    .quad 0x0000000000000007
    .quad _str_118
    .quad 0x0000000000000005
    .quad _str_119
    .quad 0x0000000000000006
.p2align 3
.globl _data_122
_data_122:
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
    .quad 0x0000000000000007
    .quad 0x0000000000000008
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000008
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000008
    .quad 0x0000000000000001
.p2align 3
.globl _data_123
_data_123:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000006
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_124
_data_124:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_125
_data_125:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000001
.p2align 3
.globl _data_126
_data_126:
    .quad 0x0000000000000005
    .quad 0x0000000000000002
    .quad 0x0000000000000005
    .quad 0xffffffffffffffff
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_121
    .quad _data_122
    .quad _data_123
    .quad _data_124
    .quad _data_125
.p2align 3
.globl _data_127
_data_127:
    .quad 0x0000000000000009
    .quad _str_120
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_128
_data_128:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_82
    .quad _str_120
    .quad 0x000000000000000a
    .quad _data_126
    .quad 0x0000000000000000
    .quad _data_127
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_84
.p2align 3
.globl _data_130
_data_130:
    .quad 0x0000000000000009
    .quad _str_129
    .quad 0x000000000000000e
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_131
_data_131:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_86
    .quad _str_129
    .quad 0x000000000000000e
    .quad _data_126
    .quad 0x0000000000000000
    .quad _data_130
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_84
.p2align 3
.globl _data_133
_data_133:
    .quad 0x0000000000000009
    .quad _str_132
    .quad 0x000000000000000c
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_134
_data_134:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_88
    .quad _str_132
    .quad 0x000000000000000c
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_133
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_136
_data_136:
    .quad 0x0000000000000009
    .quad _str_135
    .quad 0x000000000000000c
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_137
_data_137:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_90
    .quad _str_135
    .quad 0x000000000000000c
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_136
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_139
_data_139:
    .quad 0x0000000000000009
    .quad _str_138
    .quad 0x000000000000000c
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_140
_data_140:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_92
    .quad _str_138
    .quad 0x000000000000000c
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_139
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_142
_data_142:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0xffffffffffffffff
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_29
    .quad _data_30
    .quad _data_21
    .quad _data_22
    .quad _data_23
.p2align 3
.globl _data_143
_data_143:
    .quad 0x0000000000000009
    .quad _str_141
    .quad 0x0000000000000006
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_144
_data_144:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_94
    .quad _str_141
    .quad 0x0000000000000006
    .quad _data_142
    .quad 0x0000000000000000
    .quad _data_143
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_96
.p2align 3
.globl _data_146
_data_146:
    .quad 0x0000000000000009
    .quad _str_145
    .quad 0x0000000000000006
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_147
_data_147:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_98
    .quad _str_145
    .quad 0x0000000000000006
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_146
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_149
_data_149:
    .quad 0x0000000000000009
    .quad _str_148
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_150
_data_150:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_100
    .quad _str_148
    .quad 0x000000000000000a
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_149
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_152
_data_152:
    .quad 0x0000000000000009
    .quad _str_151
    .quad 0x000000000000000a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_153
_data_153:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_102
    .quad _str_151
    .quad 0x000000000000000a
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_152
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_157
_data_157:
    .quad _str_27
    .quad 0x0000000000000006
    .quad _str_155
    .quad 0x000000000000000a
.p2align 3
.globl _data_158
_data_158:
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
.p2align 3
.globl _data_159
_data_159:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000002
    .quad _str_154
    .quad 0x0000000000000007
.p2align 3
.globl _data_160
_data_160:
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000002
    .quad 0xffffffffffffffff
    .quad 0x0000000000000001
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad _data_157
    .quad _data_158
    .quad _data_159
    .quad _data_67
    .quad _data_68
.p2align 3
.globl _data_161
_data_161:
    .quad 0x0000000000000009
    .quad _str_156
    .quad 0x0000000000000004
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_162
_data_162:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_104
    .quad _str_156
    .quad 0x0000000000000004
    .quad _data_160
    .quad 0x0000000000000000
    .quad _data_161
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_106
.p2align 3
.globl _data_164
_data_164:
    .quad 0x0000000000000009
    .quad _str_163
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_165
_data_165:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_108
    .quad _str_163
    .quad 0x0000000000000009
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_164
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_167
_data_167:
    .quad 0x0000000000000009
    .quad _str_166
    .quad 0x0000000000000009
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_168
_data_168:
    .quad 0x0000000000000009
    .quad _eir___elephc_run_shutdown_functions_callable_builtin_110
    .quad _str_166
    .quad 0x0000000000000009
    .quad _data_31
    .quad 0x0000000000000000
    .quad _data_167
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_32
.p2align 3
.globl _data_170
_data_170:
    .quad 0x000000000000000b
    .quad _str_169
    .quad 0x000000000000001f
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_171
_data_171:
    .quad 0x000000000000000b
    .quad _fn__u__u_elephc_u_run_u_shutdown_u_functions
    .quad _str_169
    .quad 0x000000000000001f
    .quad _data_2
    .quad 0x0000000000000000
    .quad _data_170
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_112
.p2align 3
.globl _data_173
_data_173:
    .quad _str_3
    .quad 0x0000000000000002
    .quad _str_4
    .quad 0x0000000000000004
.p2align 3
.globl _data_174
_data_174:
    .quad 0x000000000000000a
    .quad 0x0000000000000008
    .quad 0x0000000000000001
    .quad 0x0000000000000004
    .quad 0x0000000000000008
    .quad 0x0000000000000001
.p2align 3
.globl _data_175
_data_175:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_176
_data_176:
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad 0xffffffffffffffff
    .quad 0x000000000000000a
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_173
    .quad _data_174
    .quad _data_175
    .quad _data_67
    .quad _data_68
.p2align 3
.globl _data_177
_data_177:
    .quad 0x000000000000000b
    .quad _str_172
    .quad 0x0000000000000016
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_178
_data_178:
    .quad 0x000000000000000b
    .quad _fn__u__u_elephc_u_shutdown_u_wrap
    .quad _str_172
    .quad 0x0000000000000016
    .quad _data_176
    .quad 0x0000000000000000
    .quad _data_177
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_114
.p2align 3
.globl _data_182
_data_182:
    .quad _str_179
    .quad 0x0000000000000002
    .quad _str_180
    .quad 0x0000000000000005
.p2align 3
.globl _data_183
_data_183:
    .quad 0x0000000000000001
    .quad 0x0000000000000010
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad 0x0000000000000008
    .quad 0x0000000000000001
.p2align 3
.globl _data_184
_data_184:
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad 0x0000000000000002
    .quad 0xffffffffffffffff
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad _data_182
    .quad _data_183
    .quad _data_175
    .quad _data_67
    .quad _data_68
.p2align 3
.globl _data_185
_data_185:
    .quad 0x000000000000000b
    .quad _str_181
    .quad 0x000000000000000c
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_186
_data_186:
    .quad 0x000000000000000b
    .quad _fn_processOrder
    .quad _str_181
    .quad 0x000000000000000c
    .quad _data_184
    .quad 0x0000000000000000
    .quad _data_185
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_116
.p2align 3
.globl _data_189
_data_189:
    .quad _str_187
    .quad 0x0000000000000008
    .quad _str_4
    .quad 0x0000000000000004
.p2align 3
.globl _data_190
_data_190:
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000001
    .quad _data_189
    .quad _data_174
    .quad _data_21
    .quad _data_67
    .quad _data_68
.p2align 3
.globl _data_191
_data_191:
    .quad 0x000000000000000b
    .quad _str_188
    .quad 0x000000000000001a
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_192
_data_192:
    .quad 0x000000000000000b
    .quad _fn_register_u_shutdown_u_function
    .quad _str_188
    .quad 0x000000000000001a
    .quad _data_190
    .quad 0x0000000000000000
    .quad _data_191
    .quad _eir___elephc_run_shutdown_functions_callable_invoker_118
.p2align 3
.globl _data_238
_data_238:
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0xffffffffffffffff
    .quad 0x0000000000000008
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
    .quad 0x0000000000000000
.p2align 3
.globl _data_239
_data_239:
    .quad _str_179
    .quad 0x0000000000000002
    .quad 0x0000000000000001
    .quad 0x0000000000000000
.p2align 3
.globl _data_240
_data_240:
    .quad 0x0000000000000001
    .quad 0x0000000000000001
    .quad _data_239
    .quad _data_239
    .quad 0x0000000000000000
.p2align 3
.globl _data_241
_data_241:
    .quad 0x0000000000000001
    .quad _fn__u__u_eir_u_closure_u_processOrder_u_0
    .quad _str_237
    .quad 0x000000000000001c
    .quad _data_238
    .quad _data_240
    .quad _data_7
    .quad _eir_processOrder_callable_invoker_0
.p2align 3
.globl _data_252
_data_252:
    .quad 0x0000000000000001
    .quad _fn__u__u_eir_u_closure_u_main_u_0
    .quad _str_251
    .quad 0x0000000000000014
    .quad _data_238
    .quad 0x0000000000000000
    .quad _data_7
    .quad _eir_main_callable_invoker_0

.comm _static_prop__u__u_ElephcShutdownRegistry_callbacks, 16, 3
.comm _static_prop__u__u_ElephcShutdownRegistry_running, 16, 3
.comm _enum_case_PropertyHookType_Get, 8, 3
.comm _enum_case_PropertyHookType_Set, 8, 3
.comm _enum_case_SortDirection_Ascending, 8, 3
.comm _enum_case_SortDirection_Descending, 8, 3
.data
.p2align 3
.globl _callable_user_fn_name_0
_callable_user_fn_name_0:
    .ascii "__elephc_run_shutdown_functions"
.globl _callable_user_fn_name_1
_callable_user_fn_name_1:
    .ascii "__elephc_shutdown_wrap"
.globl _callable_user_fn_name_2
_callable_user_fn_name_2:
    .ascii "main"
.globl _callable_user_fn_name_3
_callable_user_fn_name_3:
    .ascii "processOrder"
.globl _callable_user_fn_name_4
_callable_user_fn_name_4:
    .ascii "register_shutdown_function"
.p2align 3
.globl _callable_user_function_count
_callable_user_function_count:
    .quad 5
.globl _callable_user_function_table
_callable_user_function_table:
    .quad _callable_user_fn_name_0
    .quad 31
    .quad 0
    .quad _callable_user_fn_name_1
    .quad 22
    .quad 0
    .quad _callable_user_fn_name_2
    .quad 4
    .quad 0
    .quad _callable_user_fn_name_3
    .quad 12
    .quad 0
    .quad _callable_user_fn_name_4
    .quad 26
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
    .quad _instanceof_name_class_18
    .quad 16
    .quad 18
    .quad 0
    .quad _instanceof_name_class_abs_18
    .quad 17
    .quad 18
    .quad 0
    .quad _instanceof_name_class_19
    .quad 20
    .quad 19
    .quad 0
    .quad _instanceof_name_class_abs_19
    .quad 21
    .quad 19
    .quad 0
    .quad _instanceof_name_class_31
    .quad 5
    .quad 31
    .quad 0
    .quad _instanceof_name_class_abs_31
    .quad 6
    .quad 31
    .quad 0
    .quad _instanceof_name_class_32
    .quad 10
    .quad 32
    .quad 0
    .quad _instanceof_name_class_abs_32
    .quad 11
    .quad 32
    .quad 0
    .quad _instanceof_name_class_48
    .quad 19
    .quad 48
    .quad 0
    .quad _instanceof_name_class_abs_48
    .quad 20
    .quad 48
    .quad 0
    .quad _instanceof_name_class_49
    .quad 9
    .quad 49
    .quad 0
    .quad _instanceof_name_class_abs_49
    .quad 10
    .quad 49
    .quad 0
    .quad _instanceof_name_class_56
    .quad 13
    .quad 56
    .quad 0
    .quad _instanceof_name_class_abs_56
    .quad 14
    .quad 56
    .quad 0
    .quad _instanceof_name_class_77
    .quad 19
    .quad 77
    .quad 0
    .quad _instanceof_name_class_abs_77
    .quad 20
    .quad 77
    .quad 0
    .quad _instanceof_name_class_79
    .quad 24
    .quad 79
    .quad 0
    .quad _instanceof_name_class_abs_79
    .quad 25
    .quad 79
    .quad 0
    .quad _instanceof_name_class_80
    .quad 24
    .quad 80
    .quad 0
    .quad _instanceof_name_class_abs_80
    .quad 25
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
    .quad 8
    .quad 85
    .quad 0
    .quad _instanceof_name_class_abs_85
    .quad 9
    .quad 85
    .quad 0
    .quad _instanceof_name_class_89
    .quad 19
    .quad 89
    .quad 0
    .quad _instanceof_name_class_abs_89
    .quad 20
    .quad 89
    .quad 0
    .quad _instanceof_name_interface_1
    .quad 10
    .quad 1
    .quad 1
    .quad _instanceof_name_interface_abs_1
    .quad 11
    .quad 1
    .quad 1
    .quad _instanceof_name_interface_5
    .quad 9
    .quad 5
    .quad 1
    .quad _instanceof_name_interface_abs_5
    .quad 10
    .quad 5
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
.globl _instanceof_name_class_18
_instanceof_name_class_18:
    .ascii "RuntimeException"
.globl _instanceof_name_class_abs_18
_instanceof_name_class_abs_18:
    .ascii "\\RuntimeException"
.globl _instanceof_name_class_19
_instanceof_name_class_19:
    .ascii "OutOfBoundsException"
.globl _instanceof_name_class_abs_19
_instanceof_name_class_abs_19:
    .ascii "\\OutOfBoundsException"
.globl _instanceof_name_class_31
_instanceof_name_class_31:
    .ascii "Error"
.globl _instanceof_name_class_abs_31
_instanceof_name_class_abs_31:
    .ascii "\\Error"
.globl _instanceof_name_class_32
_instanceof_name_class_32:
    .ascii "ValueError"
.globl _instanceof_name_class_abs_32
_instanceof_name_class_abs_32:
    .ascii "\\ValueError"
.globl _instanceof_name_class_48
_instanceof_name_class_48:
    .ascii "OutOfRangeException"
.globl _instanceof_name_class_abs_48
_instanceof_name_class_abs_48:
    .ascii "\\OutOfRangeException"
.globl _instanceof_name_class_49
_instanceof_name_class_49:
    .ascii "TypeError"
.globl _instanceof_name_class_abs_49
_instanceof_name_class_abs_49:
    .ascii "\\TypeError"
.globl _instanceof_name_class_56
_instanceof_name_class_56:
    .ascii "JsonException"
.globl _instanceof_name_class_abs_56
_instanceof_name_class_abs_56:
    .ascii "\\JsonException"
.globl _instanceof_name_class_77
_instanceof_name_class_77:
    .ascii "ReflectionException"
.globl _instanceof_name_class_abs_77
_instanceof_name_class_abs_77:
    .ascii "\\ReflectionException"
.globl _instanceof_name_class_79
_instanceof_name_class_79:
    .ascii "__ElephcShutdownRegistry"
.globl _instanceof_name_class_abs_79
_instanceof_name_class_abs_79:
    .ascii "\\__ElephcShutdownRegistry"
.globl _instanceof_name_class_80
_instanceof_name_class_80:
    .ascii "InvalidArgumentException"
.globl _instanceof_name_class_abs_80
_instanceof_name_class_abs_80:
    .ascii "\\InvalidArgumentException"
.globl _instanceof_name_class_82
_instanceof_name_class_82:
    .ascii "ArithmeticError"
.globl _instanceof_name_class_abs_82
_instanceof_name_class_abs_82:
    .ascii "\\ArithmeticError"
.globl _instanceof_name_class_85
_instanceof_name_class_85:
    .ascii "stdClass"
.globl _instanceof_name_class_abs_85
_instanceof_name_class_abs_85:
    .ascii "\\stdClass"
.globl _instanceof_name_class_89
_instanceof_name_class_89:
    .ascii "UnhandledMatchError"
.globl _instanceof_name_class_abs_89
_instanceof_name_class_abs_89:
    .ascii "\\UnhandledMatchError"
.globl _instanceof_name_interface_1
_instanceof_name_interface_1:
    .ascii "Stringable"
.globl _instanceof_name_interface_abs_1
_instanceof_name_interface_abs_1:
    .ascii "\\Stringable"
.globl _instanceof_name_interface_5
_instanceof_name_interface_5:
    .ascii "Throwable"
.globl _instanceof_name_interface_abs_5
_instanceof_name_interface_abs_5:
    .ascii "\\Throwable"
    .p2align 3
.p2align 3
.globl _class_name_count
_class_name_count:
    .quad 90
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_missing
    .quad 0
    .quad _class_name_18
    .quad 16
    .quad _class_name_19
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
    .quad _class_name_31
    .quad 5
    .quad _class_name_32
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
    .quad _class_name_missing
    .quad 0
    .quad _class_name_48
    .quad 19
    .quad _class_name_49
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
    .quad _class_name_56
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
    .quad _class_name_missing
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
    .quad 19
    .quad _class_name_missing
    .quad 0
    .quad _class_name_79
    .quad 24
    .quad _class_name_80
    .quad 24
    .quad _class_name_missing
    .quad 0
    .quad _class_name_82
    .quad 15
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
    .quad _class_name_89
    .quad 19
.globl _class_name_missing
_class_name_missing:
    .byte 0
.globl _class_name_0
_class_name_0:
    .ascii "Exception"
.globl _class_name_1
_class_name_1:
    .ascii "LogicException"
.globl _class_name_18
_class_name_18:
    .ascii "RuntimeException"
.globl _class_name_19
_class_name_19:
    .ascii "OutOfBoundsException"
.globl _class_name_31
_class_name_31:
    .ascii "Error"
.globl _class_name_32
_class_name_32:
    .ascii "ValueError"
.globl _class_name_48
_class_name_48:
    .ascii "OutOfRangeException"
.globl _class_name_49
_class_name_49:
    .ascii "TypeError"
.globl _class_name_56
_class_name_56:
    .ascii "JsonException"
.globl _class_name_77
_class_name_77:
    .ascii "ReflectionException"
.globl _class_name_79
_class_name_79:
    .ascii "__ElephcShutdownRegistry"
.globl _class_name_80
_class_name_80:
    .ascii "InvalidArgumentException"
.globl _class_name_82
_class_name_82:
    .ascii "ArithmeticError"
.globl _class_name_85
_class_name_85:
    .ascii "stdClass"
.globl _class_name_89
_class_name_89:
    .ascii "UnhandledMatchError"
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
    .quad 102
.globl _fiber_error_class_id
_fiber_error_class_id:
    .quad 60
.globl _generator_class_id
_generator_class_id:
    .quad 75
.globl _spl_dll_class_id
_spl_dll_class_id:
    .quad 16
.globl _spl_stack_class_id
_spl_stack_class_id:
    .quad 73
.globl _spl_queue_class_id
_spl_queue_class_id:
    .quad 81
.globl _spl_fixed_array_class_id
_spl_fixed_array_class_id:
    .quad 21
.globl _spl_error_class_id
_spl_error_class_id:
    .quad 31
.globl _spl_logic_exception_class_id
_spl_logic_exception_class_id:
    .quad 1
.globl _spl_runtime_exception_class_id
_spl_runtime_exception_class_id:
    .quad 18
.globl _spl_out_of_range_exception_class_id
_spl_out_of_range_exception_class_id:
    .quad 48
.globl _spl_out_of_bounds_exception_class_id
_spl_out_of_bounds_exception_class_id:
    .quad 19
.globl _spl_invalid_argument_exception_class_id
_spl_invalid_argument_exception_class_id:
    .quad 80
.globl _spl_type_error_class_id
_spl_type_error_class_id:
    .quad 49
.globl _spl_value_error_class_id
_spl_value_error_class_id:
    .quad 32
.globl _reflection_exception_class_id
_reflection_exception_class_id:
    .quad 77
.globl _spl_arithmetic_error_class_id
_spl_arithmetic_error_class_id:
    .quad 82
.globl _interface_count
_interface_count:
    .quad 2
.globl _interface_method_ptrs
_interface_method_ptrs:
    .quad _interface_methods_1
    .quad _interface_methods_5
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
    .quad _class_interfaces_18
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
    .quad _class_interfaces_31
    .quad _class_interfaces_32
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
    .quad _class_interfaces_48
    .quad _class_interfaces_49
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
    .quad _class_interfaces_77
    .quad _class_interfaces_missing
    .quad _class_interfaces_79
    .quad _class_interfaces_80
    .quad _class_interfaces_missing
    .quad _class_interfaces_82
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_85
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_missing
    .quad _class_interfaces_89
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
    .quad _class_json_desc_18
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
    .quad _class_json_desc_31
    .quad _class_json_desc_32
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
    .quad _class_json_desc_48
    .quad _class_json_desc_49
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
    .quad _class_json_desc_77
    .quad _class_json_desc_missing
    .quad _class_json_desc_79
    .quad _class_json_desc_80
    .quad _class_json_desc_missing
    .quad _class_json_desc_82
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_85
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_missing
    .quad _class_json_desc_89
.globl _json_exception_class_id
_json_exception_class_id:
    .quad 56
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
    .quad 0
    .quad 18
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 31
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 31
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 18
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
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
    .quad 1
    .quad -1
    .quad 31
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad -1
    .quad 31
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
    .quad 72
    .quad 0
    .quad 8
    .quad 72
    .quad 0
    .quad 72
    .quad 0
    .quad 0
    .quad 16
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
.globl _class_gc_desc_count
_class_gc_desc_count:
    .quad 90
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
    .quad _class_gc_desc_18
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
    .quad _class_gc_desc_31
    .quad _class_gc_desc_32
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
    .quad _class_gc_desc_48
    .quad _class_gc_desc_49
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
    .quad _class_gc_desc_77
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_79
    .quad _class_gc_desc_80
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_82
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_85
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_missing
    .quad _class_gc_desc_89
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
    .quad _class_vtable_18
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
    .quad _class_vtable_31
    .quad _class_vtable_32
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
    .quad _class_vtable_48
    .quad _class_vtable_49
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
    .quad _class_vtable_77
    .quad _class_vtable_missing
    .quad _class_vtable_79
    .quad _class_vtable_80
    .quad _class_vtable_missing
    .quad _class_vtable_82
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_85
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_missing
    .quad _class_vtable_89
.globl _class_destruct_count
_class_destruct_count:
    .quad 90
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
.globl _class_clone_count
_class_clone_count:
    .quad 90
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
    .quad _class_propinit_18
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
    .quad _class_propinit_31
    .quad _class_propinit_32
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_48
    .quad _class_propinit_49
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
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
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
    .quad _class_propinit_80
    .quad 0
    .quad _class_propinit_82
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad _class_propinit_89
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
    .quad _class_serprop_18
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
    .quad _class_serprop_31
    .quad _class_serprop_32
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
    .quad _class_serprop_48
    .quad _class_serprop_49
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
    .quad _class_serprop_77
    .quad _class_serprop_missing
    .quad _class_serprop_79
    .quad _class_serprop_80
    .quad _class_serprop_missing
    .quad _class_serprop_82
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_85
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_missing
    .quad _class_serprop_89
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
    .quad _class_static_vtable_18
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
    .quad _class_static_vtable_31
    .quad _class_static_vtable_32
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
    .quad _class_static_vtable_48
    .quad _class_static_vtable_49
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
    .quad _class_static_vtable_77
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_79
    .quad _class_static_vtable_80
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_82
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_85
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_missing
    .quad _class_static_vtable_89
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
    .quad _class_callable_methods_18
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
    .quad _class_callable_methods_31
    .quad _class_callable_methods_32
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
    .quad _class_callable_methods_48
    .quad _class_callable_methods_49
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
    .quad _class_callable_methods_77
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_79
    .quad _class_callable_methods_80
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_82
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_85
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_missing
    .quad _class_callable_methods_89
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
.globl _class_by_name_str_18
_class_by_name_str_18:
    .ascii "RuntimeException"
.globl _class_by_name_str_19
_class_by_name_str_19:
    .ascii "OutOfBoundsException"
.globl _class_by_name_str_31
_class_by_name_str_31:
    .ascii "Error"
.globl _class_by_name_str_32
_class_by_name_str_32:
    .ascii "ValueError"
.globl _class_by_name_str_48
_class_by_name_str_48:
    .ascii "OutOfRangeException"
.globl _class_by_name_str_49
_class_by_name_str_49:
    .ascii "TypeError"
.globl _class_by_name_str_56
_class_by_name_str_56:
    .ascii "JsonException"
.globl _class_by_name_str_77
_class_by_name_str_77:
    .ascii "ReflectionException"
.globl _class_by_name_str_79
_class_by_name_str_79:
    .ascii "__ElephcShutdownRegistry"
.globl _class_by_name_str_80
_class_by_name_str_80:
    .ascii "InvalidArgumentException"
.globl _class_by_name_str_82
_class_by_name_str_82:
    .ascii "ArithmeticError"
.globl _class_by_name_str_85
_class_by_name_str_85:
    .ascii "stdClass"
.globl _class_by_name_str_89
_class_by_name_str_89:
    .ascii "UnhandledMatchError"
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
    .quad _class_by_name_str_18
    .quad 16
    .quad 18
    .quad 72
    .quad _class_by_name_str_19
    .quad 20
    .quad 19
    .quad 72
    .quad _class_by_name_str_31
    .quad 5
    .quad 31
    .quad 72
    .quad _class_by_name_str_32
    .quad 10
    .quad 32
    .quad 72
    .quad _class_by_name_str_48
    .quad 19
    .quad 48
    .quad 72
    .quad _class_by_name_str_49
    .quad 9
    .quad 49
    .quad 72
    .quad _class_by_name_str_56
    .quad 13
    .quad 56
    .quad 72
    .quad _class_by_name_str_77
    .quad 19
    .quad 77
    .quad 72
    .quad _class_by_name_str_79
    .quad 24
    .quad 79
    .quad 8
    .quad _class_by_name_str_80
    .quad 24
    .quad 80
    .quad 72
    .quad _class_by_name_str_82
    .quad 15
    .quad 82
    .quad 72
    .quad _class_by_name_str_85
    .quad 8
    .quad 85
    .quad 16
    .quad _class_by_name_str_89
    .quad 19
    .quad 89
    .quad 72
.p2align 3
.globl _class_attribute_count
_class_attribute_count:
    .quad 90
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
.globl _class_attributes_missing
_class_attributes_missing:
    .quad 0
.globl _interface_methods_1
_interface_methods_1:
    .quad 1
    .quad 0
.globl _interface_methods_5
_interface_methods_5:
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
    .quad 5
    .quad _class_interface_impl_0_5
    .quad 1
    .quad _class_interface_impl_0_1
.globl _class_interface_impl_0_5
_class_interface_impl_0_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_0_1
_class_interface_impl_0_1:
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
    .quad 5
    .quad _class_interface_impl_1_5
    .quad 1
    .quad _class_interface_impl_1_1
.globl _class_interface_impl_1_5
_class_interface_impl_1_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_1_1
_class_interface_impl_1_1:
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
.globl _class_interfaces_18
_class_interfaces_18:
    .quad 2
    .quad 5
    .quad _class_interface_impl_18_5
    .quad 1
    .quad _class_interface_impl_18_1
.globl _class_interface_impl_18_5
_class_interface_impl_18_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_18_1
_class_interface_impl_18_1:
    .quad 0
.globl _class_json_pname_18_0
_class_json_pname_18_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_18
_class_json_desc_18:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_18_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_18
_class_gc_desc_18:
    .byte 1, 0, 7, 4
.globl _class_serpname_18_0
_class_serpname_18_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_18_1
_class_serpname_18_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_18_2
_class_serpname_18_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_18_3
_class_serpname_18_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_18
_class_serprop_18:
    .quad 4
    .quad _class_serpname_18_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_18_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_18_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_18_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_18
_class_vtable_18:
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
.globl _class_static_vtable_18
_class_static_vtable_18:
    .quad 0
.globl _class_callable_method_name_18__u__u_construct
_class_callable_method_name_18__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_18__u__u_tostring
_class_callable_method_name_18__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_18_getcode
_class_callable_method_name_18_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_18_getfile
_class_callable_method_name_18_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_18_getline
_class_callable_method_name_18_getline:
    .ascii "getline"
.globl _class_callable_method_name_18_getmessage
_class_callable_method_name_18_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_18_getprevious
_class_callable_method_name_18_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_18_gettrace
_class_callable_method_name_18_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_18_gettraceasstring
_class_callable_method_name_18_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_18
_class_callable_methods_18:
    .quad 9
    .quad _class_callable_method_name_18__u__u_construct
    .quad 11
    .quad _class_callable_method_name_18__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_18_getcode
    .quad 7
    .quad _class_callable_method_name_18_getfile
    .quad 7
    .quad _class_callable_method_name_18_getline
    .quad 7
    .quad _class_callable_method_name_18_getmessage
    .quad 10
    .quad _class_callable_method_name_18_getprevious
    .quad 11
    .quad _class_callable_method_name_18_gettrace
    .quad 8
    .quad _class_callable_method_name_18_gettraceasstring
    .quad 16
.globl _class_interfaces_19
_class_interfaces_19:
    .quad 2
    .quad 5
    .quad _class_interface_impl_19_5
    .quad 1
    .quad _class_interface_impl_19_1
.globl _class_interface_impl_19_5
_class_interface_impl_19_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_19_1
_class_interface_impl_19_1:
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
.globl _class_interfaces_31
_class_interfaces_31:
    .quad 2
    .quad 5
    .quad _class_interface_impl_31_5
    .quad 1
    .quad _class_interface_impl_31_1
.globl _class_interface_impl_31_5
_class_interface_impl_31_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_31_1
_class_interface_impl_31_1:
    .quad 0
.globl _class_json_pname_31_0
_class_json_pname_31_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_31
_class_json_desc_31:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_31_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_31
_class_gc_desc_31:
    .byte 1, 0, 7, 4
.globl _class_serpname_31_0
_class_serpname_31_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_31_1
_class_serpname_31_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_31_2
_class_serpname_31_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_31_3
_class_serpname_31_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_31
_class_serprop_31:
    .quad 4
    .quad _class_serpname_31_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_31_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_31_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_31_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_31
_class_vtable_31:
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
.globl _class_static_vtable_31
_class_static_vtable_31:
    .quad 0
.globl _class_callable_method_name_31__u__u_construct
_class_callable_method_name_31__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_31__u__u_tostring
_class_callable_method_name_31__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_31_getcode
_class_callable_method_name_31_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_31_getfile
_class_callable_method_name_31_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_31_getline
_class_callable_method_name_31_getline:
    .ascii "getline"
.globl _class_callable_method_name_31_getmessage
_class_callable_method_name_31_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_31_getprevious
_class_callable_method_name_31_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_31_gettrace
_class_callable_method_name_31_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_31_gettraceasstring
_class_callable_method_name_31_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_31
_class_callable_methods_31:
    .quad 9
    .quad _class_callable_method_name_31__u__u_construct
    .quad 11
    .quad _class_callable_method_name_31__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_31_getcode
    .quad 7
    .quad _class_callable_method_name_31_getfile
    .quad 7
    .quad _class_callable_method_name_31_getline
    .quad 7
    .quad _class_callable_method_name_31_getmessage
    .quad 10
    .quad _class_callable_method_name_31_getprevious
    .quad 11
    .quad _class_callable_method_name_31_gettrace
    .quad 8
    .quad _class_callable_method_name_31_gettraceasstring
    .quad 16
.globl _class_interfaces_32
_class_interfaces_32:
    .quad 2
    .quad 5
    .quad _class_interface_impl_32_5
    .quad 1
    .quad _class_interface_impl_32_1
.globl _class_interface_impl_32_5
_class_interface_impl_32_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_32_1
_class_interface_impl_32_1:
    .quad 0
.globl _class_json_pname_32_0
_class_json_pname_32_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_32
_class_json_desc_32:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_32_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_32
_class_gc_desc_32:
    .byte 1, 0, 7, 4
.globl _class_serpname_32_0
_class_serpname_32_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_32_1
_class_serpname_32_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_32_2
_class_serpname_32_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_32_3
_class_serpname_32_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_32
_class_serprop_32:
    .quad 4
    .quad _class_serpname_32_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_32_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_32_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_32_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_32
_class_vtable_32:
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
.globl _class_static_vtable_32
_class_static_vtable_32:
    .quad 0
.globl _class_callable_method_name_32__u__u_construct
_class_callable_method_name_32__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_32__u__u_tostring
_class_callable_method_name_32__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_32_getcode
_class_callable_method_name_32_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_32_getfile
_class_callable_method_name_32_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_32_getline
_class_callable_method_name_32_getline:
    .ascii "getline"
.globl _class_callable_method_name_32_getmessage
_class_callable_method_name_32_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_32_getprevious
_class_callable_method_name_32_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_32_gettrace
_class_callable_method_name_32_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_32_gettraceasstring
_class_callable_method_name_32_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_32
_class_callable_methods_32:
    .quad 9
    .quad _class_callable_method_name_32__u__u_construct
    .quad 11
    .quad _class_callable_method_name_32__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_32_getcode
    .quad 7
    .quad _class_callable_method_name_32_getfile
    .quad 7
    .quad _class_callable_method_name_32_getline
    .quad 7
    .quad _class_callable_method_name_32_getmessage
    .quad 10
    .quad _class_callable_method_name_32_getprevious
    .quad 11
    .quad _class_callable_method_name_32_gettrace
    .quad 8
    .quad _class_callable_method_name_32_gettraceasstring
    .quad 16
.globl _class_interfaces_48
_class_interfaces_48:
    .quad 2
    .quad 5
    .quad _class_interface_impl_48_5
    .quad 1
    .quad _class_interface_impl_48_1
.globl _class_interface_impl_48_5
_class_interface_impl_48_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_48_1
_class_interface_impl_48_1:
    .quad 0
.globl _class_json_pname_48_0
_class_json_pname_48_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_48
_class_json_desc_48:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_48_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_48
_class_gc_desc_48:
    .byte 1, 0, 7, 4
.globl _class_serpname_48_0
_class_serpname_48_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_48_1
_class_serpname_48_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_48_2
_class_serpname_48_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_48_3
_class_serpname_48_3:
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_48
_class_serprop_48:
    .quad 4
    .quad _class_serpname_48_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_48_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_48_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_48_3
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_48
_class_vtable_48:
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
.globl _class_static_vtable_48
_class_static_vtable_48:
    .quad 0
.globl _class_callable_method_name_48__u__u_construct
_class_callable_method_name_48__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_48__u__u_tostring
_class_callable_method_name_48__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_48_getcode
_class_callable_method_name_48_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_48_getfile
_class_callable_method_name_48_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_48_getline
_class_callable_method_name_48_getline:
    .ascii "getline"
.globl _class_callable_method_name_48_getmessage
_class_callable_method_name_48_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_48_getprevious
_class_callable_method_name_48_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_48_gettrace
_class_callable_method_name_48_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_48_gettraceasstring
_class_callable_method_name_48_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_48
_class_callable_methods_48:
    .quad 9
    .quad _class_callable_method_name_48__u__u_construct
    .quad 11
    .quad _class_callable_method_name_48__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_48_getcode
    .quad 7
    .quad _class_callable_method_name_48_getfile
    .quad 7
    .quad _class_callable_method_name_48_getline
    .quad 7
    .quad _class_callable_method_name_48_getmessage
    .quad 10
    .quad _class_callable_method_name_48_getprevious
    .quad 11
    .quad _class_callable_method_name_48_gettrace
    .quad 8
    .quad _class_callable_method_name_48_gettraceasstring
    .quad 16
.globl _class_interfaces_49
_class_interfaces_49:
    .quad 2
    .quad 5
    .quad _class_interface_impl_49_5
    .quad 1
    .quad _class_interface_impl_49_1
.globl _class_interface_impl_49_5
_class_interface_impl_49_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_49_1
_class_interface_impl_49_1:
    .quad 0
.globl _class_json_pname_49_0
_class_json_pname_49_0:
    .ascii "message"
    .p2align 3
.globl _class_json_desc_49
_class_json_desc_49:
    .quad 0
    .quad 0
    .quad 1
    .quad _class_json_pname_49_0
    .quad 7
    .quad 0
    .quad 1
    .p2align 3
.globl _class_gc_desc_49
_class_gc_desc_49:
    .byte 1, 0, 7, 4
.globl _class_serpname_49_0
_class_serpname_49_0:
    .byte 109, 101, 115, 115, 97, 103, 101
.globl _class_serpname_49_1
_class_serpname_49_1:
    .byte 0, 42, 0, 99, 111, 100, 101
.globl _class_serpname_49_2
_class_serpname_49_2:
    .byte 0, 42, 0, 112, 114, 101, 118, 105, 111, 117, 115
.globl _class_serpname_49_3
_class_serpname_49_3:
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
    .p2align 3
.globl _class_serprop_49
_class_serprop_49:
    .quad 4
    .quad _class_serpname_49_0
    .quad 7
    .quad 8
    .quad 1
    .quad _class_serpname_49_1
    .quad 7
    .quad 24
    .quad 0
    .quad _class_serpname_49_2
    .quad 11
    .quad 40
    .quad 7
    .quad _class_serpname_49_3
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_49
_class_vtable_49:
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
.globl _class_static_vtable_49
_class_static_vtable_49:
    .quad 0
.globl _class_callable_method_name_49__u__u_construct
_class_callable_method_name_49__u__u_construct:
    .ascii "__construct"
.globl _class_callable_method_name_49__u__u_tostring
_class_callable_method_name_49__u__u_tostring:
    .ascii "__tostring"
.globl _class_callable_method_name_49_getcode
_class_callable_method_name_49_getcode:
    .ascii "getcode"
.globl _class_callable_method_name_49_getfile
_class_callable_method_name_49_getfile:
    .ascii "getfile"
.globl _class_callable_method_name_49_getline
_class_callable_method_name_49_getline:
    .ascii "getline"
.globl _class_callable_method_name_49_getmessage
_class_callable_method_name_49_getmessage:
    .ascii "getmessage"
.globl _class_callable_method_name_49_getprevious
_class_callable_method_name_49_getprevious:
    .ascii "getprevious"
.globl _class_callable_method_name_49_gettrace
_class_callable_method_name_49_gettrace:
    .ascii "gettrace"
.globl _class_callable_method_name_49_gettraceasstring
_class_callable_method_name_49_gettraceasstring:
    .ascii "gettraceasstring"
.p2align 3
.globl _class_callable_methods_49
_class_callable_methods_49:
    .quad 9
    .quad _class_callable_method_name_49__u__u_construct
    .quad 11
    .quad _class_callable_method_name_49__u__u_tostring
    .quad 10
    .quad _class_callable_method_name_49_getcode
    .quad 7
    .quad _class_callable_method_name_49_getfile
    .quad 7
    .quad _class_callable_method_name_49_getline
    .quad 7
    .quad _class_callable_method_name_49_getmessage
    .quad 10
    .quad _class_callable_method_name_49_getprevious
    .quad 11
    .quad _class_callable_method_name_49_gettrace
    .quad 8
    .quad _class_callable_method_name_49_gettraceasstring
    .quad 16
.globl _class_interfaces_56
_class_interfaces_56:
    .quad 2
    .quad 5
    .quad _class_interface_impl_56_5
    .quad 1
    .quad _class_interface_impl_56_1
.globl _class_interface_impl_56_5
_class_interface_impl_56_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_56_1
_class_interface_impl_56_1:
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
.globl _class_interfaces_77
_class_interfaces_77:
    .quad 2
    .quad 5
    .quad _class_interface_impl_77_5
    .quad 1
    .quad _class_interface_impl_77_1
.globl _class_interface_impl_77_5
_class_interface_impl_77_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_77_1
_class_interface_impl_77_1:
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
    .byte 0, 69, 120, 99, 101, 112, 116, 105, 111, 110, 0, 116, 114, 97, 99, 101
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
    .quad 16
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_77
_class_vtable_77:
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
.globl _class_interfaces_79
_class_interfaces_79:
    .quad 0
    .p2align 3
.globl _class_json_desc_79
_class_json_desc_79:
    .quad 0
    .quad 0
    .quad 0
    .p2align 3
.globl _class_gc_desc_79
_class_gc_desc_79:
    .byte 0
    .p2align 3
.globl _class_serprop_79
_class_serprop_79:
    .quad 0
    .p2align 3
.globl _class_vtable_79
_class_vtable_79:
    .quad 0
    .p2align 3
.globl _class_static_vtable_79
_class_static_vtable_79:
    .quad 0
.p2align 3
.globl _class_callable_methods_79
_class_callable_methods_79:
    .quad 0
.globl _class_interfaces_80
_class_interfaces_80:
    .quad 2
    .quad 5
    .quad _class_interface_impl_80_5
    .quad 1
    .quad _class_interface_impl_80_1
.globl _class_interface_impl_80_5
_class_interface_impl_80_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_80_1
_class_interface_impl_80_1:
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
    .quad 5
    .quad _class_interface_impl_82_5
    .quad 1
    .quad _class_interface_impl_82_1
.globl _class_interface_impl_82_5
_class_interface_impl_82_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_82_1
_class_interface_impl_82_1:
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
.globl _class_interfaces_89
_class_interfaces_89:
    .quad 2
    .quad 5
    .quad _class_interface_impl_89_5
    .quad 1
    .quad _class_interface_impl_89_1
.globl _class_interface_impl_89_5
_class_interface_impl_89_5:
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
    .quad 0
.globl _class_interface_impl_89_1
_class_interface_impl_89_1:
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
    .byte 0, 69, 114, 114, 111, 114, 0, 116, 114, 97, 99, 101
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
    .quad 12
    .quad 56
    .quad 4
    .p2align 3
.globl _class_vtable_89
_class_vtable_89:
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
.p2align 3
.globl _stdclass_class_id
_stdclass_class_id:
    .quad 85

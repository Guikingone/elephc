__rt_bcmath_binary:
    sub sp, sp, #96
    stp x29, x30, [sp, #80]
    add x29, sp, #80
    stp x1, x2, [sp, #0]
    stp x3, x4, [sp, #16]
    str x5, [sp, #32]
    str x6, [sp, #40]
    str x9, [sp, #48]
    stp xzr, xzr, [sp, #56]
    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldr x4, [sp, #32]
    ldr x5, [sp, #40]
    add x6, sp, #56
    add x7, sp, #64
    ldr x9, [sp, #48]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_binary_error
1:
    ldp x1, x2, [sp, #56]
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    b __rt_bcmath_finish_string
__rt_bcmath_binary_error:
    ldp x29, x30, [sp, #80]
    add sp, sp, #96
    b __rt_bcmath_throw

__rt_bcmath_unary_scaled:
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    stp x1, x2, [sp, #0]
    str x5, [sp, #16]
    str x6, [sp, #24]
    str x9, [sp, #32]
    stp xzr, xzr, [sp, #40]
    ldp x0, x1, [sp, #0]
    ldr x2, [sp, #16]
    ldr x3, [sp, #24]
    add x4, sp, #40
    add x5, sp, #48
    ldr x9, [sp, #32]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_unary_scaled_error
1:
    ldp x1, x2, [sp, #40]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    b __rt_bcmath_finish_string
__rt_bcmath_unary_scaled_error:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    b __rt_bcmath_throw

__rt_bcmath_unary:
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    stp x1, x2, [sp, #0]
    str x9, [sp, #16]
    stp xzr, xzr, [sp, #24]
    ldp x0, x1, [sp, #0]
    add x2, sp, #24
    add x3, sp, #32
    ldr x9, [sp, #16]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_unary_error
1:
    ldp x1, x2, [sp, #24]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    b __rt_bcmath_finish_string
__rt_bcmath_unary_error:
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    b __rt_bcmath_throw

__rt_bcmath_round:
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    stp x1, x2, [sp, #0]
    str x3, [sp, #16]
    str x4, [sp, #24]
    str x9, [sp, #32]
    stp xzr, xzr, [sp, #40]
    ldp x0, x1, [sp, #0]
    ldr x2, [sp, #16]
    ldr x3, [sp, #24]
    add x4, sp, #40
    add x5, sp, #48
    ldr x9, [sp, #32]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_round_error
1:
    ldp x1, x2, [sp, #40]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    b __rt_bcmath_finish_string
__rt_bcmath_round_error:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    b __rt_bcmath_throw

__rt_bcmath_comp:
    sub sp, sp, #80
    stp x29, x30, [sp, #64]
    add x29, sp, #64
    stp x1, x2, [sp, #0]
    stp x3, x4, [sp, #16]
    str x5, [sp, #32]
    str x6, [sp, #40]
    str x9, [sp, #48]
    str wzr, [sp, #56]
    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldr x4, [sp, #32]
    ldr x5, [sp, #40]
    add x6, sp, #56
    ldr x9, [sp, #48]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_comp_error
1:
    ldrsw x10, [sp, #56]
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    mov x0, x10
    ret
__rt_bcmath_comp_error:
    ldp x29, x30, [sp, #64]
    add sp, sp, #80
    b __rt_bcmath_throw

__rt_bcmath_scale_get:
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    str x9, [sp, #8]
    str wzr, [sp, #0]
    mov x0, sp
    blr x9
    cbz x0, 1f
    b __rt_bcmath_scale_get_error
1:
    ldrsw x10, [sp, #0]
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    mov x0, x10
    ret
__rt_bcmath_scale_get_error:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    b __rt_bcmath_throw

__rt_bcmath_scale_set:
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    str x9, [sp, #8]
    str wzr, [sp, #0]
    add x1, sp, #0
    blr x9
    cbz x0, 1f
    b __rt_bcmath_scale_set_error
1:
    ldrsw x10, [sp, #0]
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    mov x0, x10
    ret
__rt_bcmath_scale_set_error:
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    b __rt_bcmath_throw

__rt_bcmath_powmod:
    sub sp, sp, #144
    stp x29, x30, [sp, #128]
    add x29, sp, #128
    stp x1, x2, [sp, #16]
    stp x3, x4, [sp, #32]
    stp x5, x6, [sp, #48]
    str x7, [sp, #64]
    str x8, [sp, #72]
    str x9, [sp, #80]
    stp xzr, xzr, [sp, #88]
    ldp x0, x1, [sp, #16]
    ldp x2, x3, [sp, #32]
    ldp x4, x5, [sp, #48]
    ldr x6, [sp, #64]
    ldr x7, [sp, #72]
    add x10, sp, #88
    str x10, [sp, #0]
    add x10, sp, #96
    str x10, [sp, #8]
    ldr x9, [sp, #80]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_powmod_error
1:
    ldp x1, x2, [sp, #88]
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    b __rt_bcmath_finish_string
__rt_bcmath_powmod_error:
    ldp x29, x30, [sp, #128]
    add sp, sp, #144
    b __rt_bcmath_throw

__rt_bcmath_divmod:
    sub sp, sp, #160
    stp x29, x30, [sp, #144]
    add x29, sp, #144
    stp x1, x2, [sp, #16]
    stp x3, x4, [sp, #32]
    str x5, [sp, #48]
    str x6, [sp, #56]
    str x9, [sp, #64]
    stp xzr, xzr, [sp, #72]
    stp xzr, xzr, [sp, #88]
    ldp x0, x1, [sp, #16]
    ldp x2, x3, [sp, #32]
    ldr x4, [sp, #48]
    ldr x5, [sp, #56]
    add x6, sp, #72
    add x7, sp, #80
    add x10, sp, #88
    str x10, [sp, #0]
    add x10, sp, #96
    str x10, [sp, #8]
    ldr x9, [sp, #64]
    blr x9
    cbz x0, 1f
    b __rt_bcmath_divmod_error
1:
    mov x0, #2
    mov x1, #16
    bl __rt_array_new
    str x0, [sp, #104]
    ldp x1, x2, [sp, #72]
    bl __rt_array_push_str
    str x0, [sp, #104]
    ldp x1, x2, [sp, #88]
    bl __rt_array_push_str
    str x0, [sp, #104]
    ldp x0, x1, [sp, #72]
    bl __rt_bcmath_call_free
    ldp x0, x1, [sp, #88]
    bl __rt_bcmath_call_free
    ldr x10, [sp, #104]
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    mov x0, x10
    ret
__rt_bcmath_divmod_error:
    ldp x29, x30, [sp, #144]
    add sp, sp, #160
    b __rt_bcmath_throw

__rt_bcmath_finish_string:
    sub sp, sp, #64
    stp x29, x30, [sp, #48]
    add x29, sp, #48
    stp x1, x2, [sp, #0]
    bl __rt_str_persist
    stp x1, x2, [sp, #16]
    ldp x0, x1, [sp, #0]
    bl __rt_bcmath_call_free
    ldp x1, x2, [sp, #16]
    ldp x29, x30, [sp, #48]
    add sp, sp, #64
    ret

__rt_bcmath_binary:
    push rbp
    mov rbp, rsp
    sub rsp, 96
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], r8
    mov QWORD PTR [rbp - 48], r9
    mov QWORD PTR [rbp - 56], rcx
    mov QWORD PTR [rbp - 64], 0
    mov QWORD PTR [rbp - 72], 0
    lea r10, [rbp - 64]
    mov QWORD PTR [rsp], r10
    lea r10, [rbp - 72]
    mov QWORD PTR [rsp + 8], r10
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    mov r8, QWORD PTR [rbp - 40]
    mov r9, QWORD PTR [rbp - 48]
    mov r11, QWORD PTR [rbp - 56]
    call r11
    test rax, rax
    jnz __rt_bcmath_binary_error_x86_64
    mov rax, QWORD PTR [rbp - 64]
    mov rdx, QWORD PTR [rbp - 72]
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_finish_string
__rt_bcmath_binary_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_unary_scaled:
    push rbp
    mov rbp, rsp
    sub rsp, 80
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], rcx
    mov QWORD PTR [rbp - 48], 0
    mov QWORD PTR [rbp - 56], 0
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    lea r8, [rbp - 48]
    lea r9, [rbp - 56]
    mov r11, QWORD PTR [rbp - 40]
    call r11
    test rax, rax
    jnz __rt_bcmath_unary_scaled_error_x86_64
    mov rax, QWORD PTR [rbp - 48]
    mov rdx, QWORD PTR [rbp - 56]
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_finish_string
__rt_bcmath_unary_scaled_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_unary:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rcx
    mov QWORD PTR [rbp - 32], 0
    mov QWORD PTR [rbp - 40], 0
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    lea rdx, [rbp - 32]
    lea rcx, [rbp - 40]
    mov r11, QWORD PTR [rbp - 24]
    call r11
    test rax, rax
    jnz __rt_bcmath_unary_error_x86_64
    mov rax, QWORD PTR [rbp - 32]
    mov rdx, QWORD PTR [rbp - 40]
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_finish_string
__rt_bcmath_unary_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_round:
    push rbp
    mov rbp, rsp
    sub rsp, 80
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], rcx
    mov QWORD PTR [rbp - 48], 0
    mov QWORD PTR [rbp - 56], 0
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    lea r8, [rbp - 48]
    lea r9, [rbp - 56]
    mov r11, QWORD PTR [rbp - 40]
    call r11
    test rax, rax
    jnz __rt_bcmath_round_error_x86_64
    mov rax, QWORD PTR [rbp - 48]
    mov rdx, QWORD PTR [rbp - 56]
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_finish_string
__rt_bcmath_round_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_comp:
    push rbp
    mov rbp, rsp
    sub rsp, 80
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], r8
    mov QWORD PTR [rbp - 48], r9
    mov QWORD PTR [rbp - 56], rcx
    mov DWORD PTR [rbp - 64], 0
    lea r10, [rbp - 64]
    mov QWORD PTR [rsp], r10
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    mov r8, QWORD PTR [rbp - 40]
    mov r9, QWORD PTR [rbp - 48]
    mov r11, QWORD PTR [rbp - 56]
    call r11
    test rax, rax
    jnz __rt_bcmath_comp_error_x86_64
    movsxd r10, DWORD PTR [rbp - 64]
    mov rsp, rbp
    pop rbp
    mov rax, r10
    ret
__rt_bcmath_comp_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_scale_get:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov QWORD PTR [rbp - 16], rcx
    mov DWORD PTR [rbp - 8], 0
    lea rdi, [rbp - 8]
    call rcx
    test rax, rax
    jnz __rt_bcmath_scale_get_error_x86_64
    movsxd r10, DWORD PTR [rbp - 8]
    mov rsp, rbp
    pop rbp
    mov rax, r10
    ret
__rt_bcmath_scale_get_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_scale_set:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rcx
    mov DWORD PTR [rbp - 24], 0
    mov rdi, QWORD PTR [rbp - 8]
    lea rsi, [rbp - 24]
    call rcx
    test rax, rax
    jnz __rt_bcmath_scale_set_error_x86_64
    movsxd r10, DWORD PTR [rbp - 24]
    mov rsp, rbp
    pop rbp
    mov rax, r10
    ret
__rt_bcmath_scale_set_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_powmod:
    push rbp
    mov rbp, rsp
    sub rsp, 128
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], r8
    mov QWORD PTR [rbp - 48], r9
    mov QWORD PTR [rbp - 56], r10
    mov QWORD PTR [rbp - 64], r11
    mov QWORD PTR [rbp - 72], rcx
    mov QWORD PTR [rbp - 80], 0
    mov QWORD PTR [rbp - 88], 0
    mov r10, QWORD PTR [rbp - 56]
    mov QWORD PTR [rsp], r10
    mov r10, QWORD PTR [rbp - 64]
    mov QWORD PTR [rsp + 8], r10
    lea r10, [rbp - 80]
    mov QWORD PTR [rsp + 16], r10
    lea r10, [rbp - 88]
    mov QWORD PTR [rsp + 24], r10
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    mov r8, QWORD PTR [rbp - 40]
    mov r9, QWORD PTR [rbp - 48]
    mov r11, QWORD PTR [rbp - 72]
    call r11
    test rax, rax
    jnz __rt_bcmath_powmod_error_x86_64
    mov rax, QWORD PTR [rbp - 80]
    mov rdx, QWORD PTR [rbp - 88]
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_finish_string
__rt_bcmath_powmod_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_divmod:
    push rbp
    mov rbp, rsp
    sub rsp, 144
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    mov QWORD PTR [rbp - 24], rdi
    mov QWORD PTR [rbp - 32], rsi
    mov QWORD PTR [rbp - 40], r8
    mov QWORD PTR [rbp - 48], r9
    mov QWORD PTR [rbp - 56], rcx
    mov QWORD PTR [rbp - 64], 0
    mov QWORD PTR [rbp - 72], 0
    mov QWORD PTR [rbp - 80], 0
    mov QWORD PTR [rbp - 88], 0
    lea r10, [rbp - 64]
    mov QWORD PTR [rsp], r10
    lea r10, [rbp - 72]
    mov QWORD PTR [rsp + 8], r10
    lea r10, [rbp - 80]
    mov QWORD PTR [rsp + 16], r10
    lea r10, [rbp - 88]
    mov QWORD PTR [rsp + 24], r10
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    mov rdx, QWORD PTR [rbp - 24]
    mov rcx, QWORD PTR [rbp - 32]
    mov r8, QWORD PTR [rbp - 40]
    mov r9, QWORD PTR [rbp - 48]
    mov r11, QWORD PTR [rbp - 56]
    call r11
    test rax, rax
    jnz __rt_bcmath_divmod_error_x86_64
    mov rdi, 2
    mov rsi, 16
    call __rt_array_new
    mov QWORD PTR [rbp - 96], rax
    mov rdi, rax
    mov rsi, QWORD PTR [rbp - 64]
    mov rdx, QWORD PTR [rbp - 72]
    call __rt_array_push_str
    mov QWORD PTR [rbp - 96], rax
    mov rdi, rax
    mov rsi, QWORD PTR [rbp - 80]
    mov rdx, QWORD PTR [rbp - 88]
    call __rt_array_push_str
    mov QWORD PTR [rbp - 96], rax
    mov rdi, QWORD PTR [rbp - 64]
    mov rsi, QWORD PTR [rbp - 72]
    call __rt_bcmath_call_free
    mov rdi, QWORD PTR [rbp - 80]
    mov rsi, QWORD PTR [rbp - 88]
    call __rt_bcmath_call_free
    mov r10, QWORD PTR [rbp - 96]
    mov rsp, rbp
    pop rbp
    mov rax, r10
    ret
__rt_bcmath_divmod_error_x86_64:
    mov rsp, rbp
    pop rbp
    jmp __rt_bcmath_throw

__rt_bcmath_finish_string:
    push rbp
    mov rbp, rsp
    sub rsp, 48
    mov QWORD PTR [rbp - 8], rax
    mov QWORD PTR [rbp - 16], rdx
    call __rt_str_persist
    mov QWORD PTR [rbp - 24], rax
    mov QWORD PTR [rbp - 32], rdx
    mov rdi, QWORD PTR [rbp - 8]
    mov rsi, QWORD PTR [rbp - 16]
    call __rt_bcmath_call_free
    mov rax, QWORD PTR [rbp - 24]
    mov rdx, QWORD PTR [rbp - 32]
    mov rsp, rbp
    pop rbp
    ret

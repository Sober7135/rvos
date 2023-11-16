    .altmacro
    .macro SAVE n
        sd x\n, \n * 8(sp)
    .endm
    .macro LOAD n
        ld x\n, \n * 8(sp)
    .endm

    .section .text
    .global __alltraps
    .global __restore
    .align 2
__alltraps:
    #! Attention! The trap context is always stored in the kernel stack.
    # swap kernal stack pointer (originally in sscratch) and user stack pointer (originally in sp)
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    addi sp, sp, -34 * 8
    SAVE 1 # x1 is ra
    # skip x2(sp), stack pointer, store it later
    SAVE 3 # x3 is gp (global pointer)
    # skip x4(tp), thread pointer, user will not use it
    .set n, 5
    .rept 27
        SAVE %n
        .set n, n + 1
    .endr
    # store user stack pointer (current is in sscratch) into stack
    csrr t0, sscratch
    csrr t1, sstatus
    csrr t2, sepc
    sd t0, 2*8(sp)
    sd t1, 32*8(sp)
    sd t2, 33*8(sp)
    # trap_handler arguments 
    mv a0, sp
    call trap_handler

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0 # now sp point to trap context which is pushed by __alltraps or manually by app_init_context, btw sp is pointing to kernel stack
    # start restore 
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD %n
        .set n, n + 1
    .endr
    # finishing restore general register
    addi sp, sp, 34 * 8 # release trap context, the pointer is now at top of kernel stack
    csrrw sp, sscratch, sp
    sret
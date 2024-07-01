    .altmacro
    .macro SAVE n
        sd x\n, \n * 8(sp)
    .endm
    .macro LOAD n
        ld x\n, \n * 8(sp)
    .endm

    .section .text.trampoline
    .global __alltraps
    .global __restore
    .align 2
__alltraps:
    #! Attention! The trap context is always stored in the kernel stack.
    # sp point to the trap context which is stored in user space..
    # swap user space pointer (originally in sscratch) and user stack pointer (originally in sp)
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # In ch4, we allocate a page frame for storing trap context, and the sp point to the bottom of the stack, so we don't need to minus sp
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
    # load kernel satp
    ld t0, 34*8(sp) 
    # load trap_handler address
    ld t1, 36*8(sp)
    # load kernel sp
    ld sp, 35*8(sp)
    # switch kernel space
    csrw satp, t0
    sfence.vma
    // Why not straightly `call trap_handler` 
    // TLDR, because we use the virtual address
    // AFAIK, `call` instruction calculate the address between current place and trap_handler when linking, and then add it to pc
    // But we mapped the __alltraps into the trampoline, so current, the pc point to the virtual address...
    // > The nature of the problem can be summarized as follows: the virtual address where a jump instruction is actually executed is different from the address where this instruction was set when performing backend code generation and linking in the compiler/assembler/linker, producing the final machine code.
    jr t1

__restore:
    # a0: *TrapContext in user space, a1: user satp
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # start restore 
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    # ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # csrw sscratch, t2
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD %n
        .set n, n + 1
    .endr
    # let sp point to user stack
    ld sp, 2*8(sp)
    sret
















# .altmacro
# .macro SAVE_GP n
#     sd x\n, \n*8(sp)
# .endm
# .macro LOAD_GP n
#     ld x\n, \n*8(sp)
# .endm
#     .section .text.trampoline
#     .globl __alltraps
#     .globl __restore
#     .align 2
# __alltraps:
#     csrrw sp, sscratch, sp
#     # now sp->*TrapContext in user space, sscratch->user stack
#     # save other general purpose registers
#     sd x1, 1*8(sp)
#     # skip sp(x2), we will save it later
#     sd x3, 3*8(sp)
#     # skip tp(x4), application does not use it
#     # save x5~x31
#     .set n, 5
#     .rept 27
#         SAVE_GP %n
#         .set n, n+1
#     .endr
#     # we can use t0/t1/t2 freely, because they have been saved in TrapContext
#     csrr t0, sstatus
#     csrr t1, sepc
#     sd t0, 32*8(sp)
#     sd t1, 33*8(sp)
#     # read user stack from sscratch and save it in TrapContext
#     csrr t2, sscratch
#     sd t2, 2*8(sp)
#     # load kernel_satp into t0
#     ld t0, 34*8(sp)
#     # load trap_handler into t1
#     ld t1, 36*8(sp)
#     # move to kernel_sp
#     ld sp, 35*8(sp)
#     # switch to kernel space
#     csrw satp, t0
#     sfence.vma
#     # jump to trap_handler
#     jr t1

# __restore:
#     # a0: *TrapContext in user space(Constant); a1: user space token
#     # switch to user space
#     csrw satp, a1
#     sfence.vma
#     csrw sscratch, a0
#     mv sp, a0
#     # now sp points to TrapContext in user space, start restoring based on it
#     # restore sstatus/sepc
#     ld t0, 32*8(sp)
#     ld t1, 33*8(sp)
#     csrw sstatus, t0
#     csrw sepc, t1
#     # restore general purpose registers except x0/sp/tp
#     ld x1, 1*8(sp)
#     ld x3, 3*8(sp)
#     .set n, 5
#     .rept 27
#         LOAD_GP %n
#         .set n, n+1
#     .endr
#     # back to user stack
#     ld sp, 2*8(sp)
#     sret

    .altmacro
    .macro SAVE_SR n
        sd s\n, (\n+2) * 8(a0)
    .endm
    .macro LOAD_SR n
        ld s\n, (\n+2) * 8(a1)
    .endm

    .section .text
    .global __switch    
    .align 2
__switch:
    # a0  current_task_ctxt_ptr: *mut TaskContext, 
    # a1  next_task_ctxt_ptr: *const TaskContext
    # TaskContext 
    #     ra: usize,
    #     sp: usize,
    #     s :[usize; 12],
    sd ra, 0(a0)
    sd sp, 8(a0)
    .set n, 0
    .rept 12
        SAVE_SR %n
        .set n, n+1
    .endr
    # save current context to *current_task_ctxt_ptr
    # start load *next_task_ctxt_ptr into current context
    ld ra, 0(a1)
    ld sp, 8(a1)
    .set n, 0
    .rept 12
        LOAD_SR %n
        .set n, n+1
    .endr
    ret

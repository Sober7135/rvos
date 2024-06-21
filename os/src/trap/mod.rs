mod context;

use crate::config::TRAP_CONTEXT;
use crate::process::processor::{get_current_trap_context, get_current_user_token, schedule};
use crate::process::{mark_current_exit, mark_current_suspend};
use crate::timer::set_next_trigger;
use crate::{config::TRAMPOLINE, syscall::syscall};
use core::arch::{asm, global_asm};
use log::error;
use riscv::register::sepc;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie,
    sstatus::Sstatus,
    stval, stvec,
};

pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry()
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer() }
}

// TODO
#[no_mangle] // avoid mangle, the assembly inside "trap.S" can call trap_handler
pub fn trap_handler() {
    set_kernel_trap_entry();
    let context = get_current_trap_context();
    let scause = scause::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            mark_current_suspend();
            schedule();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("Illegal instruction");
            mark_current_exit(-1);
            schedule();
            unreachable!("{}:{}", file!(), line!());
        }
        Trap::Exception(Exception::UserEnvCall) => {
            context.sepc += 4;
            context.x[10] =
                syscall(context.x[17], [context.x[10], context.x[11], context.x[12]]) as usize;
        }
        Trap::Exception(Exception::LoadPageFault) => {
            error!("Load page fault");
            mark_current_exit(-1);
            schedule();
            unreachable!("{}:{}", file!(), line!());
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!(
                "Store page fault. sepc=0x{:x}, stval=0x{:x}",
                sepc::read(),
                stval::read()
            );
            mark_current_exit(-1);
            schedule();
            unreachable!("{}:{}", file!(), line!());
        }
        Trap::Exception(Exception::StoreFault) => {
            error!("Store fault");
            mark_current_exit(-1);
            schedule();
            unreachable!("{}:{}", file!(), line!());
        }
        _ => {
            error!(
                "Unsupported Trap: cause = {:?}, stval = {:#x}\ncontext = {:?}",
                scause.cause(),
                stval::read(),
                context
            );
            mark_current_exit(-1);
            schedule();
            unreachable!("{}:{}", file!(), column!());
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    extern "C" {
        // strampoline == TRAMPOLINE
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = get_current_user_token();
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            // TODO ????
            options(noreturn)
        )
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    }
}

fn set_kernel_trap_entry() {
    unsafe { stvec::write(trap_from_kernel as usize, stvec::TrapMode::Direct) }
}

#[no_mangle]
fn trap_from_kernel() -> ! {
    panic!("")
}

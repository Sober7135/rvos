mod context;

use crate::syscall::{syscall, Syscall};
use crate::task::*;
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use log::error;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie,
    sstatus::Sstatus,
    stval, stvec,
};

pub(crate) use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub(crate) fn init() {
    extern "C" {
        fn __alltraps();
    }

    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

pub(crate) fn enable_timer_interrupt() {
    unsafe { sie::set_stimer() }
}

#[no_mangle] // avoid mangle, the assembly inside "trap.S" can call trap_handler
pub(crate) fn trap_handler(ctxt: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            mark_current_suspend();
            run_next_task();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("Illegal instruction");
            mark_current_exited();
            run_next_task();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            // trace!("cause = Exception::UserEnvCall");
            // info!("x17 = {} x10 = {} x11 = {:#x} x12 = {}", ctxt.x[17], ctxt.x[10], ctxt.x[11], ctxt.x[12]);
            ctxt.sepc += 4;
            ctxt.x[10] = syscall(
                Syscall::from(ctxt.x[17]),
                [ctxt.x[10], ctxt.x[11], ctxt.x[12]],
            ) as usize;
        }
        Trap::Exception(Exception::LoadPageFault) => {
            error!("Load page fault");
            mark_current_exited();
            run_next_task();
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!("Store page fault");
            mark_current_exited();
            run_next_task();
        }
        Trap::Exception(Exception::StoreFault) => {
            error!("Store fault");
            mark_current_exited();
            run_next_task();
        }
        _ => {
            error!(
                "Unsupported Trap: cause = {:?}, stval = {:#x}\ncontext = {:?}",
                scause.cause(),
                stval::read(),
                ctxt
            );
            mark_current_exited();
            run_next_task();
        }
    }
    ctxt
}

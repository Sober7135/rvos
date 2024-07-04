mod context;

use crate::{batch::run_next_app, syscall::syscall};

use core::arch::global_asm;
use log::{debug, error};
use riscv::register::{
    scause::{self, Exception, Trap},
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
    debug!("[kernel] finish initializing the trap handler");
}

#[no_mangle] // avoid mangle, the assembly inside "trap.S" can call trap_handler
pub(crate) fn trap_handler(ctxt: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("Illegal instruction\n");
            run_next_app();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            ctxt.sepc += 4;
            ctxt.x[10] = syscall(ctxt.x[17], [ctxt.x[10], ctxt.x[11], ctxt.x[12]]) as usize;
        }
        Trap::Exception(Exception::LoadPageFault) => {
            error!("Load page fault\n");
            run_next_app();
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!("Store page fault\n");
            run_next_app();
        }
        Trap::Exception(Exception::StoreFault) => {
            error!("Store fault\n");
            run_next_app();
        }
        _ => {
            error!(
                "Unsupported Trap: cause = {:?}, stval = {:#x}\ncontext = {:?}\n",
                scause.cause(),
                stval::read(),
                ctxt
            );

            run_next_app()
        }
    }
    ctxt
}

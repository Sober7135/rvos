use crate::mm::KERNEL_SPACE;

use super::*;
use riscv::register::sstatus::{self, SPP};

#[derive(Debug)]
#[repr(C)] //? why is this necessary
pub(crate) struct TrapContext {
    /// general register
    pub(crate) x: [usize; 32],
    /// supervisor status
    pub(crate) sstatus: Sstatus,
    /// supervisor exception program counter
    pub(crate) sepc: usize,
    // only used in __alltraps
    pub(crate) kernel_satp: usize,
    // only used in __alltraps
    pub(crate) kernel_sp: usize,
    // only used in __alltraps
    pub(crate) trap_handler: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    // kernel_sp for kernel stack's sp, for storing trap context, etc.
    // current each app have a kernel stack
    // sp for user stack's sp, user stack is in the user address space, for app runtime stack
    pub(crate) fn app_init_context(entry: usize, kernel_sp: usize, user_sp: usize) -> Self {
        let mut _sstatus = sstatus::read();
        _sstatus.set_spp(SPP::User);
        let mut ctxt = TrapContext {
            x: [0; 32],
            sstatus: _sstatus,
            sepc: entry,
            kernel_satp: KERNEL_SPACE.lock().get_token(),
            kernel_sp,
            trap_handler: trap_handler as usize,
        };
        ctxt.set_sp(user_sp);
        ctxt
    }
}

use crate::mm::kernel_space::get_kernel_token;

use super::*;
use riscv::register::sstatus::{self, SPP};

#[derive(Debug)]
#[repr(C)] //? why is this necessary
pub struct TrapContext {
    /// general register
    pub x: [usize; 32],
    /// supervisor status
    pub sstatus: Sstatus,
    /// supervisor exception program counter
    pub sepc: usize,
    // only used in __alltraps
    pub kernel_satp: usize,
    // only used in __alltraps
    pub kernel_sp: usize,
    // only used in __alltraps
    pub trap_handler: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    // kernel_sp for kernel stack's sp, for storing trap context, etc.
    // current each app have a kernel stack
    // sp for user stack's sp, user stack is in the user address space, for app runtime stack
    pub fn app_init_context(entry: usize, kernel_sp: usize, user_sp: usize) -> Self {
        let mut _sstatus = sstatus::read();
        _sstatus.set_spp(SPP::User);
        let mut ctxt = TrapContext {
            x: [0; 32],
            sstatus: _sstatus,
            sepc: entry,
            kernel_satp: get_kernel_token(),
            kernel_sp,
            trap_handler: trap_handler as usize,
        };
        ctxt.set_sp(user_sp);
        ctxt
    }
}

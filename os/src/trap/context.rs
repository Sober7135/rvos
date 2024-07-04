use riscv::register::sstatus::{self, SPP};
use super::*;

#[derive(Debug)]
#[repr(C)] //? why is this necessary
pub(crate) struct TrapContext {
    /// general register
    pub(crate) x: [usize; 32],
    /// supervisor status
    pub(crate) sstatus: Sstatus,
    /// supervisor exception program counter
    pub(crate) sepc: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub(crate) fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut _sstatus = sstatus::read();
        _sstatus.set_spp(SPP::User);
        let mut ctxt = TrapContext {
            x: [0; 32],
            sstatus: _sstatus,
            sepc: entry,
        };
        ctxt.set_sp(sp);
        ctxt
    }
}

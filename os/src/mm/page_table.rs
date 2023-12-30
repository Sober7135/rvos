use super::{
    address::{PhysicalPageNumber, VirtualAddr, VirtualPageNumber},
    frame_allocator::{frame_alloca, FrameTracker},
};
use crate::mm::address::PhysicalAddr;
use alloc::vec::*;
use bitflags::*;
use log::info;

bitflags! {
    pub(crate) struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct PageTableEntry {
    pub(crate) bits: usize,
}

#[derive(Debug, Default)]
pub(crate) struct PageTable {
    pub(crate) root: PhysicalPageNumber,
    pub(crate) frames: Vec<FrameTracker>,
}

impl PageTable {
    pub(crate) fn new() -> Self {
        let frame = frame_alloca().unwrap();
        Self {
            root: frame.inner,
            frames: vec![frame],
        }
    }

    fn find_pte_create(&mut self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, offset) in vpn.get_indexes().iter().enumerate() {
            let pte = ppn.get_pte(*offset);
            if i == 2 {
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                let alloca = frame_alloca().unwrap();
                *pte = PageTableEntry::new(alloca.inner, PTEFlags::V);
                self.frames.push(alloca);
            }
            ppn = pte.get_ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, offset) in vpn.get_indexes().iter().enumerate() {
            let pte = ppn.get_pte(*offset);
            if i == 2 {
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                return None;
            }
            ppn = pte.get_ppn();
        }
        result
    }

    pub(crate) fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags) {
        if let Some(pte) = self.find_pte_create(vpn) {
            *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        }
    }

    pub(crate) fn translate(&self, va: VirtualAddr) -> PhysicalAddr {
        let pa: PhysicalAddr = self.find_pte(va.into()).unwrap().get_ppn().into();
        (pa.0 + va.get_offset()).into()
    }
}

impl PageTableEntry {
    pub(crate) fn new(ppn: PhysicalPageNumber, pte_flag: PTEFlags) -> Self {
        Self {
            bits: (ppn.0 << 10) | pte_flag.bits() as usize,
        }
    }

    pub(crate) fn empty() -> Self {
        Self { bits: 0 }
    }

    pub(crate) fn get_ppn(&self) -> PhysicalPageNumber {
        ((self.bits >> 10) & ((1 << 44) - 1)).into()
    }

    pub(crate) fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub(crate) fn is_valid(&self) -> bool {
        (self.bits & 0b1) == 0b1
    }
}

#[allow(unused)]
pub(crate) fn translate_test() {
    let mut pt = PageTable::new();
    let vpn = 0x100.into();
    pt.map(vpn, 200.into(), PTEFlags::empty());
    let translated = pt.translate(0x100003.into());
    assert_eq!(
        translated,
        PhysicalAddr::from(PhysicalAddr::from(PhysicalPageNumber::from(200)).0 + 3)
    );
    info!("translate_test PASSED!");
}

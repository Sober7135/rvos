use super::{
    address::{PhysicalPageNumber, VirtualPageNumber},
    frame_allocator::{frame_alloca, FrameTracker},
};
use crate::{
    mm::{
        address::{PhysicalAddr, StepByOne},
        VirtualAddr,
    },
    task::get_current_user_token,
};
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

    /// Don't worry about `Vec::new`, it's just temporarily used and the frames will never be pushed..
    pub(crate) fn from_token(token: usize) -> Self {
        Self {
            root: PhysicalPageNumber(token & ((1usize << 44) - 1)),
            frames: Vec::new(),
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
        let pte = self.find_pte_create(vpn).unwrap();
        // the pte will be allocated must be a invaild..
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub(crate) fn unmap(&mut self, vpn: VirtualPageNumber) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(
            pte.is_valid(),
            "vpn {:?} is never mapped before unmapping",
            vpn
        );
        *pte = PageTableEntry::empty();
    }

    pub(crate) fn translate(&self, vpn: VirtualPageNumber) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub(crate) fn get_token(&self) -> usize {
        // 8 for sv39 mode
        8usize << 60 | self.root.0
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
        // (self.bits & 0b1) == 0b1
        self.flags().contains(PTEFlags::V)
    }

    pub(crate) fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub(crate) fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub(crate) fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

#[allow(unused)]
pub(crate) fn translate_test() {
    let mut pt = PageTable::new();
    let vpn = 0x100.into();
    pt.map(vpn, 200.into(), PTEFlags::empty());
    let translated =
        PhysicalAddr::from(PhysicalAddr::from(pt.translate(vpn).unwrap().get_ppn()).0 + 3);
    assert_eq!(
        translated,
        PhysicalAddr::from(PhysicalAddr::from(PhysicalPageNumber::from(200)).0 + 3)
    );
    info!("translate_test PASSED!");
}

pub(crate) fn copy_from_user(ptr: *const u8, len: usize) -> Vec<&'static [u8]> {
    let page_table = PageTable::from_token(get_current_user_token());
    let mut v = Vec::new();
    let mut start = ptr as usize;
    let end = start + len;
    while start < end {
        let start_va = VirtualAddr::from(start);
        let mut vpn = start_va.floor();
        let addr = PhysicalAddr::from(page_table.translate(vpn).unwrap().get_ppn()).0
            + start_va.get_offset();
        vpn.step();
        // The end_va may bigger than end
        let mut end_va: VirtualAddr = vpn.into();
        end_va = end_va.min(VirtualAddr::from(end));

        v.push(unsafe {
            core::slice::from_raw_parts(
                addr as *const u8,
                usize::from(end_va) - usize::from(start_va),
            )
        });

        start = end_va.into();
    }
    v
}

use super::{
    address::{PhysicalPageNumber, VirtualPageNumber},
    frame_allocator::{frame_alloc, FrameTracker},
};
use crate::{
    config::{PAGE_SIZE, SV39_PPN_WIDTH},
    mm::{
        address::{PhysicalAddr, StepByOne},
        VirtualAddr,
    },
    process::processor::get_current_user_token,
};
use alloc::{
    string::{String, ToString},
    vec::*,
};
use bitflags::*;
use log::info;

bitflags! {
    pub struct PTEFlags: u8 {
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
pub struct PageTableEntry {
    pub bits: u64,
}

#[derive(Debug, Default)]
pub struct PageTable {
    pub ppn: PhysicalPageNumber,
    pub frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    /// Don't worry about `Vec::new`, it's just temporarily used and the frames will never be pushed..
    pub fn from_token(token: usize) -> Self {
        Self {
            ppn: PhysicalPageNumber(token & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    fn find_pte_create(&mut self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, offset) in vpn.get_indexes().iter().enumerate() {
            let pte = ppn.get_pte(*offset);
            if i == 2 {
                result = Some(pte);
                break;
            }

            if !pte.is_valid() {
                let alloca = frame_alloc().unwrap();
                *pte = PageTableEntry::new(alloca.ppn, PTEFlags::V);
                self.frames.push(alloca);
            }
            ppn = pte.get_ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtualPageNumber) -> Option<&mut PageTableEntry> {
        let mut ppn = self.ppn;
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

    pub fn map(&mut self, vpn: VirtualPageNumber, ppn: PhysicalPageNumber, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        // the pte will be allocated must be a invaild..
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtualPageNumber) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(
            pte.is_valid(),
            "vpn {:?} is never mapped before unmapping",
            vpn
        );
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&self, vpn: VirtualPageNumber) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn translate_va(&self, va: VirtualAddr) -> Option<PhysicalAddr> {
        self.translate(va.into())
            .map(|ppn| (PhysicalAddr::from(ppn.get_ppn()).0 + va.get_offset()).into())
    }

    // FIXME Assume that the str is within a page.
    pub fn translate_str(&self, ptr: *const u8) -> &str {
        let bytes = &self
            .translate(VirtualAddr::from(ptr as usize).into())
            .unwrap()
            .get_ppn()
            .get_bytes_array()[ptr as usize % PAGE_SIZE..];

        let mut len = 0;
        while unsafe { *bytes.get_unchecked(len) } != 0 {
            len += 1;
        }
        unsafe { core::str::from_utf8_unchecked(&bytes[0..len]) }
    }

    // sizeof T
    pub fn translate_ref<T>(&self, ptr: *const T) -> &'static T {
        self.translate_va(VirtualAddr::from(ptr as usize))
            .unwrap()
            .get_mut()
    }

    pub fn translate_refmut<T>(&self, ptr: *const T) -> &'static mut T {
        self.translate_va(VirtualAddr::from(ptr as usize))
            .unwrap()
            .get_mut()
    }

    pub fn get_token(&self) -> usize {
        // 8 for sv39 mode
        8usize << 60 | self.ppn.0
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhysicalPageNumber, pte_flag: PTEFlags) -> Self {
        Self {
            bits: ((ppn.0 << 10) | pte_flag.bits() as usize) as u64,
        }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn get_ppn(&self) -> PhysicalPageNumber {
        (((self.bits >> 10) & ((1 << SV39_PPN_WIDTH) - 1)) as usize).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        // (self.bits & 0b1) == 0b1
        self.flags().contains(PTEFlags::V)
    }

    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

#[allow(unused)]
pub fn translate_test() {
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

pub fn transfer_byte_buffer(ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(get_current_user_token());
    let mut v = Vec::new();
    let mut start = ptr as usize;
    let end = start + len;

    while start < end {
        let start_va = VirtualAddr::from(start);
        let mut vpn = start_va.floor();

        vpn.step();
        // The end_va may bigger than end
        let mut end_va: VirtualAddr = vpn.into();
        end_va = end_va.min(VirtualAddr::from(end));

        v.push(
            &mut page_table
                .translate(start_va.floor())
                .unwrap()
                .get_ppn()
                .get_bytes_array()[start_va.get_offset()..end_va.get_offset()],
        );
        start = end_va.into();
    }
    v
}

#[allow(unused)]
pub fn translate_str(token: usize, ptr: *const u8) -> String {
    let pgtbl = PageTable::from_token(token);
    pgtbl.translate_str(ptr).to_string()
}

#[allow(unused)]
/// size_of T must be power of 2 and less than 4096
pub fn translate_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let pgtbl = PageTable::from_token(token);
    pgtbl.translate_ref(ptr)
}

/// size_of T must be power of 2 and less than 4096
pub fn translate_refmut<T>(token: usize, ptr: *const T) -> &'static mut T {
    let pgtbl = PageTable::from_token(token);
    pgtbl.translate_refmut(ptr)
}

use core::fmt::Debug;

use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS, PA_WIDTH_SV39, PPN_WIDTH_SV39};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct PhysicalAddr(pub(crate) usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct VirtualAddr(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct PhysicalPageNumber(pub(crate) usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct VirtualPageNumber(pub(crate) usize);

impl From<usize> for PhysicalAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<PhysicalAddr> for usize {
    fn from(value: PhysicalAddr) -> Self {
        value.0
    }
}

impl PhysicalAddr {
    pub(crate) fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }

    pub(crate) fn floor(&self) -> PhysicalPageNumber {
        PhysicalPageNumber(self.0 / PAGE_SIZE)
    }

    pub(crate) fn ceil(&self) -> PhysicalPageNumber {
        PhysicalPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl Debug for PhysicalAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PhysicalAddr: {:#x}", self.0))
    }
}

impl Debug for VirtualAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VirtualAddr: {:#x}", self.0))
    }
}

impl Debug for VirtualPageNumber {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VirtualPageNumber: {:#x}", self.0))
    }
}

impl From<PhysicalAddr> for PhysicalPageNumber {
    /// For misalignment, the physical address cannot be converted to a physical page number through From/Into,
    /// but needs to be converted by its own floor or ceil method to perform rounding down or rounding up conversion.
    fn from(value: PhysicalAddr) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.floor()
    }
}

impl From<usize> for PhysicalPageNumber {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysicalPageNumber> for usize {
    fn from(value: PhysicalPageNumber) -> Self {
        value.0
    }
}

impl From<PhysicalPageNumber> for PhysicalAddr {
    fn from(value: PhysicalPageNumber) -> Self {
        PhysicalAddr(value.0 << PAGE_SIZE_BITS)
    }
}

impl PhysicalPageNumber {
    /// Get page table entry at `pte.ppn + va.vpn[i]`
    pub(crate) fn get_pte(&self, offset: usize) -> &'static mut PageTableEntry {
        let pa: PhysicalAddr = (PhysicalAddr::from(*self).0 + offset).into();
        unsafe { (pa.0 as *mut PageTableEntry).as_mut().unwrap() }
    }
}

impl From<usize> for VirtualAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<VirtualAddr> for usize {
    fn from(value: VirtualAddr) -> Self {
        value.0
    }
}

impl VirtualPageNumber {
    pub(crate) fn get_indexes(&self) -> [usize; 3] {
        let mut ret = [0usize; 3];
        let mut vpns = self.0 >> PAGE_SIZE_BITS;
        for i in 0..3 {
            ret[i] = vpns & ((1 << 9) - 1);
            vpns = vpns >> 9;
        }
        ret
    }
    pub(crate) fn get_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }
}

impl From<VirtualAddr> for VirtualPageNumber {
    fn from(value: VirtualAddr) -> Self {
        Self(value.0 >> PAGE_SIZE_BITS)
    }
}

impl From<VirtualPageNumber> for VirtualAddr {
    fn from(value: VirtualPageNumber) -> Self {
        VirtualAddr(value.0 << PAGE_SIZE_BITS)
    }
}

impl VirtualAddr {
    pub(crate) fn get_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }
}

impl From<usize> for VirtualPageNumber {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtualPageNumber> for usize {
    fn from(value: VirtualPageNumber) -> Self {
        value.0
    }
}

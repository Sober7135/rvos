use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS, SV39_ADDR_WIDTH, SV39_PPN_WIDTH};
use core::fmt::Debug;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddr(pub usize);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddr(pub usize);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalPageNumber(pub usize);

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualPageNumber(pub usize);

impl From<usize> for PhysicalAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << SV39_ADDR_WIDTH) - 1))
    }
}

impl From<usize> for VirtualAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << SV39_ADDR_WIDTH) - 1))
    }
}

impl From<PhysicalAddr> for usize {
    fn from(value: PhysicalAddr) -> Self {
        value.0
    }
}

impl From<VirtualAddr> for usize {
    fn from(value: VirtualAddr) -> Self {
        value.0
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

impl Debug for PhysicalPageNumber {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PhysicalPageNumber: {:#x}", self.0))
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

impl From<VirtualAddr> for VirtualPageNumber {
    fn from(value: VirtualAddr) -> Self {
        value.floor()
    }
}

impl From<usize> for PhysicalPageNumber {
    fn from(value: usize) -> Self {
        Self(value & ((1 << SV39_PPN_WIDTH) - 1))
    }
}

impl From<usize> for VirtualPageNumber {
    fn from(value: usize) -> Self {
        Self(value & ((1 << SV39_PPN_WIDTH) - 1))
    }
}

impl From<PhysicalPageNumber> for usize {
    fn from(value: PhysicalPageNumber) -> Self {
        value.0
    }
}

impl From<VirtualPageNumber> for usize {
    fn from(value: VirtualPageNumber) -> Self {
        value.0
    }
}

impl From<PhysicalPageNumber> for PhysicalAddr {
    fn from(value: PhysicalPageNumber) -> Self {
        PhysicalAddr(value.0 << PAGE_SIZE_BITS)
    }
}

impl From<VirtualPageNumber> for VirtualAddr {
    fn from(value: VirtualPageNumber) -> Self {
        VirtualAddr(value.0 << PAGE_SIZE_BITS)
    }
}

impl PhysicalPageNumber {
    /// Get page table entry at `pte.ppn + va.vpn[i]`
    pub fn get_pte(&self, offset: usize) -> &'static mut PageTableEntry {
        let pa: PhysicalAddr =
            (PhysicalAddr::from(*self).0 + offset * core::mem::size_of::<PageTableEntry>()).into();
        unsafe { (pa.0 as *mut PageTableEntry).as_mut().unwrap() }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysicalAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysicalAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
}

impl VirtualPageNumber {
    pub fn get_indexes(&self) -> [usize; 3] {
        let mut ret = [0usize; 3];
        let mut vpns = self.0;
        for i in 0..3 {
            ret[2 - i] = vpns & ((1 << 9) - 1);
            vpns >>= 9;
        }
        ret
    }
}

impl PhysicalAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }

    pub fn floor(&self) -> PhysicalPageNumber {
        PhysicalPageNumber(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> PhysicalPageNumber {
        PhysicalPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtualAddr {
    pub fn get_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE_BITS) - 1)
    }

    pub fn floor(&self) -> VirtualPageNumber {
        VirtualPageNumber(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> VirtualPageNumber {
        VirtualPageNumber((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

pub type VPNRange = SimpleRange<VirtualPageNumber>;

pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtualPageNumber {
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl StepByOne for PhysicalPageNumber {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    l: T,
    r: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(l: T, r: T) -> Self {
        assert!(l <= r);
        Self { l, r }
    }

    pub fn get_start(&self) -> T {
        self.l
    }

    pub fn get_end(&self) -> T {
        self.r
    }
}

impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + Clone + PartialOrd + Ord + PartialEq + Eq,
{
    type IntoIter = SimpleRangeIterator<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator {
            current: self.l,
            end: self.r,
        }
    }
}

pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Clone + Copy,
{
    current: T,
    end: T,
}

impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + Clone + PartialEq + Eq + PartialOrd + Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let ret = self.current;
            self.current.step();
            Some(ret)
        }
    }
}

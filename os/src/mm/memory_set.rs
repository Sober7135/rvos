use core::arch::asm;

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use log::info;
use riscv::register::satp;
use xmas_elf::program::ProgramHeader;

use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    mm::address::{PhysicalAddr, StepByOne},
    sync::UpSafeCell,
};

use super::{
    address::{PhysicalPageNumber, VPNRange, VirtualAddr, VirtualPageNumber},
    frame_allocator::{frame_alloc, FrameTracker},
    page_table::{PTEFlags, PageTable, PageTableEntry},
};

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss(); // start of block start symbol
    fn ebss(); // end of block start symbol
    fn ekernel();
    fn strampoline();
}

#[derive(Debug)]
pub(crate) struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

#[derive(Debug)]
pub(crate) struct MapArea {
    vpn_range: VPNRange,
    date_frames: BTreeMap<VirtualPageNumber, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MapType {
    Identical,
    Framed,
}

bitflags! {
    #[derive(Debug)]
    pub(crate) struct MapPermission : u8 {
      const R = 1 << 1;
      const W = 1 << 2;
      const X = 1 << 3;
      const U = 1 << 4;
    }
}

impl MapArea {
    pub(crate) fn new(
        start_va: VirtualAddr,
        end_va: VirtualAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        Self {
            vpn_range: VPNRange::new(start_va.floor(), end_va.ceil()),
            date_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub(crate) fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    #[allow(unused)]
    pub(crate) fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    // copy data into physical frames
    pub(crate) fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert!(
            self.map_type == MapType::Framed,
            "Map type identical cannot copy data\n"
        );

        let mut cur_vpn = self.vpn_range.get_start();
        let mut len = data.len();
        let mut start = 0;
        while len != 0 {
            let dst: usize =
                PhysicalAddr::from(page_table.translate(cur_vpn).unwrap().get_ppn()).into();
            let n = len.min(PAGE_SIZE);
            let src = &data[start..start + n];

            start += PAGE_SIZE;
            len -= n;

            unsafe { core::ptr::copy(src.as_ptr(), dst as *mut u8, n) }
            cur_vpn.step();
        }
    }

    pub(crate) fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtualPageNumber) {
        // first map in data_frames
        // then map in page_table. Actually, page table's map is writing data to pte which can be translated by mmu
        let ppn: PhysicalPageNumber;
        match self.map_type {
            MapType::Identical => ppn = PhysicalPageNumber(vpn.0),
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.date_frames.insert(vpn, frame);
            }
        }

        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    #[allow(unused)]
    pub(crate) fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtualPageNumber) {
        if let MapType::Framed = self.map_type {
            self.date_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
}

impl MemorySet {
    pub(crate) fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub(crate) fn get_token(&self) -> usize {
        self.page_table.get_token()
    }

    pub(crate) fn translate(&self, vpn: VirtualPageNumber) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    pub(crate) fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data)
        }
        self.areas.push(map_area);
    }

    /// Assume that no conflicts
    pub(crate) fn insert_framed_area(
        &mut self,
        start_va: VirtualAddr,
        end_va: VirtualAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    // We must map trampoline both user space and kernel space
    // Becasue when we get into __alltraps and then we switch to kernel space,
    // the virtual address in kernel space and user space must be some, otherwise, the os will boomm!!!
    // Mention that trampoline is not collected by areas
    pub(crate) fn map_trampoline(&mut self) {
        // WHY use R | X rather than R | W
        // because code of  __alltraps and __restore are stored in trampoline
        self.page_table.map(
            VirtualAddr::from(TRAMPOLINE).into(),
            PhysicalAddr::from(strampoline as usize).into(),
            // Why not PTEFlags::U?
            // That because in __alltraps we already in S mode. Please distinguish U mode and S mode, user space and kernel space
            PTEFlags::R | PTEFlags::X,
        );
    }

    /// Map kernel
    pub(crate) fn new_kernel() -> Self {
        let mut mm_set = MemorySet::new_bare();
        mm_set.map_trampoline();
        info!(
            "[kernel] .text   : [{:#x}, {:#x})",
            stext as usize, etext as usize
        );
        info!(
            "[kernel] .rodata : [{:#x}, {:#x})",
            srodata as usize, erodata as usize
        );
        info!(
            "[kernel] .data   : [{:#x}, {:#x})",
            sdata as usize, edata as usize
        );
        info!(
            "[kernel] .bss    : [{:#x}, {:#x})",
            sbss as usize, ebss as usize
        );

        mm_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        info!("Mapped .text");

        mm_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        info!("Mapped .rodata");

        mm_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        info!("Mapped .data");

        mm_set.push(
            MapArea::new(
                (sbss as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        info!("Mapped .bss");

        mm_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::W | MapPermission::R,
            ),
            None,
        );
        info!("Mapped physical memory");
        mm_set
    }

    /// Map from elf
    pub(crate) fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut mm_set = MemorySet::new_bare();
        // Map trampoline
        mm_set.map_trampoline();
        // Map program header, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        // check magic
        assert_eq!(
            elf_header.pt1.magic,
            [0x7f, 0x45, 0x4c, 0x46],
            "invalid elf!"
        );
        let ph_count = elf_header.pt2.ph_count();

        let mut max_end_vpn = VirtualPageNumber(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtualAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtualAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let perm = get_permission(&ph);
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, perm);
                max_end_vpn = map_area.vpn_range.get_end();
                mm_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }

        // Map user stack
        let max_end_va: VirtualAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // TODO guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        mm_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // Map TrapContext
        mm_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            mm_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub(crate) fn activate(&self) {
        let _satp = self.page_table.get_token();
        unsafe {
            satp::write(_satp);
            asm!("sfence.vma");
        }
    }
}

fn get_permission(ph: &ProgramHeader) -> MapPermission {
    let ph_flags = ph.flags();
    let mut perm = MapPermission::U;
    if ph_flags.is_read() {
        perm |= MapPermission::R;
    }
    if ph_flags.is_write() {
        perm |= MapPermission::W;
    }
    if ph_flags.is_execute() {
        perm |= MapPermission::X;
    }
    perm
}

lazy_static! {
    pub(crate) static ref KERNEL_SPACE: Arc<UpSafeCell<MemorySet>> =
        Arc::new(unsafe { UpSafeCell::new(MemorySet::new_kernel()) });
}

#[allow(unused)]
pub(crate) fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mtext: VirtualAddr = (((stext as usize) + (etext as usize)) / 2).into();
    let mrodata: VirtualAddr = (((srodata as usize) + (erodata as usize)) / 2).into();
    let mdata: VirtualAddr = (((sdata as usize) + (edata as usize)) / 2).into();
    let mbss: VirtualAddr = (((sbss as usize) + (ebss as usize)) / 2).into();

    // text
    let mut pte = kernel_space.page_table.translate(mtext.into()).unwrap();
    assert!(pte.readable());
    assert!(!pte.writable());
    assert!(pte.executable());

    // rodata
    pte = kernel_space.page_table.translate(mrodata.into()).unwrap();
    assert!(pte.readable());
    assert!(!pte.writable());
    assert!(!pte.executable());

    // data
    pte = kernel_space.page_table.translate(mdata.into()).unwrap();
    assert!(pte.readable());
    assert!(pte.writable());
    assert!(!pte.executable());

    // bss
    pte = kernel_space.page_table.translate(mbss.into()).unwrap();
    assert!(pte.readable());
    assert!(pte.writable());
    assert!(!pte.executable());

    info!("remap_test PASSED");
}

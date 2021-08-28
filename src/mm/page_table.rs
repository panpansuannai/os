use super::address::*;
use super::pte_sv39::{
    PTE,
    PTE_SIZE,
    PTEFlag
};

use super::phys_frame::{
    alloc,
    dealloc,
    mark
};
use alloc::vec::Vec;
use riscv::register::satp;

#[derive(Clone)]
pub struct PhysFrameTracker(pub PhysPageNum);

impl PhysFrameTracker {
    pub fn new(map_to_kernel: bool) -> Self {
        let num = alloc().unwrap();
        if map_to_kernel {
            unsafe { 
            super::KERNEL_PAGE_TABLE.map_frame(
                VirtualPageNum(num.0), PTEFlag::R|PTEFlag::W, num);
            }
        }
        Self(num)
    }
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        let dst = unsafe {
            core::slice::from_raw_parts_mut((PhysAddr::from(self.0).0 + offset) as *mut u8, data.len())
        };
        dst.copy_from_slice(data);
    }
}
impl Default for PhysFrameTracker {
    fn default() -> Self {
        Self(PhysPageNum(0))
    }
}

#[derive(Clone, Copy)]
pub struct PageTable{
    pub root_ppn: PhysPageNum,
}

impl const Default for PageTable{
    fn default() -> Self {
        PageTable{
            root_ppn: PhysPageNum(0),
        }
    }
}

impl PageTable{
    pub fn new(read_page_table: bool, map_parent: Option<&mut PageTable>) -> Self {
        log!(debug "New Page Table");
        let ppn = alloc().unwrap();
        let mut table = PageTable {
            root_ppn: ppn,
        };
        if read_page_table {
            unsafe { 
                table.map_frame(
                    VirtualPageNum(ppn.0), PTEFlag::R, ppn); 
            };
        }
        if let Some(map_parent) = map_parent {
            unsafe {
                map_parent.map_frame(
                    VirtualPageNum(ppn.0), PTEFlag::R|PTEFlag::W, ppn);
            }
        }
        table
    }

    pub fn map(&mut self, vaddr: VirtualPageNum, flag: PTEFlag) -> Option<PhysFrameTracker>{
        let pte = self.find_pte(vaddr).unwrap() as *mut PTE;
        unsafe { 
            if !(*pte).is_valid() {
                *pte = PTE::new(PhysFrameTracker::new(true).0, flag | PTEFlag::V); 
            }else {
                *pte = PTE::new((*pte).ppn(), flag | (*pte).flags()); 
            }
        }
        //assert!(pte.is_readable(), "&pte:0x{:x}, ppn:0x{:x}, bits:{:?}",
        //    pte as *const PTE as usize, pte.ppn().0, pte.flags());
        Some(PhysFrameTracker( unsafe { *pte } .ppn()))
    }

    pub unsafe fn map_frame(&mut self,
                            vaddr: VirtualPageNum,
                            flag: PTEFlag,
                            frame: PhysPageNum) -> Option<PhysFrameTracker>{
        log!(debug "Maping frame: 0x{:x}", vaddr.0);
        let pte = self.find_pte(vaddr).unwrap();
        if !pte.is_valid() {
            *pte = PTE::new(frame, flag | PTEFlag::V);
            mark(frame);
        }
        Some(PhysFrameTracker(pte.ppn()))
    }

    // Map the physic addresses to the same addresses in virtual space
    pub fn map_on_the_area(&mut self, area: core::ops::RangeInclusive<VirtualAddr>,
                       flags: PTEFlag) {
        let start_num = area.start().floor();
        let end_num = area.end().floor();
        (start_num.0..=end_num.0).fold(0, |_, i| {
            unsafe {self.map_frame(VirtualPageNum(i),
                flags, PhysPageNum(i));
            }
            0
        });
    }

    pub fn unmap(&mut self, vaddr: VirtualPageNum) {
        let pte = self.find_pte(vaddr);
        if let Some(pte) = pte {
            pte.set_flags(PTEFlag::empty());
        }
    }

    fn map_page_table(&mut self, page_num: PhysPageNum) {
        log!(debug "Maping page table 0x{:x}", page_num.0);
        unsafe {
            self.map_frame(VirtualPageNum(page_num.0),
                PTEFlag::R|PTEFlag::W, page_num);
        }
    }

    // Alloc a physic page for the invalid pte
    fn alloc_pte(&mut self, pte: *mut PTE, flag: PTEFlag) {
        log!(debug "Alloc pte: 0x{:x}", pte as usize);
        let mut page;
        unsafe {
            if (*pte).is_valid() {
                page = (*pte).ppn();
            }else {
                page = alloc().unwrap();
            }
            *pte = PTE::new(page, flag);
        }
        unsafe {
            if super::KERNEL_PAGE_TABLE_INIT {
                super::KERNEL_PAGE_TABLE.map_page_table(page);
            } else {
                self.map_page_table(page);
            }
        }
    }

    // Find the pte at the last level associated to the page number
    pub fn find_pte(&mut self, vaddr: VirtualPageNum) -> Option<&mut PTE> {
        log!(debug "Finding pte :0x{:x}", vaddr.0);
        let mut pte_ptr = 0 as *mut PTE;
        let mut ppn = self.root_ppn;
        for i in (0..PAGE_TABLE_LEVEL).rev() {
            pte_ptr = ppn.offset(vaddr.vpn_block_sv39(i) * PTE_SIZE).0 as *mut PTE;
            let pte = unsafe { pte_ptr.as_ref().unwrap() };
            if i > 0 {
                if !pte.is_valid() {
                    self.alloc_pte(pte_ptr, PTEFlag::V);
                }
                if pte.is_leaf() {
                    // Big page size not support
                    unimplemented!("Not support big page");
                }
            }
            ppn = pte.ppn();
        }
        log!(debug "Found pte :0x{:x} -> 0x{:x}", vaddr.0, pte_ptr as usize);
        unsafe { Some(&mut *pte_ptr) } 
    }

    pub fn activate(&self) {
        unsafe {
            satp::set(satp::Mode::Sv39, 0, self.root_ppn.0);
            asm!("sfence.vma");
        }
    }
}

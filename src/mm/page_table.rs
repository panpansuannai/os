use super::address::{
    VirtualAddr,
    VirtualPageNum,
    PhysAddr,
    PhysPageNum
};
use super::pte_sv39::{
    PTE,
    PTE_SIZE,
    PTEFlag
};

use super::phys_frame::FrameAllocator;
use super::phys_frame::StackFrameAllocator;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use riscv::register::satp;
use super::phys_frame::PHYSFRAME_ALLOCATOR;


pub struct PhysFrameTracker(pub PhysPageNum);

impl PhysFrameTracker {
    pub fn new() -> Self {
        let num = PHYSFRAME_ALLOCATOR.lock().alloc().unwrap();
        num.clear();
        Self(num)
    }
}
impl Default for PhysFrameTracker {
    fn default() -> Self {
        Self(PhysPageNum(0))
    }
}

pub struct PageTable{
    pub root_ppn: PhysPageNum,
    pub frames: Vec<PhysFrameTracker>
}

impl PageTable{
    pub fn new() -> Self {
        let ppn = PHYSFRAME_ALLOCATOR.lock().alloc().unwrap();
        let mut table = PageTable {
            root_ppn: ppn,
            frames: Vec::new()
        };
        if unsafe { super::KERNEL_PAGE_TABLE_INIT } {
            unsafe { super::KERNEL_PAGE_TABLE.lock()
                .map_frame(VirtualPageNum(ppn.0), PTEFlag::R, ppn); };
        }else {
            unsafe { table.map_frame(VirtualPageNum(ppn.0), PTEFlag::R, ppn); };
        }
        table
    }
    pub fn map(&mut self, vaddr: VirtualPageNum, flag: PTEFlag) -> Option<PhysFrameTracker>{
        let pte = self.find_pte(vaddr).unwrap() as *mut PTE;
        let _pte = unsafe { *pte };
        if !_pte.is_valid() {
            let frame_track = PhysFrameTracker::new();
            let new_pte = PTE::new(frame_track.0, flag | PTEFlag::V);
            unsafe { *pte = new_pte; }
        }
        //assert!(pte.is_readable(), "&pte:0x{:x}, ppn:0x{:x}, bits:{:?}",
        //    pte as *const PTE as usize, pte.ppn().0, pte.flags());
        Some(PhysFrameTracker(_pte.ppn()))
    }

    pub unsafe fn map_frame(&mut self, vaddr: VirtualPageNum,
                     flag: PTEFlag,
                     frame: PhysPageNum) -> Option<PhysFrameTracker>{
        log!(debug "Maping frame: 0x{:x}", vaddr.0);
        let pte = self.find_pte(vaddr).unwrap();
        let pte =  pte as *mut PTE ;
        if !(*pte).is_valid() {
            let new_pte = PTE::new(frame, flag | PTEFlag::V);
            *pte = new_pte;
            PHYSFRAME_ALLOCATOR.lock().mark(frame);
        }
        Some(PhysFrameTracker((*pte).ppn()))
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

    fn alloc_pte(&mut self, pte: *mut PTE, flag: PTEFlag) {
        log!(debug "Alloc pte: 0x{:x}", pte as usize);
        let page = PHYSFRAME_ALLOCATOR.lock().alloc().unwrap();
        unsafe { *pte = PTE::new(page, flag); }
        let page_table = page;
        drop(page);
        // Fix: 
        unsafe {
            if super::KERNEL_PAGE_TABLE_INIT {
                super::KERNEL_PAGE_TABLE.lock().map_page_table(page_table);
            } else {
                self.map_page_table(page_table);
            }
        }
    }

    pub fn find_pte(&mut self, vaddr: VirtualPageNum) -> Option<&mut PTE> {
        log!(debug "Finding pte :0x{:x}", vaddr.0);
        let mut pte_ptr = 0 as *mut PTE;
        let mut ppn = self.root_ppn;
        for i in (0..3).rev() {
            pte_ptr = (PhysAddr::from(ppn).0
                       + vaddr.vpn_block_sv39(i) * PTE_SIZE) as *mut PTE;
            let pte = unsafe { pte_ptr.as_ref().unwrap() };
            if i > 0 {
                if !pte.is_valid() {
                    self.alloc_pte(pte_ptr, PTEFlag::V);
                }
                if pte.is_leaf() {
                    // Big page size not support
                    return None
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

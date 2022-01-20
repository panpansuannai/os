use super::address::*;
use super::pte_sv39::{
    PTE,
    PTEFlag
};

use crate::config::*;
use super::kalloc::KALLOCATOR;
use riscv::register::satp;

#[derive(Copy, Clone)]
pub struct Pgtbl {
    pub root: PageNum
}

impl const Default for Pgtbl {
    fn default() -> Self {
        Self {
            root: PageNum(0)
        }
    }
}

impl Pgtbl {
    pub fn new() -> Self {
        let page = KALLOCATOR.lock().kalloc();
        Self {
            root: page
        }
    }
    pub fn init(&mut self, page: PageNum) {
        self.root = page;
    }

    pub fn walk(&mut self, va: VirtualAddr, do_alloc: bool) -> &mut PTE {
        let page: PageNum = va.into();
        let mut ppn = self.root;
        let mut pte = unsafe { ((ppn.offset(0).0) as *mut PTE).as_mut().unwrap() };
        for level in (1..PAGE_TABLE_LEVEL).rev() {
            let pte_ptr = ppn.offset(page.vpn_block_sv39(level) * core::mem::size_of::<usize>()).0 as *mut PTE;
            pte = unsafe { pte_ptr.as_mut().unwrap() };
            if pte.is_valid() {
                if pte.is_leaf() {
                    panic!("too short page table")
                }
                ppn = pte.ppn();
            } else {
                if do_alloc {
                    let page = KALLOCATOR.lock().kalloc();
                    pte.set_ppn(page);
                    pte.set_flags(PTEFlag::V);
                    ppn = page;
                } else {
                    panic!("walk invalid")
                }
            }
        }
        unsafe {
        (ppn.offset(page.vpn_block_sv39(0) * core::mem::size_of::<usize>()).0 as *mut PTE).as_mut().unwrap()
        }
    }

    pub fn mappages(&mut self, pages: core::ops::Range<VirtualAddr>,
                    mut start: PageNum, flags: PTEFlag) 
    {
        let start_num = pages.start.floor();
        let end_num = pages.end.floor();
        (start_num.0..end_num.0).map(|page| {
            self.map(Into::<PageNum>::into(page).into(), start, flags);
            start = start + 1.into();
            0
        }).count();

    }

    pub fn map(&mut self, va: VirtualAddr, page: PageNum, flags: PTEFlag) {
        let pte = self.walk(va, true);
        if pte.is_valid() {
            panic!("remap 0x{:x}", va.0)
        }
        pte.set_ppn(page);
        pte.set_flags(flags);
    }

    pub fn activate(&self) {
        unsafe {
            satp::set(satp::Mode::Sv39, 0, self.root.0);
            asm!("sfence.vma");
        }
    }
}

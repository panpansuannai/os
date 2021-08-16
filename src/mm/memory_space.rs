use super::page_table::PageTable;
use xmas_elf::ElfFile;
use super::address::*;
use super::pte_sv39::PTEFlag;
use core::ops::RangeInclusive;
use super::phys_frame::StackFrameAllocator;
use alloc::sync::Arc;
use spin::Mutex;

pub const USER_STACK_SIZE: usize = 4096;

pub struct MemorySpace {
    page_table: PageTable
}

impl MemorySpace {
    pub fn from_elf(data: &[u8]) -> Self {
        println!("[kernel] Load from elf");
        let mut space = Self {
            page_table: PageTable::new()
        };
        let elf = ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        log!(debug "Valid elf");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtualPageNum(0);
        log!(debug "Maping program section");
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = VirtualAddr((ph.virtual_addr() as usize).into());
                let end_va = VirtualAddr(((ph.virtual_addr() + ph.mem_size()) as usize).into());
                let mut map_perm = PTEFlag::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= PTEFlag::R; }
                if ph_flags.is_write() { map_perm |= PTEFlag::W; }
                if ph_flags.is_execute() { map_perm |= PTEFlag::X; }
                space.map_area_data(start_va..=end_va, map_perm,
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]);
                max_end_vpn = end_va.ceil();
            }
        }
        // User Stack
        let stack_bottom = VirtualAddr::from(max_end_vpn);
        log!(debug "Maping user stack");
        space.map_area_zero(stack_bottom..=VirtualAddr(stack_bottom.0 + USER_STACK_SIZE),
            PTEFlag::V|PTEFlag::U|PTEFlag::R|PTEFlag::W);
        space
    }

    pub fn map_area_zero(&mut self, area: RangeInclusive<VirtualAddr>,
                         flags: PTEFlag) {
        let (start, end) = area.into_inner();
        log!(debug "[kernel] Maping zero page 0x{:x} - 0x{:x}", start.0, end.0);
        let start = VirtualPageNum::from(start);
        let end = VirtualPageNum::from(end);
        for i in start.0..=end.0 {
            self.page_table.map(VirtualPageNum(i), flags);
        }
    }

    pub fn map_area_data(&mut self, area: RangeInclusive<VirtualAddr>,
                         flags: PTEFlag, data: &[u8]) {
        let (start, end) = area.into_inner();
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}", start.0, end.0);
        let start = VirtualPageNum::from(start);
        let end = VirtualPageNum::from(end);
        for i in start.0..=end.0 {
            let traker = self.page_table
                        .map(VirtualPageNum(i), flags|PTEFlag::W).unwrap();
            log!(debug "Tracker:0x{:x}", traker.0.0);
            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                        PhysAddr::from(traker.0).0 as *mut u8, core::cmp::min(data.len(), PAGE_SIZE))
            };
            log!(debug "Trying to write 0x{:x}@0x{:x}", dst.as_ptr() as usize, dst.len());
            dst.copy_from_slice(data);
        }
    }
}

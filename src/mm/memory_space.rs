use super::page_table::PageTable;
use xmas_elf::ElfFile;
use super::address::*;
use super::pte_sv39::PTEFlag;
use core::ops::RangeInclusive;

pub const USER_STACK_SIZE: usize = 4096;

pub struct MemorySpace {
    pub page_table: PageTable,
    entry : usize
}

impl MemorySpace {
    fn validate_elf_header(header: xmas_elf::header::Header) -> bool {
        let magic = header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        true
    }
    fn set_entry_point(&mut self, entry: usize) {
        self.entry = entry
    }
    fn get_pte_flags_from_ph_flags(flags: xmas_elf::program::Flags, init: PTEFlag) -> PTEFlag {
        let mut pte_flags = init;
        if flags.is_read() { pte_flags |= PTEFlag::R; }
        if flags.is_write() { pte_flags |= PTEFlag::W; }
        if flags.is_execute() { pte_flags |= PTEFlag::X; }
        pte_flags
    }
    fn map_elf_program_table(&mut self, elf: &ElfFile) {
        log!(debug "Maping program section");
        let ph_count =elf.header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va = VirtualAddr(ph.virtual_addr() as usize);
                let end_va = VirtualAddr((ph.virtual_addr() + ph.mem_size()) as usize);
                let map_perm = MemorySpace
                        ::get_pte_flags_from_ph_flags(ph.flags(), PTEFlag::U);
                self.map_area_data_each_byte(start_va..=end_va, map_perm | PTEFlag::V,
                    &elf.input[ph.offset() as usize
                        ..(ph.offset() + ph.file_size()) as usize]);
            }
        }
    }
    fn map_user_stack(&mut self) {
        // User stack start from 0
        self.map_area_zero(VirtualAddr(1024)..=VirtualAddr(0+USER_STACK_SIZE), PTEFlag::U|PTEFlag::R|PTEFlag::W);
    }
    pub fn get_stack(&self) -> usize{
        1024
    }
    pub fn from_elf(data: &[u8]) -> Self {
        println!("[kernel] Load from elf");
        let mut space = Self {
            page_table: PageTable::new(false, Some(unsafe { &mut super::KERNEL_PAGE_TABLE})),
            entry : 0
        };
        let elf = ElfFile::new(data).unwrap();
        let elf_header = elf.header;
        MemorySpace::validate_elf_header(elf_header);
        space.set_entry_point(elf_header.pt2.entry_point() as usize);
        space.map_elf_program_table(&elf);
        space.map_user_stack();
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

    pub fn map_area_data_each_byte(&mut self, area: RangeInclusive<VirtualAddr>,
                                   flags: PTEFlag, data: &[u8]) {
        let (start, end) = area.into_inner();
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}, {:?}", start.0, end.0, flags);
        for i in start.0..end.0 {
            let page_num = self.page_table.map(VirtualAddr::from(i).into(), flags).unwrap().0;
            log!(debug "Get page number 0x{:x}", page_num.0);
            let addr = page_num.offset(VirtualAddr::from(i).page_offset());
            unsafe { *(addr.0 as *mut u8) =  data[i - start.0]; }
            crate::console::turn_off_log();
        }
        crate::console::turn_on_log();
    }

    pub fn map_area_data(&mut self, area: RangeInclusive<VirtualAddr>,
                         flags: PTEFlag, mut data: &[u8]) {
        let (start, end) = area.into_inner();
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}", start.0, end.0);
        let start_num = VirtualPageNum::from(start);
        let end_num = VirtualPageNum::from(end);
        let mut offset = start.page_offset();
        for i in start_num.0..=end_num.0 {
            let traker = self.page_table
                        .map(VirtualPageNum(i), flags).unwrap();
            log!(debug "Tracker:0x{:x}", traker.0.0);
            let len = core::cmp::min(data.len(), PAGE_SIZE);
            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                    (PhysAddr::from(traker.0).0 as *mut u8).offset(offset as isize) , len)
            };
            offset = 0;
            log!(debug "Trying to write 0x{:x} -> 0x{:x}@0x{:x}", data.as_ptr() as usize,
                dst.as_ptr() as usize, dst.len());
            dst.copy_from_slice(data);
            data = unsafe { core::slice::from_raw_parts(
                    data.as_ptr().offset(len as isize), data.len() - len) } ;
        }
    }
    pub fn get_root_ppn(&self) -> PhysPageNum {
        self.page_table.root_ppn
    }

    pub fn entry(&self) -> usize {
        self.entry
    }
}

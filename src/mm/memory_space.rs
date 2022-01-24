//use super::page_table::PageTable;
use xmas_elf::ElfFile;
use super::address::*;
use super::pte_sv39::PTEFlag;
use core::ops::Range;
use crate::config::*;
use crate::trap::context::TrapContext;
use crate::process::TrapFrame;
use crate::trap::{ __alltraps, __restore };
use super::pgtbl::Pgtbl;
use super::kalloc::KALLOCATOR;

#[derive(Copy, Clone)]
pub struct MemorySpace {
    pub page_table: Pgtbl,
    pub entry : usize
}

impl const Default for MemorySpace {
    fn default() -> Self {
        Self {
            page_table: Pgtbl::default(),
            entry: 0
        }
    }
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
                self.map_area_data_each_byte(start_va..end_va, map_perm | PTEFlag::V,
                    &elf.input[ph.offset() as usize
                        ..(ph.offset() + ph.file_size()) as usize]);
            }
        }
    }
    fn map_user_stack(&mut self) {
        // User stack start from 0
        self.map_area_zero(VirtualAddr(0)..VirtualAddr(USER_STACK_SIZE),
            PTEFlag::U|PTEFlag::R|PTEFlag::W);
    }
    pub fn trampoline_page() -> PageNum {
        PageNum::highest_page()
    }

    // Return (alltraps, restore)
    pub fn trampoline_entry() -> (usize, usize) {
        let alltraps = Into::<VirtualAddr>::into(Self::trampoline_page());
        let restore = alltraps.offset((__restore as usize  - __alltraps as usize) as isize);
        (alltraps.0, restore.0)
    }

    pub fn context_page() -> PageNum {
        PageNum(PageNum::highest_page().0 - 1)
    }
    pub fn context_addr() -> VirtualAddr {
        Into::<VirtualAddr>::into(Self::context_page())
    }
    pub fn map_context(&mut self, ctx: &TrapContext) -> PageNum {
        let context_page = Self::context_page();
        let pn = KALLOCATOR.lock().kalloc();
        self.page_table.map(context_page.into(), pn, PTEFlag::R|PTEFlag::W|PTEFlag::V);
        Into::<PhysAddr>::into(pn).write(unsafe {
          core::slice::from_raw_parts(ctx as *const TrapContext as *const u8, 
                                      core::mem::size_of::<TrapContext>())
        });
        pn
    }

    pub fn map_trapframe(&mut self, trapframe: *const TrapFrame) {
        self.page_table.map(Self::context_page().into(), PhysAddr(trapframe as usize).into(),
                            PTEFlag::R|PTEFlag::W|PTEFlag::V);
    }

    // FIXME: len should be indicated by dst
    pub fn copy_virtual_address(&mut self, src: VirtualAddr, len: usize, dst: &mut [u8]) {
        let pa: PhysAddr = self.page_table.walk(src, false).ppn().offset(src.page_offset()).into();
        pa.read(unsafe {core::slice::from_raw_parts_mut(dst.as_mut_ptr(), len)});
    }

    pub fn map_trampoline(&mut self) {
        println!("[kernel] Maping trampoline");
        let page = MemorySpace::trampoline_page();
        let pn = KALLOCATOR.lock().kalloc();
        self.page_table.map(page.into(), pn, PTEFlag::R|PTEFlag::X|PTEFlag::V);
        Into::<PhysAddr>::into(pn).write(unsafe {
            core::slice::from_raw_parts(crate::trap::__alltraps as *const u8,
                                        crate::trap::trampoline as usize - crate::trap::__alltraps as usize)
        });
    }
    pub fn get_stack(&self) -> usize{
        1024
    }
    pub fn from_elf(data: &[u8]) -> Self {
        println!("[kernel] Load from elf");
        let mut space = Self {
            page_table: Pgtbl::new(),
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

    pub fn map_area_zero(&mut self, area: Range<VirtualAddr>,
                         flags: PTEFlag) {
        let (start, end) = (area.start, area.end);
        log!(debug "[kernel] Maping zero page 0x{:x} - 0x{:x}", start.0, end.0);
        let start = PageNum::from(start);
        let end = PageNum::from(end);
        for va in start.0..end.0 {
            let pte = self.page_table.walk(va.into(), true);
            if !pte.is_valid() {
                let page = KALLOCATOR.lock().kalloc();
                pte.set_ppn(page);
                pte.set_flags(flags | PTEFlag::V);
            }
            PhysAddr::from(pte.ppn().offset(va % PAGE_SIZE)).write_bytes(0, 1);
        }
    }

    pub fn map_area_data_each_byte(&mut self, area: Range<VirtualAddr>,
                                   flags: PTEFlag, data: &[u8]) {
        let start = area.start;
        let end = area.end;
        log!(debug "[kernel] Maping data page 0x{:x} - 0x{:x}, {:?}", start.0, end.0, flags);
        for va in start.0..end.0 {
            let pte = self.page_table.walk(va.into(), true);
            if !pte.is_valid() {
                let page = KALLOCATOR.lock().kalloc();
                pte.set_ppn(page);
                pte.set_flags(flags | PTEFlag::V);
            }
            PhysAddr::from(pte.ppn().offset(va % PAGE_SIZE)).write_bytes(data[va - start.0], 1);
        }
    }

    pub fn get_root_ppn(&self) -> PageNum {
        self.page_table.root
    }

    pub fn entry(&self) -> usize {
        self.entry
    }
}

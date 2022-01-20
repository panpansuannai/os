pub mod address;
pub mod pte_sv39;
pub mod phys_frame;
pub mod page_table;
pub mod pgtbl;
pub mod kalloc;
pub mod memory_space;
use kalloc::KALLOCATOR;
use memory_space::MemorySpace;
// use page_table::PageTable;
use address::*;
use pte_sv39::PTEFlag;
use crate::{link_syms, config::PHYS_FRAME_END};
use pgtbl::Pgtbl;

pub static mut KERNEL_PAGE_TABLE : Pgtbl = Pgtbl::default();
pub static mut KERNEL_MEMORY_SPACE: MemorySpace = MemorySpace::default();

static mut KERNEL_VIRT_MEM_READY: bool = false;

pub fn init() {
    // phys_frame::init();
    let frame_start = link_syms::frames as usize;
    let frame_start: PageNum = Into::<PhysAddr>::into(frame_start).into() ;
    let frame_start: PageNum = frame_start + Into::<PageNum>::into(1usize);
    let frame_end: PageNum = Into::<PhysAddr>::into(PHYS_FRAME_END).into();
    KALLOCATOR.lock().init(frame_start..frame_end);

    // Initialize the kernel page table
    let page = KALLOCATOR.lock().kalloc();
    unsafe {
    KERNEL_PAGE_TABLE.init(page);
    KERNEL_PAGE_TABLE.mappages((link_syms::skernel as usize).into()..PHYS_FRAME_END.into(), 
                               (Into::<PhysAddr>::into(link_syms::skernel as usize)).into(), 
                               PTEFlag::V | PTEFlag::R | PTEFlag::W | PTEFlag::X)
    }
    set_sstatus_sum();
    // set_sstatus_mxr();
    // map_kernel_memory_space();
    kernel_map_trampoline();
    println!("[kernel] Try to activate VM");
    unsafe { 
        KERNEL_PAGE_TABLE.activate();
    }
    // Test
    let page = KALLOCATOR.lock().kalloc();
    println!("test alloc 0x{:x}", page.0);
}

fn map_kernel_memory_space() {
    let kernel_start: VirtualAddr = (link_syms::skernel as usize).into();
    let kernel_end: VirtualAddr = (link_syms::frames as usize).into();
    println!("[kernel] Maping kernel (0x{:x}, 0x{:x})",
        kernel_start.0, kernel_end.0);
    unsafe {
        KERNEL_PAGE_TABLE.mappages(kernel_start..kernel_end+1.into(), kernel_start.into(), 
                                    PTEFlag::R|PTEFlag::W|PTEFlag::X);
    }
}

fn set_sstatus_sum() {
    unsafe { riscv::register::sstatus::set_sum(); }
}

fn set_sstatus_mxr() {
    unsafe { riscv::register::sstatus::set_mxr(); }
}

fn kernel_map_trampoline() {
    unsafe { 
        KERNEL_MEMORY_SPACE.page_table = KERNEL_PAGE_TABLE;
        KERNEL_MEMORY_SPACE.map_trampoline();
    }
}

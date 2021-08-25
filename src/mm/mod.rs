pub mod address;
pub mod pte_sv39;
pub mod phys_frame;
pub mod page_table;
pub mod memory_space;
use page_table::PageTable;
use address::*;
use pte_sv39::PTEFlag;
use crate::map_sym::*;

pub static mut KERNEL_PAGE_TABLE : PageTable = PageTable::default();

static mut KERNEL_PAGE_TABLE_INIT: bool = false;

pub fn init() {
    phys_frame::init();
    init_kernel_page_table();
    set_sstatus_sum();
    //set_sstatus_mxr();
    map_kernel_memory_space();
    println!("[kernel] Try to activate VM");
    unsafe { 
        KERNEL_PAGE_TABLE.activate();
    }
    kernel_page_table_ready();
}

fn map_kernel_memory_space() {
    let kernel_start: VirtualAddr = (skernel as usize).into();
    let kernel_end: VirtualAddr = (ekernel as usize).into();
    println!("[kernel] Maping kernel (0x{:x}, 0x{:x})",
        kernel_start.0, kernel_end.0);
    unsafe {
        KERNEL_PAGE_TABLE.map_on_the_area(kernel_start..=kernel_end,
            PTEFlag::R|PTEFlag::W|PTEFlag::X);

        KERNEL_PAGE_TABLE.map_on_the_area(
            VirtualAddr(suser as usize)..= VirtualAddr(euser as usize),
            PTEFlag::W|PTEFlag::R|PTEFlag::X);
    }
}

fn set_sstatus_sum() {
    unsafe { riscv::register::sstatus::set_sum(); }
}

fn set_sstatus_mxr() {
    unsafe { riscv::register::sstatus::set_mxr(); }
}

fn init_kernel_page_table() {
    unsafe { KERNEL_PAGE_TABLE = PageTable::new(true, None)} ;
}

fn kernel_page_table_ready() {
    unsafe { KERNEL_PAGE_TABLE_INIT = true; }
}

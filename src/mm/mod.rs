pub mod address;
pub mod pte_sv39;
pub mod phys_frame;
pub mod page_table;
pub mod memory_space;
use page_table::PageTable;
use address::*;
use pte_sv39::PTEFlag;
use phys_frame::StackFrameAllocator;
use crate::map_sym::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use core::cell::RefCell;
use core::ops::DerefMut;

lazy_static!{
pub static ref KERNEL_PAGE_TABLE: Mutex<PageTable> = Mutex::new(PageTable::new());
}

static mut KERNEL_PAGE_TABLE_INIT: bool = false;

fn set_sum() {
    unsafe { riscv::register::sstatus::set_sum(); }
}
fn set_mxr() {
    unsafe { riscv::register::sstatus::set_mxr(); }
}

pub fn init() {
    set_sum();
    map_kernel_memory();
    println!("[kernel] Try to activate VM");
    unsafe { 
        KERNEL_PAGE_TABLE.lock().activate();
        KERNEL_PAGE_TABLE_INIT = true;
    }
}

fn map_kernel_memory() {
    let kernel_start = VirtualAddr(skernel as usize);
    let kernel_end = VirtualAddr(ekernel as usize);
    println!("[kernel] Maping kernel (0x{:x}, 0x{:x})",
    kernel_start.0, kernel_end.0);
    KERNEL_PAGE_TABLE.lock().map_on_the_area(kernel_start..=kernel_end,
        PTEFlag::R|PTEFlag::W|PTEFlag::X);

    (*KERNEL_PAGE_TABLE).lock().map_on_the_area(
        VirtualAddr(suser as usize)..= VirtualAddr(euser as usize),
        PTEFlag::W|PTEFlag::R|PTEFlag::X);
}


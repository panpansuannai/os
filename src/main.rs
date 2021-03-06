// [no_std] Don't use standard library 
#![no_std]
// [no_main] Tell compiler we don't need initialization before main() #![no_main]
#![no_main]
#![feature(naked_functions)]
#![feature(llvm_asm)]
#![feature(asm)]
// [global_asm] allow include an assemble file
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(fn_align)]
#![feature(alloc_error_handler)]
#![feature(trace_macros)]
#![allow(incomplete_features)]
#![feature(const_trait_impl)]

use crate::process::cpu::init_hart;

#[macro_use]
mod macros;

#[macro_use]
mod lang_items;
mod sbi;

#[macro_use]
mod console;

mod link_syms;
mod trap;
mod entry;
mod user;
mod syscall;
mod batch;
mod task;
mod heap;
mod mm;
mod process;

mod config;

#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate buddy_system_allocator;
extern crate spin;
#[macro_use]
extern crate bitflags;

/// Clear .bss section
fn clear_bss() {
    (link_syms::sbss as usize..link_syms::ebss as usize)
        .for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    use task::{ schedule_pcb, TASKMANAGER };
    use mm::memory_space::MemorySpace;
    
    console::turn_on_log();
    // Use new stack
    unsafe { 
        asm!("mv sp, {0}",
         in(reg) batch::KERNEL_STACK.get_top());
    }
    clear_bss();
    mm::init();
    println!("[kernel] Clear bss");
    heap::init();
    println!("[kernel] Init heap");
    trap::init();
    println!("[kernel] Init trap");

    init_hart();
    // Run user space application
    println!("[kernel] Load user address space");
    let mut virtual_space = unsafe { 
        MemorySpace::from_elf(core::slice::from_raw_parts(
                user::APP_START[0].0 as *const u8,
                user::APP_START[0].1 - user::APP_START[0].0))
    };
    println!("[kernel] Maping trampoline");
    virtual_space.map_trampoline();

    println!("[kernel] Load user address space");

    println!("[kernle] Loading apps as tasks");
    TASKMANAGER.lock().load_pcb(virtual_space);
    trap::enable_timer_interupt();
    trap::time::set_next_trigger();
    schedule_pcb();
}

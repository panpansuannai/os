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
#![feature(const_trait_impl)]

#[macro_use]
mod lang_items;
mod sbi;

#[macro_use]
mod console;

mod map_sym;
mod trap;
mod entry;
mod user;
mod syscall;
mod batch;
mod task;
mod heap;
mod mm;

#[macro_use]
extern crate lazy_static;
extern crate alloc;
extern crate buddy_system_allocator;
extern crate spin;
#[macro_use]
extern crate bitflags;

/// Clear .bss section
fn clear_bss() {
    (map_sym::sbss as usize..map_sym::ebss as usize)
        .for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

// [no_mangle] Turn off Rust's name mangling
#[no_mangle]
extern "C" fn kernel_start() {
    use batch::APP_MANAGER;
    use trap::context::TrapContext;
    use task::TASK_MANAGER;
    use mm::memory_space::MemorySpace;
    use mm::address::*;
    use mm::pte_sv39::PTEFlag;
    
    // Use new stack
    unsafe { 
        asm!("mv sp, {0}",
         in(reg) batch::KERNEL_STACK.get_top());
    }
    mm::init();
    clear_bss();
    println!("[kernel] Clear bss");
    heap::init();
    println!("[kernel] Init heap");
    trap::init();
    println!("[kernel] Init trap");

    // Run user space application
    println!("[kernel] Load user address space");
    let mut virtual_space = unsafe { 
        MemorySpace::from_elf(core::slice::from_raw_parts(
                user::APP_START[0].0 as *const u8,
                user::APP_START[0].1 - user::APP_START[0].0))
    };
    println!("[kernel] Maping trampoline");
    virtual_space.page_table.map_on_the_area(
        VirtualAddr(trap::__alltraps as usize)..=VirtualAddr(trap::trampoline as usize),
        PTEFlag::R|PTEFlag::X);
    println!("[kernel] Maping kernel stack 0x{:x} - 0x{:x}", batch::KERNEL_STACK.get_top(),
            batch::KERNEL_STACK.get_bottom());
    virtual_space.page_table.map_on_the_area(
        VirtualAddr(batch::KERNEL_STACK.get_bottom())..=VirtualAddr(batch::KERNEL_STACK.get_top()),
        PTEFlag::R|PTEFlag::W);
    println!("[kernel] Load user address space");

    let context0 = TrapContext::app_init_context(
        virtual_space.entry(),
        virtual_space.get_stack(), virtual_space.get_root_ppn().0 | 0x8000000000000000, 0, 0, 0);

    println!("[kernle] Loading apps as tasks");
    TASK_MANAGER.load_task(&context0);
    //trap::enable_timer_interupt();
    //trap::time::set_next_trigger();
    TASK_MANAGER.start_next_task();
    panic!("Shut down"); 
}

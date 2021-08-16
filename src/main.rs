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
    
    // Use new stack
    unsafe { 
        asm!("mv sp, {0}",
         in(reg) batch::KERNEL_STACK.get_sp());
    }
    mm::init();
    clear_bss();
    println!("[kernel] Clear bss");
    heap::init();
    println!("[kernel] Init heap");
    trap::init();
    println!("[kernel] Init trap");
    batch::init();
    println!("[kernel] Init batch");

    log!(info ".text [{:#x}, {:#x})", 
        map_sym::stext as usize, map_sym::etext as usize);
    log!(debug ".rodata [{:#x}, {:#x})",
        map_sym::srodata as usize, map_sym::erodata as usize);
    log!(error ".data [{:#x}, {:#x})",
        map_sym::sdata as usize, map_sym::edata as usize);

    // Run user space application
    let mut app_manager = APP_MANAGER.inner.borrow();
    println!("[kernel] Load user address space");
    let virtual_space = unsafe { 
        MemorySpace::from_elf(core::slice::from_raw_parts(
                user::APP_START[0].0 as *const u8,
                user::APP_START[0].1 - user::APP_START[0].0));
    };
    println!("[kernel] Load user address space");

    let context0 = TrapContext::app_init_context(
        app_manager.app_start_addr(0).unwrap(),
        batch::USER_STACK0.get_sp(), 0, 0, 0);

    println!("[kernle] Loading apps as tasks");
    TASK_MANAGER.load_task(&context0);
    //trap::enable_timer_interupt();
    //trap::time::set_next_trigger();
    TASK_MANAGER.start_next_task();
    panic!("Shut down"); 
}

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

#[macro_use]
extern crate lazy_static;

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
    use riscv::register::sstatus::SPP;
    use task::TASK_MANAGER;

    clear_bss();
    println!("[kernel] Clear bss");
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

    let context0 = TrapContext::app_init_context(
        app_manager.app_start_addr(0).unwrap(),
        batch::USER_STACK0.get_sp(), 0, 0, 0);

    let context1 = TrapContext::app_init_context(
        app_manager.app_start_addr(1).unwrap(),
        batch::USER_STACK1.get_sp(), 0, 0, 0);

    println!("[kernle] Loading apps as tasks");
    TASK_MANAGER.load_task(&context0);
    TASK_MANAGER.load_task(&context1);
    trap::enable_timer_interupt();
    trap::time::set_next_trigger();
    TASK_MANAGER.start_next_task();
    panic!("Shut down"); 
}

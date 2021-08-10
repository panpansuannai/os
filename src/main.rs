// [no_std] Don't use standard library 
#![no_std]
// [no_main] Tell compiler we don't need initialization before main() #![no_main]
#![no_main]
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

    clear_bss();
    println!("[kernel] Clear bss");
    trap::init();
    println!("[kernel] Init trap");
    batch::init();
    println!("[kernel] Init batch");
    let satp = riscv::register::satp::read();
    use riscv::register::satp::Mode;
    match satp.mode() {
        Mode::Bare => {
            println!("[kernel] satp mode : Bare");
        },
        Mode::Sv48 => {
            println!("[kernel] satp mode : Sv48");
        },
        Mode::Sv57 => {
            println!("[kernel] satp mode : Sv57");
        },
        Mode::Sv64 => {
            println!("[kernel] satp mode : Sv64");
        },
        Mode::Sv39 => {
            println!("[kernel] satp mode : Sv39");
        },
    }
    match riscv::register::sstatus::read().spp() {
        SPP::Supervisor => {
            println!("[kernel] SuperVisor");
        },
        SPP::User => {
            println!("[kernel] User");
        }
    }

    log!(info ".text [{:#x}, {:#x})", 
        map_sym::stext as usize, map_sym::etext as usize);
    log!(debug ".rodata [{:#x}, {:#x})",
        map_sym::srodata as usize, map_sym::erodata as usize);
    log!(error ".data [{:#x}, {:#x})",
        map_sym::sdata as usize, map_sym::edata as usize);

    // Run user space application
    let mut app_manager = APP_MANAGER.inner.borrow();
    app_manager.run_app(0);
    panic!("Shut down"); 
}

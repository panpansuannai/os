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

mod lang_items;
mod sbi;
mod console;
mod map_sym;
mod trap;
mod entry;

/// Clear .bss section
fn clear_bss() {
    (map_sym::sbss as usize..map_sym::ebss as usize)
        .for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

// [no_mangle] Turn off Rust's name mangling
fn kernel_start() {
    clear_bss();
    log!(info ".text [{:#x}, {:#x})", 
        map_sym::stext as usize, map_sym::etext as usize);
    log!(debug ".rodata [{:#x}, {:#x})",
        map_sym::srodata as usize, map_sym::erodata as usize);
    log!(error ".data [{:#x}, {:#x})",
        map_sym::sdata as usize, map_sym::edata as usize);
    panic!("Shut down");
}

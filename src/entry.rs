global_asm!(".section .bss.stack
             .globl boot_stack
         boot_stack:
             .space 4096 * 16
             .globl boot_stack_top
         boot_stack_top:");

use crate::map_sym::boot_stack_top;

#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        let stack_top = boot_stack_top as usize;
        asm!("mv sp, {0}", in(reg) stack_top);
    }
    crate::kernel_start();
}

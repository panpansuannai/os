#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!("la sp, boot_stack",
        "j kernel_start",
        options(noreturn)
        );
    }
}
global_asm!(
    ".section .bss.stack
    .globl boot_stack
boot_stack:
    .space 32
    .globl boot_stack_top
boot_stack_top:");

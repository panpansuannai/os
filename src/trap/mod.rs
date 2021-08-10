pub mod context;

use riscv::register::{
    mtvec::TrapMode,
    stvec,
    scause::{
        self,
        Trap,
        Exception,
        Interrupt,
    },
    stval,
    sie,
};
use context::TrapContext;

global_asm!(include_str!("traps.s"));
extern "C" { pub fn __alltraps(); }
extern "C" { pub fn __restore(cx: usize); }
pub fn _restore(cx: usize){
    println!("[kernel] restore context");
    unsafe { __restore(cx); }
}

pub fn init() { 
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub extern "C" fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext{
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.general_reg[10] =
                crate::syscall::syscall(cx.general_reg[17],
                        [cx.general_reg[10],
                        cx.general_reg[11],
                        cx.general_reg[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            panic!("[kernel] PageFault in application, core dumped. sepc:0x{:X}", cx.sepc);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            panic!("[kernel] IllegalInstruction in application, core dumped. sepc:0x{:X}", cx.sepc);
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}


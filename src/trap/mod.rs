pub mod context;
pub mod time;

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

extern "C" { pub fn __alltraps(); }
extern "C" { pub fn __restore(cx: usize); }
extern "C" { pub fn trampoline(); }

pub fn _restore(cx: usize){
    println!("[kernel] restore context: 0x{:x}", cx);
    unsafe { log!(debug "context: {:?}", *(cx as *const TrapContext)); }
    unsafe { __restore(cx); }
}

global_asm!(include_str!("traps.s"));
pub fn init() { 
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub extern "C" fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext{
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let mut p = cx.general_reg[10];
            if cx.general_reg[17] == crate::syscall::SYS_YIELD {
                p = cx as *const TrapContext as usize;
            }
            cx.general_reg[10] =
                crate::syscall::syscall(cx.general_reg[17],
                        [p,
                        cx.general_reg[11],
                        cx.general_reg[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            if let riscv::register::sstatus::SPP::Supervisor = cx.sstatus.spp() {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }else {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            panic!(" IllegalInstruction in application, core dumped. sepc:0x{:X}", cx.sepc);
        }
        Trap::Exception(Exception::InstructionPageFault) => {
            panic!(" InstructionPageFault, core dumped, sepc: 0x{:x}, scause:{:?}", cx.sepc, scause.cause());
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::trap::time::set_next_trigger();
            crate::syscall::syscall(crate::syscall::SYS_YIELD,
                    [cx as *const TrapContext as usize, 0, 0]) as usize;
        }
        _ => {
            panic!("Unsupported trap {:?}:0x{:x}, stval = {:#x}!",
                   scause.cause(), scause.bits(), stval);
        }
    }
    cx
}


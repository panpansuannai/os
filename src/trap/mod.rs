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
use crate::task::TASK_MANAGER;

extern "C" { pub fn __alltraps(); }
extern "C" { pub fn __restore(cx: usize, satp: usize); }
extern "C" { pub fn trampoline(); }

pub fn _restore(cx: usize, satp: usize){
    println!("[kernel] restore context: 0x{:x}", cx);
    //unsafe { log!(debug "context: {:?}", *(cx as *const TrapContext)); }
    unsafe { 
    let (_, restore) = crate::mm::KERNEL_MEMORY_SPACE.trampoline_entry();
    let restore  = core::mem::transmute::<*const (), fn (usize, usize)>(restore as *const ());
    restore(cx, satp);
    };
}

global_asm!(include_str!("traps.s"));
pub fn init() { 
    unsafe {
        let (alltraps, _) = crate::mm::KERNEL_MEMORY_SPACE.trampoline_entry();
        stvec::write(alltraps, TrapMode::Direct);
    }
}

pub fn enable_timer_interupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub extern "C" fn trap_handler() -> ! {
    println!("[kernel] In trap handler");
    let mut current_task = TASK_MANAGER.get_current_task();
    let cx = current_task.get_context();
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
    TASK_MANAGER.update_current_task(current_task);
    TASK_MANAGER.start_next_task();
    loop {} 
}


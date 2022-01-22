pub mod context;
pub mod time;

use core::borrow::{BorrowMut, Borrow};

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

use crate::mm::memory_space::MemorySpace;
use crate::task::{schedule_pcb, TASKMANAGER};
use crate::process::cpu::*;
use crate::process::TrapFrame;

extern "C" { pub fn __alltraps(); }
extern "C" { pub fn __restore(cx: usize, satp: usize); }
extern "C" { pub fn trampoline(); }

pub fn _restore(cx: usize, satp: usize){
    println!("[kernel] restore context: 0x{:x}", cx);
    //unsafe { log!(debug "context: {:?}", *(cx as *const TrapContext)); }
    unsafe { 
    let (_, restore) = MemorySpace::trampoline_entry();
    let restore  = core::mem::transmute::<*const (), fn (usize, usize)>(restore as *const ());
    restore(cx, satp);
    };
}

global_asm!(include_str!("traps.s"));
pub fn init() { 
    unsafe {
        let (alltraps, _) = MemorySpace::trampoline_entry();
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
    let a = TASKMANAGER.lock();
    let mut b = a.current_pcb();
    let c = b.borrow_mut().lock();
    // Fixme: Don't skip the reference lifetime checker;
    let cx = unsafe {c.trapframe().as_mut().unwrap()};
    // Fixme: ugly
    unsafe {
        TASKMANAGER.force_unlock();
        b.force_unlock();
    }
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx["sepc"] += 4;
            let mut p = cx.general_reg[10];
            if cx.general_reg[17] == crate::syscall::SYS_YIELD {
                p = cx as *mut TrapFrame as usize;
            }
            cx.general_reg[10] =
                crate::syscall::syscall(cx.general_reg[17],
                        [p,
                        cx.general_reg[11],
                        cx.general_reg[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            panic!("store fault 0x{:x}", cx["sepc"]);
            /*
            if let riscv::register::sstatus::SPP::Supervisor = cx.sstatus.spp() {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }else {
                panic!("PageFault in application, core dumped. sepc:0x{:x}", cx.sepc);
            }
            */
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
                    [cx as *const TrapFrame as usize, 0, 0]) as usize;
        }
        _ => {
            panic!("Unsupported trap {:?}:0x{:x}, stval = {:#x}!",
                   scause.cause(), scause.bits(), stval);
        }
    }
    schedule_pcb();
}


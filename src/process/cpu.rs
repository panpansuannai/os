use riscv::register::mhartid;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use super::Pid;

#[derive(Clone, Copy)]
pub struct Cpu {
    pub hartid: usize,
    pub pid: Option<Pid>
}

lazy_static!{
    static ref HARTS: Mutex<Vec<Cpu>> = Mutex::new(Vec::new());
}

pub fn init_hart() {
    HARTS.lock().push(Cpu {
        hartid: hartid(),
        pid: None,
    });
}

pub fn current_hart() -> Cpu {
    for i in HARTS.lock().iter() {
        if i.hartid == hartid() {
            return i.clone()
        }
    }
    panic!("uninit hartid {}", mhartid::read());
}

pub fn current_hart_set_pid(pid: Pid) {
    for i in HARTS.lock().iter_mut() {
        if i.hartid == hartid() {
            i.pid = Some(pid);
        }
    }
}

pub fn hartid() -> usize {
    let ret: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) ret);
    }
    ret
}

use spin::{Mutex, MutexGuard};
use crate::mm::memory_space::MemorySpace;
use crate::process::cpu::{current_hart, current_hart_set_pid};
use crate::process::{PcbState, Pcb, Pid, restore_trapframe};

lazy_static!{
    pub static ref TASKMANAGER: Mutex<TaskManagerInner<'static>> = Mutex::new(TaskManagerInner {
            pcbs: [None; crate::user::APP_NUM],
    });
}

pub struct TaskManagerInner<'a> {
    pcbs: [Option<Mutex<Pcb<'a>>>; crate::user::APP_NUM],
}

impl<'a> TaskManagerInner<'a> {
    pub fn load_pcb(&mut self, memory_space: MemorySpace) {
        // Fixme: when ran out of pcbs
        let pcb = Mutex::new(Pcb::new(memory_space));
        pcb.lock().set_state(PcbState::Ready);
        for (i, p) in self.pcbs.iter_mut().enumerate() {
            if let None = p {
                pcb.lock().pid = Some(i);
                *p = Some(pcb);
                break;
            }
        }
    }

    pub fn get_pcb(&self, pid: Pid) -> MutexGuard<Pcb<'a>> {
        match self.pcbs[pid] {
            Some(ref p) => {
                p.lock()
            },
            None => {
                panic!("invalid pid");
            }
        }
    }

    pub fn current_pcb(&self) -> MutexGuard<Pcb<'a>> {
        let cpu = current_hart();
        self.get_pcb(cpu.pid.unwrap())
    }
}

pub fn schedule_pcb() -> ! {
    let mut satp = None;
    for i in TASKMANAGER.lock().pcbs.iter() {
        if let Some(p) = i {
            let mut pcb = p.lock();
            if let PcbState::Ready = pcb.state() {
                pcb.set_state(PcbState::Running);
                current_hart_set_pid(pcb.pid.unwrap());
                satp = Some(pcb.trapframe["satp"]);
                break;
            }
        }
    }
    if let Some(satp) = satp {
        restore_trapframe(satp);
    } else {
        panic!("No ready pcb");
    }
}


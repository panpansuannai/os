mod tasks;
use tasks::{TaskControlBlock, TaskStatus};
use crate::mm::memory_space::MemorySpace;

use core::cell::RefCell;

pub struct TaskManager {
    inner: RefCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; crate::user::APP_NUM],
    current_task: usize
}

#[repr(C)]
pub struct KernelTask {
    pub stack: usize,
    pub satp: usize,
    pub user_stack: usize
}

unsafe impl Sync for TaskManager {}
lazy_static!{
pub static ref TASK_MANAGER: TaskManager = TaskManager {
    inner: RefCell::new({
        TaskManagerInner {
            tasks: [TaskControlBlock::empty_block(); crate::user::APP_NUM],
            current_task: 0
        }
    })
};
}

impl TaskManager {
    pub fn set_current_task_ready(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].set_status(TaskStatus::Ready);
    }
    
    pub fn get_current_task(&self) -> TaskControlBlock{
        let inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current]

    }

    pub fn update_current_task(&self, tcb: TaskControlBlock) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current] = tcb;
        
    }

    pub fn start_next_task(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        let mut next = (current + 1) % inner.tasks.len();
        loop {
            if let TaskStatus::Ready = inner.tasks[next].get_status() {
                break;
            }
            if next == current {
                panic!("No tasks");
            }
            next = (next + 1) % inner.tasks.len();
        }
        println!("[kernel] Start next task:{}", next);
        inner.tasks[next].set_status(TaskStatus::Running);
        inner.current_task = next;
        let cx_ptr = inner.tasks[next].get_cx_ptr();
        let satp = inner.tasks[next].get_satp();
        drop(inner);
        crate::trap::_restore(cx_ptr, satp);
    }

    pub fn load_task(&self, memory_space: MemorySpace) {
        //println!("[kernel] Loading task for TrapContext: 0x{:x} -> sepc: 0x{:x}",
                 //cx as *const TrapContext as usize, cx.sepc);
        let mut inner = self.inner.borrow_mut();
        let mut empty = 0;
        loop {
            if let TaskStatus::UnInit = inner.tasks[empty].get_status() {
                break;
            }
            empty = (empty + 1) % inner.tasks.len();
        }
        inner.tasks[empty] = TaskControlBlock::new(TaskStatus::Ready, memory_space);
    }

    pub fn exit_current_task(&self){
        let mut inner = self.inner.borrow_mut();
        println!("[kernel] Current task {} exited", inner.current_task);
        let current = inner.current_task;
        inner.tasks[current].set_status(TaskStatus::Exit);
    }
}

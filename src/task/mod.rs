mod tasks;
use tasks::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapContext;

use core::cell::RefCell;

pub struct TaskManager {
    inner: RefCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; crate::user::APP_NUM],
    current_task: usize
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
    pub fn set_current_task_cx(&self, cx: usize) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].set_cx_ptr(cx);
    }

    pub fn set_current_task_ready(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].set_status(TaskStatus::Ready);
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
        drop(inner);
        crate::trap::_restore(cx_ptr);
    }

    pub fn load_task(&self, cx: &TrapContext) {
        println!("[kernel] Loading task for TrapContext: 0x{:x} -> sepc: 0x{:x}",
                 cx as *const TrapContext as usize, cx.sepc);
        let mut inner = self.inner.borrow_mut();
        let mut empty = 0;
        loop {
            if let TaskStatus::Empty = inner.tasks[empty].get_status() {
                break;
            }
            empty = (empty + 1) % inner.tasks.len();
        }
        inner.tasks[empty].set_cx_ptr(cx as *const TrapContext as usize);
        inner.tasks[empty].set_status(TaskStatus::Ready);
    }

    pub fn exit_current_task(&self){
        let mut inner = self.inner.borrow_mut();
        println!("[kernel] Current task {} exited", inner.current_task);
        let current = inner.current_task;
        inner.tasks[current].set_status(TaskStatus::Exit);
    }
}

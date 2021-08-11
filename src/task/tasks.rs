#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Empty,
    UnInit,
    Ready,
    Running,
    Exit
}

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    status: TaskStatus,
    task_cx_ptr: usize
}

impl TaskControlBlock {
    pub fn empty_block() -> TaskControlBlock {
        TaskControlBlock {
            status: TaskStatus::Empty,
            task_cx_ptr: 0
        }
    }
    
    pub fn set_cx_ptr(&mut self, cx:usize) {
        self.task_cx_ptr = cx;
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn get_cx_ptr(&self) -> usize {
        self.task_cx_ptr
    }

    pub fn get_status(&self) -> TaskStatus {
        self.status
    }
}

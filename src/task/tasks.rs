use crate::trap::context::TrapContext;
use crate::mm::address::*;
use crate::mm::memory_space::MemorySpace;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exit
}

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    status: TaskStatus,
    memory_space: MemorySpace,
}

impl TaskControlBlock {
    pub fn empty_block() -> TaskControlBlock {
        TaskControlBlock {
            status: TaskStatus::UnInit,
            memory_space: MemorySpace::default()
        }
    }

    pub fn new(status: TaskStatus, memory_space: MemorySpace) -> Self {
        Self {
            status,
            memory_space,
        }
    }
    
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn get_satp(&self) -> usize {
        self.memory_space.get_root_ppn().0 | 0x8000000000000000
    }

    pub fn set_satp(&mut self, satp: usize) {
        //self.task_satp = satp
    }

    pub fn get_cx_ptr(&self) -> usize {
        MemorySpace::context_addr().0
    }

    pub fn get_status(&self) -> TaskStatus {
        self.status
    }

    pub fn get_context(&mut self) -> &mut TrapContext {
        let ppn = self.memory_space.page_table.walk(MemorySpace::context_addr(), false).ppn();
        let pa: PhysAddr = ppn.into();
        unsafe {
        (pa.0 as *const TrapContext as *mut TrapContext).as_mut().unwrap()
        }
    }

    pub fn get_memory_space(&mut self) -> &mut MemorySpace{
        &mut self.memory_space
    }
}

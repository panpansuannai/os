pub mod trapframe;
pub mod cpu;

use crate::mm::address::*;
use core::sync::atomic::{self, AtomicBool};
pub use trapframe::TrapFrame;

use crate::mm::memory_space::MemorySpace;
use crate::config::*;

pub type Pid = usize;

#[derive(Clone, Copy)]
pub enum PcbState {
    UnInit,
    Ready,
    Running,
    Exit
}

pub struct Pcb<'a> {
    pub pid: Option<Pid>,
    pub state: PcbState,
    pub trapframe: &'a mut TrapFrame,
    pub memory_space: MemorySpace,
}

impl<'a> Pcb<'a> {
    // Fixme: Remember to release kernel stack and trapframe when process dead
    pub fn new(memory_space: MemorySpace) -> Self {
        use crate::mm::kalloc::KALLOCATOR;
        let mut pcb = unsafe { Self {
            pid: None,
            state: PcbState::UnInit,
            trapframe: (Into::<PhysAddr>::into(KALLOCATOR.lock().kalloc()).0 as *const TrapFrame 
                        as *mut TrapFrame).as_mut().unwrap().from_memory_space(memory_space),
            memory_space,
        }};

        // Assume that all process's stack in a page 
        let stack = KALLOCATOR.lock().kalloc();
        pcb.trapframe.kernel_sp = Into::<PhysAddr>::into(stack).0 + PAGE_SIZE;
        // Fixme: every process may has a independent page table
        pcb.trapframe.kernel_satp = riscv::register::satp::read().bits();
        // Map trapframe
        pcb.memory_space.map_trapframe(pcb.trapframe);
        pcb
    }


    pub fn state(&self) -> PcbState {
        self.state
    }

    pub fn set_state(&mut self, state: PcbState) -> PcbState {
        let old_state = self.state;
        self.state = state;
        old_state
    }

    pub fn trapframe(&mut self) -> *mut TrapFrame {
        self.trapframe as *mut TrapFrame
    }

    pub fn exit(&mut self) {
        // Fixme: Release memory
        self.state = PcbState::Exit;
    }
}

pub fn restore_trapframe(satp: usize) -> !{
    let (_, restore) = crate::mm::memory_space::MemorySpace::trampoline_entry();
    let restore  = unsafe { core::mem::transmute::<usize, fn (usize, usize) -> !>(restore) };
    let tf = Into::<PhysAddr>::into(MemorySpace::context_page()).0;
    restore(tf, satp);
}

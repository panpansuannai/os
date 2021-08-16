use super::address::{
    PhysAddr,
    PhysPageNum,
};
use alloc::vec::Vec;
use crate::spin::Mutex;
use crate::map_sym::euser;

pub const PHYS_FRAME_END: usize = 0x80800000;

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> ;
    fn dealloc(&mut self, ppn: PhysPageNum) -> Result<(), &str> ;
    fn mark(&mut self, ppn: PhysPageNum) ;
}

lazy_static!{
pub static ref PHYSFRAME_ALLOCATOR: Mutex<StackFrameAllocator> = 
    Mutex::new(StackFrameAllocator::new(
            euser as usize, PHYS_FRAME_END));
}

pub struct StackFrameAllocator {
    pub current: PhysPageNum,
    pub end: PhysPageNum,
    pub recycled: Vec<PhysPageNum>
}

impl StackFrameAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        log!(debug "New StackFrameAllocator");
        Self {
            current: PhysAddr(start).ceil(),
            end: PhysAddr(end).floor(),
            recycled: Vec::new()
        }
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if self.current <= self.end {
            self.current = PhysPageNum(self.current.0 + 1);
            log!(debug "Physic frame alloc return 0x{:x}", self.current.0 - 1);
            return Some(PhysPageNum(self.current.0 - 1))
        }
        None
    }

    fn dealloc(&mut self, ppn: PhysPageNum) -> Result<(), &str> {
        if ppn <= self.end {
            self.recycled.push(ppn);
            return Ok(())
        }
        Err("Bad deallocation")
    }

    fn mark(&mut self, ppn: PhysPageNum) {
        if self.current <= ppn {
            log!(debug "Physic frame mark 0x{:x}", ppn.0);
            self.current = PhysPageNum(ppn.0 + 1);
        }
    }
}

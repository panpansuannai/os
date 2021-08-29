use super::address::{
    PhysAddr,
    PhysPageNum,
};
use alloc::vec::Vec;
use crate::spin::Mutex;
use crate::map_sym::frames;

pub const PHYS_FRAME_END: usize = 0x80800000;

static mut CURRENT: PhysPageNum = PhysPageNum(0);
static mut END: PhysPageNum = PhysPageNum(0);

static mut RECYCLE: Vec<PhysPageNum> = Vec::new();

pub fn init() {
    unsafe {
    CURRENT = PhysAddr(frames as usize + 1).ceil();
    END = PhysAddr(PHYS_FRAME_END).floor();
    }
}

lazy_static!{
static ref PHYSFRAME_ALLOCATOR: Mutex<StackFrameAllocator> = 
    Mutex::new(StackFrameAllocator {
        current: PhysPageNum(0),
        end: PhysPageNum(0),
        recycled: Vec::new()
    });
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<PhysPageNum> ;
    fn dealloc(&mut self, ppn: PhysPageNum) -> Result<(), ()> ;
    fn mark(&mut self, ppn: PhysPageNum) ;
}


pub struct StackFrameAllocator {
    pub current: PhysPageNum,
    pub end: PhysPageNum,
    pub recycled: Vec<PhysPageNum>
}

pub fn alloc() -> Option<PhysPageNum> {
    unsafe {
        if CURRENT <= END {
            CURRENT = PhysPageNum(CURRENT.0 + 1);
            log!(debug "Physic frame alloc return 0x{:x}", CURRENT.0 - 1);
            return Some(PhysPageNum(CURRENT.0 - 1))
        }
    }
    None
}
pub fn dealloc(ppn: PhysPageNum) -> Result<(), ()> {
    unsafe {
        if ppn <= END {
            RECYCLE.push(ppn);
            return Ok(())
        }
    }
        Err(())
}

pub fn mark(ppn: PhysPageNum) {
    PHYSFRAME_ALLOCATOR.lock().mark(ppn)
}

/*
pub fn init() {
    log!(debug "Physic frame allocator init");
    let mut allocator = PHYSFRAME_ALLOCATOR.lock();
    allocator.current = PhysAddr(euser as usize).ceil();
    allocator.end = PhysPageNum::from(PHYS_FRAME_END);
}
*/

impl StackFrameAllocator {
    fn new(start: usize, end: usize) -> Self {
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

    fn dealloc(&mut self, ppn: PhysPageNum) -> Result<(), ()> {
        if ppn <= self.end {
            self.recycled.push(ppn);
            return Ok(())
        }
        Err(())
    }

    fn mark(&mut self, ppn: PhysPageNum) {
        if self.current <= ppn {
            log!(debug "Physic frame mark 0x{:x}", ppn.0);
            self.current = PhysPageNum(ppn.0 + 1);
        }
    }
}

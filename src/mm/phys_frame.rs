/*
use super::address::{
    PhysAddr,
    PageNum,
};
use alloc::vec::Vec;
use crate::spin::Mutex;
use crate::link_syms;
use crate::config::*;

static mut CURRENT: PageNum = PageNum(0);
static mut END: PageNum = PageNum(0);

static mut RECYCLE: Vec<PageNum> = Vec::new();

pub fn init() {
    unsafe {
        CURRENT = PhysAddr(link_syms::frames as usize + 1).ceil();
        END = PhysAddr(PHYS_FRAME_END).floor();
    }
}

lazy_static!{
static ref PHYSFRAME_ALLOCATOR: Mutex<StackFrameAllocator> = 
    Mutex::new(StackFrameAllocator {
        current: PageNum(0),
        end: PageNum(0),
        recycled: Vec::new()
    });
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<PageNum> ;
    fn dealloc(&mut self, ppn: PageNum) -> Result<(), ()> ;
    fn mark(&mut self, ppn: PageNum) ;
}


pub struct StackFrameAllocator {
    pub current: PageNum,
    pub end: PageNum,
    pub recycled: Vec<PageNum>
}

pub fn alloc() -> Option<PageNum> {
    unsafe {
        if CURRENT <= END {
            CURRENT = PageNum(CURRENT.0 + 1);
            log!(debug "Physic frame alloc return 0x{:x}", CURRENT.0 - 1);
            return Some(PageNum(CURRENT.0 - 1))
        }
    }
    None
}
pub fn dealloc(ppn: PageNum) -> Result<(), ()> {
    unsafe {
        if ppn <= END {
            RECYCLE.push(ppn);
            return Ok(())
        }
    }
        Err(())
}

pub fn mark(ppn: PageNum) {
    PHYSFRAME_ALLOCATOR.lock().mark(ppn)
}

/*
pub fn init() {
    log!(debug "Physic frame allocator init");
    let mut allocator = PHYSFRAME_ALLOCATOR.lock();
    allocator.current = PhysAddr(euser as usize).ceil();
    allocator.end = PageNum::from(PHYS_FRAME_END);
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
    fn alloc(&mut self) -> Option<PageNum> {
        if self.current <= self.end {
            self.current = PageNum(self.current.0 + 1);
            log!(debug "Physic frame alloc return 0x{:x}", self.current.0 - 1);
            return Some(PageNum(self.current.0 - 1))
        }
        None
    }

    fn dealloc(&mut self, ppn: PageNum) -> Result<(), ()> {
        if ppn <= self.end {
            self.recycled.push(ppn);
            return Ok(())
        }
        Err(())
    }

    fn mark(&mut self, ppn: PageNum) {
        if self.current <= ppn {
            log!(debug "Physic frame mark 0x{:x}", ppn.0);
            self.current = PageNum(ppn.0 + 1);
        }
    }
}
*/

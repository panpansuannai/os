use super::address::*;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::config::PAGE_SIZE;

struct MemRun {
    next: usize
}

lazy_static!{ 
    pub static ref KALLOCATOR: Mutex<Kallocator> = Mutex::new(Kallocator::default());
}

pub struct Kallocator(usize);

impl Default for Kallocator {
    fn default() -> Self {
        Self(0)
    }
}

use core::ops::Range;
impl Kallocator {
    pub fn init(&mut self, pages: Range<PageNum>) {
        use core::convert::AsMut;
        self.0 = pages.start.into();
        for i in (pages.start.0)..pages.end.0 {
            let mut pa: PhysAddr = Into::<PageNum>::into(i).into();
            let pa: &mut usize = pa.as_mut();
            *pa = i + 1;
        }
        let mut pa: PhysAddr = (pages.end - 1.into()).into();
        let pa: &mut usize = pa.as_mut();
        *pa = 0;
    }

    pub fn kalloc(&mut self) -> PageNum {
        if self.0 == 0 {
            panic!("run out of memory");
        }
        let pa: PhysAddr = Into::<PageNum>::into(self.0).into();
        let pa: &usize = pa.as_ref();
        let ret: PageNum = self.0.into();
        self.0 = *pa;
        // REMOVE
        if self.0 == 0 {
            println!("warn: kalloc all memory, 0x{:x}", ret.0);
        }
        // clear page
        Into::<PhysAddr>::into(ret).write_bytes(0, PAGE_SIZE);
        ret
    }

    pub fn kfree(&mut self, page: PageNum) {
        let next = self.0;
        let mut pa: PhysAddr = page.into();
        *pa.as_mut() = next;
        self.0 = page.0;
    }
    
}


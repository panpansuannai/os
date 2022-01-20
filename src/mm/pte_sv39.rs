///  Riscv SV39 PTE (Page Table Entry)
///
///  |----------|---------- ppn -----------|-------|--------|
///  | Reserved | ppn[0] | ppn[1] | ppn[2] |  RSW  | flags  |
///  |------------------------------------------------------|
///  | 10 bits  | 9 bits | 9 bits | 9 bits | 2bits | 8 bits |
///  -------------------------------------------------------|
///                                        |- PTE_FLAG_SIZE-|
/// use super::address::PageNum;
use crate::config::*;
use super::PageNum;

#[derive(Clone, Copy)]
pub struct PTE(usize);

impl PTE {
    pub fn new(ppn: PageNum, flag: PTEFlag) -> Self {
        PTE (ppn.0 << PTE_PPN_OFFSET | flag.bits() as usize)
    }

    pub fn ppn(&self) -> PageNum {
        PageNum(
            (self.0 >> PTE_PPN_OFFSET) & ((1 << 27) -1)
        )
    }

    pub fn is_valid(&self) -> bool {
        self.test_flags(PTEFlag::V)
    }

    pub fn test_flags(&self, flags: PTEFlag) -> bool {
        self.flags() & flags != PTEFlag::empty()
    }

    pub fn is_leaf(&self) -> bool {
        self.is_valid()
            && (self.test_flags(PTEFlag::R)
            ||  self.test_flags(PTEFlag::X))
    }

    pub fn is_readable(&self) -> bool {
        self.is_valid() && self.test_flags(PTEFlag::R)
    }

    pub fn set_ppn(&mut self, page_num: PageNum) {
        self.0 = (page_num.0 << PTE_PPN_OFFSET) 
            | (self.0 % (1 << PTE_PPN_OFFSET));
    }

    pub fn flags(&self) -> PTEFlag {
        PTEFlag::from_bits(self.0 & ((1 << PTE_FLAG_SIZE) - 1)).unwrap()
    }

    pub fn set_flags(&mut self, flags: PTEFlag) {
        self.0 = (self.0 >> PTE_FLAG_SIZE << PTE_FLAG_SIZE)
            | flags.bits() as usize
    }
}

impl From<*const PTE> for PTE {
    fn from(ptr: *const PTE) -> PTE {
        PTE(unsafe { *(ptr as *const usize) })
    }
}

const PTE_FLAG_V: usize = 0;
const PTE_FLAG_R: usize = 1;
const PTE_FLAG_W: usize = 2;
const PTE_FLAG_X: usize = 3;
const PTE_FLAG_U: usize = 4;
const PTE_FLAG_G: usize = 5;
const PTE_FLAG_A: usize = 6;
const PTE_FLAG_D: usize = 7;

bitflags!{
    pub struct PTEFlag: usize {
        const V = 1 << PTE_FLAG_V;
        const R = 1 << PTE_FLAG_R ;
        const W = 1 << PTE_FLAG_W ;
        const X = 1 << PTE_FLAG_X ;
        const U = 1 << PTE_FLAG_U ;
        const G = 1 << PTE_FLAG_G ;
        const A = 1 << PTE_FLAG_A ;
        const D = 1 << PTE_FLAG_D ;
    }
}


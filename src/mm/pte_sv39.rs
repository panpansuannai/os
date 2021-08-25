///  Riscv SV39 PTE (Page Table Entry)
///
///  |----------|---------- ppn -----------|-------|--------|
///  | Reserved | ppn[0] | ppn[1] | ppn[2] |  RSW  | flags  |
///  |------------------------------------------------------|
///  | 10 bits  | 9 bits | 9 bits | 9 bits | 2bits | 8 bits |
///  -------------------------------------------------------|
///                                        |- PTE_FLAG_SIZE-|
///
use super::address::PhysPageNum;

pub const PTE_FLAG_SIZE: usize = 8;
pub const PTE_PPN_OFFSET: usize = 10;
pub const PTE_SIZE: usize = core::mem::size_of::<PTE_BITS>();

type PTE_BITS = usize ;

#[derive(Clone, Copy)]
pub struct PTE {
    bits: PTE_BITS
}

impl PTE {

    pub fn new(ppn: PhysPageNum, flag: PTEFlag) -> Self {
        PTE {
            bits: (ppn.0 << PTE_PPN_OFFSET | flag.bits() as usize)
        }
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(
            self.bits / ( 1 << PTE_PPN_OFFSET)
        )
    }

    pub fn is_valid(&self) -> bool {
        self.flags() & PTEFlag::V != PTEFlag::empty()
    }

    pub fn is_leaf(&self) -> bool {
        self.is_valid()
            && (self.flags() & PTEFlag::R != PTEFlag::empty() 
                || self.flags() & PTEFlag::X != PTEFlag::empty())
    }

    pub fn is_readable(&self) -> bool {
        let flags = self.flags();
        self.is_valid()
            && (flags & PTEFlag::R != PTEFlag::empty())
    }

    pub fn set_ppn(&mut self, page_num: PhysPageNum) {
        self.bits = (page_num.0 << PTE_PPN_OFFSET) 
            & (self.bits % (1 << PTE_PPN_OFFSET));
    }

    pub fn flags(&self) -> PTEFlag {
        PTEFlag::from_bits(self.bits as u8).unwrap()
    }

    pub fn set_flags(&mut self, flags: PTEFlag) {
        self.bits = (self.bits >> PTE_FLAG_SIZE << PTE_FLAG_SIZE)
            & flags.bits() as usize
    }

}

impl From<*const PTE> for PTE {
    fn from(ptr: *const PTE) -> PTE {
        PTE {
            bits: unsafe { *(ptr as *const usize) }
        }
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
    pub struct PTEFlag: u8 {
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


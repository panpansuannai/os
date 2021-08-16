pub const PAGE_OFFSET_BIT: usize = 12;
pub const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct VirtualAddr(pub usize);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PhysPageNum(pub usize);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct VirtualPageNum(pub usize);

impl VirtualAddr{
    pub fn floor(&self) -> VirtualPageNum{
        VirtualPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> VirtualPageNum{
        VirtualPageNum((self.0 + PAGE_SIZE - 1 ) / PAGE_SIZE)
    }
}
impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1 ) / PAGE_SIZE)
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(physaddr: PhysAddr) -> PhysPageNum {
        physaddr.floor()
    }
}

impl From<usize> for PhysPageNum {
    fn from(u: usize) -> PhysPageNum {
        PhysPageNum::from(PhysAddr(u))
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(page_num: PhysPageNum) -> PhysAddr {
        PhysAddr(page_num.0 << PAGE_OFFSET_BIT)
    }
}

impl From<VirtualAddr> for VirtualPageNum {
    fn from(vaddr: VirtualAddr) -> VirtualPageNum {
        vaddr.floor()
    }
}

impl From<VirtualPageNum> for VirtualAddr {
    fn from(page_num: VirtualPageNum) -> VirtualAddr {
        VirtualAddr(page_num.0 << PAGE_OFFSET_BIT)
    }
}

impl PhysPageNum {
    pub fn clear(&self) {
        let addr: PhysAddr = self.clone().into();
        let slice = unsafe {
            core::slice::from_raw_parts_mut(addr.0 as *mut u8, PAGE_SIZE)
        };
        slice.fill(0);
    }
}
const SV39_VPN_BIT: usize = 9;
const PAGE_TABLE_LEVEL: usize = 3;
/// SV39:
/// ------------------------------
/// 0000... | vpn2 | vpn1 | vpn0 |
/// ------------------------------
impl VirtualPageNum {
    pub fn vpn_block_sv39(&self, mut level: usize) -> usize {
        if level >= PAGE_TABLE_LEVEL {
            panic!("Page Table Level larger than {}", PAGE_TABLE_LEVEL);
        }
        let mut vpn = self.0;
        while level > 0 {
            vpn = vpn >> SV39_VPN_BIT;
            level = level - 1;
        }
        vpn & ((1 << SV39_VPN_BIT) - 1)
    }
}

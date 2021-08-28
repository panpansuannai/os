pub const PAGE_OFFSET_BIT: usize = 12;
pub const PAGE_SIZE: usize = 4096;
pub const SV39_VPN_BIT: usize = 9;
pub const PAGE_TABLE_LEVEL: usize = 3;

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
    pub fn page_offset(&self) -> usize {
        self.0 % PAGE_SIZE
    }
    pub fn offset(&self, off: isize) -> Self{
        Self((self.0 as isize + off) as usize)
    }
}
impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1 ) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 % PAGE_SIZE
    }
    pub fn offset(&self, off: isize) -> Self{
        Self((self.0 as isize + off) as usize)
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(physaddr: PhysAddr) -> Self{
        physaddr.floor()
    }
}

impl From<usize> for PhysPageNum {
    fn from(u: usize) -> Self{
        PhysPageNum::from(PhysAddr(u))
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(page_num: PhysPageNum) -> Self{
        if page_num.0 >= (1 << PAGE_TABLE_LEVEL * SV39_VPN_BIT - 1) {
            return PhysAddr((page_num.0  
                               | (usize::max_value()) << (PAGE_TABLE_LEVEL * SV39_VPN_BIT)) 
                               << PAGE_OFFSET_BIT)
        }
        PhysAddr(page_num.0 << PAGE_OFFSET_BIT)
    }
}

impl From<VirtualAddr> for VirtualPageNum {
    fn from(vaddr: VirtualAddr) -> Self{
        vaddr.floor()
    }
}

impl From<VirtualPageNum> for VirtualAddr {
    fn from(page_num: VirtualPageNum) -> Self{
        if page_num.0 >= (1 << PAGE_TABLE_LEVEL * SV39_VPN_BIT - 1) {
            return VirtualAddr((page_num.0  
                               | (usize::max_value()) << (PAGE_TABLE_LEVEL * SV39_VPN_BIT)) 
                               << PAGE_OFFSET_BIT)
        }
        VirtualAddr(page_num.0 << PAGE_OFFSET_BIT)
    }
}
impl From<usize> for VirtualAddr {
    fn from(addr: usize) -> Self {
        VirtualAddr(addr)
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
    pub fn offset(&self, off: usize) -> PhysAddr {
        let addr : PhysAddr = PhysAddr::from(self.clone());
        addr.offset(off as isize)
    }
}

/// SV39:
/// ------------------------------
/// 0000... | vpn2 | vpn1 | vpn0 |
/// ------------------------------
impl VirtualPageNum {
    pub fn vpn_block_sv39(&self, level: usize) -> usize {
        if level >= PAGE_TABLE_LEVEL {
            panic!("Page Table Level larger than {}", PAGE_TABLE_LEVEL);
        }
        let vpn = self.0 >> (SV39_VPN_BIT * level);
        vpn & ((1 << SV39_VPN_BIT) - 1)
    }
    pub fn offset(&self, off: usize) -> VirtualAddr{
        let addr : VirtualAddr = VirtualAddr::from(self.clone());
        addr.offset(off as isize)
    }
    pub const fn highest_page() -> Self {
        VirtualPageNum((1 << (PAGE_TABLE_LEVEL * SV39_VPN_BIT)) - 1)
    }
}

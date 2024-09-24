use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags: u16 {
        const BRANCH = 0x1;
        const LEAF = 0x2;
        const OVERFLOW = 0x3;
        const META = 0x8;
        const DIRTY = 0x10;
        const LEAF2 = 0x20;
        const SUB = 0x40;
        const LOOSE = 0x4000;
        const KEEP = 0x8000;
    }
}

#[derive(Debug)]
pub struct Header {
    pub pageno: u64,
    pub pad: u16,
    pub flags: Flags,
    pub free_lower: u16,
    pub free_upper: u16,
}

#[derive(Debug)]
pub struct Header2 {
    pub pageno: u64,
    pub pad: u16,
    pub flags: Flags,
    pub free_lower: u16,
    pub free_upper: u16,
    pub ptrs: Vec<usize>,
}

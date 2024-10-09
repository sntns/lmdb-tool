use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags: u16 {
        const REVERSEKEY = 0x02;
        const DUPSORT = 0x04;
        const INTEGERKEY = 0x08;
        const DUPFIXED = 0x10;
        const INTEGERDUP = 0x20;
        const REVERSEDUP = 0x40;
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub magic: u32,
    pub version: u32,
    pub address: u64,
    pub mapsize: u64,
    pub main: Database,
    pub free: Database,
    pub last_pgno: u64,
    pub txnid: u64,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub pad: u32,
    pub flags: Flags,
    pub depth: u16,
    pub branch_pages: u64,
    pub leaf_pages: u64,
    pub overflow_pages: u64,
    pub entries: u64,
    pub root: Option<u64>,
}

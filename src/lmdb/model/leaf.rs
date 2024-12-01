use bitflags::bitflags;
use core::fmt;

use super::header::Flags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NodeFlags: u16 {
        const BIGDATA = 0x1;
        const SUBDATA = 0x2;
        const DUPDATA = 0x4;
    }
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Data(Vec<u8>),
    Overflow(u64, usize),
}

#[derive(Clone)]
pub struct Node {
    pub flags: NodeFlags,
    pub key: Vec<u8>,
    pub data: NodeData,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_s: String = self.key.iter().map(|&c| c as char).collect();
        match self.data {
            NodeData::Data(ref data) => {
                let data_s: String = data.iter().map(|&c| c as char).collect();
                f.debug_struct("DataNode")
                    .field("flags", &self.flags)
                    .field("key", &key_s)
                    .field("data", &data_s)
                    .finish()
            }
            NodeData::Overflow(overflow, size) => f
                .debug_struct("OverflowNode")
                .field("flags", &self.flags)
                .field("key", &key_s)
                .field("overflow-page", &overflow)
                .field("data-size", &size)
                .finish(),
        }
    }
}

impl Node {
    pub fn size(&self) -> usize {
        let data_len = match self.data {
            NodeData::Data(ref data) => data.len(),
            NodeData::Overflow(_, _) => 4, // FIXME: wordsize
        };
        4 /* data_len */ + 2 /* flags */ + 2 /* key */
            + self.key.len() + data_len
    }
}

#[derive(Debug, Clone)]
pub struct Leaf {
    pub pageno: usize,
    pub flags: Flags,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Overflow {
    pub pageno: u64,
    pub data: Vec<u8>,
}

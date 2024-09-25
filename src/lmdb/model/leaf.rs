use core::fmt;

use super::header::Flags;

#[derive(Clone)]
pub struct Node {
    pub flags: u16,
    pub key: Vec<u8>,
    pub data: Vec<u8>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_s: String = self.key.iter().map(|&c| c as char).collect();
        let data_s: String = self.data.iter().map(|&c| c as char).collect();

        f.debug_struct("Node")
            .field("flags", &self.flags)
            .field("key", &key_s)
            .field("data", &data_s)
            .finish()
    }
}

impl Node {
    pub fn size(&self) -> usize {
        4 /* data_len */ + 2 /* flags */ + 2 /* key */
            + self.key.len() + self.data.len()
    }
}

#[derive(Debug, Clone)]
pub struct Leaf {
    pub pageno: usize,
    pub flags: Flags,
    pub nodes: Vec<Node>,
}

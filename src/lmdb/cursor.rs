use super::database::Database;
use super::error::Error;
use super::model;
use super::model::Node;

use error_stack::Result;

pub struct ReadCursor<'a, 'b> {
    pub db: &'b mut Database<'a>,
    pub page: Option<model::Leaf>,
    pub node_idx: usize,
}

impl<'a, 'b> ReadCursor<'a, 'b> {
    pub fn init(db: &'b mut Database<'a>) -> Result<Self, Error> {
        let mut cur = ReadCursor {
            db,
            page: None,
            node_idx: 0,
        };
        cur.next_page()?;
        Ok(cur)
    }

    pub fn next_page(&mut self) -> Result<(), Error> {
        let idx = match &self.page {
            Some(page) => page.pageno + 1,
            None => self.db.meta.main.root.unwrap_or(2 as u64) as usize,
        };
        self.page = if idx > self.db.meta.last_pgno as usize {
                None
            } else {
                self.node_idx = 0;
                self.db.read(idx).ok()
            };
        Ok(())
    }

    pub fn next(&mut self) -> Result<Option<Node>, Error> {
        let node = match &self.page {
            Some(page) => {
                let node = &page.nodes[self.node_idx];
                Ok(Some(node.clone()))
            }
            None => Ok(None),
        };

        // Try to move next
        if let Some(page) = &self.page {
            self.node_idx += 1;
            if page.nodes.len() == self.node_idx {
                self.next_page()?
            }
        }

        node
    }
}

pub struct WriteCursor<'a, 'b> {
    pub db: &'b mut Database<'a>,
    pub page: model::Leaf,
}

impl<'a, 'b> WriteCursor<'a, 'b> {
    pub fn init(db: &'b mut Database<'a>) -> Result<Self, Error> {
        let last = db.meta.last_pgno as usize;
        let page = match db.reader {
            Some(ref reader) => {
                let mut reader = reader.lock().unwrap();
                Database::seek_page_unsafe(reader.as_mut(), last)?;
                Database::read_leaf_unsafe(reader.as_mut())?
            }
            None => model::Leaf {
                pageno: if last > 1 { last + 1 } else { 2 },
                flags: model::header::Flags::LEAF,
                nodes: Vec::<model::Node>::new(),
            },
        };
        let cur = WriteCursor { db, page };
        Ok(cur)
    }

    pub fn push(&mut self, key: Vec<u8>, data: Vec<u8>) -> Result<(), Error> {
        let node = model::Node {
            flags: 0,
            key: key.clone(),
            data: data.clone(),
        };
        self.push_node(node)
    }

    pub fn push_node(&mut self, node: model::Node) -> Result<(), Error> {
        let size = self
            .page
            .nodes
            .iter()
            .map(|node| node.size())
            .reduce(|a, b| a + b)
            .unwrap_or(0);
        if size + node.size() >= 4096 - 6 * (self.page.nodes.len() + 1) {
            let mut writer = self.db.writer.as_ref().unwrap().lock().unwrap();
            tracing::debug!("Writing page: {:#?}", self.page);
            Database::write_leaf_unsafe(writer.as_mut(), self.page.clone())?;
            self.db.meta.last_pgno = self.page.pageno as u64;
            self.db.meta.main.entries += self.page.nodes.len() as u64;
            self.db.meta.main.leaf_pages += 1;
            self.db.meta.main.depth = 1;
            self.db.meta.main.root = Some(self.db.meta.main.root.unwrap_or(self.page.pageno as u64));
            self.page = model::Leaf {
                pageno: self.page.pageno + 1,
                flags: model::header::Flags::LEAF,
                nodes: Vec::<model::Node>::new(),
            };
        }
        self.page.nodes.push(node);

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        let mut writer = self.db.writer.as_ref().unwrap().lock().unwrap();
        tracing::debug!("Writing page: {:#?}", self.page);
        Database::write_leaf_unsafe(writer.as_mut(), self.page.clone())?;
        let mut meta = self.db.meta.clone();
        meta.last_pgno = self.page.pageno as u64;
        meta.txnid += 1;
        meta.main.entries += self.page.nodes.len() as u64;
        meta.main.leaf_pages += 1;
        meta.main.depth = 1;
        meta.main.root = Some(meta.main.root.unwrap_or(self.page.pageno as u64));
        tracing::debug!("Output: {:#?}", meta);
        Database::write_meta_unsafe(writer.as_mut(), meta, (self.db.meta_id + 1) % 2)?;
        writer.flush()?;
        self.page = model::Leaf {
            pageno: self.page.pageno + 1,
            flags: model::header::Flags::LEAF,
            nodes: Vec::<model::Node>::new(),
        };
        Ok(())
    }
}

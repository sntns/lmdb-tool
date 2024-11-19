use core::net;

use error_stack::Report;
use error_stack::Result;
use error_stack::ResultExt;

use super::database::Database;
use super::database::DatabaseReader;

use super::database::DatabaseWriter;
use super::error::Error;

use super::model;
use super::model::lowlevel;
use super::model::metadata;

impl<'a> Database<'a> {
    pub(super) fn init_meta_unsafe() -> Result<(model::Metadata, model::Metadata), Error> {
        let meta = model::Metadata {
            magic: lowlevel::MAGIC,
            version: lowlevel::VERSION,
            address: 0,
            mapsize: 1048576, // Do know what this is
            main: model::Database {
                pad: 4096,
                flags: model::metadata::Flags::empty(),
                depth: 0,
                branch_pages: 0,
                leaf_pages: 0,
                overflow_pages: 0,
                entries: 0,
                root: None,
            },
            free: model::Database {
                pad: 4096,
                flags: model::metadata::Flags::INTEGERKEY,
                depth: 0,
                branch_pages: 0,
                leaf_pages: 0,
                overflow_pages: 0,
                entries: 0,
                root: None,
            },
            last_pgno: 1,
            txnid: 0,
        };
        Ok((meta.clone(), meta.clone()))
    }

    pub(super) fn write_page_header_unsafe<'b>(
        writer: &'b mut (dyn DatabaseWriter + 'a),
        header: model::Header,
    ) -> Result<(), Error> {
        writer.write_word(header.pageno)?;
        writer.write_u16(header.pad)?;
        writer.write_u16(header.flags.bits())?;
        writer.write_u16(header.free_lower)?;
        writer.write_u16(header.free_upper)?;
        Ok(())
    }

    pub(super) fn write_db_unsafe<'b>(
        writer: &'b mut (dyn DatabaseWriter + 'a),
        db: metadata::Database,
    ) -> Result<(), Error> {
        writer.write_u32(db.pad)?;
        writer.write_u16(db.flags.bits())?;
        writer.write_u16(db.depth)?;
        writer.write_word(db.branch_pages)?;
        writer.write_word(db.leaf_pages)?;
        writer.write_word(db.overflow_pages)?;
        writer.write_word(db.entries)?;
        writer.write_opt_word(db.root)?;
        Ok(())
    }

    pub(super) fn write_overflow_unsafe<'b>(
        writer: &'b mut (dyn DatabaseWriter + 'a),
        overflow: model::Overflow,
    ) -> Result<(), Error> {
        writer.seek(std::io::SeekFrom::Start((overflow.pageno as u64) * 4096))?;
        let head = writer.pos()?;
        tracing::debug!("overflow pos: {}", head);

        writer.write_word(overflow.pageno as u64)?;
        writer.write_u16(0)?;
        writer.write_u16(model::header::Flags::OVERFLOW.bits())?;
        writer.write_u16(0)?;
        writer.write_u16(0)?;
        
        writer.write_exact(&overflow.data)?;

        let tail = writer.pos()?;
        if tail > head {
            let fill = 4096 - (tail - head);
            writer.write_fill(fill)?;
        }
        Ok(())
    }

    pub(super) fn write_leaf_unsafe<'b>(
        writer: &'b mut (dyn DatabaseWriter + 'a),
        leaf: model::Leaf,
    ) -> Result<(), Error> {
        writer.seek(std::io::SeekFrom::Start((leaf.pageno as u64) * 4096))?;

        let head = writer.pos()?;
        tracing::debug!("leaf pos: {}", head);

        let nkeys = leaf.nodes.len();

        let mut nodes = leaf.nodes.clone();
        nodes.sort_by(|a, b| a.key.cmp(&b.key));
        nodes.reverse();

        let mut ptrs = Vec::<usize>::new();
        let mut offset = 4096 - 1;
        for i in 0..nkeys {
            let node = &nodes[i];
            offset -= 4 + 2 + 2 + node.key.len();
            match node.data {
                model::NodeData::Data(ref data) => offset -= data.len(),
                model::NodeData::Overflow(_, _) => offset -= writer.word_size(),
            };
            ptrs.push(offset);
        }
        ptrs.reverse();
        tracing::debug!("leaf nkeys: {}, ptrs: {:?}", nkeys, ptrs);

        writer.write_word(leaf.pageno as u64)?;
        writer.write_u16(0)?;
        writer.write_u16(leaf.flags.bits())?;
        let pos = writer.pos()?;

        let free_lower = ((nkeys << 1) + (pos - head + 4)) as u16;
        let free_upper = offset as u16;
        tracing::debug!("leaf free_lower: {}, free_upper: {}", free_lower, free_upper);

        writer.write_u16(free_lower)?;
        writer.write_u16(free_upper)?;
        for ptr in ptrs.clone() {
            writer.write_u16(ptr as u16)?;
        }

        let tail = writer.pos()?;
        if tail > head {
            let fill = 4096 - (tail - head);
            writer.write_fill(fill)?;
        }

        for i in 0..nkeys {
            let node = &nodes[i];
            let start = head + ptrs[nkeys-1-i];
            writer.seek(std::io::SeekFrom::Start(start as u64))?;

            match node.data {
                model::NodeData::Data(ref data) => {
                    tracing::debug!("Writing node @{}: key:{}B, data:{}B, flags:{:?}", start, node.key.len(), data.len(), node.flags);
                    writer.write_u32(data.len() as u32)?;
                    writer.write_u16(node.flags.bits())?;
                    writer.write_u16(node.key.len() as u16)?;
                    writer.write_exact(&node.key)?;
                    writer.write_exact(&data)?;
                    assert!(writer.pos()?==0 || writer.pos()? - start == 4 + 2 + 2 + data.len() + node.key.len());
                },
                model::NodeData::Overflow(overflow, size) => {
                    tracing::debug!("Writing overflow node @{}: key:{}B, overflow:{}, flags:{:?}", start, node.key.len(), overflow, node.flags);
                    writer.write_u32(size as u32)?;
                    writer.write_u16(node.flags.bits())?;
                    writer.write_u16(node.key.len() as u16)?;
                    writer.write_exact(&node.key)?;
                    writer.write_word(overflow)?;
                    assert!(writer.pos()?==0 || writer.pos()? - start == 4 + 2 + 2 + writer.word_size() + node.key.len());
                },
            }
        }

        Ok(())
    }

    pub(super) fn write_meta_unsafe<'b>(
        writer: &'b mut (dyn DatabaseWriter + 'a),
        meta: model::Metadata,
        pageno: usize,
    ) -> Result<(), Error> {
        let head = pageno * 4096;
        writer.seek(std::io::SeekFrom::Start(head as u64))?;
        Self::write_page_header_unsafe(
            writer,
            model::Header {
                pageno: 0,
                pad: 0,
                flags: model::header::Flags::META,
                free_lower: 0,
                free_upper: 0,
            },
        )?;

        writer.write_u32(meta.magic)?;
        writer.write_u32(meta.version)?;
        writer.write_word(meta.address)?;
        writer.write_word(meta.mapsize)?;

        Self::write_db_unsafe(writer, meta.free)?;
        Self::write_db_unsafe(writer, meta.main)?;

        writer.write_word(meta.last_pgno)?;
        writer.write_word(meta.txnid)?;

        let tail = writer.pos()?;
        if tail > head {
            let fill = 4096 - (tail - head);
            writer.write_fill(fill)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile;

    use crate::lmdb::writer::Writer32;
    use crate::lmdb::writer::Writer64;

    use crate::lmdb::reader::Reader32;
    use crate::lmdb::reader::Reader64;

    use super::super::model;
    use super::*;

    pub fn init_tracing() -> tracing::subscriber::DefaultGuard {
        let subscriber = tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .with_file(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_default(subscriber)
    }

    #[test]
    fn test_write_meta_64() {
        let _guard = init_tracing();
        let file = tempfile::NamedTempFile::new().unwrap();
        let writer = std::io::BufWriter::new(file.reopen().unwrap());
        let mut writer = Writer64::from(writer);
        let dw = &mut writer;

        let (meta1, meta2) = Database::init_meta_unsafe().unwrap();
        Database::write_meta_unsafe(dw, meta1, 0).unwrap();
        Database::write_meta_unsafe(dw, meta2, 1).unwrap();
        writer.flush().unwrap();

        // Try to read back
        let file = file.reopen().unwrap();
        let reader = std::io::BufReader::new(file);
        let mut reader = Reader64::from(reader);
        let dr = &mut reader;

        let meta = Database::pick_meta_unsafe(dr).unwrap();
        tracing::debug!("Metadata: {:?}", meta);
    }

    #[test]
    fn test_write_leaf_64() {
        let _guard = init_tracing();
        let file = tempfile::NamedTempFile::new().unwrap();
        let writer = std::io::BufWriter::new(file.reopen().unwrap());
        let mut writer = Writer64::from(writer);
        let dw = &mut writer;

        let (meta1, meta2) = Database::init_meta_unsafe().unwrap();
        Database::write_meta_unsafe(dw, meta1, 0).unwrap();
        Database::write_meta_unsafe(dw, meta2, 1).unwrap();

        let mut nodes = Vec::<model::Node>::new();
        for i in 1..3 {
            nodes.push(model::Node {
                flags: model::NodeFlags::empty(),
                key: vec![i; 1],
                data: model::NodeData::Data(vec![2 * i; 1]),
            });
        }
        Database::write_leaf_unsafe(
            dw,
            model::Leaf {
                pageno: 2,
                flags: model::header::Flags::LEAF,
                nodes,
            },
        )
        .unwrap();
        writer.flush().unwrap();

        // Try to read back
        let file = file.reopen().unwrap();
        let reader = std::io::BufReader::new(file);
        let mut reader = Reader64::from(reader);
        let dr = &mut reader;

        Database::seek_page_unsafe(dr, 2).unwrap();
        let leaf = Database::read_leaf_unsafe(dr).unwrap();
        tracing::debug!("{:#?}", leaf);
    }

    #[test]
    fn test_write_meta_32() {
        let _guard = init_tracing();
        let file = tempfile::NamedTempFile::new().unwrap();
        let writer = std::io::BufWriter::new(file.reopen().unwrap());
        let mut writer = Writer32::from(writer);
        let dw = &mut writer;

        let (meta1, meta2) = Database::init_meta_unsafe().unwrap();
        Database::write_meta_unsafe(dw, meta1, 0).unwrap();
        Database::write_meta_unsafe(dw, meta2, 1).unwrap();

        writer.flush().unwrap();

        // Try to read back
        let file = file.reopen().unwrap();
        let reader = std::io::BufReader::new(file);
        let mut reader = Reader32::from(reader);
        let dr = &mut reader;

        let meta = Database::pick_meta_unsafe(dr).unwrap();
        tracing::debug!("Metadata: {:?}", meta);
    }
}

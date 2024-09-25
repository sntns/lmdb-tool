
use error_stack::Report;
use error_stack::Result;
use error_stack::ResultExt;

use super::database::Database;
use super::database::DatabaseReader;

use super::error::Error;

use super::model::lowlevel;
use super::model;
use super::model::Leaf;

impl<'a> Database<'a> {
    pub(super) fn read_page_header_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<model::Header, Error> {
        /* MDB_page struct */
        let pageno = reader.read_word()?;
        let pad = reader.read_u16()?;
        let flags = reader.read_u16()?;
        let free_lower = reader.read_u16()?;
        let free_upper = reader.read_u16()?;
        let header = model::Header { 
            pageno: pageno, 
            pad, 
            flags: model::header::Flags::from_bits_retain(flags),
            free_lower,
            free_upper,
        };
        tracing::debug!("Page header: {:?}", header);
        Ok(header)
    }


    pub(super) fn read_meta_db_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<model::Database, Error> {
        let pad = reader.read_u32()?;
        let flags = reader.read_u16()?;
        let depth = reader.read_u16()?;
        let branch_pages = reader.read_word()?;
        let leaf_pages = reader.read_word()?;
        let overflow_pages = reader.read_word()?;
        let entries = reader.read_word()?;
        let root = reader.read_word()?;
        
        let db = model::Database {
            pad,
            flags: model::metadata::Flags::from_bits_retain(flags),
            depth,
            branch_pages,
            leaf_pages,
            overflow_pages,
            entries,
            root,
        };
        Ok(db)
    }

    pub(super) fn read_page_header2_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<model::Header2, Error> {
        /* MDB_page2 struct */
        let pos = reader.pos()?;
        let pageno = reader.read_word()?;
        let pad = reader.read_u16()?;
        let flags = reader.read_u16()?;
        let free_lower = reader.read_u16()?;
        let free_upper = reader.read_u16()?;
        let page_header_size = (reader.pos()? - pos) as u16;

        let nkeys = (free_lower - page_header_size) >> 1;
        let mut ptrs = vec![0x0 ; nkeys as usize];
        for i in 0..nkeys {
            ptrs[i as usize] = reader.read_u16()? as usize;
        }
        let header = model::Header2 { 
            pageno: pageno, 
            pad, 
            flags: model::header::Flags::from_bits_retain(flags),
            free_lower,
            free_upper,
            ptrs,
        };
        tracing::debug!("Page header2: {:?}", header);
        Ok(header)
    }

    pub(super) fn seek_page_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a), page: usize) -> Result<(), Error> {
        reader.seek(std::io::SeekFrom::Start((page * 4096) as u64))?;
        Ok(())
    }

    pub(super) fn read_meta_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<model::Metadata, Error> {
        let header = Self::read_page_header_unsafe(reader)?;
        if header.flags & model::header::Flags::META != model::header::Flags::META {
            return Err(Report::new(Error::InvalidFileFormat)
                .attach_printable("Not a meta page")
            );
        }

        /* MDB_meta */
        let magic = reader.read_u32()?;
        if magic != lowlevel::MAGIC {
            return Err(Report::new(Error::InvalidFileFormat)
                .attach_printable("Invalid magic number")
            );
        }
        let version = reader.read_u32()?;
        if version != lowlevel::VERSION {
            return Err(Report::new(Error::VersionNotSupported)
                .attach_printable(format!("Version not supported: {}", version))
            );
        }

        let address = reader.read_word()?;
        let mapsize = reader.read_word()?;
        
        /* MDB_db */
        let free = Self::read_meta_db_unsafe(reader)?;
        let main = Self::read_meta_db_unsafe(reader)?;
        
        let last_pgno = reader.read_word()?;
        let txnid = reader.read_word()?;
        let metadata = model::Metadata {
            magic,
            version,
            address,
            mapsize,
            main,
            free,
            last_pgno,
            txnid,
        };
        tracing::debug!("Metadata: {:?}", metadata);
        Ok(metadata)
    }    
    
    pub(super) fn read_leaf_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<model::Leaf, Error> {
        
        let start = reader.pos()?;
        let header = Self::read_page_header2_unsafe(reader)?;
        if header.flags & model::header::Flags::LEAF != model::header::Flags::LEAF {
            return Err(Report::new(Error::InvalidFileFormat)
                .attach_printable("Not a leaf page")
            );
        }

        let mut nodes = Vec::<_>::new();
        for i in 0..header.ptrs.len() {
            reader.seek(std::io::SeekFrom::Start((start + header.ptrs[i]) as u64))?;
            
            let size = reader.read_u32()?;
            let flags = reader.read_u16()?;
            let ksize = reader.read_u16()?;

            let mut key = vec![0u8; ksize as usize];
            reader.read_exact(&mut key)?;
            let mut data = vec![0u8; size as usize];
            reader.read_exact(&mut data)?;
            
            nodes.push(model::Node {
                flags,
                key,
                data,
            });
        }

        let leaf = model::Leaf {
            pageno: header.pageno as usize,
            flags: header.flags,
            nodes,
        };
        tracing::debug!("{:#?}", leaf);

        Ok(leaf)
    }

    pub(super) fn pick_meta_unsafe<'b>(reader: &'b mut (dyn DatabaseReader + 'a)) -> Result<(model::Metadata, usize), Error> {
        // Read the first metadata
        Self::seek_page_unsafe(reader, 0)?;
        let meta1 = Self::read_meta_unsafe(reader)?;

        // And the second metadata
        Self::seek_page_unsafe(reader, 1)?;
        let meta2 = Self::read_meta_unsafe(reader)?;

        if meta1.txnid < meta2.txnid {
            Ok((meta2, 1))
        } else {
            Ok((meta1, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lmdb::reader::{Reader32, Reader64};
    use super::*;

    macro_rules! test_case {
        ($fname:expr) => {
            std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/",
                $fname
            ))
        };
    }

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
    fn test_read_meta_64() {
        let _guard = init_tracing();
        let file = std::fs::File::open(test_case!("mender-store.64bits")).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut reader = Reader64::from(reader);
        let dr = &mut reader;

        let (meta, _) = Database::pick_meta_unsafe(dr).unwrap();
        tracing::debug!("Metadata: {:?}", meta);
        
        for i in 2..(meta.last_pgno as usize)+1 {
            Database::seek_page_unsafe(dr, i).unwrap();
            Database::read_leaf_unsafe(dr).unwrap();
        }
    }

    #[test]
    fn test_read_meta_32() {
        let _guard = init_tracing();
        let file = std::fs::File::open(test_case!("mender-store.32bits")).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut reader = Reader32::from(reader);
        let dr = &mut reader;

        let (meta, _) = Database::pick_meta_unsafe(dr).unwrap();
        tracing::debug!("Metadata: {:?}", meta);
        
        for i in 2..(meta.last_pgno as usize)+1 {
            Database::seek_page_unsafe(dr, i).unwrap();
            Database::read_leaf_unsafe(dr).unwrap();
        }
    }
}
            
use std::sync::Mutex;
use std::vec;

use byteorder;

use error_stack::Report;
use error_stack::Result;
use error_stack::ResultExt;

use super::model;
use super::model::lowlevel;
use super::cursor::ReadCursor;
use super::cursor::WriteCursor;

use super::error::Error;

pub trait DatabaseReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error>;
    fn pos(&mut self) -> Result<usize, Error> {
        self.seek(std::io::SeekFrom::Current(0))
    }
    fn read_word(&mut self) -> Result<u64, Error>;
    fn read_u16(&mut self) -> Result<u16, Error>;
    fn read_u32(&mut self) -> Result<u32, Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}

pub trait DatabaseWriter {
    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error>;
    fn pos(&mut self) -> Result<usize, Error> {
        self.seek(std::io::SeekFrom::Current(0))
    }
    fn write_word(&mut self, n: u64) -> Result<(), Error>;
    fn write_u16(&mut self, n: u16) -> Result<(), Error>;
    fn write_u32(&mut self, n: u32) -> Result<(), Error>;
    fn write_exact(&mut self, buf: &[u8]) -> Result<(), Error>;
    fn write_fill(&mut self, n: usize) -> Result<(), Error> {
        let buf = vec![0 as u8;n];
        Ok(self.write_exact(&buf)
            .change_context(Error::WriteError)?)
    }
    fn flush(&mut self) -> Result<(), Error>;
}

pub struct Database<'a> {
    pub(crate) reader: Option<Mutex<Box<dyn DatabaseReader + 'a>>>,
    pub(crate) writer: Option<Mutex<Box<dyn DatabaseWriter + 'a>>>,
    pub(crate) meta_id: usize,
    pub(crate) meta: model::Metadata,
}

impl<'a> Database<'a> {
    pub fn read_from<DR>(mut reader: DR) -> Result<Self, Error> 
    where DR: DatabaseReader + 'a 
    {
        let rdr: &mut (dyn DatabaseReader + 'a) = &mut reader;
        let (meta,meta_id) = Self::pick_meta_unsafe(rdr)?;

        Ok(Self { 
            reader: Some(Mutex::new(Box::new(reader))),
            writer: None,
            meta_id,
            meta,
        })
    }

    pub fn from_reader<DR, R>(reader: R) -> Result<Self, Error> 
    where R: std::io::Read + std::io::Seek,
          DR: DatabaseReader + From<R> + 'a {
        let reader = DR::from(reader);
        Self::read_from(reader)
    }

    pub fn write_from<DW>(mut writer: DW) -> Result<Self, Error> 
    where DW: DatabaseWriter + 'a 
    {
        let wtr: &mut (dyn DatabaseWriter + 'a) = &mut writer;
        let (meta1, meta2) = Self::init_meta_unsafe()?;
        Self::write_meta_unsafe(wtr, meta1.clone(), 0)?;
        Self::write_meta_unsafe(wtr, meta2.clone(), 1)?;
        
        Ok(Self { 
            reader: None,
            writer: Some(Mutex::new(Box::new(writer))),
            meta_id: 0,
            meta: meta1,
        })
    }

    pub fn from_writer<DW, W>(writer: W) -> Result<Self, Error> 
    where W: std::io::Write + std::io::Seek,
          DW: DatabaseWriter + From<W> + 'a {
        let writer = DW::from(writer);
        Self::write_from(writer)
    }

}


impl<'a> Database<'a> {
    pub fn read_cursor<'b>(&'b mut self) -> Result<ReadCursor<'a, 'b>, Error> {
        ReadCursor::init(self)
    }

    pub fn write_cursor<'b>(&'b mut self) -> Result<WriteCursor<'a, 'b>, Error> {
        WriteCursor::init(self)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::lmdb::Factory;
    use crate::lmdb::WordSize;

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

    static INIT: Once = Once::new();

    pub fn setup() -> () { 
        INIT.call_once(|| {
            tracing_subscriber::fmt::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .init();
        });
    }

    #[test]
    fn test_read_64() {
        setup();
            
        let mut db = Factory::open(test_case!("mender-store.64bits")).unwrap();
        let mut cur = db.read_cursor().unwrap();
        let mut i = 0;
        while let Some(node) = cur.next().unwrap() {    
            tracing::debug!("#{}: {:#?}", i, node);
            i+=1;
        }
    }

    #[test]
    fn test_read_32() {
        setup();
            
        let mut db = Factory::open(test_case!("mender-store.32bits")).unwrap();
        let mut cur = db.read_cursor().unwrap();
        let mut i = 0;
        while let Some(node) = cur.next().unwrap() {    
            tracing::debug!("#{}: {:#?}", i, node);
            i+=1;
        }
    }

    #[test]
    fn test_write_32() {
        setup();
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut db = Factory::create(file.path().into(), WordSize::Word32).unwrap();
        let mut cur = db.write_cursor().unwrap();
        cur.push(vec![1;1], vec![2;2]).unwrap();
        cur.commit().unwrap();

        tracing::info!("Reading back {:?}", file.path());

        let mut db = Factory::open(file.path().into()).unwrap();
        let mut cur = db.read_cursor().unwrap();
        let mut i = 0;
        while let Some(node) = cur.next().unwrap() {    
            tracing::debug!("#{}: {:#?}", i, node);
            i+=1;
        }
    }

    #[test]
    fn test_write_64() {
        setup();
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut db = Factory::create(file.path().into(), WordSize::Word64).unwrap();
        let mut cur = db.write_cursor().unwrap();
        cur.push(vec![1;1], vec![2;2]).unwrap();
        cur.commit().unwrap();

        tracing::info!("Reading back {:?}", file.path());

        let mut db = Factory::open(file.path().into()).unwrap();
        let mut cur = db.read_cursor().unwrap();
        let mut i = 0;
        while let Some(node) = cur.next().unwrap() {    
            tracing::debug!("#{}: {:#?}", i, node);
            i+=1;
        }
    }

    #[test]
    fn test_write_multi_page_64() {
        setup();
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut db = Factory::create(file.path().into(), WordSize::Word64).unwrap();
        let mut cur = db.write_cursor().unwrap();

        for i in 0..4096 {
            cur.push(vec![(i%255) as u8;1], vec![(i%255) as u8;2]).unwrap();
        }
        cur.commit().unwrap();

        tracing::info!("Reading back {:?}", file.path());

        let mut db = Factory::open(file.path().into()).unwrap();
        let mut cur = db.read_cursor().unwrap();
        let mut i = 0;
        while cur.next().unwrap().is_some() {    
            i+=1;
        }
        assert_eq!(i, 4096);
        tracing::debug!("Total pages: {}", i);
        tracing::debug!("Metadata: {:?}", db.meta);
        assert_ne!(db.meta.last_pgno, 1);
        assert_eq!(db.meta.main.entries, 4096);


    }
}
            
use byteorder::ReadBytesExt;
use byteorder::LE;
use std::io::Seek;
use clap;

use error_stack::Report;
use error_stack::Result;
use error_stack::ResultExt;

use super::database::Database;
use super::error::Error;
use super::reader;
use super::writer;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSize {
    Word32,
    Word64,
}

impl From<String> for WordSize {
    fn from(s: String) -> Self {
        match s.as_str() {
            "32" => WordSize::Word32,
            "64" => WordSize::Word64,
            _ => panic!("Invalid word size"),
        }
    }
}

impl Into<u8> for WordSize {
    fn into(self) -> u8 {
        match self {
            WordSize::Word32 => 32,
            WordSize::Word64 => 64,
        }
    }
}

pub struct Factory;

impl Factory {
    pub fn detect(database: std::path::PathBuf) -> Result<WordSize, Error> {
        let file = std::fs::File::open(database.clone()).change_context(Error::ReadError)?;
        let mut rdr = std::io::BufReader::new(file);

        let pageno = rdr.read_u32::<LE>().change_context(Error::ReadError)?;
        let pad = rdr.read_u16::<LE>().change_context(Error::ReadError)?;
        let flags = rdr.read_u16::<LE>().change_context(Error::ReadError)?;

        if pageno == 0 && pad == 0 && flags == 0x8 {
            return Ok(WordSize::Word32);
        }

        rdr.seek(std::io::SeekFrom::Start(0))
            .change_context(Error::ReadError)?;
        let pageno = rdr.read_u64::<LE>().change_context(Error::ReadError)?;
        let pad = rdr.read_u16::<LE>().change_context(Error::ReadError)?;
        let flags = rdr.read_u16::<LE>().change_context(Error::ReadError)?;

        if pageno == 0 && pad == 0 && flags == 0x8 {
            return Ok(WordSize::Word64);
        }

        Err(Report::new(Error::InvalidFileFormat)
            .attach_printable("Neither 32bits nor 64bits page header found"))
    }

    pub fn open<'a>(database: std::path::PathBuf) -> Result<Database<'a>, Error> {
        let file = std::fs::File::open(database.clone()).change_context(Error::ReadError)?;
        let rdr = std::io::BufReader::new(file);

        match Self::detect(database.clone())? {
            WordSize::Word32 => Database::from_reader::<reader::Reader32<_>, _>(rdr),
            WordSize::Word64 => Database::from_reader::<reader::Reader64<_>, _>(rdr),
        }
    }

    pub fn create<'a>(database: std::path::PathBuf, s: WordSize) -> Result<Database<'a>, Error> {
        let file = std::fs::File::create(database.clone()).change_context(Error::WriteError)?;
        let wtr = std::io::BufWriter::new(file);

        match s {
            WordSize::Word32 => Database::from_writer::<writer::Writer32<_>, _>(wtr),
            WordSize::Word64 => Database::from_writer::<writer::Writer64<_>, _>(wtr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_case {
        ($fname:expr) => {
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/", $fname))
        };
    }

    #[test]
    fn test_detect_bitsize() {
        let database = test_case!("mender-store.32bits");
        let size = Factory::detect(database).unwrap();
        assert_eq!(size, WordSize::Word32);

        let database = test_case!("mender-store.64bits");
        let size = Factory::detect(database).unwrap();
        assert_eq!(size, WordSize::Word64);
    }
}

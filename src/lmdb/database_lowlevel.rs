use error_stack::Result;
use error_stack::ResultExt;

use super::database::Database;

use super::error::Error;

use super::model::Leaf;

impl<'a> Database<'a> {
    pub fn read(&mut self, page: usize) -> Result<Leaf, Error> {
        let reader = self.reader.as_mut().ok_or(Error::NoReader)?;
        let reader = reader.get_mut().unwrap();
        Self::seek_page_unsafe(reader.as_mut(), page)?;
        Self::read_leaf_unsafe(reader.as_mut())
            .attach_printable(format!("failed to read page {}", page))
    }

    pub fn read_overflow(&mut self, page: usize, size: usize) -> Result<Vec<u8>, Error> {
        let reader = self.reader.as_mut().ok_or(Error::NoReader)?;
        let reader = reader.get_mut().unwrap();
        Self::seek_page_unsafe(reader.as_mut(), page)?;
        Self::read_overflow_unsafe(reader.as_mut(), size)
            .attach_printable(format!("failed to read overflow page {}", page))
    }
}

#[cfg(test)]
mod tests {
    use crate::lmdb::reader::Reader32;
    use crate::lmdb::reader::Reader64;

    use super::*;

    macro_rules! test_case {
        ($fname:expr) => {
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/", $fname))
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

        for i in 2..(meta.last_pgno as usize) + 1 {
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

        for i in 2..(meta.last_pgno as usize) + 1 {
            Database::seek_page_unsafe(dr, i).unwrap();
            Database::read_leaf_unsafe(dr).unwrap();
        }
    }
}

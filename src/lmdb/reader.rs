use byteorder;
use byteorder::LittleEndian;

use error_stack::Result;
use error_stack::ResultExt;

use super::error::Error;
use super::database::DatabaseReader;

#[derive(Debug)]
pub struct Reader32<R> where R: byteorder::ReadBytesExt + std::io::Seek {
    reader: R,
}

impl<R> From<R> for Reader32<R> where R: byteorder::ReadBytesExt + std::io::Seek {
    fn from(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> DatabaseReader for Reader32<R> where R: byteorder::ReadBytesExt + std::io::Seek {

    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error> {
        Ok(self.reader.seek(pos)
            .change_context(Error::ReadError)? as usize)
    }

    fn read_word(&mut self) -> Result<u64, Error> {
        Ok(self.reader.read_u32::<LittleEndian>()
            .change_context(Error::ReadError)? as u64)
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        Ok(self.reader.read_u16::<LittleEndian>()
            .change_context(Error::ReadError)?)
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(self.reader.read_u32::<LittleEndian>()
            .change_context(Error::ReadError)?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        Ok(self.reader.read_exact(buf)
            .change_context(Error::ReadError)?)
    }
}


pub struct Reader64<R> where R: byteorder::ReadBytesExt + std::io::Seek {
    reader: R,
}

impl<R> From<R> for Reader64<R> where R: byteorder::ReadBytesExt + std::io::Seek {
    fn from(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> DatabaseReader for Reader64<R> where R: byteorder::ReadBytesExt + std::io::Seek {

    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error> {
        Ok(self.reader.seek(pos)
            .change_context(Error::ReadError)? as usize)
    }

    fn read_word(&mut self) -> Result<u64, Error> {
        Ok(self.reader.read_u64::<LittleEndian>()
            .change_context(Error::ReadError)?)
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        Ok(self.reader.read_u16::<LittleEndian>()
            .change_context(Error::ReadError)?)
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(self.reader.read_u32::<LittleEndian>()
            .change_context(Error::ReadError)?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        Ok(self.reader.read_exact(buf)
            .change_context(Error::ReadError)?)
    }
}
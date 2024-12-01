use byteorder;
use byteorder::LittleEndian;

use error_stack::Result;
use error_stack::ResultExt;

use super::database::DatabaseWriter;
use super::error::Error;

#[derive(Debug)]
pub struct Writer32<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    writer: W,
}

impl<W> From<W> for Writer32<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    fn from(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> DatabaseWriter for Writer32<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    fn word_size(&self) -> usize {
        4
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error> {
        Ok(self.writer.seek(pos).change_context(Error::WriteError)? as usize)
    }

    fn write_word(&mut self, n: u64) -> Result<(), Error> {
        self.writer
            .write_u32::<LittleEndian>(n as u32)
            .change_context(Error::WriteError)
    }

    fn write_opt_word(&mut self, n: Option<u64>) -> Result<(), Error> {
        match n {
            Some(n) => self
                .writer
                .write_u32::<LittleEndian>(n as u32)
                .change_context(Error::WriteError),
            None => self
                .writer
                .write_i32::<LittleEndian>(-1)
                .change_context(Error::WriteError),
        }
    }

    fn write_u16(&mut self, n: u16) -> Result<(), Error> {
        self.writer
            .write_u16::<LittleEndian>(n)
            .change_context(Error::WriteError)
    }

    fn write_u32(&mut self, n: u32) -> Result<(), Error> {
        self.writer
            .write_u32::<LittleEndian>(n)
            .change_context(Error::WriteError)
    }

    fn write_exact(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.writer.write_all(buf).change_context(Error::WriteError)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush().change_context(Error::WriteError)
    }
}

pub struct Writer64<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    writer: W,
}

impl<W> From<W> for Writer64<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    fn from(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> DatabaseWriter for Writer64<W>
where
    W: byteorder::WriteBytesExt + std::io::Seek,
{
    fn word_size(&self) -> usize {
        8
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<usize, Error> {
        Ok(self.writer.seek(pos).change_context(Error::WriteError)? as usize)
    }

    fn write_word(&mut self, n: u64) -> Result<(), Error> {
        self.writer
            .write_u64::<LittleEndian>(n)
            .change_context(Error::WriteError)
    }

    fn write_opt_word(&mut self, n: Option<u64>) -> Result<(), Error> {
        match n {
            Some(n) => self
                .writer
                .write_u64::<LittleEndian>(n)
                .change_context(Error::WriteError),
            None => self
                .writer
                .write_i64::<LittleEndian>(-1)
                .change_context(Error::WriteError),
        }
    }

    fn write_u16(&mut self, n: u16) -> Result<(), Error> {
        self.writer
            .write_u16::<LittleEndian>(n)
            .change_context(Error::WriteError)
    }

    fn write_u32(&mut self, n: u32) -> Result<(), Error> {
        self.writer
            .write_u32::<LittleEndian>(n)
            .change_context(Error::WriteError)
    }

    fn write_exact(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.writer.write_all(buf).change_context(Error::WriteError)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush().change_context(Error::WriteError)
    }
}

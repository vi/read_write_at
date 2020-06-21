//! TODO

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::io::{Read,Write,Seek,SeekFrom,Result,Error,ErrorKind};

/// Generalisation of [`std::os::unix::fs::FileExt`](https://doc.rust-lang.org/stable/std/os/unix/fs/trait.FileExt.html)
pub trait ReadAt {
    /// Reads a number of bytes starting from a given offset.
    /// Returns the number of bytes read.
    /// The offset is relative to the start of the file and thus independent from the current cursor.
    /// If type thta implements trait has concept of a cursor then it should not be affected by this function.
    /// Short reads are not considered as errors.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    /// Similar to `read_at`, but without short reads.
    // Implementation is copied from `https://doc.rust-lang.org/stable/src/std/sys/unix/ext/fs.rs.html` in 2020-06-22.
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }
}
/// Generalisation of 
pub trait ReadAtMut {
    /// TODO
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize>;

    /// Similar to `read_at`, but without short reads.
    // Implementation is copied from `https://doc.rust-lang.org/stable/src/std/sys/unix/ext/fs.rs.html` in 2020-06-22.
    fn read_exact_at(&mut self, mut buf: &mut [u8], mut offset: u64) -> Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }
}

impl<T: ReadAt> ReadAtMut for T{ 
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize> {
        ReadAt::read_at(self, buf, offset)
    }
}
/// TODO
pub trait WriteAt {
    /// TODO
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    /// Similar to `write_at`, but without short writes. if entirety of the provided buffer cannot be written,
    /// an error is returned.
    // Implementation is copied from `https://doc.rust-lang.org/stable/src/std/sys/unix/ext/fs.rs.html` in 2020-06-22.
    fn write_all_at(&self, mut buf: &[u8], mut offset: u64) -> Result<()> {
        while !buf.is_empty() {
            match self.write_at(buf, offset) {
                Ok(0) => {
                    return Err(Error::new(
                        ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => {
                    buf = &buf[n..];
                    offset += n as u64
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
/// TODO
pub trait WriteAtMut {
    /// TODO
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize>;

    /// Similar to `write_at`, but without short writes. if entirety of the provided buffer cannot be written,
    /// an error is returned.
    // Implementation is copied from `https://doc.rust-lang.org/stable/src/std/sys/unix/ext/fs.rs.html` in 2020-06-22.
    fn write_all_at(&mut self, mut buf: &[u8], mut offset: u64) -> Result<()> {
        while !buf.is_empty() {
            match self.write_at(buf, offset) {
                Ok(0) => {
                    return Err(Error::new(
                        ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => {
                    buf = &buf[n..];
                    offset += n as u64
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl<T: WriteAt> WriteAtMut for T{ 
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        WriteAt::write_at(self, buf, offset)
    }
}

// cfg line is copied from https://doc.rust-lang.org/stable/src/std/os/mod.rs.html at 2020-06-22
#[cfg(any(target_os = "redox", unix, target_os = "vxworks", target_os = "hermit"))]
impl WriteAt for std::fs::File {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::write_at(self, buf, offset)
    }
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<()> {
        std::os::unix::fs::FileExt::write_all_at(self, buf, offset)
    }
}

// cfg line is copied from https://doc.rust-lang.org/stable/src/std/os/mod.rs.html at 2020-06-22
#[cfg(any(target_os = "redox", unix, target_os = "vxworks", target_os = "hermit"))]
impl ReadAt for std::fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::unix::fs::FileExt::read_at(self, buf, offset)
    }
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        std::os::unix::fs::FileExt::read_exact_at(self, buf, offset)
    }
}

#[cfg(windows)]
/// Note that cursor is affected. That why it's `WriteAtMut` instead of `WriteAt`
impl WriteAtMut for std::fs::File {
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_write(self, buf, offset)
    }
}
#[cfg(windows)]
/// Note that cursor is affected. That why it's `ReadAtMut` instead of `ReadAt`
impl ReadAtMut for std::fs::File {
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize> {
        std::os::windows::fs::FileExt::seek_read(self, buf, offset)
    }
}

/// A wrapper that calls `Seek::seek` and `Read::read` or `Write::write` for each call of `read_at` or `write_at`
/// Can be used for read-only access as well.
pub struct ReadWriteSeek<T:Seek>(T);

impl<T:Read+Seek> ReadAtMut for ReadWriteSeek<T> {
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let o = Seek::seek(&mut self.0, SeekFrom::Start(offset))?;
        if o != offset {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "seek hasn't returned the required offset",
            ));
        }
        Read::read(&mut self.0, buf)
    }
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<()> {
        let o = Seek::seek(&mut self.0, SeekFrom::Start(offset))?;
        if o != offset {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "seek hasn't returned the required offset",
            ));
        }
        Read::read_exact(&mut self.0, buf)
    }
}

impl<T:Write+Seek> WriteAtMut for ReadWriteSeek<T> {
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        let o = Seek::seek(&mut self.0, SeekFrom::Start(offset))?;
        if o != offset {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "seek hasn't returned the required offset",
            ));
        }
        Write::write(&mut self.0, buf)
    }
    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<()> {
        let o = Seek::seek(&mut self.0, SeekFrom::Start(offset))?;
        if o != offset {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "seek hasn't returned the required offset",
            ));
        }
        Write::write_all(&mut self.0, buf)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

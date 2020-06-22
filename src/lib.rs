//! Abstraction of a file- or block derive-like object, data from/to which can be read/written at offsets.
//! 
//! There are alreay some analogues of those traits, including in libstd.
//! But they are either platform-specific or tied to implementation of some algorithm.
//! 
//! This crate focuses on abstraction itself, providing mostly wrappers and helper functions.
//! 
//! Traits are given in two varieties: with mutable `&mut self` and immutable `&self` methods.
//! 
//! libstd's platform-specific FileExt traits are forwarded for std::fs::File.
//! 
//! There is a generic wrapper for using `Read+Seek` or `Read+Write+Seek` objects
//! 
//! Immutable version of traits are implemented for `RefCell`s or `Mutex`s over mutable versions.
//! You may need to implement `std::ops::DerefMut` for your types to use them although.
//! 
//! TODO: vectored IO

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

impl<T: ReadAt+?Sized> ReadAtMut for T{ 
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

impl<T: WriteAt+?Sized> WriteAtMut for T{ 
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        WriteAt::write_at(self, buf, offset)
    }
}


/// A combined ReadAt and WriteAt for trait objects.
pub trait ReadWriteAt : ReadAt + WriteAt {}
impl<T:ReadAt+WriteAt> ReadWriteAt for T {}

/// A combined ReadAtMut and WriteAtMut for trait objects
pub trait ReadWriteAtMut : ReadAtMut + WriteAtMut {}
impl<T:ReadAtMut+WriteAtMut> ReadWriteAtMut for T {}


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

impl<T:Seek> std::ops::Deref for ReadWriteSeek<T> {
    type Target = Self;
    fn deref(&self) -> &Self {
        self
    }
}
impl<T:Seek> std::ops::DerefMut for ReadWriteSeek<T> {
    fn deref_mut(&mut self) -> &mut Self {
        self
    }
}


/// A wrapper struct to allow accessing `RefCell` and `Mutex` helper impls for trait objects.
pub struct DerefWrapper<T: std::ops::DerefMut> (T);

impl<T,U> ReadAtMut for DerefWrapper<U>
where T:ReadAtMut+?Sized, U: std::ops::DerefMut<Target = T>
{
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize> {
        ReadAtMut::read_at(std::ops::DerefMut::deref_mut(&mut self.0), buf, offset)
    }
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<()> {
        ReadAtMut::read_exact_at(std::ops::DerefMut::deref_mut(&mut self.0), buf, offset)
    }
}

impl<T,U> WriteAtMut for DerefWrapper<U>
where T:WriteAtMut+?Sized, U: std::ops::DerefMut<Target = T>
{
    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize> {
        WriteAtMut::write_at(std::ops::DerefMut::deref_mut (&mut self.0), buf, offset)
    }
    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<()> {
        WriteAtMut::write_all_at(std::ops::DerefMut::deref_mut(&mut self.0), buf, offset)
    }
}



impl<T> ReadAt for std::cell::RefCell<T> 
where T:ReadAtMut+?Sized
{
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let mut se = self.borrow_mut();
        ReadAtMut::read_at(&mut *se, buf, offset)
    }
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        let mut se = self.borrow_mut();
        ReadAtMut::read_exact_at(&mut *se, buf, offset)
    }
}

impl<T> WriteAt for std::cell::RefCell<T> 
where T:WriteAtMut+?Sized
{
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        let mut se = self.borrow_mut();
        WriteAtMut::write_at(&mut *se, buf, offset)
    }
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<()> {
        let mut se = self.borrow_mut();
        WriteAtMut::write_all_at(&mut *se, buf, offset)
    }
}



impl<T> ReadAt for std::sync::Mutex<T> 
where T:ReadAtMut+?Sized
{
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let se = self.lock();
        let mut se = match se {
            Ok(x) => x,
            Err(_) =>  return Err(Error::new(
                ErrorKind::Other,
                "poisoned mutex encountered",
            )),
        };
        ReadAtMut::read_at(&mut *se, buf, offset)
    }
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> Result<()> {
        let se = self.lock();
        let mut se = match se {
            Ok(x) => x,
            Err(_) =>  return Err(Error::new(
                ErrorKind::Other,
                "poisoned mutex encountered",
            )),
        };
        ReadAtMut::read_exact_at(&mut *se, buf, offset)
    }
}

impl<T> WriteAt for std::sync::Mutex<T> 
where T:WriteAtMut+?Sized
{
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        let se = self.lock();
        let mut se = match se {
            Ok(x) => x,
            Err(_) =>  return Err(Error::new(
                ErrorKind::Other,
                "poisoned mutex encountered",
            )),
        };
        WriteAtMut::write_at(&mut *se, buf, offset)
    }
    fn write_all_at(&self, buf: &[u8], offset: u64) -> Result<()> {
        let se = self.lock();
        let mut se = match se {
            Ok(x) => x,
            Err(_) =>  return Err(Error::new(
                ErrorKind::Other,
                "poisoned mutex encountered",
            )),
        };
        WriteAtMut::write_all_at(&mut *se, buf, offset)
    }
}


//pub struct DerefWrapper

#[cfg(test)]
mod tests {
    use super::*;

    fn i_want_immut<T:ReadAt+?Sized>(t:&T) {
        let mut v = vec![0,0,0];
        t.read_exact_at(&mut v[..], 3).unwrap();
        assert_eq!(v, vec![7,8,9]);
    }
    fn i_want_mut<T:ReadAtMut+?Sized>(t:&mut T) {
        let mut v = vec![0,0];
        t.read_exact_at(&mut v[..], 2).unwrap();
        assert_eq!(v, vec![6,7]);
    }
    fn i_have_obj() -> Box<dyn ReadAtMut> { 
        let v = vec![4u8, 5,6,7,8,9,10,11];
        let o = ReadWriteSeek(std::io::Cursor::new(v));
        Box::new(o)
     }

    #[test]
    fn check_refc_wrapping_works() {
        let mut o = i_have_obj();
        i_want_mut(&mut *o);
        let rc = std::cell::RefCell::new(DerefWrapper(o));
        i_want_immut(&rc);

        let v = vec![4u8, 5,6,7,8,9,10,11];
        let mut o2 = ReadWriteSeek(std::io::Cursor::new(v));
        i_want_mut(&mut o2);
        let rc2 = std::cell::RefCell::new(o2);
        i_want_immut(&rc2);
    }


    fn i_have_obj2() -> Box<dyn ReadWriteAtMut + Send> { 
        let v = vec![4u8, 5,6,7,8,9,10,11];
        let o = ReadWriteSeek(std::io::Cursor::new(v));
        Box::new(o)
    }
    fn i_want_immut2<T:ReadWriteAt+?Sized>(t:&T) {
        let mut v = vec![0,0,0];
        t.read_exact_at(&mut v[..], 0).unwrap();
        assert_eq!(v, vec![4,5,6]);

        std::thread::sleep(std::time::Duration::from_millis(100));

        t.read_exact_at(&mut v[..], 0).unwrap();
        assert_eq!(v, vec![4,44,44]);
    }
    fn i_want_immut3<T:ReadWriteAt+?Sized>(t:&T) {
        let v = vec![44,44, 44];
        std::thread::sleep(std::time::Duration::from_millis(50));
        t.write_all_at(&v[..], 1).unwrap();
    }

    #[test]
    fn check_mutex_wrapping_works() {
        let o = i_have_obj2();
        let rc = std::sync::Mutex::new(DerefWrapper(o));
        let rc = std::sync::Arc::new(rc);
        let rc2 = rc.clone();
        
        let g1 = std::thread::spawn(move|| {
            i_want_immut2(&*rc)
        });
        let g2 = std::thread::spawn(move|| {
            i_want_immut3(&*rc2)
        });
        g1.join().unwrap();
        g2.join().unwrap();
    }

    #[allow(unused)]
    #[cfg(unix)]
    fn check_refc_wrapping_works2() {
      
        let f : std::fs::File = unimplemented!();
        i_want_mut(&mut f);
        let rc2 = std::cell::RefCell::new(f);
        i_want_immut(&rc2);
    }
}

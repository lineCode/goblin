//! A byte-offset based string table.
//! Commonly used in ELF binaries, Unix archives, and even PE binaries.

use core::ops::Index;
use core::slice;
use core::str;
use core::fmt;
use scroll::{self, ctx, Pread};
#[cfg(feature = "std")]
use error;

/// A common string table format which is indexed by byte offsets (and not
/// member index). Constructed using [`parse`](#method.parse)
/// with your choice of delimiter. Please be careful.
pub struct Strtab<'a> {
    bytes: &'a[u8],
    delim: ctx::StrCtx,
}

#[inline(always)]
fn get_str(offset: usize, bytes: &[u8], delim: ctx::StrCtx) -> scroll::Result<&str> {
    bytes.pread_with::<&str>(offset, delim)
}

impl<'a> Strtab<'a> {
    /// Construct a new strtab with `bytes` as the backing string table, using `delim` as the delimiter between entries
    pub fn new (bytes: &'a [u8], delim: u8) -> Self {
        Strtab { delim: ctx::StrCtx::from(delim), bytes: bytes }
    }
    /// Construct a strtab from a `ptr`, and a `size`, using `delim` as the delimiter
    pub unsafe fn from_raw(ptr: *const u8, size: usize, delim: u8) -> Strtab<'a> {
        Strtab { delim: ctx::StrCtx::from(delim), bytes: slice::from_raw_parts(ptr, size) }
    }
    #[cfg(feature = "std")]
    /// Parses a strtab from `bytes` at `offset` with `len` size as the backing string table, using `delim` as the delimiter
    pub fn parse(bytes: &'a [u8], offset: usize, len: usize, delim: u8) -> error::Result<Strtab<'a>> {
        let bytes: &'a [u8] = bytes.pread_slice(offset, len)?;
        Ok(Strtab { bytes: bytes, delim: ctx::StrCtx::from(delim) })
    }
    #[cfg(feature = "std")]
    /// Converts the string table to a vector, with the original `delim` used to separate the strings
    pub fn to_vec(self) -> error::Result<Vec<String>> {
        let len = self.bytes.len();
        let mut strings = Vec::with_capacity(len);
        let mut i = 0;
        while i < len {
            let string = self.get(i)?;
            i = i + string.len() + 1;
            strings.push(string.to_string());
        }
        Ok(strings)
    }
    /// Safely parses and gets a str reference from the backing bytes starting at byte `offset`
    pub fn get(&self, offset: usize) -> scroll::Result<&'a str> {
        get_str(offset, self.bytes, self.delim)
    }
}

impl<'a> fmt::Debug for Strtab<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "delim: {:?} {:?}", self.delim, str::from_utf8(&self.bytes))
    }
}

impl<'a> Default for Strtab<'a> {
    fn default() -> Strtab<'a> {
        Strtab { bytes: &[], delim: ctx::StrCtx::default() }
    }
}

impl<'a> Index<usize> for Strtab<'a> {
    type Output = str;
    /// Gets str reference at starting at byte `offset`.
    /// **NB**: this will panic if the underlying bytes are not valid utf8, or the offset is invalid
    fn index(&self, offset: usize) -> &Self::Output {
        get_str(offset, &self.bytes, self.delim).unwrap()
    }
}

#[test]
fn as_vec_no_final_null() {
    let bytes = b"\0printf\0memmove\0busta";
    let strtab = unsafe { Strtab::from_raw(bytes.as_ptr(), bytes.len(), 0x0) };
    let vec = strtab.to_vec().unwrap();
    assert_eq!(vec.len(), 4);
    assert_eq!(vec, vec!["", "printf", "memmove", "busta"]);
}

#[test]
fn as_vec_no_first_null_no_final_null() {
    let bytes = b"printf\0memmove\0busta";
    let strtab = unsafe { Strtab::from_raw(bytes.as_ptr(), bytes.len(), 0x0) };
    let vec = strtab.to_vec().unwrap();
    assert_eq!(vec.len(), 3);
    assert_eq!(vec, vec!["printf", "memmove", "busta"]);
}

#[test]
fn to_vec_final_null() {
    let bytes = b"\0printf\0memmove\0busta\0";
    let strtab = unsafe { Strtab::from_raw(bytes.as_ptr(), bytes.len(), 0x0) };
    let vec = strtab.to_vec().unwrap();
    assert_eq!(vec.len(), 4);
    assert_eq!(vec, vec!["", "printf", "memmove", "busta"]);
}

#[test]
fn to_vec_newline_delim() {
    let bytes = b"\nprintf\nmemmove\nbusta\n";
    let strtab = unsafe { Strtab::from_raw(bytes.as_ptr(), bytes.len(), '\n' as u8) };
    let vec = strtab.to_vec().unwrap();
    assert_eq!(vec.len(), 4);
    assert_eq!(vec, vec!["", "printf", "memmove", "busta"]);
}

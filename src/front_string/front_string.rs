use std::{fmt, mem, ops::Deref};

use crate::FrontVec;

pub struct FrontString {
    /// Must always contain valid UTF8 sequence of bytes.
    buf: FrontVec<u8>,
}

impl FrontString {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: FrontVec::with_capacity(capacity),
        }
    }

    pub fn push_char_front(&mut self, ch: char) {
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf);
        self.buf.extend_front(bytes.as_bytes());
    }

    pub fn pop_char_front(&mut self) -> Option<char> {
        self.chars().next().map(|first_char: char| {
            // Pop off all of first_char's bytes.
            for _ in 0..first_char.len_utf8() {
                unsafe {
                    self.buf.pop_front().unwrap_unchecked();
                }
            }

            first_char
        })
    }

    pub fn push_str_front<S: AsRef<str>>(&mut self, s: S) {
        self.buf.extend_front(s.as_ref().as_bytes());
    }
}

impl From<&str> for FrontString {
    fn from(s: &str) -> Self {
        let mut fs = FrontString::new();
        fs.push_str_front(s);
        fs
    }
}

impl Deref for FrontString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let byte_slice = self.buf.as_ref();
        // SAFETY: Because `self.buf` always contains valid UTF8, this is safe.
        unsafe { std::str::from_utf8_unchecked(byte_slice) }
    }
}

impl AsRef<str> for FrontString {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl AsRef<[u8]> for FrontString {
    fn as_ref(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl<S: AsRef<str>> PartialEq<S> for FrontString {
    fn eq(&self, other: &S) -> bool {
        <Self as AsRef<str>>::as_ref(self) == other.as_ref()
    }
}

impl Eq for FrontString {}

impl fmt::Debug for FrontString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice: &str = self.as_ref();
        write!(f, "{slice:?}")
    }
}

impl fmt::Display for FrontString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice: &str = self.as_ref();
        write!(f, "{slice}")
    }
}

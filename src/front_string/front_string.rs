use std::{mem, ops::Deref};

use crate::FrontVec;

pub struct FrontString {
    /// Must always contain valid UTF8 sequence of bytes.
    buf: FrontVec<u8>,
}

impl FrontString {
    pub fn new() -> Self {
        Self {
            buf: FrontVec::new(),
        }
    }

    pub fn push_char_front(&mut self, ch: char) {
        let bytes_needed = ch.len_utf8();
        let byte_repr: [u8; 4] = unsafe { mem::transmute(ch) };
        for byte in &byte_repr[4 - bytes_needed..] {
            self.buf.push_front(*byte);
        }
    }

    pub fn pop_char_front(&mut self) -> Option<char> {
        self.chars().next().map(|first_char| {
            // Pop off all of first_char's bytes.
            for _ in 0..first_char.len_utf8() {
                self.buf.pop_front();
            }

            first_char
        })
    }

    pub fn push_str_front(&mut self, s: &str) {
        self.buf.extend_front(s.as_bytes());
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

use std::{fmt, mem::MaybeUninit, ops::Deref};

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

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Ensures capacity has at least `additional` more bytes of capacity.
    ///
    /// Returns `true` if a reallocation happened, `false` otherwise.
    pub fn reserve_front(&mut self, additional: usize) -> bool {
        self.buf.reserve_front(additional)
    }

    pub fn push_char_front(&mut self, ch: char) {
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).bytes();
        self.buf.extend_front(bytes);
    }

    pub fn pop_char_front(&mut self) -> Option<char> {
        self.chars().next().map(|first_char: char| {
            // Pop off all of first_char's bytes.
            for _ in 0..first_char.len_utf8() {
                // SAFETY:
                // TODO[safety argument omitted]
                unsafe {
                    self.buf.pop_front().unwrap_unchecked();
                }
            }

            first_char
        })
    }

    pub fn push_str_front<S: AsRef<str>>(&mut self, s: S) {
        self.buf.extend_front(s.as_ref().bytes());
    }

    /// Returns a mutable slice that references the uninitialized portion of the
    /// underlying buffer.
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.buf.spare_capacity_mut()
    }

    /// # Safety
    /// * `new_len` must be less than or equal to `capacity()`.
    /// * The elements at `old_len..new_len` must be initialized.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        unsafe { self.buf.set_len(new_len) }
    }

    /// Writes the bytes from an `ExactSizeIterator` of bytes onto the front of
    /// the string.
    ///
    /// Returns `true` if a reallocation happened, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use front_vec::FrontString;
    /// # use assert2::assert;
    /// let mut s = FrontString::from("world!");
    /// let prefix = "Hello, ";
    /// s.prepend_from_bytes_iter(prefix.bytes());
    /// assert!(s == "Hello, world!");
    /// ```
    pub fn prepend_from_bytes_iter<Bs>(&mut self, bytes: Bs) -> bool
    where
        Bs: ExactSizeIterator<Item = u8>,
    {
        let did_realloc = self.reserve_front(bytes.len());

        let bytes_len = bytes.len();
        let spare = self.spare_capacity_mut();
        let spare_len = spare.len();
        let begin_write = spare_len - bytes_len;
        let reserved_space = &mut spare[begin_write..];

        // MEMORY DIAGRAM:
        //
        // [ ???????????????????????????? |  reserved_space | initialized front vec ]
        // |<- (spare_len - bytes_len) -> |<-- bytes_len -->|<--------- len ------->|
        // |<-------- spare_len = (cap - len) ------------->|
        // |<-------------------------------- cap --------------------------------->|

        for (byte, slot) in bytes.zip(reserved_space.iter_mut()) {
            slot.write(byte);
        }

        unsafe {
            self.set_len(self.len() + bytes_len);
        }

        did_realloc
    }

    /// Shortens the `FrontString`, keeping the **last** `len` bytes and
    /// dropping the rest.
    /// If `len` is greater than the current length, this has no effect.
    /// Note that this method has no effect on the allocated capacity of the
    /// `FrontString`.
    ///
    /// # Panics
    /// Panics if `new_len` does not lie on a `char` boundary.
    pub fn truncate(&mut self, new_len: usize) {
        let new_len = usize::min(self.len(), new_len);

        if !self.is_char_boundary(new_len) {
            panic!("new length is not on a char boundary");
        }

        self.buf.truncate(new_len);
    }
}

impl From<&str> for FrontString {
    fn from(s: &str) -> Self {
        let mut fs = FrontString::new();
        fs.push_str_front(s);
        fs
    }
}

impl From<String> for FrontString {
    fn from(s: String) -> Self {
        // SAFETY:
        //  1. A String is valid utf8 bytes.
        //  2. The Vec<u8> produced from a String contains valid utf8 bytes.
        //  3. A FrontVec<u8> produced from the Vec<u8> contains valid utf8 bytes.
        //  4. Therefore Self contains valid utf8 bytes.
        let byte_vec: Vec<u8> = s.into();
        Self {
            buf: byte_vec.into(),
        }
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

impl Default for FrontString {
    fn default() -> Self {
        FrontString::new()
    }
}

impl Clone for FrontString {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
        }
    }
}

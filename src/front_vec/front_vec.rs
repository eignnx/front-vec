use std::{
    alloc::{alloc, Layout},
    fmt,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::Unique,
    slice::SliceIndex,
};

/// # Memory Layout:
/// ```ignore
/// [?, ?, ?, ?, e1, e2, e3]
///              ^^^^^^^^^^ initialized region
/// ^^^^^^^^^^^ uninitialized region
/// ```
pub struct FrontVec<T> {
    buf: Unique<MaybeUninit<T>>,
    cap: usize,
    len: usize,
    _marker: PhantomData<T>,
}

fn alloc_buf<T>(len: usize) -> Unique<MaybeUninit<T>> {
    assert_ne!(mem::size_of::<T>(), 0);

    if len == 0 {
        return Unique::dangling();
    }

    let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
    // SAFETY:
    // TODO[safety argument omitted]
    let ptr = unsafe { alloc(layout) as *mut MaybeUninit<T> };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout)
    };
    // SAFETY:
    // TODO[safety argument omitted]
    unsafe { Unique::new_unchecked(ptr) }
}

impl<T> FrontVec<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(cap: usize) -> Self {
        assert_ne!(mem::size_of::<T>(), 0);
        Self {
            buf: alloc_buf(cap),
            cap,
            len: 0,
            _marker: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn double_no_realloc(&mut self) {
        self.grow_no_realloc(self.cap * 2);
    }

    pub fn grow_no_realloc(&mut self, new_cap: usize) {
        // First alloc a new buffer and swap it out with the old buffer.
        let old_buf = mem::replace(&mut self.buf, alloc_buf(new_cap));

        let old_cap = self.cap;
        self.cap = new_cap;

        // If old buffer wasn't `Unique::dangling()`...
        if old_cap > 0 {
            // SAFETY:
            // TODO[safety argument omitted]
            // Calculate pointers to the beginnings of the initialized elements.
            let old_front = unsafe { old_buf.as_ptr().add(old_cap - self.len) };
            // SAFETY:
            // TODO[safety argument omitted]
            let front = unsafe { self.buf.as_ptr().add(self.cap - self.len) };

            // SAFETY:
            // TODO[safety argument omitted]
            // Copy all initialized elements from old to new.
            unsafe {
                front.copy_from_nonoverlapping(old_front, self.len);
            }
            // Deallocate old buffer.
            let old_layout = Layout::array::<MaybeUninit<T>>(old_cap).unwrap();
            // SAFETY:
            // TODO[safety argument omitted]
            unsafe {
                std::alloc::dealloc(old_buf.as_ptr() as *mut u8, old_layout);
            }
        }
    }

    fn front_internal_index(&self) -> usize {
        self.cap - self.len
    }

    /// # Safety
    /// This doesn't necessarily return something that points into the buf.
    unsafe fn before_front_mut(&mut self) -> &mut MaybeUninit<T> {
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe {
            self.buf
                .as_ptr()
                .add(self.front_internal_index() - 1)
                .as_mut()
                .unwrap_unchecked()
        }
    }

    fn front_mut(&mut self) -> &mut MaybeUninit<T> {
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe {
            self.buf
                .as_ptr()
                .add(self.front_internal_index())
                .as_mut()
                .unwrap_unchecked()
        }
    }

    fn front_ptr(&self) -> *const MaybeUninit<T> {
        let ptr = self.buf.as_ptr() as *const MaybeUninit<T>;
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe { ptr.add(self.front_internal_index()) }
    }

    pub fn push_front(&mut self, val: T) {
        if self.cap == 0 {
            self.buf = alloc_buf(4);
            self.cap = 4;
        } else if self.len >= self.cap {
            self.double_no_realloc();
        }

        // SAFETY:
        // TODO[safety argument omitted]
        let slot = unsafe { self.before_front_mut() };
        slot.write(val);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let front = self.front_mut();
        // SAFETY:
        // TODO[safety argument omitted]
        let val = unsafe { front.assume_init_read() };
        self.len -= 1;
        Some(val)
    }

    /// Returns false if capacity was already sufficient, returns true if a
    /// reallocation was done.
    pub fn reserve_front(&mut self, extra_space_needed: usize) -> bool {
        let available_space = self.capacity() - self.len();

        if available_space >= extra_space_needed {
            false
        } else {
            self.grow_no_realloc(self.capacity() + extra_space_needed);
            true
        }
    }

    /// Returns a slice that references the uninitialized portion of the underlying
    /// buffer.
    pub fn get_uninit_raw_parts(&self) -> (*const MaybeUninit<T>, usize) {
        (self.buf.as_ptr(), self.cap - self.len)
    }

    /// Returns a mutable slice that references the uninitialized portion of the
    /// underlying buffer.
    pub fn get_uninit_raw_parts_mut(&mut self) -> (*mut MaybeUninit<T>, usize) {
        (self.buf.as_ptr(), self.cap - self.len)
    }

    pub fn extend_front(&mut self, slice: &[T]) {
        if slice.is_empty() {
            return;
        }

        self.reserve_front(slice.len());

        let (buf, _) = self.get_uninit_raw_parts_mut();
        // SAFETY:
        // TODO[safety argument omitted]
        let buf = unsafe { buf.as_mut().unwrap() };
        // SAFETY:
        // TODO[safety argument omitted]
        let buf: *mut T = unsafe { buf.assume_init_mut() } as *mut T;
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe {
            // NOTE: doesn't drop anything..... hmmmmmm
            buf.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
        }

        self.len += slice.len();
    }

    /// Shortens the `FrontVec`, keeping the **last** `len` elements and
    /// dropping the rest.
    /// If `len` is greater than the current length, this has no effect.
    /// Note that this method has no effect on the allocated capacity of the
    /// `FrontVec`.
    pub fn truncate(&mut self, len: usize) {
        let new_len = usize::min(len, self.len);
        let to_drop = self.len - new_len;
        if mem::size_of::<T>() > 0 {
            for item in &mut self[0..to_drop] {
                // SAFETY:
                //
                // Assume `ptr` is `(&mut self[i]) as *mut _)` for any index `i` within
                // bounds.
                //
                // > `to_drop` must be [valid] for both reads and writes.
                // Assuming `self` is valid, `ptr` is valid.
                //
                // > `to_drop` must be properly aligned, even if `T` has size 0.
                // If `self` has a stored value already, then `ptr` is aligned. Also,
                // `size_of::<T>()` is non-zero due to the if statement.
                //
                // > `to_drop` must be nonnull, even if `T` has size 0.
                // Since `ptr` comes from a `&mut T`, it's nonnull.
                //
                // > The value `to_drop` points to must be valid for dropping, which may mean
                // > it must uphold additional invariants. These invariants depend on the type
                // > of the value being dropped. For instance, when dropping a Box, the box's
                // > pointer to the heap must be valid.
                // We assume a `&mut T` references a valid-to-drop value.
                //
                // > While `drop_in_place` is executing, the only way to access parts of
                // > `to_drop` is through the `&mut self` references supplied to the
                // > `Drop::drop` methods that `drop_in_place` invokes.
                // Yes, `ptr` is a unique pointer to it's pointee because it comes from a
                // `&mut T`.
                //
                // > Additionally, if `T` is not [`Copy`], using the pointed-to value after
                // > calling `drop_in_place` can cause undefined behavior. Note that `*to_drop =
                // > foo` counts as a use because it will cause the value to be dropped
                // > again. [`write()`] can be used to overwrite data without causing it to be
                // > dropped.
                // The pointer `ptr` is immediately discarded at the end of the loop body. It
                // won't be used again after the `drop_in_place`.
                unsafe {
                    std::ptr::drop_in_place(item as *mut _);
                }
            }
        }
        self.len = new_len;
    }
}

impl<T> AsMut<[T]> for FrontVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        let front = self.front_mut().as_mut_ptr();
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe { std::slice::from_raw_parts_mut(front, self.len) }
    }
}

impl<T> AsRef<[T]> for FrontVec<T> {
    fn as_ref(&self) -> &[T] {
        let front = self.front_ptr();
        // SAFETY:
        // TODO[safety argument omitted]
        let slice = unsafe { std::slice::from_raw_parts(front, self.len) };
        unsafe { MaybeUninit::slice_assume_init_ref(slice) }
    }
}

impl<T> Deref for FrontVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for FrontVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Drop for FrontVec<T> {
    fn drop(&mut self) {
        for item in self.as_mut() {
            drop(item)
        }

        if self.cap == 0 {
            // No buffer has been allocated, so DO NOT deallocate it.
            return;
        }

        let buf = self.buf.as_ptr() as *mut u8;
        let layout = Layout::array::<T>(self.cap).unwrap();
        // SAFETY:
        // TODO[safety argument omitted]
        unsafe {
            std::alloc::dealloc(buf, layout);
        }
    }
}

// impl<T> Index<usize> for FrontVec<T> {
//     type Output = T;

//     #[track_caller]
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get(index).unwrap()
//     }
// }

// impl<T> IndexMut<usize> for FrontVec<T> {
//     #[track_caller]
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         self.get_mut(index).unwrap()
//     }
// }

impl<T, I: SliceIndex<[T]>> Index<I> for FrontVec<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for FrontVec<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T: fmt::Debug> fmt::Debug for FrontVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice: &[T] = self.as_ref();
        slice.fmt(f)
    }
}

impl<T: Copy> From<&[T]> for FrontVec<T> {
    fn from(slice: &[T]) -> Self {
        let mut v = FrontVec::with_capacity(slice.len());

        for item in slice.iter().rev() {
            v.push_front(*item);
        }

        v
    }
}

impl<T: Copy, const N: usize> From<&[T; N]> for FrontVec<T> {
    fn from(array: &[T; N]) -> Self {
        array.as_ref().into()
    }
}

impl<T> From<Vec<T>> for FrontVec<T> {
    /// Note: Any extra capacity is dropped.
    fn from(v: Vec<T>) -> Self {
        let bs = v.into_boxed_slice();
        let len = bs.len();
        let cap = len;
        let buf = Unique::from(Box::leak(bs)).cast();
        Self {
            buf,
            len,
            cap,
            _marker: Default::default(),
        }
    }
}

impl<T: Clone> Clone for FrontVec<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T: PartialEq> PartialEq for FrontVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: Eq> Eq for FrontVec<T> {}

impl<T> Default for FrontVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

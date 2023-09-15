use std::{
    fs::{self, File, OpenOptions},
    io, mem,
    ops::{Deref, DerefMut},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    ptr, slice,
    sync::atomic::Ordering,
};

use crate::{
    stats::{COUNT_ACTIVE_SEGMENT, COUNT_MMAP_FAILED, COUNT_MUNMAP_FAILED},
    utils::page_size,
};

/// Segment is a constant slice of type T that is memory mapped to disk.
///
/// It is the basic building block of memory mapped data structure.
///
/// It cannot growth / shrink.
#[derive(Debug)]
pub struct Segment<T> {
    addr: *mut T,
    len: usize,
    capacity: usize,
    path: Option<PathBuf>,
}

impl<T> Segment<T> {
    /// Create a zero size segment.
    pub const fn null() -> Self {
        Self {
            addr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            path: None,
        }
    }

    /// Memory map a segment to disk.
    ///
    /// File will be created and init with computed capacity.
    pub fn open_rw<P: AsRef<Path>>(path: P, capacity: usize) -> io::Result<Self> {
        if capacity == 0 {
            return Ok(Self::null());
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        // Fill the file with 0
        unsafe { ftruncate::<T>(&file, capacity) }?;

        // Map the block
        let addr = unsafe { mmap(&file, capacity) }?;
        Ok(Self {
            addr,
            len: 0,
            capacity,
            path: Some(path.as_ref().to_path_buf()),
        })
    }

    /// Currently used segment size.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Shortens the segment, keeping the first `new_len` elements and dropping
    /// the rest.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len > self.len {
            return;
        }

        unsafe {
            let remaining_len = self.len - new_len;
            let items = ptr::slice_from_raw_parts_mut(self.addr.add(new_len), remaining_len);
            self.set_len(new_len);
            ptr::drop_in_place(items);
        }
    }

    /// Remove `delete_count` element at beginning of the segment.
    ///
    /// Element will be drop in place.
    ///
    /// If delete count is greater than the segment len, then this call will be
    /// equivalent to calling `clear` function.
    pub fn truncate_first(&mut self, delete_count: usize) {
        let new_len = self.len.saturating_add_signed(-(delete_count as isize));
        if new_len == 0 {
            self.clear()
        } else {
            unsafe {
                let items = slice::from_raw_parts_mut(self.addr, delete_count);
                ptr::drop_in_place(items);
                ptr::copy(self.addr.add(delete_count), self.addr, new_len);
                self.set_len(new_len);
            }
        }
    }

    /// Clears the segment, removing all values.
    pub fn clear(&mut self) {
        unsafe {
            let items = slice::from_raw_parts_mut(self.addr, self.len);
            self.set_len(0);
            ptr::drop_in_place(items);
        }
    }

    /// Forces the length of the segment to `new_len`.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    /// Bytes use on disk for this segment.
    pub fn disk_size(&self) -> usize {
        self.capacity * mem::size_of::<T>()
    }

    /// Try to add new element to the segment.
    ///
    /// If the segment is already full, value will be return in `Err`.
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len == self.capacity {
            return Err(value);
        }

        unsafe {
            let dst = self.addr.add(self.len);
            ptr::write(dst, value);
        }

        self.len += 1;
        Ok(())
    }

    /// Remove last element of the segment and reduce its capacity.
    ///
    /// Value will be return if segment is not empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe {
            let src = self.addr.add(self.len);
            Some(ptr::read(src))
        }
    }

    /// Move data contained in `other` segment to the end of current segment.
    ///
    /// ```rust
    /// # use mmap_vec::Segment;
    /// let mut s1 = Segment::<i32>::open_rw("test_extend_from_segment_1", 2).unwrap();
    /// let mut s2 = Segment::<i32>::open_rw("test_extend_from_segment_2", 5).unwrap();
    ///
    /// s1.push_within_capacity(7);
    /// s1.push_within_capacity(-3);
    /// s2.push_within_capacity(-4);
    /// s2.push_within_capacity(37);
    ///
    /// assert_eq!(&s1[..], [7, -3]);
    /// assert_eq!(&s2[..], [-4, 37]);
    ///
    /// s2.extend_from_segment(s1);
    /// assert_eq!(&s2[..], [-4, 37, 7, -3]);
    /// ```
    pub fn extend_from_segment(&mut self, mut other: Segment<T>) {
        let new_len = other.len + self.len;
        assert!(
            new_len <= self.capacity,
            "New segment is too small: new_len={}, capacity={}",
            new_len,
            self.capacity
        );

        unsafe {
            ptr::copy_nonoverlapping(other.addr, self.addr.add(self.len), other.len);
            self.set_len(new_len);
            other.set_len(0);
        };
    }

    /// Inform the kernel that the complete segment will be access in a near future.
    ///
    /// All underlying pages should be load in RAM.
    ///
    /// This function is only a wrapper above `libc::madvise`.
    ///
    /// Will panic if `libc::madvise` return an error.
    pub fn advice_prefetch_all_pages(&self) {
        if self.addr.is_null() || self.len == 0 {
            return;
        }

        let madvise_code = unsafe {
            libc::madvise(
                self.addr.cast(),
                self.len * mem::size_of::<T>(),
                libc::MADV_WILLNEED,
            )
        };
        assert_eq!(
            madvise_code,
            0,
            "madvise error: {}",
            io::Error::last_os_error()
        );
    }

    /// Inform the kernel that underlying page for `index` will be access in a near future.
    ///
    /// This function is only a wrapper above `libc::madvise`.
    pub fn advice_prefetch_page_at(&self, index: usize) {
        if self.addr.is_null() || index >= self.len {
            return;
        }

        let page_size = page_size();
        let page_mask = !(page_size.wrapping_add_signed(-1));

        let madvise_code = unsafe {
            libc::madvise(
                (self.addr.add(index) as usize & page_mask) as *mut libc::c_void,
                page_size,
                libc::MADV_WILLNEED,
            )
        };
        assert_eq!(
            madvise_code,
            0,
            "madvise error: {}",
            io::Error::last_os_error()
        );
    }
}

impl<T> Deref for Segment<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.addr, self.len) }
    }
}

impl<T> DerefMut for Segment<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.addr, self.len) }
    }
}

impl<T> Drop for Segment<T> {
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe { ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.addr, self.len)) }
        }

        if self.capacity > 0 {
            let _ = unsafe { munmap(self.addr, self.capacity) };
        }

        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }
}

unsafe impl<T> Send for Segment<T> {}
unsafe impl<T> Sync for Segment<T> {}

unsafe fn ftruncate<T>(file: &File, capacity: usize) -> io::Result<()> {
    let segment_size = capacity * mem::size_of::<T>();
    let fd = file.as_raw_fd();

    if libc::ftruncate(fd, segment_size as libc::off_t) != 0 {
        COUNT_MMAP_FAILED.fetch_add(1, Ordering::Relaxed);
        return Err(io::Error::last_os_error());
    } else {
        Ok(())
    }
}

unsafe fn mmap<T>(file: &File, capacity: usize) -> io::Result<*mut T> {
    let segment_size = capacity * mem::size_of::<T>();

    // It is safe to not keep a reference to the initial file descriptor.
    // See: https://stackoverflow.com/questions/17490033/do-i-need-to-keep-a-file-open-after-calling-mmap-on-it
    let fd = file.as_raw_fd();

    let addr = libc::mmap(
        std::ptr::null_mut(),
        segment_size as libc::size_t,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        0,
    );

    if addr == libc::MAP_FAILED {
        COUNT_MMAP_FAILED.fetch_add(1, Ordering::Relaxed);
        Err(io::Error::last_os_error())
    } else {
        COUNT_ACTIVE_SEGMENT.fetch_add(1, Ordering::Relaxed);
        Ok(addr.cast())
    }
}

unsafe fn munmap<T>(addr: *mut T, capacity: usize) -> io::Result<()> {
    debug_assert!(!addr.is_null());
    debug_assert!(capacity > 0);

    let unmap_code = libc::munmap(addr.cast(), capacity * mem::size_of::<T>());

    if unmap_code != 0 {
        COUNT_MUNMAP_FAILED.fetch_add(1, Ordering::Relaxed);
        Err(io::Error::last_os_error())
    } else {
        COUNT_ACTIVE_SEGMENT.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
}

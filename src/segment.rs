use std::{
    fs::{self, OpenOptions},
    io,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    os::{fd::AsRawFd, unix::prelude::FileExt},
    path::{Path, PathBuf},
    ptr, slice,
};

/// Segment is a constant slice of type T that is memory mapped to disk.
///
/// It is the basic building block of memory mapped data structure.
///
/// It cannot growth / shrink.
#[derive(Debug)]
pub struct Segment<T> {
    addr: *mut libc::c_void,
    len: usize,
    capacity: usize,
    path: Option<PathBuf>,
    _phantom: PhantomData<T>,
}

impl<T> Segment<T> {
    /// Create a zero size segment.
    pub const fn null() -> Self {
        Self {
            addr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            path: None,
            _phantom: PhantomData,
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

        // Write a 0 at end of file to force its existence
        let segment_size = capacity * mem::size_of::<T>();
        file.write_at(&[0], (segment_size - 1) as u64)?;

        // It is safe to not keep a reference to the initial file descriptor.
        // See: https://stackoverflow.com/questions/17490033/do-i-need-to-keep-a-file-open-after-calling-mmap-on-it
        let fd = file.as_raw_fd();
        let offset = 0;

        let addr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                segment_size as libc::size_t,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                offset,
            )
        };

        if addr == libc::MAP_FAILED {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self {
                addr,
                len: 0,
                capacity,
                path: Some(path.as_ref().to_path_buf()),
                _phantom: PhantomData,
            })
        }
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
            let items =
                ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(new_len), remaining_len);
            self.set_len(new_len);
            ptr::drop_in_place(items);
        }
    }

    /// Clears the segment, removing all values.
    pub fn clear(&mut self) {
        unsafe {
            let items = slice::from_raw_parts_mut(self.addr as *mut T, self.len);
            self.set_len(0);
            ptr::drop_in_place(items);
        }
    }

    /// Forces the length of the segment to `new_len`.
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
            let dst = self.as_mut_ptr().add(self.len);
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
            let src = self.as_ptr().add(self.len);
            Some(ptr::read(src))
        }
    }

    /// Erase segment content with `other` segment argument.
    pub fn fill_from(&mut self, mut other: Segment<T>) {
        assert!(self.len == 0, "New segment contains already some data");
        assert!(
            other.capacity < self.capacity,
            "Copy segment size error (src: {}, dst: {})",
            other.capacity,
            self.capacity
        );

        unsafe {
            ptr::copy(other.addr as *const T, self.addr as *mut T, other.capacity);
            self.set_len(other.len);
            other.set_len(0);
        };
    }
}

impl<T> Deref for Segment<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.addr as *const T, self.len) }
    }
}

impl<T> DerefMut for Segment<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.addr as *mut T, self.len) }
    }
}

impl<T> Drop for Segment<T> {
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe {
                ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
            }
        }

        if self.capacity > 0 {
            assert!(!self.addr.is_null());

            unsafe {
                // Just use debug assert here, if `munmap` failed, we cannot do so much more ...
                debug_assert!(
                    libc::munmap(self.addr, self.capacity) == 0,
                    "munmap failed: {}",
                    io::Error::last_os_error()
                );
            }
        }

        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }
}

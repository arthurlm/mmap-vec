#![allow(clippy::partialeq_ne_impl)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]

/*! # Rust memory mapped vector

[![CI Status](https://github.com/arthurlm/mmap-vec/workflows/Test/badge.svg)](https://github.com/arthurlm/mmap-vec/actions/)
[![codecov](https://codecov.io/gh/arthurlm/mmap-vec/graph/badge.svg?token=1TXRTK3C3Q)](https://codecov.io/gh/arthurlm/mmap-vec)
[![docs.rs](https://docs.rs/mmap-vec/badge.svg)](https://docs.rs/mmap-vec/)
[![Crates.io](https://img.shields.io/crates/v/mmap-vec)](https://crates.io/crates/mmap-vec)
[![LICENSE](https://img.shields.io/crates/l/mmap-vec)](https://raw.githubusercontent.com/arthurlm/mmap-vec/main/LICENSE)

This crate contains implementation / helper to create data struct that are memory mapped.

Sometime, you have to deal with vector / data that cannot fit in memory.
Moving them to disk and memory map them is a good way to deal with this problem.

## How to use it ?

That is so simple !

```rust
use mmap_vec::MmapVec;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Row {
    id: usize,
    age: u8,
}

let row1 = Row { id: 42, age: 18 };
let row2 = Row { id: 894, age: 99 };

// Create a memory mapped vec 😎
let mut v = MmapVec::<Row>::new();

// Push can trigger new mmap segment creation, so it can fail.
v.push(row1).unwrap();
v.push(row2).unwrap();

// Check the content
assert_eq!(v[0], row1);
assert_eq!(&v[..], &[row1, row2]);

// Pop content
assert_eq!(v.pop(), Some(row2));
assert_eq!(v.pop(), Some(row1));
```

Check the unit tests for more example.

## How it works ?

The main idea here is to provide a basic `struct Segment`.

This struct provides constant size memory mapped array of type `T`.
Wrapping `Segment` into a new struct `MmapVec` that handle segment growth / shrink does the trick.

### Where does the segment are store on disk ?

For now data are stored in `.cache` (if using 'cache-dirs' feature) or `/tmp` under a dedicated folder.

UUID V4 are generated in order to avoid collision when creating segment.

```text
❯ ls /tmp/mmap-vec-rs -1
/tmp/mmap-vec-rs/00d977bf-b556-475e-8de5-d35e7baaa39d.seg
/tmp/mmap-vec-rs/6cb81228-9cf3-4918-a3ef-863907b32830.seg
/tmp/mmap-vec-rs/8a86eeaa-1fa8-4535-9e23-6c59e0c9c376.seg
/tmp/mmap-vec-rs/de62bde3-6524-4c4b-b514-24f6a44d6323.seg
```

### Does segment creation is configurable ?

Yes ! Check out `test_custom_segment_creator::test_custom_segment_builder` for example.

Since segment creation are manage through a trait. You are free to configure it the way you want.

### Does this work on Windows ?

__Nope__. I am not targeting this OS and would like to keep this crate as simple as possible.

I also would like to reduce dependencies as much as possible.

```text
❯ cargo tree
mmap-vec v0.1.1
├── libc v0.2.147
├── uuid v1.4.1
|   └── getrandom v0.2.10
|       ├── cfg-if v1.0.0
|       └── libc v0.2.147
# Optional using 'cache-dir' feature
├── dirs v5.0.1
│   └── dirs-sys v0.4.1
│       ├── libc v0.2.147
│       └── option-ext v0.2.0
[dev-dependencies]
└── glob v0.3.1
```

### Is this crate production ready ?

Yes 😁 !
Since v0.1.1. But feature are a little bit limited for now ...

Github PR to help on this are welcomed !

Prefetching API is not fully stable for now and may change in the future.

## Ideas / new features ?

- Implement custom `std::alloc::Allocator` to use with `std::vec::Vec`
- Serde support
 */

use std::{
    fs, io, mem,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

pub use error::MmapVecError;
pub use segment::Segment;
pub use segment_builder::{DefaultSegmentBuilder, SegmentBuilder};
pub use stats::MmapStats;
pub use vec_builder::MmapVecBuilder;

use crate::utils::page_size;

pub mod error;

mod segment;
mod segment_builder;
mod stats;
mod utils;
mod vec_builder;

/// A disk memory mapped vector.
#[derive(Debug)]
pub struct MmapVec<T, B: SegmentBuilder = DefaultSegmentBuilder> {
    pub(crate) segment: Segment<T>,
    pub(crate) builder: B,
    pub(crate) path: Option<PathBuf>,
}

impl<T, B> MmapVec<T, B>
where
    B: SegmentBuilder,
{
    /// Create a zero size mmap vec.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            segment: Segment::null(),
            builder: B::default(),
            path: None,
        }
    }

    /// Create a mmap vec with a given capacity.
    ///
    /// This function can fail if FS / IO failed.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> io::Result<Self> {
        MmapVecBuilder::new().capacity(capacity).try_build()
    }

    /// Currently used vec size.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.segment.capacity()
    }

    /// Bytes use on disk for this vec.
    pub fn disk_size(&self) -> usize {
        self.segment.disk_size()
    }

    /// Shortens the vec, keeping the first `new_len` elements and dropping
    /// the rest.
    #[inline(always)]
    pub fn truncate(&mut self, new_len: usize) {
        self.segment.truncate(new_len);
    }

    /// Remove `delete_count` element at beginning of the vec.
    ///
    /// Element will be drop in place.
    ///
    /// If delete count is greater than the segment len, then this call will be
    /// equivalent to calling `clear` function.
    ///
    /// Example:
    /// ```rust
    /// # use mmap_vec::MmapVec;
    /// let mut v = MmapVec::<u8>::new();
    /// assert!(v.push(8).is_ok());
    /// assert!(v.push(5).is_ok());
    /// assert!(v.push(3).is_ok());
    /// assert!(v.push(12).is_ok());
    /// assert_eq!(&v[..], &[8, 5, 3, 12]);
    ///
    /// v.truncate_first(2);
    /// assert_eq!(&v[..], [3, 12]);
    ///
    /// v.truncate_first(100);
    /// assert_eq!(&v[..], []);
    /// ```
    #[inline(always)]
    pub fn truncate_first(&mut self, delete_count: usize) {
        self.segment.truncate_first(delete_count);
    }

    /// Clears the vec, removing all values.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.segment.clear();
    }

    /// Remove last value of the vec.
    ///
    /// Value will be return if data structure is not empty.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.segment.pop()
    }

    /// Append a value to the vec.
    ///
    /// If vec is too small, new segment will be created.
    /// Data will then be moved to new segment.
    ///
    /// This is why this function can fail, because it depends on FS / IO calls.
    pub fn push(&mut self, value: T) -> io::Result<()> {
        // Check if we need to growth inner segment.
        if self.segment.len() == self.segment.capacity() {
            let min_capacity = page_size() / mem::size_of::<T>();
            let new_capacity = std::cmp::max(self.segment.capacity() * 2, min_capacity);
            let (new_segment, new_path) = self.builder.create_new_segment::<T>(new_capacity)?;
            debug_assert!(new_segment.capacity() > self.segment.capacity());

            // Copy previous data to new segment.
            let old_segment = mem::replace(&mut self.segment, new_segment);
            let old_path = mem::replace(&mut self.path, Some(new_path));
            self.segment.extend_from_segment(old_segment);

            if let Some(path) = old_path {
                let _ = fs::remove_file(path);
            }
        }

        // Add new value to vec.
        assert!(
            self.push_within_capacity(value).is_ok(),
            "Fail to push to newly created segment"
        );

        Ok(())
    }

    /// Try to push a new value to the data structure.
    ///
    /// If vec is too small, value will be return as an `Err`.
    #[inline(always)]
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        self.segment.push_within_capacity(value)
    }

    /// Resize the vec without copying data.
    ///
    /// # How it works ?
    ///
    /// 1. It first check we need to grow the segment.
    /// 2. Call `Segment::<T>::open_rw` with a bigger capacity that what we already reserve.
    ///    At this point, the file is mmap twice.
    /// 3. Replace `self.segment` we newly mapped segment if there is no error.
    /// 4. Update segment len to avoid calling drop on unwanted data.
    pub fn reserve(&mut self, additional: usize) -> Result<(), MmapVecError> {
        let current_len = self.len();
        let mut new_capacity = current_len + additional;

        if self.capacity() < new_capacity {
            let path = self.path.as_ref().ok_or(MmapVecError::MissingSegmentPath)?;

            // Round to upper page new capacity
            let page_size = page_size();
            let page_capacity = page_size / mem::size_of::<T>();
            if new_capacity % page_capacity != 0 {
                new_capacity += page_capacity - (new_capacity % page_capacity);
            }
            assert!(new_capacity > self.segment.capacity());

            // Map again path with a new segment
            let new_segment = Segment::<T>::open_rw(path, new_capacity)?;

            // At this point we cannot panic anymore !
            // We have to carefully unmap region to avoir calling multiple times drop
            let mut old_segment = mem::replace(&mut self.segment, new_segment);
            assert_ne!(old_segment.addr, self.segment.addr);

            // Update capacity to nothing should be dropped twice.
            unsafe {
                old_segment.set_len(0);
                self.segment.set_len(current_len);
            }
        }

        Ok(())
    }

    /// Inform the kernel that the complete segment will be access in a near future.
    #[inline(always)]
    pub fn advice_prefetch_all_pages(&self) {
        self.segment.advice_prefetch_all_pages()
    }

    /// Inform the kernel that underlying page for `index` will be access in a near future.
    #[inline(always)]
    pub fn advice_prefetch_page_at(&self, index: usize) {
        self.segment.advice_prefetch_page_at(index)
    }

    /// Get underlying file path if any.
    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}

impl<T, B> MmapVec<T, B>
where
    B: SegmentBuilder + Clone,
    T: Clone,
{
    /// Try cloning the vector.
    ///
    /// A new segment will be created for output vec.
    /// Capacity of the new vec will be the same as source vec.
    pub fn try_clone(&self) -> io::Result<Self> {
        let (mut other_segment, other_path) = self.builder.create_new_segment(self.capacity())?;

        // Bellow code could be optimize, but we have to deal with Clone implementation that can panic ...
        for row in &self[..] {
            // It is "safe" here to call panic on error since we already have reserved correct segment capacity.
            assert!(
                other_segment.push_within_capacity(row.clone()).is_ok(),
                "Fail to push to newly cloned segment"
            );
        }

        Ok(Self {
            builder: self.builder.clone(),
            segment: other_segment,
            path: Some(other_path),
        })
    }
}

impl<T, B> Default for MmapVec<T, B>
where
    B: SegmentBuilder,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, B> Deref for MmapVec<T, B>
where
    B: SegmentBuilder,
{
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.segment.deref()
    }
}

impl<T, B> DerefMut for MmapVec<T, B>
where
    B: SegmentBuilder,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.segment.deref_mut()
    }
}

impl<T, U, B1, B2> PartialEq<MmapVec<U, B2>> for MmapVec<T, B1>
where
    B1: SegmentBuilder,
    B2: SegmentBuilder,
    T: PartialEq<U>,
{
    #[inline(always)]
    fn eq(&self, other: &MmapVec<U, B2>) -> bool {
        self[..] == other[..]
    }

    #[inline(always)]
    fn ne(&self, other: &MmapVec<U, B2>) -> bool {
        self[..] != other[..]
    }
}

impl<T, B> Eq for MmapVec<T, B>
where
    B: SegmentBuilder,
    T: Eq,
{
}

impl<T, B> Drop for MmapVec<T, B>
where
    B: SegmentBuilder,
{
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }
}

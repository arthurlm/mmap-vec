#![warn(missing_docs)]

/*! # Rust memory mapped vec

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
let mut v = MmapVec::new();

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

> Where does the segment are store on disk ?

For now data are stored in `/tmp` under a dedicated folder.

UUID V4 are generated in order to avoid collision when creating segment.

```bash
❯ ls /tmp/mmap-vec-rs -1
/tmp/mmap-vec-rs/00d977bf-b556-475e-8de5-d35e7baaa39d.seg
/tmp/mmap-vec-rs/6cb81228-9cf3-4918-a3ef-863907b32830.seg
/tmp/mmap-vec-rs/8a86eeaa-1fa8-4535-9e23-6c59e0c9c376.seg
/tmp/mmap-vec-rs/de62bde3-6524-4c4b-b514-24f6a44d6323.seg
```

> Does segment creation is configurable ?

Not for now. But PR are welcomed !

> Does this work on Windows ?

__Nope__. I am not targeting this OS and would like to keep this crate as simple as possible.

I also would like to reduce dependencies as much as possible.

```bash
❯ cargo tree
mmap-vec v0.1.0
├── libc v0.2.147
└── uuid v1.4.1
    └── getrandom v0.2.10
        ├── cfg-if v1.0.0
        └── libc v0.2.147
```

> Is this crate production ready ?

Check TODO and DONE bellow for this 😁.

## TODO & DONE

- [ ] __production ready__ base code
- [x] unit tests
- [x] doc / example
- [ ] serde support
- [ ] Ability to survive fork
- [ ] CI
- [ ] deployment

## Ideas ?

- Implement custom `std::alloc::Allocator` to use with `std::vec::Vec`
 */

use std::{
    io,
    ops::{Deref, DerefMut},
};

pub use segment::Segment;
use utils::create_unique_segment;

mod segment;
mod utils;

/// A disk memory mapped vector.
#[derive(Debug)]
pub struct MmapVec<T> {
    segment: Segment<T>,
}

impl<T> MmapVec<T> {
    /// Create a zero size mmap vec.
    pub const fn new() -> Self {
        Self {
            segment: Segment::null(),
        }
    }

    /// Create a mmap vec with a given capacity.
    ///
    /// This function can fail if FS / IO failed.
    pub fn with_capacity(capacity: usize) -> io::Result<Self> {
        Ok(Self {
            segment: create_unique_segment(capacity)?,
        })
    }

    /// Currently used vec size.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.segment.capacity()
    }

    /// Remove last value of the vec.
    ///
    /// Value will be return if data structure is not empty.
    #[inline]
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
            let new_capacity = std::cmp::max(self.segment.capacity() * 2, 1);
            let new_segment = create_unique_segment::<T>(new_capacity)?;
            debug_assert!(new_segment.capacity() > self.segment.capacity());

            // Copy previous data to new segment.
            let old_segment = std::mem::replace(&mut self.segment, new_segment);
            self.segment.copy_from(old_segment);
        }

        // Add new value to vec.
        if self.push_within_capacity(value).is_err() {
            panic!("Fail to push to newly created segment")
        }

        Ok(())
    }

    /// Try to push a new value to the data structure.
    ///
    /// If vec is too small, value will be return as an `Err`.
    #[inline]
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        self.segment.push_within_capacity(value)
    }
}

impl<T> Default for MmapVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MmapVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.segment.deref()
    }
}

impl<T> DerefMut for MmapVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.segment.deref_mut()
    }
}

use std::{io, marker::PhantomData, mem};

use crate::{utils::page_size, DefaultSegmentBuilder, MmapVec, Segment, SegmentBuilder};

/// Helps to create vec with custom parameters.
///
/// Example usage:
///
/// ```rust
/// # use mmap_vec::{DefaultSegmentBuilder, MmapVecBuilder};
/// let seg_builder = DefaultSegmentBuilder::with_path("/tmp/rust-mmap");
/// seg_builder.create_dir_all().expect("Fail to create mmap dir");
///
/// let vec = MmapVecBuilder::<usize>::new()
///     .capacity(500)
///     .segment_builder(seg_builder.clone())
///     .try_build()
///     .expect("Fail to create mmap vec");
/// ```
pub struct MmapVecBuilder<T, SB: SegmentBuilder = DefaultSegmentBuilder> {
    segment_builder: SB,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T, SB: SegmentBuilder> MmapVecBuilder<T, SB> {
    /// Create new struct.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update segment builder.
    #[inline(always)]
    pub fn segment_builder(mut self, segment_builder: SB) -> Self {
        self.segment_builder = segment_builder;
        self
    }

    /// Update capacity.
    #[inline(always)]
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Try building a new vec with given parameter.
    ///
    /// This function may failed if segment creation failed.
    pub fn try_build(self) -> io::Result<MmapVec<T, SB>> {
        let path = self.segment_builder.new_segment_path();
        let segment = Segment::open_rw(&path, self.capacity)?;

        Ok(MmapVec {
            segment,
            builder: self.segment_builder,
            path,
        })
    }
}

impl<T, SB: SegmentBuilder> Default for MmapVecBuilder<T, SB> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            segment_builder: SB::default(),
            capacity: page_size() / mem::size_of::<T>(),
            _phantom: PhantomData,
        }
    }
}

use std::{io, marker::PhantomData, mem};

use crate::{DefaultSegmentBuilder, MmapVec, SegmentBuilder};

const PAGE_SIZE: usize = 4096;

/// Helps to create vec with custom parameters.
#[derive(Debug)]
pub struct MmapVecBuilder<T, SB: SegmentBuilder = DefaultSegmentBuilder> {
    segment_builder: SB,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T, SB: SegmentBuilder> MmapVecBuilder<T, SB> {
    /// Create new struct.
    pub fn new() -> Self {
        Self {
            segment_builder: Default::default(),
            capacity: PAGE_SIZE / mem::size_of::<T>(),
            _phantom: PhantomData,
        }
    }

    /// Update segment builder.
    pub fn segment_builder(mut self, segment_builder: SB) -> Self {
        self.segment_builder = segment_builder;
        self
    }

    /// Update capacity.
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Create new vec
    pub fn build(self) -> io::Result<MmapVec<T, SB>> {
        let segment = self.segment_builder.create_new_segment(self.capacity)?;

        Ok(MmapVec {
            segment,
            builder: self.segment_builder,
        })
    }
}

impl<SB: SegmentBuilder + Default> Default for MmapVecBuilder<SB> {
    fn default() -> Self {
        Self::new()
    }
}

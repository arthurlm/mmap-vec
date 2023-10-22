use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) static COUNT_ACTIVE_SEGMENT: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_FTRUNCATE_FAILED: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_MMAP_FAILED: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_MUNMAP_FAILED: AtomicU64 = AtomicU64::new(0);

/// Provides few statistics about low level segment allocation.
///
/// This stats can be useful to debug or to export in various monitoring
/// systems.
#[derive(Default)]
pub struct MmapStats;

impl fmt::Debug for MmapStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MmapStats")
            .field("active", &self.active_segment())
            .field("ftruncate_failed", &self.ftruncate_failed())
            .field("map_failed", &self.map_failed())
            .field("unmap_failed", &self.unmap_failed())
            .finish()
    }
}

impl MmapStats {
    /// Get number of current segment mounted by this library.
    ///
    /// On linux there is a `sysctl` limit you can access with:
    /// ```shell
    /// sysctl vm.max_map_count
    /// ```
    #[inline(always)]
    pub fn active_segment(&self) -> u64 {
        COUNT_ACTIVE_SEGMENT.load(Ordering::Relaxed)
    }

    /// Get number of file truncate failed.
    #[inline(always)]
    pub fn ftruncate_failed(&self) -> u64 {
        COUNT_FTRUNCATE_FAILED.load(Ordering::Relaxed)
    }

    /// Get number of segment creation failed.
    #[inline(always)]
    pub fn map_failed(&self) -> u64 {
        COUNT_MMAP_FAILED.load(Ordering::Relaxed)
    }

    /// Get number of segment deletion failed.
    #[inline(always)]
    pub fn unmap_failed(&self) -> u64 {
        COUNT_MUNMAP_FAILED.load(Ordering::Relaxed)
    }
}

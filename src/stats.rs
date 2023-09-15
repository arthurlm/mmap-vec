use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static COUNT_ACTIVE_SEGMENT: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_FTRUNCATE_FAILED: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_MMAP_FAILED: AtomicU64 = AtomicU64::new(0);
pub(crate) static COUNT_MUNMAP_FAILED: AtomicU64 = AtomicU64::new(0);

/// Provides few statistics about low level segment allocation.
///
/// This stats can be useful to debug or to export in various monitoring
/// systems.
#[derive(Debug, Default)]
pub struct MmapStats;

impl MmapStats {
    /// Get number of current segment mounted by this library.
    ///
    /// On linux there is a `systctl` limit you can access with:
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

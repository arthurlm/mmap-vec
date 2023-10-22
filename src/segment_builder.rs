use std::{
    env, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use uuid::Uuid;

/// Trait that contains everything we need to deals with unique segment creation.
pub trait SegmentBuilder: Default {
    /// Create path for new unique segment.
    fn new_segment_path(&self) -> PathBuf;
}

/// Default implementation for `SegmentBuilder` trait.
#[derive(Debug, Clone)]
pub struct DefaultSegmentBuilder {
    /// Base folder where all segment will be created.
    ///
    /// When custom segment builder is used, this struct will be clone and
    /// associated to every vec. So using `Arc` save some memory space here.
    /// Performances impact for reading it is negligible compared to new segment creation.
    store_path: Arc<PathBuf>,
}

impl DefaultSegmentBuilder {
    /// Init struct with given path.
    ///
    /// Folder needs to exists and have correct permissions.
    /// This will not be checked here and it is the responsibility of the user to do
    /// this work.
    ///
    /// In case folder does not exists segment creation may failed.
    #[inline(always)]
    pub fn with_path<P: AsRef<Path>>(store_path: P) -> Self {
        Self {
            store_path: Arc::new(store_path.as_ref().to_path_buf()),
        }
    }

    /// Make sure store folder exists.
    #[inline]
    pub fn create_dir_all(&self) -> io::Result<()> {
        fs::create_dir_all(self.store_path.as_ref())
    }
}

impl Default for DefaultSegmentBuilder {
    fn default() -> Self {
        #[cfg(not(feature = "cache-dir"))]
        let mut path = env::temp_dir();
        #[cfg(feature = "cache-dir")]
        let mut path = dirs::cache_dir().unwrap_or_else(env::temp_dir);

        path.push("mmap-vec-rs");

        let out = Self::with_path(path);

        // Ignore create dir fail
        let _ = out.create_dir_all();

        out
    }
}

impl SegmentBuilder for DefaultSegmentBuilder {
    fn new_segment_path(&self) -> PathBuf {
        let segment_id = Uuid::new_v4().as_hyphenated().to_string();
        self.store_path.join(format!("{segment_id}.seg"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniqueness() {
        let builder = DefaultSegmentBuilder::default();
        let path1 = builder.new_segment_path();
        let path2 = builder.new_segment_path();
        assert_ne!(path1, path2);
    }
}

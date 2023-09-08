use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::Segment;

/// Trait that contains everything we need to deals with unique segment creation.
pub trait SegmentBuilder: Default {
    /// Create / allocate new memory mapped segment.
    fn create_new_segment<T>(&self, capacity: usize) -> io::Result<Segment<T>>;
}

/// Default implementation for `SegmentBuilder` trait.
#[derive(Debug)]
pub struct DefaultSegmentBuilder {
    store_path: PathBuf,
}

impl DefaultSegmentBuilder {
    /// Init struct with given path.
    ///
    /// Folder needs to exists and have correct permissions.
    /// This will not be checked here and it is the responsibility of the user to do
    /// this work.
    ///
    /// In case folder does not exists segment creation may failed.
    pub fn with_path<P: AsRef<Path>>(store_path: P) -> Self {
        Self {
            store_path: store_path.as_ref().to_path_buf(),
        }
    }

    /// Make sure store folder exists.
    pub fn create_dir_all(&self) -> io::Result<()> {
        fs::create_dir_all(&self.store_path)
    }
}

impl Default for DefaultSegmentBuilder {
    fn default() -> Self {
        let path = env::temp_dir().join("mmap-vec-rs");
        let out = Self::with_path(path);

        // Ignore create dir fail
        let _ = out.create_dir_all();

        out
    }
}

impl SegmentBuilder for DefaultSegmentBuilder {
    fn create_new_segment<T>(&self, capacity: usize) -> io::Result<Segment<T>> {
        let segment_id = Uuid::new_v4().as_hyphenated().to_string();
        let path = self.store_path.join(format!("{segment_id}.seg"));
        Segment::open_rw(path, capacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniqueness() {
        let builder = DefaultSegmentBuilder::default();
        let path1 = builder.create_new_segment::<u8>(8).unwrap();
        let path2 = builder.create_new_segment::<u8>(8).unwrap();
        assert_ne!(path1.as_ptr(), path2.as_ptr());
    }
}

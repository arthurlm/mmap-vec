use std::{env, fs, io, path::PathBuf};

use uuid::Uuid;

use crate::Segment;

pub(crate) fn get_segment_dir() -> io::Result<PathBuf> {
    let path = env::temp_dir().join("mmap-vec-rs");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub(crate) fn create_unique_segment<T>(capacity: usize) -> io::Result<Segment<T>> {
    let mut path = get_segment_dir()?;
    let segment_id = Uuid::new_v4().as_hyphenated().to_string();

    path.push(format!("{segment_id}.seg"));
    Segment::open_rw(path, capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniqueness() {
        let path1 = create_unique_segment::<u8>(8).unwrap();
        let path2 = create_unique_segment::<u8>(8).unwrap();
        assert_ne!(path1.as_ptr(), path2.as_ptr());
    }
}

use std::{
    fs, io,
    ops::{Deref, DerefMut},
    path::Path,
};

use mmap_vec::Segment;

#[derive(Debug)]
pub struct TemporarySegment<T, P: AsRef<Path>> {
    seg: Option<Segment<T>>,
    path: P,
}

impl<T, P> TemporarySegment<T, P>
where
    P: AsRef<Path>,
{
    pub fn open_rw(path: P, capacity: usize) -> io::Result<Self> {
        let seg = Segment::open_rw(&path, capacity)?;
        Ok(Self {
            seg: Some(seg),
            path,
        })
    }

    pub fn into_inner(mut self) -> Segment<T> {
        let _ = fs::remove_file(&self.path);
        self.seg.take().unwrap()
    }
}

impl<T, P> Drop for TemporarySegment<T, P>
where
    P: AsRef<Path>,
{
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl<T, P> Deref for TemporarySegment<T, P>
where
    P: AsRef<Path>,
{
    type Target = Segment<T>;

    fn deref(&self) -> &Self::Target {
        self.seg.as_ref().unwrap()
    }
}

impl<T, P> DerefMut for TemporarySegment<T, P>
where
    P: AsRef<Path>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.seg.as_mut().unwrap()
    }
}

use std::{fs, path::PathBuf};

use mmap_vec::Segment;

pub use data_gen::*;

mod data_gen;

fn assert_empty(mut segment: Segment<DataRow>) {
    assert_eq!(segment.len(), 0);
    assert_eq!(segment.capacity(), 0);
    assert_eq!(segment.disk_size(), 0);
    // assert_eq!(&segment[..], &[]); // Check why this does not work with null slice.

    // Check we cannot add / pop anything
    assert_eq!(segment.push_within_capacity(ROW1), Err(ROW1));
    assert_eq!(segment.pop(), None);
}

#[test]
fn test_null() {
    assert_empty(Segment::null());
}

#[test]
fn test_open_empty_segment() {
    assert_empty(Segment::open_rw("test_open_empty_segment.seg", 0).unwrap());
}

#[test]
fn test_open_valid_segment() {
    let mut segment = Segment::open_rw("test_pull_push.seg", 3).unwrap();

    // Check initial layout.
    assert_eq!(segment.len(), 0);
    assert_eq!(segment.capacity(), 3);
    assert_eq!(segment.disk_size(), 24 * 3);
    assert_eq!(&segment[..], &[]);

    // Check we cannot pop anything.
    assert_eq!(segment.pop(), None);

    // Add few items.
    assert_eq!(segment.push_within_capacity(ROW1), Ok(()));
    assert_eq!(segment.len(), 1);
    assert_eq!(segment.capacity(), 3);
    assert_eq!(&segment[..], &[ROW1]);

    assert_eq!(segment.push_within_capacity(ROW2), Ok(()));
    assert_eq!(segment.push_within_capacity(ROW3), Ok(()));
    assert_eq!(segment.len(), 3);
    assert_eq!(segment.capacity(), 3);
    assert_eq!(&segment[..], &[ROW1, ROW2, ROW3]);

    // Add more items than segment can hold.
    assert_eq!(segment.push_within_capacity(ROW4), Err(ROW4));
    assert_eq!(segment.len(), 3);
    assert_eq!(segment.capacity(), 3);
    assert_eq!(&segment[..], &[ROW1, ROW2, ROW3]);

    // Pop everything.
    assert_eq!(segment.pop(), Some(ROW3));
    assert_eq!(segment.pop(), Some(ROW2));
    assert_eq!(segment.pop(), Some(ROW1));

    assert_eq!(segment.pop(), None);
    assert_eq!(&segment[..], &[]);

    // Add back some elements and check data are well replaced.
    assert_eq!(segment.push_within_capacity(ROW4), Ok(()));
    assert_eq!(&segment[..], &[ROW4]);

    assert_eq!(segment.pop(), Some(ROW4));
    assert_eq!(&segment[..], &[]);
}

#[test]
fn test_drop_file() {
    let path: PathBuf = "test_drop_file.seg".into();

    // Remove pre-test files.
    let _ = fs::remove_file(&path);
    assert!(!path.exists());

    // Create segment.
    let segment = Segment::<DataRow>::open_rw(&path, 3).unwrap();
    assert!(path.exists());

    // Drop segment and check file as been removed.
    drop(segment);
    assert!(!path.exists());
}

#[test]
fn test_copy() {
    let mut segment1 = Segment::open_rw("test_copy_1", 2).unwrap();
    let mut segment2 = Segment::open_rw("test_copy_2", 4).unwrap();

    // Init and check segments.
    assert_eq!(segment1.push_within_capacity(ROW1), Ok(()));
    assert_eq!(segment1.push_within_capacity(ROW2), Ok(()));
    assert_eq!(segment1.push_within_capacity(ROW3), Err(ROW3));
    assert_eq!(segment2.push_within_capacity(ROW3), Ok(()));

    assert_eq!(&segment1[..], &[ROW1, ROW2]);
    assert_eq!(&segment2[..], &[ROW3]);

    // Erase data in seg2.
    segment2.copy_from(segment1);
    assert_eq!(&segment2[..], &[ROW1, ROW2]);
}

#[test]
#[should_panic]
fn test_copy_failed() {
    let mut segment1 = Segment::<u8>::open_rw("test_copy_failed_1", 2).unwrap();
    let segment2 = Segment::<u8>::open_rw("test_copy_failed_2", 4).unwrap();

    segment1.copy_from(segment2);
}

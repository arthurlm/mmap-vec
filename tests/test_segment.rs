use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

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

    assert_eq!(&segment1[..], &[ROW1, ROW2]);
    assert_eq!(&segment2[..], &[]);

    // Erase data in seg2.
    segment2.fill_from(segment1);
    assert_eq!(&segment2[..], &[ROW1, ROW2]);
}

#[test]
#[should_panic]
fn test_copy_already_filled() {
    let mut segment1 = Segment::open_rw("test_copy_already_filled_1", 2).unwrap();
    let mut segment2 = Segment::open_rw("test_copy_already_filled_2", 4).unwrap();

    assert_eq!(segment1.push_within_capacity(ROW1), Ok(()));
    assert_eq!(segment2.push_within_capacity(ROW2), Ok(()));

    segment2.fill_from(segment1);
}

#[test]
#[should_panic]
fn test_copy_bad_capacity() {
    let mut segment1 = Segment::<u8>::open_rw("test_copy_bad_capacity_1", 2).unwrap();
    let segment2 = Segment::<u8>::open_rw("test_copy_bad_capacity_2", 4).unwrap();

    segment1.fill_from(segment2);
}

#[test]
fn test_drop() {
    let mut segment = Segment::<DroppableRow>::open_rw("test_drop", 5).unwrap();
    let counter = Arc::new(AtomicU8::new(0));

    // Check push / pull inc
    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    segment.pop();
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    // Check drop inc
    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    drop(segment);
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

#[test]
fn test_truncate() {
    let mut segment = Segment::<DroppableRow>::open_rw("test_truncate", 5).unwrap();
    let counter = Arc::new(AtomicU8::new(0));

    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(segment.len(), 3);

    // Trigger with too high value
    segment.truncate(500000);
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(segment.len(), 3);

    // Trigger resize
    segment.truncate(2);
    assert_eq!(segment.len(), 2);
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    segment.truncate(0);
    assert_eq!(segment.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 3);

    // Trigger on empty segment
    segment.truncate(0);
    assert_eq!(segment.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 3);
}

#[test]
fn test_truncate_first() {
    // Truncate on empty segment
    {
        let mut segment = Segment::<u8>::open_rw("test_truncate_first", 5).unwrap();
        assert_eq!(&segment[..], []);

        segment.truncate_first(0);
        assert_eq!(&segment[..], []);

        segment.truncate_first(3);
        assert_eq!(&segment[..], []);

        segment.truncate_first(10);
        assert_eq!(&segment[..], []);
    }

    fn build_test_seg() -> Segment<u8> {
        let mut segment = Segment::<u8>::open_rw("test_truncate_first", 5).unwrap();
        segment.push_within_capacity(1).unwrap();
        segment.push_within_capacity(2).unwrap();
        segment.push_within_capacity(6).unwrap();
        segment.push_within_capacity(4).unwrap();
        assert_eq!(&segment[..], [1, 2, 6, 4]);
        segment
    }

    // Truncate 0 on with data segment
    {
        let mut segment = build_test_seg();
        segment.truncate_first(0);
        assert_eq!(&segment[..], [1, 2, 6, 4]);
    }

    // Truncate half on with data segment
    {
        let mut segment = build_test_seg();
        segment.truncate_first(2);
        assert_eq!(&segment[..], [6, 4]);
    }

    // Truncate almost everything on with data segment
    {
        let mut segment = build_test_seg();
        segment.truncate_first(3);
        assert_eq!(&segment[..], [4]);
    }

    // Truncate everything on with data segment
    {
        let mut segment = build_test_seg();
        segment.truncate_first(4);
        assert_eq!(&segment[..], []);
    }

    // Truncate above capacity on segment with data
    {
        let mut segment = build_test_seg();
        segment.truncate_first(500);
        assert_eq!(&segment[..], []);
    }
}

#[test]
fn test_drop_with_truncate_first() {
    let counter = Arc::new(AtomicU8::new(0));

    fn build_test_seg(counter: Arc<AtomicU8>) -> Segment<DroppableRow> {
        counter.store(0, Ordering::Relaxed);

        let mut segment = Segment::open_rw("test_drop_with_truncate_first", 5).unwrap();
        segment
            .push_within_capacity(DroppableRow::new(counter.clone()))
            .unwrap();
        segment
            .push_within_capacity(DroppableRow::new(counter.clone()))
            .unwrap();
        segment
            .push_within_capacity(DroppableRow::new(counter.clone()))
            .unwrap();
        segment
            .push_within_capacity(DroppableRow::new(counter.clone()))
            .unwrap();
        assert_eq!(segment.len(), 4);
        segment
    }

    // Truncate 0 on with data segment
    {
        let mut segment = build_test_seg(counter.clone());

        segment.truncate_first(0);
        assert_eq!(counter.load(Ordering::Relaxed), 0);

        drop(segment);
        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }

    // Truncate half on with data segment
    {
        let mut segment = build_test_seg(counter.clone());

        segment.truncate_first(2);
        assert_eq!(counter.load(Ordering::Relaxed), 2);

        drop(segment);
        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }

    // Truncate almost everything on with data segment
    {
        let mut segment = build_test_seg(counter.clone());

        segment.truncate_first(3);
        assert_eq!(counter.load(Ordering::Relaxed), 3);

        drop(segment);
        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }

    // Truncate everything on with data segment
    {
        let mut segment = build_test_seg(counter.clone());

        segment.truncate_first(4);
        assert_eq!(counter.load(Ordering::Relaxed), 4);

        drop(segment);
        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }

    // Truncate above capacity on segment with data
    {
        let mut segment = build_test_seg(counter.clone());

        segment.truncate_first(500);
        assert_eq!(counter.load(Ordering::Relaxed), 4);

        drop(segment);
        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }
}

#[test]
fn test_clear() {
    let mut segment = Segment::<DroppableRow>::open_rw("test_clear", 5).unwrap();
    let counter = Arc::new(AtomicU8::new(0));

    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert!(segment
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(segment.len(), 2);

    // Trigger cleanup
    segment.clear();
    assert_eq!(segment.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 2);

    // Trigger on empty segment
    segment.clear();
    assert_eq!(segment.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

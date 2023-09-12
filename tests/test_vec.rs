use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use mmap_vec::MmapVec;

pub use data_gen::*;

mod data_gen;

#[test]
fn test_resize() {
    let mut v = MmapVec::<DataRow>::new();
    assert_eq!(v.capacity(), 0);

    // Trigger first growth
    v.push(ROW1).unwrap();
    assert_eq!(v.capacity(), 170);
    assert_eq!(v[0], ROW1);
    assert_eq!(&v[..], &[ROW1]);

    // Fill vec
    while v.len() < v.capacity() {
        v.push(ROW1).unwrap();
    }
    assert_eq!(v.capacity(), 170);

    // Trigger second growth
    v.push(ROW2).unwrap();
    assert_eq!(v.capacity(), 340);

    // Fill vec
    while v.len() < v.capacity() {
        v.push(ROW1).unwrap();
    }
    assert_eq!(v.capacity(), 340);

    // Trigger third growth
    v.push(ROW2).unwrap();
    assert_eq!(v.capacity(), 680);
}

#[test]
fn test_with_capacity() {
    let v = MmapVec::<DataRow>::with_capacity(500).unwrap();
    assert_eq!(v.capacity(), 500);
}

#[test]
fn test_assign() {
    let mut v = MmapVec::<DataRow>::new();

    v.push(ROW1).unwrap();
    assert_eq!(&v[..], &[ROW1]);

    v[0] = ROW2;
    assert_eq!(&v[..], &[ROW2]);
}

#[test]
fn test_pop() {
    let mut v = MmapVec::<DataRow>::new();
    v.push(ROW1).unwrap();
    v.push(ROW2).unwrap();
    assert_eq!(&v[..], &[ROW1, ROW2]);

    assert_eq!(v.pop(), Some(ROW2));
    assert_eq!(v.pop(), Some(ROW1));
    assert_eq!(v.pop(), None);
}

#[test]
fn test_drop() {
    let mut v = MmapVec::<DroppableRow>::new();
    let counter = Arc::new(AtomicU8::new(0));

    // Check push / pull inc
    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    v.pop();
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    // Check drop inc
    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    drop(v);
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

#[test]
fn test_truncate() {
    let mut v = MmapVec::<DroppableRow>::new();
    let counter = Arc::new(AtomicU8::new(0));

    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(v.len(), 3);

    // Trigger with too high value
    v.truncate(500000);
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(v.len(), 3);

    // Trigger resize
    v.truncate(2);
    assert_eq!(v.len(), 2);
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    v.truncate(0);
    assert_eq!(v.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 3);

    // Trigger on empty segment
    v.truncate(0);
    assert_eq!(v.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 3);
}

#[test]
fn test_truncate_first() {
    fn build_vec() -> MmapVec<u8> {
        let mut output = MmapVec::new();
        assert!(output.push(8).is_ok());
        assert!(output.push(5).is_ok());
        assert!(output.push(3).is_ok());
        assert!(output.push(12).is_ok());
        assert_eq!(&output[..], &[8, 5, 3, 12]);
        output
    }

    // Truncate 0
    {
        let mut v = build_vec();
        v.truncate_first(0);
        assert_eq!(&v[..], [8, 5, 3, 12]);
    }

    // Truncate half
    {
        let mut v = build_vec();
        v.truncate_first(2);
        assert_eq!(&v[..], [3, 12]);
    }

    // Truncate len
    {
        let mut v = build_vec();
        v.truncate_first(v.len());
        assert_eq!(&v[..], []);
    }

    // Truncate too much
    {
        let mut v = build_vec();
        v.truncate_first(v.len() + 1000);
        assert_eq!(&v[..], []);
    }
}

#[test]
fn test_clear() {
    let mut v = MmapVec::<DroppableRow>::new();
    let counter = Arc::new(AtomicU8::new(0));

    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert!(v.push(DroppableRow::new(counter.clone())).is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);
    assert_eq!(v.len(), 2);

    // Trigger cleanup
    v.clear();
    assert_eq!(v.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 2);

    // Trigger on empty segment
    v.clear();
    assert_eq!(v.len(), 0);
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

#[test]
fn test_equals() {
    let mut s1 = MmapVec::<i32>::new();
    let mut s2 = MmapVec::<i32>::new();

    // Check when empty.
    assert_eq!(s1, s2);

    // Check with different size.
    s1.push(42).unwrap();
    s1.push(17).unwrap();
    assert_ne!(s1, s2);

    // Check equals again but with data this time.
    s2.push(42).unwrap();
    s2.push(17).unwrap();
    assert_eq!(s1, s2);

    // Check different data.
    s1.push(15).unwrap();
    s2.push(-15).unwrap();
    assert_ne!(s1, s2);
}

#[test]
fn test_try_clone_null() {
    let mut s1 = MmapVec::<i32>::default();
    assert_eq!(s1.capacity(), 0);

    // Clone and check equals !
    let mut s2 = s1.try_clone().unwrap();
    assert_eq!(s2.capacity(), 0);
    assert_eq!(s1, s2);

    // Push data and check segment are different.
    s1.push(-8).unwrap();
    s2.push(93).unwrap();

    assert_eq!(&s1[..], [-8]);
    assert_eq!(&s2[..], [93]);
}

#[test]
fn test_try_clone_with_data() {
    let mut s1 = MmapVec::<i32>::new();
    s1.push(42).unwrap();
    s1.push(17).unwrap();

    // Clone and check equals !
    let mut s2 = s1.try_clone().unwrap();
    assert_eq!(s1, s2);
    assert_eq!(&s1[..], [42, 17]);

    // Push data and check segment are different.
    s1.push(-8).unwrap();
    s2.push(93).unwrap();

    assert_eq!(&s1[..], [42, 17, -8]);
    assert_eq!(&s2[..], [42, 17, 93]);
}

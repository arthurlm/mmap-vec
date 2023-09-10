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

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

    v.push(ROW1).unwrap();
    assert_eq!(v.capacity(), 1);
    assert_eq!(v[0], ROW1);
    assert_eq!(&v[..], &[ROW1]);

    v.push(ROW2).unwrap();
    assert_eq!(v.capacity(), 2);
    assert_eq!(&v[..], &[ROW1, ROW2]);

    v.push(ROW3).unwrap();
    assert_eq!(v.capacity(), 4);
    assert_eq!(&v[..], &[ROW1, ROW2, ROW3]);
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

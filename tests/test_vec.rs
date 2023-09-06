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

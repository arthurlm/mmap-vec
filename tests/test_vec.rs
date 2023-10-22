use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use mmap_vec::{DefaultSegmentBuilder, MmapVec};

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
    let counter = Arc::new(AtomicU32::new(0));

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
    let counter = Arc::new(AtomicU32::new(0));

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
    let counter = Arc::new(AtomicU32::new(0));

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

#[test]
fn test_advice_prefetch() {
    // Test prefetch with null
    {
        let v = MmapVec::<i32>::new();
        v.advice_prefetch_all_pages();
        v.advice_prefetch_page_at(0);
        v.advice_prefetch_page_at(42);
    }

    // Test prefetch wih no data
    {
        let v = MmapVec::<i32>::new();
        v.advice_prefetch_all_pages();
        v.advice_prefetch_page_at(0);
        v.advice_prefetch_page_at(18);
        v.advice_prefetch_page_at(25);
    }

    // Test prefetch with data
    {
        let mut v = MmapVec::<i32>::new();
        assert!(v.push(5).is_ok());
        assert!(v.push(9).is_ok());
        assert!(v.push(2).is_ok());
        assert!(v.push(8).is_ok());
        v.advice_prefetch_all_pages();
        v.advice_prefetch_page_at(0);
        v.advice_prefetch_page_at(18);
        v.advice_prefetch_page_at(25);
    }
}

#[test]
fn test_reserve_in_place() {
    const PAGE_SIZE: usize = 4096;

    // Test on null segment
    {
        let mut s = MmapVec::<i32>::new();
        assert_eq!(s.capacity(), 0);
        s.reserve(50).unwrap();
        assert_eq!(s.capacity(), 1024);
    }

    // Test on valid segment with free space
    {
        let mut s = MmapVec::<i32>::with_capacity(100).unwrap();
        assert_eq!(s.capacity(), 100);

        assert!(s.reserve(50).is_ok());
        assert_eq!(s.capacity(), 100);
    }

    // Test on valid segment with free space
    {
        // Fill the vec
        let mut s = MmapVec::<i32>::with_capacity(100).unwrap();
        assert_eq!(s.capacity(), 100);

        // Reserve few bytes and check rounding
        while s.len() < s.capacity() {
            assert_eq!(s.push_within_capacity(0), Ok(()));
        }

        assert!(s.reserve(50).is_ok());
        assert_eq!(s.capacity(), 1024);
        assert_eq!(s.disk_size(), PAGE_SIZE);

        // Reserve one full page
        while s.len() < s.capacity() {
            assert_eq!(s.push_within_capacity(0), Ok(()));
        }

        assert!(s.reserve(1024).is_ok());
        assert_eq!(s.capacity(), 2048);
        assert_eq!(s.disk_size(), 2 * PAGE_SIZE);

        // Reserve a single byte
        while s.len() < s.capacity() {
            assert_eq!(s.push_within_capacity(0), Ok(()));
        }

        assert!(s.reserve(1).is_ok());
        assert_eq!(s.capacity(), 3072);
        assert_eq!(s.disk_size(), 3 * PAGE_SIZE);
    }
}

#[test]
fn test_reserve_in_place_drop() {
    let mut s = MmapVec::<DroppableRow>::with_capacity(100).unwrap();
    let counter = Arc::new(AtomicU32::new(0));

    // Fill vec
    while s.len() < s.capacity() {
        assert!(s
            .push_within_capacity(DroppableRow::new(counter.clone()))
            .is_ok());
    }
    assert_eq!(s.capacity(), 100);
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    // Trigger resize
    assert!(s.reserve(50).is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    // Fill vec again
    assert!(s
        .push_within_capacity(DroppableRow::new(counter.clone()))
        .is_ok());
    assert_eq!(s.capacity(), 512);
    assert_eq!(s.len(), 101);
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    drop(s);
    assert_eq!(counter.load(Ordering::Relaxed), 101);
}

#[test]
fn test_drop_file() {
    // Create vec.
    let vec = MmapVec::<i32>::with_capacity(100).unwrap();
    let path = vec.path();
    assert!(path.exists());

    // Drop vec and check file as been removed.
    drop(vec);
    assert!(!path.exists());
}

#[test]
fn test_try_from_array() {
    let vec = MmapVec::<_, DefaultSegmentBuilder>::try_from([8, 6, 4, -48, 16]).unwrap();
    assert_eq!(&vec[..], [8, 6, 4, -48, 16]);
}

#[test]
fn test_try_from_slice() {
    let vec = MmapVec::<_, DefaultSegmentBuilder>::try_from([8, 6, 4, -48, 16].as_slice()).unwrap();
    assert_eq!(&vec[..], [8, 6, 4, -48, 16]);
}

#[test]
fn test_try_from_vec() {
    let vec = MmapVec::<_, DefaultSegmentBuilder>::try_from(Vec::from([8, 6, 4, -48, 16])).unwrap();
    assert_eq!(&vec[..], [8, 6, 4, -48, 16]);
}

#[test]
#[should_panic = "Zero sized type are not supported"]
fn test_zero_sized_type() {
    struct VoidStruct;

    let _vec = MmapVec::<VoidStruct>::with_capacity(50).unwrap();
}

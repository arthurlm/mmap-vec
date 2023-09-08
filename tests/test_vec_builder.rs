use mmap_vec::MmapVecBuilder;

#[test]
fn test_capacity() {
    let v = MmapVecBuilder::<u8>::new().build().unwrap();
    assert_eq!(v.capacity(), 4096);

    let v = MmapVecBuilder::<u64>::new().build().unwrap();
    assert_eq!(v.capacity(), 512);

    let v = MmapVecBuilder::<i64>::new().build().unwrap();
    assert_eq!(v.capacity(), 512);

    let v = MmapVecBuilder::<i64>::new().capacity(128).build().unwrap();
    assert_eq!(v.capacity(), 128);
}

use mmap_vec::MmapVecBuilder;

#[test]
fn test_capacity() {
    let v = MmapVecBuilder::<u8>::new().try_build().unwrap();
    assert_eq!(v.capacity(), 4096);

    let v = MmapVecBuilder::<u64>::new().try_build().unwrap();
    assert_eq!(v.capacity(), 512);

    let v = MmapVecBuilder::<i64>::new().try_build().unwrap();
    assert_eq!(v.capacity(), 512);

    let v = MmapVecBuilder::<i64>::new()
        .capacity(128)
        .try_build()
        .unwrap();
    assert_eq!(v.capacity(), 128);
}

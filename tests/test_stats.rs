use mmap_vec::{MmapStats, MmapVec};

#[test]
fn test_stats() {
    assert_eq!(MmapStats.active_segment(), 0);
    assert_eq!(MmapStats.map_failed(), 0);
    assert_eq!(MmapStats.unmap_failed(), 0);

    let v = MmapVec::<u8>::with_capacity(500).unwrap();
    assert_eq!(MmapStats.active_segment(), 1);
    assert_eq!(MmapStats.map_failed(), 0);
    assert_eq!(MmapStats.unmap_failed(), 0);

    drop(v);
    assert_eq!(MmapStats.active_segment(), 0);
    assert_eq!(MmapStats.map_failed(), 0);
    assert_eq!(MmapStats.unmap_failed(), 0);
}

use mmap_vec::{MmapStats, MmapVec};

#[test]
fn test_stats() {
    let stats = MmapStats::default();

    assert_eq!(stats.active_segment(), 0);
    assert_eq!(stats.ftruncate_failed(), 0);
    assert_eq!(stats.map_failed(), 0);
    assert_eq!(stats.unmap_failed(), 0);
    assert_eq!(
        format!("{stats:?}"),
        "MmapStats { active: 0, ftruncate_failed: 0, map_failed: 0, unmap_failed: 0 }"
    );

    let v = MmapVec::<u8>::with_capacity(500).unwrap();
    assert_eq!(stats.active_segment(), 1);
    assert_eq!(stats.ftruncate_failed(), 0);
    assert_eq!(stats.map_failed(), 0);
    assert_eq!(stats.unmap_failed(), 0);
    assert_eq!(
        format!("{stats:?}"),
        "MmapStats { active: 1, ftruncate_failed: 0, map_failed: 0, unmap_failed: 0 }"
    );

    drop(v);
    assert_eq!(stats.active_segment(), 0);
    assert_eq!(stats.ftruncate_failed(), 0);
    assert_eq!(stats.map_failed(), 0);
    assert_eq!(stats.unmap_failed(), 0);
    assert_eq!(
        format!("{stats:?}"),
        "MmapStats { active: 0, ftruncate_failed: 0, map_failed: 0, unmap_failed: 0 }"
    );
}

use glob::glob;
use mmap_vec::{DefaultSegmentBuilder, MmapVec};

fn get_seg_count() -> usize {
    let mut count = 0;
    for entry in glob("/tmp/test_custom_segment_builder/*.seg").expect("Fail to list segment") {
        if entry.is_ok() {
            count += 1;
        }
    }
    count
}

#[test]
fn test_custom_segment_builder() {
    let builder = DefaultSegmentBuilder::with_path("/tmp/test_custom_segment_builder");
    builder.create_dir_all().unwrap();

    let start_file_count = get_seg_count();

    // Create new segment
    let mut v = MmapVec::with_capacity_and_builder(500, builder).unwrap();
    v.push(42).unwrap();

    let new_file_count = get_seg_count();
    assert!(
        new_file_count >= (start_file_count + 1),
        "{} >= {}",
        new_file_count,
        start_file_count + 1
    );

    // Drop it
    drop(v);
    assert_eq!(get_seg_count(), start_file_count);
}

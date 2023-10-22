#[cfg(feature = "serde")]
use mmap_vec::MmapVec;

#[test]
#[cfg(feature = "serde")]
fn test_serializer() {
    {
        let vec = MmapVec::<u32>::new();
        assert_eq!(serde_json::to_string(&vec).unwrap(), "[]");
    }
    {
        let mut vec = MmapVec::<u32>::new();
        vec.push(42).unwrap();
        vec.push(8).unwrap();
        vec.push(52).unwrap();
        assert_eq!(serde_json::to_string(&vec).unwrap(), "[42,8,52]");
    }
}

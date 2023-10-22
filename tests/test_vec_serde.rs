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

#[test]
#[cfg(feature = "serde")]
fn test_deserialize() {
    {
        let vec: MmapVec<u32> = serde_json::from_str("[]").unwrap();
        assert_eq!(&vec[..], Vec::<u32>::new());
    }
    {
        let vec: MmapVec<u32> = serde_json::from_str("[8,6,42]").unwrap();
        assert_eq!(&vec[..], [8, 6, 42]);
    }
}

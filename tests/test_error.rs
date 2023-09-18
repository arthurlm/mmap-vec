use std::io;

use mmap_vec::MmapVecError;

#[test]
fn test_display() {
    assert_eq!(
        format!("{}", MmapVecError::MissingSegmentPath),
        "missing segment path"
    );
    assert_eq!(
        format!("{}", MmapVecError::Io("foo".to_string())),
        "I/O: foo"
    );
}

#[test]
fn test_convert() {
    let custom_io_error: MmapVecError = io::Error::new(io::ErrorKind::Other, "oh no!").into();
    assert_eq!(custom_io_error, MmapVecError::Io("oh no!".to_string()));
}

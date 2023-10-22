use std::mem;

pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

pub const fn check_zst<T>() {
    if mem::size_of::<T>() == 0 {
        panic!("Zero sized type are not supported with MmapVec. What is the point of mapping ZST to disk ?");
    }
}

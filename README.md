# Rust memory mapped vector

[![CI Status](https://github.com/arthurlm/mmap-vec/workflows/Test/badge.svg)](https://github.com/arthurlm/mmap-vec/actions/)
[![codecov](https://codecov.io/gh/arthurlm/mmap-vec/graph/badge.svg?token=1TXRTK3C3Q)](https://codecov.io/gh/arthurlm/mmap-vec)
[![docs.rs](https://docs.rs/mmap-vec/badge.svg)](https://docs.rs/mmap-vec/)
[![Crates.io](https://img.shields.io/crates/v/mmap-vec)](https://crates.io/crates/mmap-vec)
[![LICENSE](https://img.shields.io/crates/l/mmap-vec)](https://raw.githubusercontent.com/arthurlm/mmap-vec/main/LICENSE)
![MSRV](https://img.shields.io/badge/MSRV-1.66.1-blue)
[![dependency status](https://deps.rs/repo/github/arthurlm/mmap-vec/status.svg)](https://deps.rs/repo/github/arthurlm/mmap-vec)

This crate contains implementation / helper to create data struct that are memory mapped.

Sometime, you have to deal with vector / data that cannot fit in memory.
Moving them to disk and memory map them is a good way to deal with this problem.

## How to use it ?

That is so simple !

```rust
use mmap_vec::MmapVec;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Row {
    id: usize,
    age: u8,
}

let row1 = Row { id: 42, age: 18 };
let row2 = Row { id: 894, age: 99 };

// Create a memory mapped vec 😎
let mut v = MmapVec::<Row>::new();

// Push can trigger new mmap segment creation, so it can fail.
v.push(row1).unwrap();
v.push(row2).unwrap();

// Check the content
assert_eq!(v[0], row1);
assert_eq!(&v[..], &[row1, row2]);

// Pop content
assert_eq!(v.pop(), Some(row2));
assert_eq!(v.pop(), Some(row1));
```

Check the unit tests for more example.

## How it works ?

The main idea here is to provide a basic `struct Segment`.

This struct provides constant size memory mapped array of type `T`.
Wrapping `Segment` into a new struct `MmapVec` that handle segment growth / shrink does the trick.

### Where does the segment are store on disk ?

For now data are stored in `.cache` (if using 'cache-dirs' feature) or `/tmp` under a dedicated folder.

UUID V4 are generated in order to avoid collision when creating segment.

```
❯ ls /tmp/mmap-vec-rs -1
/tmp/mmap-vec-rs/00d977bf-b556-475e-8de5-d35e7baaa39d.seg
/tmp/mmap-vec-rs/6cb81228-9cf3-4918-a3ef-863907b32830.seg
/tmp/mmap-vec-rs/8a86eeaa-1fa8-4535-9e23-6c59e0c9c376.seg
/tmp/mmap-vec-rs/de62bde3-6524-4c4b-b514-24f6a44d6323.seg
```

### Does segment creation is configurable ?

Yes ! Check out `test_custom_segment_creator::test_custom_segment_builder` for example.

Since segment creation are manage through a trait. You are free to configure it the way you want.

### Does this work on Windows ?

__Nope__. I am not targeting this OS and would like to keep this crate as simple as possible.

I also would like to reduce dependencies as much as possible.

```
❯ cargo tree
mmap-vec v0.1.1
├── libc v0.2.147
├── uuid v1.4.1
|   └── getrandom v0.2.10
|       ├── cfg-if v1.0.0
|       └── libc v0.2.147
# Optional using 'cache-dir' feature
├── dirs v5.0.1
│   └── dirs-sys v0.4.1
│       ├── libc v0.2.147
│       └── option-ext v0.2.0
[dev-dependencies]
└── glob v0.3.1
```

### Is this crate production ready ?

Yes 😁 !
Since v0.1.1. But feature are a little bit limited for now ...

Github PR to help on this are welcomed !

Prefetching API is not fully stable for now and may change in the future.

## Ideas / new features ?

- Implement custom `std::alloc::Allocator` to use with `std::vec::Vec`

License: MIT

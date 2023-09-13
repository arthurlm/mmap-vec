/*
Test with:

    docker run -it --rm --name io-bencher -v /home/$(whoami)/.cache:/root/.cache app io_bencher

 */

use std::{
    io::{self, Write},
    time::Instant,
};

use mmap_vec::MmapVec;
use rand::prelude::*;

fn print_time<F>(name: &str, f: F)
where
    F: FnOnce(),
{
    print!("Testing {name}: ");
    io::stdout().flush().unwrap();

    let timer = Instant::now();
    f();
    let elapsed = timer.elapsed();

    println!("DONE in {:?}", elapsed);
}

fn main() {
    let mut rng = thread_rng();

    let mut v = MmapVec::<i64>::with_capacity(1 << 30).expect("Fail to allocate mmap vector");
    // let mut v = Vec::<i64>::with_capacity(1 << 30);

    v.advice_prefetch_page_at(0);
    print_time("write sequential", || {
        for i in 0..v.capacity() {
            assert!(v.push_within_capacity(i as i64).is_ok());
            // v.push(i as i64);
        }
    });

    v.advice_prefetch_page_at(0);
    print_time("read  sequential", || {
        for i in 0..v.capacity() {
            assert_eq!(v[i], i as i64);
        }
    });

    const RAND_COUNT: usize = 1 << 15;

    let indexes: Vec<_> = (0..RAND_COUNT)
        .map(|_| {
            let index = rng.gen::<usize>() % v.len();
            v.advice_prefetch_page_at(index);
            index
        })
        .collect();

    print_time("write rand (8Mb values)", || {
        for i in &indexes {
            v[*i] = *i as i64;
        }
    });
    print_time("read  rand (8Mb values)", || {
        for i in &indexes {
            assert_eq!(v[*i], *i as i64);
        }
    });
}

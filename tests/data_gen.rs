use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DataRow {
    pub bid: f64,
    pub ask: f64,
    pub volume: u32,
}

const _: () = {
    if std::mem::size_of::<DataRow>() != 24 {
        panic!("invalid DataRow size");
    }
};

pub const ROW1: DataRow = DataRow {
    bid: 8.3,
    ask: 12.5,
    volume: 1000,
};
pub const ROW2: DataRow = DataRow {
    bid: 14.3,
    ask: 18.25,
    volume: 1234,
};
pub const ROW3: DataRow = DataRow {
    bid: -8.5,
    ask: 9.6,
    volume: 102,
};
pub const ROW4: DataRow = DataRow {
    bid: -8.3,
    ask: 6.89,
    volume: 106,
};

#[derive(Debug, Clone)]
pub struct DroppableRow {
    counter: Arc<AtomicU32>,
}

impl DroppableRow {
    pub fn new(counter: Arc<AtomicU32>) -> Self {
        Self { counter }
    }
}

impl Drop for DroppableRow {
    fn drop(&mut self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

use std::sync::{Arc, atomic::{AtomicI32, Ordering}};


pub struct AlphaAddonInterface {
    cc: Arc<AtomicI32>
}

impl AlphaAddonInterface {
    pub fn new(counter: Arc<AtomicI32>) -> Self {
        Self { cc: counter }
    }

    pub fn get_counter(&self) -> i32 {
        self.cc.load(Ordering::Relaxed)
    }

    pub fn set_counter(&self, v: i32) {
        self.cc.store(v, Ordering::Relaxed);
    }
}

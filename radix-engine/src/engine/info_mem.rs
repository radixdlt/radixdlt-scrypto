use std::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{GlobalAlloc, Layout};


pub struct InfoAlloc<T: GlobalAlloc> {
    allocator: T,
    alloc_counter: AtomicUsize
}


impl<T: GlobalAlloc> InfoAlloc<T> {

    pub const fn new(allocator: T) -> Self {
        InfoAlloc {
            allocator,
            alloc_counter: AtomicUsize::new(0)
        }
    }

    pub fn reset_counter(&self) {
        self.alloc_counter.store(0, Ordering::Relaxed);
    }

    #[inline]
    fn increase_counter(&self, value: usize) {
        self.alloc_counter.fetch_add(value, Ordering::Relaxed);
    }

    #[inline]
    fn decrease_counter(&self, value: usize) {
        self.alloc_counter.fetch_sub(value, Ordering::Relaxed);
    }

    pub fn get_counter_value(&self) -> usize {
        self.alloc_counter.load(Ordering::Relaxed)
    }
}


unsafe impl<T: GlobalAlloc> GlobalAlloc for InfoAlloc<T> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.increase_counter(layout.size());
        self.allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.decrease_counter(layout.size());
        self.allocator.realloc(ptr, layout, new_size)
    }
}


use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicIsize, Ordering},
};

#[global_allocator]
pub static INFO_ALLOC: InfoAlloc<System> = InfoAlloc::new(System);

/// Heap allocations tracker
///
/// This allocator information provider can count allocations up to isize::MAX (9_223_372_036_854_775_807),
/// in case of exceeding this value code will panic with message: 'Value out of range'.
pub struct InfoAlloc<T: GlobalAlloc> {
    /// Heap allocator to use, default usage: System
    allocator: T,
    /// Sum of bytes allocated during measurements (for reallocations only additional memory is counted, no dealocation is counted)
    sum_counter: AtomicIsize,
    /// Current level of allocated bytes (allocation and deallocation are counted, incl. reallocation)
    current_level: AtomicIsize,
    /// Maximum level (peak) of allocated bytes (the max of this field and current_level)
    max_level: AtomicIsize,
}

impl<T: GlobalAlloc> InfoAlloc<T> {
    pub const fn new(allocator: T) -> Self {
        InfoAlloc {
            allocator,
            sum_counter: AtomicIsize::new(0),
            current_level: AtomicIsize::new(0),
            max_level: AtomicIsize::new(0),
        }
    }

    /// Resets internal counters. Usually used to start measurement.
    pub fn reset_counters(&self) {
        self.sum_counter.store(0, Ordering::Release);
        self.current_level.store(0, Ordering::Release);
        self.max_level.store(0, Ordering::Release);
    }

    #[inline]
    fn increase_counters(&self, value: usize) {
        let ivalue: isize = value.try_into().expect("Value out of range");

        self.sum_counter.fetch_add(ivalue, Ordering::AcqRel);

        let old_value = self.current_level.fetch_add(ivalue, Ordering::AcqRel);
        self.max_level
            .fetch_max(old_value + ivalue, Ordering::AcqRel);
    }

    #[inline]
    fn decrease_counters(&self, value: usize) {
        self.current_level.fetch_sub(
            value.try_into().expect("Value out of range"),
            Ordering::AcqRel,
        );
    }

    #[inline]
    fn realloc_decrease_counter(&self, old_size: usize, new_size: usize) {
        let old_size: isize = old_size.try_into().expect("Value out of range");
        let new_size: isize = new_size.try_into().expect("Value out of range");
        self.current_level
            .fetch_sub(old_size - new_size, Ordering::AcqRel);

        if new_size > old_size {
            self.sum_counter
                .fetch_add(new_size - old_size, Ordering::AcqRel);

            let current_level = self.current_level.load(Ordering::Acquire);
            self.max_level.fetch_max(current_level, Ordering::AcqRel);
        }
    }

    /// Returns current counters values: sum fo all allocations, current allocation level, peak allocation level
    /// Counters can have negative values because of memory allocations before calling to reset_counters() function
    /// and deallocating them during measurements. In that case counter value is set to 0.
    pub fn get_counters_value(&self) -> (usize, usize, usize) {
        (
            self.sum_counter.load(Ordering::Acquire).max(0) as usize,
            self.current_level.load(Ordering::Acquire).max(0) as usize,
            self.max_level.load(Ordering::Acquire).max(0) as usize,
        )
    }
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for InfoAlloc<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.increase_counters(layout.size());
        self.allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.decrease_counters(layout.size());
        self.allocator.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.realloc_decrease_counter(layout.size(), new_size);
        self.allocator.realloc(ptr, layout, new_size)
    }
}

#[test]
fn info_mem_check() {
    INFO_ALLOC.reset_counters();

    // allocate 10 bytes
    let mut v = Vec::<u8>::with_capacity(10);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (10, 10, 10));

    // no allocation/deallocation occurs
    v.push(10);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (10, 10, 10));

    // deallocate 9 bytes
    v.shrink_to_fit();
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (10, 1, 10));

    // allocate 3 bytes
    let _a = Box::new(1u8);
    let _b = Box::new(1u8);
    let _c = Box::new(1u8);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (13, 4, 10));

    // allocate 10 bytes
    let mut v = Vec::<u8>::with_capacity(10);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (23, 14, 14));

    // no allocation/deallocation occurs
    v.push(10);

    // deallocate 9 bytes
    v.shrink_to_fit();
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (23, 5, 14));

    // allocate 10 bytes
    let mut v = Vec::<u8>::with_capacity(10);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (33, 15, 15));

    // allocate 10 bytes (by default capacity of vector is extended by 2)
    v.extend([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (43, 25, 25));

    // deallocate 9 bytes
    v.shrink_to_fit();
    let (sum, current, peak) = INFO_ALLOC.get_counters_value();
    assert_eq!((sum, current, peak), (43, 16, 25));
}

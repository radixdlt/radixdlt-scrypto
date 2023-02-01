use crate::transaction::ResourcesUsage;
use crate::types::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicIsize, Ordering},
};
use perfcnt::{
    linux::{HardwareEventType, PerfCounterBuilderLinux},
    AbstractPerfCounter, PerfCounter,
};

#[global_allocator]
static INFO_ALLOC: InfoAlloc<System> = InfoAlloc::new(System);

/// Heap allocations tracker
///
/// This allocator information provider can count allocations up to isize::MAX (9_223_372_036_854_775_807),
/// in case of exceeding this value code will panic with message: 'Value out of range'.
pub struct InfoAlloc<T: GlobalAlloc> {
    /// Heap allocator to use, default usage: System
    allocator: T,
    /// Sum of bytes allocated during measurements (no dealocation is counted)
    sum_counter: AtomicIsize,
    /// Current level of allocated bytes (allocation and deallocation are counted)
    current_level: AtomicIsize,
    /// Maximum level (peak) of allocated bytes (allocation and deallocation are counted)
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
    fn realloc_decrease_counter(&self, value: usize) {
        self.sum_counter.fetch_sub(
            value.try_into().expect("Value out of range"),
            Ordering::AcqRel,
        );
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
        self.realloc_decrease_counter(layout.size());
        self.allocator.realloc(ptr, layout, new_size)
    }
}

/// CPU cycles tracker
///
/// Performance counters are used to read Reference CPU cycles.
pub struct InfoCpu {
    perf: PerfCounter,
}

impl InfoCpu {
    pub fn new() -> Self {
        Self {
            perf: PerfCounterBuilderLinux::from_hardware_event(HardwareEventType::RefCPUCycles)
                .finish()
                .expect("Failed to initialize CPU performance counter"),
        }
    }

    pub fn start_measurement(&self) {
        self.perf
            .start()
            .expect("Failed to start CPU performance counter");
    }

    pub fn end_measurement(&mut self) -> u64 {
        self.perf
            .stop()
            .expect("Failed to stop CPU performance counter");
        self.perf
            .read()
            .expect("Failed to read value of CPU performance counter")
    }
}

pub struct ResourcesTracker {
    cpu: InfoCpu,
}

impl ResourcesTracker {
    pub fn start_measurement() -> Self {
        let ret = Self {
            cpu: InfoCpu::new(),
        };

        ret.cpu.start_measurement();
        INFO_ALLOC.reset_counters();
        ret
    }

    pub fn end_measurement(&mut self) -> ResourcesUsage {
        let cpu_cycles = self.cpu.end_measurement();
        let (heap_allocations_sum, _heap_current_level, heap_peak_memory) =
            INFO_ALLOC.get_counters_value();
        ResourcesUsage {
            heap_allocations_sum,
            heap_peak_memory,
            cpu_cycles,
        }
    }
}

use crate::transaction::ResourcesUsage;
use perfcnt::{
    linux::{HardwareEventType, PerfCounterBuilderLinux},
    AbstractPerfCounter, PerfCounter,
};
use radix_engine_profiling::info_alloc::*;

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
        INFO_ALLOC.set_enable(true);
        INFO_ALLOC.reset_counters();
        ret
    }

    pub fn end_measurement(&mut self) -> ResourcesUsage {
        let cpu_cycles = self.cpu.end_measurement();
        let (heap_allocations_sum, _heap_current_level, heap_peak_memory) =
            INFO_ALLOC.get_counters_value();
        INFO_ALLOC.set_enable(false);
        ResourcesUsage {
            heap_allocations_sum,
            heap_peak_memory,
            cpu_cycles,
        }
    }
}

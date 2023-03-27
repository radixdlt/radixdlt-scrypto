use shared_memory::*;
use std::{fs::File, io::prelude::*};

pub mod data_analyzer;
pub use data_analyzer::{DataAnalyzer, OutputData, OutputDataEvent, OutputParam};

/// Shared memory name
const SHARED_MEM_ID: &str = "/shm-scrypto";
/// Maximum pre-allocated data entries
const OUTPUT_DATA_COUNT: usize = 500000;

std::thread_local! {
    /// Global QEMU plugin object variable
    pub static QEMU_PLUGIN: std::cell::RefCell<QemuPluginInterface<'static>> = std::cell::RefCell::new(QemuPluginInterface::new());
    /// Global QEMU plugin calibrator object variable, needs to be executed before QEMU plugin to measure and initialize data.
    pub static QEMU_PLUGIN_CALIBRATOR: std::cell::RefCell<QemuPluginInterfaceCalibrator> = std::cell::RefCell::new(QemuPluginInterfaceCalibrator::new());
}

pub struct QemuPluginInterface<'a> {
    counters_stack: Vec<(&'a str, u64)>,
    stack_top: usize,
    output_data: Vec<OutputData<'a>>,
    counter_offset: u64,
    counter_offset_parent: u64,
    shmem: Shmem,
}

impl<'a> QemuPluginInterface<'a> {
    pub fn new() -> Self {
        let shmem_conf = ShmemConf::new().os_id(SHARED_MEM_ID);
        let shmem = match shmem_conf.open() {
            Ok(v) => v,
            Err(x) => panic!("Unable to open shmem {:?}", x),
        };

        let mut ret = Self {
            counters_stack: vec![("", 0); 20],
            stack_top: 0,
            output_data: Vec::with_capacity(OUTPUT_DATA_COUNT),
            counter_offset: 0,
            counter_offset_parent: 0,
            shmem,
        };

        // Test connection with QEMU plugin.
        ret.communicate_with_server();

        ret
    }

    pub fn get_current_stack(&self) -> usize {
        self.stack_top
    }

    pub fn start_counting(&mut self, key: &'static str, arg: &[data_analyzer::OutputParam]) {
        if self.stack_top == self.counters_stack.len() {
            panic!("Stack too small, extend elements count of counters_stack field.");
        }

        self.counters_stack[self.stack_top].0 = key;

        self.output_data.push(OutputData {
            event: OutputDataEvent::FunctionEnter,
            stack_depth: self.stack_top,
            cpu_instructions: 0,
            cpu_instructions_calibrated: 0,
            function_name: key,
            param: arg.to_vec(),
        });

        self.stack_top += 1;
        self.counters_stack[self.stack_top - 1].1 = self.communicate_with_server();
    }

    pub fn stop_counting(
        &mut self,
        key: &'static str,
        arg: &[data_analyzer::OutputParam],
    ) -> (usize, u64) {
        let n = self.communicate_with_server();

        if self.stack_top == 0 {
            panic!("Not counting!");
        }
        self.stack_top -= 1;

        self.counters_stack[self.stack_top].1 = n - self.counters_stack[self.stack_top].1;

        self.output_data.push(OutputData {
            event: OutputDataEvent::FunctionExit,
            stack_depth: self.stack_top,
            cpu_instructions: self.counters_stack[self.stack_top].1,
            cpu_instructions_calibrated: 0,
            function_name: key,
            param: arg.to_vec(),
        });

        let ret = self.counters_stack[self.stack_top].1;
        (self.stack_top, ret)
    }

    fn communicate_with_server(&mut self) -> u64 {
        let raw_ptr = self.shmem.as_ptr() as *const u64;
        let ret = unsafe { std::ptr::read_volatile(raw_ptr) };

        ret
    }

    // Applies calibration data to each item.
    fn prepare_output_data(&mut self) {
        if self.output_data.is_empty() {
            return;
        }

        let overflow_range = 1;
        let mut ov_cnt = 0;
        for i in (0..=self.output_data.len() - 1).rev() {
            if !matches!(self.output_data[i].event, OutputDataEvent::FunctionExit) {
                continue;
            }

            self.output_data[i].cpu_instructions_calibrated = match self.output_data[i]
                .cpu_instructions
                .checked_sub(self.counter_offset)
            {
                Some(v) => v,
                None => {
                    if self.counter_offset - self.output_data[i].cpu_instructions > overflow_range {
                        println!(
                            "subtraction overflow: {}  {}  {}",
                            self.output_data[i].cpu_instructions,
                            self.output_data[i].function_name,
                            self.output_data[i].stack_depth
                        );
                        ov_cnt += 1;
                    }
                    0
                }
            };

            if i > 0 {
                for j in (0..=i - 1).rev() {
                    if !matches!(self.output_data[j].event, OutputDataEvent::FunctionExit) {
                        if j == i - 1 {
                            break;
                        }
                        continue;
                    }
                    if self.output_data[j].stack_depth > self.output_data[i].stack_depth {
                        self.output_data[i].cpu_instructions_calibrated = match self.output_data[i]
                            .cpu_instructions_calibrated
                            .checked_sub(self.counter_offset_parent)
                        {
                            Some(v) => v,
                            None => {
                                if self.counter_offset - self.output_data[i].cpu_instructions
                                    > overflow_range
                                {
                                    println!(
                                        "Subtraction overflow 2: {}  {}  {}",
                                        self.output_data[i].cpu_instructions,
                                        self.output_data[i].function_name,
                                        self.output_data[i].stack_depth
                                    );
                                    ov_cnt += 1;
                                }
                                0
                            }
                        };
                    } else {
                        break;
                    }
                }
            }
        }

        println!("Subtraction overflow count {}", ov_cnt);
    }

    #[allow(dead_code)]
    fn save_output_to_file(&self, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            for v in &self.output_data {
                v.write(&mut file);
            }
            file.flush()
                .expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }
}

impl<'a> Drop for QemuPluginInterface<'a> {
    fn drop(&mut self) {
        self.prepare_output_data();

        // Uncomment following function call for plugin debug purposes
        // self.save_output_to_file("/tmp/out.txt");

        DataAnalyzer::save_xml(&self.output_data, "/tmp/out.xml");
    }
}

pub struct QemuPluginInterfaceCalibrator {}
impl QemuPluginInterfaceCalibrator {
    fn new() -> QemuPluginInterfaceCalibrator {
        let mut ret = QemuPluginInterfaceCalibrator {};
        ret.calibrate_counters();
        ret
    }

    // measures time of QEMU Plugin instrumentation (only between calls)
    fn calibrate_inner() -> u64 {
        QEMU_PLUGIN.with(|v| v.borrow_mut().start_counting("calibrate_inner", &[]));
        QEMU_PLUGIN
            .with(|v| v.borrow_mut().stop_counting("calibrate_inner", &[]))
            .1
    }

    // measures time of QEMU Plugin instrumentation (with calls)
    fn calibrate() -> (u64, u64) {
        QEMU_PLUGIN.with(|v| v.borrow_mut().start_counting("calibrate", &[]));
        let ret = QemuPluginInterfaceCalibrator::calibrate_inner();
        (
            QEMU_PLUGIN
                .with(|v| v.borrow_mut().stop_counting("calibrate", &[]))
                .1,
            ret,
        )
    }

    fn calibrate_counters(&mut self) {
        let loop_max = 10;

        let mut cal_data_parent: Vec<u64> = Vec::new();
        let mut cal_data_child: Vec<u64> = Vec::new();
        for _ in 0..loop_max {
            let (parent, child) = QemuPluginInterfaceCalibrator::calibrate();
            cal_data_parent.push(parent);
            cal_data_child.push(child);
        }

        DataAnalyzer::discard_spikes(&mut cal_data_child, 2);
        let child_out = DataAnalyzer::average(&cal_data_child);
        DataAnalyzer::discard_spikes(&mut cal_data_parent, 2);
        let parent_out = DataAnalyzer::average(&cal_data_parent);

        println!("child out: {:?}\navg: {}", cal_data_child, child_out);
        println!("parent out: {:?}\navg: {}", cal_data_parent, parent_out);

        println!(
            "QemuPlugin counter offset self: {} instructions, parent for each child: {}",
            child_out,
            parent_out - child_out + child_out / 3
        );

        QEMU_PLUGIN.with(|v| {
            v.borrow_mut().counter_offset = child_out;
            v.borrow_mut().counter_offset_parent = parent_out - child_out + child_out / 3;
            v.borrow_mut().output_data.clear();
        });
    }
}

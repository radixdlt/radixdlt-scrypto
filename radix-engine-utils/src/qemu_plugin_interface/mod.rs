#![allow(dead_code)]

use std::os::unix::net::UnixDatagram;
use std::fs::File;
use std::io::prelude::*;
use shared_memory::*;


const SRV_SOCKET_FN: &str = "/tmp/scrypto-qemu-plugin-server.socket";
const CLI_SOCKET_FN: &str = "/tmp/scrypto-qemu-plugin-client.socket";
const OUTPUT_DATA_COUNT: usize = 500000;

std::thread_local! {
    pub static QEMU_PLUGIN: std::cell::RefCell<QemuPluginInterface<'static>> = std::cell::RefCell::new(QemuPluginInterface::new(true));
}


enum OutputDataEvent {
    FunctionEnter,
    FunctionExit
}

enum OutputParam {
    Number(i64),
    //Literal(fstr<50>)
}

struct OutputData<'a> {
    event: OutputDataEvent,
    stack_depth: usize,
    cpu_instructions: u64,
    cpu_instructions_calibrated: u64,
    function_name: &'a str,
    param: Option<OutputParam>,
}


pub struct QemuPluginInterface<'a> {
    enabled: bool,
    counters_stack: Vec<(String,u64)>,
    stack_top: usize,
    socket: UnixDatagram,
    output_data: Vec<OutputData<'a>>,
    counter_offset: u64,
    counter_offset_parent: u64,
    shmem: Shmem 
}

impl<'a> QemuPluginInterface<'a> {
    pub fn new(enabled: bool) -> Self {

        std::fs::remove_file(CLI_SOCKET_FN).unwrap_or_default();

        let socket = UnixDatagram::bind(CLI_SOCKET_FN).unwrap();
        socket.set_read_timeout(None).unwrap();

        //shared_memory::Shmem::
        let shmem_conf = ShmemConf::new().os_id("/shm-radix");
        let shmem = match shmem_conf.open() {
            Ok(v) => v,
            Err(x) => panic!("Unable to open shmem {:?}", x)
        };

        let mut ret = Self {
            enabled,
            counters_stack: Vec::with_capacity(100),
            stack_top: 0,
            socket,
            output_data: Vec::with_capacity(OUTPUT_DATA_COUNT),
            counter_offset: 0,
            counter_offset_parent: 0,
            shmem
        };


        // test connection
        ret.communicate_with_server(SRV_SOCKET_FN);

        for _ in 0..ret.counters_stack.capacity() {
            ret.counters_stack.push((String::with_capacity(50),0));
        }

        ret.calibrate_counters();

        ret
    }


    fn calibrate_inner(&mut self) -> u64 {
        self.start_counting("test_inner");
        self.stop_counting("test_inner").1
    }

    fn calibrate(&mut self, call_inner: bool) -> (u64, u64) {
        self.start_counting("test");
        let ret = if call_inner {
            self.calibrate_inner()
        } else {
            0
        };
        (self.stop_counting("test").1, ret)
    }


    fn calibrate_counters(&mut self) {
        let mut cnt = 0;
        let mut cnt_parent = 0;

        let loop_max = 10;

        for _ in 0..loop_max {
            let (parent, child) = self.calibrate(true);
            cnt_parent += parent;
            cnt += child;
            println!("1 child/parent: {} {}", child, parent);
        }
        for _ in 0..loop_max {
            let (parent, _child) = self.calibrate(false);
            cnt_parent -= parent;
            println!("2 parent: {}", parent);
        }

        self.output_data.clear();

        self.counter_offset = cnt / loop_max;
        self.counter_offset_parent = cnt_parent / loop_max;
        println!("QemuPlugin counter offset: {} instructions, parent: {}", self.counter_offset, self.counter_offset_parent)
    }
    
    pub fn get_current_stack(&self) -> usize {
        self.stack_top
    }

    pub fn start_counting(&mut self, key: &'static str) {
        if !self.enabled {
            return;
        }

        if self.stack_top == self.counters_stack.len() {
            panic!("Stack too small");
        }

        self.counters_stack[self.stack_top].0.push_str(key);

        let n = self.communicate_with_server(SRV_SOCKET_FN);

        self.counters_stack[self.stack_top].1 = n;

        self.output_data.push(OutputData {
            event: OutputDataEvent::FunctionEnter,
            stack_depth: self.stack_top,
            cpu_instructions: n,
            cpu_instructions_calibrated: 0,
            function_name: key,
            param: None });

        self.stack_top += 1;
    }

    pub fn stop_counting(&mut self, key: &'static str) -> (usize, u64) {
        if !self.enabled {
            return (0,0);
        }

        if self.stack_top == 0 {
            panic!("Not counting!");
        }
        self.stack_top -= 1;

        let n = self.communicate_with_server(SRV_SOCKET_FN);

        self.counters_stack[self.stack_top].1 = n - self.counters_stack[self.stack_top].1;

        self.output_data.push(OutputData {
            event: OutputDataEvent::FunctionExit,
            stack_depth: self.stack_top,
            cpu_instructions: self.counters_stack[self.stack_top].1,
            cpu_instructions_calibrated: 0,
            function_name: key,
            param: None });

        let ret = self.counters_stack[self.stack_top].1;
        (self.stack_top, ret)
    }

    fn communicate_with_server(&mut self, _addr: &str) -> u64 {

        let raw_ptr = self.shmem.as_ptr() as *const u64;
        let ret = unsafe {
            std::ptr::read_volatile(raw_ptr)
        };

        return ret;

/*        self.socket.send_to(b"", _addr).unwrap();
        //let mut buf = Vec::with_capacity(64);
        let mut buf = [0; 100];
        //self.socket.recv(&mut buf)
        let (_count, _address) = self.socket.recv_from(&mut buf).unwrap();

        let ret = u64::from_be_bytes(buf[..8].try_into().unwrap());

        //println!("socket {:?} sent {:?} -> {}", address, &buf[..count], ret);
        //let s = [0..ret].map(|_| " ").collect::<String>();
        //let s = String::from_utf8(vec![b' '; ret as usize]).unwrap();
        ret*/

    }

    fn prepare_output_data(&mut self) {
        if self.output_data.is_empty() {
            return;
        }

        let mut ov_cnt = 0;
        for i in (0..=self.output_data.len()-1).rev() {
            if ! matches!(self.output_data[i].event, OutputDataEvent::FunctionExit) {
                continue;
            }

            self.output_data[i].cpu_instructions_calibrated = match self.output_data[i].cpu_instructions.checked_sub(self.counter_offset) {
                Some(v) => v,
                None => {
                    println!("Subtraction overflow on {}, {}", self.output_data[i].function_name, i );
                    ov_cnt += 1;
                    0
                }
            };

            if i > 0 {
                for j in (0..=i-1).rev() {
                    if ! matches!(self.output_data[j].event, OutputDataEvent::FunctionExit) {
                        if j == i - 1 {
                            break;
                        }
                        continue;
                    }
                    if self.output_data[j].stack_depth > self.output_data[i].stack_depth {

                        self.output_data[i].cpu_instructions_calibrated = match self.output_data[i].cpu_instructions_calibrated.checked_sub(self.counter_offset_parent) {
                            Some(v) => v,
                            None => {
                                println!("Subtraction overflow on {}, {}, {}", self.output_data[i].function_name, i, j );
                                ov_cnt += 1;
                                0
                            }
                        };
                    } else {
                        break;
                    }
                }
            }
        }

        println!("Subtraction overflow count {}", ov_cnt );
    }

    fn save_output_to_file(&self, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            for v in &self.output_data {
                v.write(&mut file);
            }
            file.flush().expect("Unable to flush /tmp/out.txt file.")
        } else {
            panic!("Unable to create /tmp/out.txt file.")
        }
    }
}

impl<'a> Drop for QemuPluginInterface<'a> {
    fn drop(&mut self) {
        self.prepare_output_data();
        self.save_output_to_file("/tmp/out.txt");
    }
}

impl<'a> OutputData<'a> {
    fn write(&self, file: &mut File) {
        let spaces = std::iter::repeat(' ').take(4 * self.stack_depth).collect::<String>();
        match self.event {
            OutputDataEvent::FunctionEnter => 
                file.write(format!("{}++enter: {} {} {}\n", spaces, self.function_name, self.stack_depth, "").as_bytes()).expect("Unable to write output data"),
            OutputDataEvent::FunctionExit =>
                file.write(format!("{}--exit: {} {} {} {} {}\n", spaces, self.function_name, self.stack_depth, self.cpu_instructions, self.cpu_instructions_calibrated, "").as_bytes()).expect("Unable to write output data")
        };
    }
}


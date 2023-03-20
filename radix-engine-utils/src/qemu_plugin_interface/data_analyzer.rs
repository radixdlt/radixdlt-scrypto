use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use fixedstr::str32;



pub enum OutputDataEvent {
    FunctionEnter,
    FunctionExit
}

pub enum OutputParam {
    NumberI64(i64),
    NumberU64(u64),
    Literal(str32)
}

pub struct OutputData<'a> {
    pub event: OutputDataEvent,
    pub stack_depth: usize,
    pub cpu_instructions: u64,
    pub cpu_instructions_calibrated: u64,
    pub function_name: &'a str, 
    pub param: Option<OutputParam>,
}


pub struct DataAnalyzer {
}

impl DataAnalyzer {

    pub fn discard_spikes(data: &mut Vec<u64>, delta_range: u64)
    {
        // 1. calculate median
        data.sort();
        let center_idx = data.len() / 2;
        let median = data[center_idx]; // todo: handle odd length

        // 2. discard items out of median + range
        data.retain(|&i| if i > median {
                i - median <= delta_range
            } else {
                median - i <= delta_range
            } );
    }

    pub fn average(data: &Vec<u64>) -> u64 {
        data.iter().sum::<u64>() / data.len() as u64
    }

    pub fn generate_and_save_buckets<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        let mut buckets: HashMap<&str, HashMap<u64, u32>> = HashMap::new();

        for v in data {
            if ! matches!(v.event, OutputDataEvent::FunctionExit) {
                continue;
            }
            if let Some(e) = buckets.get_mut(v.function_name) {
                if let Some(h) = e.get_mut(&v.cpu_instructions_calibrated) {
                    *h += 1;
                } else {
                    e.insert(v.cpu_instructions_calibrated, 1);
                }
            } else {
                buckets.insert(v.function_name, HashMap::new());
            }
        }

        if let Ok(mut file) = File::create(file_name) {
            file.write_fmt(format_args!("function_name;instructions_count;count\n")).expect(&format!("Unable write to {} file.", file_name));
            for v in buckets {
                for w in v.1 {
                    file.write_fmt(format_args!("{};{};{}\n", v.0, w.0, w.1)).expect(&format!("Unable write to {} file.", file_name));
                }
            }
            file.flush().expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }

}
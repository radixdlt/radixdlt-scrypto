use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::fmt::Formatter;
use fixedstr::str32;


pub enum OutputDataEvent {
    FunctionEnter,
    FunctionExit
}
impl Display for OutputDataEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OutputDataEvent::FunctionEnter => f.write_fmt(format_args!("enter")).unwrap(),
            OutputDataEvent::FunctionExit => f.write_fmt(format_args!("exit")).unwrap()
        };
        Ok(())
    }
}

#[derive(Clone)]
pub enum OutputParam {
    NumberI64(i64),
    NumberU64(u64),
    Literal(str32)
}
impl Display for OutputParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OutputParam::NumberI64(v) => f.write_fmt(format_args!("{}",v)).unwrap(),
            OutputParam::NumberU64(v) => f.write_fmt(format_args!("{}",v)).unwrap(),
            OutputParam::Literal(v) => f.write_fmt(format_args!("{}",v)).unwrap(),
        };
        Ok(())
    }
}
impl Default for OutputParam {
    fn default() -> Self {
        OutputParam::Literal(str32::new())
    }
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

    pub fn save_csv<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            file.write_fmt(format_args!("event;function_name;stack_depth;instructions_count;instructions_count_calibrated;macro_arg\n")).expect(&format!("Unable write to {} file.", file_name));

            for v in data {
                file.write_fmt(format_args!("{};{};{};{};{};{}\n", 
                    v.event, 
                    v.function_name, 
                    v.stack_depth, 
                    v.cpu_instructions, 
                    v.cpu_instructions_calibrated, 
                    v.param.clone().unwrap_or_default())
                ).expect(&format!("Unable write to {} file.", file_name));
            }
            file.flush().expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }


    pub fn save_json<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {

            let mut prev_stack_depth = 0;

            for (_i, v) in data.iter().enumerate() {
                if v.stack_depth > prev_stack_depth {
                    file.write_fmt(format_args!("[")).unwrap();
                } else if v.stack_depth < prev_stack_depth {
                    file.write_fmt(format_args!("]")).unwrap();
                } else {
                    file.write_fmt(format_args!(",")).unwrap();
                }
                let spaces = std::iter::repeat(' ').take(1 * v.stack_depth).collect::<String>();

                file.write_fmt(format_args!("{}{{\"e\":\"{}\",\"f\":\"{}\",\"s\":{},\"i\":{},\"c\":{},\"p\":\"{}\"}}\n",
                    spaces, 
                    v.event, 
                    v.function_name, 
                    v.stack_depth, 
                    v.cpu_instructions, 
                    v.cpu_instructions_calibrated, 
                    v.param.clone().unwrap_or_default())
                ).expect(&format!("Unable write to {} file.", file_name));

                prev_stack_depth = v.stack_depth;
            }
            file.flush().expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }

    pub fn save_xml<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {

            let mut stack_fcn: Vec<&'a str> = vec!["root"];
            let mut prev_stack_depth = 0;
            file.write_fmt(format_args!("<root>\n")).unwrap();

            for (i, v) in data.iter().enumerate() {
                let mut cpu_ins_cal = v.cpu_instructions_calibrated;

                // set cpu instructions from exit event to enter event
                if matches!(v.event, OutputDataEvent::FunctionEnter) {
                    for w in data[i..].into_iter() {
                        if v.stack_depth == w.stack_depth && 
                           v.function_name == w.function_name &&
                           matches!(w.event, OutputDataEvent::FunctionExit) {
                                cpu_ins_cal = w.cpu_instructions_calibrated;
                                break;
                           }
                    }
                }

                if v.stack_depth > prev_stack_depth {
                    file.write_fmt(format_args!(">\n")).unwrap();
                } else if v.stack_depth < prev_stack_depth {
                    let spaces = std::iter::repeat(' ').take(v.stack_depth).collect::<String>();
                    file.write_fmt(format_args!("{}</{}>\n", spaces, stack_fcn.pop().unwrap())).unwrap();
                } else if i > 0 && matches!(v.event, OutputDataEvent::FunctionExit) {
                    file.write_fmt(format_args!("/>\n")).unwrap();
                    stack_fcn.pop();
                }

                if !matches!(v.event, OutputDataEvent::FunctionExit) {
                    let spaces = std::iter::repeat(' ').take(v.stack_depth).collect::<String>();
                    stack_fcn.push(v.function_name);

                    file.write_fmt(format_args!("{}<{} ins=\"{}\"",
                            spaces, 
                            v.function_name, 
                            cpu_ins_cal)
                        ).expect(&format!("Unable write to {} file.", file_name));

                    if v.param.is_some() {
                        file.write_fmt(format_args!(" arg=\"{}\"",
                                v.param.clone().unwrap_or_default().to_string().replace('\"', "&quot;"))
                            ).expect(&format!("Unable write to {} file.", file_name));
                    }
                }

                prev_stack_depth = v.stack_depth;
            }
            file.write_fmt(format_args!("</root>")).unwrap();

            file.flush().expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }


}
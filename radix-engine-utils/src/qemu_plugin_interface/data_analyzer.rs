use fixedstr::{str32, str64};
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
};

pub enum OutputDataEvent {
    FunctionEnter,
    FunctionExit,
}

impl Display for OutputDataEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OutputDataEvent::FunctionEnter => f.write_fmt(format_args!("enter")).unwrap(),
            OutputDataEvent::FunctionExit => f.write_fmt(format_args!("exit")).unwrap(),
        };
        Ok(())
    }
}

#[derive(Clone)]
pub enum OutputParamValue {
    NumberI64(i64),
    NumberU64(u64),
    Literal(str64), // using contant 64-bytes length string for speed optimisation
}
impl Display for OutputParamValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OutputParamValue::NumberI64(v) => f.write_fmt(format_args!("{}", v)).unwrap(),
            OutputParamValue::NumberU64(v) => f.write_fmt(format_args!("{}", v)).unwrap(),
            OutputParamValue::Literal(v) => f.write_fmt(format_args!("{}", v)).unwrap(),
        };
        Ok(())
    }
}
impl Default for OutputParamValue {
    fn default() -> Self {
        OutputParamValue::Literal(str64::new())
    }
}

#[derive(Clone)]
pub struct OutputParam {
    pub name: str32,
    pub value: OutputParamValue,
}
impl OutputParam {
    pub fn new(name: &str, value: OutputParamValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

pub struct OutputData<'a> {
    /// Logged event
    pub event: OutputDataEvent,
    /// Current stack depth
    pub stack_depth: usize,
    /// CPU instructions count
    pub cpu_instructions: u64,
    /// CPU instructions count with subtracted calibration values
    pub cpu_instructions_calibrated: u64,
    /// Called function name
    pub function_name: &'a str,
    /// Function parameters to log
    pub param: Vec<OutputParam>,
}

impl<'a> OutputData<'a> {
    pub fn write(&self, file: &mut File) {
        let spaces = std::iter::repeat(' ')
            .take(4 * self.stack_depth)
            .collect::<String>();

        match self.event {
            OutputDataEvent::FunctionEnter => file
                .write_fmt(format_args!(
                    "{}++enter: {} {}",
                    spaces, self.function_name, self.stack_depth
                ))
                .expect("Unable to write output data"),
            OutputDataEvent::FunctionExit => file
                .write_fmt(format_args!(
                    "{}--exit: {} {} {} {}",
                    spaces,
                    self.function_name,
                    self.stack_depth,
                    self.cpu_instructions,
                    self.cpu_instructions_calibrated
                ))
                .expect("Unable to write output data"),
        };

        for p in &self.param {
            file.write_fmt(format_args!(
                " {}=\"{}\"",
                p.name,
                p.value.to_string().replace('\"', "&quot;")
            ))
            .expect(&format!("Unable write data."));
        }

        file.write_fmt(format_args!("\n"))
            .expect(&format!("Unable write data."));
    }
}

pub struct DataAnalyzer {}
impl DataAnalyzer {
    /// Function discards spikes in passed vector data, used for calibration.
    pub fn discard_spikes(data: &mut Vec<u64>, delta_range: u64) {
        // 1. calculate median
        data.sort();
        let center_idx = data.len() / 2;
        let median = data[center_idx];

        // 2. discard items out of median + range
        data.retain(|&i| {
            if i > median {
                i - median <= delta_range
            } else {
                median - i <= delta_range
            }
        });
    }

    /// Function calculates average for passed vector.
    pub fn average(data: &Vec<u64>) -> u64 {
        data.iter().sum::<u64>() / data.len() as u64
    }

    /// Function stores passed data as csv file.
    pub fn save_csv<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            file.write_fmt(format_args!("event;function_name;stack_depth;instructions_count;instructions_count_calibrated\n")).expect(&format!("Unable write to {} file.", file_name));

            for v in data {
                file.write_fmt(format_args!(
                    "{};{};{};{};{}\n",
                    v.event,
                    v.function_name,
                    v.stack_depth,
                    v.cpu_instructions,
                    v.cpu_instructions_calibrated
                ))
                .expect(&format!("Unable write to {} file.", file_name));
            }
            file.flush()
                .expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }

    /// Function stores passed data as xml file.
    pub fn save_xml<'a>(data: &Vec<OutputData<'a>>, file_name: &str) {
        if let Ok(mut file) = File::create(file_name) {
            let mut stack_fcn: Vec<&'a str> = vec!["root"];
            let mut prev_stack_depth = 0;
            file.write_fmt(format_args!("<root>\n")).unwrap();

            for (i, v) in data.iter().enumerate() {
                let mut cpu_ins_cal = v.cpu_instructions_calibrated;
                let mut param: &Vec<OutputParam> = &Vec::new();

                // get cpu instructions and param from exit event
                if matches!(v.event, OutputDataEvent::FunctionEnter) {
                    for w in data[i..].into_iter() {
                        if v.stack_depth == w.stack_depth
                            && v.function_name == w.function_name
                            && matches!(w.event, OutputDataEvent::FunctionExit)
                        {
                            cpu_ins_cal = w.cpu_instructions_calibrated;
                            param = &w.param;
                            break;
                        }
                    }
                }

                if v.stack_depth > prev_stack_depth {
                    file.write_fmt(format_args!(">\n")).unwrap();
                } else if v.stack_depth < prev_stack_depth {
                    let spaces = std::iter::repeat(' ')
                        .take(v.stack_depth)
                        .collect::<String>();
                    file.write_fmt(format_args!("{}</{}>\n", spaces, stack_fcn.pop().unwrap()))
                        .unwrap();
                } else if i > 0 && matches!(v.event, OutputDataEvent::FunctionExit) {
                    file.write_fmt(format_args!("/>\n")).unwrap();
                    stack_fcn.pop();
                }

                if !matches!(v.event, OutputDataEvent::FunctionExit) {
                    let spaces = std::iter::repeat(' ')
                        .take(v.stack_depth)
                        .collect::<String>();
                    stack_fcn.push(v.function_name);

                    file.write_fmt(format_args!(
                        "{}<{} ins=\"{}\"",
                        spaces, v.function_name, cpu_ins_cal
                    ))
                    .expect(&format!("Unable write to {} file.", file_name));

                    if !param.is_empty() {
                        // use param from exit event if available
                        for p in param {
                            file.write_fmt(format_args!(
                                " {}=\"{}\"",
                                p.name,
                                p.value.to_string().replace('\"', "&quot;")
                            ))
                            .expect(&format!("Unable write to {} file.", file_name));
                        }
                    }
                    if !v.param.is_empty() {
                        for p in &v.param {
                            // skip same name argument, as they are prohibited in XML
                            if param
                                .into_iter()
                                .find(|&item| item.name == p.name)
                                .is_some()
                            {
                                continue;
                            }
                            file.write_fmt(format_args!(
                                " {}=\"{}\"",
                                p.name,
                                p.value.to_string().replace('\"', "&quot;")
                            ))
                            .expect(&format!("Unable write to {} file.", file_name));
                        }
                    }
                }

                prev_stack_depth = v.stack_depth;
            }
            file.write_fmt(format_args!("</root>")).unwrap();

            file.flush()
                .expect(&format!("Unable to flush {} file.", file_name))
        } else {
            panic!("Unable to create {} file.", file_name)
        }
    }
}

use crate::transaction::*;
use radix_common::prelude::*;
use std::path::PathBuf;

#[derive(Copy, Clone)]
pub enum CostingTaskMode {
    OutputCosting,
    AssertCosting,
}

impl CostingTaskMode {
    pub fn run(
        &self,
        folder: impl Into<PathBuf>,
        file: &str,
        fee_summary: &TransactionFeeSummary,
        fee_details: &TransactionFeeDetails,
    ) {
        match self {
            CostingTaskMode::OutputCosting => {
                write_cost_breakdown(fee_summary, fee_details, folder, file);
            }
            CostingTaskMode::AssertCosting => {
                let expected = load_cost_breakdown(folder, file);
                assert_eq!(&fee_details.execution_cost_breakdown, &expected.0);
                assert_eq!(&fee_details.finalization_cost_breakdown, &expected.1);
            }
        }
    }
}

fn load_cost_breakdown(
    folder: impl Into<PathBuf>,
    file: &str,
) -> (BTreeMap<String, u32>, BTreeMap<String, u32>) {
    let path = folder.into().join(file);
    let content = std::fs::read_to_string(&path).unwrap();
    let mut execution_breakdown = BTreeMap::<String, u32>::new();
    let mut finalization_breakdown = BTreeMap::<String, u32>::new();
    let lines: Vec<String> = content.split('\n').map(String::from).collect();
    let mut is_execution = true;
    for i in 7..lines.len() {
        if lines[i].starts_with('-') {
            let mut tokens = lines[i].split(',');
            let entry = tokens.next().unwrap().trim()[2..].to_string();
            let cost = tokens.next().unwrap().trim();
            if is_execution {
                &mut execution_breakdown
            } else {
                &mut finalization_breakdown
            }
            .insert(entry, u32::from_str(cost).unwrap());
        } else {
            is_execution = false;
        }
    }
    (execution_breakdown, finalization_breakdown)
}

pub fn write_cost_breakdown(
    fee_summary: &TransactionFeeSummary,
    fee_details: &TransactionFeeDetails,
    folder: impl Into<PathBuf>,
    file: &str,
) {
    use std::fs::File;
    use std::io::Write;

    let buffer = format_cost_breakdown(fee_summary, fee_details);

    let folder = folder.into();
    let file_path = folder.join(file);
    let _ = std::fs::create_dir_all(&folder);
    let mut f = File::create(&file_path).unwrap();
    f.write_all(buffer.as_bytes()).unwrap();
}

pub fn format_cost_breakdown(
    fee_summary: &TransactionFeeSummary,
    fee_details: &TransactionFeeDetails,
) -> String {
    use std::fmt::Write as _;
    fn decimal_to_float(d: Decimal) -> f64 {
        f64::from_str(d.to_string().as_str()).unwrap()
    }
    fn percentage_u32(numerator: u32, denominator: u32) -> f64 {
        percentage_decimal(Decimal::from(numerator), Decimal::from(denominator))
    }
    fn percentage_decimal(numerator: Decimal, denominator: Decimal) -> f64 {
        decimal_to_float(
            numerator
                .checked_div(denominator)
                .unwrap_or(-Decimal::one())
                .checked_mul(100)
                .unwrap(),
        )
    }

    let mut buffer = String::new();
    let total_cost = fee_summary.total_cost();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "Total Cost (XRD)",
        total_cost.to_string(),
        100.0,
    )
    .unwrap();

    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "- Execution Cost (XRD)",
        fee_summary.total_execution_cost_in_xrd.to_string(),
        percentage_decimal(fee_summary.total_execution_cost_in_xrd, total_cost),
    )
    .unwrap();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "- Finalization Cost (XRD)",
        fee_summary.total_finalization_cost_in_xrd.to_string(),
        percentage_decimal(fee_summary.total_finalization_cost_in_xrd, total_cost),
    )
    .unwrap();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "- Storage Cost (XRD)",
        fee_summary.total_storage_cost_in_xrd.to_string(),
        percentage_decimal(fee_summary.total_storage_cost_in_xrd, total_cost),
    )
    .unwrap();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "- Tipping Cost (XRD)",
        fee_summary.total_tipping_cost_in_xrd.to_string(),
        percentage_decimal(fee_summary.total_tipping_cost_in_xrd, total_cost),
    )
    .unwrap();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "- Royalty Cost (XRD)",
        fee_summary.total_royalty_cost_in_xrd.to_string(),
        percentage_decimal(fee_summary.total_royalty_cost_in_xrd, total_cost),
    )
    .unwrap();
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "Execution Cost Breakdown",
        fee_details.execution_cost_breakdown.values().sum::<u32>(),
        100.0,
    )
    .unwrap();
    for (k, v) in &fee_details.execution_cost_breakdown {
        writeln!(
            &mut buffer,
            "- {k:<73},{v:>25}, {:8.1}%",
            percentage_u32(*v, fee_summary.total_execution_cost_units_consumed),
        )
        .unwrap();
    }
    writeln!(
        &mut buffer,
        "{:<75},{:>25}, {:8.1}%",
        "Finalization Cost Breakdown",
        fee_details
            .finalization_cost_breakdown
            .values()
            .sum::<u32>(),
        100.0,
    )
    .unwrap();
    for (k, v) in &fee_details.finalization_cost_breakdown {
        writeln!(
            &mut buffer,
            "- {k:<73},{v:>25}, {:8.1}%",
            percentage_u32(*v, fee_summary.total_finalization_cost_units_consumed),
        )
        .unwrap();
    }
    buffer
}

use crate::transaction::*;
use radix_common::prelude::*;

pub fn format_cost_breakdown(
    fee_summary: &TransactionFeeSummary,
    fee_details: &TransactionFeeDetails,
) -> String {
    use core::fmt::Write;
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
                .unwrap_or_default() // default for rejections
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

#[cfg(not(feature = "alloc"))]
pub use std_only::*;

#[cfg(not(feature = "alloc"))]
mod std_only {
    use super::format_cost_breakdown;
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
            base_path: impl Into<PathBuf>,
            relative_file_path: &str,
            fee_summary: &TransactionFeeSummary,
            fee_details: &TransactionFeeDetails,
        ) {
            match self {
                CostingTaskMode::OutputCosting => {
                    write_cost_breakdown(fee_summary, fee_details, base_path, relative_file_path);
                }
                CostingTaskMode::AssertCosting => {
                    verify_cost_breakdown(fee_summary, fee_details, base_path, relative_file_path)
                        .unwrap();
                }
            }
        }
    }

    fn verify_cost_breakdown(
        fee_summary: &TransactionFeeSummary,
        fee_details: &TransactionFeeDetails,
        folder: impl Into<PathBuf>,
        relative_file_path: &str,
    ) -> Result<(), String> {
        let path = folder.into().join(relative_file_path);
        let content = std::fs::read_to_string(&path).map_err(|err| {
            format!("Costing breakdown read error ({err:?}): {relative_file_path}")
        })?;
        let expected = format_cost_breakdown(fee_summary, fee_details);
        if content != expected {
            // We don't use an assert_eq here so that it doesn't dump massive text on failure
            return Err(format!(
                "Costing breakdown needs updating: {relative_file_path}"
            ));
        }
        Ok(())
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
        let _ = std::fs::create_dir_all(&file_path.parent().unwrap());
        let mut f = File::create(&file_path).unwrap_or_else(|err| {
            panic!("Failed to create file for costing breakdown: {file_path:?} ({err:?})");
        });
        f.write_all(buffer.as_bytes()).unwrap();
    }
}

use colored::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::address::Bech32Encoder;
use scrypto::core::Network;
use scrypto::engine::types::*;
use scrypto::values::*;
use transaction::model::*;

use crate::engine::{ResourceChange, RuntimeError};
use crate::fee::FeeSummary;
use crate::state_manager::StateDiff;

#[derive(Debug)]
pub enum TransactionStatus {
    Rejected,
    Succeeded(Vec<Vec<u8>>),
    Failed(RuntimeError),
}

impl TransactionStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Succeeded(..))
    }
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed(..))
    }
    pub fn is_rejection(&self) -> bool {
        matches!(self, Self::Rejected)
    }
}

/// Represents a transaction receipt.
pub struct TransactionReceipt {
    pub status: TransactionStatus,
    pub fee_summary: FeeSummary,
    pub transaction_network: Network,
    pub execution_time: Option<u128>,
    pub instructions: Vec<ExecutableInstruction>,
    pub application_logs: Vec<(Level, String)>,
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
    pub state_updates: StateDiff,
    pub resource_changes: Vec<ResourceChange>,
}

impl TransactionReceipt {
    pub fn expect_success(&self) -> &Vec<Vec<u8>> {
        match &self.status {
            TransactionStatus::Succeeded(output) => output,
            TransactionStatus::Failed(err) => panic!("Expected success but was:\n{:?}", err),
            TransactionStatus::Rejected => panic!("Expected success but was rejection"),
        }
    }

    pub fn expect_failure<F>(&self, f: F)
    where
        F: FnOnce(&RuntimeError) -> bool,
    {
        if let TransactionStatus::Failed(e) = &self.status {
            if !f(e) {
                panic!("Expected failure but was different error:\n{:?}", self);
            }
        } else {
            panic!("Expected failure but was:\n{:?}", self);
        }
    }

    pub fn expect_rejection(&self) {
        if !matches!(self.status, TransactionStatus::Rejected) {
            panic!("Expected rejection but was:\n{:?}", self);
        }
    }
}

macro_rules! prefix {
    ($i:expr, $list:expr) => {
        if $i == $list.len() - 1 {
            "└─"
        } else {
            "├─"
        }
    };
}

impl fmt::Debug for TransactionReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bech32_encoder = Bech32Encoder::new_from_network(&self.transaction_network);

        write!(
            f,
            "{} {}",
            "Transaction Status:".bold().green(),
            match &self.status {
                TransactionStatus::Succeeded(_) => "SUCCESS".blue(),
                TransactionStatus::Failed(e) => format!("FAILURE: {}", e).red(),
                TransactionStatus::Rejected => "REJECTION".red(),
            }
        )?;

        write!(
            f,
            "\n{} {} XRD burned, {} XRD tipped to validators",
            "Transaction Fee:".bold().green(),
            self.fee_summary.burned,
            self.fee_summary.tipped,
        )?;

        write!(
            f,
            "\n{} {} limit, {} consumed, {} XRD per cost unit",
            "Cost Units:".bold().green(),
            self.fee_summary.cost_unit_limit,
            self.fee_summary.cost_unit_consumed,
            self.fee_summary.cost_unit_price,
        )?;

        write!(
            f,
            "\n{} {} ms",
            "Execution Time:".bold().green(),
            self.execution_time
                .map(|v| v.to_string())
                .unwrap_or(String::from("?"))
        )?;

        write!(f, "\n{}", "Instructions:".bold().green())?;
        for (i, inst) in self.instructions.iter().enumerate() {
            write!(
                f,
                "\n{} {}",
                prefix!(i, self.instructions),
                match inst {
                    ExecutableInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        method_name,
                        arg,
                    } => format!(
                        "CallFunction {{ package_address: {}, blueprint_name: {:?}, method_name: {:?}, arg: {:?} }}",
                        bech32_encoder.encode_package_address(&package_address).unwrap(),
                        blueprint_name,
                        method_name,
                        ScryptoValue::from_slice(&arg).expect("Invalid call data")
                    ),
                    ExecutableInstruction::CallMethod {
                        component_address,
                        method_name,
                        arg,
                    } => format!(
                        "CallMethod {{ component_address: {}, method_name: {:?}, call_data: {:?} }}",
                        bech32_encoder.encode_component_address(&component_address).unwrap(),
                        method_name,
                        ScryptoValue::from_slice(&arg).expect("Invalid call data")
                    ),
                    ExecutableInstruction::PublishPackage { .. } => "PublishPackage {..}".to_owned(),
                    i @ _ => format!("{:?}", i),
                }
            )?;
        }

        if let TransactionStatus::Succeeded(outputs) = &self.status {
            write!(f, "\n{}", "Instruction Outputs:".bold().green())?;
            for (i, output) in outputs.iter().enumerate() {
                write!(
                    f,
                    "\n{} {:?}",
                    prefix!(i, outputs),
                    ScryptoValue::from_slice(output).expect("Invalid return data")
                )?;
            }
        }

        write!(
            f,
            "\n{} {}",
            "Logs:".bold().green(),
            self.application_logs.len()
        )?;
        for (i, (level, msg)) in self.application_logs.iter().enumerate() {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.red()),
                Level::Warn => ("WARN".yellow(), msg.yellow()),
                Level::Info => ("INFO".green(), msg.green()),
                Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                Level::Trace => ("TRACE".normal(), msg.normal()),
            };
            write!(f, "\n{} [{:5}] {}", prefix!(i, self.application_logs), l, m)?;
        }

        write!(
            f,
            "\n{} {}",
            "New Entities:".bold().green(),
            self.new_package_addresses.len()
                + self.new_component_addresses.len()
                + self.new_resource_addresses.len()
        )?;

        for (i, package_address) in self.new_package_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Package: {}",
                prefix!(i, self.new_package_addresses),
                bech32_encoder
                    .encode_package_address(package_address)
                    .unwrap()
            )?;
        }
        for (i, component_address) in self.new_component_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Component: {}",
                prefix!(i, self.new_component_addresses),
                bech32_encoder
                    .encode_component_address(component_address)
                    .unwrap()
            )?;
        }
        for (i, resource_address) in self.new_resource_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Resource: {}",
                prefix!(i, self.new_resource_addresses),
                bech32_encoder
                    .encode_resource_address(resource_address)
                    .unwrap()
            )?;
        }

        Ok(())
    }
}

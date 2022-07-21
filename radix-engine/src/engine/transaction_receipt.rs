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

use crate::engine::CommitReceipt;
use crate::engine::RuntimeError;

pub struct TransactionFeeSummary {
    /// Whether the fee loan has been fully repaid.
    /// Clients should use this flag to decide whether to include the transaction into a block.
    pub system_loan_full_repaid: bool,
    /// The specified max cost units can be consumed
    pub max_cost_units: u32,
    /// The total number of cost units consumed
    pub cost_units_consumed: u32,
    /// The cost unit price in XRD
    pub cost_units_price: Decimal,
    /// The total amount of XRD burned
    pub burned: Decimal,
    /// The total amount of XRD tipped to validators
    pub tipped: Decimal,
}

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction_network: Network,
    pub transaction_fee: TransactionFeeSummary,
    pub execution_time: Option<u128>,
    pub instructions: Vec<ExecutableInstruction>,
    pub result: Result<Vec<Vec<u8>>, RuntimeError>,
    pub logs: Vec<(Level, String)>,
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
    pub commit_receipt: CommitReceipt,
}

impl Receipt {
    pub fn expect_success(&self) -> &Vec<Vec<u8>> {
        match &self.result {
            Ok(output) => output,
            Err(err) => panic!("Expected success but was:\n{:?}", err),
        }
    }

    pub fn expect_err<F>(&self, f: F)
    where
        F: FnOnce(&RuntimeError) -> bool,
    {
        if let Err(e) = &self.result {
            if !f(e) {
                panic!("Expected error but was different error:\n{:?}", self);
            }
        } else {
            panic!("Expected error but was successful:\n{:?}", self);
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

impl fmt::Debug for Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bech32_encoder = Bech32Encoder::new_from_network(&self.transaction_network);

        write!(
            f,
            "{} {}",
            "Transaction Status:".bold().green(),
            match &self.result {
                Ok(_) => "SUCCESS".blue(),
                Err(e) => e.to_string().red(),
            }
        )?;

        write!(
            f,
            "\n{} {} XRD burned, {} XRD tipped to validators",
            "Transaction Fee:".bold().green(),
            self.transaction_fee.burned,
            self.transaction_fee.tipped,
        )?;

        write!(
            f,
            "\n{} {} max, {} consumed, {} XRD per cost unit",
            "Cost Units:".bold().green(),
            self.transaction_fee.max_cost_units,
            self.transaction_fee.cost_units_consumed,
            self.transaction_fee.cost_units_price,
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
                        bech32_encoder.encode_package_address(package_address).unwrap(),
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
                        bech32_encoder.encode_component_address(component_address).unwrap(),
                        method_name,
                        ScryptoValue::from_slice(&arg).expect("Invalid call data")
                    ),
                    ExecutableInstruction::PublishPackage { .. } => "PublishPackage {..}".to_owned(),
                    i @ _ => format!("{:?}", i),
                }
            )?;
        }

        if let Ok(outputs) = &self.result {
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

        write!(f, "\n{} {}", "Logs:".bold().green(), self.logs.len())?;
        for (i, (level, msg)) in self.logs.iter().enumerate() {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.red()),
                Level::Warn => ("WARN".yellow(), msg.yellow()),
                Level::Info => ("INFO".green(), msg.green()),
                Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                Level::Trace => ("TRACE".normal(), msg.normal()),
            };
            write!(f, "\n{} [{:5}] {}", prefix!(i, self.logs), l, m)?;
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

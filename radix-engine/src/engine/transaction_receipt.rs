use colored::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::address::Bech32Addressable;
use scrypto::core::Network;
use scrypto::engine::types::*;
use scrypto::values::*;
use transaction::model::*;

use crate::engine::CommitReceipt;
use crate::engine::RuntimeError;

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction_network: Network,
    pub commit_receipt: Option<CommitReceipt>,
    pub instructions: Vec<ExecutableInstruction>,
    pub result: Result<(), RuntimeError>,
    pub outputs: Vec<Vec<u8>>,
    pub logs: Vec<(Level, String)>,
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
    pub execution_time: Option<u128>,
    pub cost_units_consumed: u32,
}

impl Receipt {
    pub fn expect_success(&self) {
        if self.result.is_err() {
            panic!("Expected success but was:\n{:?}", self);
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
        write!(
            f,
            "{} {}",
            "Transaction Status:".bold().green(),
            match &self.result {
                Ok(()) => "SUCCESS".blue(),
                Err(e) => e.to_string().red(),
            }
            .bold()
        )?;

        write!(
            f,
            "\n{} {}",
            "Cost Units Consumed:".bold().green(),
            self.cost_units_consumed
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
                        package_address.to_bech32_string(&self.transaction_network).unwrap(),
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
                        component_address.to_bech32_string(&self.transaction_network).unwrap(),
                        method_name,
                        ScryptoValue::from_slice(&arg).expect("Invalid call data")
                    ),
                    ExecutableInstruction::PublishPackage { .. } => "PublishPackage {..}".to_owned(),
                    i @ _ => format!("{:?}", i),
                }
            )?;
        }

        write!(f, "\n{}", "Instruction Outputs:".bold().green())?;
        for (i, output) in self.outputs.iter().enumerate() {
            write!(
                f,
                "\n{} {:?}",
                prefix!(i, self.outputs),
                ScryptoValue::from_slice(output).expect("Invalid return data")
            )?;
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
                package_address
                    .to_bech32_string(&self.transaction_network)
                    .unwrap()
            )?;
        }
        for (i, component_address) in self.new_component_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Component: {}",
                prefix!(i, self.new_component_addresses),
                component_address
                    .to_bech32_string(&self.transaction_network)
                    .unwrap()
            )?;
        }
        for (i, resource_address) in self.new_resource_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Resource: {}",
                prefix!(i, self.new_resource_addresses),
                resource_address
                    .to_bech32_string(&self.transaction_network)
                    .unwrap()
            )?;
        }

        Ok(())
    }
}

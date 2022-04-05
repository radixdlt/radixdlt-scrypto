use colored::*;
use scrypto::engine::types::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::values::*;

use crate::engine::CommitReceipt;
use crate::errors::*;
use crate::model::*;

/// Represents a transaction receipt.
pub struct Receipt {
    pub commit_receipt: Option<CommitReceipt>,
    pub transaction: ValidatedTransaction,
    pub result: Result<(), RuntimeError>,
    pub outputs: Vec<ScryptoValue>,
    pub logs: Vec<(Level, String)>,
    pub new_package_addresses: Vec<PackageAddress>,
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
    pub execution_time: Option<u128>,
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
            "\n{} {} ms",
            "Execution Time:".bold().green(),
            self.execution_time
                .map(|v| v.to_string())
                .unwrap_or(String::from("?"))
        )?;

        write!(f, "\n{}", "Instructions:".bold().green())?;
        for (i, inst) in self.transaction.instructions.iter().enumerate() {
            write!(
                f,
                "\n{} {:?}",
                prefix!(i, self.transaction.instructions),
                inst
            )?;
        }

        write!(f, "\n{}", "Instruction Outputs:".bold().green())?;
        for (i, result) in self.outputs.iter().enumerate() {
            write!(f, "\n{} {:?}", prefix!(i, self.outputs), result)?;
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
            )?;
        }
        for (i, component_address) in self.new_component_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Component: {}",
                prefix!(i, self.new_component_addresses),
                component_address
            )?;
        }
        for (i, resource_address) in self.new_resource_addresses.iter().enumerate() {
            write!(
                f,
                "\n{} Resource: {}",
                prefix!(i, self.new_resource_addresses),
                resource_address
            )?;
        }

        Ok(())
    }
}

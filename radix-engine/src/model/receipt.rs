use colored::*;
use scrypto::engine::types::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;

use crate::errors::*;
use crate::model::*;

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction: ValidatedTransaction,
    pub result: Result<(), RuntimeError>,
    pub outputs: Vec<ValidatedData>,
    pub logs: Vec<(Level, String)>,
    pub new_package_ids: Vec<PackageId>,
    pub new_component_ids: Vec<ComponentId>,
    pub new_resource_def_ids: Vec<ResourceDefId>,
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
            self.new_package_ids.len()
                + self.new_component_ids.len()
                + self.new_resource_def_ids.len()
        )?;

        for (i, package_id) in self.new_package_ids.iter().enumerate() {
            write!(
                f,
                "\n{} Package: {}",
                prefix!(i, self.new_package_ids),
                package_id
            )?;
        }
        for (i, component_id) in self.new_component_ids.iter().enumerate() {
            write!(
                f,
                "\n{} Component: {}",
                prefix!(i, self.new_component_ids),
                component_id
            )?;
        }
        for (i, resource_def_id) in self.new_resource_def_ids.iter().enumerate() {
            write!(
                f,
                "\n{} ResourceDef: {}",
                prefix!(i, self.new_resource_def_ids),
                resource_def_id
            )?;
        }

        Ok(())
    }
}

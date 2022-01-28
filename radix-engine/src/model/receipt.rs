use colored::*;
use scrypto::kernel::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction: ValidatedTransaction,
    pub error: Option<RuntimeError>,
    pub returns: Vec<ValidatedData>,
    pub logs: Vec<(LogLevel, String)>,
    pub new_entities: Vec<Address>,
    pub execution_time: Option<u128>,
}

impl Receipt {
    pub fn package(&self, nth: usize) -> Option<Address> {
        self.new_entities
            .iter()
            .filter(|a| matches!(a, Address::Package(_)))
            .map(Clone::clone)
            .nth(nth)
    }

    pub fn component(&self, nth: usize) -> Option<Address> {
        self.new_entities
            .iter()
            .filter(|a| matches!(a, Address::Component(_)))
            .map(Clone::clone)
            .nth(nth)
    }

    pub fn resource_def(&self, nth: usize) -> Option<Address> {
        self.new_entities
            .iter()
            .filter(|a| matches!(a, Address::ResourceDef(_)))
            .map(Clone::clone)
            .nth(nth)
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
            match &self.error {
                None => "SUCCESS".blue(),
                Some(e) => e.to_string().red(),
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

        write!(f, "\n{}", "Returns:".bold().green())?;
        for (i, result) in self.returns.iter().enumerate() {
            write!(f, "\n{} {:?}", prefix!(i, self.returns), result)?;
        }

        write!(f, "\n{} {}", "Logs:".bold().green(), self.logs.len())?;
        for (i, (level, msg)) in self.logs.iter().enumerate() {
            let (l, m) = match level {
                LogLevel::Error => ("ERROR".red(), msg.red()),
                LogLevel::Warn => ("WARN".yellow(), msg.yellow()),
                LogLevel::Info => ("INFO".green(), msg.green()),
                LogLevel::Debug => ("DEBUG".cyan(), msg.cyan()),
                LogLevel::Trace => ("TRACE".normal(), msg.normal()),
            };
            write!(f, "\n{} [{:5}] {}", prefix!(i, self.logs), l, m)?;
        }

        write!(
            f,
            "\n{} {}",
            "New Entities:".bold().green(),
            self.new_entities.len()
        )?;
        for (i, address) in self.new_entities.iter().enumerate() {
            let ty = match address {
                Address::Package(_) => "Package",
                Address::Component(_) => "Component",
                Address::ResourceDef(_) => "ResourceDef",
                Address::PublicKey(_) => "PublicKey",
            };
            write!(f, "\n{} {}: {}", prefix!(i, self.new_entities), ty, address)?;
        }

        Ok(())
    }
}

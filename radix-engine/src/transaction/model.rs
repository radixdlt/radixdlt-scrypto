use colored::*;
use sbor::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::model::*;
use crate::utils::*;

/// A transaction consists a sequence of instructions.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction in transaction
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Instruction {
    /// Creates an empty bucket
    ReserveBucket { resource_def: Address },

    /// Borrows a bucket, thus creating a reference.
    BorrowBucket { bucket: BID },

    /// Moves resource to a reserved bucket.
    MoveToBucket {
        amount: Amount,
        resource_def: Address,
        bucket: BID,
    },

    /// Calls a function.
    CallFunction {
        blueprint: (Address, String),
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Calls a method.
    CallMethod {
        component: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// Deposits all buckets of resource to a component.
    DepositAll { component: Address, method: String },

    /// Marks the end of a transaction.
    End,
}

/// Represents a transaction self.
pub struct Receipt {
    pub transaction: Transaction,
    pub success: bool,
    pub results: Vec<Result<Option<Vec<u8>>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
    pub new_addresses: Vec<Address>,
}

impl Receipt {
    pub fn nth_package(&self, n: usize) -> Option<Address> {
        self.new_addresses
            .iter()
            .filter(|a| matches!(a, Address::Package(_)))
            .map(Clone::clone)
            .nth(n)
    }

    pub fn nth_component(&self, n: usize) -> Option<Address> {
        self.new_addresses
            .iter()
            .filter(|a| matches!(a, Address::Component(_)))
            .map(Clone::clone)
            .nth(n)
    }

    pub fn nth_resource_def(&self, n: usize) -> Option<Address> {
        self.new_addresses
            .iter()
            .filter(|a| matches!(a, Address::ResourceDef(_)))
            .map(Clone::clone)
            .nth(n)
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
            "{} {}\n",
            "Transaction Status:".bold().green(),
            if self.success {
                "SUCCESS".blue()
            } else {
                "FAILURE".red()
            }
            .bold()
        )?;

        write!(f, "{}\n", "Instructions:".bold().green())?;
        for (i, inst) in self.transaction.instructions.iter().enumerate() {
            write!(
                f,
                "{} {:?}",
                prefix!(i, self.transaction.instructions),
                inst
            )?;
        }

        write!(f, "{}\n", "Results:".bold().green())?;
        for (i, result) in self.results.iter().enumerate() {
            let msg = match result {
                Ok(r) => match r {
                    Some(rtn) => {
                        format!("Ok({})", format_sbor(rtn).unwrap())
                    }
                    None => "Ok".to_string(),
                },
                Err(err) => format!("Err({:?})", err),
            };
            write!(f, "{} {}\n", prefix!(i, self.results), msg)?;
        }

        write!(f, "{} {}\n", "Logs:".bold().green(), self.logs.len())?;
        for (i, (level, msg)) in self.logs.iter().enumerate() {
            let (l, m) = match level {
                Level::Error => ("ERROR".red(), msg.red()),
                Level::Warn => ("WARN".yellow(), msg.yellow()),
                Level::Info => ("INFO".green(), msg.green()),
                Level::Debug => ("DEBUG".cyan(), msg.cyan()),
                Level::Trace => ("TRACE".normal(), msg.normal()),
            };
            write!(f, "{} [{:5}] {}\n", prefix!(i, self.logs), l, m)?;
        }

        write!(
            f,
            "{} {}\n",
            "New Addresses:".bold().green(),
            self.new_addresses.len()
        )?;
        for (i, address) in self.new_addresses.iter().enumerate() {
            let ty = match address {
                Address::Package(_) => "Package",
                Address::Component(_) => "Component",
                Address::ResourceDef(_) => "ResourceDef",
            };
            write!(
                f,
                "{} {}: {}\n",
                prefix!(i, self.new_addresses),
                ty,
                address
            )?;
        }

        Ok(())
    }
}

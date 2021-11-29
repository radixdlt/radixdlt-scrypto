use colored::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::utils::*;

/// Represents a universally recognizable value.
#[derive(Clone, TypeId, Encode, Decode)]
pub struct SmartValue {
    pub encoded: Vec<u8>,
}

impl SmartValue {
    pub fn from<T: Encode>(v: T) -> Self {
        Self {
            encoded: scrypto_encode(&v),
        }
    }
}

impl fmt::Debug for SmartValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.encoded.len() <= 1024 {
            write!(f, "{}", format_data(&self.encoded).unwrap())
        } else {
            write!(f, "LargeValue(len: {})", self.encoded.len())
        }
    }
}

/// A transaction consists a sequence of instructions.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction in transaction
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Instruction {
    /// Declares a temporary bucket for later use.
    DeclareTempBucket,

    /// Declares a temporary bucket ref for later use.
    DeclareTempBucketRef,

    /// Takes resource from transaction context to a temporary bucket.
    TakeFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Bid,
    },

    /// Borrows resource from transaction context to a temporary bucket ref.
    ///
    /// A bucket will be created to support the reference and it will stay within the context.
    BorrowFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Rid,
    },

    /// Calls a blueprint function.
    ///
    /// Buckets and bucket refs in arguments moves from transaction context to the callee.
    CallFunction {
        package_address: Address,
        blueprint_name: String,
        function: String,
        args: Vec<SmartValue>,
    },

    /// Calls a component method.
    ///
    /// Buckets and bucket refs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: Address,
        method: String,
        args: Vec<SmartValue>,
    },

    /// Drops all bucket refs.
    DropAllBucketRefs,

    /// Deposits all resources from transaction context into the designated account.
    DepositAllBuckets { account: Address },

    /// Marks the end of transaction with signatures.
    End { signers: Vec<Address> },
}

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction: Transaction,
    pub success: bool,
    pub results: Vec<Result<Option<SmartValue>, RuntimeError>>,
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
            if self.success {
                "SUCCESS".blue()
            } else {
                "FAILURE".red()
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

        write!(f, "\n{}", "Results:".bold().green())?;
        for (i, result) in self.results.iter().enumerate() {
            write!(f, "\n{} {:?}", prefix!(i, self.results), result)?;
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

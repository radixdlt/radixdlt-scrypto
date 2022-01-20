use colored::*;
use sbor::any::Value;
use sbor::*;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::rust::fmt;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::rust::vec;
use scrypto::types::*;

use crate::engine::*;
use crate::utils::*;

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
        args: Vec<Vec<u8>>,
    },

    /// Calls a component method.
    ///
    /// Buckets and bucket refs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// Drops all bucket refs.
    DropAllBucketRefs,

    /// Deposits all resources from transaction context into the designated account.
    DepositAllBuckets { account: Address },

    /// Marks the end of transaction with signatures.
    /// TODO: replace public key address with signature.
    End { signatures: Vec<Address> },
}

impl Transaction {
    pub fn check(&self) -> Result<CheckedTransaction, CheckTransactionError> {
        // TODO should also consider semantic check, e.g. unused temp bucket/-ref.

        let mut instructions = vec![];
        let mut signers = vec![];
        for (i, inst) in self.instructions.iter().enumerate() {
            match inst.clone() {
                Instruction::DeclareTempBucket => {
                    instructions.push(CheckedInstruction::DeclareTempBucket);
                }
                Instruction::DeclareTempBucketRef => {
                    instructions.push(CheckedInstruction::DeclareTempBucketRef);
                }
                Instruction::TakeFromContext {
                    amount,
                    resource_address,
                    to,
                } => {
                    instructions.push(CheckedInstruction::TakeFromContext {
                        amount,
                        resource_address,
                        to,
                    });
                }
                Instruction::BorrowFromContext {
                    amount,
                    resource_address,
                    to,
                } => {
                    instructions.push(CheckedInstruction::BorrowFromContext {
                        amount,
                        resource_address,
                        to,
                    });
                }
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args,
                } => {
                    let mut checked_args = vec![];
                    for arg in args {
                        checked_args.push(
                            CheckedValue::from_untrusted(&arg)
                                .map_err(|_| CheckTransactionError::InvalidCallArgument)?,
                        );
                    }
                    instructions.push(CheckedInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        function,
                        args: checked_args,
                    });
                }
                Instruction::CallMethod {
                    component_address,
                    method,
                    args,
                } => {
                    let mut checked_args = vec![];
                    for arg in args {
                        checked_args.push(
                            CheckedValue::from_untrusted(&arg)
                                .map_err(|_| CheckTransactionError::InvalidCallArgument)?,
                        );
                    }
                    instructions.push(CheckedInstruction::CallMethod {
                        component_address,
                        method,
                        args: checked_args,
                    });
                }
                Instruction::DropAllBucketRefs => {
                    instructions.push(CheckedInstruction::DeclareTempBucketRef);
                }
                Instruction::DepositAllBuckets { account } => {
                    instructions.push(CheckedInstruction::DepositAllBuckets { account });
                }
                Instruction::End { signatures } => {
                    if i != self.instructions.len() - 1 {
                        return Err(CheckTransactionError::UnexpectedEnd);
                    }
                    signers.extend(signatures);
                }
            }
        }

        Ok(CheckedTransaction {
            instructions,
            signers,
        })
    }
}

#[derive(Debug)]
pub enum CheckTransactionError {
    InvalidCallArgument,
    InvalidSignature,
    UnexpectedEnd,
}

#[derive(Debug, Clone)]
pub struct CheckedTransaction {
    pub instructions: Vec<CheckedInstruction>,
    pub signers: Vec<Address>,
}

#[derive(Debug, Clone)]
pub enum CheckedInstruction {
    DeclareTempBucket,
    DeclareTempBucketRef,
    TakeFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Bid,
    },
    BorrowFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Rid,
    },
    CallFunction {
        package_address: Address,
        blueprint_name: String,
        function: String,
        args: Vec<CheckedValue>,
    },
    CallMethod {
        component_address: Address,
        method: String,
        args: Vec<CheckedValue>,
    },
    DropAllBucketRefs,
    DepositAllBuckets {
        account: Address,
    },
}

#[derive(Clone)]
pub struct CheckedValue {
    pub bytes: Vec<u8>,
    pub value: Value,
}

impl CheckedValue {
    pub fn from_trusted<T: Encode>(v: T) -> Self {
        let bytes = scrypto_encode(&v);
        let value = decode_any(&bytes).unwrap();

        Self { bytes, value }
    }

    pub fn from_untrusted(raw: &[u8]) -> Result<Self, DecodeError> {
        let bytes = raw.to_vec();
        let value = decode_any(&bytes)?;
        // TODO: recursively check custom types
        // We should also consider if SBOR should be Scrypto-specific or generic.
        // The benefits of the former is that we can move the custom type validation
        // to SBOR.
        Ok(Self { bytes, value })
    }
}

impl fmt::Debug for CheckedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: format the value based on the tiny lang introduced by transaction manifest.
        if self.bytes.len() <= 1024 {
            write!(f, "{}", format_data(&self.bytes).unwrap())
        } else {
            write!(f, "LargeValue(len: {})", self.bytes.len())
        }
    }
}

/// Represents a transaction receipt.
pub struct Receipt {
    pub transaction: CheckedTransaction,
    pub success: bool,
    pub results: Vec<Result<Option<CheckedValue>, RuntimeError>>,
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

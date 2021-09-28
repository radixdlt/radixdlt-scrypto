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

/// Represents an instruction
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Instruction {
    /// Reserve buckets for calls.
    ReserveBuckets {
        n: u8,
    },

    /// Move resource to a reserved bucket.
    MoveToBucket {
        amount: Amount,
        resource_address: Address,
        index: u8,
    },

    /// Call a function.
    CallFunction {
        blueprint: (Address, String),
        function: String,
        args: Args,
    },

    /// Call a method.
    CallMethod {
        component: Address,
        method: String,
        args: Args,
    },

    /// Deposit all buckets of resource to a component.
    DepositAll {
        component: Address,
        method: String,
    },

    Finalize,
}

#[derive(Clone, TypeId, Encode, Decode)]
pub struct Args(pub Vec<Vec<u8>>);

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|v| format_sbor(v).unwrap())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct TransactionReceipt {
    pub transaction: Transaction,
    pub success: bool,
    pub results: Vec<Result<Option<Vec<u8>>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
    pub new_addresses: Vec<Address>,
}

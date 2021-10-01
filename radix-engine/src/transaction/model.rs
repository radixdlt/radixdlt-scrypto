use sbor::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::model::*;

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

/// Represents a transaction receipt.
#[derive(Debug)]
pub struct Receipt {
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

use radix_engine::execution::*;
use radix_engine::model::Level;
use sbor::*;
use scrypto::types::*;

/// A transaction consists a sequence of instructions.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction
#[derive(Debug, Clone, Encode, Decode)]
pub enum Instruction {
    /// Reserve `n` buckets upfront.
    ReserveBuckets {
        n: u8,
    },

    /// Create a bucket to be used for function call.
    NewBucket {
        offset: u8,
        amount: U256,
        resource: Address,
    },

    /// Call a function.
    CallFunction {
        package: Address,
        blueprint: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Call a method.
    CallMethod {
        component: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// Pass all remaining resources to a component.
    DepositAll {
        component: Address,
        method: String,
    },

    Finalize,
}

#[derive(Debug)]
pub struct TransactionReceipt {
    pub transaction: Transaction,
    pub success: bool,
    pub execution_time: u128,
    pub results: Vec<Result<Vec<u8>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
    pub new_addresses: Vec<Address>,
}

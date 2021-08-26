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
    ReserveBuckets { n: u8 },

    /// Create a bucket to be used for function call.
    PrepareBucket {
        id: u8,
        amount: U256,
        resource: Address,
    },

    /// Invoke a function.
    InvokeFunction {
        package: Address,
        blueprint: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Invoke a method.
    InvokeMethod {
        component: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },
}

#[derive(Debug)]
pub struct TransactionReceipt {
    pub transaction: Transaction,
    pub results: Vec<Result<Vec<u8>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
}

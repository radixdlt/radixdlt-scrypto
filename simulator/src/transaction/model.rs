use radix_engine::execution::*;
use sbor::DecodeError;
use scrypto::types::*;

/// A transaction is a collection of actions to execute.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    Withdraw {
        amount: U256,
        resource: Address,
    },

    CallBlueprint {
        package: Address,
        blueprint: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    CallComponent {
        component: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },

    Deposit {
        amount: U256,
        resource: Address,
    },

    DepositAll,
}

#[derive(Debug)]
pub struct TransactionReceipt {
    pub transaction: Transaction,
    pub success: bool,
    pub results: Vec<Result<Vec<u8>, RuntimeError>>,
    pub logs: Vec<(Level, String)>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TransactionError {
    PackageNotFound(Address),
    FailedToExportAbi(RuntimeError),
    FailedToParseAbi(DecodeError),
    FinalizationError(RuntimeError),
    FailedToWithdraw,
    FailedToDeposit,
}

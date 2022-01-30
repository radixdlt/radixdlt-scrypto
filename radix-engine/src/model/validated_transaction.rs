use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::*;

#[derive(Debug, Clone)]
pub struct ValidatedTransaction {
    pub instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<Address>,
}

#[derive(Debug, Clone)]
pub enum ValidatedInstruction {
    CreateBucket {
        amount: Decimal,
        resource_address: Address,
    },
    CreateBucketRef {
        bid: Bid,
    },
    CloneBucketRef {
        rid: Rid,
    },
    DropBucketRef {
        rid: Rid,
    },
    CallFunction {
        package_address: Address,
        blueprint_name: String,
        function: String,
        args: Vec<ValidatedData>,
    },
    CallMethod {
        component_address: Address,
        method: String,
        args: Vec<ValidatedData>,
    },
    CallMethodWithAllResources {
        component_address: Address,
        method: String,
    },
}

use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use scrypto_abi::BlueprintAbi;

pub struct TransactionRuntimeAbi;

impl TransactionRuntimeAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const TRANSACTION_RUNTIME_BLUEPRINT: &str = "TransactionRuntime";

pub const TRANSACTION_RUNTIME_GET_HASH_IDENT: &str = "get_hash";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGetHashInput {}

pub const TRANSACTION_RUNTIME_GENERATE_UUID_IDENT: &str = "generate_uuid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionRuntimeGenerateUuid {}

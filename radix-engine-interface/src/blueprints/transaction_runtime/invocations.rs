use crate::*;
use sbor::rust::fmt::Debug;
use scrypto_schema::PackageSchema;

pub const TRANSACTION_RUNTIME_BLUEPRINT: &str = "TransactionRuntime";

pub const TRANSACTION_RUNTIME_GET_HASH_IDENT: &str = "get_hash";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionRuntimeGetHashInput {}

pub const TRANSACTION_RUNTIME_GENERATE_UUID_IDENT: &str = "generate_uuid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionRuntimeGenerateUuid {}

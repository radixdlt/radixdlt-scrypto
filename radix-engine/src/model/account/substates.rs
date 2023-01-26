use crate::types::*;
use radix_engine_interface::api::types::KeyValueStoreId;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountSubstate {
    pub vaults: KeyValueStoreId,
}

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccountSubstate {
    /// An owned [`KeyValueStore`] which maps the [`ResourceAddress`] to an [`Own`] of the vault
    /// containing that resource.
    pub vaults: Own,
}

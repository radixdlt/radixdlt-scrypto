use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::math::*;
use crate::resource::*;

/// Represents the minting config
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum MintParams {
    /// To mint fungible resource, represented by an amount
    Fungible { amount: Decimal },

    /// To mint non-fungible resource, represented by non-fungible id and data pairs
    NonFungible {
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
    },
}

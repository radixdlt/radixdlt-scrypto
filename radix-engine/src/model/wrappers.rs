use crate::model::{NonFungible, Resource};
use crate::types::*;

/// To support non-fungible deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleWrapper(pub Option<NonFungible>);

/// To support key value store entry deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntryWrapper(pub Option<Vec<u8>>);

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SingleBalanceVault(pub Resource);

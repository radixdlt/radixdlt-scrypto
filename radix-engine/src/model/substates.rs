use crate::model::{NonFungible, Resource};
use crate::types::*;

/// To support non-fungible deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NonFungibleSubstate(pub Option<NonFungible>);

/// To support key value store entry deletion, we wrap it into a container
/// when persisting into the substate store.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntrySubstate(pub Option<Vec<u8>>);

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VaultSubstate(pub Resource);

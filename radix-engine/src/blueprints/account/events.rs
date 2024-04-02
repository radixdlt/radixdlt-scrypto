use crate::internal_prelude::*;
use radix_common::math::Decimal;
use radix_common::{ScryptoEvent, ScryptoSbor};
use radix_engine_interface::blueprints::account::*;
use sbor::rust::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub enum WithdrawEvent {
    Fungible(ResourceAddress, Decimal),
    NonFungible(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub enum DepositEvent {
    Fungible(ResourceAddress, Decimal),
    NonFungible(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub enum RejectedDepositEvent {
    Fungible(ResourceAddress, Decimal),
    NonFungible(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetResourcePreferenceEvent {
    pub resource_address: ResourceAddress,
    pub preference: ResourcePreference,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveResourcePreferenceEvent {
    pub resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetDefaultDepositRuleEvent {
    pub default_deposit_rule: DefaultDepositRule,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddAuthorizedDepositorEvent {
    pub authorized_depositor_badge: ResourceOrNonFungible,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveAuthorizedDepositorEvent {
    pub authorized_depositor_badge: ResourceOrNonFungible,
}

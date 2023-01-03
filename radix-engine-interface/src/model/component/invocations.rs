use sbor::rust::fmt::Debug;
use sbor::*;

use crate::api::api::*;
use crate::api::types::{ComponentId, RENodeId};
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentGlobalizeInvocation {
    pub component_id: ComponentId,
}

impl Invocation for ComponentGlobalizeInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for ComponentGlobalizeInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<SerializedInvocation> for ComponentGlobalizeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Component(ComponentInvocation::Globalize(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentGlobalizeWithOwnerInvocation {
    pub component_id: ComponentId,
    pub owner_badge: NonFungibleAddress,
}

impl Invocation for ComponentGlobalizeWithOwnerInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<SerializedInvocation> for ComponentGlobalizeWithOwnerInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Component(ComponentInvocation::GlobalizeWithOwner(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentSetRoyaltyConfigInvocation {
    /// TODO: change to component id, after `borrow_component` returns component id
    pub receiver: RENodeId,
    pub royalty_config: RoyaltyConfig,
}

impl Invocation for ComponentSetRoyaltyConfigInvocation {
    type Output = ();
}

impl SerializableInvocation for ComponentSetRoyaltyConfigInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ComponentSetRoyaltyConfigInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Component(ComponentInvocation::SetRoyaltyConfig(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentClaimRoyaltyInvocation {
    /// TODO: change to component id, after `borrow_component` returns component id
    pub receiver: RENodeId,
}

impl Invocation for ComponentClaimRoyaltyInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ComponentClaimRoyaltyInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for ComponentClaimRoyaltyInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Component(ComponentInvocation::ClaimRoyalty(self)).into()
    }
}

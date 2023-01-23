use sbor::rust::fmt::Debug;

use crate::api::types::{ComponentId, RENodeId};
use crate::api::wasm::*;
use crate::api::*;
use crate::model::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ComponentGlobalizeInvocation {
    pub component_id: ComponentId,
}

impl Invocation for ComponentGlobalizeInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for ComponentGlobalizeInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for ComponentGlobalizeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Component(ComponentInvocation::Globalize(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ComponentGlobalizeWithOwnerInvocation {
    pub component_id: ComponentId,
    pub owner_badge: NonFungibleGlobalId,
}

impl Invocation for ComponentGlobalizeWithOwnerInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for ComponentGlobalizeWithOwnerInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for ComponentGlobalizeWithOwnerInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Component(ComponentInvocation::GlobalizeWithOwner(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ComponentSetRoyaltyConfigInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Component(ComponentInvocation::SetRoyaltyConfig(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for ComponentClaimRoyaltyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Component(ComponentInvocation::ClaimRoyalty(self)).into()
    }
}

use radix_engine_derive::LegacyDescribe;
use radix_engine_interface::api::types::{
    ComponentId, ComponentOffset, GlobalAddress, RENodeId, ScryptoReceiver, SubstateOffset,
};
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::{
    scrypto_decode, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::model::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;

use super::ComponentAccessRules;

/// Represents the state of a component.
pub trait ComponentState<C: LocalComponent>: ScryptoEncode + ScryptoDecode {
    /// Instantiates a component from this data structure.
    fn instantiate(self) -> C;
}

/// A separate trait for standardized calls so that component methods don't
/// name clash
/// TODO: unify with LocalComponent and use Own<C> and GlobalRef<C> Deref structures
pub trait GlobalComponent {
    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
    fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self;
    fn add_access_check(&mut self, access_rules: AccessRules) -> &mut Self;
    fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig) -> &mut Self;
    fn claim_royalty(&self) -> Bucket;
    fn access_rules_chain(&self) -> Vec<ComponentAccessRules>;
}

/// A separate trait for standardized calls so that component methods don't
/// name clash
pub trait LocalComponent {
    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
    fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self;
    fn add_access_check(&mut self, access_rules: AccessRules) -> &mut Self;
    fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig) -> &mut Self;
    fn globalize(self) -> ComponentAddress;
    fn globalize_with_owner(self, owner_badge: NonFungibleGlobalId) -> ComponentAddress;
}

// TODO: de-duplication
#[derive(
    Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct ComponentInfoSubstate {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

// TODO: de-duplication
#[derive(Debug, Clone, Categorize, Encode, Decode, LegacyDescribe, PartialEq, Eq)]
pub struct ComponentStateSubstate {
    pub raw: Vec<u8>,
}

/// Represents an instantiated component.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Component(pub ComponentId);

impl Component {
    /// Invokes a method on this component.
    pub fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .invoke_method(ScryptoReceiver::Component(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    /// Returns the package ID of this component.
    pub fn package_address(&self) -> PackageAddress {
        let pointer = DataPointer::new(
            RENodeId::Component(self.0),
            SubstateOffset::Component(ComponentOffset::Info),
        );
        let state: DataRef<ComponentInfoSubstate> = pointer.get();
        state.package_address
    }

    /// Returns the blueprint name of this component.
    pub fn blueprint_name(&self) -> String {
        let pointer = DataPointer::new(
            RENodeId::Component(self.0),
            SubstateOffset::Component(ComponentOffset::Info),
        );
        let state: DataRef<ComponentInfoSubstate> = pointer.get();
        state.blueprint_name.clone()
    }

    /// Add access check on the component.
    pub fn add_access_check(&mut self, access_rules: AccessRules) -> &mut Self {
        ScryptoEnv
            .invoke(AccessRulesAddAccessCheckInvocation {
                receiver: RENodeId::Component(self.0),
                access_rules,
            })
            .unwrap();
        self
    }

    /// Set the royalty configuration of the component.
    pub fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig) -> &mut Self {
        ScryptoEnv
            .invoke(ComponentSetRoyaltyConfigInvocation {
                receiver: RENodeId::Component(self.0),
                royalty_config,
            })
            .unwrap();
        self
    }

    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        ScryptoEnv
            .invoke(MetadataSetInvocation {
                receiver: RENodeId::Component(self.0),
                key: name.as_ref().to_owned(),
                value: value.as_ref().to_owned(),
            })
            .unwrap();
        self
    }

    pub fn globalize(self) -> ComponentAddress {
        ScryptoEnv
            .invoke(ComponentGlobalizeInvocation {
                component_id: self.0,
            })
            .unwrap()
    }

    /// Globalize with owner badge. This will add additional access rules to protect native
    /// methods, such as metadata and royalty.
    pub fn globalize_with_owner(self, owner_badge: NonFungibleGlobalId) -> ComponentAddress {
        ScryptoEnv
            .invoke(ComponentGlobalizeWithOwnerInvocation {
                component_id: self.0,
                owner_badge,
            })
            .unwrap()
    }

    /// Returns the layers of access rules on this component.
    pub fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let mut env = ScryptoEnv;
        let length = env
            .invoke(AccessRulesGetLengthInvocation {
                receiver: RENodeId::Component(self.0),
            })
            .unwrap();
        (0..length)
            .into_iter()
            .map(|id| ComponentAccessRules::new(self.0, id))
            .collect()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl GlobalComponentRef {
    /// Invokes a method on this component.
    pub fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .invoke_method(ScryptoReceiver::Global(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        ScryptoEnv
            .invoke(MetadataSetInvocation {
                receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
                key: name.as_ref().to_owned(),
                value: value.as_ref().to_owned(),
            })
            .unwrap();
        self
    }

    /// Add access check on the component.
    pub fn add_access_check(&mut self, access_rules: AccessRules) -> &mut Self {
        let mut env = ScryptoEnv;
        env.invoke(AccessRulesAddAccessCheckInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            access_rules,
        })
        .unwrap();
        self
    }

    /// Returns the package ID of this component.
    pub fn package_address(&self) -> PackageAddress {
        let pointer = DataPointer::new(
            RENodeId::Global(GlobalAddress::Component(self.0)),
            SubstateOffset::Component(ComponentOffset::Info),
        );
        let state: DataRef<ComponentInfoSubstate> = pointer.get();
        state.package_address
    }

    /// Returns the blueprint name of this component.
    pub fn blueprint_name(&self) -> String {
        let pointer = DataPointer::new(
            RENodeId::Global(GlobalAddress::Component(self.0)),
            SubstateOffset::Component(ComponentOffset::Info),
        );
        let state: DataRef<ComponentInfoSubstate> = pointer.get();
        state.blueprint_name.clone()
    }

    pub fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        let mut env = ScryptoEnv;

        env.invoke(ComponentSetRoyaltyConfigInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            royalty_config,
        })
        .unwrap();
    }

    pub fn claim_royalty(&self) -> Bucket {
        let mut env = ScryptoEnv;

        env.invoke(ComponentClaimRoyaltyInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
        })
        .unwrap()
    }

    /// Returns the layers of access rules on this component.
    pub fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let mut env = ScryptoEnv;
        let length = env
            .invoke(AccessRulesGetLengthInvocation {
                receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            })
            .unwrap();
        (0..length)
            .into_iter()
            .map(|id| ComponentAccessRules::new(self.0, id))
            .collect()
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Component {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Component {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Component(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Component {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Component(component_id) => Ok(Self(component_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Component {
    fn describe() -> scrypto_abi::Type {
        Type::Component
    }
}

use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::component::{
    ComponentClaimRoyaltyInvocation, ComponentSetRoyaltyConfigInvocation,
};
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesAddAccessCheckInvocation, AccessRulesGetLengthInvocation,
};
use radix_engine_interface::api::node_modules::metadata::MetadataSetInvocation;
use radix_engine_interface::api::types::{ComponentId, GlobalAddress, RENodeId};
use radix_engine_interface::api::Invokable;
use radix_engine_interface::api::{types::*, ClientComponentApi};
use radix_engine_interface::blueprints::resource::{AccessRules, Bucket};
use radix_engine_interface::data::{
    scrypto_decode, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

use super::ComponentAccessRules;

pub trait ComponentState<T: Component + LocalComponent>: ScryptoEncode + ScryptoDecode {
    fn instantiate(self) -> T;
}

pub trait Component {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T;

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V);
    fn add_access_check(&mut self, access_rules: AccessRules);
    fn protect_with_owner_badge(&mut self, owner_badge: NonFungibleGlobalId);
    fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig);
    fn claim_royalty(&mut self) -> Bucket;

    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
    fn access_rules_chain(&self) -> Vec<ComponentAccessRules>;
    // fn metadata<K: AsRef<str>>(&self, name: K) -> Option<String>;
}

pub trait LocalComponent {
    fn globalize(self) -> ComponentAddress;
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct OwnedComponent(pub ComponentId);

impl Component for OwnedComponent {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(ScryptoReceiver::Component(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) {
        ScryptoEnv
            .invoke(MetadataSetInvocation {
                receiver: RENodeId::Component(self.0),
                key: name.as_ref().to_owned(),
                value: value.as_ref().to_owned(),
            })
            .unwrap();
    }

    fn add_access_check(&mut self, access_rules: AccessRules) {
        ScryptoEnv
            .invoke(AccessRulesAddAccessCheckInvocation {
                receiver: RENodeId::Component(self.0),
                access_rules,
            })
            .unwrap();
    }

    fn protect_with_owner_badge(&mut self, owner_badge: NonFungibleGlobalId) {
        todo!()
    }

    fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig) {
        ScryptoEnv
            .invoke(ComponentSetRoyaltyConfigInvocation {
                receiver: RENodeId::Component(self.0),
                royalty_config,
            })
            .unwrap();
    }

    fn claim_royalty(&mut self) -> Bucket {
        todo!()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv.get_type_info(self.0).unwrap().0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv.get_type_info(self.0).unwrap().1
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
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

impl LocalComponent for OwnedComponent {
    fn globalize(self) -> ComponentAddress {
        ScryptoEnv.globalize_component(self.0).unwrap()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl Component for GlobalComponentRef {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(ScryptoReceiver::Global(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) {
        ScryptoEnv
            .invoke(MetadataSetInvocation {
                receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
                key: name.as_ref().to_owned(),
                value: value.as_ref().to_owned(),
            })
            .unwrap();
    }

    fn add_access_check(&mut self, access_rules: AccessRules) {
        let mut env = ScryptoEnv;
        env.invoke(AccessRulesAddAccessCheckInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            access_rules,
        })
        .unwrap();
    }

    fn package_address(&self) -> PackageAddress {
        todo!()
    }

    fn blueprint_name(&self) -> String {
        todo!()
    }

    fn set_royalty_config(&mut self, royalty_config: RoyaltyConfig) {
        let mut env = ScryptoEnv;

        env.invoke(ComponentSetRoyaltyConfigInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            royalty_config,
        })
        .unwrap();
    }

    fn protect_with_owner_badge(&mut self, owner_badge: NonFungibleGlobalId) {
        todo!()
    }

    fn claim_royalty(&mut self) -> Bucket {
        let mut env = ScryptoEnv;

        env.invoke(ComponentClaimRoyaltyInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
        })
        .unwrap()
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
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

impl Categorize<ScryptoCustomValueKind> for OwnedComponent {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for OwnedComponent {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Component(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for OwnedComponent {
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

impl scrypto_abi::LegacyDescribe for OwnedComponent {
    fn describe() -> scrypto_abi::Type {
        Type::Component
    }
}

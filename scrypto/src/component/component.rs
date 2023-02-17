use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesAddAccessCheckInvocation, AccessRulesGetLengthInvocation,
};
use radix_engine_interface::api::node_modules::metadata::{MetadataSetInput, METADATA_SET_IDENT};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInvocation, ComponentSetRoyaltyConfigInvocation,
};
use radix_engine_interface::api::types::{ComponentId, GlobalAddress, RENodeId};
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::api::{types::*, ClientComponentApi};
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRuleKey, AccessRules, Bucket,
};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::rule;
use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

use super::ComponentAccessRules;

pub trait ComponentState<T: Component + LocalComponent>: ScryptoEncode + ScryptoDecode {
    fn instantiate(self) -> T;
}

// TODO: I've temporarily disabled &mut requirement on the Component trait.
// If not, I will have to overhaul the `ComponentSystem` infra, which will be likely be removed anyway.
//
// Since there is no mutability semantics in the system and kernel, there is no technical benefits
// with &mut, other than to frustrate developers.

pub trait Component {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T;

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V);
    fn add_access_check(&self, access_rules: AccessRules);
    fn set_royalty_config(&self, royalty_config: RoyaltyConfig);
    fn claim_royalty(&self) -> Bucket;

    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
    fn access_rules_chain(&self) -> Vec<ComponentAccessRules>;
    // TODO: fn metadata<K: AsRef<str>>(&self, name: K) -> Option<String>;

    /// Protects this component with owner badge
    fn with_owner_badge(&self, owner_badge: NonFungibleGlobalId) {
        let mut access_rules =
            AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
            AccessRule::AllowAll,
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::ScryptoMethod(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::ComponentRoyalty(
                ComponentRoyaltyFn::SetRoyaltyConfig,
            )),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            AccessRuleKey::Native(NativeFn::ComponentRoyalty(ComponentRoyaltyFn::ClaimRoyalty)),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );

        self.add_access_check(access_rules);
    }
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

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        ScryptoEnv
            .call_module_method(
                ScryptoReceiver::Component(self.0),
                NodeModuleId::Metadata,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn add_access_check(&self, access_rules: AccessRules) {
        ScryptoEnv
            .call_native(AccessRulesAddAccessCheckInvocation {
                receiver: RENodeId::Component(self.0),
                access_rules,
            })
            .unwrap();
    }

    fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        ScryptoEnv
            .call_native(ComponentSetRoyaltyConfigInvocation {
                receiver: RENodeId::Component(self.0),
                royalty_config,
            })
            .unwrap();
    }

    fn claim_royalty(&self) -> Bucket {
        ScryptoEnv
            .call_native(ComponentClaimRoyaltyInvocation {
                receiver: RENodeId::Component(self.0),
            })
            .unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv.get_component_type_info(self.0).unwrap().0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv.get_component_type_info(self.0).unwrap().1
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let mut env = ScryptoEnv;
        let length = env
            .call_native(AccessRulesGetLengthInvocation {
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

    fn set_metadata<K: AsRef<str>, V: AsRef<str>>(&self, name: K, value: V) {
        ScryptoEnv
            .call_module_method(
                ScryptoReceiver::Global(self.0),
                NodeModuleId::Metadata,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn add_access_check(&self, access_rules: AccessRules) {
        let mut env = ScryptoEnv;
        env.call_native(AccessRulesAddAccessCheckInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            access_rules,
        })
        .unwrap();
    }

    fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        let mut env = ScryptoEnv;
        env.call_native(ComponentSetRoyaltyConfigInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
            royalty_config,
        })
        .unwrap();
    }

    fn claim_royalty(&self) -> Bucket {
        let mut env = ScryptoEnv;
        env.call_native(ComponentClaimRoyaltyInvocation {
            receiver: RENodeId::Global(GlobalAddress::Component(self.0)),
        })
        .unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv.get_global_component_type_info(self.0).unwrap().0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv.get_global_component_type_info(self.0).unwrap().1
    }

    fn access_rules_chain(&self) -> Vec<ComponentAccessRules> {
        let mut env = ScryptoEnv;
        let length = env
            .call_native(AccessRulesGetLengthInvocation {
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

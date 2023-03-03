use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentRoyaltyCreateInput, ComponentSetRoyaltyConfigInput,
    COMPONENT_ROYALTY_BLUEPRINT, COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
    COMPONENT_ROYALTY_CREATE_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::types::{ObjectId, RENodeId};
use radix_engine_interface::api::{types::*, ClientObjectApi, ClientPackageApi};
use radix_engine_interface::blueprints::resource::{require, AccessRule, AccessRules, Bucket, MethodKey, AccessRuleEntry};
use radix_engine_interface::constants::{ACCESS_RULES_PACKAGE, ROYALTY_PACKAGE};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::rule;
use sbor::rust::vec::Vec;
use scrypto::modules::Metadata;
use crate::modules::AttachedMetadata;

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
    fn package_address(&self) -> PackageAddress;
    fn blueprint_name(&self) -> String;
}

pub trait LocalComponent: Sized {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        config: RoyaltyConfig,
    ) -> ComponentAddress;


    fn globalize(self) -> ComponentAddress {
        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()), AccessRuleEntry::AccessRule(AccessRule::DenyAll));
        let access_rules = access_rules.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            access_rules,
            Metadata::new(),
            RoyaltyConfig::default(),
        )
    }

    fn globalize_with_metadata(self, metadata: Metadata) -> ComponentAddress {
        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()), AccessRuleEntry::AccessRule(AccessRule::DenyAll));
        let access_rules = access_rules.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            access_rules,
            metadata,
            RoyaltyConfig::default(),
        )
    }

    fn globalize_with_royalty_config(self, config: RoyaltyConfig) -> ComponentAddress {
        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()), AccessRuleEntry::AccessRule(AccessRule::DenyAll));
        let access_rules = access_rules.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            access_rules,
            Metadata::new(),
            config,
        )
    }

    fn globalize_with_access_rules(self, access_rules: AccessRules) -> ComponentAddress {
        self.globalize_with_modules(access_rules, Metadata::new(), RoyaltyConfig::default())
    }

    fn globalize_with_owner_badge(
        self,
        owner_badge: NonFungibleGlobalId,
        royalty_config: RoyaltyConfig,
    ) -> ComponentAddress {
        let mut access_rules =
            AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
            AccessRule::AllowAll,
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(),
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );

        self.globalize_with_modules(access_rules, Metadata::new(), royalty_config)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct OwnedComponent(pub ObjectId);

impl Component for OwnedComponent {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(RENodeId::Object(self.0), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_object_type_info(RENodeId::Object(self.0))
            .unwrap()
            .0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_object_type_info(RENodeId::Object(self.0))
            .unwrap()
            .1
    }
}

impl LocalComponent for OwnedComponent {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        config: RoyaltyConfig,
    ) -> ComponentAddress {
        let metadata: Own = Own::Object(metadata.0);

        let rtn = ScryptoEnv
            .call_function(
                ROYALTY_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
                COMPONENT_ROYALTY_CREATE_IDENT,
                scrypto_encode(&ComponentRoyaltyCreateInput {
                    royalty_config: config,
                })
                .unwrap(),
            )
            .unwrap();
        let royalty: Own = scrypto_decode(&rtn).unwrap();

        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput { access_rules }).unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();

        let address = ScryptoEnv
            .globalize(
                RENodeId::Object(self.0),
                btreemap!(
                    NodeModuleId::AccessRules => scrypto_encode(&access_rules).unwrap(),
                    NodeModuleId::Metadata => scrypto_encode(&metadata).unwrap(),
                    NodeModuleId::ComponentRoyalty => scrypto_encode(&royalty).unwrap()
                ),
            )
            .unwrap();

        address.into()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl GlobalComponentRef {
    pub fn access_rules(&self) -> ComponentAccessRules {
        ComponentAccessRules::new(self.0)
    }

    pub fn metadata(&self) -> AttachedMetadata {
        AttachedMetadata(self.0.into())
    }

    pub fn set_royalty_config(&self, royalty_config: RoyaltyConfig) {
        ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                scrypto_encode(&ComponentSetRoyaltyConfigInput { royalty_config }).unwrap(),
            )
            .unwrap();
    }

    pub fn claim_royalty(&self) -> Bucket {
        let rtn = ScryptoEnv
            .call_module_method(
                RENodeId::GlobalComponent(self.0),
                NodeModuleId::ComponentRoyalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}

impl Component for GlobalComponentRef {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(RENodeId::GlobalComponent(self.0.into()), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_object_type_info(RENodeId::GlobalComponent(self.0))
            .unwrap()
            .0
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_object_type_info(RENodeId::GlobalComponent(self.0))
            .unwrap()
            .1
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
        Own::Object(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for OwnedComponent {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Object(component_id) => Ok(Self(component_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for OwnedComponent {
    fn describe() -> scrypto_abi::Type {
        Type::Component
    }
}

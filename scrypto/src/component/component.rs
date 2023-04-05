use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{
    AccessRules, AttachedAccessRules, AttachedMetadata, AttachedRoyalty, Royalty,
};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRuleEntry, AccessRulesConfig, MethodKey, NonFungibleGlobalId,
};
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::OWN_ID;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::rule;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Describe, Encode, EncodeError, Encoder, GlobalTypeId,
    ValueKind,
};
use scrypto::modules::Metadata;

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
        royalty: Royalty,
    ) -> ComponentAddress;

    fn globalize(self) -> ComponentAddress {
        let mut access_rules_config = AccessRulesConfig::new();
        access_rules_config.set_method_access_rule(
            MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        );
        let access_rules_config =
            access_rules_config.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            AccessRules::new(access_rules_config),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_metadata(self, metadata: Metadata) -> ComponentAddress {
        let mut access_rules_config = AccessRulesConfig::new();
        access_rules_config.set_method_access_rule(
            MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        );
        let access_rules_config =
            access_rules_config.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            AccessRules::new(access_rules_config),
            metadata,
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_royalty_config(self, royalty_config: RoyaltyConfig) -> ComponentAddress {
        let mut access_rules_config = AccessRulesConfig::new();
        access_rules_config.set_method_access_rule(
            MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        );
        let access_rules_config =
            access_rules_config.default(AccessRule::AllowAll, AccessRule::DenyAll);

        self.globalize_with_modules(
            AccessRules::new(access_rules_config),
            Metadata::new(),
            Royalty::new(royalty_config),
        )
    }

    fn globalize_with_access_rules(
        self,
        access_rules_config: AccessRulesConfig,
    ) -> ComponentAddress {
        self.globalize_with_modules(
            AccessRules::new(access_rules_config),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_owner_badge(
        self,
        owner_badge: NonFungibleGlobalId,
        royalty_config: RoyaltyConfig,
    ) -> ComponentAddress {
        let mut access_rules_config =
            AccessRulesConfig::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
        access_rules_config.set_method_access_rule_and_mutability(
            MethodKey::new(TypedModuleId::Metadata, METADATA_GET_IDENT),
            AccessRule::AllowAll,
            rule!(require(owner_badge.clone())),
        );
        access_rules_config.set_method_access_rule_and_mutability(
            MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules_config.set_method_access_rule_and_mutability(
            MethodKey::new(
                TypedModuleId::Royalty,
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );
        access_rules_config.set_method_access_rule_and_mutability(
            MethodKey::new(
                TypedModuleId::Royalty,
                COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
            ),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );

        self.globalize_with_modules(
            AccessRules::new(access_rules_config),
            Metadata::new(),
            Royalty::new(royalty_config),
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct OwnedComponent(pub Own);

impl Component for OwnedComponent {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.0.as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_object_info(self.0.as_node_id())
            .unwrap()
            .blueprint
            .package_address
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_object_info(self.0.as_node_id())
            .unwrap()
            .blueprint
            .blueprint_name
    }
}

impl LocalComponent for OwnedComponent {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress {
        let metadata: Own = metadata.0;
        let access_rules: Own = access_rules.0;
        let royalty: Own = royalty.0;

        let address = ScryptoEnv
            .globalize(
                self.0.as_node_id().clone(),
                btreemap!(
                    TypedModuleId::AccessRules => access_rules.0,
                    TypedModuleId::Metadata => metadata.0,
                    TypedModuleId::Royalty => royalty.0,
                ),
            )
            .unwrap();

        ComponentAddress::new_unchecked(address.into())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl GlobalComponentRef {
    pub fn access_rules(&self) -> AttachedAccessRules {
        AttachedAccessRules(self.0.into())
    }

    pub fn metadata(&self) -> AttachedMetadata {
        AttachedMetadata(self.0.into())
    }

    pub fn royalty(&self) -> AttachedRoyalty {
        AttachedRoyalty(self.0.into())
    }
}

impl Component for GlobalComponentRef {
    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.0.as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_object_info(self.0.as_node_id())
            .unwrap()
            .blueprint
            .package_address
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_object_info(self.0.as_node_id())
            .unwrap()
            .blueprint
            .blueprint_name
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
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for OwnedComponent {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

// TODO: generics support for Scrypto components?
impl Describe<ScryptoCustomTypeKind> for OwnedComponent {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::WellKnown([OWN_ID]);
}

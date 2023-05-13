use std::ops::Deref;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{
    AccessRules, AttachedAccessRules, AttachedRoyalty, Royalty,
};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::{
    require, AuthorityRules, MethodAuthorities, NonFungibleGlobalId,
};
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::{
    own_type_data, OWN_ID,
};
use radix_engine_interface::data::scrypto::{
    scrypto_decode, ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::rule;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;
use sbor::*;
use scrypto::modules::{Attached, Metadata};
use crate::prelude::ScryptoSbor;

pub trait ComponentState<T: Component + LocalComponent>: ScryptoEncode + ScryptoDecode {
    fn instantiate(self) -> T;
}

// TODO: I've temporarily disabled &mut requirement on the Component trait.
// If not, I will have to overhaul the `ComponentSystem` infra, which will be likely be removed anyway.
//
// Since there is no mutability semantics in the system and kernel, there is no technical benefits
// with &mut, other than to frustrate developers.

pub trait Component {
    fn handle(&mut self) -> &mut ComponentHandle;
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

    // TODO - Change all this into a builder when we do the auth changes
    fn globalize_at_address_with_modules(
        self,
        preallocated_address: ComponentAddress,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress;

    fn globalize(self) -> ComponentAddress {
        self.globalize_with_modules(
            AccessRules::new(MethodAuthorities::new(), AuthorityRules::new()),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_at_address(self, preallocated_address: ComponentAddress) -> ComponentAddress {
        self.globalize_at_address_with_modules(
            preallocated_address,
            AccessRules::new(MethodAuthorities::new(), AuthorityRules::new()),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_metadata(self, metadata: Metadata) -> ComponentAddress {
        self.globalize_with_modules(
            AccessRules::new(MethodAuthorities::new(), AuthorityRules::new()),
            metadata,
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_access_rules(
        self,
        method_authorities: MethodAuthorities,
        authority_rules: AuthorityRules,
    ) -> ComponentAddress {
        self.globalize_with_modules(
            AccessRules::new(method_authorities, authority_rules),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_owner_badge(
        self,
        owner_badge: NonFungibleGlobalId,
        royalty_config: RoyaltyConfig,
    ) -> ComponentAddress {
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_rule(
            "owner".clone(),
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        );

        let access_rules = AccessRules::new(MethodAuthorities::new(), authority_rules);

        self.globalize_with_modules(access_rules, Metadata::new(), Royalty::new(royalty_config))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ComponentHandle {
    Own(Own)
}

impl ComponentHandle {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            ComponentHandle::Own(own) => own.as_node_id()
        }
    }
}

impl Component for ComponentHandle {
    fn handle(&mut self) -> &mut ComponentHandle {
        self
    }

    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        match self {
            ComponentHandle::Own(own) => {
                let output = ScryptoEnv
                    .call_method(own.as_node_id(), method, args)
                    .unwrap();
                scrypto_decode(&output).unwrap()
            }
        }
    }

    fn package_address(&self) -> PackageAddress {
        match self {
            ComponentHandle::Own(own) => {
                ScryptoEnv
                    .get_object_info(own.as_node_id())
                    .unwrap()
                    .blueprint
                    .package_address
            }
        }
    }

    fn blueprint_name(&self) -> String {
        match self {
            ComponentHandle::Own(own) => {
                ScryptoEnv
                    .get_object_info(own.as_node_id())
                    .unwrap()
                    .blueprint
                    .blueprint_name
            }
        }
    }
}

impl<T: Into<ComponentHandle>> LocalComponent for T {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress {
        let metadata: Own = metadata.to_owned();
        let access_rules: Own = access_rules.0;
        let royalty: Own = royalty.0;

        let handle = self.into();

        let address = ScryptoEnv
            .globalize(btreemap!(
                ObjectModuleId::Main => handle.as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ))
            .unwrap();

        ComponentAddress::new_or_panic(address.into())
    }

    fn globalize_at_address_with_modules(
        self,
        preallocated_address: ComponentAddress,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress {

        let handle = self.into();

        let modules: BTreeMap<ObjectModuleId, NodeId> = btreemap!(
            ObjectModuleId::Main => handle.as_node_id().clone(),
            ObjectModuleId::AccessRules => access_rules.0.0,
            ObjectModuleId::Metadata => metadata.to_owned().0,
            ObjectModuleId::Royalty => royalty.0.0,
        );

        ScryptoEnv
            .globalize_with_address(modules, preallocated_address.into())
            .unwrap();

        ComponentAddress::new_or_panic(preallocated_address.into())
    }
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Global<O>(pub O);

impl<O> Deref for Global<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: From<ComponentAddress>> From<ComponentAddress> for Global<T> {
    fn from(value: ComponentAddress) -> Self {
        let t: T = value.into();
        Global(t)
    }
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GlobalComponentRef(pub ComponentAddress);

impl GlobalComponentRef {
    pub fn access_rules(&self) -> AttachedAccessRules {
        AttachedAccessRules(self.0.into())
    }

    pub fn metadata(&self) -> Attached<Metadata> {
        Metadata::attached(self.0.into())
    }

    pub fn royalty(&self) -> AttachedRoyalty {
        AttachedRoyalty(self.0.into())
    }
}

impl Component for GlobalComponentRef {
    fn handle(&mut self) -> &mut ComponentHandle {
        todo!()
    }

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

/*
impl Categorize<ScryptoCustomValueKind> for ComponentHandle {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for ComponentHandle {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for ComponentHandle {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

// TODO: generics support for Scrypto components?
impl Describe<ScryptoCustomTypeKind> for ComponentHandle {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(OWN_ID);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        own_type_data()
    }
}
 */
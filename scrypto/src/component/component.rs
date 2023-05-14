use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ScryptoSbor};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::{
    require, AuthorityRules, MethodAuthorities, NonFungibleGlobalId,
};
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
use scrypto::modules::{Attached, Metadata};
use std::ops::Deref;

pub trait ComponentState<T: LocalComponent>: ScryptoEncode + ScryptoDecode {
    const BLUEPRINT_NAME: &'static str;

    fn instantiate(self) -> T {
        let node_id = ScryptoEnv
            .new_simple_object(Self::BLUEPRINT_NAME, vec![scrypto_encode(&self).unwrap()])
            .unwrap();

        T::new(ComponentHandle::Own(Own(node_id)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ComponentHandle {
    Own(Own),
    Global(GlobalAddress),
}

impl ComponentHandle {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            ComponentHandle::Own(own) => own.as_node_id(),
            ComponentHandle::Global(address) => address.as_node_id(),
        }
    }
}

// TODO: I've temporarily disabled &mut requirement on the Component trait.
// If not, I will have to overhaul the `ComponentSystem` infra, which will be likely be removed anyway.
//
// Since there is no mutability semantics in the system and kernel, there is no technical benefits
// with &mut, other than to frustrate developers.

pub trait Component {
    fn new(handle: ComponentHandle) -> Self;

    fn handle(&self) -> &ComponentHandle;

    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.handle().as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn package_address(&self) -> PackageAddress {
        ScryptoEnv
            .get_object_info(self.handle().as_node_id())
            .unwrap()
            .blueprint
            .package_address
    }

    fn blueprint_name(&self) -> String {
        ScryptoEnv
            .get_object_info(self.handle().as_node_id())
            .unwrap()
            .blueprint
            .blueprint_name
    }
}

pub struct AnyComponent(ComponentHandle);

impl Component for AnyComponent {
    fn new(handle: ComponentHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ComponentHandle {
        &self.0
    }
}

pub trait LocalComponent: Component + Sized {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self>;

    // TODO - Change all this into a builder when we do the auth changes
    fn globalize_at_address_with_modules(
        self,
        preallocated_address: ComponentAddress,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self>;

    fn globalize(self) -> Global<Self> {
        self.globalize_with_modules(
            AccessRules::new(MethodAuthorities::new(), AuthorityRules::new()),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_at_address(self, preallocated_address: ComponentAddress) -> Global<Self> {
        self.globalize_at_address_with_modules(
            preallocated_address,
            AccessRules::new(MethodAuthorities::new(), AuthorityRules::new()),
            Metadata::new(),
            Royalty::new(RoyaltyConfig::default()),
        )
    }

    fn globalize_with_metadata(self, metadata: Metadata) -> Global<Self> {
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
    ) -> Global<Self> {
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
    ) -> Global<Self> {
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

impl<T: Component> LocalComponent for T {
    fn globalize_with_modules(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self> {
        let address = ScryptoEnv
            .globalize(btreemap!(
                ObjectModuleId::Main => self.handle().as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.handle().as_node_id().clone(),
                ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
                ObjectModuleId::Royalty => royalty.handle().as_node_id().clone(),
            ))
            .unwrap();

        Global(Self::new(ComponentHandle::Global(address)))
    }

    fn globalize_at_address_with_modules(
        self,
        preallocated_address: ComponentAddress,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self> {
        let modules: BTreeMap<ObjectModuleId, NodeId> = btreemap!(
            ObjectModuleId::Main => self.handle().as_node_id().clone(),
            ObjectModuleId::AccessRules => access_rules.handle().as_node_id().clone(),
            ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
            ObjectModuleId::Royalty => royalty.handle().as_node_id().clone(),
        );

        ScryptoEnv
            .globalize_with_address(modules, preallocated_address.into())
            .unwrap();

        Global(Self::new(ComponentHandle::Global(
            preallocated_address.into(),
        )))
    }
}

#[derive(Debug, Copy, PartialEq, Eq, Hash)]
pub struct Global<O: Component>(pub O);

impl<O: Component> Clone for Global<O> {
    fn clone(&self) -> Self {
        Global(O::new(self.0.handle().clone()))
    }
}

impl<O: Component> Deref for Global<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<O: Component> Global<O> {
    pub fn component_address(&self) -> ComponentAddress {
        ComponentAddress::new_or_panic(self.handle().as_node_id().0)
    }

    pub fn metadata(&self) -> Attached<Metadata> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let metadata = Metadata::attached(address);
        Attached(metadata, PhantomData::default())
    }

    pub fn access_rules(&self) -> Attached<AccessRules> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let access_rules = AccessRules::attached(address);
        Attached(access_rules, PhantomData::default())
    }

    pub fn royalty(&self) -> Attached<Royalty> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let royalty = Royalty::attached(address);
        Attached(royalty, PhantomData::default())
    }
}

impl<O: Component> From<ComponentAddress> for Global<O> {
    fn from(value: ComponentAddress) -> Self {
        Global(Component::new(ComponentHandle::Global(value.into())))
    }
}

impl<O: Component> Categorize<ScryptoCustomValueKind> for Global<O> {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Reference)
    }
}

impl<O: Component, E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for Global<O>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self.0.handle() {
            ComponentHandle::Global(address) => encoder.write_slice(&address.to_vec()),
            _ => panic!("Unexpected"),
        }
    }
}

impl<O: Component, D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Global<O>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Reference::decode_body_with_value_kind(decoder, value_kind).map(|reference| {
            let o = O::new(ComponentHandle::Global(GlobalAddress::new_or_panic(
                reference.as_node_id().0,
            )));
            Self(o)
        })
    }
}

// TODO: generics support for Scrypto components?
impl<O: Component> Describe<ScryptoCustomTypeKind> for Global<O> {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(REFERENCE_ID);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        reference_type_data()
    }
}

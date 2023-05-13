use std::ops::Deref;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, AttachedAccessRules, AttachedRoyalty, ModuleHandle, Royalty};
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
use crate::prelude::{scrypto_encode, ScryptoSbor};

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
            ComponentHandle::Global(address) => address.as_node_id()
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

pub trait LocalComponent: Component + Sized {
    fn globalize2(
        self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self>;

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


impl<T: Component> LocalComponent for T {
    fn globalize2(
        mut self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> Global<Self> {
        let access_rules: Own = access_rules.0;
        let royalty: Own = royalty.0;

        let address = ScryptoEnv
            .globalize(btreemap!(
                ObjectModuleId::Main => self.handle().as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
                ObjectModuleId::Royalty => royalty.0,
            ))
            .unwrap();

        Global(T::new(ComponentHandle::Global(address)))
    }

    fn globalize_with_modules(
        mut self,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress {
        let access_rules: Own = access_rules.0;
        let royalty: Own = royalty.0;

        let address = ScryptoEnv
            .globalize(btreemap!(
                ObjectModuleId::Main => self.handle().as_node_id().clone(),
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
                ObjectModuleId::Royalty => royalty.0,
            ))
            .unwrap();

        ComponentAddress::new_or_panic(address.into())
    }

    fn globalize_at_address_with_modules(
        mut self,
        preallocated_address: ComponentAddress,
        access_rules: AccessRules,
        metadata: Metadata,
        royalty: Royalty,
    ) -> ComponentAddress {
        let modules: BTreeMap<ObjectModuleId, NodeId> = btreemap!(
            ObjectModuleId::Main => self.handle().as_node_id().clone(),
            ObjectModuleId::AccessRules => access_rules.0.0,
            ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
            ObjectModuleId::Royalty => royalty.0.0,
        );

        ScryptoEnv
            .globalize_with_address(modules, preallocated_address.into())
            .unwrap();

        ComponentAddress::new_or_panic(preallocated_address.into())
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub struct Global<O: Component>(pub O);

impl<O: Component> Deref for Global<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<O: Component> Global<O> {
    pub fn metadata(&self) -> Attached<Metadata> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let metadata = Metadata::attached(address);
        Attached(metadata, PhantomData::default())
    }
}

impl<O: Component> From<ComponentAddress> for Global<O> {
    fn from(value: ComponentAddress) -> Self {
        Global(Component::new(ComponentHandle::Global(value.into())))
    }
}

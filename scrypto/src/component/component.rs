use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ObjectStub, ObjectStubHandle};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::{MethodKey, RoleList, Roles};
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::own_type_data;
use radix_engine_interface::data::scrypto::{
    ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::types::*;
use sbor::rust::ops::Deref;
use sbor::rust::prelude::*;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Describe, Encode, EncodeError, Encoder, GlobalTypeId,
    ValueKind,
};
use scrypto::modules::{Attached, Metadata};
use scrypto::prelude::well_known_scrypto_custom_types::OWN_ID;

pub struct Blueprint<C>(PhantomData<C>);

pub trait HasStub {
    type Stub: ObjectStub;
}

pub trait HasMethods {
    type Permissions: MethodMapping<MethodPermission>;
    type Royalties: MethodMapping<MethodRoyalty>;
}

pub trait ComponentState: HasMethods + HasStub + ScryptoEncode + ScryptoDecode {
    const BLUEPRINT_NAME: &'static str;

    fn instantiate(self) -> Owned<Self> {
        let node_id = ScryptoEnv
            .new_simple_object(Self::BLUEPRINT_NAME, vec![scrypto_encode(&self).unwrap()])
            .unwrap();

        let stub = Self::Stub::new(ObjectStubHandle::Own(Own(node_id)));
        Owned(stub)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyComponent(ObjectStubHandle);

impl HasStub for AnyComponent {
    type Stub = Self;
}

impl ObjectStub for AnyComponent {
    fn new(handle: ObjectStubHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Owned<C: HasStub>(pub C::Stub);

impl<C: HasStub> Deref for Owned<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: HasStub> Categorize<ScryptoCustomValueKind> for Owned<C> {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<C: HasStub, E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for Owned<C>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self.0.handle() {
            ObjectStubHandle::Own(own) => encoder.write_slice(&own.to_vec()),
            _ => panic!("Unexpected"),
        }
    }
}

impl<C: HasStub, D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Owned<C>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|own| {
            let o = C::Stub::new(ObjectStubHandle::Own(own));
            Self(o)
        })
    }
}

// TODO: generics support for Scrypto components?
impl<C: HasStub> Describe<ScryptoCustomTypeKind> for Owned<C> {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(OWN_ID);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        own_type_data()
    }
}

impl<C: HasStub> Owned<C> {
    pub fn prepare_to_globalize(self) -> Globalizing<C> {
        Globalizing {
            stub: self.0,
            metadata: None,
            royalty: RoyaltyConfig::default(),

            authority_rules: Roles::new(),
            protected_module_methods: BTreeMap::new(),

            address: None,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum RoyaltyMethod {
    set_royalty_config,
    claim_royalty,
}

impl ModuleMethod for RoyaltyMethod {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Royalty;

    fn to_ident(&self) -> String {
        match self {
            RoyaltyMethod::claim_royalty => COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
            RoyaltyMethod::set_royalty_config => {
                COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string()
            }
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum MetadataMethod {
    set,
}

impl ModuleMethod for MetadataMethod {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Metadata;

    fn to_ident(&self) -> String {
        match self {
            MetadataMethod::set => METADATA_SET_IDENT.to_string(),
        }
    }
}

pub trait ModuleMethod {
    const MODULE_ID: ObjectModuleId;

    fn to_ident(&self) -> String;
}

pub enum MethodPermission {
    Public,
    Protected(RoleList),
}

impl<const N: usize> From<[&str; N]> for MethodPermission {
    fn from(value: [&str; N]) -> Self {
        MethodPermission::Protected(value.into())
    }
}

pub enum MethodRoyalty {
    Free,
    Charge(u32),
}

impl From<u32> for MethodRoyalty {
    fn from(value: u32) -> Self {
        Self::Charge(value)
    }
}

/*
pub enum MethodPermissionMutability {
    Locked,
    Mutable(RoleList),
}
 */

pub trait MethodMapping<T> {
    const MODULE_ID: ObjectModuleId;

    fn to_mapping(self) -> Vec<(String, T)>;
}

pub struct RoyaltyMethods<T> {
    pub set_royalty_config: T,
    pub claim_royalty: T,
}

impl<T> MethodMapping<T> for RoyaltyMethods<T> {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Royalty;

    fn to_mapping(self) -> Vec<(String, T)> {
        vec![
            (COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(), self.set_royalty_config),
            (COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(), self.claim_royalty),
        ]
    }
}

pub struct RoyaltiesConfig<R: MethodMapping<MethodRoyalty>> {
    pub method_royalties: R,
    pub permissions: RoyaltyMethods<MethodPermission>,
}

pub struct MetadataMethods<T> {
    pub set: T,
    pub get: T,
    pub remove: T,
}

impl<T> MethodMapping<T> for MetadataMethods<T> {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Metadata;

    fn to_mapping(self) -> Vec<(String, T)> {
        vec![
            (METADATA_SET_IDENT.to_string(), self.set),
            (METADATA_GET_IDENT.to_string(), self.get),
            (METADATA_REMOVE_IDENT.to_string(), self.remove),
        ]
    }
}

pub struct MetadataInit {
    pub metadata: Metadata,
    pub permissions: MetadataMethods<MethodPermission>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Globalizing<C: HasStub> {
    pub stub: C::Stub,
    pub metadata: Option<Metadata>,
    pub royalty: RoyaltyConfig,
    pub authority_rules: Roles,
    pub protected_module_methods: BTreeMap<MethodKey, RoleList>,
    pub address: Option<ComponentAddress>,
}

impl<C: HasStub> Deref for Globalizing<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<C: HasStub + HasMethods> Globalizing<C> {
    pub fn define_roles(mut self, authority_rules: Roles) -> Self {
        self.authority_rules = authority_rules;
        self
    }

    pub fn methods(mut self, permissions: C::Permissions) -> Self {
        self.set_permissions(permissions);
        self
    }

    pub fn metadata(mut self, init: MetadataInit) -> Self {
        if self.metadata.is_some() {
            panic!("Metadata already set.");
        }
        self.metadata = Some(init.metadata);
        self.set_permissions(init.permissions);

        self
    }

    pub fn royalties(mut self, royalties: RoyaltiesConfig<C::Royalties>) -> Self {
        for (method, royalty) in royalties.method_royalties.to_mapping() {
            match royalty {
                MethodRoyalty::Free => {}
                MethodRoyalty::Charge(amount) => self.royalty.set_rule(method, amount),
            }
        }

        self.set_permissions(royalties.permissions);

        self
    }

    pub fn with_address(mut self, address: ComponentAddress) -> Self {
        self.address = Some(address);
        self
    }

    fn set_permissions<T: MethodMapping<MethodPermission>>(&mut self, permissions: T) {
        for (method, permissions) in permissions.to_mapping() {
            match permissions {
                MethodPermission::Public => {}
                MethodPermission::Protected(role_list) => {
                    self.protected_module_methods
                        .insert(MethodKey::new(T::MODULE_ID, method), role_list);
                }
            }
        }
    }

    pub fn globalize(mut self) -> Global<C> {
        let metadata = self.metadata.take().unwrap_or_else(|| Metadata::default());
        let royalty = Royalty::new(self.royalty);
        let access_rules = AccessRules::new(self.protected_module_methods, self.authority_rules);

        let modules = btreemap!(
            ObjectModuleId::Main => self.stub.handle().as_node_id().clone(),
            ObjectModuleId::AccessRules => access_rules.handle().as_node_id().clone(),
            ObjectModuleId::Metadata => metadata.handle().as_node_id().clone(),
            ObjectModuleId::Royalty => royalty.handle().as_node_id().clone(),
        );

        let address = if let Some(address) = self.address {
            let address: GlobalAddress = address.into();
            ScryptoEnv.globalize_with_address(modules, address).unwrap();
            address
        } else {
            ScryptoEnv.globalize(modules).unwrap()
        };

        Global(C::Stub::new(ObjectStubHandle::Global(address)))
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Global<O: HasStub>(pub O::Stub);

impl<O: HasStub> Copy for Global<O> {}

impl<O: HasStub> Clone for Global<O> {
    fn clone(&self) -> Self {
        Global(O::Stub::new(self.0.handle().clone()))
    }
}

impl<O: HasStub> Deref for Global<O> {
    type Target = O::Stub;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<O: HasStub> Global<O> {
    // TODO: Change to GlobalAddress?
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

impl<O: HasStub> From<ComponentAddress> for Global<O> {
    fn from(value: ComponentAddress) -> Self {
        Global(ObjectStub::new(ObjectStubHandle::Global(value.into())))
    }
}

impl<O: HasStub> From<PackageAddress> for Global<O> {
    fn from(value: PackageAddress) -> Self {
        Global(ObjectStub::new(ObjectStubHandle::Global(value.into())))
    }
}

impl<O: HasStub> Categorize<ScryptoCustomValueKind> for Global<O> {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Reference)
    }
}

impl<O: HasStub, E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E>
    for Global<O>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self.0.handle() {
            ObjectStubHandle::Global(address) => encoder.write_slice(&address.to_vec()),
            _ => panic!("Unexpected"),
        }
    }
}

impl<O: HasStub, D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Global<O>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Reference::decode_body_with_value_kind(decoder, value_kind).map(|reference| {
            let o = O::Stub::new(ObjectStubHandle::Global(GlobalAddress::new_or_panic(
                reference.as_node_id().0,
            )));
            Self(o)
        })
    }
}

// TODO: generics support for Scrypto components?
impl<O: HasStub> Describe<ScryptoCustomTypeKind> for Global<O> {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(REFERENCE_ID);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        reference_type_data()
    }
}

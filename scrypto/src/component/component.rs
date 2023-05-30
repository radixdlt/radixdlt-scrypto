use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::{scrypto_encode, ObjectStub, ObjectStubHandle};
use crate::runtime::*;
use crate::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::well_known_scrypto_custom_types::{
    component_address_type_data, COMPONENT_ADDRESS_ID,
};
use radix_engine_common::prelude::{
    OwnValidation, ReferenceValidation, ScryptoCustomTypeValidation,
};
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::{
    MethodPermission, OwnerRole, Roles, OWNER_ROLE,
};
use radix_engine_interface::data::scrypto::{
    ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::types::*;
use sbor::rust::ops::Deref;
use sbor::rust::prelude::*;
use sbor::*;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Describe, Encode, EncodeError, Encoder, GlobalTypeId,
    ValueKind,
};
use scrypto::modules::{Attached, Metadata};

pub trait HasTypeInfo {
    const PACKAGE_ADDRESS: Option<PackageAddress>;
    const BLUEPRINT_NAME: &'static str;
    const OWNED_TYPE_NAME: &'static str;
    const GLOBAL_TYPE_NAME: &'static str;
}

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

impl<T: HasTypeInfo + HasStub> Describe<ScryptoCustomTypeKind> for Owned<T> {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::Novel(const_sha1::sha1(T::OWNED_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names(T::OWNED_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(T::PACKAGE_ADDRESS, T::BLUEPRINT_NAME.to_string()),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

impl<C: HasStub + HasMethods> Owned<C> {
    pub fn prepare_to_globalize(self, owner_entry: OwnerRole) -> Globalizing<C> {
        let mut roles = Roles::new();
        roles.define_role(OWNER_ROLE, owner_entry.to_role_entry(OWNER_ROLE));

        Globalizing {
            stub: self.0,
            metadata: None,
            royalty: RoyaltyConfig::default(),
            roles,
            address: None,
        }
    }
}

pub enum MethodRoyalty {
    Free,
    Xrd(Decimal),
    Usd(Decimal),
}

pub trait MethodMapping<T> {
    const MODULE_ID: ObjectModuleId;

    fn to_mapping(self) -> Vec<(String, T)>;
    fn methods() -> Vec<&'static str>;
}

pub struct RoyaltyMethods<T> {
    pub set_royalty_config: T,
    pub claim_royalty: T,
}

impl<T> MethodMapping<T> for RoyaltyMethods<T> {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Royalty;

    fn to_mapping(self) -> Vec<(String, T)> {
        vec![
            (
                COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
                self.set_royalty_config,
            ),
            (
                COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
                self.claim_royalty,
            ),
        ]
    }

    fn methods() -> Vec<&'static str> {
        vec![
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
        ]
    }
}

pub struct RoyaltiesConfig<R: MethodMapping<MethodRoyalty>> {
    pub method_royalties: R,
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

    fn methods() -> Vec<&'static str> {
        vec![
            METADATA_SET_IDENT,
            METADATA_GET_IDENT,
            METADATA_REMOVE_IDENT,
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Globalizing<C: HasStub> {
    pub stub: C::Stub,
    pub metadata: Option<Metadata>,
    pub royalty: RoyaltyConfig,
    pub roles: Roles,
    pub address: Option<ComponentAddress>,
}

impl<C: HasStub> Deref for Globalizing<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<C: HasStub + HasMethods> Globalizing<C> {
    pub fn roles(mut self, roles: Roles) -> Self {
        self.roles = roles;
        self
    }

    pub fn metadata(mut self, metadata: Metadata) -> Self {
        if self.metadata.is_some() {
            panic!("Metadata already set.");
        }
        self.metadata = Some(metadata);

        self
    }

    pub fn royalties(mut self, royalties: C::Royalties) -> Self {
        for (method, royalty) in royalties.to_mapping() {
            match royalty {
                MethodRoyalty::Xrd(x) => self.royalty.set_rule(method, RoyaltyAmount::Xrd(x)),
                MethodRoyalty::Usd(x) => self.royalty.set_rule(method, RoyaltyAmount::Usd(x)),
                MethodRoyalty::Free => {}
            }
        }

        self
    }

    pub fn with_address(mut self, address: ComponentAddress) -> Self {
        self.address = Some(address);
        self
    }

    pub fn globalize(mut self) -> Global<C> {
        let metadata = self.metadata.take().unwrap_or_else(|| Metadata::default());
        let royalty = Royalty::new(self.royalty);

        let access_rules = AccessRules::new(self.roles);

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

impl<T: HasTypeInfo + HasStub> Describe<ScryptoCustomTypeKind> for Global<T> {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::Novel(const_sha1::sha1(T::GLOBAL_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names(T::GLOBAL_TYPE_NAME),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(
                    T::PACKAGE_ADDRESS,
                    T::BLUEPRINT_NAME.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

impl Describe<ScryptoCustomTypeKind> for Global<AnyComponent> {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::WellKnown([COMPONENT_ADDRESS_ID]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        component_address_type_data()
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::modules::{Attachable, HasMetadata, RoleAssignment, Royalty};
use crate::prelude::{scrypto_encode, HasRoleAssignment, ObjectStub, ObjectStubHandle};
use crate::runtime::*;
use crate::*;
use radix_common::data::scrypto::{
    ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_common::prelude::well_known_scrypto_custom_types::{
    component_address_type_data, own_type_data, COMPONENT_ADDRESS_TYPE, OWN_TYPE,
};
use radix_common::prelude::{
    scrypto_decode, OwnValidation, ReferenceValidation, ScryptoCustomTypeValidation,
};
use radix_common::ScryptoSbor;
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::{AttachedModuleId, FieldValue};
use radix_engine_interface::blueprints::resource::{
    AccessRule, FungibleBucket, MethodAccessibility, OwnerRole, OwnerRoleEntry, RoleAssignmentInit,
};
use radix_engine_interface::object_modules::metadata::{
    MetadataConversionError, MetadataInit, MetadataVal, METADATA_GET_IDENT, METADATA_REMOVE_IDENT,
    METADATA_SET_IDENT,
};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::types::*;
use sbor::rust::ops::Deref;
use sbor::rust::ops::DerefMut;
use sbor::rust::prelude::*;
use sbor::*;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Describe, Encode, EncodeError, Encoder, RustTypeId,
    ValueKind,
};
use scrypto::modules::{Attached, HasComponentRoyalties, Metadata};

pub trait HasTypeInfo {
    const PACKAGE_ADDRESS: Option<PackageAddress>;
    const BLUEPRINT_NAME: &'static str;
    const OWNED_TYPE_NAME: &'static str;
    const GLOBAL_TYPE_NAME: &'static str;

    fn blueprint_id() -> BlueprintId {
        let package_address = Self::PACKAGE_ADDRESS.unwrap_or(Runtime::package_address());
        BlueprintId::new(&package_address, Self::BLUEPRINT_NAME)
    }
}

pub struct Blueprint<C: HasTypeInfo>(PhantomData<C>);

impl<C: HasTypeInfo> Blueprint<C> {
    pub fn call_function<A: ScryptoEncode, T: ScryptoDecode>(function_name: &str, args: A) -> T {
        let package_address = C::PACKAGE_ADDRESS.unwrap_or(Runtime::package_address());

        let output = ScryptoVmV1Api::blueprint_call(
            package_address,
            C::BLUEPRINT_NAME,
            function_name,
            scrypto_encode(&args).unwrap(),
        );
        scrypto_decode(&output).unwrap()
    }

    pub fn call_function_raw<T: ScryptoDecode>(function_name: &str, args: Vec<u8>) -> T {
        let package_address = C::PACKAGE_ADDRESS.unwrap_or(Runtime::package_address());
        let output =
            ScryptoVmV1Api::blueprint_call(package_address, C::BLUEPRINT_NAME, function_name, args);
        scrypto_decode(&output).unwrap()
    }
}

pub trait HasStub {
    type Stub: ObjectStub;
}

pub trait HasMethods {
    type Permissions: MethodMapping<MethodAccessibility>;
    type Royalties: MethodMapping<(RoyaltyAmount, bool)>;
}

pub trait ComponentState: HasMethods + HasStub + ScryptoEncode + ScryptoDecode {
    const BLUEPRINT_NAME: &'static str;

    fn instantiate(self) -> Owned<Self> {
        let node_id = ScryptoVmV1Api::object_new(
            Self::BLUEPRINT_NAME,
            indexmap![0u8 => FieldValue::new(&self)],
        );

        let stub = Self::Stub::new(ObjectStubHandle::Own(Own(node_id)));
        Owned(stub)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyComponent(pub(crate) ObjectStubHandle);

impl HasStub for AnyComponent {
    type Stub = Self;
}

impl ObjectStub for AnyComponent {
    type AddressType = ComponentAddress;

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

impl<C: HasStub> DerefMut for Owned<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(T::OWNED_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
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
    pub fn prepare_to_globalize(self, owner_role: OwnerRole) -> Globalizing<C> {
        Globalizing {
            stub: self.0,
            owner_role,
            metadata_config: None,
            royalty_config: None,
            roles: RoleAssignmentInit::new(),
            address_reservation: None,
        }
    }
}

pub trait FnMapping<T> {
    fn to_mapping(self) -> Vec<(String, T)>;
}

pub trait MethodMapping<T> {
    const MODULE_ID: ModuleId;

    fn to_mapping(self) -> Vec<(String, T)>;
    fn methods() -> Vec<&'static str>;
}

pub struct MetadataMethods<T> {
    pub set: T,
    pub get: T,
    pub remove: T,
}

impl<T> MethodMapping<T> for MetadataMethods<T> {
    const MODULE_ID: ModuleId = ModuleId::Metadata;

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

    pub owner_role: OwnerRole,
    pub metadata_config: Option<ModuleConfig<MetadataInit>>,
    pub royalty_config: Option<ModuleConfig<ComponentRoyaltyConfig>>,
    pub address_reservation: Option<GlobalAddressReservation>,

    pub roles: RoleAssignmentInit,
}

impl<C: HasStub> Deref for Globalizing<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<C: HasStub + HasMethods> Globalizing<C> {
    pub fn roles(mut self, roles: RoleAssignmentInit) -> Self {
        self.roles = roles;
        self
    }

    pub fn metadata(mut self, metadata_config: ModuleConfig<MetadataInit>) -> Self {
        self.metadata_config = Some(metadata_config);

        self
    }

    pub fn enable_component_royalties(
        mut self,
        royalties: (C::Royalties, RoleAssignmentInit),
    ) -> Self {
        let mut royalty_amounts = index_map_new();
        for (method, (royalty, updatable)) in royalties.0.to_mapping() {
            royalty_amounts.insert(method, (royalty, !updatable));
        }

        let royalty_config = ModuleConfig {
            init: ComponentRoyaltyConfig { royalty_amounts },
            roles: royalties.1,
        };

        self.royalty_config = Some(royalty_config);

        self
    }

    pub fn with_address(mut self, address_reservation: GlobalAddressReservation) -> Self {
        self.address_reservation = Some(address_reservation);
        self
    }

    pub fn globalize(mut self) -> Global<C> {
        let mut modules = index_map_new();
        let mut roles = index_map_new();

        // Main
        {
            roles.insert(ModuleId::Main, self.roles);
        }

        // Metadata
        {
            let metadata_config = self
                .metadata_config
                .take()
                .unwrap_or_else(|| Default::default());

            let metadata = Metadata::new_with_data(metadata_config.init);
            modules.insert(
                AttachedModuleId::Metadata,
                metadata.handle().as_node_id().clone(),
            );
            roles.insert(ModuleId::Metadata, metadata_config.roles);
        };

        // Royalties
        if let Some(royalty_config) = self.royalty_config {
            roles.insert(ModuleId::Royalty, royalty_config.roles);
            let royalty = Royalty::new(royalty_config.init);
            modules.insert(
                AttachedModuleId::Royalty,
                royalty.handle().as_node_id().clone(),
            );
        }

        // Role Assignment
        {
            let role_assignment = RoleAssignment::new(self.owner_role, roles);
            modules.insert(
                AttachedModuleId::RoleAssignment,
                role_assignment.handle().as_node_id().clone(),
            );
        }

        let address = ScryptoVmV1Api::object_globalize(
            self.stub.handle().as_node_id().clone(),
            modules,
            self.address_reservation,
        );

        Global(C::Stub::new(ObjectStubHandle::Global(address)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Global<O: HasStub>(pub O::Stub);

impl<O: HasStub> core::hash::Hash for Global<O> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.handle().as_node_id().hash(state);
    }
}

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

impl<O: HasStub> DerefMut for Global<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<O: HasStub> Global<O> {
    pub fn address(&self) -> <<O as HasStub>::Stub as ObjectStub>::AddressType {
        let rtn = <<O as HasStub>::Stub as ObjectStub>::AddressType::try_from(
            self.handle().as_node_id().0,
        );
        match rtn {
            Ok(address) => address,
            Err(..) => panic!("Invalid address type"),
        }
    }

    fn metadata(&self) -> Attached<Metadata> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let metadata = Metadata::attached(address);
        Attached(metadata, PhantomData::default())
    }

    fn role_assignment(&self) -> Attached<RoleAssignment> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let role_assignment = RoleAssignment::attached(address);
        Attached(role_assignment, PhantomData::default())
    }
}

impl<O, S> Global<O>
where
    O: HasStub<Stub = S>,
    S: ObjectStub<AddressType = ComponentAddress>,
{
    fn component_royalties(&self) -> Attached<Royalty> {
        let address = GlobalAddress::new_or_panic(self.handle().as_node_id().0);
        let royalty = Royalty::attached(address);
        Attached(royalty, PhantomData::default())
    }
}

impl<O: HasStub> HasMetadata for Global<O> {
    fn set_metadata<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V) {
        self.metadata().set(name, value);
    }

    fn get_metadata<K: ToString, V: MetadataVal>(
        &self,
        name: K,
    ) -> Result<Option<V>, MetadataConversionError> {
        self.metadata().get(name)
    }

    fn remove_metadata<K: ToString>(&self, name: K) -> bool {
        self.metadata().remove(name)
    }

    fn lock_metadata<K: ToString>(&self, name: K) {
        self.metadata().lock(name)
    }
}

impl<O: HasStub> HasRoleAssignment for Global<O> {
    fn set_owner_role<A: Into<AccessRule>>(&self, rule: A) {
        self.role_assignment().set_owner_role(rule)
    }

    fn get_owner_role(&self) -> OwnerRoleEntry {
        self.role_assignment().get_owner_role()
    }

    fn lock_owner_role(&self) {
        self.role_assignment().lock_owner_role()
    }

    fn set_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.role_assignment().set_role(name, rule);
    }

    fn get_role(&self, name: &str) -> Option<AccessRule> {
        self.role_assignment().get_role(name)
    }

    fn set_metadata_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.role_assignment().set_metadata_role(name, rule);
    }

    fn set_component_royalties_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.role_assignment()
            .set_component_royalties_role(name, rule);
    }
}

impl<O, S> HasComponentRoyalties for Global<O>
where
    O: HasStub<Stub = S>,
    S: ObjectStub<AddressType = ComponentAddress>,
{
    fn set_royalty<M: ToString>(&self, method: M, amount: RoyaltyAmount) {
        self.component_royalties().set_royalty(method, amount);
    }

    fn lock_royalty<M: ToString>(&self, method: M) {
        self.component_royalties().lock_royalty(method);
    }

    fn claim_component_royalties(&self) -> FungibleBucket {
        self.component_royalties().claim_royalties()
    }
}

trait TypeCheckable {
    fn check(node_id: &NodeId) -> Result<(), ComponentCastError>;
}

impl<O: HasTypeInfo> TypeCheckable for O {
    fn check(node_id: &NodeId) -> Result<(), ComponentCastError> {
        if !ScryptoVmV1Api::object_instance_of(node_id, &O::blueprint_id()) {
            return Err(ComponentCastError::InvalidCast {
                node_id: *node_id,
                blueprint_id: O::blueprint_id(),
            });
        }

        Ok(())
    }
}

impl TypeCheckable for AnyComponent {
    fn check(_node_id: &NodeId) -> Result<(), ComponentCastError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum ComponentCastError {
    InvalidCast {
        node_id: NodeId,
        blueprint_id: BlueprintId,
    },
}

impl<O: HasStub + TypeCheckable> From<ComponentAddress> for Global<O> {
    fn from(value: ComponentAddress) -> Self {
        O::check(value.as_node_id()).unwrap();
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
    const TYPE_ID: RustTypeId =
        RustTypeId::Novel(const_sha1::sha1(T::GLOBAL_TYPE_NAME.as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
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
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(COMPONENT_ADDRESS_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        component_address_type_data()
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

impl Describe<ScryptoCustomTypeKind> for Owned<AnyComponent> {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(OWN_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        own_type_data()
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

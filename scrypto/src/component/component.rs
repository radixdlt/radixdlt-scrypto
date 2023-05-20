use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ObjectStub, ObjectStubHandle};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::{METADATA_SET_IDENT, MetadataVal};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::api::node_modules::royalty::{COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT};
use radix_engine_interface::blueprints::resource::{AccessRule, AuthorityKey, AuthorityRules, MethodKey};
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

pub trait ComponentState: HasStub + ScryptoEncode + ScryptoDecode {
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

            authority_rules: AuthorityRules::new(),
            protected_module_methods: BTreeMap::new(),

            address: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum RoyaltyMethod {
    SetRoyaltyConfig,
    ClaimRoyalty,
}

impl ToString for RoyaltyMethod {
    fn to_string(&self) -> String {
        match self {
            RoyaltyMethod::ClaimRoyalty => COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
            RoyaltyMethod::SetRoyaltyConfig => COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Globalizing<C: HasStub> {
    pub stub: C::Stub,
    pub metadata: Option<Metadata>,
    pub royalty: RoyaltyConfig,

    pub authority_rules: AuthorityRules,
    pub protected_module_methods: BTreeMap<MethodKey, Vec<String>>,

    pub address: Option<ComponentAddress>,
}

impl<C: HasStub> Deref for Globalizing<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<C: HasStub> Globalizing<C> {
    pub fn metadata<K: AsRef<str>, V: MetadataVal>(mut self, name: K, value: V) -> Self {
        let metadata = self.metadata.get_or_insert(Metadata::new());
        metadata.set(name, value);
        self
    }

    pub fn royalty(mut self, method: &str, amount: u32) -> Self {
        self.royalty.set_rule(method, amount);
        self
    }

    pub fn royalty_default(mut self, amount: u32) -> Self {
        self.royalty.default_rule = amount;
        self
    }

    pub fn define_roles(mut self, authority_rules: AuthorityRules) -> Self {
        self.authority_rules = authority_rules;
        self
    }

    pub fn protect_royalty(mut self, protected_royalty_methods: BTreeMap<RoyaltyMethod, Vec<String>>) -> Self {
        for (protected_royalty_method, authorities) in protected_royalty_methods {
            self.protected_module_methods.insert(
                MethodKey::new(ObjectModuleId::Royalty, protected_royalty_method),
                authorities
            );
        }

        self
    }

    pub fn protect_metadata_set<S: Into<String>>(
        mut self,
        authority: S,
    ) -> Self {
        self.protected_module_methods.insert(
            MethodKey::new(ObjectModuleId::Metadata, METADATA_SET_IDENT),
            vec![authority.into()],
        );
        self
    }

    pub fn owner_authority<A: Into<AccessRule>, B: Into<AccessRule>>(
        mut self,
        entry: A,
        mutability: B,
    ) -> Self {
        self.authority_rules
            .set_owner_authority(entry.into(), mutability.into());
        self
    }

    pub fn globalize_at_address(mut self, address: ComponentAddress) -> Global<C> {
        let _ = self.address.insert(address);
        self.globalize()
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

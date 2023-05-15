use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ObjectStub, ObjectStubHandle};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
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

pub trait ComponentState: HasStub + ScryptoEncode + ScryptoDecode {
    const BLUEPRINT_NAME: &'static str;

    fn instantiate(self) -> Globalizeable<Self> {
        let node_id = ScryptoEnv
            .new_simple_object(Self::BLUEPRINT_NAME, vec![scrypto_encode(&self).unwrap()])
            .unwrap();

        let stub = Self::Stub::new(ObjectStubHandle::Own(Own(node_id)));
        Globalizeable::new(stub)
    }
}

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
pub struct Globalizeable<C: HasStub> {
    pub stub: C::Stub,
    pub metadata: Option<Metadata>,
    pub royalty: Option<Royalty>,
    pub access_rules: Option<AccessRules>,
    pub address: Option<ComponentAddress>,
}

impl<C: HasStub> Deref for Globalizeable<C> {
    type Target = C::Stub;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<C: HasStub> Globalizeable<C> {
    fn new(stub: C::Stub) -> Self {
        Self {
            stub,
            metadata: None,
            royalty: None,
            access_rules: None,
            address: None,
        }
    }

    pub fn own(self) -> C::Stub {
        if self.metadata.is_some() || self.royalty.is_some() || self.access_rules.is_some() {
            panic!("Cannot own with already attached objects.");
        }
        self.stub
    }

    fn handle(&self) -> &ObjectStubHandle {
        self.stub.handle()
    }

    pub fn attach_metadata(mut self, metadata: Metadata) -> Self {
        let _ = self.metadata.insert(metadata);
        self
    }

    pub fn attach_royalty(mut self, royalty: Royalty) -> Self {
        let _ = self.royalty.insert(royalty);
        self
    }

    pub fn attach_access_rules(mut self, access_rules: AccessRules) -> Self {
        let _ = self.access_rules.insert(access_rules);
        self
    }

    pub fn attach_address(mut self, address: ComponentAddress) -> Self {
        let _ = self.address.insert(address);
        self
    }

    pub fn globalize(mut self) -> Global<C> {
        let metadata = self.metadata.take().unwrap_or_else(|| Metadata::default());
        let royalty = self.royalty.take().unwrap_or_else(|| Royalty::default());
        let access_rules = self
            .access_rules
            .take()
            .unwrap_or_else(|| AccessRules::default());

        let modules = btreemap!(
            ObjectModuleId::Main => self.handle().as_node_id().clone(),
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

#[derive(Debug, Copy, PartialEq, Eq, Hash)]
pub struct Global<O: HasStub>(pub O::Stub);

impl<O: HasStub> Clone for Global<O> {
    fn clone(&self) -> Self {
        Global(O::Stub::new(self.0.handle().clone()))
    }
}

pub trait HasStub {
    type Stub: ObjectStub;
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

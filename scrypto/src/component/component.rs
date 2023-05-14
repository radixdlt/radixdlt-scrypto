use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Attachable, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ScryptoSbor};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientObjectApi};
use radix_engine_interface::data::scrypto::{
    scrypto_decode, ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::types::*;
use sbor::rust::ops::Deref;
use sbor::rust::prelude::*;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Describe, Encode, EncodeError, Encoder, GlobalTypeId,
    ValueKind,
};
use scrypto::modules::{Attached, Metadata};

pub trait ComponentState<C: Component>: ScryptoEncode + ScryptoDecode {
    const BLUEPRINT_NAME: &'static str;

    fn instantiate(self) -> Globalizeable<C> {
        let node_id = ScryptoEnv
            .new_simple_object(Self::BLUEPRINT_NAME, vec![scrypto_encode(&self).unwrap()])
            .unwrap();

        let stub = C::new(ObjectStubHandle::Own(Own(node_id)));
        Globalizeable::new(stub)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ObjectStubHandle {
    Own(Own),
    Global(GlobalAddress),
}

impl ObjectStubHandle {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            ObjectStubHandle::Own(own) => own.as_node_id(),
            ObjectStubHandle::Global(address) => address.as_node_id(),
        }
    }
}

pub trait Component {
    fn new(handle: ObjectStubHandle) -> Self;

    fn handle(&self) -> &ObjectStubHandle;

    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.handle().as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn blueprint(&self) -> Blueprint {
        ScryptoEnv
            .get_object_info(self.handle().as_node_id())
            .unwrap()
            .blueprint
    }
}

pub struct AnyComponent(ObjectStubHandle);

impl Component for AnyComponent {
    fn new(handle: ObjectStubHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Globalizeable<O: Component> {
    pub stub: O,
    pub metadata: Option<Metadata>,
    pub royalty: Option<Royalty>,
    pub access_rules: Option<AccessRules>,
    pub address: Option<ComponentAddress>,
}

impl<O: Component> Deref for Globalizeable<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl<O: Component> Globalizeable<O> {
    fn new(stub: O) -> Self {
        Self {
            stub,
            metadata: None,
            royalty: None,
            access_rules: None,
            address: None,
        }
    }

    pub fn own(self) -> O {
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

    pub fn globalize(mut self) -> Global<O> {
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

        Global(O::new(ObjectStubHandle::Global(address)))
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
        Global(Component::new(ObjectStubHandle::Global(value.into())))
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
            ObjectStubHandle::Global(address) => encoder.write_slice(&address.to_vec()),
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
            let o = O::new(ObjectStubHandle::Global(GlobalAddress::new_or_panic(
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

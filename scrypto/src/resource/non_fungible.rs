use radix_engine_lib::crypto::{hash, PublicKey};
use radix_engine_lib::resource::{NonFungibleAddress, NonFungibleId, ResourceAddress};
use sbor::rust::marker::PhantomData;

use crate::borrow_resource_manager;
use crate::constants::{ECDSA_SECP256K1_TOKEN, EDDSA_ED25519_TOKEN};
use crate::resource::*;

pub trait ScryptoNonFungibleId {
    /// Creates a non-fungible ID from some uuid.
    fn random() -> Self;
}

impl ScryptoNonFungibleId for NonFungibleId {
    fn random() -> Self {
        let bytes = crate::core::Runtime::generate_uuid().to_be_bytes().to_vec();
        Self::from_bytes(bytes)
    }
}

/// Represents a non-fungible unit.
#[derive(Debug)]
pub struct NonFungible<T: NonFungibleData> {
    address: NonFungibleAddress,
    data: PhantomData<T>,
}

impl<T: NonFungibleData> From<NonFungibleAddress> for NonFungible<T> {
    fn from(address: NonFungibleAddress) -> Self {
        Self {
            address,
            data: PhantomData,
        }
    }
}

impl<T: NonFungibleData> NonFungible<T> {
    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.address.resource_address()
    }

    /// Returns the non-fungible address.
    pub fn address(&self) -> NonFungibleAddress {
        self.address.clone()
    }

    /// Returns the non-fungible ID.
    pub fn id(&self) -> NonFungibleId {
        self.address.non_fungible_id()
    }

    /// Returns the associated data of this unit.
    pub fn data(&self) -> T {
        borrow_resource_manager!(self.resource_address()).get_non_fungible_data(&self.id())
    }

    /// Updates the associated data of this unit.
    pub fn update_data(&self, new_data: T) {
        borrow_resource_manager!(self.resource_address())
            .update_non_fungible_data(&self.id(), new_data);
    }
}

pub trait FromPublicKey: Sized {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self;
}

impl FromPublicKey for NonFungibleAddress {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => NonFungibleAddress::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleId::from_bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
            PublicKey::EddsaEd25519(public_key) => NonFungibleAddress::new(
                EDDSA_ED25519_TOKEN,
                NonFungibleId::from_bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
        }
    }
}

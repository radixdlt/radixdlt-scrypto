use radix_engine_interface::{prelude::PublicKey, types::ComponentAddress};
use transaction::prelude::*;

use crate::internal_prelude::ScenarioCore;

pub struct VirtualAccount {
    pub key: PrivateKey,
    pub public_key: PublicKey,
    pub address: ComponentAddress,
}

impl VirtualAccount {
    pub fn for_private_key(key: PrivateKey) -> Self {
        let public_key: PublicKey = key.public_key();
        let account_address = ComponentAddress::virtual_account_from_public_key(&public_key);
        Self {
            key,
            address: account_address,
            public_key,
        }
    }

    pub fn encode(&self, context: &ScenarioCore) -> String {
        self.address
            .to_string(AddressDisplayContext::with_encoder(&context.encoder()))
    }
}

impl Into<GlobalAddress> for &VirtualAccount {
    fn into(self) -> GlobalAddress {
        self.address.into()
    }
}

pub fn secp256k1_account_1() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Secp256k1PrivateKey::from_u64(33311)
            .expect("Should be valid")
            .into(),
    )
}

pub fn secp256k1_account_2() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Secp256k1PrivateKey::from_u64(32144)
            .expect("Should be valid")
            .into(),
    )
}

pub fn secp256k1_account_3() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Secp256k1PrivateKey::from_u64(53213)
            .expect("Should be valid")
            .into(),
    )
}

pub fn ed25519_account_1() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Ed25519PrivateKey::from_u64(12451)
            .expect("Should be valid")
            .into(),
    )
}

pub fn ed25519_account_2() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Ed25519PrivateKey::from_u64(43213)
            .expect("Should be valid")
            .into(),
    )
}

pub fn ed25519_account_3() -> VirtualAccount {
    VirtualAccount::for_private_key(
        Ed25519PrivateKey::from_u64(54421)
            .expect("Should be valid")
            .into(),
    )
}

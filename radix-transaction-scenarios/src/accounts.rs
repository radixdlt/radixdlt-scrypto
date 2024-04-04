use crate::internal_prelude::*;
use radix_transactions::prelude::*;

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

pub fn secp256k1_account_from_u64(val: u64) -> VirtualAccount {
    VirtualAccount::for_private_key(
        Secp256k1PrivateKey::from_u64(val)
            .expect("Should be valid")
            .into(),
    )
}

pub fn secp256k1_account_1() -> VirtualAccount {
    secp256k1_account_from_u64(33311)
}

pub fn secp256k1_account_2() -> VirtualAccount {
    secp256k1_account_from_u64(32144)
}

pub fn secp256k1_account_3() -> VirtualAccount {
    secp256k1_account_from_u64(53213)
}

pub fn secp256k1_account_dashboard() -> VirtualAccount {
    secp256k1_account_from_u64(53214)
}

pub fn secp256k1_account_sandbox() -> VirtualAccount {
    secp256k1_account_from_u64(53215)
}

pub fn ed25519_account_from_u64(key: u64) -> VirtualAccount {
    VirtualAccount::for_private_key(
        Ed25519PrivateKey::from_u64(key)
            .expect("Should be valid")
            .into(),
    )
}

pub fn ed25519_account_1() -> VirtualAccount {
    ed25519_account_from_u64(12451)
}

pub fn ed25519_account_2() -> VirtualAccount {
    ed25519_account_from_u64(43213)
}

pub fn ed25519_account_3() -> VirtualAccount {
    ed25519_account_from_u64(54421)
}

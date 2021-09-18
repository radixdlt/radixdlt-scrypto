use sbor::{describe::Type, *};

use crate::constants::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// Represents a resource container on ledger.
#[derive(Debug, Encode, Decode)]
pub struct Vault {
    vid: VID,
}

impl Describe for Vault {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_VAULT.to_owned(),
        }
    }
}

impl From<VID> for Vault {
    fn from(vid: VID) -> Self {
        Self { vid }
    }
}

impl From<Vault> for VID {
    fn from(a: Vault) -> VID {
        a.vid
    }
}

impl Vault {
    pub fn new(resource: Address) -> Self {
        let input = CreateEmptyVaultInput { resource };
        let output: CreateEmptyVaultOutput = call_kernel(CREATE_EMPTY_VAULT, input);

        output.vault.into()
    }

    pub fn wrap(bucket: Bucket) -> Self {
        let vault = Vault::new(bucket.resource());
        vault.put(bucket);
        vault
    }

    pub fn put(&self, other: Bucket) {
        let input = PutIntoVaultInput {
            vault: self.vid,
            bucket: other.into(),
        };
        let _: PutIntoVaultOutput = call_kernel(PUT_INTO_VAULT, input);
    }

    pub fn take<A: Into<Amount>>(&self, amount: A) -> Bucket {
        let input = TakeFromVaultInput {
            vault: self.vid,
            amount: amount.into(),
        };
        let output: TakeFromVaultOutput = call_kernel(TAKE_FROM_VAULT, input);

        output.bucket.into()
    }

    pub fn amount(&self) -> Amount {
        let input = GetVaultAmountInput { vault: self.vid };
        let output: GetVaultAmountOutput = call_kernel(GET_VAULT_AMOUNT, input);

        output.amount
    }

    pub fn resource(&self) -> Address {
        let input = GetVaultResourceInput { vault: self.vid };
        let output: GetVaultResourceOutput = call_kernel(GET_VAULT_RESOURCE, input);

        output.resource
    }
}

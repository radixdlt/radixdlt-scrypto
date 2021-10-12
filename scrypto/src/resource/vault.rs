use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;

/// Represents a persistent resource container on ledger state.
#[derive(Debug)]
pub struct Vault {
    vid: Vid,
}

impl From<Vid> for Vault {
    fn from(vid: Vid) -> Self {
        Self { vid }
    }
}

impl From<Vault> for Vid {
    fn from(a: Vault) -> Vid {
        a.vid
    }
}

impl Vault {
    pub fn new<A: Into<Address>>(resource_def: A) -> Self {
        let input = CreateEmptyVaultInput {
            resource_def: resource_def.into(),
        };
        let output: CreateEmptyVaultOutput = call_kernel(CREATE_EMPTY_VAULT, input);

        output.vault.into()
    }

    pub fn with_bucket(bucket: Bucket) -> Self {
        let vault = Vault::new(bucket.resource_def().address());
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

    pub fn take_all(&self) -> Bucket {
        self.take(self.amount())
    }

    pub fn amount(&self) -> Amount {
        let input = GetVaultAmountInput { vault: self.vid };
        let output: GetVaultAmountOutput = call_kernel(GET_VAULT_AMOUNT, input);

        output.amount
    }

    pub fn resource_def(&self) -> ResourceDef {
        let input = GetVaultResourceAddressInput { vault: self.vid };
        let output: GetVaultResourceAddressOutput = call_kernel(GET_VAULT_RESOURCE_DEF, input);

        output.resource_def.into()
    }

    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }
}

//========
// SBOR
//========

impl TypeId for Vault {
    fn type_id() -> u8 {
        Vid::type_id()
    }
}

impl Encode for Vault {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.vid.encode_value(encoder);
    }
}

impl Decode for Vault {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Vid::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Vault {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_VAULT.to_owned(),
            generics: vec![],
        }
    }
}

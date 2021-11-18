use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::BTreeSet;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

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
    /// Creates an empty vault to permanently hold resource of the given definition.
    pub fn new<A: Into<ResourceDef>>(resource_def: A) -> Self {
        let input = CreateEmptyVaultInput {
            resource_def: resource_def.into().address(),
        };
        let output: CreateEmptyVaultOutput = call_kernel(CREATE_EMPTY_VAULT, input);

        output.vault.into()
    }

    /// Creates an empty vault and fills it with an initial bucket of resources.
    pub fn with_bucket(bucket: Bucket) -> Self {
        let vault = Vault::new(bucket.resource_def().address());
        vault.put(bucket);
        vault
    }

    /// Puts a bucket of resources into this vault.
    pub fn put(&self, other: Bucket) {
        let input = PutIntoVaultInput {
            vault: self.vid,
            bucket: other.into(),
        };
        let _: PutIntoVaultOutput = call_kernel(PUT_INTO_VAULT, input);
    }

    /// Takes some amount of resources out of this vault.
    pub fn take<A: Into<Decimal>>(&self, amount: A) -> Bucket {
        let input = TakeFromVaultInput {
            vault: self.vid,
            amount: amount.into(),
        };
        let output: TakeFromVaultOutput = call_kernel(TAKE_FROM_VAULT, input);

        output.bucket.into()
    }

    /// Takes all resourced stored in this vault.
    pub fn take_all(&self) -> Bucket {
        self.take(self.amount())
    }

    /// Returns the amount of resources within this vault.
    pub fn amount(&self) -> Decimal {
        let input = GetVaultDecimalInput { vault: self.vid };
        let output: GetVaultDecimalOutput = call_kernel(GET_VAULT_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources within this vault.
    pub fn resource_def(&self) -> ResourceDef {
        let input = GetVaultResourceAddressInput { vault: self.vid };
        let output: GetVaultResourceAddressOutput = call_kernel(GET_VAULT_RESOURCE_DEF, input);

        output.resource_def.into()
    }

    /// Returns the resource definition address.
    pub fn resource_address(&self) -> Address {
        self.resource_def().address()
    }

    /// Checks if this vault is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Use resources in this vault as authorization for an operation.
    pub fn authorize<F: FnOnce(BucketRef) -> O, O>(&self, f: F) -> O {
        let bucket = self.take(1);
        let output = f(bucket.borrow());
        self.put(bucket);
        output
    }

    /// Takes an NFT from this vault, by id.
    ///
    /// # Panics
    /// Panics if this is not an NFT vault or the specified NFT is not found.
    pub fn take_nft(&self, id: u64) -> Bucket {
        let input = TakeNftFromVaultInput {
            vault: self.vid,
            id,
        };
        let output: TakeNftFromVaultOutput = call_kernel(TAKE_NFT_FROM_VAULT, input);

        output.bucket.into()
    }

    /// Gets the data of an NFT in this vault, by id.
    ///
    /// # Panics
    /// Panics if this is not an NFT vault or the specified NFT is not found.
    pub fn get_nft<T: Decode>(&self, id: u64) -> T {
        let input = GetNftInVaultInput {
            vault: self.vid,
            id,
        };
        let output: GetNftInVaultOutput = call_kernel(TAKE_NFT_FROM_VAULT, input);

        scrypto_unwrap(scrypto_decode(&output.data))
    }

    /// Reads all the NFT IDs in this vault.
    ///
    /// # Panics
    /// Panics if this is not an NFT vault.
    pub fn get_nft_ids(&self) -> BTreeSet<u64> {
        let input = GetNftIdsInVaultInput { vault: self.vid };
        let output: GetNftIdsInVaultOutput = call_kernel(GET_NFT_IDS_IN_VAULT, input);

        output.ids
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

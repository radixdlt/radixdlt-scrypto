use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::engine::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
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
    /// Creates an empty vault to permanently hold resource of the given definition.
    pub fn new<A: Into<ResourceDef>>(resource_def: A) -> Self {
        let input = CreateEmptyVaultInput {
            resource_address: resource_def.into().address(),
        };
        let output: CreateEmptyVaultOutput = call_engine(CREATE_EMPTY_VAULT, input);

        output.vid.into()
    }

    /// Creates an empty vault and fills it with an initial bucket of resources.
    pub fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_def().address());
        vault.put(bucket);
        vault
    }

    /// Puts a bucket of resources into this vault.
    pub fn put(&mut self, bucket: Bucket) {
        let input = PutIntoVaultInput {
            vid: self.vid,
            bid: bucket.into(),
        };
        let _: PutIntoVaultOutput = call_engine(PUT_INTO_VAULT, input);
    }

    /// Takes some amount of resource from this vault into a bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let input = TakeFromVaultInput {
            vid: self.vid,
            amount: amount.into(),
            auth: None,
        };
        let output: TakeFromVaultOutput = call_engine(TAKE_FROM_VAULT, input);

        output.bid.into()
    }

    /// Takes some amount of resource from this vault into a bucket.
    ///
    /// This variant of `take` accepts an additional auth parameter to support resources
    /// with or without `RESTRICTED_TRANSFER` flag on.
    pub fn take_with_auth<A: Into<Decimal>>(&mut self, amount: A, auth: BucketRef) -> Bucket {
        let input = TakeFromVaultInput {
            vid: self.vid,
            amount: amount.into(),
            auth: Some(auth.into()),
        };
        let output: TakeFromVaultOutput = call_engine(TAKE_FROM_VAULT, input);

        output.bid.into()
    }

    /// Takes all resource stored in this vault.
    pub fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Takes all resource stored in this vault.
    ///
    /// This variant of `take_all` accepts an additional auth parameter to support resources
    /// with or without `RESTRICTED_TRANSFER` flag on.
    pub fn take_all_with_auth(&mut self, auth: BucketRef) -> Bucket {
        self.take_with_auth(self.amount(), auth)
    }

    /// Takes a non-fungible from this vault, by id.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible is not found.
    pub fn take_non_fungible(&self, key: &NonFungibleKey) -> Bucket {
        let input = TakeNonFungibleFromVaultInput {
            vid: self.vid,
            key: key.clone(),
            auth: None,
        };
        let output: TakeNonFungibleFromVaultOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_VAULT, input);

        output.bid.into()
    }

    /// Takes a non-fungible from this vault, by id.
    ///
    /// This variant of `take_non_fungible` accepts an additional auth parameter to support resources
    /// with or without `RESTRICTED_TRANSFER` flag on.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible is not found.
    pub fn take_non_fungible_with_auth(&self, key: &NonFungibleKey, auth: BucketRef) -> Bucket {
        let input = TakeNonFungibleFromVaultInput {
            vid: self.vid,
            key: key.clone(),
            auth: Some(auth.into()),
        };
        let output: TakeNonFungibleFromVaultOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_VAULT, input);

        output.bid.into()
    }

    /// This is a convenience method for using the contained resource for authorization.
    ///
    /// It conducts the following actions in one shot:
    /// 1. Takes `1` resource from this vault into a bucket;
    /// 2. Creates a `BucketRef`.
    /// 3. Applies the specified function `f` with the created bucket reference;
    /// 4. Puts the `1` resource back into this vault.
    ///
    pub fn authorize<F: FnOnce(BucketRef) -> O, O>(&mut self, f: F) -> O {
        let bucket = self.take(1);
        let output = f(bucket.present());
        self.put(bucket);
        output
    }

    /// This is a convenience method for using the contained resource for authorization.
    ///
    /// It conducts the following actions in one shot:
    /// 1. Takes `1` resource from this vault into a bucket;
    /// 2. Creates a `BucketRef`.
    /// 3. Applies the specified function `f` with the created bucket reference;
    /// 4. Puts the `1` resource back into this vault.
    ///
    /// This variant of `authorize` accepts an additional auth parameter to support resources
    /// with or without `RESTRICTED_TRANSFER` flag on.
    ///
    pub fn authorize_with_auth<F: FnOnce(BucketRef) -> O, O>(
        &mut self,
        f: F,
        auth: BucketRef,
    ) -> O {
        let bucket = self.take_with_auth(1, auth);
        let output = f(bucket.present());
        self.put(bucket);
        output
    }

    /// Returns the amount of resources within this vault.
    pub fn amount(&self) -> Decimal {
        let input = GetVaultDecimalInput { vid: self.vid };
        let output: GetVaultDecimalOutput = call_engine(GET_VAULT_AMOUNT, input);

        output.amount
    }

    /// Returns the resource definition of resources within this vault.
    pub fn resource_def(&self) -> ResourceDef {
        let input = GetVaultResourceAddressInput { vid: self.vid };
        let output: GetVaultResourceAddressOutput = call_engine(GET_VAULT_RESOURCE_ADDRESS, input);

        output.resource_address.into()
    }

    /// Returns the resource definition address.
    pub fn resource_address(&self) -> Address {
        self.resource_def().address()
    }

    /// Checks if this vault is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn get_non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let input = GetNonFungibleKeysInVaultInput { vid: self.vid };
        let output: GetNonFungibleKeysInVaultOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_VAULT, input);
        let resource_address = self.resource_address();
        output
            .keys
            .iter()
            .map(|id| NonFungible::from((resource_address, id.clone())))
            .collect()
    }

    /// Get all non-fungible IDs in this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn get_non_fungible_keys(&self) -> Vec<NonFungibleKey> {
        let input = GetNonFungibleKeysInVaultInput { vid: self.vid };
        let output: GetNonFungibleKeysInVaultOutput =
            call_engine(GET_NON_FUNGIBLE_KEYS_IN_VAULT, input);

        output.keys
    }

    /// Get the non-fungible id and panic if not singleton.
    pub fn get_non_fungible_key(&self) -> NonFungibleKey {
        let keys = self.get_non_fungible_keys();
        assert!(
            keys.len() == 1,
            "Expect 1 non-fungible, but found {}",
            keys.len()
        );
        keys[0].clone()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleKey) -> T {
        self.resource_def().get_non_fungible_data(id)
    }

    /// Updates the mutable part of the data of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &self,
        id: &NonFungibleKey,
        new_data: T,
        auth: BucketRef,
    ) {
        self.resource_def()
            .update_non_fungible_data(id, new_data, auth)
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

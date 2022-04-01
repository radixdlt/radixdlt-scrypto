use sbor::*;

use crate::crypto::*;
use crate::engine::{api::*, call_engine, types::VaultId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::resource_manager;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::BTreeSet;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a persistent resource container on ledger state.
pub struct Vault(pub VaultId);

impl Vault {
    /// Creates an empty vault to permanently hold resource of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = CreateEmptyVaultInput {
            resource_address: resource_address,
        };
        let output: CreateEmptyVaultOutput = call_engine(CREATE_EMPTY_VAULT, input);

        Self(output.vault_id)
    }

    /// Creates an empty vault and fills it with an initial bucket of resource.
    pub fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    /// Puts a bucket of resources into this vault.
    pub fn put(&mut self, bucket: Bucket) {
        let input = PutIntoVaultInput {
            vault_id: self.0,
            bucket_id: bucket.0,
        };
        let _: PutIntoVaultOutput = call_engine(PUT_INTO_VAULT, input);
    }

    /// Takes some amount of resource from this vault into a bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let input = TakeFromVaultInput {
            vault_id: self.0,
            amount: amount.into(),
        };
        let output: TakeFromVaultOutput = call_engine(TAKE_FROM_VAULT, input);

        Bucket(output.bucket_id)
    }

    /// Takes all resource stored in this vault.
    pub fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Takes a non-fungible from this vault, by id.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible is not found.
    pub fn take_non_fungible(&self, non_fungible_id: &NonFungibleId) -> Bucket {
        let input = TakeNonFungibleFromVaultInput {
            vault_id: self.0,
            non_fungible_id: non_fungible_id.clone(),
        };
        let output: TakeNonFungibleFromVaultOutput =
            call_engine(TAKE_NON_FUNGIBLE_FROM_VAULT, input);

        Bucket(output.bucket_id)
    }

    /// Creates an ownership proof of this vault.
    pub fn create_proof(&self) -> Proof {
        let input = CreateVaultProofInput { vault_id: self.0 };
        let output: CreateVaultProofOutput = call_engine(CREATE_VAULT_PROOF, input);

        Proof(output.proof_id)
    }

    /// Creates an ownership proof of this vault, by amount.
    pub fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
        let input = CreateVaultProofByAmountInput {
            vault_id: self.0,
            amount,
        };
        let output: CreateVaultProofByAmountOutput =
            call_engine(CREATE_VAULT_PROOF_BY_AMOUNT, input);

        Proof(output.proof_id)
    }

    /// Creates an ownership proof of this vault, by non-fungible ID set.
    pub fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>) -> Proof {
        let input = CreateVaultProofByIdsInput {
            vault_id: self.0,
            ids: ids.clone(),
        };
        let output: CreateVaultProofByIdsOutput = call_engine(CREATE_VAULT_PROOF_BY_IDS, input);

        Proof(output.proof_id)
    }

    /// Uses resources in this vault as authorization for an operation.
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        AuthZone::push(self.create_proof());
        let output = f();
        AuthZone::pop().drop();
        output
    }

    /// Returns the amount of resources within this vault.
    pub fn amount(&self) -> Decimal {
        let input = GetVaultAmountInput { vault_id: self.0 };
        let output: GetVaultAmountOutput = call_engine(GET_VAULT_AMOUNT, input);

        output.amount
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        let input = GetVaultResourceAddressInput { vault_id: self.0 };
        let output: GetVaultResourceAddressOutput = call_engine(GET_VAULT_RESOURCE_ADDRESS, input);

        output.resource_address
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
        let input = GetNonFungibleIdsInVaultInput { vault_id: self.0 };
        let output: GetNonFungibleIdsInVaultOutput =
            call_engine(GET_NON_FUNGIBLE_IDS_IN_VAULT, input);
        let resource_address = self.resource_address();
        output
            .non_fungible_ids
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Get all non-fungible IDs in this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn get_non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
        let input = GetNonFungibleIdsInVaultInput { vault_id: self.0 };
        let output: GetNonFungibleIdsInVaultOutput =
            call_engine(GET_NON_FUNGIBLE_IDS_IN_VAULT, input);

        output.non_fungible_ids
    }

    /// Returns the address of  a singleton non-fungible.
    ///
    /// # Panic
    /// If this vault is empty or contains more than one non-fungibles.
    pub fn get_non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.get_non_fungible_ids();
        assert!(
            non_fungible_ids.len() == 1,
            "Expect 1 non-fungible, but found {}",
            non_fungible_ids.len()
        );
        non_fungible_ids.into_iter().next().unwrap()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        resource_manager!(self.resource_address()).get_non_fungible_data(id)
    }

    /// Updates the mutable part of the data of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId, new_data: T) {
        resource_manager!(self.resource_address()).update_non_fungible_data(id, new_data)
    }
}

//========
// error
//========

/// Represents an error when decoding vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseVaultError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseVaultError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseVaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Vault {
    type Error = ParseVaultError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self((
                Hash(copy_u8_array(&slice[0..32])),
                u32::from_le_bytes(copy_u8_array(&slice[32..])),
            ))),
            _ => Err(ParseVaultError::InvalidLength(slice.len())),
        }
    }
}

impl Vault {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = self.0 .0.to_vec();
        v.extend(self.0 .1.to_le_bytes());
        v
    }
}

custom_type!(Vault, CustomType::Vault, Vec::new());

//======
// text
//======

impl FromStr for Vault {
    type Err = ParseVaultError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseVaultError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Vault {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Vault {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

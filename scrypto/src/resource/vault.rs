use crate::args;
use crate::buffer::{scrypto_decode, scrypto_encode};
use crate::core::SNodeRef;
use sbor::*;

use crate::crypto::*;
use crate::engine::{api::*, call_engine, types::VaultId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::BTreeSet;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a persistent resource container on ledger state.
#[derive(PartialEq, Eq, Hash)]
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
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "put_into_vault".to_string(),
            args: args![bucket],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Takes some amount of resource from this vault into a bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let amount: Decimal = amount.into();
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "take_from_vault".to_string(),
            args: args![amount],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        let bucket: Bucket = scrypto_decode(&output.rtn).unwrap();
        bucket
    }

    /// Takes all resource stored in this vault.
    pub fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Takes a specific non-fungible from this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]))
    }

    /// Takes non-fungibles from this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible resource is not found.
    pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Bucket {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "take_non_fungibles_from_vault".to_string(),
            args: vec![scrypto_encode(non_fungible_ids)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Creates an ownership proof of this vault.
    pub fn create_proof(&self) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "create_vault_proof".to_string(),
            args: vec![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Creates an ownership proof of this vault, by amount.
    pub fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "create_vault_proof_by_amount".to_string(),
            args: vec![scrypto_encode(&amount)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
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
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "get_vault_amount".to_string(),
            args: vec![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "get_vault_resource_address".to_string(),
            args: vec![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Checks if this vault is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible ids contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::VaultRef(self.0),
            function: "get_non_fungible_ids_in_vault".to_string(),
            args: vec![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self
            .non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
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

scrypto_type!(Vault, ScryptoType::Vault, Vec::new());

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

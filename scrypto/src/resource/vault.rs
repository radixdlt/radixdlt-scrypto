use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::{MethodFnIdent, MethodIdent, ResourceManagerMethodFnIdent};
use scrypto::engine::types::GlobalAddress;

use crate::abi::*;
use crate::buffer::scrypto_encode;
use crate::core::{FnIdent, NativeMethodFnIdent, Receiver, VaultMethodFnIdent};
use crate::crypto::*;
use crate::engine::types::RENodeId;
use crate::engine::{api::*, call_engine, types::VaultId};
use crate::math::*;
use crate::misc::*;
use crate::native_functions;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultPutInput {
    pub bucket: Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultTakeInput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultTakeNonFungiblesInput {
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultGetAmountInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultGetResourceAddressInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultGetNonFungibleIdsInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultCreateProofInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultCreateProofByAmountInput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct VaultLockFeeInput {
    pub amount: Decimal,
}

/// Represents a persistent resource container on ledger state.
#[derive(PartialEq, Eq, Hash)]
pub struct Vault(pub VaultId);

impl Vault {
    /// Creates an empty vault to permanently hold resource of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = RadixEngineInput::Invoke(
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Resource(
                    resource_address,
                ))),
                method_fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::ResourceManager(
                    ResourceManagerMethodFnIdent::CreateVault,
                )),
            }),
            scrypto_encode(&ResourceManagerCreateVaultInput {}),
        );
        call_engine(input)
    }

    /// Creates an empty vault and fills it with an initial bucket of resource.
    pub fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    fn take_internal(&mut self, amount: Decimal) -> Bucket {
        let input = RadixEngineInput::Invoke(
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Vault(self.0)),
                method_fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Vault(
                    VaultMethodFnIdent::Take,
                )),
            }),
            scrypto_encode(&VaultTakeInput { amount }),
        );
        call_engine(input)
    }

    fn lock_fee_internal(&mut self, amount: Decimal) {
        let input = RadixEngineInput::Invoke(
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Vault(self.0)),
                method_fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Vault(
                    VaultMethodFnIdent::LockFee,
                )),
            }),
            scrypto_encode(&VaultTakeInput { amount }),
        );
        call_engine(input)
    }

    fn lock_contingent_fee_internal(&mut self, amount: Decimal) {
        let input = RadixEngineInput::Invoke(
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Vault(self.0)),
                method_fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Vault(
                    VaultMethodFnIdent::LockContingentFee,
                )),
            }),
            scrypto_encode(&VaultTakeInput { amount }),
        );
        call_engine(input)
    }

    native_functions! {
        Receiver::Ref(RENodeId::Vault(self.0)), NativeMethodFnIdent::Vault => {
            pub fn put(&mut self, bucket: Bucket) -> () {
                VaultMethodFnIdent::Put,
                VaultPutInput {
                    bucket
                }
            }

            pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Bucket {
                VaultMethodFnIdent::TakeNonFungibles,
                VaultTakeNonFungiblesInput {
                    non_fungible_ids: non_fungible_ids.clone(),
                }
            }

            pub fn amount(&self) -> Decimal {
                VaultMethodFnIdent::GetAmount,
                VaultGetAmountInput {}
            }

            pub fn resource_address(&self) -> ResourceAddress {
                VaultMethodFnIdent::GetResourceAddress,
                VaultGetResourceAddressInput {}
            }

            pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
                VaultMethodFnIdent::GetNonFungibleIds,
                VaultGetNonFungibleIdsInput {}
            }

            pub fn create_proof(&self) -> Proof {
                VaultMethodFnIdent::CreateProof,
                VaultCreateProofInput {}
            }

            pub fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
                VaultMethodFnIdent::CreateProofByAmount,
                VaultCreateProofByAmountInput { amount }
            }

            pub fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>) -> Proof {
                VaultMethodFnIdent::CreateProofByIds,
                VaultCreateProofByIdsInput { ids: ids.clone() }
            }
        }
    }

    /// Locks the specified amount as transaction fee.
    ///
    /// Unused fee will be refunded to the vaults from the most recently locked to the least.
    pub fn lock_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_fee_internal(amount.into())
    }

    /// Locks the given amount of resource as contingent fee.
    ///
    /// The locked amount will be used as transaction only if the transaction succeeds;
    /// Unused amount will be refunded the original vault.
    pub fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_contingent_fee_internal(amount.into())
    }

    /// Takes some amount of resource from this vault into a bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        self.take_internal(amount.into())
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

    /// Uses resources in this vault as authorization for an operation.
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this vault is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.non_fungible_ids();
        if non_fungible_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_ids().into_iter().next().unwrap()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        non_fungibles.into_iter().next().unwrap()
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

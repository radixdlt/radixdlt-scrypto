use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::ScryptoCustomValueKind;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use radix_engine_interface::Categorize;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::scrypto_env_native_fn;
use scrypto_abi::Type;

use crate::resource::*;
use crate::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Vault(pub VaultId); // scrypto stub

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Vault {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Vault {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Vault(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Vault {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Vault(vault_id) => Ok(Self(vault_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Vault {
    fn describe() -> scrypto_abi::Type {
        Type::Vault
    }
}

pub trait ScryptoVault {
    fn with_bucket(bucket: Bucket) -> Self;
    fn amount(&self) -> Decimal;
    fn new(resource_address: ResourceAddress) -> Self;
    fn take_internal(&mut self, amount: Decimal) -> Bucket;
    fn lock_fee_internal(&mut self, amount: Decimal) -> ();
    fn lock_contingent_fee_internal(&mut self, amount: Decimal) -> ();
    fn put(&mut self, bucket: Bucket) -> ();
    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Bucket;
    fn resource_address(&self) -> ResourceAddress;
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;
    fn create_proof(&self) -> Proof;
    fn create_proof_by_amount(&self, amount: Decimal) -> Proof;
    fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleLocalId>) -> Proof;
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket;
    fn take_all(&mut self) -> Bucket;
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Bucket;
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
    fn is_empty(&self) -> bool;
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
    fn non_fungible_local_id(&self) -> NonFungibleLocalId;
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;
}

impl ScryptoVault for Vault {
    /// Creates an empty vault and fills it with an initial bucket of resource.
    fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        env.invoke(VaultGetAmountInvocation { receiver: self.0 })
            .unwrap()
    }

    fn new(resource_address: ResourceAddress) -> Self {
        let mut env = ScryptoEnv;
        Self(
            env.invoke(ResourceManagerCreateVaultInvocation {
                receiver: resource_address,
            })
            .unwrap()
            .vault_id(),
        )
    }

    scrypto_env_native_fn! {

        fn take_internal(&mut self, amount: Decimal) -> Bucket {
            VaultTakeInvocation {
                receiver: self.0,
                amount,
            }
        }

        fn lock_fee_internal(&mut self, amount: Decimal) -> () {
            VaultLockFeeInvocation {
                receiver: self.0,
                amount,
                contingent: false,
            }
        }

        fn lock_contingent_fee_internal(&mut self, amount: Decimal) -> () {
            VaultLockFeeInvocation {
                receiver: self.0,
                amount,
                contingent: true,
            }
        }


        fn put(&mut self, bucket: Bucket) -> () {
            VaultPutInvocation {
                receiver: self.0,
                bucket: Bucket(bucket.0),
            }
        }

        fn take_non_fungibles(&mut self, non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>) -> Bucket {
            VaultTakeNonFungiblesInvocation {
                receiver: self.0,
                non_fungible_local_ids: non_fungible_local_ids.clone(),
            }
        }

        fn resource_address(&self) -> ResourceAddress {
            VaultGetResourceAddressInvocation {
                receiver: self.0,
            }
        }

        fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
            VaultGetNonFungibleLocalIdsInvocation {
                receiver: self.0,
            }
        }

        fn create_proof(&self) -> Proof {
            VaultCreateProofInvocation {
                receiver: self.0,
            }
        }

        fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
            VaultCreateProofByAmountInvocation {  receiver: self.0,amount }
        }

        fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleLocalId>) -> Proof {
            VaultCreateProofByIdsInvocation {  receiver: self.0, ids: ids.clone(), }
        }
    }

    /// Locks the specified amount as transaction fee.
    ///
    /// Unused fee will be refunded to the vaults from the most recently locked to the least.
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_fee_internal(amount.into())
    }

    /// Locks the given amount of resource as contingent fee.
    ///
    /// The locked amount will be used as transaction only if the transaction succeeds;
    /// Unused amount will be refunded the original vault.
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_contingent_fee_internal(amount.into())
    }

    /// Takes some amount of resource from this vault into a bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let bucket = self.take_internal(amount.into());
        Bucket(bucket.0)
    }

    /// Takes all resource stored in this vault.
    fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Takes a specific non-fungible from this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible resource is not found.
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Bucket {
        let bucket = self.take_non_fungibles(&BTreeSet::from([non_fungible_local_id.clone()]));
        Bucket(bucket.0)
    }

    /// Uses resources in this vault as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this vault is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_local_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleGlobalId::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible_local_id(&self) -> NonFungibleLocalId {
        let non_fungible_local_ids = self.non_fungible_local_ids();
        if non_fungible_local_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_local_ids().into_iter().next().unwrap()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}

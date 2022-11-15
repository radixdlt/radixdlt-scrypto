use radix_engine_lib::engine::scrypto_env::ScryptoEnv;
use radix_engine_lib::engine::types::BucketId;
use radix_engine_lib::resource::{BucketGetAmountInvocation, BucketGetNonFungibleIdsInvocation, BucketPutInvocation, BucketTakeInvocation, BucketTakeNonFungiblesInvocation, NonFungibleId, ResourceAddress};
use radix_engine_lib::scrypto_env_native_fn;
use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::misc::copy_u8_array;

use crate::abi::*;
use crate::math::*;

/// Represents a transient resource container.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

pub mod sys {
    use radix_engine_lib::engine::api::SysNativeInvokable;
    use radix_engine_lib::resource::{BucketCreateProofInvocation, BucketGetResourceAddressInvocation, ResourceAddress, ResourceManagerBurnInvocation, ResourceManagerCreateBucketInvocation};
    use crate::resource::*;
    use sbor::rust::fmt::Debug;
    use sbor::*;

    impl Bucket {
        pub fn sys_new<Y, E: Debug + TypeId + Decode>(
            receiver: ResourceAddress,
            sys_calls: &mut Y,
        ) -> Result<radix_engine_lib::resource::Bucket, E>
        where
            Y: SysNativeInvokable<ResourceManagerCreateBucketInvocation, E>,
        {
            sys_calls.sys_invoke(ResourceManagerCreateBucketInvocation { receiver })
        }

        pub fn sys_burn<Y, E: Debug + TypeId + Decode>(self, env: &mut Y) -> Result<(), E>
        where
            Y: SysNativeInvokable<ResourceManagerBurnInvocation, E>
                + SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
        {
            let receiver = self.sys_resource_address(env)?;
            env.sys_invoke(ResourceManagerBurnInvocation {
                receiver,
                bucket: radix_engine_lib::resource::Bucket(self.0),
            })
        }

        pub fn sys_resource_address<Y, E>(&self, env: &mut Y) -> Result<ResourceAddress, E>
        where
            Y: SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
            E: Debug + TypeId + Decode,
        {
            env.sys_invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
        }

        pub fn sys_create_proof<Y, E: Debug + TypeId + Decode>(
            &self,
            sys_calls: &mut Y,
        ) -> Result<radix_engine_lib::resource::Proof, E>
        where
            Y: SysNativeInvokable<BucketCreateProofInvocation, E>,
        {
            sys_calls.sys_invoke(BucketCreateProofInvocation { receiver: self.0 })
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod scr {
    use crate::engine::scrypto_env::ScryptoEnv;
    use crate::resource::*;

    impl Bucket {
        /// Creates a new bucket to hold resources of the given definition.
        pub fn new(resource_address: ResourceAddress) -> Self {
            Self::sys_new(resource_address, &mut ScryptoEnv).unwrap()
        }

        pub fn burn(self) {
            self.sys_burn(&mut ScryptoEnv).unwrap()
        }

        pub fn create_proof(&self) -> Proof {
            self.sys_create_proof(&mut ScryptoEnv).unwrap()
        }
    }
}

impl Bucket {
    pub fn resource_address(&self) -> ResourceAddress {
        self.sys_resource_address(&mut ScryptoEnv).unwrap()
    }

    scrypto_env_native_fn! {
        fn take_internal(&mut self, amount: Decimal) -> radix_engine_lib::resource::Bucket {
            BucketTakeInvocation {
                receiver: self.0,
                amount,
            }
        }

        pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> radix_engine_lib::resource::Bucket {
            BucketTakeNonFungiblesInvocation {
                receiver: self.0,
                ids: non_fungible_ids.clone()
            }
        }
        pub fn put(&mut self, other: Self) -> () {
            BucketPutInvocation {
                receiver: self.0,
                bucket: radix_engine_lib::resource::Bucket(other.0),
            }
        }
        pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
            BucketGetNonFungibleIdsInvocation {
                receiver: self.0,
            }
        }
        pub fn amount(&self) -> Decimal {
            BucketGetAmountInvocation {
                receiver: self.0,
            }
        }
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let bucket = self.take_internal(amount.into());
        Bucket(bucket.0)
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        let bucket = self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]));
        Bucket(bucket.0)
    }

    /// Uses resources in this bucket as authorization for an operation.
    #[cfg(target_arch = "wasm32")]
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    #[cfg(target_arch = "wasm32")]
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
    #[cfg(target_arch = "wasm32")]
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
    #[cfg(target_arch = "wasm32")]
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT bucket");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}

//========
// error
//========

/// Represents an error when decoding bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBucketError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Bucket {
    type Error = ParseBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseBucketError::InvalidLength(slice.len())),
        }
    }
}

impl Bucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

scrypto_type!(Bucket, ScryptoType::Bucket, Vec::new());

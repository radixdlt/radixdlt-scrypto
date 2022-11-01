use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::engine::scrypto_env::*;
use crate::engine::{api::*, types::*};
use crate::math::*;
use crate::misc::*;
use crate::native_fn;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketTakeInvocation {
    pub receiver: BucketId,
    pub amount: Decimal,
}

impl SysInvocation for BucketTakeInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for BucketTakeInvocation {}

impl Into<NativeFnInvocation> for BucketTakeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::Take(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketPutInvocation {
    pub receiver: BucketId,
    pub bucket: Bucket,
}

impl SysInvocation for BucketPutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for BucketPutInvocation {}

impl Into<NativeFnInvocation> for BucketPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(BucketMethodInvocation::Put(
            self,
        )))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketTakeNonFungiblesInvocation {
    pub receiver: BucketId,
    pub ids: BTreeSet<NonFungibleId>,
}

impl SysInvocation for BucketTakeNonFungiblesInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for BucketTakeNonFungiblesInvocation {}

impl Into<NativeFnInvocation> for BucketTakeNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::TakeNonFungibles(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetNonFungibleIdsInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl ScryptoNativeInvocation for BucketGetNonFungibleIdsInvocation {}

impl Into<NativeFnInvocation> for BucketGetNonFungibleIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetNonFungibleIds(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetAmountInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetAmountInvocation {
    type Output = Decimal;
}

impl ScryptoNativeInvocation for BucketGetAmountInvocation {}

impl Into<NativeFnInvocation> for BucketGetAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetAmount(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetResourceAddressInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl ScryptoNativeInvocation for BucketGetResourceAddressInvocation {}

impl Into<NativeFnInvocation> for BucketGetResourceAddressInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetResourceAddress(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketCreateProofInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketCreateProofInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for BucketCreateProofInvocation {}

impl Into<NativeFnInvocation> for BucketCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::CreateProof(self),
        ))
    }
}

/// Represents a transient resource container.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

pub mod sys {
    use crate::resource::bucket::BucketCreateProofInvocation;
    use crate::resource::*;
    use sbor::rust::fmt::Debug;
    use sbor::*;
    use scrypto::engine::api::SysNativeInvokable;

    impl Bucket {
        pub fn sys_new<Y, E: Debug + TypeId + Decode>(
            receiver: ResourceAddress,
            sys_calls: &mut Y,
        ) -> Result<Bucket, E>
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
                bucket: self,
            })
        }

        pub fn sys_resource_address<Y, E: Debug + TypeId + Decode>(
            &self,
            env: &mut Y,
        ) -> Result<ResourceAddress, E>
        where
            Y: SysNativeInvokable<BucketGetResourceAddressInvocation, E>,
        {
            env.sys_invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
        }

        pub fn sys_create_proof<Y, E: Debug + TypeId + Decode>(
            &self,
            sys_calls: &mut Y,
        ) -> Result<Proof, E>
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

        pub fn resource_address(&self) -> ResourceAddress {
            self.sys_resource_address(&mut ScryptoEnv).unwrap()
        }
    }
}

impl Bucket {
    native_fn! {
        fn take_internal(&mut self, amount: Decimal) -> Self {
            BucketTakeInvocation {
                receiver: self.0,
                amount,
            }
        }

        pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Self {
            BucketTakeNonFungiblesInvocation {
                receiver: self.0,
                ids: non_fungible_ids.clone()
            }
        }
        pub fn put(&mut self, other: Self) -> () {
            BucketPutInvocation {
                receiver: self.0,
                bucket: other,
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
        self.take_internal(amount.into())
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]))
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

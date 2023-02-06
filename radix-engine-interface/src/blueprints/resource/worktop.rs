use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::math::Decimal;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopPutInvocation {
    pub bucket: Bucket,
}

impl Clone for WorktopPutInvocation {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for WorktopPutInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::Put))
    }
}

impl SerializableInvocation for WorktopPutInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::Put)
    }
}

impl Into<CallTableInvocation> for WorktopPutInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::Put(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeAmountInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::TakeAmount))
    }
}

impl SerializableInvocation for WorktopTakeAmountInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::TakeAmount)
    }
}

impl Into<CallTableInvocation> for WorktopTakeAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::TakeAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeNonFungiblesInvocation {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeNonFungiblesInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::TakeNonFungibles))
    }
}

impl SerializableInvocation for WorktopTakeNonFungiblesInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::TakeNonFungibles)
    }
}

impl Into<CallTableInvocation> for WorktopTakeNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::TakeNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeAllInvocation {
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopTakeAllInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::TakeAll))
    }
}

impl SerializableInvocation for WorktopTakeAllInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::TakeAll)
    }
}

impl Into<CallTableInvocation> for WorktopTakeAllInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::TakeAll(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopAssertContainsInvocation {
    pub resource_address: ResourceAddress,
}

impl Invocation for WorktopAssertContainsInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::AssertContains))
    }
}

impl SerializableInvocation for WorktopAssertContainsInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::AssertContains)
    }
}

impl Into<CallTableInvocation> for WorktopAssertContainsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::AssertContains(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopAssertContainsAmountInvocation {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}
impl Invocation for WorktopAssertContainsAmountInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::AssertContainsAmount))
    }
}

impl SerializableInvocation for WorktopAssertContainsAmountInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::AssertContainsAmount)
    }
}

impl Into<CallTableInvocation> for WorktopAssertContainsAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::AssertContainsAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopAssertContainsNonFungiblesInvocation {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl Invocation for WorktopAssertContainsNonFungiblesInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::AssertContainsNonFungibles))
    }
}

impl SerializableInvocation for WorktopAssertContainsNonFungiblesInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::AssertContainsNonFungibles)
    }
}

impl Into<CallTableInvocation> for WorktopAssertContainsNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::AssertContainsNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopDrainInvocation {}

impl Invocation for WorktopDrainInvocation {
    type Output = Vec<Bucket>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Worktop(WorktopFn::Drain))
    }
}

impl SerializableInvocation for WorktopDrainInvocation {
    type ScryptoOutput = Vec<Bucket>;

    fn native_fn() -> NativeFn {
        NativeFn::Worktop(WorktopFn::Drain)
    }
}

impl Into<CallTableInvocation> for WorktopDrainInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Worktop(WorktopInvocation::Drain(self)).into()
    }
}

use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::math::Decimal;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

pub const WORKTOP_BLUEPRINT: &str = "Worktop";

pub const WORKTOP_PUT_IDENT: &str = "Worktop_put";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopPutInput {
    pub bucket: Bucket,
}

impl Clone for WorktopPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const WORKTOP_TAKE_IDENT: &str = "Worktop_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub const WORKTOP_TAKE_NON_FUNGIBLES_IDENT: &str = "Worktop_take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub const WORKTOP_TAKE_ALL_IDENT: &str = "Worktop_take_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct WorktopTakeAllInput {
    pub resource_address: ResourceAddress,
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

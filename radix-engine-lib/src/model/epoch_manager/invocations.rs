use sbor::rust::fmt::Debug;

use crate::model::*;
use crate::engine::{api::*, scrypto_env::*};
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {}

impl SysInvocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl ScryptoNativeInvocation for EpochManagerCreateInvocation {}

impl Into<NativeFnInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::EpochManager(
            EpochManagerFunctionInvocation::Create(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: SystemAddress,
}

impl SysInvocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for EpochManagerGetCurrentEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerGetCurrentEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::GetCurrentEpoch(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerSetEpochInvocation {
    pub receiver: SystemAddress,
    pub epoch: u64,
}

impl SysInvocation for EpochManagerSetEpochInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for EpochManagerSetEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::SetEpoch(self),
        ))
    }
}

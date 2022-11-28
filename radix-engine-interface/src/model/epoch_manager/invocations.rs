use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {}

impl Invocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl ScryptoNativeInvocation for EpochManagerCreateInvocation {
    type ScryptoOutput = SystemAddress;
}

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

impl Invocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for EpochManagerGetCurrentEpochInvocation {
    type ScryptoOutput = u64;
}

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

impl Invocation for EpochManagerSetEpochInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for EpochManagerSetEpochInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::SetEpoch(self),
        ))
    }
}

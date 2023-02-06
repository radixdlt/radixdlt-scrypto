use crate::api::types::*;
use crate::blueprints::resource::Bucket;
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct FeeReserveLockFeeInvocation {
    pub receiver: FeeReserveId, // Not in use
    pub bucket: Bucket,
}

impl Clone for FeeReserveLockFeeInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for FeeReserveLockFeeInvocation {
    type Output = Bucket;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::FeeReserve(FeeReserveFn::LockFee))
    }
}

impl SerializableInvocation for FeeReserveLockFeeInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::FeeReserve(FeeReserveFn::LockFee)
    }
}

impl Into<CallTableInvocation> for FeeReserveLockFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::FeeReserve(FeeReserveInvocation::LockFee(self)).into()
    }
}

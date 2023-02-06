use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::fee_reserve::*;
use radix_engine_interface::blueprints::resource::Bucket;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum FeeReserveError {
    OutOfUUid,
}

impl ExecutableInvocation for FeeReserveLockFeeInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor = ResolvedActor::method(
            NativeFn::FeeReserve(FeeReserveFn::LockFee),
            ResolvedReceiver::new(RENodeId::FeeReserve),
        );
        let call_frame_update = CallFrameUpdate::empty();

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for FeeReserveLockFeeInvocation {
    type Output = Bucket;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        let handle = api.lock_substate(
            RENodeId::FeeReserve,
            NodeModuleId::SELF,
            SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
            LockFlags::read_only(),
        )?;
        let substate = api.get_ref(handle)?;
        let transaction_runtime_substate = substate.transaction_runtime();
        Ok((todo!(), CallFrameUpdate::empty()))
    }
}

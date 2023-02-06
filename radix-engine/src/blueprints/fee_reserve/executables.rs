use crate::blueprints::resource::BucketSubstate;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::kernel_modules::fee::ExecutionFeeReserve;
use crate::system::kernel_modules::fee::FeeReserveError;
use crate::system::node::RENodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::fee_reserve::*;
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::blueprints::resource::ResourceOperationError;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum FeeReserveBlueprintError {
    FeeReserveError(FeeReserveError),
    ResourceError(ResourceOperationError),
}

impl From<FeeReserveError> for FeeReserveBlueprintError {
    fn from(value: FeeReserveError) -> Self {
        Self::FeeReserveError(value)
    }
}

impl From<ResourceOperationError> for FeeReserveBlueprintError {
    fn from(value: ResourceOperationError) -> Self {
        Self::ResourceError(value)
    }
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
        let bucket: BucketSubstate = api.drop_node(RENodeId::Bucket(self.bucket.0))?.into();

        let handle = api.lock_substate(
            RENodeId::FeeReserve,
            NodeModuleId::SELF,
            SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api.get_ref_mut(handle)?;
        let fee_reserve_substate = substate.fee_reserve();

        let changes = fee_reserve_substate
            .fee_reserve
            .lock_fee(
                self.vault_id,
                bucket.resource().map_err(FeeReserveBlueprintError::from)?,
                self.contingent,
            )
            .map_err(FeeReserveBlueprintError::from)?;

        let bucket_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            bucket_id,
            RENodeInit::Bucket(BucketSubstate::new(changes)),
            btreemap!(),
        )?;

        Ok((Bucket(bucket_id.into()), CallFrameUpdate::empty()))
    }
}

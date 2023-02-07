use crate::blueprints::resource::BucketSubstate;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::kernel_modules::costing::CostingError;
use crate::system::kernel_modules::costing::ExecutionFeeReserve;
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
pub enum FeeReserveError {
    CostingError(CostingError),
    ResourceError(ResourceOperationError),
}

impl From<CostingError> for FeeReserveError {
    fn from(value: CostingError) -> Self {
        Self::CostingError(value)
    }
}

impl From<ResourceOperationError> for FeeReserveError {
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
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::FeeReserve);
        call_frame_update
            .nodes_to_move
            .push(RENodeId::Bucket(self.bucket.0));

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
                bucket.resource().map_err(FeeReserveError::from)?,
                self.contingent,
            )
            .map_err(FeeReserveError::from)?;

        let bucket_node_id = api.allocate_node_id(RENodeType::Bucket)?;
        api.create_node(
            bucket_node_id,
            RENodeInit::Bucket(BucketSubstate::new(changes)),
            btreemap!(),
        )?;

        Ok((
            Bucket(bucket_node_id.into()),
            CallFrameUpdate::move_node(bucket_node_id),
        ))
    }
}

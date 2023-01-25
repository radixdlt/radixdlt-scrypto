use crate::{
    blueprints::transaction_processor::{NativeOutput, TransactionProcessorError},
    errors::{ApplicationError, RuntimeError},
    kernel::kernel_api::LockFlags,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    types::*,
};
use radix_engine_interface::api::ClientStaticInvokeApi;
use radix_engine_interface::api::{
    types::{
        AccessRulesChainInvocation, AuthZoneStackInvocation, BucketInvocation, CallTableInvocation,
        ClockInvocation, ComponentInvocation, EpochManagerInvocation, IdentityInvocation,
        LoggerInvocation, MetadataInvocation, NativeInvocation, PackageInvocation, ProofInvocation,
        ResourceInvocation, TransactionRuntimeInvocation, ValidatorInvocation, VaultInvocation,
        WorktopInvocation,
    },
    types::{ScryptoInvocation, ScryptoReceiver},
};

pub fn invoke_scrypto_fn<Y, E>(
    invocation: ScryptoInvocation,
    api: &mut Y,
) -> Result<IndexedScryptoValue, E>
where
    Y: ClientStaticInvokeApi<E>,
{
    let rtn = api.invoke(invocation)?;
    Ok(IndexedScryptoValue::from_value(rtn))
}

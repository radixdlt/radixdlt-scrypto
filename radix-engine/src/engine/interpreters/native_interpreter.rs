use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

pub struct NativeInterpreter;
use sbor::rust::fmt::Debug;
use sbor::*;
use scrypto::engine::api::{ScryptoSyscalls, SysInvokableNative};
use scrypto::resource::AuthZoneDrainInput;

impl<E: Into<ApplicationError>> Into<RuntimeError> for InvokeError<E> {
    fn into(self) -> RuntimeError {
        match self {
            InvokeError::Downstream(runtime_error) => runtime_error,
            InvokeError::Error(e) => RuntimeError::ApplicationError(e.into()),
        }
    }
}

impl Into<ApplicationError> for TransactionProcessorError {
    fn into(self) -> ApplicationError {
        ApplicationError::TransactionProcessorError(self)
    }
}

impl Into<ApplicationError> for PackageError {
    fn into(self) -> ApplicationError {
        ApplicationError::PackageError(self)
    }
}

impl Into<ApplicationError> for ResourceManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::ResourceManagerError(self)
    }
}

impl Into<ApplicationError> for BucketError {
    fn into(self) -> ApplicationError {
        ApplicationError::BucketError(self)
    }
}

impl Into<ApplicationError> for ProofError {
    fn into(self) -> ApplicationError {
        ApplicationError::ProofError(self)
    }
}

impl Into<ApplicationError> for AuthZoneError {
    fn into(self) -> ApplicationError {
        ApplicationError::AuthZoneError(self)
    }
}

impl Into<ApplicationError> for WorktopError {
    fn into(self) -> ApplicationError {
        ApplicationError::WorktopError(self)
    }
}

impl Into<ApplicationError> for VaultError {
    fn into(self) -> ApplicationError {
        ApplicationError::VaultError(self)
    }
}

impl Into<ApplicationError> for ComponentError {
    fn into(self) -> ApplicationError {
        ApplicationError::ComponentError(self)
    }
}

impl Into<ApplicationError> for EpochManagerError {
    fn into(self) -> ApplicationError {
        ApplicationError::EpochManagerError(self)
    }
}

pub trait InvokableNative<'a>:
    Invokable<EpochManagerCreateInput>
    + Invokable<PackagePublishInput>
    + Invokable<ResourceManagerBurnInput>
    + Invokable<ResourceManagerCreateInput>
    + Invokable<TransactionProcessorRunInput<'a>>
    + Invokable<BucketTakeInput>
    + Invokable<BucketCreateProofInput>
    + Invokable<BucketTakeNonFungiblesInput>
    + Invokable<BucketGetNonFungibleIdsInput>
    + Invokable<BucketGetAmountInput>
    + Invokable<BucketPutInput>
    + Invokable<BucketGetResourceAddressInput>
    + Invokable<AuthZonePopInput>
    + Invokable<AuthZonePushInput>
    + Invokable<AuthZoneCreateProofInput>
    + Invokable<AuthZoneCreateProofByAmountInput>
    + Invokable<AuthZoneCreateProofByIdsInput>
    + Invokable<AuthZoneClearInput>
    + Invokable<AuthZoneDrainInput>
    + Invokable<ProofGetAmountInput>
    + Invokable<ProofGetNonFungibleIdsInput>
    + Invokable<ProofGetResourceAddressInput>
    + Invokable<ProofCloneInput>
    + Invokable<WorktopPutInput>
    + Invokable<WorktopTakeAmountInput>
    + Invokable<WorktopTakeAllInput>
    + Invokable<WorktopTakeNonFungiblesInput>
    + Invokable<WorktopAssertContainsInput>
    + Invokable<WorktopAssertContainsAmountInput>
    + Invokable<WorktopAssertContainsNonFungiblesInput>
    + Invokable<WorktopDrainInput>
    + Invokable<VaultTakeInput>
    + Invokable<VaultPutInput>
    + Invokable<VaultLockFeeInput>
    + Invokable<VaultTakeNonFungiblesInput>
    + Invokable<VaultGetAmountInput>
    + Invokable<VaultGetResourceAddressInput>
    + Invokable<VaultGetNonFungibleIdsInput>
    + Invokable<VaultCreateProofInput>
    + Invokable<VaultCreateProofByAmountInput>
    + Invokable<VaultCreateProofByIdsInput>
    + Invokable<ComponentAddAccessCheckInput>
    + Invokable<ResourceManagerBurnInput>
    + Invokable<ResourceManagerUpdateAuthInput>
    + Invokable<ResourceManagerLockAuthInput>
    + Invokable<ResourceManagerCreateVaultInput>
    + Invokable<ResourceManagerCreateBucketInput>
    + Invokable<ResourceManagerMintInput>
    + Invokable<ResourceManagerGetMetadataInput>
    + Invokable<ResourceManagerGetResourceTypeInput>
    + Invokable<ResourceManagerGetTotalSupplyInput>
    + Invokable<ResourceManagerUpdateMetadataInput>
    + Invokable<ResourceManagerUpdateNonFungibleDataInput>
    + Invokable<ResourceManagerNonFungibleExistsInput>
    + Invokable<ResourceManagerGetNonFungibleInput>
    + Invokable<ResourceManagerSetResourceAddressInput>
    + Invokable<EpochManagerGetCurrentEpochInput>
    + Invokable<EpochManagerSetEpochInput>
{
}

// TODO: This should be cleaned up
pub enum NativeInvocationInfo {
    Function(NativeFunction, CallFrameUpdate),
    Method(NativeMethod, RENodeId, CallFrameUpdate),
}

impl<N: NativeExecutable> Invocation for N {
    type Output = <N as NativeExecutable>::NativeOutput;
}

pub struct NativeResolver;

impl<N: NativeInvocation> Resolver<N> for NativeResolver {
    type Exec = NativeExecutor<N>;

    fn resolve<D: MethodDeref>(
        invocation: N,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let info = invocation.info();
        let (actor, call_frame_update) = match info {
            NativeInvocationInfo::Method(method, receiver, mut call_frame_update) => {
                // TODO: Move this logic into kernel
                let resolved_receiver = if let Some(derefed) = deref.deref(receiver)? {
                    ResolvedReceiver::derefed(derefed, receiver)
                } else {
                    ResolvedReceiver::new(receiver)
                };
                let resolved_node_id = resolved_receiver.receiver;
                call_frame_update.node_refs_to_copy.insert(resolved_node_id);

                let actor = REActor::Method(ResolvedMethod::Native(method), resolved_receiver);
                (actor, call_frame_update)
            }
            NativeInvocationInfo::Function(native_function, call_frame_update) => {
                let actor = REActor::Function(ResolvedFunction::Native(native_function));
                (actor, call_frame_update)
            }
        };

        let input = ScryptoValue::from_typed(&invocation);
        let executor = NativeExecutor(invocation, input);
        Ok((actor, call_frame_update, executor))
    }
}

pub trait NativeInvocation: NativeExecutable + Encode + Debug {
    fn info(&self) -> NativeInvocationInfo;
}

pub trait NativeExecutable: Invocation {
    type NativeOutput: Debug;

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + Invokable<ScryptoInvocation> + InvokableNative<'a> + ScryptoSyscalls<RuntimeError> + SysInvokableNative<RuntimeError>,
        R: FeeReserve;
}

pub struct NativeExecutor<N: NativeExecutable>(pub N, pub ScryptoValue);

impl<N: NativeExecutable> Executor for NativeExecutor<N> {
    type Output = <N as Invocation>::Output;

    fn args(&self) -> &ScryptoValue {
        &self.1
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R> + Invokable<ScryptoInvocation> + InvokableNative<'a> + ScryptoSyscalls<RuntimeError> + SysInvokableNative<RuntimeError>,
        R: FeeReserve,
    {
        N::execute(self.0, system_api)
    }
}

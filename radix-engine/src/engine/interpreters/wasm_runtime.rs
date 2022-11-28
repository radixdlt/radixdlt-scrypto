use crate::engine::*;
use crate::fee::*;
use crate::model::InvokeError;
use crate::types::{scrypto_decode, scrypto_encode, ScryptoInvocation};
use crate::wasm::*;
use radix_engine_interface::api::api::{EngineApi, SysInvokableNative};
use radix_engine_interface::api::wasm_input::{
    AccessRulesMethodInvocation, AuthZoneMethodInvocation, BucketMethodInvocation,
    EpochManagerFunctionInvocation, EpochManagerMethodInvocation, NativeFnInvocation,
    NativeFunctionInvocation, NativeMethodInvocation, PackageFunctionInvocation,
    ProofMethodInvocation, RadixEngineInput, ResourceManagerFunctionInvocation,
    ResourceManagerMethodInvocation, VaultMethodInvocation, WorktopMethodInvocation,
};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoEncode};
use sbor::rust::vec::Vec;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + EngineApi<RuntimeError>
        + Invokable<ScryptoInvocation>
        + SysInvokableNative<RuntimeError>,
{
    system_api: &'y mut Y,
}

impl<'y, Y> RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + EngineApi<RuntimeError>
        + Invokable<ScryptoInvocation>
        + SysInvokableNative<RuntimeError>,
{
    pub fn new(system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime { system_api }
    }

    pub fn invoke_native_fn(
        &mut self,
        native_fn_invocation: NativeFnInvocation,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match native_fn_invocation {
            NativeFnInvocation::Function(native_function) => match native_function {
                NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                    EpochManagerFunctionInvocation::Create(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                    ResourceManagerFunctionInvocation::Create(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerFunctionInvocation::BurnBucket(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Package(invocation) => match invocation {
                    PackageFunctionInvocation::Publish(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
            NativeFnInvocation::Method(native_method) => match native_method {
                NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                    BucketMethodInvocation::Take(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::CreateProof(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::TakeNonFungibles(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetNonFungibleIds(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::Put(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetResourceAddress(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AuthZone(auth_zone_method) => match auth_zone_method {
                    AuthZoneMethodInvocation::Pop(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::Push(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::CreateProof(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::CreateProofByAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::CreateProofByIds(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::Clear(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneMethodInvocation::Drain(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Proof(proof_method) => match proof_method {
                    ProofMethodInvocation::GetAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetNonFungibleIds(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetResourceAddress(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::Clone(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Vault(vault_method) => match vault_method {
                    VaultMethodInvocation::Take(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::Put(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::LockFee(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::TakeNonFungibles(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetResourceAddress(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetNonFungibleIds(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProof(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByIds(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AccessRules(component_method) => match component_method {
                    AccessRulesMethodInvocation::AddAccessCheck(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AccessRulesMethodInvocation::UpdateAuth(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AccessRulesMethodInvocation::LockAuth(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                    ResourceManagerMethodInvocation::Burn(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateAuth(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::LockAuth(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateVault(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateBucket(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::Mint(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetMetadata(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetResourceType(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetTotalSupply(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateMetadata(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::NonFungibleExists(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetNonFungible(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                    match epoch_manager_method {
                        EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => self
                            .system_api
                            .sys_invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        EpochManagerMethodInvocation::SetEpoch(invocation) => self
                            .system_api
                            .sys_invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                    }
                }
                NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                    WorktopMethodInvocation::TakeNonFungibles(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Put(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Drain(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsNonFungibles(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContains(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAll(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAmount(invocation) => self
                        .system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
        }
    }
}

fn encode<T: ScryptoEncode>(output: T) -> Result<Vec<u8>, InvokeError<WasmError>> {
    scrypto_encode(&output).map_err(|err| {
        InvokeError::Downstream(RuntimeError::KernelError(
            KernelError::InvalidSborValueOnEncode(err),
        ))
    })
}

impl<'y, Y> WasmRuntime for RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + EngineApi<RuntimeError>
        + Invokable<ScryptoInvocation>
        + SysInvokableNative<RuntimeError>,
{
    // TODO: expose API for reading blobs
    // TODO: do we want to allow dynamic creation of blobs?
    // TODO: do we check existence of blobs when being passed as arguments/return?

    fn main(&mut self, input: IndexedScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        let rtn = match input {
            RadixEngineInput::InvokeScryptoFunction(function_ident, args) => self
                .system_api
                .sys_invoke_scrypto_function(function_ident, args)?,
            RadixEngineInput::InvokeScryptoMethod(method_ident, args) => self
                .system_api
                .sys_invoke_scrypto_method(method_ident, args)?,
            RadixEngineInput::InvokeNativeFn(native_fn) => {
                self.invoke_native_fn(native_fn).map(|v| v.raw)?
            }
            RadixEngineInput::CreateNode(node) => encode(self.system_api.sys_create_node(node)?)?,
            RadixEngineInput::GetVisibleNodeIds() => {
                encode(self.system_api.sys_get_visible_nodes()?)?
            }
            RadixEngineInput::DropNode(node_id) => encode(self.system_api.sys_drop_node(node_id)?)?,
            RadixEngineInput::LockSubstate(node_id, offset, mutable) => encode(
                self.system_api
                    .sys_lock_substate(node_id, offset, mutable)?,
            )?,
            RadixEngineInput::Read(lock_handle) => self.system_api.sys_read(lock_handle)?,
            RadixEngineInput::Write(lock_handle, value) => {
                encode(self.system_api.sys_write(lock_handle, value)?)?
            }
            RadixEngineInput::DropLock(lock_handle) => {
                encode(self.system_api.sys_drop_lock(lock_handle)?)?
            }
            RadixEngineInput::GetActor() => encode(self.system_api.sys_get_actor()?)?,
            RadixEngineInput::GetTransactionHash() => {
                encode(self.system_api.sys_get_transaction_hash()?)?
            }
            RadixEngineInput::GenerateUuid() => encode(self.system_api.sys_generate_uuid()?)?,
            RadixEngineInput::EmitLog(level, message) => {
                encode(self.system_api.sys_emit_log(level, message)?)?
            }
        };

        Ok(rtn)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.system_api
            .consume_cost_units(n)
            .map_err(InvokeError::downstream)
    }
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    fee_reserve: SystemLoanFeeReserve,
}

impl NopWasmRuntime {
    pub fn new(fee_reserve: SystemLoanFeeReserve) -> Self {
        Self { fee_reserve }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: IndexedScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        Ok(IndexedScryptoValue::unit().raw)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.fee_reserve
            .consume_flat(n, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}

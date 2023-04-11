use crate::blueprints::package::PackageCodeTypeSubstate;
use crate::errors::{KernelError, RuntimeError, SystemUpstreamError};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation, KernelNodeApi, KernelSubstateApi, KernelUpstream};
use crate::system::module::SystemModule;
use crate::system::module_mixer::SystemModuleMixer;
use crate::system::system_downstream::SystemDownstream;
use crate::system::system_modules::virtualization::VirtualizationModule;
use crate::types::*;
use crate::vm::wasm::{WasmEngine, WasmRuntime};
use crate::vm::{NativeVm, ScryptoRuntime, ScryptoVm};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::{ClientApi, ClientBlueprintApi};
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::api::ClientTransactionLimitsApi;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{
    Proof, ProofDropInput, PROOF_BLUEPRINT, PROOF_DROP_IDENT,
};
use radix_engine_interface::schema::BlueprintSchema;

pub trait SystemUpstreamApi {
    fn invoke<Y>(
        &mut self,
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi;
}
use core::fmt::Debug;
use radix_engine_interface::api::types::{NativeFunction, PackageFunction, PackageId, RENodeId};

use crate::engine::*;
use crate::engine::{
    CallFrameUpdate, LockFlags, NativeExecutable, NativeInvocation, NativeInvocationInfo,
    RuntimeError, SystemApi,
};
use crate::model::{GlobalAddressSubstate, PackageSubstate};
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::types::{NativeMethod, SubstateOffset};
use radix_engine_interface::model::*;

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl Package {
    fn new(
        code: Vec<u8>,
        abi: HashMap<String, BlueprintAbi>,
    ) -> Result<PackageSubstate, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(PackageSubstate {
            code: code,
            blueprint_abis: abi,
        })
    }
}

impl NativeExecutable for PackagePublishInvocation {
    type NativeOutput = PackageAddress;

    fn execute<Y>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation>,
    {
        let code = system_api.read_blob(&invocation.code.0)?.to_vec();
        let blob = system_api.read_blob(&invocation.abi.0)?;
        let abi = scrypto_decode::<HashMap<String, BlueprintAbi>>(blob).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let package = Package::new(code, abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;

        let node_id = system_api.create_node(RENode::Package(package))?;
        let package_id: PackageId = node_id.into();

        let global_node_id =
            system_api.create_node(RENode::Global(GlobalAddressSubstate::Package(package_id)))?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for PackagePublishInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::Package(PackageFunction::Publish),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for PackageSetRoyaltyConfigInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        // TODO: auth check
        let node_id = RENodeId::Global(GlobalAddress::Package(input.receiver));
        let offset = SubstateOffset::Package(PackageOffset::RoyaltyConfig);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_mut = system_api.get_ref_mut(handle)?;
        substate_mut.package_royalty_config().royalty_config = input.royalty_config;

        system_api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for PackageSetRoyaltyConfigInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Package(PackageMethod::SetRoyaltyConfig),
            RENodeId::Global(GlobalAddress::Package(self.receiver)),
            CallFrameUpdate::empty(),
        )
    }
}

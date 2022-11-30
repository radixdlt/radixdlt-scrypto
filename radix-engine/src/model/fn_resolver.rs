use crate::types::*;
use radix_engine_interface::api::types::{
    AccessRulesMethod, AuthZoneStackMethod, BucketMethod, EpochManagerFunction, EpochManagerMethod,
    GlobalAddress, NativeFunction, NativeMethod, PackageFunction, ProofMethod, RENodeId,
    ResourceManagerFunction, ResourceManagerMethod, TransactionProcessorFunction, VaultMethod,
    WorktopMethod,
};

pub fn resolve_native_function(
    blueprint_name: &str,
    function_name: &str,
) -> Option<NativeFunction> {
    match blueprint_name {
        EPOCH_MANAGER_BLUEPRINT => EpochManagerFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::EpochManager),
        CLOCK_BLUEPRINT => ClockFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::Clock),
        RESOURCE_MANAGER_BLUEPRINT => ResourceManagerFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::ResourceManager),
        PACKAGE_BLUEPRINT => PackageFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::Package),
        TRANSACTION_PROCESSOR_BLUEPRINT => TransactionProcessorFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::TransactionProcessor),
        _ => None,
    }
}

// TODO: receiver should be receiver type rather than node_id
pub fn resolve_native_method(receiver: RENodeId, method_name: &str) -> Option<NativeMethod> {
    match receiver {
        RENodeId::Package(_) | RENodeId::Global(GlobalAddress::Package(_)) => {
            PackageMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::Package)
        }
        RENodeId::Component(_) | RENodeId::Global(GlobalAddress::Component(_)) => {
            ComponentMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::Component)
        }
        RENodeId::ResourceManager(_) | RENodeId::Global(GlobalAddress::Resource(_)) => {
            ResourceManagerMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::ResourceManager)
        }
        RENodeId::EpochManager(_) | RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)) => {
            EpochManagerMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::EpochManager)
        }
        RENodeId::Clock(_) | RENodeId::Global(GlobalAddress::System(CLOCK)) => {
            ClockMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::Clock)
        }
        RENodeId::Bucket(_) => BucketMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Bucket),

        RENodeId::Proof(_) => ProofMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Proof),

        RENodeId::AuthZoneStack(_) => AuthZoneStackMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::AuthZoneStack),

        RENodeId::Worktop => WorktopMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Worktop),

        RENodeId::Vault(_) => VaultMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Vault),

        RENodeId::Global(_)
        | RENodeId::KeyValueStore(_)
        | RENodeId::NonFungibleStore(_)
        | RENodeId::FeeReserve(_) => None,
    }
    .or(AccessRulesMethod::from_str(method_name)
        .ok()
        .map(NativeMethod::AccessRules))
    .or(MetadataMethod::from_str(method_name)
        .ok()
        .map(NativeMethod::Metadata))
}

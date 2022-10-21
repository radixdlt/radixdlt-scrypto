use crate::types::*;

pub fn resolve_native_function(
    blueprint_name: &str,
    function_name: &str,
) -> Option<NativeFunction> {
    match blueprint_name {
        "System" => SystemFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::System),
        "ResourceManager" => ResourceManagerFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::ResourceManager),
        "Package" => PackageFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::Package),
        "TransactionProcessor" => TransactionProcessorFunction::from_str(function_name)
            .ok()
            .map(NativeFunction::TransactionProcessor),
        _ => None,
    }
}

pub fn resolve_native_method(receiver: &Receiver, method_name: &str) -> Option<NativeMethod> {
    match receiver.node_id() {
        RENodeId::Bucket(_) => BucketMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Bucket),

        RENodeId::Proof(_) => ProofMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Proof),

        RENodeId::AuthZoneStack(_) => AuthZoneMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::AuthZone),

        RENodeId::Worktop => WorktopMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Worktop),

        RENodeId::Component(_) => {
            ComponentMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::Component)
        }
        RENodeId::Global(GlobalAddress::Component(component_address))
        if matches!(component_address, ComponentAddress::Normal(..) | ComponentAddress::Account(..)) => {
            ComponentMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::Component)
        }
        RENodeId::System(_) => SystemMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::System),
        RENodeId::Global(GlobalAddress::Component(component_address)) if matches!(component_address, ComponentAddress::System(..)) => {
            SystemMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::System)
        }

        RENodeId::Vault(_) => VaultMethod::from_str(method_name)
            .ok()
            .map(NativeMethod::Vault),

        RENodeId::ResourceManager(_) | RENodeId::Global(GlobalAddress::Resource(_)) => {
            ResourceManagerMethod::from_str(method_name)
                .ok()
                .map(NativeMethod::ResourceManager)
        }
        RENodeId::Global(_)
        | RENodeId::KeyValueStore(_)
        | RENodeId::NonFungibleStore(_)
        | RENodeId::Package(_) => None,
    }
}

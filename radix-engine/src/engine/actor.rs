use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, PartialOrd, Ord)]
pub enum NativeMethod {
    Component(ComponentMethod),
    System(SystemMethod),
    AuthZone(AuthZoneMethod),
    ResourceManager(ResourceManagerMethod),
    Bucket(BucketMethod),
    Vault(VaultMethod),
    Proof(ProofMethod),
    Worktop(WorktopMethod),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, PartialOrd, Ord)]
pub enum NativeFunction {
    System(SystemFunction),
    ResourceManager(ResourceManagerFunction),
    Package(PackageFunction),
    TransactionProcessor(TransactionProcessorFunction),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ResolvedMethod {
    Scrypto {
        package_address: PackageAddress,
        package_id: PackageId,
        blueprint_name: String,
        ident: String,
        export_name: String,
    },
    Native(NativeMethod),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ResolvedFunction {
    Scrypto {
        package_address: PackageAddress,
        package_id: PackageId,
        blueprint_name: String,
        ident: String,
        export_name: String,
    },
    Native(NativeFunction),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum REActor {
    Function(ResolvedFunction),
    Method(ResolvedMethod, Receiver),
}

impl REActor {
    pub fn is_substate_readable(&self, node_id: RENodeId, offset: SubstateOffset) -> bool {
        match self {
            REActor::Function(ResolvedFunction::Native(..))
            | REActor::Method(ResolvedMethod::Native(..), ..) => true,
            REActor::Function(ResolvedFunction::Scrypto { .. }) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                (RENodeId::Global(_), SubstateOffset::Global(GlobalOffset::Global)) => true,
                (RENodeId::Component(_), SubstateOffset::Component(ComponentOffset::Info)) => true,
                _ => false,
            },
            REActor::Method(
                ResolvedMethod::Scrypto { .. },
                Receiver::Ref(RENodeId::Component(component_address)),
            ) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                (RENodeId::Component(_), SubstateOffset::Component(ComponentOffset::Info)) => true,
                (RENodeId::Component(addr), SubstateOffset::Component(ComponentOffset::State)) => {
                    addr.eq(component_address)
                }
                (RENodeId::Global(_), SubstateOffset::Global(GlobalOffset::Global)) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_substate_writeable(&self, node_id: RENodeId, offset: SubstateOffset) -> bool {
        match self {
            REActor::Function(ResolvedFunction::Native(..))
            | REActor::Method(ResolvedMethod::Native(..), ..) => true,
            REActor::Function(ResolvedFunction::Scrypto { .. }) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                _ => false,
            },
            REActor::Method(
                ResolvedMethod::Scrypto { .. },
                Receiver::Ref(RENodeId::Component(component_address)),
            ) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                (RENodeId::Component(addr), SubstateOffset::Component(ComponentOffset::State)) => {
                    addr.eq(component_address)
                }
                _ => false,
            },
            _ => false,
        }
    }
}

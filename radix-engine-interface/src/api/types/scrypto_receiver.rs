use crate::api::types::*;
use crate::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Resource(ResourceAddress),
    Package(PackageAddress),
    Vault(VaultId),
    Component(ComponentId),
    Proof(ProofId),
    Bucket(BucketId),
    Worktop,
    Logger,
    TransactionRuntime,
    AuthZoneStack,
    AccessController(AccessControllerId),
    Validator(ValidatorId),
}

impl Into<RENodeId> for ScryptoReceiver {
    fn into(self) -> RENodeId {
        match self {
            ScryptoReceiver::Global(component_address) => {
                RENodeId::Global(GlobalAddress::Component(component_address))
            }
            ScryptoReceiver::Resource(resource_address) => {
                RENodeId::Global(GlobalAddress::Resource(resource_address))
            }
            ScryptoReceiver::Package(package_address) => {
                RENodeId::Global(GlobalAddress::Package(package_address))
            }
            ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
            ScryptoReceiver::Vault(vault_id) => RENodeId::Vault(vault_id),
            ScryptoReceiver::Bucket(bucket_id) => RENodeId::Bucket(bucket_id),
            ScryptoReceiver::Proof(proof_id) => RENodeId::Proof(proof_id),
            ScryptoReceiver::AccessController(id) => RENodeId::AccessController(id),
            ScryptoReceiver::Validator(id) => RENodeId::Validator(id),
            ScryptoReceiver::Worktop => RENodeId::Worktop,
            ScryptoReceiver::Logger => RENodeId::Logger,
            ScryptoReceiver::TransactionRuntime => RENodeId::TransactionRuntime,
            ScryptoReceiver::AuthZoneStack => RENodeId::AuthZoneStack,
        }
    }
}

impl From<RENodeId> for ScryptoReceiver {
    fn from(value: RENodeId) -> Self {
        match value {
            RENodeId::Bucket(id) => ScryptoReceiver::Bucket(id),
            RENodeId::Proof(id) => ScryptoReceiver::Proof(id),
            RENodeId::AuthZoneStack => ScryptoReceiver::AuthZoneStack,
            RENodeId::Worktop => ScryptoReceiver::Worktop,
            RENodeId::Logger => ScryptoReceiver::Logger,
            RENodeId::TransactionRuntime => ScryptoReceiver::TransactionRuntime,
            RENodeId::Global(address) => match address {
                GlobalAddress::Package(address) => ScryptoReceiver::Package(address),
                GlobalAddress::Resource(address) => ScryptoReceiver::Resource(address),
                GlobalAddress::Component(address) => ScryptoReceiver::Global(address),
            },
            RENodeId::Component(id) => ScryptoReceiver::Component(id),
            RENodeId::AccessController(id) => ScryptoReceiver::AccessController(id),
            RENodeId::Vault(id) => ScryptoReceiver::Vault(id),
            RENodeId::Validator(id) => ScryptoReceiver::Validator(id),
            RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::Package(..)
            | RENodeId::EpochManager(..)
            | RENodeId::Identity(..)
            | RENodeId::Clock(..)
            | RENodeId::Account(..) => {
                todo!()
            }
        }
    }
}

use crate::runtime::Runtime;
use radix_engine_interface::api::api::{EngineApi, InvokableModel};
use radix_engine_interface::data::{scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::model::*;
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn sys_new<Y, E: Debug + ScryptoDecode>(
        resource_type: ResourceType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        mint_params: Option<MintParams>,
        api: &mut Y,
    ) -> Result<(Self, Option<Bucket>), E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(ResourceManagerCreateInvocation {
            resource_type,
            metadata,
            access_rules,
            mint_params,
        })
        .map(|(address, bucket)| (ResourceManager(address), bucket))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_uuid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &mut self,
        data: T,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
    {
        // TODO: Implement UUID generation in ResourceManager
        let uuid = Runtime::generate_uuid(api)?;
        let mut entries = BTreeMap::new();
        entries.insert(
            NonFungibleId::UUID(uuid),
            (scrypto_encode(&data).unwrap(), scrypto_encode(&()).unwrap()),
        );

        api.invoke(ResourceManagerMintInvocation {
            mint_params: MintParams::NonFungible { entries },
            receiver: self.0,
        })
    }
}

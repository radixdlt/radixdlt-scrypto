use radix_engine_interface::api::api::{EngineApi, Invokable};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn sys_new_non_fungible<Y, E: Debug + ScryptoDecode>(
        id_type: NonFungibleIdTypeId,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerCreateNonFungibleInvocation, E>,
    {
        api.invoke(ResourceManagerCreateNonFungibleInvocation {
            resource_address: None,
            id_type,
            metadata,
            access_rules,
        })
        .map(|address| ResourceManager(address))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_uuid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &mut self,
        data: T,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerMintUuidNonFungibleInvocation, E>,
    {
        // TODO: Implement UUID generation in ResourceManager
        let mut entries = Vec::new();
        entries.push((scrypto_encode(&data).unwrap(), scrypto_encode(&()).unwrap()));

        api.invoke(ResourceManagerMintUuidNonFungibleInvocation {
            entries,
            receiver: self.0,
        })
    }

    pub fn get_non_fungible_data<Y, E: Debug + ScryptoDecode, T: ScryptoDecode>(
        &self,
        id: NonFungibleId,
        api: &mut Y,
    ) -> Result<T, E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerGetNonFungibleInvocation, E>,
    {
        let output = api.invoke(ResourceManagerGetNonFungibleInvocation {
            id,
            receiver: self.0,
        })?;

        let data = scrypto_decode(&output[0]).unwrap();
        Ok(data)
    }

    pub fn burn<Y, E: Debug + ScryptoDecode>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerBurnInvocation, E>,
    {
        api.invoke(ResourceManagerBurnInvocation {
            receiver: self.0,
            bucket,
        })
    }

    pub fn new_empty_bucket<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>,
    {
        api.invoke(ResourceManagerCreateBucketInvocation { receiver: self.0 })
    }
}

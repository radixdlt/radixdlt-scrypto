use radix_engine_interface::api::{ClientApi, ClientNodeApi};
use radix_engine_interface::api::Invokable;
use radix_engine_interface::args;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{RESOURCE_MANAGER_BLUEPRINT, RESOURCE_MANAGER_PACKAGE};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn new_fungible<Y, E: Debug + ScryptoDecode>(
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientNodeApi<E> + ClientApi<E>,
    {
        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT.to_string(),
            "create_fungible".to_string(),
            scrypto_encode(&ResourceManagerCreateFungibleInvocation {
                metadata,
                access_rules,
                divisibility,
            }).unwrap()
        ).unwrap();
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    pub fn new_fungible_with_initial_supply<Y, E: Debug + ScryptoDecode>(
        divisibility: u8,
        amount: Decimal,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<(Self, Bucket), E>
    where
        Y: ClientNodeApi<E> + ClientApi<E>,
    {
        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT.to_string(),
            "create_fungible_with_initial_supply".to_string(),
            scrypto_encode(&ResourceManagerCreateFungibleWithInitialSupplyInvocation {
                resource_address: None,
                metadata,
                access_rules,
                divisibility,
                initial_supply: amount,
            }).unwrap()
        ).unwrap();
        let (resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(result.as_slice()).unwrap();
        Ok((ResourceManager(resource_address), bucket))
    }

    pub fn new_non_fungible<Y, E: Debug + ScryptoDecode>(
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientNodeApi<E> + ClientApi<E>,
    {

        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT.to_string(),
            "create_non_fungible".to_string(),
            scrypto_encode(&ResourceManagerCreateNonFungibleInvocation {
                resource_address: None,
                id_type,
                metadata,
                access_rules,
            }).unwrap()
        )?;
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        local_id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E> + Invokable<ResourceManagerMintNonFungibleInvocation, E>,
    {
        let mut entries = BTreeMap::new();
        entries.insert(
            local_id,
            (scrypto_encode(&()).unwrap(), scrypto_encode(&()).unwrap()),
        );

        api.invoke(ResourceManagerMintNonFungibleInvocation {
            entries,
            receiver: self.0,
        })
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_uuid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &mut self,
        data: T,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E> + Invokable<ResourceManagerMintUuidNonFungibleInvocation, E>,
    {
        // TODO: Implement UUID generation in ResourceManager
        let mut entries = Vec::new();
        entries.push((scrypto_encode(&data).unwrap(), scrypto_encode(&()).unwrap()));

        api.invoke(ResourceManagerMintUuidNonFungibleInvocation {
            entries,
            receiver: self.0,
        })
    }

    /// Mints non-fungible resources
    pub fn mint_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientNodeApi<E> + Invokable<ResourceManagerMintFungibleInvocation, E>,
    {
        api.invoke(ResourceManagerMintFungibleInvocation {
            receiver: self.0,
            amount,
        })
    }

    pub fn get_non_fungible_mutable_data<Y, E: Debug + ScryptoDecode, T: ScryptoDecode>(
        &self,
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<T, E>
    where
        Y: ClientNodeApi<E> + Invokable<ResourceManagerGetNonFungibleInvocation, E>,
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
        Y: ClientNodeApi<E> + Invokable<ResourceManagerBurnInvocation, E>,
    {
        api.invoke(ResourceManagerBurnInvocation {
            receiver: self.0,
            bucket,
        })
    }

    pub fn total_supply<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientNodeApi<E> + Invokable<ResourceManagerGetTotalSupplyInvocation, E>,
    {
        api.invoke(ResourceManagerGetTotalSupplyInvocation { receiver: self.0 })
    }

    pub fn new_empty_bucket<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: Invokable<ResourceManagerCreateBucketInvocation, E>,
    {
        api.invoke(ResourceManagerCreateBucketInvocation { receiver: self.0 })
    }
}

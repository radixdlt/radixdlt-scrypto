use crate::runtime::Runtime;
use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::math::Decimal;
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

    /// Mints non-fungible resources
    pub fn mint_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerMintInvocation, E>,
    {
        api.invoke(ResourceManagerMintInvocation {
            mint_params: MintParams::Fungible { amount },
            receiver: self.0,
        })
    }

    pub fn get_non_fungible_data<Y, E: Debug + ScryptoDecode, T: ScryptoDecode>(
        &self,
        id: NonFungibleId,
        api: &mut Y,
    ) -> Result<T, E>
    where
        Y: EngineApi<E> + InvokableModel<E>,
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
        Y: EngineApi<E> + InvokableModel<E>,
    {
        api.invoke(ResourceManagerBurnInvocation {
            receiver: self.0,
            bucket,
        })
    }

    pub fn total_supply<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: EngineApi<E> + Invokable<ResourceManagerGetTotalSupplyInvocation, E>,
    {
        api.invoke(ResourceManagerGetTotalSupplyInvocation { receiver: self.0 })
    }
}

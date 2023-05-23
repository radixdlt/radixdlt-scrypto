use crate::blueprints::pool::single_resource_pool::*;
use crate::errors::*;
use crate::kernel::kernel_api::*;
use native_sdk::modules::access_rules::*;
use native_sdk::modules::metadata::*;
use native_sdk::modules::royalty::*;
use native_sdk::resource::*;
use radix_engine_common::math::*;
use radix_engine_common::prelude::scrypto_encode;
use radix_engine_common::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

pub const SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT: &'static str = "SingleResourcePool";

pub struct SingleResourcePoolBlueprint;
impl SingleResourcePoolBlueprint {
    pub fn instantiate<Y>(
        resource_address: ResourceAddress,
        pool_manager_rule: AccessRule,
        api: &mut Y,
    ) -> Result<SingleResourcePoolInstantiateOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi,
    {
        // Validate that the resource is a fungible resource - a pool can't be created with non
        // fungible resources.
        let resource_manager = ResourceManager(resource_address);
        if let ResourceType::NonFungible { .. } = resource_manager.resource_type(api)? {
            Err(
                SingleResourcePoolError::CantCreateAPoolOfNonFungibleResources { resource_address },
            )?
        }

        // Allowing the component address of the pool - this will be used later for the component
        // caller badge.
        let node_id = api.kernel_allocate_node_id(EntityType::GlobalSingleResourcePool)?;
        let address = GlobalAddress::new_or_panic(node_id.0);

        let pool_unit_resource = {
            let component_caller_badge = NonFungibleGlobalId::global_caller_badge(address.into());

            let mut access_rules = BTreeMap::new();

            access_rules.insert(
                Mint,
                (
                    rule!(require(component_caller_badge.clone())),
                    AccessRule::DenyAll,
                ),
            );
            access_rules.insert(
                Burn,
                (rule!(require(component_caller_badge)), AccessRule::DenyAll),
            );
            access_rules.insert(Recall, (AccessRule::DenyAll, AccessRule::DenyAll));

            // TODO: Pool unit resource metadata - two things are needed to do this:
            // 1- Better APIs for initializing the metadata so that it's not string string.
            // 2- A fix for the issue with references so that we can have the component address of
            //    the pool component in the metadata of the pool unit resource (currently results
            //    in an error because we're passing a reference to a node that doesn't exist).

            ResourceManager::new_fungible(18, Default::default(), access_rules, api)?.0
        };

        let object_id = {
            let vault = Vault::create(resource_address, api)?;
            let substate = SingleResourcePoolSubstate {
                vault: vault.0,
                pool_unit_resource,
                initial_pool_unit_amount: None,
            };
            api.new_simple_object(
                SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
                vec![scrypto_encode(&substate).unwrap()],
            )?
        };
        let access_rules =
            AccessRules::create(authority_rules(pool_manager_rule), btreemap!(), api)?.0;
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => object_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address,
        )?;

        Ok(ComponentAddress::new_or_panic(address.as_node_id().0))
    }

    pub fn contribute<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<SingleResourcePoolContributeOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // No check that the bucket is of the same resource as the vault. This check will be handled
        // by the vault itself on deposit.

        let substate_key = SingleResourcePoolField::SingleResourcePool.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut single_resource_pool_substate =
            api.field_lock_read_typed::<SingleResourcePoolSubstate>(handle)?;

        let mut pool_unit_resource_manager =
            single_resource_pool_substate.pool_unit_resource_manager();
        let mut vault = single_resource_pool_substate.vault();

        let amount_of_resources_in_pool = vault.amount(api)?;
        let amount_of_contributed_resources = bucket.amount(api)?;

        // Case: The pool has had contributions before
        let (is_first_deposit, pool_units_to_mint) = if let Some(first_pool_unit_amount_minted) =
            single_resource_pool_substate.initial_pool_unit_amount
        {
            // Case: If the pool units have all been burned then mint the initial amount upon a
            // contribution
            if pool_unit_resource_manager.total_supply(api)? == Decimal::ZERO {
                (false, first_pool_unit_amount_minted)
            }
            // Case: There are no more resources in this pool - thus the regular calculation would
            // result in a divide by zero error.
            else if amount_of_resources_in_pool == Decimal::ZERO {
                return Err(SingleResourcePoolError::IllegalState.into());
            } else {
                (
                    false,
                    (bucket.amount(api)? / amount_of_resources_in_pool)
                        * pool_unit_resource_manager.total_supply(api)?,
                )
            }
        }
        // Case: This is the first contribution to the pool
        else {
            single_resource_pool_substate.initial_pool_unit_amount =
                Some(amount_of_contributed_resources);
            (true, amount_of_contributed_resources)
        };

        vault.put(bucket, api)?;
        let pool_units = pool_unit_resource_manager.mint_fungible(pool_units_to_mint, api)?;

        if is_first_deposit {
            api.field_lock_write_typed(handle, single_resource_pool_substate)?;
        }
        api.field_lock_release(handle)?;

        Ok(pool_units)
    }

    pub fn redeem<Y>(
        _bucket: Bucket,
        _api: &mut Y,
    ) -> Result<SingleResourcePoolRedeemOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_deposit<Y>(
        _bucket: Bucket,
        _api: &mut Y,
    ) -> Result<SingleResourcePoolProtectedDepositOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn protected_withdraw<Y>(
        _amount: Decimal,
        _api: &mut Y,
    ) -> Result<SingleResourcePoolProtectedWithdrawOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_redemption_value<Y>(
        _amount_of_pool_units: Decimal,
        _api: &mut Y,
    ) -> Result<SingleResourcePoolGetRedemptionValueOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }

    pub fn get_vault_amount<Y>(
        _api: &mut Y,
    ) -> Result<SingleResourcePoolGetVaultAmountOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        todo!()
    }
}

fn authority_rules(pool_manager_rule: AccessRule) -> AuthorityRules {
    let mut authority_rules = AuthorityRules::new();
    /*
    FIXME: When we have a way to map methods to authorities I would like to:

    pool_manager_authority => [
        SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT,
        SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
    ]
    public => all else
     */
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
        pool_manager_rule.clone(),
        pool_manager_rule.clone(),
    );

    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_REDEEM_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );
    authority_rules.set_main_authority_rule(
        SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
        rule!(allow_all),
        rule!(allow_all),
    );

    authority_rules
}

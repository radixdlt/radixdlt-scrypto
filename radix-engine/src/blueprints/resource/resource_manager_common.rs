use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> Roles {
    let mut roles = Roles::new();

    {
        let (mint_access_rule, mint_mutability) = access_rules_map
            .remove(&Mint)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (burn_access_rule, burn_mutability) = access_rules_map
            .remove(&Burn)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
            access_rules_map
                .remove(&UpdateNonFungibleData)
                .unwrap_or((AllowAll, rule!(deny_all)));
        let (update_metadata_access_rule, update_metadata_mutability) = access_rules_map
            .remove(&UpdateMetadata)
            .unwrap_or((DenyAll, rule!(deny_all)));

        {
            roles.define_role(
                SET_METADATA_UPDATE_ROLE,
                RoleEntry::new(update_metadata_mutability, [SET_METADATA_UPDATE_ROLE], true),
            );

            roles.define_role(
                SET_METADATA_ROLE,
                RoleEntry::new(
                    update_metadata_access_rule,
                    [SET_METADATA_UPDATE_ROLE],
                    true,
                ),
            );
        }

        // Mint
        {
            roles.define_role(
                MINT_UPDATE_ROLE,
                RoleEntry::new(mint_mutability, [MINT_UPDATE_ROLE], true),
            );
            roles.define_role(
                MINT_ROLE,
                RoleEntry::new(mint_access_rule, [MINT_UPDATE_ROLE], true),
            );
        }

        // Burn
        {
            roles.define_role(
                BURN_UPDATE_ROLE,
                RoleEntry::new(burn_mutability, [BURN_UPDATE_ROLE], true),
            );
            roles.define_role(
                BURN_ROLE,
                RoleEntry::new(burn_access_rule, [BURN_UPDATE_ROLE], true),
            );
        }

        // Non Fungible Update data
        {
            roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE,
                RoleEntry::new(
                    update_non_fungible_data_mutability,
                    [UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE],
                    true,
                ),
            );

            roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_ROLE,
                RoleEntry::new(
                    update_non_fungible_data_access_rule,
                    [UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE],
                    true,
                ),
            );
        }
    }

    {
        let (deposit_access_rule, deposit_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Deposit)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (withdraw_access_rule, withdraw_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Withdraw)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (recall_access_rule, recall_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Recall)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (freeze_access_rule, freeze_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Freeze)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (unfreeze_access_rule, unfreeze_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Unfreeze)
            .unwrap_or((DenyAll, rule!(deny_all)));

        // Withdraw
        {
            roles.define_role(
                WITHDRAW_ROLE,
                RoleEntry::new(withdraw_access_rule, [WITHDRAW_UPDATE_ROLE], true),
            );
            roles.define_role(
                WITHDRAW_UPDATE_ROLE,
                RoleEntry::new(withdraw_mutability, [WITHDRAW_UPDATE_ROLE], true),
            );
        }

        // Recall
        {
            roles.define_role(
                RECALL_ROLE,
                RoleEntry::new(recall_access_rule, [RECALL_UPDATE_ROLE], true),
            );
            roles.define_role(
                RECALL_UPDATE_ROLE,
                RoleEntry::new(recall_mutability, [RECALL_UPDATE_ROLE], true),
            );
        }

        // Freeze
        {
            roles.define_role(
                FREEZE_ROLE,
                RoleEntry::new(freeze_access_rule, [FREEZE_UPDATE_ROLE], true),
            );
            roles.define_role(
                FREEZE_UPDATE_ROLE,
                RoleEntry::new(freeze_mutability, [FREEZE_UPDATE_ROLE], true),
            );
        }

        // Unfreeze
        {
            roles.define_role(
                UNFREEZE_ROLE,
                RoleEntry::new(unfreeze_access_rule, [UNFREEZE_ROLE], true),
            );
            roles.define_role(
                UNFREEZE_UPDATE_ROLE,
                RoleEntry::new(unfreeze_mutability, [UNFREEZE_UPDATE_ROLE], true),
            );
        }

        // Deposit
        {
            roles.define_role(
                DEPOSIT_ROLE,
                RoleEntry::new(deposit_access_rule, [DEPOSIT_UPDATE_ROLE], true),
            );
            roles.define_role(
                DEPOSIT_UPDATE_ROLE,
                RoleEntry::new(deposit_mutability, [DEPOSIT_UPDATE_ROLE], true),
            );
        }

        // Internal
        {
            roles.define_role(
                "this_package",
                RoleEntry::immutable(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
            );
        }
    }

    roles
}

pub fn globalize_resource_manager<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);
    let resman_access_rules = AccessRules::create(roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    api.globalize_with_address(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
    )?;

    Ok(())
}

pub fn globalize_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);
    let resman_access_rules = AccessRules::create(roles, api)?.0;
    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}

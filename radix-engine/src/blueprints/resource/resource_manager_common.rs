use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataValue, METADATA_SETTER_ROLE, METADATA_SETTER_UPDATER_ROLE,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> BTreeMap<ObjectModuleId, Roles> {
    let mut main_roles = Roles::new();

    // Meta roles
    {
        main_roles.define_role(
            RESOURCE_PACKAGE_ROLE,
            RoleEntry::immutable(require(package_of_direct_caller(RESOURCE_PACKAGE))),
        );
    }

    // Main roles
    {
        // Mint
        let (mint_access_rule, mint_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Mint)
            .unwrap_or((DenyAll, DenyAll));
        {
            main_roles.define_role(
                MINT_UPDATE_ROLE,
                RoleEntry::new(mint_mutability, [MINT_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                MINT_ROLE,
                RoleEntry::new(mint_access_rule, [MINT_UPDATE_ROLE], false),
            );
        }

        // Burn
        let (burn_access_rule, burn_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Burn)
            .unwrap_or((DenyAll, DenyAll));
        {
            main_roles.define_role(
                BURN_UPDATE_ROLE,
                RoleEntry::new(burn_mutability, [BURN_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                BURN_ROLE,
                RoleEntry::new(burn_access_rule, [BURN_UPDATE_ROLE], false),
            );
        }

        // Non Fungible Update data
        let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
            access_rules_map
                .remove(&ResourceMethodAuthKey::UpdateNonFungibleData)
                .unwrap_or((AllowAll, DenyAll));
        {
            main_roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE,
                RoleEntry::new(
                    update_non_fungible_data_mutability,
                    [UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE],
                    false,
                ),
            );

            main_roles.define_role(
                UPDATE_NON_FUNGIBLE_DATA_ROLE,
                RoleEntry::new(
                    update_non_fungible_data_access_rule,
                    [UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE],
                    false,
                ),
            );
        }

        // Withdraw
        let (withdraw_access_rule, withdraw_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Withdraw)
            .unwrap_or((AllowAll, DenyAll));
        {
            main_roles.define_role(
                WITHDRAW_ROLE,
                RoleEntry::new(withdraw_access_rule, [WITHDRAW_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                WITHDRAW_UPDATE_ROLE,
                RoleEntry::new(withdraw_mutability, [WITHDRAW_UPDATE_ROLE], false),
            );
        }

        // Recall
        let (recall_access_rule, recall_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Recall)
            .unwrap_or((DenyAll, DenyAll));
        {
            main_roles.define_role(
                RECALL_ROLE,
                RoleEntry::new(recall_access_rule, [RECALL_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                RECALL_UPDATE_ROLE,
                RoleEntry::new(recall_mutability, [RECALL_UPDATE_ROLE], false),
            );
        }

        // Freeze
        if let Some((freeze_access_rule, freeze_mutability)) =
            access_rules_map.remove(&ResourceMethodAuthKey::Freeze)
        {
            main_roles.define_role(
                FREEZE_ROLE,
                RoleEntry::new(freeze_access_rule, [FREEZE_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                FREEZE_UPDATE_ROLE,
                RoleEntry::new(freeze_mutability, [FREEZE_UPDATE_ROLE], false),
            );
        }

        // Unfreeze
        let (unfreeze_access_rule, unfreeze_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Unfreeze)
            .unwrap_or((DenyAll, DenyAll));
        {
            main_roles.define_role(
                UNFREEZE_ROLE,
                RoleEntry::new(unfreeze_access_rule, [UNFREEZE_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                UNFREEZE_UPDATE_ROLE,
                RoleEntry::new(unfreeze_mutability, [UNFREEZE_UPDATE_ROLE], false),
            );
        }

        // Deposit
        let (deposit_access_rule, deposit_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Deposit)
            .unwrap_or((AllowAll, DenyAll));
        {
            main_roles.define_role(
                DEPOSIT_ROLE,
                RoleEntry::new(deposit_access_rule, [DEPOSIT_UPDATE_ROLE], false),
            );
            main_roles.define_role(
                DEPOSIT_UPDATE_ROLE,
                RoleEntry::new(deposit_mutability, [DEPOSIT_UPDATE_ROLE], false),
            );
        }
    }

    // Metadata
    let (update_metadata_access_rule, update_metadata_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::UpdateMetadata)
        .unwrap_or((DenyAll, DenyAll));
    let metadata_roles = {
        let mut metadata_roles = Roles::new();

        metadata_roles.define_role(
            METADATA_SETTER_ROLE,
            RoleEntry::new(
                update_metadata_access_rule,
                [METADATA_SETTER_UPDATER_ROLE],
                false,
            ),
        );

        metadata_roles.define_role(
            METADATA_SETTER_UPDATER_ROLE,
            RoleEntry::new(
                update_metadata_mutability,
                [METADATA_SETTER_UPDATER_ROLE],
                false,
            ),
        );

        metadata_roles
    };

    btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata_roles,
    )
}

pub fn features(
    track_total_supply: bool,
    access_rules: &BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> Vec<&str> {
    let mut features = Vec::new();

    if track_total_supply {
        features.push(TRACK_TOTAL_SUPPLY_FEATURE);
    }
    if access_rules.contains_key(&ResourceMethodAuthKey::Freeze) {
        features.push(VAULT_FREEZE_FEATURE);
    }
    if access_rules.contains_key(&ResourceMethodAuthKey::Recall) {
        features.push(VAULT_RECALL_FEATURE);
    }
    if access_rules.contains_key(&ResourceMethodAuthKey::Mint) {
        features.push(MINT_FEATURE);
    }

    features
}

pub fn globalize_resource_manager<Y>(
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);
    let resman_access_rules = AccessRules::create(OwnerRole::None, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata, api)?;

    let address = api.globalize(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        Some(resource_address_reservation),
    )?;

    Ok(ResourceAddress::new_or_panic(address.into()))
}

pub fn globalize_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);
    let resman_access_rules = AccessRules::create(OwnerRole::None, roles, api)?.0;
    let metadata = Metadata::create_with_data(metadata, api)?;

    let modules = btreemap!(
        ObjectModuleId::Main => object_id,
        ObjectModuleId::AccessRules => resman_access_rules.0,
        ObjectModuleId::Metadata => metadata.0,
    );

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        modules,
        resource_address_reservation,
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, MetadataValue>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(OwnerRole::None, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata, api)?;

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        resource_address_reservation,
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}

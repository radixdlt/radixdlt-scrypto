use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_main_access_rules(
    mut access_rules_map: BTreeMap<ResourceAction, ResourceActionRoleInit>,
) -> RolesInit {
    let mut main_roles = RolesInit::new();

    // Meta roles
    {
        main_roles.define_immutable_role(
            RESOURCE_PACKAGE_ROLE,
            rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
        );
    }

    // Main roles
    {
        // Mint
        {
            let minter_role_init = access_rules_map
                .remove(&ResourceAction::Mint)
                .unwrap_or(ResourceActionRoleInit::locked(DenyAll));
            main_roles.set_raw2(MINTER_UPDATER_ROLE, minter_role_init.updater);
            main_roles.set_raw2(MINTER_ROLE, minter_role_init.actor);
        }

        // Burn
        {
            let burner_role_init = access_rules_map
                .remove(&ResourceAction::Burn)
                .unwrap_or(ResourceActionRoleInit::locked(DenyAll));
            main_roles.set_raw2(BURNER_UPDATER_ROLE, burner_role_init.updater);
            main_roles.set_raw2(BURNER_ROLE, burner_role_init.actor);
        }

        // Non Fungible Update data
        {
            let non_fungible_data_updater_role_init = access_rules_map
                .remove(&ResourceAction::UpdateNonFungibleData)
                .unwrap_or(ResourceActionRoleInit::locked(DenyAll));

            main_roles.set_raw2(
                NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE,
                non_fungible_data_updater_role_init.updater,
            );
            main_roles.set_raw2(
                NON_FUNGIBLE_DATA_UPDATER_ROLE,
                non_fungible_data_updater_role_init.actor,
            );
        }

        // Withdraw
        {
            let withdrawer_role_init = access_rules_map
                .remove(&ResourceAction::Withdraw)
                .unwrap_or(ResourceActionRoleInit::locked(AllowAll));
            main_roles.set_raw2(WITHDRAWER_ROLE, withdrawer_role_init.actor);
            main_roles.set_raw2(WITHDRAWER_UPDATER_ROLE, withdrawer_role_init.updater);
        }

        // Recall
        {
            let recaller_role_init = access_rules_map
                .remove(&ResourceAction::Recall)
                .unwrap_or(ResourceActionRoleInit::locked(DenyAll));
            main_roles.set_raw2(RECALLER_ROLE, recaller_role_init.actor);
            main_roles.set_raw2(RECALLER_UPDATER_ROLE, recaller_role_init.updater);
        }

        // Freeze/Unfreeze Role
        {
            let freezer_role_init = access_rules_map
                .remove(&ResourceAction::Freeze)
                .unwrap_or(ResourceActionRoleInit::locked(DenyAll));
            main_roles.set_raw2(FREEZER_ROLE, freezer_role_init.actor);
            main_roles.set_raw2(FREEZER_UPDATER_ROLE, freezer_role_init.updater);
        }

        // Deposit
        {
            let depositor_role_init = access_rules_map
                .remove(&ResourceAction::Deposit)
                .unwrap_or(ResourceActionRoleInit::locked(AllowAll));
            main_roles.set_raw2(DEPOSITOR_ROLE, depositor_role_init.actor);
            main_roles.set_raw2(DEPOSITOR_UPDATER_ROLE, depositor_role_init.updater);
        }
    }

    main_roles
}

pub fn features(
    track_total_supply: bool,
    access_rules: &BTreeMap<ResourceAction, ResourceActionRoleInit>,
) -> Vec<&str> {
    let mut features = Vec::new();

    if track_total_supply {
        features.push(TRACK_TOTAL_SUPPLY_FEATURE);
    }

    if access_rules.contains_key(&ResourceAction::Freeze) {
        features.push(VAULT_FREEZE_FEATURE);
    }

    if access_rules.contains_key(&ResourceAction::Recall) {
        features.push(VAULT_RECALL_FEATURE);
    }

    if access_rules.contains_key(&ResourceAction::Mint) {
        features.push(MINT_FEATURE);
    }

    if access_rules.contains_key(&ResourceAction::Burn) {
        features.push(BURN_FEATURE);
    }

    features
}

pub fn globalize_resource_manager<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, ResourceActionRoleInit>,
    metadata: ModuleConfig<MetadataInit>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);

    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );

    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

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
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, ResourceActionRoleInit>,
    metadata: ModuleConfig<MetadataInit>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);
    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;
    let metadata = Metadata::create_with_data(metadata.init, api)?;

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
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, ResourceActionRoleInit>,
    metadata: ModuleConfig<MetadataInit>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);
    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

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

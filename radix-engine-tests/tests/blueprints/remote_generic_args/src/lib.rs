use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct Type1 {
    a: String,
}

#[derive(ScryptoSbor)]
struct Type2 {
    b: u32,
}

#[blueprint]
#[experimental_types(Type1)]
mod non_fungible_data {
    struct NFD {}

    impl NFD {
        pub fn create_non_fungible_resource_with_remote_type(
            package_address: PackageAddress,
            blueprint_name: String,
            type_name: String,
        ) {
            let non_fungible_schema = NonFungibleDataSchema::new_remote(
                BlueprintTypeId {
                    package_address,
                    blueprint_name,
                    type_name,
                },
                Vec::<String>::new(),
            );

            ScryptoVmV1Api::blueprint_call(
                RESOURCE_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&NonFungibleResourceManagerCreateWithInitialSupplyInput {
                    owner_role: Default::default(),
                    track_total_supply: true,
                    id_type: IntegerNonFungibleLocalId::id_type(),
                    non_fungible_schema,
                    resource_roles: Default::default(),
                    metadata: Default::default(),
                    entries: Default::default(),
                    address_reservation: Default::default(),
                })
                .unwrap(),
            );
        }
    }
}

#[blueprint]
#[experimental_types(Type2)]
mod key_value_store {
    struct KVS {
        kv_store: Own,
    }

    impl KVS {
        pub fn create_key_value_store_with_remote_type(
            package_address: PackageAddress,
            blueprint_name: String,
            type_name: String,
        ) {
            let kv_store = ScryptoVmV1Api::kv_store_new(KeyValueStoreDataSchema::Remote {
                key_type: BlueprintTypeId {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                value_type: BlueprintTypeId {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                allow_ownership: false,
            });

            Self {
                kv_store: Own(kv_store),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}

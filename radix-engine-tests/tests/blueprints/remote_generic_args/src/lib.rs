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
#[types(Type1)]
mod non_fungible_data {
    struct NFD {
        vault: Vault,
    }

    impl NFD {
        pub fn create_non_fungible_resource_with_remote_type(
            package_address: PackageAddress,
            blueprint_name: String,
            type_name: String,
        ) {
            let non_fungible_schema = NonFungibleDataSchema::new_remote(
                BlueprintTypeIdentifier {
                    package_address,
                    blueprint_name,
                    type_name,
                },
                Vec::<String>::new(),
            );

            let bytes = ScryptoVmV1Api::blueprint_call(
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
                    entries: indexmap!(
                        NonFungibleLocalId::integer(5) => (scrypto_decode(&scrypto_encode(&Type1{a: "a".to_string()}).unwrap()).unwrap(),)
                    ),
                    address_reservation: Default::default(),
                })
                .unwrap(),
            );

            let bucket = scrypto_decode::<(ResourceAddress, NonFungibleBucket)>(&bytes)
                .unwrap()
                .1;
            Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}

#[blueprint]
#[types(Type2)]
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
            // Create
            let kv_store = ScryptoVmV1Api::kv_store_new(KeyValueStoreDataSchema::Remote {
                key_type: BlueprintTypeIdentifier {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                value_type: BlueprintTypeIdentifier {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                allow_ownership: false,
            });

            // Insert
            let handle = ScryptoVmV1Api::kv_store_open_entry(
                &kv_store,
                &scrypto_encode(&Type2 { b: 1 }).unwrap(),
                LockFlags::MUTABLE,
            );
            ScryptoVmV1Api::kv_entry_write(handle, scrypto_encode(&Type2 { b: 1 }).unwrap());
            ScryptoVmV1Api::kv_entry_close(handle);

            Self {
                kv_store: Own(kv_store),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}

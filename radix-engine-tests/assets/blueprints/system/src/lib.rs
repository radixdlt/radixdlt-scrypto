use scrypto::prelude::*;

#[blueprint]
mod recursive_test {
    struct HandleMismatchTest {}

    impl HandleMismatchTest {
        pub fn new() -> Global<HandleMismatchTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn treat_field_handle_as_kv_store_handle(&self) {
            let lock_handle =
                ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only());

            ScryptoVmV1Api::kv_entry_remove(lock_handle);
        }
    }
}

#[blueprint]
mod address_reservation_test {
    struct AddressReservationTest {
        own: ScryptoValue,
    }

    impl AddressReservationTest {
        pub fn drop_address_reservation(_reservation: GlobalAddressReservation) {
            // No longer works as object dropping API has been removed for WASM
        }

        pub fn put_address_reservation_into_component_state(reservation: GlobalAddressReservation) {
            Self {
                own: scrypto_decode(&scrypto_encode(&reservation).unwrap()).unwrap(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn put_address_reservation_into_kv_store(reservation: GlobalAddressReservation) {
            let kv_store = KeyValueStore::<u32, ScryptoValue>::new();
            kv_store.insert(
                1u32,
                scrypto_decode(&scrypto_encode(&reservation).unwrap()).unwrap(),
            );

            Self {
                own: scrypto_decode(&scrypto_encode(&kv_store).unwrap()).unwrap(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn globalize_address_reservation(reservation: GlobalAddressReservation) {
            ScryptoVmV1Api::object_globalize(
                reservation.0.as_node_id().clone(),
                index_map_new(),
                None,
            );
        }
    }
}

#[blueprint]
mod write_after_locking_field_substate_test {
    struct WriteAfterLockingTest {}

    impl WriteAfterLockingTest {
        // Currently, substate locking API isn't exposed to Scrypto, so testing through native blueprints or object modules.

        pub fn write_after_locking_field_substate() {
            let owner_role = OwnerRole::Updatable(rule!(allow_all));
            let global = WriteAfterLockingTest {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .globalize();

            global.lock_owner_role();

            global.set_owner_role(rule!(deny_all));
        }

        pub fn write_after_locking_key_value_store_entry() {
            let owner_role = OwnerRole::Updatable(rule!(allow_all));
            let global = WriteAfterLockingTest {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .globalize();

            global.set_metadata("key", "value".to_owned());
            global.lock_metadata("key");
            global.set_metadata("key", "value2".to_owned());
        }

        pub fn write_after_locking_key_value_collection_entry() {
            let bucket = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "Katz's Sandwiches".to_owned(), locked;
                    }
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => rule!(allow_all);
                    non_fungible_data_updater_updater => rule!(allow_all);
                })
                .mint_initial_supply([Sandwich {
                    name: "Zero".to_owned(),
                    available: true,
                    tastes_great: true,
                    reference: None,
                    own: None,
                }]);
            let non_fungible_local_id = bucket.non_fungible_local_id();
            let resource_manager = bucket.resource_manager();

            bucket.burn();

            resource_manager.update_non_fungible_data(&non_fungible_local_id, "available", false);
        }
    }
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[mutable]
    pub available: bool,
    pub tastes_great: bool,
    #[mutable]
    pub reference: Option<ComponentAddress>,
    #[mutable]
    pub own: Option<Own>,
}

#[blueprint]
mod role_assignment_of_role_assignment {
    struct RoleAndRole {}

    impl RoleAndRole {
        pub fn set_role_of_role_assignment() {
            let mut init = RoleAssignmentInit::new();
            init.define_role("test", rule!(allow_all));

            let role_assignment = RoleAssignment::new(
                OwnerRole::Updatable(rule!(allow_all)),
                indexmap!(
                    ModuleId::RoleAssignment => init,
                ),
            );

            role_assignment.set_role_assignment_role("test", rule!(deny_all));
        }

        pub fn set_role_of_role_assignment_v2() {
            let role_assignment =
                RoleAssignment::new(OwnerRole::Updatable(rule!(allow_all)), indexmap!());

            role_assignment.set_role_assignment_role("_reserved_key", rule!(deny_all));

            // Clean up
            let object = RoleAndRole {}.instantiate();
            let metadata = Metadata::new();
            ScryptoVmV1Api::object_globalize(
                object.0.handle.as_node_id().clone(),
                indexmap!(
                    AttachedModuleId::RoleAssignment => role_assignment.0.as_node_id().clone(),
                    AttachedModuleId::Metadata => metadata.0.as_node_id().clone()
                ),
                None,
            );
        }

        pub fn call_role_assignment_method_of_role_assignment() {
            let role_assignment =
                RoleAssignment::new(OwnerRole::Updatable(rule!(allow_all)), indexmap!());

            ScryptoVmV1Api::object_call_module(
                role_assignment.0.as_node_id(),
                AttachedModuleId::RoleAssignment,
                ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
                scrypto_encode(&()).unwrap(),
            );

            // Clean up
            let object = RoleAndRole {}.instantiate();
            let metadata = Metadata::new();
            ScryptoVmV1Api::object_globalize(
                object.0.handle.as_node_id().clone(),
                indexmap!(
                    AttachedModuleId::RoleAssignment => role_assignment.0.as_node_id().clone(),
                    AttachedModuleId::Metadata => metadata.0.as_node_id().clone()
                ),
                None,
            );
        }
    }
}

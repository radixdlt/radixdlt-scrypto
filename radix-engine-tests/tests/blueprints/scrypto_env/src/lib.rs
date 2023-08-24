use scrypto::api::node_modules::royalty::COMPONENT_ROYALTY_SETTER_ROLE;
use scrypto::prelude::*;

#[blueprint]
mod scrypto_env_test {
    struct ScryptoEnvTest {}

    impl ScryptoEnvTest {
        pub fn create_node_with_invalid_blueprint() {
            ScryptoVmV1Api::object_new(
                "invalid_blueprint",
                btreemap![0u8 => FieldValue::new(&ScryptoEnvTest {})],
            );
        }

        pub fn create_and_open_mut_substate_twice(heap: bool) {
            let obj = Self {}.instantiate();
            if heap {
                obj.open_mut_substate_twice();
                obj.prepare_to_globalize(OwnerRole::None).globalize();
            } else {
                let globalized = obj.prepare_to_globalize(OwnerRole::None).globalize();
                globalized.open_mut_substate_twice();
            }
        }

        pub fn open_mut_substate_twice(&mut self) {
            ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE);

            ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE);
        }

        pub fn bech32_encode_address(address: ComponentAddress) -> String {
            Runtime::bech32_encode_address(address)
        }
    }
}

#[blueprint]
mod component_role_rule {
    struct ComponentRoleRuleTest {}

    impl ComponentRoleRuleTest {
        pub fn query_role_rules() -> (Option<AccessRule>, Option<AccessRule>, Option<AccessRule>) {
            let global = Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            (
                global.role_assignment().get_role("no_existent"),
                global
                    .role_assignment()
                    .get_metadata_role(METADATA_SETTER_ROLE),
                global
                    .role_assignment()
                    .get_component_royalties_role(COMPONENT_ROYALTY_SETTER_ROLE),
            )
        }
    }
}

#[blueprint]
mod local_auth_zone {
    struct LocalAuthZoneTest {}

    impl LocalAuthZoneTest {
        pub fn pop_empty_auth_zone() -> Option<Proof> {
            LocalAuthZone::pop()
        }
    }
}

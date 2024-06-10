#[cfg(test)]
mod tests {
    use radix_common::prelude::*;
    use radix_engine_interface::prelude::*;
    use sbor::validate_payload_against_schema;

    #[test]
    fn test_custom_type_values_are_valid() {
        // These tests continue tests from the definition of scrypto's well-known types in `custom_well_known_types.rs`
        // in the `radix-common` crate.
        // In particular, we only test types here which are only actually fully defined in `radix-engine-interface`.

        // MISC
        let nf_global_id = NonFungibleGlobalId::from_public_key(&PublicKey::Ed25519(
            Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
        ));
        test_equivalence(NON_FUNGIBLE_GLOBAL_ID_TYPE, nf_global_id.clone());
        test_equivalence(URL_TYPE, UncheckedUrl::of("https://example.com"));
        test_equivalence(ORIGIN_TYPE, UncheckedOrigin::of("example.com"));

        // ROLE ASSIGNMENT
        let resource_or_non_fungible_1 = ResourceOrNonFungible::Resource(XRD);
        let resource_or_non_fungible_2 = ResourceOrNonFungible::NonFungible(nf_global_id);
        let resource_or_non_fungible_list = vec![
            resource_or_non_fungible_1.clone(),
            resource_or_non_fungible_2.clone(),
        ];
        let basic_requirement = BasicRequirement::Require(resource_or_non_fungible_1.clone());
        let composite_requirement =
            CompositeRequirement::BasicRequirement(basic_requirement.clone());
        let composite_requirement_list = vec![composite_requirement.clone()];
        let access_rule = AccessRule::Protected(composite_requirement.clone());

        test_equivalence(ACCESS_RULE_TYPE, access_rule);
        test_equivalence(COMPOSITE_REQUIREMENT_TYPE, composite_requirement);
        test_statically_valid(COMPOSITE_REQUIREMENT_LIST_TYPE, composite_requirement_list);
        test_equivalence(BASIC_REQUIREMENT_TYPE, basic_requirement);
        test_equivalence(RESOURCE_OR_NON_FUNGIBLE_TYPE, resource_or_non_fungible_1);
        test_equivalence(RESOURCE_OR_NON_FUNGIBLE_TYPE, resource_or_non_fungible_2);
        test_statically_valid(
            RESOURCE_OR_NON_FUNGIBLE_LIST_TYPE,
            resource_or_non_fungible_list,
        );
        test_equivalence(OWNER_ROLE_TYPE, OwnerRole::None);
        test_equivalence(ROLE_KEY_TYPE, RoleKey::from("MyRoleName"));

        // OTHER MODULE TYPES
        test_equivalence(MODULE_ID_TYPE, ModuleId::Main);
        test_equivalence(ATTACHED_MODULE_ID_TYPE, AttachedModuleId::Metadata);
        test_equivalence(ROYALTY_AMOUNT_TYPE, RoyaltyAmount::Free);
        test_equivalence(ROYALTY_AMOUNT_TYPE, RoyaltyAmount::Usd(dec!("1.6")));
        test_equivalence(ROYALTY_AMOUNT_TYPE, RoyaltyAmount::Xrd(dec!("1.6")));
    }

    fn test_equivalence<T: ScryptoEncode + ScryptoDescribe>(id: WellKnownTypeId, value: T) {
        test_type_data_equivalent::<T>(id);
        test_statically_valid(id, value);
    }

    fn test_statically_valid<T: ScryptoEncode>(id: WellKnownTypeId, value: T) {
        let type_name = core::any::type_name::<T>();

        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            &scrypto_encode(&value).unwrap(),
            &ScryptoCustomSchema::empty_schema(),
            id.into(),
            &(),
            10,
        )
        .unwrap_or_else(|err| {
            panic!("Expected value for {type_name} to match well known type but got: {err:?}")
        });
    }

    fn test_type_data_equivalent<T: ScryptoDescribe>(id: WellKnownTypeId) {
        let type_name = core::any::type_name::<T>();

        assert_eq!(T::TYPE_ID, RustTypeId::from(id), "The ScryptoDescribe impl for {type_name} has a TYPE_ID which does not equal its well known type id");
        let localized_type_data =
            localize_well_known_type_data::<ScryptoCustomSchema>(T::type_data());
        let resolved = resolve_scrypto_well_known_type(id)
            .unwrap_or_else(|| panic!("Well known index for {type_name} not found in lookup"));
        assert_eq!(&localized_type_data, resolved, "The ScryptoDescribe impl for {type_name} has type data which does not equal its well known type data");
    }
}

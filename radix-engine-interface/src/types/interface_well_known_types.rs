#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;
    use sbor::validate_payload_against_schema;

    #[test]
    fn test_custom_type_values_are_valid() {
        // These tests continue tests from the definition of scrypto's well-known types in `custom_well_known_types.rs`
        // in the `radix-engine-common` crate.
        // In particular, we only test types here which are only actually fully defined in `radix-engine-interface`.
        test_statically_valid(
            NON_FUNGIBLE_GLOBAL_ID_TYPE,
            NonFungibleGlobalId::from_public_key(&PublicKey::Ed25519(Ed25519PublicKey(
                [0; Ed25519PublicKey::LENGTH],
            ))),
        );
        test_statically_valid(URL_TYPE, UncheckedUrl::of("https://example.com"));
        test_statically_valid(ORIGIN_TYPE, UncheckedOrigin::of("example.com"));
    }

    fn test_statically_valid<T: ScryptoEncode + ScryptoDescribe>(id: WellKnownTypeId, value: T) {
        let type_name = core::any::type_name::<T>();
        // First - validate payload against well known type index
        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            &scrypto_encode(&value).unwrap(),
            &ScryptoCustomSchema::empty_schema(),
            id.into(),
            &(),
            10,
        )
        .unwrap_or_else(|err| {
            panic!("Expected well known index for {type_name} to be valid: {err:?}")
        });

        // Second - check that the type's impl is using the well known type index
        assert_eq!(T::TYPE_ID, GlobalTypeId::from(id), "The ScryptoDescribe impl for {type_name} has a TYPE_ID which does not equal its well known type id");
        let localized_type_data =
            localize_well_known_type_data::<ScryptoCustomSchema>(T::type_data());
        let resolved = resolve_scrypto_well_known_type(id)
            .unwrap_or_else(|| panic!("Well known index for {type_name} not found in lookup"));
        assert_eq!(&localized_type_data, resolved, "The ScryptoDescribe impl for {type_name} has type data which does not equal its well known type data");
    }
}

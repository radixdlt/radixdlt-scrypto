use radix_common::network::NetworkDefinition;
use radix_transactions::manifest::*;

macro_rules! check_manifest {
    ($manifest_kind:expr, $manifest:expr, $blob_provider:expr $(,)?) => {{
        let manifest = include_str!(concat!("assets/", $manifest, ".rtm"));
        let diagnostic = include_str!(concat!("assets/", $manifest, ".diag"));

        let x = compile_any_manifest_with_pretty_error(
            manifest,
            $manifest_kind,
            &NetworkDefinition::simulator(),
            $blob_provider,
            CompileErrorDiagnosticsStyle::PlainText,
        )
        .unwrap_err();

        if x != diagnostic {
            let path = format!("tests/assets/{}.diag.res", $manifest);

            std::fs::write(&path, &x).expect("Unable to write file");

            eprintln!("expected diagnostic report:\n{}", &diagnostic);
            eprintln!(
                "current diagnostic report (also available in {}):\n{}",
                path, &x
            );
            panic!("diagnostic reports differ");
        }
    }};
    ($manifest_kind:expr, $manifest:expr $(,)?) => {{
        check_manifest!(
            $manifest_kind,
            $manifest,
            // The MockBlobProvider pretends any blob is valid
            MockBlobProvider::default()
        )
    }};
    ($manifest:expr $(,)?) => {{
        check_manifest!(
            ManifestKind::V1,
            $manifest,
            // The MockBlobProvider pretends any blob is valid
            MockBlobProvider::default()
        )
    }};
}

// When adding new manifest *.rtm file to test, you can create empty *.diag file.
// Then run the test, which will fail and create *.diag.res file, which can be further
// renamed to *.diag
#[test]
fn test_manifest_parser_error_diagnostics_unexpected_token() {
    // UnexpectedToken
    check_manifest!("manifest_parser_error_unexpected_token_1");
    check_manifest!("manifest_parser_error_unexpected_token_2");
    check_manifest!("manifest_parser_error_unexpected_token_3");
    check_manifest!("manifest_parser_error_unexpected_token_4");
    check_manifest!("manifest_parser_error_unexpected_token_5");
    check_manifest!("manifest_parser_error_unexpected_token_6");
    check_manifest!("manifest_parser_error_unexpected_token_7");
    check_manifest!("manifest_parser_error_unexpected_token_8");
}

#[test]
fn test_manifest_parser_error_diagnostics_invalid_argument() {
    // InvalidArgument
    check_manifest!("manifest_parser_error_invalid_argument_1");
    check_manifest!("manifest_parser_error_invalid_argument_2");
}

#[test]
fn test_manifest_parser_error_diagnostics_invalid_number_of_types() {
    // InvalidNumberOfTypes
    check_manifest!("manifest_parser_error_invalid_number_of_types_1");
    check_manifest!("manifest_parser_error_invalid_number_of_types_2");
    check_manifest!("manifest_parser_error_invalid_number_of_types_3");
    check_manifest!("manifest_parser_error_invalid_number_of_types_4");
}

#[test]
fn test_manifest_parser_error_diagnostics_invalid_number_of_values() {
    // InvalidNumberOfValues
    check_manifest!("manifest_parser_error_invalid_number_of_values_1");
    check_manifest!("manifest_parser_error_invalid_number_of_values_2");
    check_manifest!("manifest_parser_error_invalid_number_of_values_3");
}

#[test]
fn test_manifest_parser_error_diagnostics_unexpected_eof() {
    // UnexpectedEof
    check_manifest!("manifest_parser_error_unexpected_eof_1");
    check_manifest!("manifest_parser_error_unexpected_eof_2");
    check_manifest!("manifest_parser_error_unexpected_eof_3");
}

#[test]
fn test_manifest_parser_error_diagnostics_unknown_enum_discriminator() {
    // UnknownEnumDiscriminator
    check_manifest!("manifest_parser_error_unknown_enum_discriminator_1");
    check_manifest!("manifest_parser_error_unknown_enum_discriminator_2");
}

#[test]
fn test_manifest_parser_error_diagnostics_max_depth_exceeded() {
    // MaxDepthExceeded
    check_manifest!("manifest_parser_error_max_depth_exceeded_1");
    check_manifest!("manifest_parser_error_max_depth_exceeded_2");
}

#[test]
fn test_manifest_lexer_error_unexpected_char() {
    // UnexpectedChar
    check_manifest!("manifest_lexer_error_unexpected_char_1");
    check_manifest!("manifest_lexer_error_unexpected_char_2");
    check_manifest!("manifest_lexer_error_unexpected_char_3");
    check_manifest!("manifest_lexer_error_unexpected_char_4");
    check_manifest!("manifest_lexer_error_unexpected_char_5");
}

#[test]
fn test_manifest_lexer_error_invalid_integer_literal() {
    // InvalidIntegerLiteral
    check_manifest!("manifest_lexer_error_invalid_integer_literal_1");
}

#[test]
fn test_manifest_lexer_error_invalid_integer_type() {
    // InvalidIntegerType
    check_manifest!("manifest_lexer_error_invalid_integer_type_1");
    check_manifest!("manifest_lexer_error_invalid_integer_type_2");
}

#[test]
fn test_manifest_lexer_error_invalid_integer() {
    // InvalidInteger
    check_manifest!("manifest_lexer_error_invalid_integer_1");
    check_manifest!("manifest_lexer_error_invalid_integer_2");
    check_manifest!("manifest_lexer_error_invalid_integer_3");
}

#[test]
fn test_manifest_lexer_error_invalid_unicode() {
    // InvalidUnicode
    check_manifest!("manifest_lexer_error_invalid_unicode_1");
}

#[test]
fn test_manifest_lexer_error_missing_unicode_surrogate() {
    // MissingUnicodeSurrogate
    check_manifest!("manifest_lexer_error_missing_unicode_surrogate_1");
}

#[test]
fn test_manifest_lexer_error_diagnostics_unexpected_eof() {
    // UnexpectedEof
    check_manifest!("manifest_lexer_error_unexpected_eof_1");
    check_manifest!("manifest_lexer_error_unexpected_eof_2");
}

#[test]
fn test_manifest_generator_error_invalid_ast_value() {
    // InvalidAstValue
    check_manifest!("manifest_generator_error_invalid_ast_value_1");
    check_manifest!("manifest_generator_error_invalid_ast_value_2");
    check_manifest!("manifest_generator_error_invalid_ast_value_3");
    check_manifest!("manifest_generator_error_invalid_ast_value_4");
    check_manifest!("manifest_generator_error_invalid_ast_value_5");
}

#[test]
fn test_manifest_generator_error_invalid_ast_type() {
    // InvalidAstType
    check_manifest!("manifest_generator_error_invalid_ast_type_1");
}

#[test]
fn test_manifest_generator_error_unexpected_value() {
    // UnexpectedValue
    check_manifest!("manifest_generator_error_unexpected_value_1");
    check_manifest!("manifest_generator_error_unexpected_value_2");
}

#[test]
fn test_manifest_generator_error_invalid_decimal() {
    // InvalidDecimal
    check_manifest!("manifest_generator_error_invalid_decimal_1");
    check_manifest!("manifest_generator_error_invalid_decimal_2");
    check_manifest!("manifest_generator_error_invalid_decimal_3");
}

#[test]
fn test_manifest_generator_error_invalid_precise_decimal() {
    // InvalidPreciseDecimal
    check_manifest!("manifest_generator_error_invalid_precise_decimal_1");
}

#[test]
fn test_manifest_generator_error_invalid_expression() {
    // InvalidExpression
    check_manifest!("manifest_generator_error_invalid_expression_1");
}

#[test]
fn test_manifest_generator_error_invalid_non_fungible_local_id() {
    // InvalidNonFungibleLocalId
    check_manifest!("manifest_generator_error_invalid_non_fungible_local_id_1");
}

#[test]
fn test_manifest_generator_error_invalid_non_fungible_global_id() {
    // InvalidNonFungibleGlobalId
    check_manifest!("manifest_generator_error_invalid_non_fungible_global_id_1");
}

#[test]
fn test_manifest_generator_error_invalid_blob_hash() {
    // InvalidBlobHash
    check_manifest!("manifest_generator_error_invalid_blob_hash_1");
    check_manifest!("manifest_generator_error_invalid_blob_hash_2");
}

#[test]
fn test_manifest_generator_error_blob_not_found() {
    // BlobNotFound
    check_manifest!(
        ManifestKind::V1,
        "manifest_generator_error_blob_not_found_1",
        BlobProvider::default()
    );
}

#[test]
fn test_manifest_generator_error_invalid_bytes_hex() {
    // InvalidBytesHex
    check_manifest!("manifest_generator_error_invalid_bytes_hex_1");
}

#[test]
fn test_manifest_generator_error_invalid_global_address() {
    // InvalidGlobalAddress
    check_manifest!("manifest_generator_error_invalid_global_address_1");
}

#[test]
fn test_manifest_generator_error_invalid_package_address() {
    // InvalidPackageAddress
    check_manifest!("manifest_generator_error_invalid_package_address_1");
    check_manifest!("manifest_generator_error_invalid_package_address_2");
}

#[test]
fn test_manifest_generator_error_invalid_resource_address() {
    // InvalidResourceAddress
    check_manifest!("manifest_generator_error_invalid_resource_address_1");
}

#[test]
fn test_manifest_generator_error_invalid_internal_address_1() {
    // InvalidInternalAddress
    check_manifest!("manifest_generator_error_invalid_internal_address_1");
}

#[test]
fn test_manifest_generator_error_undefined_address_reservation() {
    // NameResolverError(UndefinedAddressReservation)
    check_manifest!("manifest_generator_error_undefined_address_reservation_1");
}

#[test]
fn test_manifest_generator_error_undefined_named_address() {
    // NameResolverError(UndefinedNamedAddress)
    check_manifest!("manifest_generator_error_undefined_named_address_1");
}

#[test]
fn test_manifest_generator_error_name_already_defined() {
    // NameResolverError(UndefinedNamedAddress)
    check_manifest!("manifest_generator_error_name_already_defined_1");
    check_manifest!("manifest_generator_error_name_already_defined_2");
    check_manifest!("manifest_generator_error_name_already_defined_3");
    check_manifest!("manifest_generator_error_name_already_defined_4");
}

#[test]
fn test_manifest_generator_error_undefined_bucket() {
    // NameResolverError(UndefinedBucket)
    check_manifest!("manifest_generator_error_undefined_bucket_1");
}

#[test]
fn test_manifest_generator_error_undefined_proof() {
    // NameResolverError(UndefinedBucket)
    check_manifest!("manifest_generator_error_undefined_proof_1");
}

#[test]
fn test_manifest_generator_error_bucket_not_found() {
    // IdValidationError(BucketNotFound)
    check_manifest!("manifest_generator_error_bucket_not_found_1");
    check_manifest!("manifest_generator_error_bucket_not_found_2");
}

#[test]
fn test_manifest_generator_error_bucket_locked() {
    // IdValidationError(BucketLocked)
    check_manifest!("manifest_generator_error_bucket_locked_1");
}

#[test]
fn test_manifest_generator_error_proof_not_found() {
    // IdValidationError(BucketNotFound)
    check_manifest!("manifest_generator_error_proof_not_found_1");
    check_manifest!("manifest_generator_error_proof_not_found_2");
}

#[test]
fn test_manifest_generator_error_invalid_sub_transaction_id() {
    // InvalidSubTransactionId(String)
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_invalid_sub_transaction_id_1"
    );
}

#[test]
fn test_manifest_generator_error_instruction_not_supported_in_manifest_version() {
    // InstructionNotSupportedInManifestVersion
    check_manifest!(
        ManifestKind::V1,
        "manifest_generator_error_instruction_not_supported_in_manifest_version_1"
    );
}

#[test]
fn test_manifest_generator_error_duplicate_subintent_hash() {
    // ManifestBuildError(ManifestBuildError::DuplicateChildSubintentHash)
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_duplicate_subintent_hash_1"
    );
}

#[test]
fn test_manifest_generator_error_child_subintents_unsupported_by_manifest_type() {
    // ManifestBuildError(ManifestBuildError::ChildSubintentsUnsupportedByManifestType)
    check_manifest!(
        ManifestKind::V1,
        "manifest_generator_error_child_subintents_unsupported_by_manifest_type_1"
    );
}

#[test]
fn test_manifest_generator_error_preallocated_addresses_unsupported_by_manifest_type() {
    // ManifestBuildError(ManifestBuildError::PreallocatedAddressesUnsupportedByManifestType)
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_preallocated_addresses_unsupported_by_manifest_type_1"
    );
}

#[test]
fn test_manifest_generator_error_header_instruction_must_come_first() {
    // HeaderInstructionMustComeFirst
    check_manifest!(
        ManifestKind::SubintentV2,
        "manifest_generator_error_header_instruction_must_come_first_1"
    );
}

#[test]
fn test_manifest_generator_error_intent_cannot_be_used_in_value() {
    // IntentCannotBeUsedInValue
    check_manifest!(
        ManifestKind::SubintentV2,
        "manifest_generator_error_intent_cannot_be_used_in_value_1"
    );
}

#[test]
fn test_manifest_generator_error_intent_cannot_be_used_as_value_kind() {
    // IntentCannotBeUsedAsValueKind
    check_manifest!(
        ManifestKind::SubintentV2,
        "manifest_generator_error_intent_cannot_be_used_as_value_kind_1"
    );
}

#[test]
fn test_manifest_generator_error_named_intent_cannot_be_used_in_value() {
    // NamedIntentCannotBeUsedInValue
    check_manifest!(
        ManifestKind::SubintentV2,
        "manifest_generator_error_named_intent_cannot_be_used_in_value_1"
    );
}

#[test]
fn test_manifest_generator_error_named_intent_cannot_be_used_as_value_kind() {
    // NamedIntentCannotBeUsedAsValueKind
    check_manifest!(
        ManifestKind::SubintentV2,
        "manifest_generator_error_named_intent_cannot_be_used_as_value_kind_1"
    );
}

#[test]
fn test_manifest_generator_error_argument_could_not_be_read_as_expected_type() {
    // ArgumentCouldNotBeReadAsExpectedType { type_name: String, error_message: String, },
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_argument_could_not_be_read_as_expected_type_1"
    );
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_argument_could_not_be_read_as_expected_type_2"
    );
    check_manifest!(
        ManifestKind::V2,
        "manifest_generator_error_argument_could_not_be_read_as_expected_type_3"
    );
}

#[test]
fn test_manifest_compiler_error_plain_text() {
    check_manifest!(
        ManifestKind::V1,
        "manifest_compiler_error_plain_text_1",
        BlobProvider::default(),
    );
}

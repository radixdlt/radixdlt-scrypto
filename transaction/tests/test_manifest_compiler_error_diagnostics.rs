use radix_engine_common::network::NetworkDefinition;
use transaction::manifest::blob_provider::*;
use transaction::manifest::compiler::*;

macro_rules! check_manifest {
    ( $manifest:expr) => {{
        let manifest = include_str!(concat!($manifest, ".rtm"));
        let diagnostic = include_str!(concat!($manifest, ".diag"));

        let err = compile(
            manifest,
            &NetworkDefinition::simulator(),
            BlobProvider::default(),
        )
        .unwrap_err();

        let x = compile_error_diagnostics(manifest, err);

        if x != diagnostic {
            std::fs::write(format!("tests/{}.diag.res", $manifest), &x)
                .expect("Unable to write file");
        }

        assert_eq!(x, diagnostic);
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
}

#[test]
fn test_manifest_parser_error_diagnostics_unexpected_token_or_missing_semicolon() {
    // UnexpectedTokenOrMissingSemicolon
    check_manifest!("manifest_parser_error_unexpected_token_or_missing_semicolon_1");
    check_manifest!("manifest_parser_error_unexpected_token_or_missing_semicolon_2");
}

#[test]
fn test_manifest_parser_error_diagnostics_invalid_number_of_types() {
    // InvalidNumberOfTypes
    check_manifest!("manifest_parser_error_invalid_number_of_types_1");
    check_manifest!("manifest_parser_error_invalid_number_of_types_2");
}

#[test]
fn test_manifest_parser_error_diagnostics_invalid_number_of_values() {
    // InvalidNumberOfValues
    check_manifest!("manifest_parser_error_invalid_number_of_values_1");
    check_manifest!("manifest_parser_error_invalid_number_of_values_2");
}

#[test]
fn test_manifest_parser_error_diagnostics_unexpected_eof() {
    // UnexpectedEof
    check_manifest!("manifest_parser_error_unexpected_eof_1");
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
fn test_manifest_generator_error_invalid_expression() {
    // InvalidExpression
    check_manifest!("manifest_generator_error_invalid_expression_1");
}

#[test]
fn test_manifest_generator_error_invalid_global_address() {
    // InvalidGlobalAddress
    check_manifest!("manifest_generator_error_invalid_global_address_1");
}

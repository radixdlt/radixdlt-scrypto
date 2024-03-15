//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known SCHEMA is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_engine_common::prelude::*;
use radix_engine_interface::prelude::*;
use transaction::data::{from_decimal, from_non_fungible_local_id, from_precise_decimal};
use transaction::prelude::*;

use super::{parse_resource_specifier, ResourceSpecifier};

/// Represents an error when building a transaction.
#[derive(Debug, Clone)]
pub enum BuildCallInstructionError {
    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match SCHEMA.
    FailedToBuildArguments(BuildCallArgumentsError),

    /// Failed to export the SCHEMA of a function.
    FailedToExportFunctionSchema(PackageAddress, String, String),

    /// Failed to export the SCHEMA of a method.
    FailedToExportMethodSchema(ComponentAddress, String),

    /// Account is required but not provided.
    AccountNotProvided,
}

/// Represents an error when parsing arguments.
#[derive(Debug, Clone)]
pub enum BuildCallArgumentsError {
    WrongNumberOfArguments(usize, usize),
    BuildCallArgumentError(BuildCallArgumentError),
    RustToManifestValueError(RustToManifestValueError),
}

/// Represents an error when parsing an argument.
#[derive(Debug, Clone)]
pub enum BuildCallArgumentError {
    /// The argument is of unsupported type.
    UnsupportedType(ScryptoTypeKind<LocalTypeId>),

    /// Failure when parsing an argument.
    FailedToParse(String),

    /// Failed to interpret this string as a resource specifier
    InvalidResourceSpecifier(String),
}

impl From<BuildCallArgumentsError> for BuildCallInstructionError {
    fn from(error: BuildCallArgumentsError) -> Self {
        Self::FailedToBuildArguments(error)
    }
}

impl From<BuildCallArgumentError> for BuildCallArgumentsError {
    fn from(error: BuildCallArgumentError) -> Self {
        Self::BuildCallArgumentError(error)
    }
}

impl From<RustToManifestValueError> for BuildCallArgumentsError {
    fn from(error: RustToManifestValueError) -> Self {
        Self::RustToManifestValueError(error)
    }
}

/// Creates resource proof from an account.
pub fn create_proof_from_account<'a>(
    builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
    account: ComponentAddress,
    resource_specifier: String,
) -> Result<ManifestBuilder, BuildCallArgumentError> {
    let resource_specifier = parse_resource_specifier(&resource_specifier, address_bech32_decoder)
        .map_err(|_| BuildCallArgumentError::InvalidResourceSpecifier(resource_specifier))?;
    let builder = match resource_specifier {
        ResourceSpecifier::Amount(amount, resource_address) => {
            builder.create_proof_from_account_of_amount(account, resource_address, amount)
        }
        ResourceSpecifier::Ids(non_fungible_local_ids, resource_address) => builder
            .create_proof_from_account_of_non_fungibles(
                account,
                resource_address,
                non_fungible_local_ids,
            ),
    };
    Ok(builder)
}

pub fn build_call_arguments<'a>(
    mut builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
    schema: &VersionedScryptoSchema,
    type_id: LocalTypeId,
    args: Vec<String>,
    account: Option<ComponentAddress>,
) -> Result<(ManifestBuilder, ManifestValue), BuildCallArgumentsError> {
    let mut built_args = Vec::<ManifestValue>::new();
    match schema.v1().resolve_type_kind(type_id) {
        Some(TypeKind::Tuple { field_types }) => {
            if args.len() != field_types.len() {
                return Err(BuildCallArgumentsError::WrongNumberOfArguments(
                    args.len(),
                    field_types.len(),
                ));
            }

            for (i, f) in field_types.iter().enumerate() {
                let (returned_builder, value) = build_call_argument(
                    builder,
                    address_bech32_decoder,
                    schema,
                    schema
                        .v1()
                        .resolve_type_kind(*f)
                        .expect("Inconsistent schema"),
                    schema
                        .v1()
                        .resolve_type_validation(*f)
                        .expect("Inconsistent schema"),
                    args[i].clone(),
                    account,
                )?;
                builder = returned_builder;
                built_args.push(value);
            }
        }
        _ => panic!("Inconsistent schema"),
    }
    let manifest_value = to_manifest_value(&ManifestValue::Tuple { fields: built_args })?;
    Ok((builder, manifest_value))
}

macro_rules! parse_basic_type {
    ($builder:expr, $argument:expr, $type:tt) => {
        Ok((
            $builder,
            ManifestValue::$type {
                value: $argument
                    .parse()
                    .map_err(|_| BuildCallArgumentError::FailedToParse($argument))?,
            },
        ))
    };
}

macro_rules! matches_bucket {
    ($type_validation:expr) => {
        matches!(
            $type_validation,
            TypeValidation::Custom(
                ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)
            )
        )
    };
    ($type_validation:expr, $bucket_blueprint:expr) => {
        matches!(
            $type_validation,
            TypeValidation::Custom(
                ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), blueprint)
                )
            ) if blueprint == $bucket_blueprint
        )
    };
}

fn transform_scrypto_type_kind(
    type_kind: &ScryptoTypeKind<LocalTypeId>,
    type_validation: &TypeValidation<ScryptoCustomTypeValidation>,
) -> Result<ManifestValueKind, BuildCallArgumentError> {
    match type_kind {
        ScryptoTypeKind::Bool => Ok(ManifestValueKind::Bool),
        ScryptoTypeKind::I8 => Ok(ManifestValueKind::I8),
        ScryptoTypeKind::I16 => Ok(ManifestValueKind::I16),
        ScryptoTypeKind::I32 => Ok(ManifestValueKind::I32),
        ScryptoTypeKind::I64 => Ok(ManifestValueKind::I64),
        ScryptoTypeKind::I128 => Ok(ManifestValueKind::I128),
        ScryptoTypeKind::U8 => Ok(ManifestValueKind::U8),
        ScryptoTypeKind::U16 => Ok(ManifestValueKind::U16),
        ScryptoTypeKind::U32 => Ok(ManifestValueKind::U32),
        ScryptoTypeKind::U64 => Ok(ManifestValueKind::U64),
        ScryptoTypeKind::U128 => Ok(ManifestValueKind::U128),
        ScryptoTypeKind::String => Ok(ManifestValueKind::String),
        ScryptoTypeKind::Array { .. } => Ok(ManifestValueKind::Array),
        ScryptoTypeKind::Tuple { .. } => Ok(ManifestValueKind::Tuple),
        ScryptoTypeKind::Enum { .. } => Ok(ManifestValueKind::Enum),
        ScryptoTypeKind::Map { .. } => Ok(ManifestValueKind::Map),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Decimal) => {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Decimal))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal) => Ok(
            ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal),
        ),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId) => Ok(
            ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId),
        ),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference) => {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Address))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own)
            if matches_bucket!(type_validation)
                || matches_bucket!(type_validation, FUNGIBLE_BUCKET_BLUEPRINT)
                || matches_bucket!(type_validation, NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
        {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Bucket))
        }

        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof))
            ) =>
        {
            Ok(ManifestValueKind::Custom(ManifestCustomValueKind::Proof))
        }
        _ => Err(BuildCallArgumentError::UnsupportedType(type_kind.clone())),
    }
}

// Splits argument tuple or array elements delimited with "," into vector of elements.
// Elements with brackets (round or square) are taken as is (even if they may include ",")
fn split_argument_tuple_or_array(argument: &str) -> Result<Vec<String>, BuildCallArgumentError> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut round_brackets_level = 0;
    let mut square_brackets_level = 0;
    let mut prev_c: Option<char> = None;

    let mut chars = argument.chars();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        match c {
            ',' if round_brackets_level == 0 && square_brackets_level == 0 => {
                result.push(current.clone());
                current.clear();
            }
            '(' => {
                current.push(c);
                round_brackets_level += 1;
            }
            ')' => {
                current.push(c);
                if round_brackets_level > 0 {
                    round_brackets_level -= 1;
                } else {
                    // Non matching closing bracket
                    return Err(BuildCallArgumentError::FailedToParse(format!(
                        "Non-matching closing bracket ')' : {}",
                        argument
                    )));
                }
            }
            '[' => {
                current.push(c);
                square_brackets_level += 1;
            }
            ']' => {
                current.push(c);
                if square_brackets_level > 0 {
                    square_brackets_level -= 1;
                } else {
                    // Non matching closing bracket
                    return Err(BuildCallArgumentError::FailedToParse(format!(
                        "Non-matching closing bracket ']' : {}",
                        argument
                    )));
                }
            }
            _ => match prev_c {
                Some(prev_c) if prev_c == ']' || prev_c == ')' => {
                    return Err(BuildCallArgumentError::FailedToParse(format!(
                        "Invalid argument after closing bracket {:?}: {}",
                        prev_c, argument
                    )))
                }
                _ => current.push(c),
            },
        }
        prev_c = Some(c);
    }
    if round_brackets_level != 0 || square_brackets_level != 0 {
        Err(BuildCallArgumentError::FailedToParse(format!(
            "Non matching brackets found : {}",
            argument
        )))
    } else {
        if !current.is_empty() {
            result.push(current);
        }

        Ok(result)
    }
}

fn parse_tuple(
    mut builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
    schema: &VersionedScryptoSchema,
    field_types: &[LocalTypeId],
    argument: String,
    account: Option<ComponentAddress>,
) -> Result<(ManifestBuilder, ManifestValue), BuildCallArgumentError> {
    let mut manifests_vec: Vec<ManifestValue> = vec![];
    if !argument.starts_with("(") || !argument.ends_with(")") {
        return Err(BuildCallArgumentError::FailedToParse(format!(
            "Tuple argument not within round brackets: {}",
            argument
        )));
    }
    let args_parts = split_argument_tuple_or_array(&argument[1..argument.len() - 1])?;

    for (f, arg) in field_types.iter().zip(args_parts) {
        let (b, mv) = build_call_argument(
            builder,
            address_bech32_decoder,
            schema,
            &schema
                .v1()
                .resolve_type_kind(*f)
                .expect("Inconsistent schema"),
            &schema
                .v1()
                .resolve_type_validation(*f)
                .expect("Inconsistent schema"),
            arg.to_owned(),
            account,
        )?;
        builder = b;
        manifests_vec.push(mv);
    }
    Ok((
        builder,
        ManifestValue::Tuple {
            fields: manifests_vec,
        },
    ))
}

fn parse_array(
    mut builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
    schema: &VersionedScryptoSchema,
    element_type: &LocalTypeId,
    argument: String,
    account: Option<ComponentAddress>,
) -> Result<(ManifestBuilder, ManifestValue), BuildCallArgumentError> {
    let mut elements: Vec<ManifestValue> = vec![];
    if !argument.starts_with("[") || !argument.ends_with("]") {
        return Err(BuildCallArgumentError::FailedToParse(format!(
            "Array argument not within square brackets: {}",
            argument
        )));
    }
    let args_parts = split_argument_tuple_or_array(&argument[1..argument.len() - 1])?;

    let element_type_kind = schema
        .v1()
        .resolve_type_kind(*element_type)
        .expect("Inconsistent schema");
    let element_type_validation = schema
        .v1()
        .resolve_type_validation(*element_type)
        .expect("Inconsistent schema");

    for arg in args_parts {
        let (b, mv) = build_call_argument(
            builder,
            address_bech32_decoder,
            schema,
            &element_type_kind,
            &element_type_validation,
            arg.to_owned(),
            account,
        )?;
        builder = b;
        elements.push(mv);
    }

    Ok((
        builder,
        ManifestValue::Array {
            element_value_kind: transform_scrypto_type_kind(
                element_type_kind,
                element_type_validation,
            )?,
            elements,
        },
    ))
}

fn build_call_argument<'a>(
    mut builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
    schema: &VersionedScryptoSchema,
    type_kind: &ScryptoTypeKind<LocalTypeId>,
    type_validation: &TypeValidation<ScryptoCustomTypeValidation>,
    argument: String,
    account: Option<ComponentAddress>,
) -> Result<(ManifestBuilder, ManifestValue), BuildCallArgumentError> {
    match type_kind {
        ScryptoTypeKind::Bool => parse_basic_type!(builder, argument, Bool),
        ScryptoTypeKind::I8 => parse_basic_type!(builder, argument, I8),
        ScryptoTypeKind::I16 => parse_basic_type!(builder, argument, I16),
        ScryptoTypeKind::I32 => parse_basic_type!(builder, argument, I32),
        ScryptoTypeKind::I64 => parse_basic_type!(builder, argument, I64),
        ScryptoTypeKind::I128 => parse_basic_type!(builder, argument, I128),
        ScryptoTypeKind::U8 => parse_basic_type!(builder, argument, U8),
        ScryptoTypeKind::U16 => parse_basic_type!(builder, argument, U16),
        ScryptoTypeKind::U32 => parse_basic_type!(builder, argument, U32),
        ScryptoTypeKind::U64 => parse_basic_type!(builder, argument, U64),
        ScryptoTypeKind::U128 => parse_basic_type!(builder, argument, U128),
        ScryptoTypeKind::String => Ok((builder, ManifestValue::String { value: argument })),
        ScryptoTypeKind::Tuple { field_types } => parse_tuple(
            builder,
            address_bech32_decoder,
            schema,
            field_types,
            argument,
            account,
        ),
        ScryptoTypeKind::Array { element_type } => parse_array(
            builder,
            address_bech32_decoder,
            schema,
            element_type,
            argument,
            account,
        ),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Decimal) => Ok((
            builder,
            ManifestValue::Custom {
                value: ManifestCustomValue::Decimal(from_decimal(
                    argument
                        .parse()
                        .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?,
                )),
            },
        )),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal) => Ok((
            builder,
            ManifestValue::Custom {
                value: ManifestCustomValue::PreciseDecimal(from_precise_decimal(
                    argument
                        .parse()
                        .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?,
                )),
            },
        )),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId) => Ok((
            builder,
            ManifestValue::Custom {
                value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(
                    argument
                        .parse()
                        .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?,
                )),
            },
        )),
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalPackage
                ))
            ) =>
        {
            let value = PackageAddress::try_from_bech32(&address_bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalComponent
                ))
            ) =>
        {
            let value = ComponentAddress::try_from_bech32(&address_bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalResourceManager
                ))
            ) =>
        {
            let value = ResourceAddress::try_from_bech32(&address_bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobal
                ))
            ) =>
        {
            let value = GlobalAddress::try_from_bech32(&address_bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalTyped(_package_address, _bp_name)
                ))
            ) =>
        {
            let value = GlobalAddress::try_from_bech32(&address_bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own)
            if matches_bucket!(type_validation)
                || matches_bucket!(type_validation, FUNGIBLE_BUCKET_BLUEPRINT)
                || matches_bucket!(type_validation, NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
        {
            let resource_specifier = parse_resource_specifier(&argument, address_bech32_decoder)
                .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?;

            let bucket_name = builder.generate_bucket_name("taken");
            let builder = match resource_specifier {
                ResourceSpecifier::Amount(amount, resource_address) => {
                    if let Some(account) = account {
                        builder = builder.withdraw_from_account(account, resource_address, amount);
                    }
                    builder.take_from_worktop(resource_address, amount, &bucket_name)
                }
                ResourceSpecifier::Ids(ids, resource_address) => {
                    if let Some(account) = account {
                        builder = builder.withdraw_non_fungibles_from_account(
                            account,
                            resource_address,
                            ids.clone(),
                        );
                    }
                    builder.take_non_fungibles_from_worktop(resource_address, ids, &bucket_name)
                }
            };
            let bucket = builder.bucket(bucket_name);
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Bucket(bucket),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own)
            if matches!(
                type_validation,
                TypeValidation::Custom(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof))
            ) =>
        {
            let resource_specifier = parse_resource_specifier(&argument, address_bech32_decoder)
                .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?;
            let proof_name = builder.generate_proof_name("proof");
            let builder = match resource_specifier {
                ResourceSpecifier::Amount(amount, resource_address) => {
                    if let Some(account) = account {
                        builder
                            .create_proof_from_account_of_amount(account, resource_address, amount)
                            .pop_from_auth_zone(&proof_name)
                    } else {
                        todo!("Take from worktop and create proof")
                    }
                }
                ResourceSpecifier::Ids(ids, resource_address) => {
                    if let Some(account) = account {
                        builder
                            .create_proof_from_account_of_non_fungibles(
                                account,
                                resource_address,
                                ids,
                            )
                            .pop_from_auth_zone(&proof_name)
                    } else {
                        todo!("Take from worktop and create proof")
                    }
                }
            };
            let proof = builder.proof(proof_name);
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Proof(proof),
                },
            ))
        }
        _ => Err(BuildCallArgumentError::UnsupportedType(type_kind.clone())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
    use transaction::model::InstructionV1;

    #[test]
    pub fn parsing_of_u8_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U8;

        // Act
        let parsed_arg: u8 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u8)
    }

    #[test]
    pub fn parsing_of_u16_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U16;

        // Act
        let parsed_arg: u16 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u16)
    }

    #[test]
    pub fn parsing_of_u32_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U32;

        // Act
        let parsed_arg: u32 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u32)
    }

    #[test]
    pub fn parsing_of_u64_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U64;

        // Act
        let parsed_arg: u64 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u64)
    }

    #[test]
    pub fn parsing_of_u128_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U128;

        // Act
        let parsed_arg: u128 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u128)
    }

    #[test]
    pub fn parsing_of_i8_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::I8;

        // Act
        let parsed_arg: i8 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i8)
    }

    #[test]
    pub fn parsing_of_i16_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::I16;

        // Act
        let parsed_arg: i16 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i16)
    }

    #[test]
    pub fn parsing_of_i32_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::I32;

        // Act
        let parsed_arg: i32 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i32)
    }

    #[test]
    pub fn parsing_of_i64_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::I64;

        // Act
        let parsed_arg: i64 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i64)
    }

    #[test]
    pub fn parsing_of_i128_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::I128;

        // Act
        let parsed_arg: i128 = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i128)
    }

    #[test]
    pub fn parsing_of_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Decimal);

        // Act
        let parsed_arg: Decimal = build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, Decimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_component_address_succeeds() {
        // Arrange
        let component_address = component_address(EntityType::GlobalAccount, 5);

        let arg = AddressBech32Encoder::for_simulator()
            .encode(component_address.as_ref())
            .unwrap();
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference);
        let type_validation = TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
            ReferenceValidation::IsGlobalComponent,
        ));

        // Act
        let parsed_arg: ComponentAddress =
            build_and_decode_arg(arg, None, type_kind, type_validation)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, component_address)
    }

    #[test]
    pub fn parsing_of_global_address_succeeds() {
        // Arrange
        let component_address = component_address(EntityType::GlobalAccount, 5);
        let global_address: GlobalAddress = component_address.into();

        let arg = AddressBech32Encoder::for_simulator()
            .encode(global_address.as_ref())
            .unwrap();
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference);
        let type_validation = TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
            ReferenceValidation::IsGlobal,
        ));

        // Act
        let parsed_arg: GlobalAddress = build_and_decode_arg(arg, None, type_kind, type_validation)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, global_address)
    }

    #[test]
    pub fn parsing_of_global_address_typed_succeeds() {
        // Arrange
        let package_address = package_address(EntityType::GlobalPackage, 5);
        let global_address: GlobalAddress = package_address.into();

        let arg = AddressBech32Encoder::for_simulator()
            .encode(global_address.as_ref())
            .unwrap();

        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference);
        let type_validation = TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
            ReferenceValidation::IsGlobalTyped(
                Some(IDENTITY_PACKAGE),
                IDENTITY_BLUEPRINT.to_string(),
            ),
        ));

        // Act
        let parsed_arg: GlobalAddress = build_and_decode_arg(arg, None, type_kind, type_validation)
            .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, global_address)
    }

    #[test]
    pub fn parsing_of_package_address_succeeds() {
        // Arrange
        let package_address = package_address(EntityType::GlobalPackage, 5);

        let arg = AddressBech32Encoder::for_simulator()
            .encode(package_address.as_ref())
            .unwrap();
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference);
        let type_validation = TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
            ReferenceValidation::IsGlobalPackage,
        ));

        // Act
        let parsed_arg: PackageAddress =
            build_and_decode_arg(arg, None, type_kind, type_validation)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, package_address)
    }

    #[test]
    pub fn parsing_of_resource_address_succeeds() {
        // Arrange
        let resource_address = resource_address(EntityType::GlobalFungibleResourceManager, 5);

        let arg = AddressBech32Encoder::for_simulator()
            .encode(resource_address.as_ref())
            .unwrap();
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference);
        let type_validation = TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
            ReferenceValidation::IsGlobalResourceManager,
        ));

        // Act
        let parsed_arg: ResourceAddress =
            build_and_decode_arg(arg, None, type_kind, type_validation)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, resource_address)
    }

    #[test]
    pub fn parsing_of_precise_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal);

        // Act
        let parsed_arg: PreciseDecimal =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, PreciseDecimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_string_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "<HelloWorld>";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleLocalId::string("HelloWorld").unwrap()
        )
    }

    #[test]
    pub fn parsing_of_bytes_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "[c41fa9ef2ab31f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c]";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleLocalId::bytes(vec![
                196, 31, 169, 239, 42, 179, 31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73,
                178, 64, 175, 124, 185, 57, 234, 210, 144, 88, 253, 255, 44
            ])
            .unwrap()
        )
    }

    #[test]
    pub fn parsing_of_u64_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "#12#";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, NonFungibleLocalId::integer(12))
    }

    #[test]
    pub fn parsing_of_ruid_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "{1111111111111111-2222222222222222-3333333333333333-4444444444444444}";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleLocalId::ruid([
                0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                0x22, 0x22, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44,
            ])
        )
    }

    #[test]
    pub fn parsing_of_fungible_bucket_succeeds() {
        let amount = 2000;
        let arg = format!(
            "{}:{}",
            AddressBech32Encoder::for_simulator()
                .encode(XRD.as_ref())
                .unwrap(),
            amount
        );

        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own);
        let type_validations = vec![
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)),
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(
                    Some(RESOURCE_PACKAGE),
                    FUNGIBLE_BUCKET_BLUEPRINT.to_string(),
                ),
            )),
        ];

        for type_validation in type_validations {
            // Act
            let (builder, parsed_arg): (ManifestBuilder, ManifestBucket) =
                build_and_decode(&arg, None, type_kind.clone(), type_validation)
                    .expect("Failed to parse arg");
            let instructions = builder.build().instructions;

            // Assert
            assert_eq!(
                instructions.get(0).unwrap(),
                &InstructionV1::TakeFromWorktop {
                    resource_address: XRD,
                    amount: amount.into()
                }
            );
            assert_eq!(parsed_arg, ManifestBucket(0u32));
        }
    }

    #[test]
    pub fn parsing_of_non_fungible_bucket_succeeds() {
        // Arrange
        let local_ids: [u64; 3] = [12, 600, 123];
        let resource_address = resource_address(EntityType::GlobalNonFungibleResourceManager, 5);

        let arg = format!(
            "{}:#{}#,#{}#,#{}#",
            AddressBech32Encoder::for_simulator()
                .encode(resource_address.as_ref())
                .unwrap(),
            local_ids[0],
            local_ids[1],
            local_ids[2]
        );

        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own);
        let type_validations = vec![
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)),
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(
                    Some(RESOURCE_PACKAGE),
                    NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string(),
                ),
            )),
        ];

        for type_validation in type_validations {
            // Act
            let (builder, parsed_arg): (ManifestBuilder, ManifestBucket) =
                build_and_decode(&arg, None, type_kind.clone(), type_validation)
                    .expect("Failed to parse arg");
            let instructions = builder.build().instructions;
            let ids = local_ids
                .map(|id| NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(id)))
                .to_vec();

            // Assert
            assert_eq!(
                instructions.get(0).unwrap(),
                &InstructionV1::TakeNonFungiblesFromWorktop {
                    resource_address,
                    ids
                }
            );
            assert_eq!(parsed_arg, ManifestBucket(0u32));
        }
    }

    #[test]
    pub fn parsing_of_decimal_array_succeeds() {
        // Arrange
        let arg = "[12,13,14]";
        let element_type = LocalTypeId::WellKnown(well_known_scrypto_custom_types::DECIMAL_TYPE);
        let type_kind = ScryptoTypeKind::Array { element_type };

        // Act
        let parsed_arg: Vec<Decimal> =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, vec![dec!("12"), dec!("13"), dec!("14")]);
    }

    #[test]
    pub fn parsing_of_tuple_succeeds() {
        // Arrange
        let arg = "(12,13,-14)";
        let field_types = vec![
            LocalTypeId::WellKnown(well_known_scrypto_custom_types::DECIMAL_TYPE),
            LocalTypeId::WellKnown(basic_well_known_types::U8_TYPE),
            LocalTypeId::WellKnown(basic_well_known_types::I8_TYPE),
        ];
        let type_kind = ScryptoTypeKind::Tuple { field_types };

        // Act
        let parsed_arg: (Decimal, u8, i8) =
            build_and_decode_arg(arg, None, type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, (dec!("12"), 13u8, -14i8));
    }

    #[test]
    pub fn parsing_of_nested_tuple_succeeds() {
        // Arrange
        let arg = "(12,(true, #12#),[1,2])";
        let field_types = vec![
            LocalTypeId::WellKnown(basic_well_known_types::BOOL_TYPE),
            LocalTypeId::WellKnown(well_known_scrypto_custom_types::NON_FUNGIBLE_LOCAL_ID_TYPE),
        ];
        let element_type = LocalTypeId::WellKnown(well_known_scrypto_custom_types::DECIMAL_TYPE);

        let schema = VersionedScryptoSchema::V1(SchemaV1 {
            type_kinds: vec![
                ScryptoTypeKind::Array { element_type },
                ScryptoTypeKind::Tuple { field_types },
            ],
            type_metadata: vec![],
            type_validations: vec![TypeValidation::None, TypeValidation::None],
        });

        let field_types = vec![
            LocalTypeId::WellKnown(well_known_scrypto_custom_types::DECIMAL_TYPE),
            LocalTypeId::SchemaLocalIndex(1),
            LocalTypeId::SchemaLocalIndex(0),
        ];
        let type_kind = ScryptoTypeKind::Tuple { field_types };

        // Act
        let parsed_arg: (Decimal, (bool, NonFungibleLocalId), Vec<Decimal>) =
            build_and_decode_arg(arg, Some(schema), type_kind, TypeValidation::None)
                .expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            (
                dec!("12"),
                (true, NonFungibleLocalId::integer(12),),
                vec![dec!("1"), dec!("2")]
            )
        );
    }

    #[cfg(test)]
    fn build_and_decode<S: AsRef<str>, T: ManifestDecode>(
        arg: S,
        schema: Option<VersionedScryptoSchema>,
        type_kind: ScryptoTypeKind<LocalTypeId>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    ) -> Result<(ManifestBuilder, T), BuildAndDecodeArgError> {
        let schema = if let Some(schema) = schema {
            schema
        } else {
            VersionedScryptoSchema::V1(SchemaV1 {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            })
        };

        let builder = ManifestBuilder::new();
        let (builder, built_arg) = build_call_argument(
            builder,
            &AddressBech32Decoder::for_simulator(),
            &schema,
            &type_kind,
            &type_validation,
            arg.as_ref().to_owned(),
            None,
        )
        .map_err(BuildAndDecodeArgError::BuildCallArgumentError)?;

        let bytes = manifest_encode(&built_arg).map_err(BuildAndDecodeArgError::EncodeError)?;

        Ok((
            builder,
            manifest_decode(&bytes).map_err(BuildAndDecodeArgError::DecodeError)?,
        ))
    }

    #[cfg(test)]
    fn build_and_decode_arg<S: AsRef<str>, T: ManifestDecode>(
        arg: S,
        schema: Option<VersionedScryptoSchema>,
        type_kind: ScryptoTypeKind<LocalTypeId>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    ) -> Result<T, BuildAndDecodeArgError> {
        build_and_decode(arg, schema, type_kind, type_validation).map(|(_, arg)| arg)
    }

    #[derive(Debug, Clone)]
    #[cfg(test)]
    enum BuildAndDecodeArgError {
        BuildCallArgumentError(BuildCallArgumentError),
        EncodeError(EncodeError),
        DecodeError(DecodeError),
    }

    #[test]
    fn parsing_argument_split() {
        let input = "aaa,(bbb,(abc,(def,ghi),ccc";
        let error = split_argument_tuple_or_array(input).unwrap_err();
        assert!(matches!(error, BuildCallArgumentError::FailedToParse(..)));

        let input = "aaa,(bbb,(abc,(def,ghi),)ccc";
        let error = split_argument_tuple_or_array(input).unwrap_err();
        assert!(matches!(error, BuildCallArgumentError::FailedToParse(..)));

        let input = "aaa,(bbb,(abc,(def,ghi),)ccc";
        let error = split_argument_tuple_or_array(input).unwrap_err();
        assert!(matches!(error, BuildCallArgumentError::FailedToParse(..)));

        let input = "aaa,(bbb,(abc,[def,ghi])),ccc";
        assert_eq!(
            split_argument_tuple_or_array(input).unwrap(),
            vec![
                "aaa".to_string(),
                "(bbb,(abc,[def,ghi]))".to_string(),
                "ccc".to_string()
            ]
        );

        let input = "aaa,bbb,(abc,[def,ghi]),ccc";
        assert_eq!(
            split_argument_tuple_or_array(input).unwrap(),
            vec![
                "aaa".to_string(),
                "bbb".to_string(),
                "(abc,[def,ghi])".to_string(),
                "ccc".to_string()
            ]
        );

        let input = "aaa,bbb,(abc,[def,ghi]), ";
        assert_eq!(
            split_argument_tuple_or_array(input).unwrap(),
            vec![
                "aaa".to_string(),
                "bbb".to_string(),
                "(abc,[def,ghi])".to_string(),
            ]
        );
    }
}

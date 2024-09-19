//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known SCHEMA is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_transactions::data::{from_decimal, from_non_fungible_local_id, from_precise_decimal};
use radix_transactions::prelude::*;

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

fn build_call_argument<'a>(
    mut builder: ManifestBuilder,
    address_bech32_decoder: &AddressBech32Decoder,
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
    use radix_transactions::manifest::*;
    use radix_transactions::model::InstructionV1;

    #[test]
    pub fn parsing_of_u8_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::U8;

        // Act
        let parsed_arg: u8 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: u16 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: u32 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: u64 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: u128 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: i8 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: i16 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: i32 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: i64 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: i128 = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
        let parsed_arg: Decimal = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
            build_and_decode_arg(arg, type_kind, type_validation).expect("Failed to parse arg");

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
        let parsed_arg: GlobalAddress =
            build_and_decode_arg(arg, type_kind, type_validation).expect("Failed to parse arg");

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
        let parsed_arg: GlobalAddress =
            build_and_decode_arg(arg, type_kind, type_validation).expect("Failed to parse arg");

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
            build_and_decode_arg(arg, type_kind, type_validation).expect("Failed to parse arg");

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
            build_and_decode_arg(arg, type_kind, type_validation).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, resource_address)
    }

    #[test]
    pub fn parsing_of_precise_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let type_kind = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal);

        // Act
        let parsed_arg: PreciseDecimal = build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
            build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
            build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
            build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
            build_and_decode_arg(arg, type_kind, TypeValidation::None)
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
                build_and_decode(&arg, type_kind.clone(), type_validation)
                    .expect("Failed to parse arg");
            let instructions = builder.build_no_validate().instructions;

            // Assert
            assert_eq!(
                instructions.get(0).unwrap(),
                &InstructionV1::TakeFromWorktop(TakeFromWorktop {
                    resource_address: XRD,
                    amount: amount.into()
                })
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
                build_and_decode(&arg, type_kind.clone(), type_validation)
                    .expect("Failed to parse arg");
            let instructions = builder.build_no_validate().instructions;
            let ids = local_ids
                .map(|id| NonFungibleLocalId::Integer(IntegerNonFungibleLocalId::new(id)))
                .to_vec();

            // Assert
            assert_eq!(
                instructions.get(0).unwrap(),
                &InstructionV1::TakeNonFungiblesFromWorktop(TakeNonFungiblesFromWorktop {
                    resource_address,
                    ids
                })
            );
            assert_eq!(parsed_arg, ManifestBucket(0u32));
        }
    }

    pub fn build_and_decode<S: AsRef<str>, T: ManifestDecode>(
        arg: S,
        type_kind: ScryptoTypeKind<LocalTypeId>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    ) -> Result<(ManifestBuilder, T), BuildAndDecodeArgError> {
        let builder = ManifestBuilder::new();
        let (builder, built_arg) = build_call_argument(
            builder,
            &AddressBech32Decoder::for_simulator(),
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

    pub fn build_and_decode_arg<S: AsRef<str>, T: ManifestDecode>(
        arg: S,
        type_kind: ScryptoTypeKind<LocalTypeId>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    ) -> Result<T, BuildAndDecodeArgError> {
        build_and_decode(arg, type_kind, type_validation).map(|(_, arg)| arg)
    }

    #[allow(dead_code)] // As it's used for displaying the error messages
    #[derive(Debug, Clone)]
    pub enum BuildAndDecodeArgError {
        BuildCallArgumentError(BuildCallArgumentError),
        EncodeError(EncodeError),
        DecodeError(DecodeError),
    }
}

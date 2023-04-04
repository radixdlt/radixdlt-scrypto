//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known SCHEMA is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_engine::types::*;
use radix_engine_interface::schema::BlueprintSchema;
use transaction::builder::ManifestBuilder;
use transaction::data::{from_decimal, from_non_fungible_local_id, from_precise_decimal};
use transaction::model::Instruction;

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
}

/// Represents an error when parsing an argument.
#[derive(Debug, Clone)]
pub enum BuildCallArgumentError {
    /// The argument is of unsupported type.
    UnsupportedType(ScryptoTypeKind<LocalTypeIndex>),

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

/// Calls a function.
///
/// The implementation will automatically prepare the arguments based on the
/// function SCHEMA, including resource buckets and proofs.
///
/// If an Account component address is provided, resources will be withdrawn from the given account;
/// otherwise, they will be taken from transaction worktop.
pub fn add_call_function_instruction_with_schema<'a>(
    builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    package_address: PackageAddress,
    blueprint_name: &str,
    function: &str,
    args: Vec<String>,
    account: Option<ComponentAddress>,
    blueprint_schema: &BlueprintSchema,
) -> Result<&'a mut ManifestBuilder, BuildCallInstructionError> {
    let function_schema = blueprint_schema
        .find_function(function)
        .ok_or_else(|| BuildCallInstructionError::FunctionNotFound(function.to_owned()))?;

    let (builder, built_args) = build_call_arguments(
        builder,
        bech32_decoder,
        &blueprint_schema.schema,
        function_schema.input,
        args,
        account,
    )?;

    builder.add_instruction(Instruction::CallFunction {
        package_address,
        blueprint_name: blueprint_name.to_string(),
        function_name: function.to_string(),
        args: built_args,
    });
    Ok(builder)
}

/// Calls a method.
///
/// The implementation will automatically prepare the arguments based on the
/// method SCHEMA, including resource buckets and proofs.
///
/// If an Account component address is provided, resources will be withdrawn from the given account;
/// otherwise, they will be taken from transaction worktop.
pub fn add_call_method_instruction_with_schema<'a>(
    builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    component_address: ComponentAddress,
    method_name: &str,
    args: Vec<String>,
    account: Option<ComponentAddress>,
    blueprint_schema: &BlueprintSchema,
) -> Result<&'a mut ManifestBuilder, BuildCallInstructionError> {
    let function_schema = blueprint_schema
        .find_method(method_name)
        .ok_or_else(|| BuildCallInstructionError::MethodNotFound(method_name.to_owned()))?;

    let (builder, built_args) = build_call_arguments(
        builder,
        bech32_decoder,
        &blueprint_schema.schema,
        function_schema.input,
        args,
        account,
    )?;

    builder.add_instruction(Instruction::CallMethod {
        component_address,
        method_name: method_name.to_owned(),
        args: built_args,
    });
    Ok(builder)
}

/// Creates resource proof from an account.
pub fn create_proof_from_account<'a>(
    builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    account: ComponentAddress,
    resource_specifier: String,
) -> Result<&'a mut ManifestBuilder, BuildCallArgumentError> {
    let resource_specifier = parse_resource_specifier(&resource_specifier, bech32_decoder)
        .map_err(|_| BuildCallArgumentError::InvalidResourceSpecifier(resource_specifier))?;
    let builder = match resource_specifier {
        ResourceSpecifier::Amount(amount, resource_address) => {
            builder.create_proof_from_account_by_amount(account, resource_address, amount)
        }
        ResourceSpecifier::Ids(non_fungible_local_ids, resource_address) => builder
            .create_proof_from_account_by_ids(account, resource_address, &non_fungible_local_ids),
    };
    Ok(builder)
}

fn build_call_arguments<'a>(
    mut builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    schema: &ScryptoSchema,
    type_index: LocalTypeIndex,
    args: Vec<String>,
    account: Option<ComponentAddress>,
) -> Result<(&'a mut ManifestBuilder, ManifestValue), BuildCallArgumentsError> {
    let mut built_args = Vec::<ManifestValue>::new();
    match schema.resolve_type_kind(type_index) {
        Some(TypeKind::Tuple { field_types }) => {
            if args.len() != field_types.len() {
                return Err(BuildCallArgumentsError::WrongNumberOfArguments(
                    args.len(),
                    field_types.len(),
                ));
            }

            for (i, f) in field_types.iter().enumerate() {
                let tuple = build_call_argument(
                    builder,
                    bech32_decoder,
                    schema.resolve_type_kind(*f).expect("Inconsistent schema"),
                    args[i].clone(),
                    account,
                )?;
                builder = tuple.0;
                built_args.push(tuple.1);
            }
        }
        _ => panic!("Inconsistent schema"),
    }

    Ok((
        builder,
        to_manifest_value(&ManifestValue::Tuple { fields: built_args }),
    ))
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

fn build_call_argument<'a>(
    builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    type_kind: &ScryptoTypeKind<LocalTypeIndex>,
    argument: String,
    account: Option<ComponentAddress>,
) -> Result<(&'a mut ManifestBuilder, ManifestValue), BuildCallArgumentError> {
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
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PackageAddress) => {
            let value = PackageAddress::try_from_bech32(&bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::ComponentAddress) => {
            let value = ComponentAddress::try_from_bech32(&bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::ResourceAddress) => {
            let value = ResourceAddress::try_from_bech32(&bech32_decoder, &argument)
                .ok_or(BuildCallArgumentError::FailedToParse(argument))?;
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Address(value.into()),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Bucket) => {
            let resource_specifier = parse_resource_specifier(&argument, bech32_decoder)
                .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?;
            let bucket_id = match resource_specifier {
                ResourceSpecifier::Amount(amount, resource_address) => {
                    if let Some(account) = account {
                        builder.withdraw_from_account(account, resource_address, amount);
                    }
                    builder
                        .add_instruction(Instruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        })
                        .1
                        .unwrap()
                }
                ResourceSpecifier::Ids(ids, resource_address) => {
                    if let Some(account) = account {
                        builder.withdraw_non_fungibles_from_account(
                            account,
                            resource_address,
                            &ids,
                        );
                    }
                    builder
                        .add_instruction(Instruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        })
                        .1
                        .unwrap()
                }
            };
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Bucket(bucket_id),
                },
            ))
        }
        ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Proof) => {
            let resource_specifier = parse_resource_specifier(&argument, bech32_decoder)
                .map_err(|_| BuildCallArgumentError::FailedToParse(argument))?;
            let proof_id = match resource_specifier {
                ResourceSpecifier::Amount(amount, resource_address) => {
                    if let Some(account) = account {
                        builder.create_proof_from_account_by_amount(
                            account,
                            resource_address,
                            amount,
                        );
                        builder
                            .add_instruction(Instruction::PopFromAuthZone)
                            .2
                            .unwrap()
                    } else {
                        todo!("Take from worktop and create proof")
                    }
                }
                ResourceSpecifier::Ids(ids, resource_address) => {
                    if let Some(account) = account {
                        builder.create_proof_from_account_by_ids(account, resource_address, &ids);
                        builder
                            .add_instruction(Instruction::PopFromAuthZone)
                            .2
                            .unwrap()
                    } else {
                        todo!("Take from worktop and create proof")
                    }
                }
            };
            Ok((
                builder,
                ManifestValue::Custom {
                    value: ManifestCustomValue::Proof(proof_id),
                },
            ))
        }
        _ => Err(BuildCallArgumentError::UnsupportedType(type_kind.clone())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use transaction::builder::ManifestBuilder;

    #[test]
    pub fn parsing_of_u8_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::U8;

        // Act
        let parsed_arg: u8 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u8)
    }

    #[test]
    pub fn parsing_of_u16_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::U16;

        // Act
        let parsed_arg: u16 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u16)
    }

    #[test]
    pub fn parsing_of_u32_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::U32;

        // Act
        let parsed_arg: u32 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u32)
    }

    #[test]
    pub fn parsing_of_u64_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::U64;

        // Act
        let parsed_arg: u64 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u64)
    }

    #[test]
    pub fn parsing_of_u128_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::U128;

        // Act
        let parsed_arg: u128 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u128)
    }

    #[test]
    pub fn parsing_of_i8_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::I8;

        // Act
        let parsed_arg: i8 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i8)
    }

    #[test]
    pub fn parsing_of_i16_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::I16;

        // Act
        let parsed_arg: i16 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i16)
    }

    #[test]
    pub fn parsing_of_i32_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::I32;

        // Act
        let parsed_arg: i32 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i32)
    }

    #[test]
    pub fn parsing_of_i64_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::I64;

        // Act
        let parsed_arg: i64 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i64)
    }

    #[test]
    pub fn parsing_of_i128_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::I128;

        // Act
        let parsed_arg: i128 = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i128)
    }

    #[test]
    pub fn parsing_of_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Decimal);

        // Act
        let parsed_arg: Decimal = build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, Decimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_component_address_succeeds() {
        // Arrange
        let component_address = component_address(EntityType::GlobalAccount, 5);

        let arg = Bech32Encoder::for_simulator()
            .encode(component_address.as_ref())
            .unwrap();
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::ComponentAddress);

        // Act
        let parsed_arg: ComponentAddress =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, component_address)
    }

    #[test]
    pub fn parsing_of_package_address_succeeds() {
        // Arrange
        let package_address = package_address(EntityType::GlobalPackage, 5);

        let arg = Bech32Encoder::for_simulator()
            .encode(package_address.as_ref())
            .unwrap();
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PackageAddress);

        // Act
        let parsed_arg: PackageAddress =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, package_address)
    }

    #[test]
    pub fn parsing_of_resource_address_succeeds() {
        // Arrange
        let resource_address = resource_address(EntityType::GlobalFungibleResource, 5);

        let arg = Bech32Encoder::for_simulator()
            .encode(resource_address.as_ref())
            .unwrap();
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::ResourceAddress);

        // Act
        let parsed_arg: ResourceAddress =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, resource_address)
    }

    #[test]
    pub fn parsing_of_precise_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal);

        // Act
        let parsed_arg: PreciseDecimal =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, PreciseDecimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_string_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "<HelloWorld>";
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

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
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

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
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, NonFungibleLocalId::integer(12))
    }

    #[test]
    pub fn parsing_of_uuid_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "{f7223dbc-bbd6-4769-8d6f-effce550080d}";
        let arg_type = ScryptoTypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId);

        // Act
        let parsed_arg: NonFungibleLocalId =
            build_and_decode_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleLocalId::uuid(0xf7223dbc_bbd6_4769_8d6f_effce550080d).unwrap()
        )
    }

    pub fn build_and_decode_arg<S: AsRef<str>, T: ManifestDecode>(
        arg: S,
        arg_type: ScryptoTypeKind<LocalTypeIndex>,
    ) -> Result<T, BuildAndDecodeArgError> {
        let (_, built_arg) = build_call_argument(
            &mut ManifestBuilder::new(),
            &Bech32Decoder::for_simulator(),
            &arg_type,
            arg.as_ref().to_owned(),
            None,
        )
        .map_err(BuildAndDecodeArgError::BuildCallArgumentError)?;

        let bytes = manifest_encode(&built_arg).map_err(BuildAndDecodeArgError::EncodeError)?;

        manifest_decode(&bytes).map_err(BuildAndDecodeArgError::DecodeError)
    }

    #[derive(Debug, Clone)]
    pub enum BuildAndDecodeArgError {
        BuildCallArgumentError(BuildCallArgumentError),
        EncodeError(EncodeError),
        DecodeError(DecodeError),
    }
}

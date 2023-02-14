//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known ABI is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_engine::types::*;
use radix_engine_interface::abi::{BlueprintAbi, Type};
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::math::{ParseDecimalError, PreciseDecimal};
use transaction::builder::ManifestBuilder;
use transaction::model::BasicInstruction;

use crate::resim::SimulatorNonFungibleGlobalId;

// =======
// Macros
// =======

#[macro_export]
macro_rules! args_from_bytes_vec {
    ($args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(::radix_engine_interface::data::scrypto_decode(&arg).unwrap());
        }
        let input_struct = ::radix_engine_interface::data::ScryptoValue::Tuple { fields };
        ::radix_engine_interface::data::scrypto_encode(&input_struct).unwrap()
    }};
}

// ======
// Types
// ======

#[derive(Debug, PartialEq, Eq)]
enum ResourceSpecifier {
    Amount(Decimal, ResourceAddress),
    Ids(BTreeSet<NonFungibleLocalId>, ResourceAddress),
}

// ==========
// Functions
// ==========

/// Calls a function.
///
/// The implementation will automatically prepare the arguments based on the
/// function ABI, including resource buckets and proofs.
///
/// If an Account component address is provided, resources will be withdrawn from the given account;
/// otherwise, they will be taken from transaction worktop.
pub fn add_call_function_instruction_with_abi<'a>(
    manifest_builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    package_address: PackageAddress,
    blueprint_name: &str,
    function: &str,
    args: Vec<String>,
    account: Option<ComponentAddress>,
    blueprint_abi: &BlueprintAbi,
) -> Result<&'a mut ManifestBuilder, BuildCallWithAbiError> {
    let abi = blueprint_abi
        .fns
        .iter()
        .find(|f| f.ident == function)
        .map(Clone::clone)
        .ok_or_else(|| BuildCallWithAbiError::FunctionNotFound(function.to_owned()))?;

    let (manifest_builder, arguments) =
        parse_args(manifest_builder, bech32_decoder, &abi.input, args, account)?;

    let mut fields = Vec::new();
    for arg in arguments {
        fields.push(scrypto_decode(&arg).unwrap());
    }
    let input_struct = ScryptoValue::Tuple { fields };
    let bytes = scrypto_encode(&input_struct).unwrap();

    manifest_builder.add_instruction(BasicInstruction::CallFunction {
        package_address,
        blueprint_name: blueprint_name.to_string(),
        function_name: function.to_string(),
        args: bytes,
    });
    Ok(manifest_builder)
}

/// Calls a method.
///
/// The implementation will automatically prepare the arguments based on the
/// method ABI, including resource buckets and proofs.
///
/// If an Account component address is provided, resources will be withdrawn from the given account;
/// otherwise, they will be taken from transaction worktop.
pub fn add_call_method_instruction_with_abi<'a>(
    manifest_builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    component_address: ComponentAddress,
    method_name: &str,
    args: Vec<String>,
    account: Option<ComponentAddress>,
    blueprint_abi: &BlueprintAbi,
) -> Result<&'a mut ManifestBuilder, BuildCallWithAbiError> {
    let abi = blueprint_abi
        .fns
        .iter()
        .find(|m| m.ident == method_name)
        .map(Clone::clone)
        .ok_or_else(|| BuildCallWithAbiError::MethodNotFound(method_name.to_owned()))?;

    let (manifest_builder, arguments) =
        parse_args(manifest_builder, bech32_decoder, &abi.input, args, account)?;

    manifest_builder.add_instruction(BasicInstruction::CallMethod {
        component_address,
        method_name: method_name.to_owned(),
        args: args_from_bytes_vec!(arguments),
    });
    Ok(manifest_builder)
}

/// Creates resource proof from an account.
pub fn add_create_proof_instruction_from_account_with_resource_specifier<'a>(
    manifest_builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    account: ComponentAddress,
    resource_specifier: String,
) -> Result<&'a mut ManifestBuilder, BuildArgsError> {
    let resource_specifier = parse_resource_specifier(&resource_specifier, bech32_decoder)
        .map_err(|_| BuildArgsError::InvalidResourceSpecifier(resource_specifier))?;
    let builder = match resource_specifier {
        ResourceSpecifier::Amount(amount, resource_address) => {
            manifest_builder.create_proof_from_account_by_amount(account, amount, resource_address)
        }
        ResourceSpecifier::Ids(non_fungible_local_ids, resource_address) => manifest_builder
            .create_proof_from_account_by_ids(account, &non_fungible_local_ids, resource_address),
    };
    Ok(builder)
}

fn parse_args<'a>(
    manifest_builder: &'a mut ManifestBuilder,
    bech32_decoder: &Bech32Decoder,
    arg_type: &Type,
    args: Vec<String>,
    account: Option<ComponentAddress>,
) -> Result<(&'a mut ManifestBuilder, Vec<Vec<u8>>), BuildArgsError> {
    let mut encoded = Vec::new();

    match arg_type {
        Type::Struct {
            name: _,
            fields: Fields::Named { named },
        } => {
            for (i, (_, t)) in named.iter().enumerate() {
                let arg = args
                    .get(i)
                    .ok_or_else(|| BuildArgsError::MissingArgument(i, t.clone()))?;
                let res = match t {
                    Type::Bool => parse_basic_ty::<bool>(i, t, arg),
                    Type::I8 => parse_basic_ty::<i8>(i, t, arg),
                    Type::I16 => parse_basic_ty::<i16>(i, t, arg),
                    Type::I32 => parse_basic_ty::<i32>(i, t, arg),
                    Type::I64 => parse_basic_ty::<i64>(i, t, arg),
                    Type::I128 => parse_basic_ty::<i128>(i, t, arg),
                    Type::U8 => parse_basic_ty::<u8>(i, t, arg),
                    Type::U16 => parse_basic_ty::<u16>(i, t, arg),
                    Type::U32 => parse_basic_ty::<u32>(i, t, arg),
                    Type::U64 => parse_basic_ty::<u64>(i, t, arg),
                    Type::U128 => parse_basic_ty::<u128>(i, t, arg),
                    Type::String => parse_basic_ty::<String>(i, t, arg),
                    Type::Decimal => {
                        let value = arg.parse::<Decimal>().map_err(|_| {
                            BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                        })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::PreciseDecimal => {
                        let value = arg.parse::<PreciseDecimal>().map_err(|_| {
                            BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                        })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::PackageAddress => {
                        let value = bech32_decoder
                            .validate_and_decode_package_address(arg)
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::ComponentAddress => {
                        let value = bech32_decoder
                            .validate_and_decode_component_address(arg)
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::ResourceAddress => {
                        let value = bech32_decoder
                            .validate_and_decode_resource_address(arg)
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::Hash => {
                        let value = arg.parse::<Hash>().map_err(|_| {
                            BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                        })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::NonFungibleLocalId => {
                        let value = arg.parse::<NonFungibleLocalId>().map_err(|_| {
                            BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                        })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::NonFungibleGlobalId => {
                        // Using the same parsing logic implemented for the
                        // `SimulatorNonFungibleGlobalId` type since it is identical.
                        let value = arg
                            .parse::<SimulatorNonFungibleGlobalId>()
                            .map(|simulator_non_fungible_global_id| {
                                simulator_non_fungible_global_id.0
                            })
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::Bucket => {
                        let resource_specifier = parse_resource_specifier(arg, bech32_decoder)
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        let bucket_id = match resource_specifier {
                            ResourceSpecifier::Amount(amount, resource_address) => {
                                if let Some(account) = account {
                                    manifest_builder.withdraw_from_account_by_amount(
                                        account,
                                        amount,
                                        resource_address,
                                    );
                                }
                                manifest_builder
                                    .add_instruction(BasicInstruction::TakeFromWorktopByAmount {
                                        amount,
                                        resource_address,
                                    })
                                    .1
                                    .unwrap()
                            }
                            ResourceSpecifier::Ids(ids, resource_address) => {
                                if let Some(account) = account {
                                    manifest_builder.withdraw_from_account_by_ids(
                                        account,
                                        &ids,
                                        resource_address,
                                    );
                                }
                                manifest_builder
                                    .add_instruction(BasicInstruction::TakeFromWorktopByIds {
                                        ids,
                                        resource_address,
                                    })
                                    .1
                                    .unwrap()
                            }
                        };
                        Ok(scrypto_encode(&bucket_id).unwrap())
                    }
                    Type::Proof => {
                        let resource_specifier = parse_resource_specifier(arg, bech32_decoder)
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        let proof_id = match resource_specifier {
                            ResourceSpecifier::Amount(amount, resource_address) => {
                                if let Some(account) = account {
                                    manifest_builder.create_proof_from_account_by_amount(
                                        account,
                                        amount,
                                        resource_address,
                                    );
                                    manifest_builder
                                        .add_instruction(BasicInstruction::PopFromAuthZone)
                                        .2
                                        .unwrap()
                                } else {
                                    todo!("Take from worktop and create proof")
                                }
                            }
                            ResourceSpecifier::Ids(ids, resource_address) => {
                                if let Some(account) = account {
                                    manifest_builder.create_proof_from_account_by_ids(
                                        account,
                                        &ids,
                                        resource_address,
                                    );
                                    manifest_builder
                                        .add_instruction(BasicInstruction::PopFromAuthZone)
                                        .2
                                        .unwrap()
                                } else {
                                    todo!("Take from worktop and create proof")
                                }
                            }
                        };
                        Ok(scrypto_encode(&proof_id).unwrap())
                    }
                    _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
                };
                encoded.push(res?);
            }
            Ok(())
        }
        _ => Err(BuildArgsError::UnsupportedRootType(arg_type.clone())),
    }?;

    Ok((manifest_builder, encoded))
}

fn parse_basic_ty<T>(i: usize, t: &Type, arg: &str) -> Result<Vec<u8>, BuildArgsError>
where
    T: FromStr + ScryptoEncode,
    T::Err: fmt::Debug,
{
    let value = arg
        .parse::<T>()
        .map_err(|_| BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned()))?;
    Ok(scrypto_encode(&value).unwrap())
}

/// Attempts to parse a string as a [`ResourceSpecifier`] object.
///
/// Given a string, this function attempts to parse that string as a fungible or non-fungible
/// [`ResourceSpecifier`]. When a resource address is encountered in the string, the passed bech32m
/// decoder is used to attempt to decode the address.
///
/// The format expected for the string representation of fungible and non-fungible resource
/// specifiers differs. The following elaborates on the formats and the parsing modes.
///
/// ## Fungible Resource Specifiers
///
/// The string representation of fungible resource addresses is that it is a [`Decimal`] amount
/// followed by a resource address for that given resource. Separating the amount and the resource
/// address is a comma. The following is what that looks like as well as an example of that
///
/// ```txt
/// <amount>,<resource_address>
/// ```
///
/// As an example, say that `resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0` is a
/// fungible resource which we wish to create a resource specifier of `12.91` of, then the string
/// format to use for the fungible specifier would be:
///
/// ```txt
/// 12.91,resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0
/// ```
///
/// ## Non-Fungible Resource Specifiers
///
/// The string representation of non-fungible resource specifiers follows the same format which will
/// be used for the wallet, explorer, and other parts of our system. A string non-fungible resource
/// specifier beings with a Bech32m encoded resource address, then a colon, and then a list of comma
/// separated non-fungible ids that we wish to specify.
///
/// The type of these non-fungible ids does not need to be provided in the non-fungible resource
/// specifier string representation; this is because the type is automatically looked up for the
/// given resource address and then used as context for the parsing of the given non-fungible id
/// string.
///
/// The format of the string representation of non-fungible resource specifiers is:
///
/// ```txt
/// <resource_address>:<non_fungible_local_id_1>,<non_fungible_local_id_2>,...,<non_fungible_local_id_n>
/// ```
///
/// As an example, say that `resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0` is a
/// non-fungible resource which has a non-fungible id type of [`NonFungibleIdType::Integer`], say that
/// we wish to specify non-fungible tokens of this resource with the ids: 12, 900, 181, the string
/// representation of the non-fungible resource specifier would be:
///
/// ```txt
/// resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0:#12#,#900#,#181#
/// ```
///
/// As you can see from the example above, there was no need to specify the non-fungible id type in
/// the resource specifier string, as mentioned above, this is because this information can be
/// looked up from the simulator's substate store.
fn parse_resource_specifier(
    input: &str,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceSpecifier, ParseResourceSpecifierError> {
    // If the input contains a colon (:) then we assume it to be a non-fungible resource specifier
    // string.
    let is_fungible = !input.contains(':');
    if is_fungible {
        // Split up the input two two parts: the amount and resource address
        let tokens = input
            .trim()
            .split(',')
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        // There MUST only be two tokens in the tokens vector, one for the amount, and another for
        // the resource address. If there is more or less, then this function returns an error.
        if tokens.len() != 2 {
            return Err(ParseResourceSpecifierError::MoreThanOneAmountSpecified);
        }

        let amount_string = tokens[0];
        let resource_address_string = tokens[1];

        let amount = amount_string
            .parse()
            .map_err(ParseResourceSpecifierError::InvalidAmount)?;
        let resource_address = bech32_decoder
            .validate_and_decode_resource_address(resource_address_string)
            .map_err(ParseResourceSpecifierError::InvalidResourceAddress)?;

        Ok(ResourceSpecifier::Amount(amount, resource_address))
    } else {
        // Splitting up the input into two parts: the resource address and the non-fungible ids
        let tokens = input
            .trim()
            .split(':')
            .map(|s| s.trim())
            .collect::<Vec<_>>();

        // There MUST only be two tokens in the tokens vector, one for the resource address, and
        // another for the non-fungible ids. If there is more or less, then this function returns
        // an error.
        if tokens.len() != 2 {
            return Err(ParseResourceSpecifierError::MoreThanOneAmountSpecified);
        }

        // Paring the resource address fully first to use it for the non-fungible id type ledger
        // lookup
        let resource_address_string = tokens[0];
        let resource_address = bech32_decoder
            .validate_and_decode_resource_address(resource_address_string)
            .map_err(ParseResourceSpecifierError::InvalidResourceAddress)?;

        // Parsing the non-fungible ids with the available id type
        let non_fungible_local_ids = tokens[1]
            .split(',')
            .map(|s| NonFungibleLocalId::from_str(s.trim()))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(ParseResourceSpecifierError::InvalidNonFungibleLocalId)?;

        Ok(ResourceSpecifier::Ids(
            non_fungible_local_ids,
            resource_address,
        ))
    }
}

// ========
// Errors
// ========

#[derive(Debug)]
enum ParseResourceSpecifierError {
    InvalidAmount(ParseDecimalError),
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    MoreThanOneAmountSpecified,
}

/// Represents an error when parsing arguments.
#[derive(Debug, Clone)]
pub enum BuildArgsError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    UnsupportedRootType(Type),

    /// Failure when parsing an argument.
    FailedToParse(usize, Type, String),

    /// Failed to interpret this string as a resource specifier
    InvalidResourceSpecifier(String),
}

/// Represents an error when building a transaction.
#[derive(Debug, Clone)]
pub enum BuildCallWithAbiError {
    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match ABI.
    FailedToBuildArgs(BuildArgsError),

    /// Failed to export the ABI of a function.
    FailedToExportFunctionAbi(PackageAddress, String, String),

    /// Failed to export the ABI of a method.
    FailedToExportMethodAbi(ComponentAddress, String),

    /// Account is required but not provided.
    AccountNotProvided,
}

impl From<BuildArgsError> for BuildCallWithAbiError {
    fn from(error: BuildArgsError) -> Self {
        Self::FailedToBuildArgs(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use radix_engine_interface::abi::Type;
    use transaction::builder::ManifestBuilder;

    #[test]
    pub fn parsing_of_u8_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::U8;

        // Act
        let parsed_arg: u8 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u8)
    }

    #[test]
    pub fn parsing_of_u16_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::U16;

        // Act
        let parsed_arg: u16 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u16)
    }

    #[test]
    pub fn parsing_of_u32_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::U32;

        // Act
        let parsed_arg: u32 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u32)
    }

    #[test]
    pub fn parsing_of_u64_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::U64;

        // Act
        let parsed_arg: u64 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u64)
    }

    #[test]
    pub fn parsing_of_u128_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::U128;

        // Act
        let parsed_arg: u128 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12u128)
    }

    #[test]
    pub fn parsing_of_i8_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::I8;

        // Act
        let parsed_arg: i8 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i8)
    }

    #[test]
    pub fn parsing_of_i16_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::I16;

        // Act
        let parsed_arg: i16 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i16)
    }

    #[test]
    pub fn parsing_of_i32_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::I32;

        // Act
        let parsed_arg: i32 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i32)
    }

    #[test]
    pub fn parsing_of_i64_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::I64;

        // Act
        let parsed_arg: i64 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i64)
    }

    #[test]
    pub fn parsing_of_i128_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::I128;

        // Act
        let parsed_arg: i128 = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, 12i128)
    }

    #[test]
    pub fn parsing_of_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::Decimal;

        // Act
        let parsed_arg: Decimal = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, Decimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_component_address_succeeds() {
        // Arrange
        let component_address = ComponentAddress::Account([1u8; 26]);

        let arg =
            Bech32Encoder::for_simulator().encode_component_address_to_string(&component_address);
        let arg_type = Type::ComponentAddress;

        // Act
        let parsed_arg: ComponentAddress = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, component_address)
    }

    #[test]
    pub fn parsing_of_package_address_succeeds() {
        // Arrange
        let package_address = PackageAddress::Normal([1u8; 26]);

        let arg = Bech32Encoder::for_simulator().encode_package_address_to_string(&package_address);
        let arg_type = Type::PackageAddress;

        // Act
        let parsed_arg: PackageAddress = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, package_address)
    }

    #[test]
    pub fn parsing_of_resource_address_succeeds() {
        // Arrange
        let resource_address = ResourceAddress::Normal([1u8; 26]);

        let arg =
            Bech32Encoder::for_simulator().encode_resource_address_to_string(&resource_address);
        let arg_type = Type::ResourceAddress;

        // Act
        let parsed_arg: ResourceAddress = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, resource_address)
    }

    #[test]
    pub fn parsing_of_precise_decimal_succeeds() {
        // Arrange
        let arg = "12";
        let arg_type = Type::PreciseDecimal;

        // Act
        let parsed_arg: PreciseDecimal = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, PreciseDecimal::from_str("12").unwrap())
    }

    #[test]
    pub fn parsing_of_hash_succeeds() {
        // Arrange
        let arg = "c41fa9ef2ab31f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c";
        let arg_type = Type::Hash;

        // Act
        let parsed_arg: Hash = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            Hash([
                196, 31, 169, 239, 42, 179, 31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73,
                178, 64, 175, 124, 185, 57, 234, 210, 144, 88, 253, 255, 44
            ])
        )
    }

    #[test]
    pub fn parsing_of_string_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "<HelloWorld>";
        let arg_type = Type::NonFungibleLocalId;

        // Act
        let parsed_arg: NonFungibleLocalId = parse_arg(arg, arg_type).expect("Failed to parse arg");

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
        let arg_type = Type::NonFungibleLocalId;

        // Act
        let parsed_arg: NonFungibleLocalId = parse_arg(arg, arg_type).expect("Failed to parse arg");

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
        let arg_type = Type::NonFungibleLocalId;

        // Act
        let parsed_arg: NonFungibleLocalId = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(parsed_arg, NonFungibleLocalId::integer(12))
    }

    #[test]
    pub fn parsing_of_uuid_non_fungible_local_id_succeeds() {
        // Arrange
        let arg = "{f7223dbc-bbd6-4769-8d6f-effce550080d}";
        let arg_type = Type::NonFungibleLocalId;

        // Act
        let parsed_arg: NonFungibleLocalId = parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleLocalId::uuid(0xf7223dbc_bbd6_4769_8d6f_effce550080d).unwrap()
        )
    }

    #[test]

    pub fn parsing_of_bytes_non_fungible_global_id_succeeds() {
        // Arrange
        let arg = "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqshxgp7h:[1f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c]";
        let arg_type = Type::NonFungibleGlobalId;

        // Act
        let parsed_arg: NonFungibleGlobalId =
            parse_arg(arg, arg_type).expect("Failed to parse arg");

        // Assert
        assert_eq!(
            parsed_arg,
            NonFungibleGlobalId::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleLocalId::bytes(vec![
                    31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73, 178, 64, 175, 124, 185,
                    57, 234, 210, 144, 88, 253, 255, 44
                ])
                .unwrap()
            )
        )
    }

    #[test]
    pub fn parsing_of_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "900,resource_sim1qzms24rcrka4kdr2pn9zsw8jcghdvw6q2tux0rzq6gfsnhhmh4";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Amount(
                900.into(),
                bech32_decoder
                    .validate_and_decode_resource_address(
                        "resource_sim1qzms24rcrka4kdr2pn9zsw8jcghdvw6q2tux0rzq6gfsnhhmh4"
                    )
                    .unwrap()
            )
        )
    }

    #[test]
    pub fn parsing_of_single_non_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqshxgp7h:[1f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c]";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Ids(
                BTreeSet::from([NonFungibleLocalId::bytes(vec![
                    31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73, 178, 64, 175, 124, 185,
                    57, 234, 210, 144, 88, 253, 255, 44
                ])
                .unwrap()]),
                ECDSA_SECP256K1_TOKEN
            )
        )
    }

    #[test]

    pub fn parsing_of_multiple_non_fungible_resource_specifier_succeeds() {
        // Arrange
        let resource_specifier_string =
            "resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqshxgp7h:[1f5db2614c1c4c626e9c279349b240af7cb939ead29058fdff2c],[d85dc446d8e5eff48db25b56f6b5001d14627b5a199598485a8d],[005d1ae87b0e7c5401d38e58d43291ffbd9ba6e1da54f87504a7]";
        let bech32_decoder = Bech32Decoder::for_simulator();

        // Act
        let resource_specifier =
            parse_resource_specifier(resource_specifier_string, &bech32_decoder)
                .expect("Failed to parse resource specifier");

        // Assert
        assert_eq!(
            resource_specifier,
            ResourceSpecifier::Ids(
                BTreeSet::from([
                    NonFungibleLocalId::bytes(vec![
                        31, 93, 178, 97, 76, 28, 76, 98, 110, 156, 39, 147, 73, 178, 64, 175, 124,
                        185, 57, 234, 210, 144, 88, 253, 255, 44
                    ])
                    .unwrap(),
                    NonFungibleLocalId::bytes(vec![
                        216, 93, 196, 70, 216, 229, 239, 244, 141, 178, 91, 86, 246, 181, 0, 29,
                        20, 98, 123, 90, 25, 149, 152, 72, 90, 141
                    ])
                    .unwrap(),
                    NonFungibleLocalId::bytes(vec![
                        0, 93, 26, 232, 123, 14, 124, 84, 1, 211, 142, 88, 212, 50, 145, 255, 189,
                        155, 166, 225, 218, 84, 248, 117, 4, 167
                    ])
                    .unwrap()
                ]),
                ECDSA_SECP256K1_TOKEN
            )
        )
    }

    pub fn parse_arg<S: AsRef<str>, T: ScryptoDecode>(
        arg: S,
        arg_type: Type,
    ) -> Result<T, ParseArgsError> {
        let (_, encoded_arg) = parse_args(
            &mut ManifestBuilder::new(),
            &Bech32Decoder::for_simulator(),
            &Type::Struct {
                name: "MyNamedStruct".into(),
                fields: radix_engine::types::Fields::Named {
                    named: vec![("my_named_field".into(), arg_type)],
                },
            },
            vec![arg.as_ref().to_owned()],
            None,
        )
        .map_err(ParseArgsError::BuildArgsError)?;
        scrypto_decode(&encoded_arg[0]).map_err(ParseArgsError::DecodeError)
    }

    #[derive(Debug, Clone)]
    pub enum ParseArgsError {
        BuildArgsError(BuildArgsError),
        DecodeError(DecodeError),
    }
}

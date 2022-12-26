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

use crate::ledger::{lookup_id_type, LedgerLookupError};
use crate::resim::SimulatorNonFungibleAddress;

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

enum ResourceSpecifier {
    Amount(Decimal, ResourceAddress),
    Ids(BTreeSet<NonFungibleId>, ResourceAddress),
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
        ResourceSpecifier::Ids(non_fungible_ids, resource_address) => manifest_builder
            .create_proof_from_account_by_ids(account, &non_fungible_ids, resource_address),
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
                    Type::NonFungibleId => {
                        let value =
                            NonFungibleId::try_from_combined_simple_string(arg).map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::NonFungibleAddress => {
                        // Using the same parsing logic implemented for the
                        // `SimulatorNonFungibleAddress` type since it is identical.
                        let value = arg
                            .parse::<SimulatorNonFungibleAddress>()
                            .map(|simulator_non_fungible_address| simulator_non_fungible_address.0)
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
                        Ok(scrypto_encode(&Bucket(bucket_id)).unwrap())
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
                        Ok(scrypto_encode(&Proof(proof_id)).unwrap())
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
/// <resource_address>:<non_fungible_id_1>,<non_fungible_id_2>,...,<non_fungible_id_n>
/// ```
///
/// As an example, say that `resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0` is a
/// non-fungible resource which has a non-fungible id type of [`NonFungibleIdType::U32`], say that
/// we wish to specify non-fungible tokens of this resource with the ids: 12, 900, 181, the string
/// representation of the non-fungible resource specifier would be:
///
/// ```txt
/// resource_sim1qqw9095s39kq2vxnzymaecvtpywpkughkcltw4pzd4pse7dvr0:12,900,181
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
        let non_fungible_id_type = lookup_id_type(&resource_address)
            .map_err(ParseResourceSpecifierError::LedgerLookupError)?;

        // Parsing the non-fungible ids with the available id type
        let non_fungible_ids = tokens[1]
            .split(',')
            .map(|s| s.trim())
            .map(|s| NonFungibleId::try_from_simple_string(non_fungible_id_type, s))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(ParseResourceSpecifierError::InvalidNonFungibleId)?;

        Ok(ResourceSpecifier::Ids(non_fungible_ids, resource_address))
    }
}

// ========
// Errors
// ========

enum ParseResourceSpecifierError {
    InvalidAmount(ParseDecimalError),
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
    LedgerLookupError(LedgerLookupError),
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

    /// Failed to perform a ledger lookup
    LedgerLookupError(LedgerLookupError),
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

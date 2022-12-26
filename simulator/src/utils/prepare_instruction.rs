//! This module implements a number of useful utility functions used to prepare instructions for
//! calling methods and functions when a known ABI is provided. This module implements all of the
//! parsing logic as well as the logic needed ot add these instructions to the original manifest
//! builder that is being used.

use radix_engine::types::*;
use radix_engine_interface::abi::{BlueprintAbi, Type};
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::math::PreciseDecimal;
use transaction::builder::ManifestBuilder;
use transaction::model::BasicInstruction;

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
    let resource_specifier = parse_resource_specifier(&resource_specifier, &bech32_decoder)
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
                        let value = NonFungibleAddress::try_from_canonical_combined_string(
                            &bech32_decoder,
                            arg,
                        )
                        .map_err(|_| BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned()))?;
                        Ok(scrypto_encode(&value).unwrap())
                    }
                    Type::Bucket => {
                        let resource_specifier = parse_resource_specifier(arg, &bech32_decoder)
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
                        let resource_specifier = parse_resource_specifier(arg, &bech32_decoder)
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

fn parse_resource_specifier(
    input: &str,
    bech32_decoder: &Bech32Decoder,
) -> Result<ResourceSpecifier, ParseResourceSpecifierError> {
    let tokens: Vec<&str> = input.trim().split(',').map(|s| s.trim()).collect();

    // check length
    if tokens.len() < 2 {
        return Err(ParseResourceSpecifierError::IncompleteResourceSpecifier);
    }

    // parse resource address
    let resource_address_token = tokens[tokens.len() - 1];
    let resource_address = bech32_decoder
        .validate_and_decode_resource_address(resource_address_token)
        .map_err(|_| {
            ParseResourceSpecifierError::InvalidResourceAddress(resource_address_token.to_owned())
        })?;

    // parse non-fungible ids or amount
    if tokens[0].contains('#') {
        let mut ids = BTreeSet::<NonFungibleId>::new();
        for id in tokens[..tokens.len() - 1].iter() {
            let mut id = *id;
            if id.starts_with('#') {
                // Support the ids optionally starting with a # (which was an old encoding)
                // EG: #String#123,resource_address
                id = &id[1..];
            }
            ids.insert(
                NonFungibleId::try_from_combined_simple_string(id).map_err(|_| {
                    ParseResourceSpecifierError::InvalidNonFungibleId(id.to_string())
                })?,
            );
        }
        Ok(ResourceSpecifier::Ids(ids, resource_address))
    } else {
        if tokens.len() != 2 {
            return Err(ParseResourceSpecifierError::MoreThanOneAmountSpecified);
        }
        let amount: Decimal = tokens[0]
            .parse()
            .map_err(|_| ParseResourceSpecifierError::InvalidAmount(tokens[0].to_owned()))?;
        Ok(ResourceSpecifier::Amount(amount, resource_address))
    }
}

// ========
// Errors
// ========

enum ParseResourceSpecifierError {
    IncompleteResourceSpecifier,
    InvalidResourceAddress(String),
    InvalidAmount(String),
    InvalidNonFungibleId(String),
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

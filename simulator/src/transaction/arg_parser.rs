use radix_engine::execution::*;
use radix_engine::model::*;
use sbor::model::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::rust::collections::*;
use scrypto::rust::convert::TryFrom;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::transaction::*;

/// Parse arguments based on function ABI.
pub fn parse_args(
    types: &Vec<Type>,
    args: &Vec<&str>,
    alloc: &mut AddressAllocator,
) -> Result<(Vec<Vec<u8>>, HashMap<u8, Bucket>, HashMap<u8, Bucket>), ParseArgError> {
    let mut result = Vec::new();
    let mut tokens = HashMap::new();
    let mut badges = HashMap::new();

    for (i, t) in types.iter().enumerate() {
        let arg = args
            .get(i)
            .ok_or(ParseArgError::MissingArgument(i, t.clone()))?
            .clone();
        let res = match t {
            Type::Bool => parse_basic::<bool>(i, t, arg),
            Type::I8 => parse_basic::<i8>(i, t, arg),
            Type::I16 => parse_basic::<i16>(i, t, arg),
            Type::I32 => parse_basic::<i32>(i, t, arg),
            Type::I64 => parse_basic::<i64>(i, t, arg),
            Type::I128 => parse_basic::<i128>(i, t, arg),
            Type::U8 => parse_basic::<u8>(i, t, arg),
            Type::U16 => parse_basic::<u16>(i, t, arg),
            Type::U32 => parse_basic::<u32>(i, t, arg),
            Type::U64 => parse_basic::<u64>(i, t, arg),
            Type::U128 => parse_basic::<u128>(i, t, arg),
            Type::String => parse_basic::<String>(i, t, arg),
            Type::Custom { name } => {
                parse_custom_type(i, t, arg, name, alloc, &mut tokens, &mut badges)
            }
            _ => Err(ParseArgError::UnsupportedType(i, t.clone())),
        };
        result.push(res?);
    }

    Ok((result, tokens, badges))
}

/// Parse system package and pre-allocate the buckets.
pub fn parse_custom_type(
    i: usize,
    ty: &Type,
    arg: &str,
    name: &str,
    alloc: &mut AddressAllocator,
    tokens: &mut HashMap<u8, Bucket>,
    badges: &mut HashMap<u8, Bucket>,
) -> Result<Vec<u8>, ParseArgError> {
    match name {
        "U256" => parse_u256(i, ty, arg),
        "Address" => parse_basic::<Address>(i, ty, arg),
        "H256" => parse_basic::<H256>(i, ty, arg),
        "Tokens" | "Badges" => {
            let mut split = arg.split(":");
            let amount = split.next().and_then(|v| U256::from_dec_str(v).ok());
            let resource = split.next().and_then(|v| Address::try_from(v).ok());
            match (amount, resource) {
                (Some(a), Some(r)) => {
                    let n = alloc.count();
                    if n >= 255 {
                        return Err(ParseArgError::BucketLimitReached);
                    }

                    let bid = alloc.new_transient_bid();
                    if name == "Tokens" {
                        tokens.insert(n as u8, Bucket::new(a, r));
                        Ok(scrypto_encode(&scrypto::resource::Tokens::from(bid)))
                    } else {
                        badges.insert(n as u8, Bucket::new(a, r));
                        Ok(scrypto_encode(&scrypto::resource::Badges::from(bid)))
                    }
                }
                _ => Err(ParseArgError::UnableToParse(i, ty.clone(), arg.to_owned())),
            }
        }
        _ => Err(ParseArgError::UnsupportedType(i, ty.clone())),
    }
}

/// Parse basic type from string.
pub fn parse_basic<T>(i: usize, ty: &Type, arg: &str) -> Result<Vec<u8>, ParseArgError>
where
    T: FromStr + Encode,
    T::Err: fmt::Debug,
{
    let value = arg
        .parse::<T>()
        .map_err(|_| ParseArgError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
    Ok(scrypto_encode(&value))
}

/// Parse a U256 from a decimal string.
pub fn parse_u256(i: usize, ty: &Type, arg: &str) -> Result<Vec<u8>, ParseArgError> {
    let value = U256::from_dec_str(&arg)
        .map_err(|_| ParseArgError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
    Ok(scrypto_encode(&value))
}

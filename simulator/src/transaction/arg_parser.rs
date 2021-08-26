use radix_engine::model::*;
use sbor::model::*;
use sbor::*;
use scrypto::abi;
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
    func: &abi::Function,
    args: &Vec<String>,
) -> Result<(Vec<Vec<u8>>, HashMap<BID, Bucket>), ParseArgError> {
    let mut result = Vec::new();
    let mut buckets = HashMap::new();

    for (i, t) in func.inputs.iter().enumerate() {
        let arg = args
            .get(i)
            .ok_or(ParseArgError::MissingArgument(i, t.clone()))?
            .clone();
        let res = match t {
            Type::Bool => parse_basic::<bool>(i, t.clone(), arg),
            Type::I8 => parse_basic::<i8>(i, t.clone(), arg),
            Type::I16 => parse_basic::<i16>(i, t.clone(), arg),
            Type::I32 => parse_basic::<i32>(i, t.clone(), arg),
            Type::I64 => parse_basic::<i64>(i, t.clone(), arg),
            Type::I128 => parse_basic::<i128>(i, t.clone(), arg),
            Type::U8 => parse_basic::<u8>(i, t.clone(), arg),
            Type::U16 => parse_basic::<u16>(i, t.clone(), arg),
            Type::U32 => parse_basic::<u32>(i, t.clone(), arg),
            Type::U64 => parse_basic::<u64>(i, t.clone(), arg),
            Type::U128 => parse_basic::<u128>(i, t.clone(), arg),
            Type::String => parse_basic::<String>(i, t.clone(), arg),
            Type::H256 => parse_basic::<H256>(i, t.clone(), arg),
            Type::Address => parse_basic::<Address>(i, t.clone(), arg),
            Type::U256 => parse_u256(i, t.clone(), arg),
            Type::SystemType { name } => {
                parse_system_type(i, t.clone(), arg, name.clone(), &mut buckets)
            }
            _ => Err(ParseArgError::UnsupportedType(i, t.clone())),
        };
        result.push(res?);
    }

    Ok((result, buckets))
}

/// Parse system package and pre-allocate the buckets.
pub fn parse_system_type(
    i: usize,
    ty: Type,
    arg: String,
    name: String,
    buckets: &mut HashMap<BID, Bucket>,
) -> Result<Vec<u8>, ParseArgError> {
    match name.as_str() {
        "::scrypto::resource::Tokens" | "::scrypto::resource::Badges" => {
            let mut split = arg.split(":");
            let amount = split.next().and_then(|v| U256::from_dec_str(v).ok());
            let resource = split.next().and_then(|v| Address::try_from(v).ok());
            match (amount, resource) {
                (Some(a), Some(r)) => {
                    let bid = BID::Transient(buckets.len() as u32);
                    buckets.insert(bid, Bucket::new(a, r));
                    Ok(scrypto_encode(&bid))
                }
                _ => Err(ParseArgError::UnableToParse(i, ty, arg)),
            }
        }
        _ => Err(ParseArgError::UnsupportedType(i, ty)),
    }
}

/// Parse basic type from string.
pub fn parse_basic<T>(i: usize, ty: Type, arg: String) -> Result<Vec<u8>, ParseArgError>
where
    T: FromStr + Encode,
    T::Err: fmt::Debug,
{
    let value = arg
        .parse::<T>()
        .map_err(|_| ParseArgError::UnableToParse(i, ty, arg))?;
    Ok(scrypto_encode(&value))
}

/// Parse a U256 from a decimal string.
pub fn parse_u256(i: usize, ty: Type, arg: String) -> Result<Vec<u8>, ParseArgError> {
    let value = U256::from_dec_str(&arg).map_err(|_| ParseArgError::UnableToParse(i, ty, arg))?;
    Ok(scrypto_encode(&value))
}

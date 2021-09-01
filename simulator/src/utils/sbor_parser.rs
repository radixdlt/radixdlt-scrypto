use sbor::*;
use scrypto::constants::*;
use scrypto::resource::*;
use scrypto::types::*;

pub fn parse_sbor_data(data: &[u8]) -> Result<String, DecodeError> {
    let mut decoder = Decoder::with_type(data);
    let result = traverse(None, &mut decoder);
    decoder.check_end()?;
    result
}

fn traverse(ty_known: Option<u8>, dec: &mut Decoder) -> Result<String, DecodeError> {
    let ty = match ty_known {
        Some(t) => t,
        None => dec.read_type()?,
    };

    match ty {
        constants::TYPE_UNIT => Ok("()".to_owned()),
        constants::TYPE_BOOL => Ok(<bool>::decode_value(dec)?.to_string()),
        constants::TYPE_I8 => Ok(<i8>::decode_value(dec)?.to_string()),
        constants::TYPE_I16 => Ok(<i16>::decode_value(dec)?.to_string()),
        constants::TYPE_I32 => Ok(<i32>::decode_value(dec)?.to_string()),
        constants::TYPE_I64 => Ok(<i64>::decode_value(dec)?.to_string()),
        constants::TYPE_I128 => Ok(<i128>::decode_value(dec)?.to_string()),
        constants::TYPE_U8 => Ok(<u8>::decode_value(dec)?.to_string()),
        constants::TYPE_U16 => Ok(<u16>::decode_value(dec)?.to_string()),
        constants::TYPE_U32 => Ok(<u32>::decode_value(dec)?.to_string()),
        constants::TYPE_U64 => Ok(<u64>::decode_value(dec)?.to_string()),
        constants::TYPE_U128 => Ok(<u128>::decode_value(dec)?.to_string()),
        constants::TYPE_STRING => Ok(format!("\"{}\"", <String>::decode_value(dec)?)),
        constants::TYPE_OPTION => {
            // index
            let index = dec.read_u8()?;
            // optional value
            match index {
                0 => Ok("None".to_owned()),
                1 => Ok(format!("Some({})", traverse(None, dec)?)),
                _ => Err(DecodeError::InvalidIndex(index)),
            }
        }
        constants::TYPE_BOX => Ok(format!("Box({})", traverse(None, dec)?)),
        constants::TYPE_ARRAY | constants::TYPE_VEC => {
            // element type
            let ele_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut buf = String::from("[");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                buf.push_str(traverse(Some(ele_ty), dec)?.as_str());
            }
            buf.push(']');
            Ok(buf)
        }
        constants::TYPE_TUPLE => {
            //length
            let len = dec.read_len()?;
            // values
            let mut buf = String::from("(");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                buf.push_str(traverse(None, dec)?.as_str());
            }
            buf.push(')');
            Ok(buf)
        }
        constants::TYPE_STRUCT => {
            // fields
            let fields = traverse(None, dec)?;
            Ok(fields)
        }
        constants::TYPE_ENUM => {
            // index
            let index = dec.read_u8()?;
            // fields
            let fields = traverse(None, dec)?;
            Ok(format!("#{} {}", index, fields))
        }
        constants::TYPE_FIELDS_NAMED => {
            //length
            let len = dec.read_len()?;
            // named fields
            let mut buf = String::from("{ ");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                // value
                let value = traverse(None, dec)?;
                buf.push_str(format!("#{}: {}", i, value).as_str());
            }
            buf.push_str(" }");
            Ok(buf)
        }
        constants::TYPE_FIELDS_UNNAMED => {
            //length
            let len = dec.read_len()?;
            // named fields
            let mut buf = String::from("(");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                // value
                let value = traverse(None, dec)?;
                buf.push_str(value.as_str());
            }
            buf.push(')');
            Ok(buf)
        }
        constants::TYPE_FIELDS_UNIT => Ok("()".to_owned()),
        // collections
        constants::TYPE_TREE_SET | constants::TYPE_HASH_SET => {
            // element type
            let ele_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // elements
            let mut buf = String::from("[");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                // value
                let value = traverse(Some(ele_ty), dec)?;
                buf.push_str(value.as_str());
            }
            buf.push(']');
            Ok(buf)
        }
        constants::TYPE_TREE_MAP | constants::TYPE_HASH_MAP => {
            // length
            let len = dec.read_len()?;
            // key type
            let key_ty = dec.read_type()?;
            // value type
            let value_ty = dec.read_type()?;
            // elements
            let mut buf = String::from("{ ");
            for i in 0..len {
                if i != 0 {
                    buf.push_str(", ");
                }
                // key
                let key = traverse(Some(key_ty), dec)?;
                // value
                let value = traverse(Some(value_ty), dec)?;
                buf.push_str(format!("{}: {}", key, value).as_str());
            }
            buf.push_str(" }");
            Ok(buf)
        }
        // scrypto types
        SCRYPTO_TYPE_U256 => Ok(<U256>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_ADDRESS => Ok(<Address>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_H256 => Ok(<H256>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_MID => Ok(<MID>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_BID => Ok(<BID>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_RID => Ok(<RID>::decode_value(dec)?.to_string()),
        SCRYPTO_TYPE_TOKENS => Ok(format!(
            "Tokens::{}",
            Into::<BID>::into(<Tokens>::decode_value(dec)?).to_string()
        )),
        SCRYPTO_TYPE_TOKENS_REF => Ok(format!(
            "TokensRef::{}",
            Into::<RID>::into(<TokensRef>::decode_value(dec)?).to_string()
        )),
        SCRYPTO_TYPE_BADGES => Ok(format!(
            "Badges::{}",
            Into::<BID>::into(<Badges>::decode_value(dec)?).to_string()
        )),
        SCRYPTO_TYPE_BADGES_REF => Ok(format!(
            "BadgesRef::{}",
            Into::<RID>::into(<BadgesRef>::decode_value(dec)?).to_string()
        )),
        _ => Err(DecodeError::InvalidType {
            expected: 0xff,
            actual: ty,
        }),
    }
}

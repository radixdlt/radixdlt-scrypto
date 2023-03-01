use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn extract_attributes(
    attrs: &[Attribute],
    name: &str,
) -> Option<HashMap<String, Option<String>>> {
    for attr in attrs {
        if !attr.path.is_ident(name) {
            continue;
        }

        let mut fields = HashMap::new();
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(MetaList { nested, .. }) = meta {
                nested.into_iter().for_each(|m| match m {
                    NestedMeta::Meta(m) => match m {
                        Meta::NameValue(name_value) => {
                            if let Some(ident) = name_value.path.get_ident() {
                                if let Lit::Str(s) = name_value.lit {
                                    fields.insert(ident.to_string(), Some(s.value()));
                                }
                            }
                        }
                        Meta::Path(path) => {
                            if let Some(ident) = path.get_ident() {
                                fields.insert(ident.to_string(), None);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                })
            }
        }
        return Some(fields);
    }

    None
}

pub fn is_mutable(f: &Field) -> bool {
    extract_attributes(&f.attrs, "mutable").is_some()
}

pub fn handle_non_fungible_data(input: TokenStream) -> Result<TokenStream> {
    trace!("handle_non_fungible_data() starts");

    let DeriveInput { ident, data, .. } = parse2(input)?;
    trace!("Processing: {}", ident.to_string());

    let output = match data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // immutable
                let im: Vec<&Field> = named.iter().filter(|f| !is_mutable(f)).collect();
                let im_n = Index::from(im.len());
                let im_ids = im.iter().map(|f| &f.ident);
                let im_ids2 = im_ids.clone();
                let im_types = im.iter().map(|f| &f.ty);
                // mutable
                let m: Vec<&Field> = named.iter().filter(|f| is_mutable(f)).collect();
                let m_n = Index::from(m.len());
                let m_ids = m.iter().map(|f| &f.ident);
                let m_ids2 = m_ids.clone();
                let m_types = m.iter().map(|f| &f.ty);

                quote! {
                    impl ::scrypto::resource::NonFungibleData for #ident {
                        fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, ::sbor::DecodeError> {
                            use ::sbor::{value_kind::*, *};
                            let mut decoder_nm = ::scrypto::data::scrypto::ScryptoDecoder::new(immutable_data, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                            decoder_nm.read_and_check_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                            decoder_nm.read_and_check_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                            decoder_nm.read_and_check_size(#im_n)?;

                            let mut decoder_m = ::scrypto::data::scrypto::ScryptoDecoder::new(mutable_data, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                            decoder_m.read_and_check_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                            decoder_m.read_and_check_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                            decoder_m.read_and_check_size(#m_n)?;

                            let decoded = Self {
                                #(#im_ids: decoder_nm.decode::<#im_types>()?,)*
                                #(#m_ids: decoder_m.decode::<#m_types>()?,)*
                            };

                            decoder_nm.check_end()?;
                            decoder_m.check_end()?;

                            Ok(decoded)
                        }

                        fn immutable_data(&self) -> Result<::sbor::rust::vec::Vec<u8>, ::sbor::EncodeError> {
                            use ::sbor::{value_kind::*, *};

                            let mut bytes = Vec::with_capacity(512);
                            let mut encoder = ::scrypto::data::scrypto::ScryptoEncoder::new(&mut bytes, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                            encoder.write_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                            encoder.write_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                            encoder.write_size(#im_n)?;
                            #(
                                encoder.encode(&self.#im_ids2)?;
                            )*

                            Ok(bytes)
                        }

                        fn mutable_data(&self) -> Result<::sbor::rust::vec::Vec<u8>, ::sbor::EncodeError> {
                            use ::sbor::{value_kind::*, *};
                            use ::sbor::rust::vec::Vec;

                            let mut bytes = Vec::with_capacity(512);
                            let mut encoder = ::scrypto::data::scrypto::ScryptoEncoder::new(&mut bytes, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                            encoder.write_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                            encoder.write_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                            encoder.write_size(#m_n)?;
                            #(
                                encoder.encode(&self.#m_ids2)?;
                            )*

                            Ok(bytes)
                        }
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                return Err(Error::new(
                    Span::call_site(),
                    "Struct with unnamed fields is not supported!",
                ));
            }
            syn::Fields::Unit => {
                return Err(Error::new(
                    Span::call_site(),
                    "Struct with no fields is not supported!",
                ));
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(Error::new(
                Span::call_site(),
                "Enum or union can not be used as non-fungible data presently!",
            ));
        }
    };

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("NonFungibleData", &output);

    trace!("handle_non_fungible_data() finishes");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    use super::*;

    fn assert_code_eq(a: TokenStream, b: TokenStream) {
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_non_fungible() {
        let input = TokenStream::from_str(
            "pub struct MyStruct { pub field_1: u32, #[mutable] pub field_2: String, }",
        )
        .unwrap();
        let output = handle_non_fungible_data(input).unwrap();

        assert_code_eq(
            output,
            quote! {
                impl ::scrypto::resource::NonFungibleData for MyStruct {
                    fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, ::sbor::DecodeError> {
                        use ::sbor::{value_kind::*, *};
                        let mut decoder_nm = ::scrypto::data::scrypto::ScryptoDecoder::new(immutable_data, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                        decoder_nm.read_and_check_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                        decoder_nm.read_and_check_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                        decoder_nm.read_and_check_size(1)?;
                        let mut decoder_m = ::scrypto::data::scrypto::ScryptoDecoder::new(mutable_data, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                        decoder_m.read_and_check_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                        decoder_m.read_and_check_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                        decoder_m.read_and_check_size(1)?;
                        let decoded = Self {
                            field_1: decoder_nm.decode::<u32>()?,
                            field_2: decoder_m.decode::<String>()?,
                        };
                        decoder_nm.check_end()?;
                        decoder_m.check_end()?;
                        Ok(decoded)
                    }
                    fn immutable_data(&self) -> Result<::sbor::rust::vec::Vec<u8>, ::sbor::EncodeError> {
                        use ::sbor::{value_kind::*, *};
                        let mut bytes = Vec::with_capacity(512);
                        let mut encoder = ::scrypto::data::scrypto::ScryptoEncoder::new(&mut bytes, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                        encoder.write_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                        encoder.write_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                        encoder.write_size(1)?;
                        encoder.encode(&self.field_1)?;
                        Ok(bytes)
                    }
                    fn mutable_data(&self) -> Result<::sbor::rust::vec::Vec<u8>, ::sbor::EncodeError> {
                        use ::sbor::{value_kind::*, *};
                        use ::sbor::rust::vec::Vec;
                        let mut bytes = Vec::with_capacity(512);
                        let mut encoder = ::scrypto::data::scrypto::ScryptoEncoder::new(&mut bytes, ::scrypto::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH);
                        encoder.write_payload_prefix(::scrypto::data::scrypto::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
                        encoder.write_value_kind(::scrypto::data::scrypto::ScryptoValueKind::Tuple)?;
                        encoder.write_size(1)?;
                        encoder.encode(&self.field_2)?;
                        Ok(bytes)
                    }
                }
            },
        );
    }
}

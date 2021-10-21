use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::*;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_auth(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    trace!("Started processing auth macro");

    let meta = parse2::<MetaList>(attr)?;
    println!("{:?}", meta);

    let mut badges = Vec::new();
    if let Some(ident) = meta.path.get_ident() {
        if ident.to_string().as_str() == "all" {
            for m in &meta.nested {
                if let NestedMeta::Meta(x) = m {
                    if let Meta::Path(p) = x {
                        badges.push(
                            p.get_ident()
                                .ok_or(Error::new(p.span(), "Missing identity"))?,
                        );
                    } else {
                        return Err(Error::new(x.span(), "Only path is allowed"));
                    }
                } else {
                    return Err(Error::new(m.span(), "Only meta is allowed"));
                }
            }
        } else {
            return Err(Error::new(meta.path.span(), "Unsupported predicate"));
        }
    } else {
        return Err(Error::new(
            meta.path.span(),
            "Missing predicate, try `all` or `any` (TODO)",
        ));
    }

    let f = parse2::<ItemFn>(item)?;

    // function visibility, identity, inputs and output
    let f_attrs = f.attrs;
    let f_vis = f.vis;
    let f_ident = f.sig.ident;
    let mut f_inputs: Vec<FnArg> = f.sig.inputs.iter().map(Clone::clone).collect();
    f_inputs.push(parse_quote! {
        badges: ::scrypto::rust::vec::Vec<::scrypto::resource::BucketRef>
    });
    let f_output = f.sig.output;

    // function body
    let f_body = f.block;

    // generate output
    let i = 0..badges.len();
    let output = quote! {
        #(#f_attrs)*
        #f_vis fn #f_ident (#(#f_inputs),*) #f_output {
            #(badges[#i].check(self.#badges);)*
            #f_body
        }
    };
    trace!("Finished processing auth macro");

    #[cfg(feature = "trace")]
    crate::utils::print_compiled_code("auth", &output);

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
    fn test_auth_all() {
        let attr = TokenStream::from_str("all(foo, bar)").unwrap();
        let item = TokenStream::from_str("#[other] pub fn x(&self) -> u32 { self.a }").unwrap();
        let output = handle_auth(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[other]
                pub fn x(
                    &self,
                    badges: ::scrypto::rust::vec::Vec<::scrypto::resource::BucketRef>
                ) -> u32 {
                    badges[0usize].check(self.foo);
                    badges[1usize].check(self.bar);
                    {
                        self.a
                    }
                }
            },
        );
    }
}

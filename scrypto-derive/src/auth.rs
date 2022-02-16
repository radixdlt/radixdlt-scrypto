use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::*;

use crate::ast;

macro_rules! trace {
    ($($arg:expr),*) => {{
        #[cfg(feature = "trace")]
        println!($($arg),*);
    }};
}

pub fn handle_auth(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    trace!("Started processing auth macro");

    // parse auth
    let attr_span = attr.span();
    let auth = parse2::<ast::Auth>(attr)?;
    let mut allowed_badges = Vec::new();
    for a in auth.allowed {
        if let Some(ident) = a.get_ident() {
            allowed_badges.push(ident.clone());
        } else {
            return Err(Error::new(a.span(), "Only path value is allowed"));
        }
    }

    if allowed_badges.is_empty() {
        return Err(Error::new(
            attr_span,
            "You need to specify at list one auth",
        ));
    }

    // parse function
    let f = parse2::<ItemFn>(item)?;
    let f_attrs = f.attrs;
    let f_vis = f.vis;
    let f_ident = f.sig.ident;
    let mut f_inputs: Vec<FnArg> = f.sig.inputs.iter().map(Clone::clone).collect();
    f_inputs.push(parse_quote! {
        auth: ::scrypto::resource::BucketRef
    });
    let f_output = f.sig.output;
    if let Some(a) = f_attrs
        .iter()
        .find(|a| a.path.get_ident().map(ToString::to_string) == Some("auth".to_string()))
    {
        return Err(Error::new(a.span(), "Only one auth attribute is allowed"));
    }

    // function body
    let f_body = f.block;

    // generate output
    let output = quote! {
        #(#f_attrs)*
        #f_vis fn #f_ident (#(#f_inputs),*) #f_output {
            if !(#(auth.contains(self.#allowed_badges.clone()))||*) {
                panic!("Not authorized!")
            }

            #f_body
        }
    };
    trace!("Finished processing auth macro");

    #[cfg(feature = "trace")]
    crate::utils::print_generated_code("auth", &output);

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
    fn test_auth() {
        let attr = TokenStream::from_str("foo, bar").unwrap();
        let item = TokenStream::from_str("#[other] pub fn x(&self) -> u32 { self.a }").unwrap();
        let output = handle_auth(attr, item).unwrap();

        assert_code_eq(
            output,
            quote! {
                #[other]
                pub fn x(&self, auth: ::scrypto::resource::BucketRef) -> u32 {
                    if !(auth.contains(self.foo.clone()) || auth.contains(self.bar.clone())) {
                        panic!("Not authorized!")
                    }
                    {
                        self.a
                    }
                }
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_double_auth_should_fail() {
        let attr = TokenStream::from_str("simple").unwrap();
        let item = TokenStream::from_str("#[auth(a)] pub fn x(&self) -> u32 { self.a }").unwrap();
        handle_auth(attr, item).unwrap();
    }
}

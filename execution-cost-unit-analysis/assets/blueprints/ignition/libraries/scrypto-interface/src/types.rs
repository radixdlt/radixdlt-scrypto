// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![allow(clippy::type_complexity, clippy::enum_variant_names, dead_code)]

use quote::*;
use syn::parse::*;
use syn::punctuated::*;
use syn::spanned::Spanned;
use syn::token::*;
use syn::*;

#[derive(Clone, Debug)]
pub struct DefineInterfaceInput {
    pub blueprint_ident: Ident,
    pub struct_ident: Option<(Token![as], Ident)>,
    pub generate:
        Option<(Token![impl], Bracket, Punctuated<GenerationItem, Token![,]>)>,
    pub brace: Brace,
    pub signatures: Vec<Signature>,
}

impl DefineInterfaceInput {
    pub fn struct_ident(&self) -> &Ident {
        self.struct_ident
            .as_ref()
            .map(|(_, ident)| ident)
            .unwrap_or(&self.blueprint_ident)
    }
}

impl Parse for DefineInterfaceInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let blueprint_ident = input.parse()?;
        let struct_ident = if input.peek(Token![as]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };
        let generate = if input.peek(Token![impl]) {
            let impl_token = input.parse()?;

            let content;
            let bracket = bracketed!(content in input);

            let inner =
                content.parse_terminated(GenerationItem::parse, Token![,])?;
            Some((impl_token, bracket, inner))
        } else {
            None
        };

        let content;
        let brace = braced!(content in input);
        let mut signatures = vec![];
        while content.peek(Token![fn]) || content.peek(Token![#]) {
            signatures.push(content.parse()?);
        }

        Ok(Self {
            blueprint_ident,
            struct_ident,
            generate,
            brace,
            signatures,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GenerationItem {
    pub attributes: Vec<Attribute>,
    pub generate: Generate,
}

impl Parse for GenerationItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let generate = input.parse::<Generate>()?;
        Ok(Self {
            attributes,
            generate,
        })
    }
}

impl_enum_parse! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Generate {
        Trait,
        ScryptoStub,
        ScryptoTestStub,
        ManifestBuilderStub,
    }
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub attrs: Vec<Attribute>,
    pub token_fn: Token![fn],
    pub ident: Ident,
    pub paren: Paren,
    pub arguments: Arguments,
    pub rtn: ReturnType,
    pub semi_colon: Token![;],
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let token_fn = input.parse()?;
        let ident = input.parse()?;

        let content;
        let paren = parenthesized!(content in input);

        let arguments = content.parse()?;
        let rtn = input.parse()?;
        let semi_colon = input.parse()?;

        Ok(Self {
            attrs,
            token_fn,
            ident,
            paren,
            arguments,
            rtn,
            semi_colon,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    NoReceiver {
        arguments: Punctuated<Argument, Token![,]>,
    },
    SelfReferenceReceiver {
        ampersand_token: Token![&],
        self_token: Token![self],
        arguments: Punctuated<Argument, Token![,]>,
    },
    SelfMutReferenceReceiver {
        ampersand_token: Token![&],
        mut_token: Token![mut],
        self_token: Token![self],
        arguments: Punctuated<Argument, Token![,]>,
    },
}

impl Arguments {
    pub fn arguments(&self) -> &Punctuated<Argument, Token![,]> {
        match self {
            Self::NoReceiver { ref arguments }
            | Self::SelfReferenceReceiver { ref arguments, .. }
            | Self::SelfMutReferenceReceiver { ref arguments, .. } => arguments,
        }
    }

    pub fn arguments_mut(&mut self) -> &mut Punctuated<Argument, Token![,]> {
        match self {
            Self::NoReceiver { ref mut arguments }
            | Self::SelfReferenceReceiver {
                ref mut arguments, ..
            }
            | Self::SelfMutReferenceReceiver {
                ref mut arguments, ..
            } => arguments,
        }
    }

    pub fn manifest_arguments(
        &self,
    ) -> syn::Result<Punctuated<Argument, Token![,]>> {
        let mut list = Punctuated::new();

        for argument in self.arguments() {
            list.push(argument.manifest_argument()?)
        }

        Ok(list)
    }

    pub fn len(&self) -> usize {
        match self {
            Self::NoReceiver { arguments } => arguments.len(),
            Self::SelfReferenceReceiver { arguments, .. }
            | Self::SelfMutReferenceReceiver { arguments, .. } => {
                // Unwrap here is fine, this all happens at compile time. Plus,
                // who would have more than u64 or u32 worth of arguments?
                arguments.len().checked_add(1).unwrap()
            }
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Self::NoReceiver { .. } => true,
            Self::SelfReferenceReceiver { .. }
            | Self::SelfMutReferenceReceiver { .. } => false,
        }
    }

    pub fn is_method(&self) -> bool {
        match self {
            Self::NoReceiver { .. } => false,
            Self::SelfReferenceReceiver { .. }
            | Self::SelfMutReferenceReceiver { .. } => true,
        }
    }

    pub fn arg_idents(&self) -> impl Iterator<Item = &Ident> {
        self.arguments().iter().map(|Argument { ident, .. }| ident)
    }

    pub fn add_argument_to_end(&mut self, ident: Ident, ty: syn::Type) {
        let span = ident.span();
        self.arguments_mut().push(Argument {
            attributes: vec![],
            ident,
            colon: Token![:](span),
            ty,
        })
    }

    pub fn add_argument_to_beginning(&mut self, ident: Ident, ty: syn::Type) {
        let span = ident.span();
        self.arguments_mut().insert(
            0,
            Argument {
                attributes: vec![],
                ident,
                colon: Token![:](span),
                ty,
            },
        )
    }
}

impl ToTokens for Arguments {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let arguments = match self {
            Arguments::NoReceiver { arguments } => arguments,
            Arguments::SelfReferenceReceiver {
                ampersand_token,
                self_token,
                arguments,
            } => {
                ampersand_token.to_tokens(tokens);
                self_token.to_tokens(tokens);
                Token![,](self_token.span()).to_tokens(tokens);
                arguments
            }
            Arguments::SelfMutReferenceReceiver {
                ampersand_token,
                mut_token,
                self_token,
                arguments,
            } => {
                ampersand_token.to_tokens(tokens);
                mut_token.to_tokens(tokens);
                self_token.to_tokens(tokens);
                Token![,](self_token.span()).to_tokens(tokens);
                arguments
            }
        };

        arguments.to_tokens(tokens)
    }
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut arguments = if input.peek(Token![&]) {
            let ampersand_token = input.parse::<Token![&]>()?;

            if input.peek(Token![self]) {
                Self::SelfReferenceReceiver {
                    ampersand_token,
                    self_token: input.parse()?,
                    arguments: Punctuated::new(),
                }
            } else if input.peek(Token![mut]) & input.peek2(Token![self]) {
                Self::SelfMutReferenceReceiver {
                    ampersand_token,
                    mut_token: input.parse()?,
                    self_token: input.parse()?,
                    arguments: Punctuated::new(),
                }
            } else {
                return Err(input.error("Arguments are invalid"));
            }
        } else {
            Self::NoReceiver {
                arguments: Punctuated::new(),
            }
        };

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let args_inner = match arguments {
            Self::NoReceiver { ref mut arguments }
            | Self::SelfReferenceReceiver {
                ref mut arguments, ..
            }
            | Self::SelfMutReferenceReceiver {
                ref mut arguments, ..
            } => arguments,
        };
        *args_inner =
            Punctuated::<Argument, Token![,]>::parse_terminated(input)?;

        Ok(arguments)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument {
    pub attributes: Vec<Attribute>,
    pub ident: Ident,
    pub colon: Token![:],
    pub ty: syn::Type,
}

impl Argument {
    pub fn manifest_type(&self) -> syn::Result<syn::Type> {
        for attribute in self.attributes.iter() {
            if let Meta::NameValue(MetaNameValue {
                ref path,
                ref value,
                ..
            }) = attribute.meta
            {
                if path.is_ident("manifest_type") {
                    let Expr::Lit(ExprLit {
                        lit: Lit::Str(str_lit),
                        ..
                    }) = value
                    else {
                        return Err(syn::Error::new_spanned(
                            value,
                            "Expect this to be a string literal",
                        ));
                    };

                    return str_lit.parse_with(syn::Type::parse);
                }
            }
        }
        Ok(self.ty.clone())
    }

    pub fn manifest_argument(&self) -> syn::Result<Self> {
        let mut argument = self.clone();
        argument.ty = argument.manifest_type()?;
        Ok(argument)
    }
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let ident = input.parse::<Ident>()?;
        let colon = input.parse::<Token![:]>()?;
        let ty = input.parse::<syn::Type>()?;

        Ok(Self {
            attributes,
            ident,
            colon,
            ty,
        })
    }
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parsing_of_no_arguments_produces_a_no_receiver_empty_arguments_list() {
        // Arrange
        let arguments = "";

        // Act
        let arguments =
            parse::<Arguments>(arguments).expect("Parsing must succeed");

        // Assert
        assert_eq!(
            arguments,
            Arguments::NoReceiver {
                arguments: Punctuated::new()
            }
        );
    }

    #[test]
    fn parsing_of_a_self_reference_produced_self_reference_arguments_empty_arguments_list(
    ) {
        // Arrange
        let arguments = "&self";

        // Act
        let arguments =
            parse::<Arguments>(arguments).expect("Parsing must succeed");

        // Assert
        assert!(matches!(
            arguments,
            Arguments::SelfReferenceReceiver { arguments, .. }
                if arguments.is_empty()
        ));
    }

    #[test]
    fn parsing_of_a_mut_self_reference_produced_mut_self_reference_arguments_empty_arguments_list(
    ) {
        // Arrange
        let arguments = "&mut self";

        // Act
        let arguments =
            parse::<Arguments>(arguments).expect("Parsing must succeed");

        // Assert
        assert!(matches!(
            arguments,
            Arguments::SelfMutReferenceReceiver { arguments, .. }
                if arguments.is_empty()
        ));
    }

    #[test]
    fn trailing_comma_in_a_self_reference_is_ignored() {
        // Arrange
        let arguments = "&self,";

        // Act
        let arguments =
            parse::<Arguments>(arguments).expect("Parsing must succeed");

        // Assert
        assert!(matches!(
            arguments,
            Arguments::SelfReferenceReceiver { arguments, .. }
                if arguments.is_empty()
        ));
    }

    #[test]
    fn trailing_comma_in_a_mut_self_reference_is_ignored() {
        // Arrange
        let arguments = "&mut self,";

        // Act
        let arguments =
            parse::<Arguments>(arguments).expect("Parsing must succeed");

        // Assert
        assert!(matches!(
            arguments,
            Arguments::SelfMutReferenceReceiver { arguments, .. }
                if arguments.is_empty()
        ));
    }

    #[test]
    fn multiple_receivers_are_invalid_1() {
        // Arrange
        let arguments = "&self, &self";

        // Act
        let arguments = parse::<Arguments>(arguments);

        // Assert
        assert!(arguments.is_err(), "Must fail");
    }

    #[test]
    fn multiple_receivers_are_invalid_2() {
        // Arrange
        let arguments = "&self, &mut self";

        // Act
        let arguments = parse::<Arguments>(arguments);

        // Assert
        assert!(arguments.is_err(), "Must fail");
    }

    #[test]
    fn parsing_of_arguments_of_no_receiver_succeeds() {
        // Arrange
        let arguments = "name: String, item: u32";

        // Act
        let arguments = parse::<Arguments>(arguments).expect("Can't fail!");

        // Assert
        let Arguments::NoReceiver { arguments } = arguments else {
            panic!("Must succeed!")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(to_string(&arguments[0].ident), "name");
        assert_eq!(to_string(&arguments[0].ty), "String");
        assert_eq!(to_string(&arguments[1].ident), "item");
        assert_eq!(to_string(&arguments[1].ty), "u32");
    }

    #[test]
    fn parsing_of_arguments_of_self_reference_receiver_succeeds() {
        // Arrange
        let arguments = "&self, name: String, item: u32";

        // Act
        let arguments = parse::<Arguments>(arguments).expect("Can't fail!");

        // Assert
        let Arguments::SelfReferenceReceiver { arguments, .. } = arguments
        else {
            panic!("Must succeed!")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(to_string(&arguments[0].ident), "name");
        assert_eq!(to_string(&arguments[0].ty), "String");
        assert_eq!(to_string(&arguments[1].ident), "item");
        assert_eq!(to_string(&arguments[1].ty), "u32");
    }

    #[test]
    fn parsing_of_arguments_of_mut_self_reference_receiver_succeeds() {
        // Arrange
        let arguments = "&mut self, name: String, item: u32";

        // Act
        let arguments = parse::<Arguments>(arguments).expect("Can't fail!");

        // Assert
        let Arguments::SelfMutReferenceReceiver { arguments, .. } = arguments
        else {
            panic!("Must succeed!")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(to_string(&arguments[0].ident), "name");
        assert_eq!(to_string(&arguments[0].ty), "String");
        assert_eq!(to_string(&arguments[1].ident), "item");
        assert_eq!(to_string(&arguments[1].ty), "u32");
    }

    #[test]
    fn parsing_of_arguments_of_self_reference_receiver_with_trailing_comma_succeeds(
    ) {
        // Arrange
        let arguments = "&self, name: String, item: u32,";

        // Act
        let arguments = parse::<Arguments>(arguments).expect("Can't fail!");

        // Assert
        let Arguments::SelfReferenceReceiver { arguments, .. } = arguments
        else {
            panic!("Must succeed!")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(to_string(&arguments[0].ident), "name");
        assert_eq!(to_string(&arguments[0].ty), "String");
        assert_eq!(to_string(&arguments[1].ident), "item");
        assert_eq!(to_string(&arguments[1].ty), "u32");
    }

    #[test]
    fn parsing_of_arguments_of_mut_self_reference_receiver_with_trailing_comma_succeeds(
    ) {
        // Arrange
        let arguments = "&mut self, name: String, item: u32,";

        // Act
        let arguments = parse::<Arguments>(arguments).expect("Can't fail!");

        // Assert
        let Arguments::SelfMutReferenceReceiver { arguments, .. } = arguments
        else {
            panic!("Must succeed!")
        };
        assert_eq!(arguments.len(), 2);
        assert_eq!(to_string(&arguments[0].ident), "name");
        assert_eq!(to_string(&arguments[0].ty), "String");
        assert_eq!(to_string(&arguments[1].ident), "item");
        assert_eq!(to_string(&arguments[1].ty), "u32");
    }

    #[test]
    fn parsing_of_simple_function_succeeds() {
        // Arrange
        let signature = "fn item();";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(signature.arguments.arguments().len(), 0);
        assert_eq!(signature.rtn, ReturnType::Default);
    }

    #[test]
    fn parsing_of_function_with_return_type_succeeds() {
        // Arrange
        let signature = "fn item() -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(signature.arguments.arguments().len(), 0);
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_tuple_return_type_succeeds() {
        // Arrange
        let signature = "fn item() -> (u32, u32);";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(signature.arguments.arguments().len(), 0);
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "(u32 , u32)")
        );
    }

    #[test]
    fn parsing_of_function_with_attributes_succeeds() {
        // Arrange
        let signature = "#[doc = \"Some doc\"] fn item() -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 1);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(signature.arguments.arguments().len(), 0);
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_arguments_succeeds() {
        // Arrange
        let signature = "fn item(value: u32) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_arguments_and_trailing_comma_succeeds() {
        // Arrange
        let signature = "fn item(value: u32,) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_self_reference_with_arguments_succeeds() {
        // Arrange
        let signature = "fn item(&self, value: u32) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_self_reference_with_arguments_and_trailing_comma_succeeds(
    ) {
        // Arrange
        let signature = "fn item(&self, value: u32,) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_mut_self_reference_with_arguments_succeeds() {
        // Arrange
        let signature = "fn item(&mut self, value: u32) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_function_with_mut_self_reference_with_arguments_and_trailing_comma_succeeds(
    ) {
        // Arrange
        let signature = "fn item(&mut self, value: u32,) -> u32;";

        // Act
        let signature = parse::<Signature>(signature).expect("Must succeed!");

        // Assert
        assert_eq!(signature.attrs.len(), 0);
        assert_eq!(to_string(&signature.ident), "item");
        assert_eq!(
            to_string(&signature.arguments.arguments()[0].ident),
            "value"
        );
        assert_eq!(to_string(&signature.arguments.arguments()[0].ty), "u32");
        assert!(
            matches!(signature.rtn, ReturnType::Type(_, item) if to_string(&item) == "u32")
        );
    }

    #[test]
    fn parsing_of_define_interface_succeeds() {
        // Arguments
        let define_interface = r#"
        Blueprint as StructName impl [Trait, ScryptoStub, ScryptoTestStub, ManifestBuilderStub] {
            fn function1();
            fn function2(&self);
            fn function3(&mut self);

            fn function4(item: u32);
            fn function5(&self, item: u32);
            fn function6(&mut self, item: u32);
        }
        "#;

        // Act
        let define_interface = parse::<DefineInterfaceInput>(define_interface)
            .expect("Must succeed!");

        // Assert
        assert_eq!(to_string(&define_interface.blueprint_ident), "Blueprint");
        assert!(define_interface
            .struct_ident
            .is_some_and(|item| to_string(&item.1) == "StructName"));
        assert!(define_interface.generate.is_some_and(|item| item
            .2
            .iter()
            .map(|item| item.generate)
            .collect::<Vec<_>>()
            == vec![
                Generate::Trait,
                Generate::ScryptoStub,
                Generate::ScryptoTestStub,
                Generate::ManifestBuilderStub
            ]))
    }

    #[test]
    fn manifest_types_specified_for_arguments_are_handled_as_expected() {
        // Arrange
        let argument = r#"
        #[manifest_type = "ManifestBucket"]
        bucket: Bucket
        "#;

        // Act
        let argument = parse::<Argument>(argument).expect("Must succeed!");
        let manifest_argument =
            argument.manifest_argument().expect("Must succeed!");

        // Assert
        assert_eq!(to_string(&manifest_argument.ident), "bucket");
        assert_eq!(to_string(&manifest_argument.ty), "ManifestBucket");
    }

    fn parse<T>(input: &str) -> syn::Result<T>
    where
        T: syn::parse::Parse,
    {
        let token_stream =
            proc_macro2::token_stream::TokenStream::from_str(input)?;
        syn::parse2(token_stream)
    }

    fn to_string<T>(item: &T) -> String
    where
        T: ToTokens,
    {
        quote!(#item).to_string()
    }
}

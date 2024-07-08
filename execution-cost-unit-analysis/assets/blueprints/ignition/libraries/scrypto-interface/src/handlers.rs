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

use std::collections::*;

use heck::ToSnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::spanned::*;
use syn::*;

use crate::types::{Signature, *};

pub fn handle_define_interface(
    input: TokenStream2,
) -> syn::Result<TokenStream2> {
    let define_interface = parse2::<DefineInterfaceInput>(input)?;

    let generate = define_interface
        .generate
        .as_ref()
        .map(|(_, _, generate)| generate.iter().cloned().collect())
        .unwrap_or(
            Generate::ALL
                .iter()
                .map(|generate| GenerationItem {
                    attributes: Default::default(),
                    generate: *generate,
                })
                .collect::<HashSet<_>>(),
        );

    let mut generated = vec![];
    for GenerationItem {
        attributes,
        generate,
    } in generate
    {
        match generate {
            Generate::Trait => {
                generated.push(generate_trait(&define_interface, &attributes))
            }
            Generate::ScryptoStub => generated
                .push(generate_scrypto_stub(&define_interface, &attributes)),
            Generate::ScryptoTestStub => generated.push(
                generate_scrypto_test_stub(&define_interface, &attributes),
            ),
            Generate::ManifestBuilderStub => generated.push(
                generate_manifest_builder_stub(&define_interface, &attributes)?,
            ),
        };
    }

    Ok(quote!(
        #(#generated)*
    ))
}

fn generate_trait(
    input: &DefineInterfaceInput,
    attributes: &[Attribute],
) -> TokenStream2 {
    let struct_ident = input.struct_ident();
    let trait_ident = format_ident!("{}InterfaceTrait", struct_ident);

    let signatures = input
        .signatures
        .iter()
        .map(
            |Signature {
                 attrs,
                 token_fn,
                 ident,
                 arguments,
                 rtn,
                 semi_colon,
                 ..
             }| {
                quote! {
                    #(#attrs)*
                    #[allow(clippy::too_many_arguments)]
                    #token_fn #ident ( #arguments ) #rtn #semi_colon
                }
            },
        )
        .collect::<Vec<_>>();

    quote!(
        #(#attributes)*
        pub trait #trait_ident {
            #(#signatures)*
        }
    )
}

fn generate_scrypto_stub(
    input: &DefineInterfaceInput,
    attributes: &[Attribute],
) -> TokenStream2 {
    let struct_ident = input.struct_ident();
    let struct_ident = format_ident!("{}InterfaceScryptoStub", struct_ident);
    let blueprint_ident = &input.blueprint_ident;

    let try_from_impl = [
        "ComponentAddress",
        "ResourceAddress",
        "PackageAddress",
        "InternalAddress",
        "GlobalAddress",
    ]
    .iter()
    .map(|ty| -> syn::Type {
        let ty_ident = Ident::new(ty, blueprint_ident.span());
        parse_quote!(::radix_engine_interface::prelude::#ty_ident)
    })
    .map(|ty| {
        quote! {
            #[allow(clippy::too_many_arguments)]
            impl TryFrom<#struct_ident> for #ty
            {
                type Error = <
                    #ty as TryFrom<::radix_engine_interface::prelude::NodeId>
                >::Error;

                fn try_from(
                    value: #struct_ident
                ) -> Result<Self, Self::Error>
                {
                    <#ty>::try_from(
                        *value.0.as_node_id()
                    )
                }
            }
        }
    })
    .collect::<Vec<_>>();

    let functions = input
        .signatures
        .iter()
        .map(
            |Signature {
                 attrs,
                 token_fn,
                 ident,
                 arguments,
                 rtn,
                 ..
             }| {
                let arg_idents = arguments.arg_idents();

                let mut arguments = arguments.clone();
                if arguments.is_function() {
                    arguments.add_argument_to_end(
                        Ident::new("blueprint_package_address", ident.span()),
                        parse_quote!(::radix_engine_interface::prelude::PackageAddress),
                    );
                }

                let inner = if arguments.is_function() {
                    quote! {
                        let rtn = ::scrypto::prelude::ScryptoVmV1Api::blueprint_call(
                            blueprint_package_address,
                            stringify!(#blueprint_ident),
                            stringify!(#ident),
                            ::radix_common::prelude::scrypto_args!(#(#arg_idents),*)
                        );
                        ::radix_common::prelude::scrypto_decode(&rtn).unwrap()
                    }
                } else {
                    quote! {
                        let rtn = ::scrypto::prelude::ScryptoVmV1Api::object_call(
                            &self.0.0,
                            stringify!(#ident),
                            ::radix_common::prelude::scrypto_args!(#(#arg_idents),*)
                        );
                        ::radix_common::prelude::scrypto_decode(&rtn).unwrap()
                    }
                };

                quote! {
                    #(#attrs)*
                    #[allow(clippy::too_many_arguments)]
                    pub #token_fn #ident ( #arguments ) #rtn {
                        #inner
                    }
                }
            },
        )
        .collect::<Vec<_>>();

    quote! {
        #[derive(
            ::radix_common::prelude::ScryptoSbor,
            ::radix_common::prelude::ManifestSbor,
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash
        )]
        #(#attributes)*
        #[sbor(transparent)]
        pub struct #struct_ident (
            ::radix_common::prelude::Reference
        );

        #(#attributes)*
        #[allow(clippy::too_many_arguments)]
        const _: () = {
            impl<T> From<T> for #struct_ident
            where
                T: ::core::convert::Into<::radix_engine_interface::prelude::NodeId>
            {
                fn from(value: T) -> Self {
                    Self(::radix_common::prelude::Reference(value.into()))
                }
            }

            #(#try_from_impl)*

            #[allow(clippy::too_many_arguments)]
            impl #struct_ident {
                #(#functions)*

                pub fn blueprint_id(
                    package_address: ::radix_engine_interface::prelude::PackageAddress
                ) -> ::radix_engine_interface::prelude::BlueprintId {
                    ::radix_engine_interface::prelude::BlueprintId {
                        package_address,
                        blueprint_name: stringify!(#blueprint_ident).to_string()
                    }
                }
            }
        };
    }
}

fn generate_scrypto_test_stub(
    input: &DefineInterfaceInput,
    attributes: &[Attribute],
) -> TokenStream2 {
    let struct_ident = input.struct_ident();
    let struct_ident =
        format_ident!("{}InterfaceScryptoTestStub", struct_ident);
    let blueprint_ident = &input.blueprint_ident;

    let try_from_impl = [
        "ComponentAddress",
        "ResourceAddress",
        "PackageAddress",
        "InternalAddress",
        "GlobalAddress",
    ]
    .iter()
    .map(|ty| -> syn::Type {
        let ty_ident = Ident::new(ty, blueprint_ident.span());
        parse_quote!(::radix_engine_interface::prelude::#ty_ident)
    })
    .map(|ty| {
        quote! {
            #[allow(clippy::too_many_arguments)]
            impl TryFrom<#struct_ident> for #ty
            {
                type Error = <
                    #ty as TryFrom<::radix_engine_interface::prelude::NodeId>
                >::Error;

                fn try_from(
                    value: #struct_ident
                ) -> Result<Self, Self::Error>
                {
                    <#ty>::try_from(
                        *value.0.as_node_id()
                    )
                }
            }
        }
    })
    .collect::<Vec<_>>();

    let functions = input
        .signatures
        .iter()
        .map(
            |Signature {
                 attrs,
                 token_fn,
                 ident,
                 arguments,
                 rtn,
                 ..
             }| {
                let arg_idents = arguments.arg_idents();

                let mut arguments = arguments.clone();
                if arguments.is_function() {
                    arguments.add_argument_to_end(
                        Ident::new("blueprint_package_address", ident.span()),
                        parse_quote!(::radix_engine_interface::prelude::PackageAddress),
                    );
                }
                arguments
                    .add_argument_to_end(Ident::new("env", ident.span()), parse_quote!(&mut Y));

                let inner = if arguments.is_function() {
                    quote! {
                        env.call_function(
                            blueprint_package_address,
                            stringify!(#blueprint_ident),
                            stringify!(#ident),
                            ::radix_common::prelude::scrypto_args!(#(#arg_idents),*)
                        )
                        .map(|rtn| ::radix_common::prelude::scrypto_decode(&rtn).unwrap())
                    }
                } else {
                    quote! {
                        env.call_method(
                            &self.0.0,
                            stringify!(#ident),
                            ::radix_common::prelude::scrypto_args!(#(#arg_idents),*)
                        )
                        .map(|rtn| ::radix_common::prelude::scrypto_decode(&rtn).unwrap())
                    }
                };

                let rtn = match rtn {
                    ReturnType::Default => parse_quote!(()),
                    ReturnType::Type(_, ty) => *ty.clone(),
                };

                quote! {
                    #(#attrs)*
                    #[allow(clippy::too_many_arguments)]
                    pub #token_fn #ident <Y, E> ( #arguments ) -> Result<#rtn, E>
                    where
                        Y: ::radix_engine_interface::prelude::SystemApi<E>,
                        E: ::core::fmt::Debug
                    {
                        #inner
                    }
                }
            },
        )
        .collect::<Vec<_>>();

    quote! {
        #[derive(
            ::radix_common::prelude::ScryptoSbor,
            ::radix_common::prelude::ManifestSbor,
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash
        )]
        #(#attributes)*
        #[sbor(transparent)]
        pub struct #struct_ident (
            ::radix_common::prelude::Reference
        );

        #(#attributes)*
        #[allow(clippy::too_many_arguments)]
        const _: () = {
            impl<T> From<T> for #struct_ident
            where
                T: ::core::convert::Into<::radix_engine_interface::prelude::NodeId>
            {
                fn from(value: T) -> Self {
                    Self(::radix_common::prelude::Reference(value.into()))
                }
            }

            #(#try_from_impl)*

            #[allow(clippy::too_many_arguments)]
            impl #struct_ident {
                #(#functions)*

                pub fn blueprint_id(
                    package_address: ::radix_engine_interface::prelude::PackageAddress
                ) -> ::radix_engine_interface::prelude::BlueprintId {
                    ::radix_engine_interface::prelude::BlueprintId {
                        package_address,
                        blueprint_name: stringify!(#blueprint_ident).to_string()
                    }
                }
            }
        };
    }
}

fn generate_manifest_builder_stub(
    input: &DefineInterfaceInput,
    attributes: &[Attribute],
) -> syn::Result<TokenStream2> {
    let struct_ident = input.struct_ident();
    let trait_ident =
        format_ident!("{}InterfaceManifestBuilderExtensionTrait", struct_ident);
    let blueprint_ident = &input.blueprint_ident;

    let signatures = input
        .signatures
        .iter()
        .map(
            |Signature {
                 attrs,
                 token_fn,
                 ident,
                 arguments,
                 semi_colon,
                 ..
             }| {
                let mut arguments = arguments.clone();
                if arguments.is_function() {
                    arguments.add_argument_to_beginning(
                        Ident::new("blueprint_package_address", ident.span()),
                        parse_quote!(::radix_engine_interface::prelude::PackageAddress),
                    );
                } else {
                    arguments.add_argument_to_beginning(
                        Ident::new("component_address", ident.span()),
                        parse_quote!(impl ::radix_transactions::builder::ResolvableGlobalAddress),
                    );
                }

                let fn_ident =
                    format_ident!("{}_{}", struct_ident.to_string().to_snake_case(), ident);

                arguments.manifest_arguments().map(|arguments| {
                    quote! {
                        #(#attrs)*
                        #[allow(clippy::too_many_arguments)]
                        #token_fn #fn_ident ( self, #arguments ) -> Self #semi_colon
                    }
                })
            },
        )
        .collect::<syn::Result<Vec<_>>>()?;

    let implementations = input
        .signatures
        .iter()
        .map(
            |Signature {
                 attrs,
                 token_fn,
                 ident,
                 arguments: original_arguments,
                 ..
             }|
             -> syn::Result<TokenStream2> {
                let mut arguments = original_arguments.clone();
                let inner = if arguments.is_function() {
                    arguments.add_argument_to_beginning(
                        Ident::new("blueprint_package_address", ident.span()),
                        parse_quote!(::radix_engine_interface::prelude::PackageAddress),
                    );

                    let original_arguments = original_arguments
                        .manifest_arguments()?
                        .iter()
                        .cloned()
                        .map(|Argument { ident, .. }| ident)
                        .collect::<Vec<_>>();
                    quote! {
                        self.call_function(
                            blueprint_package_address,
                            stringify!(#blueprint_ident),
                            stringify!(#ident),
                            &( #(#original_arguments,)* )
                        )
                    }
                } else {
                    arguments.add_argument_to_beginning(
                        Ident::new("component_address", ident.span()),
                        parse_quote!(impl ::radix_transactions::builder::ResolvableGlobalAddress),
                    );

                    let original_arguments = original_arguments
                        .manifest_arguments()?
                        .iter()
                        .cloned()
                        .map(|Argument { ident, .. }| ident)
                        .collect::<Vec<_>>();
                    quote! {
                        self.call_method(
                            component_address,
                            stringify!(#ident),
                            &( #(#original_arguments,)* )
                        )
                    }
                };

                let fn_ident =
                    format_ident!("{}_{}", struct_ident.to_string().to_snake_case(), ident);

                let arguments = arguments.manifest_arguments()?;
                Ok(quote! {
                    #(#attrs)*
                    #[allow(clippy::too_many_arguments)]
                    #token_fn #fn_ident (self, #arguments) -> Self {
                        #inner
                    }
                })
            },
        )
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote!(
        #(#attributes)*
        pub trait #trait_ident {
            #(#signatures)*
        }

        #(#attributes)*
        #[allow(clippy::too_many_arguments, unused_mut)]
        const _: () = {
            impl #trait_ident for ::radix_transactions::builder::ManifestBuilder {
                #(#implementations)*
            }
        };
    ))
}

pub fn handle_blueprint_with_traits(
    _: TokenStream2,
    item: TokenStream2,
) -> Result<TokenStream2> {
    // Parse the passed token stream as a module. After we do that, we will
    // remove all of the trait impls from inside.
    let span = item.span();
    let mut module = syn::parse2::<ItemMod>(item)?;
    let trait_impls = if let Some((brace, items)) = module.content {
        let (trait_impls, items) =
            items.into_iter().partition::<Vec<_>, _>(|item| {
                matches!(
                    item,
                    Item::Impl(ItemImpl {
                        trait_: Some(_),
                        ..
                    })
                )
            });
        module.content = Some((brace, items));
        trait_impls
    } else {
        vec![]
    };

    // Find the impl block in the module that is not for a trait and then add
    // all of the trait implementations to it.
    if let Some((_, ref mut items)) = module.content {
        let impl_item = items
            .iter_mut()
            .filter_map(|item| {
                if let Item::Impl(item_impl @ ItemImpl { trait_: None, .. }) =
                    item
                {
                    Some(item_impl)
                } else {
                    None
                }
            })
            .next()
            .ok_or(syn::Error::new(
                span,
                "No impl block found that is not for a trait",
            ))?;

        for trait_impl_item in trait_impls.iter() {
            let Item::Impl(ItemImpl { items, .. }) = trait_impl_item else {
                continue;
            };

            // Make any item that accepts a vis become public
            let items = items
                .iter()
                .cloned()
                .map(|item| match item {
                    ImplItem::Const(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Const(item)
                    }
                    ImplItem::Fn(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Fn(item)
                    }
                    ImplItem::Type(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Type(item)
                    }
                    item @ ImplItem::Macro(..)
                    | item @ ImplItem::Verbatim(..) => item,
                    _ => todo!(),
                })
                .collect::<Vec<_>>();

            impl_item.items.extend(items)
        }
    }

    if let Some((_, ref items)) = module.content {
        // Getting the name of the blueprint by finding the first struct item we
        // find inside the module.
        let blueprint_ident = items
            .iter()
            .filter_map(|item| {
                if let Item::Struct(ItemStruct { ident, .. }) = item {
                    Some(ident)
                } else {
                    None
                }
            })
            .next()
            .ok_or(syn::Error::new(
                span,
                "No struct item found inside of module",
            ))?;

        let unreachable_trait_impls = trait_impls
            .clone()
            .into_iter()
            .filter_map(|item| {
                if let Item::Impl(item) = item {
                    Some(item)
                } else {
                    None
                }
            })
            .map(|mut impl_item| {
                impl_item.items = impl_item
                    .items
                    .into_iter()
                    .map(|mut impl_item| {
                        if let ImplItem::Fn(ref mut func_impl_item) = impl_item
                        {
                            func_impl_item.block =
                                parse_quote!({ unreachable!() });
                        };
                        impl_item
                    })
                    .collect();
                impl_item
            });

        // The module should now be a perfectly well structured blueprint that
        // is ready to go through the blueprint code generation process.
        Ok(quote::quote! {
            #[::scrypto::prelude::blueprint]
            #module

            #[allow(clippy::too_many_arguments, unused_mut)]
            const _: () = {
                struct #blueprint_ident;

                #[allow(unused_variables, unused_mut)]
                #(#unreachable_trait_impls)*
            };
        })
    } else {
        // The module should now be a perfectly well structured blueprint that
        // is ready to go through the blueprint code generation process.
        Ok(quote::quote! {
            #[::scrypto::prelude::blueprint]
            #module
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn blueprint_with_trait_generates_expected_code() {
        // Arrange
        let input = r#"
        #[blueprint_with_traits]
        mod blueprint{
            struct Blueprint;

            impl Blueprint {}

            impl MyTrait for Blueprint {
                fn func1() {
                    todo!("func1");
                }
                fn func2(item: u32) {
                    todo!("func2");
                }
                fn func3() -> u32 {
                    todo!("func3");
                }
                fn func4(item: u32) -> u32 {
                    todo!("func4");
                }

                fn ref_method1(&self) {
                    todo!("ref_method1");
                }
                fn ref_method2(&self, item: u32) {
                    todo!("ref_method2");
                }
                fn ref_method3(&self) -> u32 {
                    todo!("ref_method3");
                }
                fn ref_method4(&self, item: u32) -> u32 {
                    todo!("ref_method4");
                }

                fn mut_ref_method1(&mut self) {
                    todo!("mut_ref_method1");
                }
                fn mut_ref_method2(&mut self, item: u32) {
                    todo!("mut_ref_method2");
                }
                fn mut_ref_method3(&mut self) -> u32 {
                    todo!("mut_ref_method3");
                }
                fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    todo!("mut_ref_method4");
                }
            }
        }
        "#;
        let expected_output = r#"
        #[::scrypto::prelude::blueprint] 
        #[blueprint_with_traits]
        mod blueprint{
            struct Blueprint;

            impl Blueprint {
                pub fn func1() {
                    todo!("func1");
                }
                pub fn func2(item: u32) {
                    todo!("func2");
                }
                pub fn func3() -> u32 {
                    todo!("func3");
                }
                pub fn func4(item: u32) -> u32 {
                    todo!("func4");
                }

                pub fn ref_method1(&self) {
                    todo!("ref_method1");
                }
                pub fn ref_method2(&self, item: u32) {
                    todo!("ref_method2");
                }
                pub fn ref_method3(&self) -> u32 {
                    todo!("ref_method3");
                }
                pub fn ref_method4(&self, item: u32) -> u32 {
                    todo!("ref_method4");
                }

                pub fn mut_ref_method1(&mut self) {
                    todo!("mut_ref_method1");
                }
                pub fn mut_ref_method2(&mut self, item: u32) {
                    todo!("mut_ref_method2");
                }
                pub fn mut_ref_method3(&mut self) -> u32 {
                    todo!("mut_ref_method3");
                }
                pub fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    todo!("mut_ref_method4");
                }
            }
        }

        #[allow(clippy::too_many_arguments, unused_mut)]
        const _: () = {
            struct Blueprint;

            #[allow (unused_variables, unused_mut)]
            impl MyTrait for Blueprint {
                fn func1() {
                    unreachable!()
                }
                fn func2(item: u32) {
                    unreachable!()
                }
                fn func3() -> u32 {
                    unreachable!()
                }
                fn func4(item: u32) -> u32 {
                    unreachable!()
                }

                fn ref_method1(&self) {
                    unreachable!()
                }
                fn ref_method2(&self, item: u32) {
                    unreachable!()
                }
                fn ref_method3(&self) -> u32 {
                    unreachable!()
                }
                fn ref_method4(&self, item: u32) -> u32 {
                    unreachable!()
                }

                fn mut_ref_method1(&mut self) {
                    unreachable!()
                }
                fn mut_ref_method2(&mut self, item: u32) {
                    unreachable!()
                }
                fn mut_ref_method3(&mut self) -> u32 {
                    unreachable!()
                }
                fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    unreachable!()
                }
            }
        };
        "#;

        // Act
        let output = handle_blueprint_with_traits(
            TokenStream2::from_str("").unwrap(),
            TokenStream2::from_str(input).unwrap(),
        )
        .unwrap();

        // Assert
        assert_eq!(
            output.to_string(),
            TokenStream2::from_str(expected_output).unwrap().to_string()
        );
    }

    #[test]
    fn simple_define_interface_works_as_expected() {
        // Arrange
        let define_interface = r#"
        Blueprint as StructName {
            fn func1();
            fn func2(&self) -> u32;
            fn func3(&mut self, item: u32) -> (u32, u32);
        }
        "#;

        // Act
        let rtn = handle_define_interface(
            TokenStream2::from_str(define_interface).unwrap(),
        );

        // Assert
        rtn.expect("Interface has been defined successfully!");
    }
}

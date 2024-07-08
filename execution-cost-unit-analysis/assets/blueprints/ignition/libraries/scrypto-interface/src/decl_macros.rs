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

macro_rules! impl_enum_parse {
    (
        $(#[$meta: meta])*
        $vis: vis enum $ident: ident {
            $(
                $variant: ident
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $ident {
            $(
                $variant
            ),*
        }

        const _: () = {
            impl ::core::convert::TryFrom<::syn::Ident> for $ident {
                type Error = ::syn::Error;

                fn try_from(ident: ::syn::Ident) -> ::syn::Result<$ident> {
                    match ident.to_string().as_str() {
                        $(
                            stringify!($variant) => Ok(Self::$variant),
                        )*
                        _ => Err(::syn::Error::new(
                            ident.span(),
                            format!("\"{}\" is not a valid \"{}\". Valid values are: {:?}", ident, stringify!($ident), $ident::STRINGS)
                        ))
                    }
                }
            }

            impl $ident {
                pub const STRINGS: &'static [&'static str] = &[
                    $(
                        stringify!($variant)
                    ),*
                ];

                pub const ALL: &'static [$ident] = &[
                    $(
                        Self::$variant
                    ),*
                ];
            }

            impl ::syn::parse::Parse for $ident {
                fn parse(input: ParseStream) -> Result<Self> {
                    <$ident>::try_from(Ident::parse(input)?)
                }
            }
        };
    };
}

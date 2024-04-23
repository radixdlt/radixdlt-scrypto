use core::iter;
use proc_macro2::*;
use std::{collections::HashMap, str::FromStr};
use syn::*;

type TokenIter = <TokenStream as IntoIterator>::IntoIter;
type PeekableTokenIter = iter::Peekable<TokenIter>;

pub(crate) fn replace(token_stream: TokenStream) -> Result<TokenStream> {
    let settings = Settings {
        tag: "EAGER".to_string(),
    };
    let mut state = EagerState::new();
    replace_recursive(&settings, &mut state, token_stream.into_iter())
}

struct Settings {
    tag: String,
}

struct EagerState {
    variables: HashMap<String, TokenStream>,
}

impl EagerState {
    fn new() -> Self {
        Self {
            variables: Default::default(),
        }
    }

    fn set_variable(&mut self, name: String, tokens: TokenStream) {
        self.variables.insert(name, tokens);
    }

    fn get_variable(&self, name: &str) -> Option<&TokenStream> {
        self.variables.get(name)
    }
}

fn replace_recursive(
    settings: &Settings,
    state: &mut EagerState,
    token_iter: TokenIter,
) -> Result<TokenStream> {
    let mut tokens = token_iter.peekable();
    let mut expanded = TokenStream::new();
    let tag = &settings.tag;
    loop {
        match consume_next_meaningful_token_batch(&mut tokens, tag)? {
            MeaningfulTokenBatch::EagerCallStart(call_kind_group, eager_call_intent) => {
                let call_output =
                    execute_eager_call(settings, state, eager_call_intent, call_kind_group.span())?;
                expanded.extend(call_output);
            }
            MeaningfulTokenBatch::EagerVariable { marker, name } => {
                let Some(substituted) = state.get_variable(&name.to_string()) else {
                    let marker = marker.as_char();
                    let name_str = name.to_string();
                    let name_str = &name_str;
                    return Err(Error::new(
                        name.span(),
                        format!("The variable {marker}{name_str} wasn't set. If this wasn't intended to be a variable, work around this with {marker}[!{tag}!]({name_str})"),
                    ));
                };
                expanded.extend(substituted.clone());
            }
            MeaningfulTokenBatch::Group(group) => {
                // If it's a group, run replace on its contents recursively.
                expanded.extend(iter::once(TokenTree::Group(Group::new(
                    group.delimiter(),
                    replace_recursive(settings, state, group.stream().into_iter())?,
                ))));
            }
            MeaningfulTokenBatch::Leaf(token_tree) => {
                expanded.extend(iter::once(token_tree));
            }
            MeaningfulTokenBatch::EndOfStream => break,
        }
    }
    return Ok(expanded);
}

enum MeaningfulTokenBatch {
    EagerCallStart(Group, EagerCallIntent),
    EagerVariable { marker: Punct, name: Ident },
    Group(Group),
    Leaf(TokenTree),
    EndOfStream,
}

fn consume_next_meaningful_token_batch(
    tokens: &mut PeekableTokenIter,
    tag: &str,
) -> Result<MeaningfulTokenBatch> {
    Ok(match tokens.next() {
        None => MeaningfulTokenBatch::EndOfStream,
        Some(TokenTree::Group(group)) => {
            if let Some(eager_call_intent) = denotes_eager_call_intent(tag, &group)? {
                MeaningfulTokenBatch::EagerCallStart(group, eager_call_intent)
            } else {
                MeaningfulTokenBatch::Group(group)
            }
        }
        Some(TokenTree::Punct(punct)) => {
            if punct.as_char() == '#' {
                if let Some(TokenTree::Ident(_)) = tokens.peek() {
                    let Some(TokenTree::Ident(name)) = tokens.next() else {
                        unreachable!();
                    };
                    MeaningfulTokenBatch::EagerVariable {
                        marker: punct,
                        name,
                    }
                } else {
                    MeaningfulTokenBatch::Leaf(TokenTree::Punct(punct))
                }
            } else {
                MeaningfulTokenBatch::Leaf(TokenTree::Punct(punct))
            }
        }
        Some(leaf) => MeaningfulTokenBatch::Leaf(leaf),
    })
}

enum EagerIntentKind {
    Output(EagerFunctionKind),
    Set(EagerFunctionKind),
}
enum EagerFunctionKind {
    Stringify,
    Concat,
    Ident,
    Literal,
    Tokens,
}

struct EagerCallIntent {
    intent_kind: EagerIntentKind,
    args: TokenIter,
}

fn denotes_eager_call_intent<'g>(tag: &str, group: &'g Group) -> Result<Option<EagerCallIntent>> {
    // Until we see [!EAGER...] we will assume we're not matching an eager call.
    if group.delimiter() != Delimiter::Bracket {
        return Ok(None);
    }
    let mut tokens = group.stream().into_iter();
    if consume_expected_punct(&mut tokens, '!').is_none() {
        return Ok(None);
    }
    if consume_expected_ident(&mut tokens, tag).is_none() {
        return Ok(None);
    }
    // We have now established we're in an eager call intent.
    // Anything wrong after this point is a compile error.

    let Some(TokenTree::Punct(punct)) = tokens.next() else {
        return Err(eager_intent_error_for_tag(group.span(), tag));
    };

    let intent_kind = match punct.as_char() {
        // [!EAGER! ..] is interpreted as a pass-through of tokens, but doing replacements
        '!' => EagerIntentKind::Output(EagerFunctionKind::Tokens),
        ':' => {
            let Some(TokenTree::Ident(call_type)) = tokens.next() else {
                return Err(eager_intent_error_for_tag(group.span(), tag));
            };
            if &call_type.to_string() == "set" {
                let Some(TokenTree::Punct(punct)) = tokens.next() else {
                    return Err(eager_intent_error_for_tag(group.span(), tag));
                };
                match punct.as_char() {
                    '!' => EagerIntentKind::Set(EagerFunctionKind::Tokens),
                    ':' => {
                        let Some(TokenTree::Ident(func_name)) = tokens.next() else {
                            return Err(eager_intent_error_for_tag(group.span(), tag));
                        };
                        let intent_kind =
                            EagerIntentKind::Set(parse_supported_func_name(&func_name)?);
                        if consume_expected_punct(&mut tokens, '!').is_none() {
                            return Err(eager_intent_error_for_tag(group.span(), tag));
                        }
                        intent_kind
                    }
                    _ => return Err(eager_intent_error_for_tag(group.span(), tag)),
                }
            } else {
                let intent_kind = EagerIntentKind::Output(parse_supported_func_name(&call_type)?);
                if consume_expected_punct(&mut tokens, '!').is_none() {
                    return Err(eager_intent_error_for_tag(group.span(), tag));
                }
                intent_kind
            }
        }
        _ => return Err(eager_intent_error_for_tag(group.span(), tag)),
    };

    Ok(Some(EagerCallIntent {
        intent_kind,
        args: tokens,
    }))
}

fn eager_intent_error_for_tag(span: Span, tag: &str) -> Error {
    Error::new(
        span,
        format!("Expected `[!{tag}! ..]`, `[!{tag}:<func>! ..]`, `[!{tag}:set! ..]`, `[!{tag}:set:<func>! #var = ..]` for <func> one of: stringify, concat, ident or literal.")
    )
}

fn parse_supported_func_name(ident: &Ident) -> Result<EagerFunctionKind> {
    Ok(match ident.to_string().as_ref() {
        "stringify" => EagerFunctionKind::Stringify,
        "concat" => EagerFunctionKind::Concat,
        "ident" => EagerFunctionKind::Ident,
        "literal" => EagerFunctionKind::Literal,
        "tokens" => EagerFunctionKind::Tokens,
        func => {
            return Err(Error::new(
                ident.span(),
                format!("Unknown EAGER function: {func}"),
            ))
        }
    })
}

fn consume_expected_ident(tokens: &mut TokenIter, ident_str: &str) -> Option<Ident> {
    let Some(TokenTree::Ident(ident)) = tokens.next() else {
        return None;
    };
    if &ident.to_string() != ident_str {
        return None;
    }
    Some(ident)
}

fn consume_expected_punct(tokens: &mut TokenIter, char: char) -> Option<Punct> {
    let Some(TokenTree::Punct(punct)) = tokens.next() else {
        return None;
    };
    if punct.as_char() != char {
        return None;
    }
    Some(punct)
}

fn execute_eager_call(
    settings: &Settings,
    state: &mut EagerState,
    call_intent: EagerCallIntent,
    span: Span,
) -> Result<TokenStream> {
    match call_intent.intent_kind {
        EagerIntentKind::Output(func) => {
            execute_eager_function(settings, state, func, span, call_intent.args)
        }
        EagerIntentKind::Set(func) => {
            let mut tokens = call_intent.args;
            const SET_ERROR_MESSAGE: &'static str =
                "A set call is expected to start with `#VariableName = ..`.";
            match consume_expected_punct(&mut tokens, '#') {
                Some(_) => {}
                _ => return Err(Error::new(span, SET_ERROR_MESSAGE)),
            }
            let Some(TokenTree::Ident(ident)) = tokens.next() else {
                return Err(Error::new(span, SET_ERROR_MESSAGE));
            };
            match consume_expected_punct(&mut tokens, '=') {
                Some(_) => {}
                _ => return Err(Error::new(span, SET_ERROR_MESSAGE)),
            }

            let result_tokens = execute_eager_function(settings, state, func, span, tokens)?;
            state.set_variable(ident.to_string(), result_tokens);

            return Ok(TokenStream::new());
        }
    }
}

fn execute_eager_function(
    settings: &Settings,
    state: &mut EagerState,
    function_kind: EagerFunctionKind,
    span: Span,
    token_iter: TokenIter,
) -> Result<TokenStream> {
    let replaced_arguments = replace_recursive(settings, state, token_iter)?;
    Ok(match function_kind {
        EagerFunctionKind::Stringify => stringify(span, replaced_arguments)?,
        EagerFunctionKind::Concat => concat(span, replaced_arguments)?,
        EagerFunctionKind::Ident => concat_ident(span, replaced_arguments)?,
        EagerFunctionKind::Literal => concat_literal(span, replaced_arguments)?,
        EagerFunctionKind::Tokens => replaced_arguments,
    })
}

fn stringify(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let stringify_str = arguments.to_string();
    let mut literal = Literal::string(&stringify_str);
    literal.set_span(span);
    Ok(TokenTree::Literal(literal).into())
}

fn concat(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let concat_str = concat_internal(arguments);
    let mut literal = Literal::string(&concat_str);
    literal.set_span(span);
    Ok(TokenTree::Literal(literal).into())
}

fn concat_ident(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let concat_str = concat_internal(arguments);
    // As per paste
    let ident = match std::panic::catch_unwind(|| Ident::new(&concat_str, span)) {
        Ok(literal) => literal,
        Err(_) => {
            return Err(Error::new(
                span,
                &format!("`{:?}` is not a valid ident", concat_str),
            ));
        }
    };
    Ok(TokenTree::Ident(ident).into())
}

fn concat_literal(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let concat_str = concat_internal(arguments);
    // Similar to paste
    let mut literal = Literal::from_str(&concat_str)
        .map_err(|_| Error::new(span, &format!("`{:?}` is not a valid literal", concat_str)))?;
    literal.set_span(span);
    Ok(TokenTree::Literal(literal).into())
}

fn concat_internal(arguments: TokenStream) -> String {
    let mut output_str = String::new();
    for token_tree in arguments {
        match token_tree {
            TokenTree::Literal(literal) => {
                let lit: Lit = parse_quote!(#literal);
                match lit {
                    Lit::Str(lit_str) => output_str.push_str(&lit_str.value()),
                    Lit::Char(lit_char) => output_str.push(lit_char.value()),
                    _ => {
                        output_str.push_str(&literal.to_string());
                    }
                }
            }
            _ => output_str.push_str(&token_tree.to_string()),
        }
    }
    output_str
}

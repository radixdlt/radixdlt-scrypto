use core::iter;
use proc_macro2::*;
use std::{collections::HashMap, str::FromStr};
use syn::*;

type TokenIter = <TokenStream as IntoIterator>::IntoIter;
type PeekableTokenIter = iter::Peekable<TokenIter>;

pub(crate) fn replace(token_stream: TokenStream) -> Result<TokenStream> {
    let mut state = EagerState::new();
    replace_recursive(&mut state, token_stream.into_iter())
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

fn replace_recursive(state: &mut EagerState, token_iter: TokenIter) -> Result<TokenStream> {
    let mut tokens = token_iter.peekable();
    let mut expanded = TokenStream::new();
    loop {
        match consume_next_meaningful_token_batch(&mut tokens)? {
            MeaningfulTokenBatch::EagerCallStart(call_kind_group, eager_call_intent) => {
                let call_output =
                    execute_eager_call(state, eager_call_intent, call_kind_group.span())?;
                expanded.extend(call_output);
            }
            MeaningfulTokenBatch::EagerVariable { marker, name } => {
                let Some(substituted) = state.get_variable(&name.to_string()) else {
                    let marker = marker.as_char();
                    let name_str = name.to_string();
                    let name_str = &name_str;
                    return Err(Error::new(
                        name.span(),
                        format!("The variable {marker}{name_str} wasn't set. If this wasn't intended to be a variable, work around this with [!raw! {marker}{name_str}]"),
                    ));
                };
                expanded.extend(substituted.clone());
            }
            MeaningfulTokenBatch::Group(group) => {
                // If it's a group, run replace on its contents recursively.
                expanded.extend(iter::once(TokenTree::Group(Group::new(
                    group.delimiter(),
                    replace_recursive(state, group.stream().into_iter())?,
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
) -> Result<MeaningfulTokenBatch> {
    Ok(match tokens.next() {
        None => MeaningfulTokenBatch::EndOfStream,
        Some(TokenTree::Group(group)) => {
            if let Some(eager_call_intent) = denotes_eager_call_intent(&group)? {
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
    ProcessedTokens,
    RawTokens,
}

struct EagerCallIntent {
    intent_kind: EagerIntentKind,
    args: TokenIter,
}

fn denotes_eager_call_intent<'g>(group: &'g Group) -> Result<Option<EagerCallIntent>> {
    if group.delimiter() != Delimiter::Bracket {
        return Ok(None);
    }

    let mut tokens = group.stream().into_iter();
    if consume_expected_punct(&mut tokens, '!').is_none() {
        return Ok(None);
    }
    let Some(TokenTree::Ident(call_ident)) = tokens.next() else {
        return Ok(None);
    };

    // We have now checked enough that we're confident the user is pretty intentionally using
    // the call convention. Any issues we hit from this point will be a helpful compiler error.
    let intent_kind = match call_ident.to_string().as_ref() {
        "SET" => {
            let Some(TokenTree::Punct(punct)) = tokens.next() else {
                return Err(eager_call_intent_error(group.span()));
            };
            match punct.as_char() {
                '!' => EagerIntentKind::Set(EagerFunctionKind::ProcessedTokens),
                ':' => {
                    let Some(TokenTree::Ident(func_name)) = tokens.next() else {
                        return Err(eager_call_intent_error(group.span()));
                    };
                    let intent_kind = EagerIntentKind::Set(parse_supported_func_name(&func_name)?);
                    if consume_expected_punct(&mut tokens, '!').is_none() {
                        return Err(eager_call_intent_error(group.span()));
                    }
                    intent_kind
                }
                _ => return Err(eager_call_intent_error(group.span())),
            }
        }
        _ => {
            let intent_kind = EagerIntentKind::Output(parse_supported_func_name(&call_ident)?);
            if consume_expected_punct(&mut tokens, '!').is_none() {
                return Err(eager_call_intent_error(group.span()));
            }
            intent_kind
        }
    };

    Ok(Some(EagerCallIntent {
        intent_kind,
        args: tokens,
    }))
}

fn eager_call_intent_error(span: Span) -> Error {
    Error::new(
        span,
        "Expected `[!<func>! ..]`, `[!SET! #var = ..]` or `[!SET:<func>! #var = ..]` for <func> one of: stringify, concat, ident, literal or raw.",
    )
}

fn parse_supported_func_name(ident: &Ident) -> Result<EagerFunctionKind> {
    Ok(match ident.to_string().as_ref() {
        "stringify" => EagerFunctionKind::Stringify,
        "concat" => EagerFunctionKind::Concat,
        "ident" => EagerFunctionKind::Ident,
        "literal" => EagerFunctionKind::Literal,
        "raw" => EagerFunctionKind::RawTokens,
        func => {
            return Err(Error::new(
                ident.span(),
                format!("Unknown function: {func}"),
            ))
        }
    })
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
    state: &mut EagerState,
    call_intent: EagerCallIntent,
    span: Span,
) -> Result<TokenStream> {
    match call_intent.intent_kind {
        EagerIntentKind::Output(func) => {
            execute_eager_function(state, func, span, call_intent.args)
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

            let result_tokens = execute_eager_function(state, func, span, tokens)?;
            state.set_variable(ident.to_string(), result_tokens);

            return Ok(TokenStream::new());
        }
    }
}

fn execute_eager_function(
    state: &mut EagerState,
    function_kind: EagerFunctionKind,
    span: Span,
    token_iter: TokenIter,
) -> Result<TokenStream> {
    Ok(match function_kind {
        EagerFunctionKind::Stringify => stringify(span, replace_recursive(state, token_iter)?)?,
        EagerFunctionKind::Concat => concat(span, replace_recursive(state, token_iter)?)?,
        EagerFunctionKind::Ident => concat_ident(span, replace_recursive(state, token_iter)?)?,
        EagerFunctionKind::Literal => concat_literal(span, replace_recursive(state, token_iter)?)?,
        EagerFunctionKind::ProcessedTokens => replace_recursive(state, token_iter)?,
        EagerFunctionKind::RawTokens => token_iter.collect(),
    })
}

fn stringify(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let output = arguments.to_string();
    Ok(str_literal_token_stream(span, &output))
}

fn concat(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let mut output = String::new();
    concat_recursive_internal(&mut output, arguments);
    Ok(str_literal_token_stream(span, &output))
}

fn str_literal_token_stream(span: Span, content: &str) -> TokenStream {
    let mut literal = Literal::string(content);
    literal.set_span(span);
    TokenTree::Literal(literal).into()
}

fn concat_ident(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let mut output = String::new();
    concat_recursive_internal(&mut output, arguments);
    // As per paste
    let ident = match std::panic::catch_unwind(|| Ident::new(&output, span)) {
        Ok(literal) => literal,
        Err(_) => {
            return Err(Error::new(
                span,
                &format!("`{:?}` is not a valid ident", output),
            ));
        }
    };
    Ok(TokenTree::Ident(ident).into())
}

fn concat_literal(span: Span, arguments: TokenStream) -> Result<TokenStream> {
    let mut output = String::new();
    concat_recursive_internal(&mut output, arguments);
    let mut literal = Literal::from_str(&output)
        .map_err(|_| Error::new(span, &format!("`{:?}` is not a valid literal", output)))?;
    literal.set_span(span);
    Ok(TokenTree::Literal(literal).into())
}

fn concat_recursive_internal(output: &mut String, arguments: TokenStream) {
    for token_tree in arguments {
        match token_tree {
            TokenTree::Literal(literal) => {
                let lit: Lit = parse_quote!(#literal);
                match lit {
                    Lit::Str(lit_str) => output.push_str(&lit_str.value()),
                    Lit::Char(lit_char) => output.push(lit_char.value()),
                    _ => {
                        output.push_str(&literal.to_string());
                    }
                }
            }
            TokenTree::Group(group) => match group.delimiter() {
                Delimiter::Parenthesis => {
                    output.push('(');
                    concat_recursive_internal(output, group.stream());
                    output.push(')');
                }
                Delimiter::Brace => {
                    output.push('{');
                    concat_recursive_internal(output, group.stream());
                    output.push('}');
                }
                Delimiter::Bracket => {
                    output.push('[');
                    concat_recursive_internal(output, group.stream());
                    output.push(']');
                }
                Delimiter::None => {
                    concat_recursive_internal(output, group.stream());
                }
            },
            TokenTree::Punct(punct) => {
                output.push(punct.as_char());
            }
            TokenTree::Ident(ident) => output.push_str(&ident.to_string()),
        }
    }
}

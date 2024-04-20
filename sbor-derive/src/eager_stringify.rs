use proc_macro::*;

pub(crate) fn replace_recursive(token_stream: TokenStream) -> TokenStream {
    let mut tokens = token_stream.into_iter().peekable();
    let mut expanded = TokenStream::new();
    loop {
        let Some(token_tree) = tokens.next() else {
            break;
        };

        if let Some(eager_stringify_ident_span) =
            is_eager_stringify_followed_by_exclamation_mark(&token_tree, &mut tokens)
        {
            let exclamation_mark = tokens.next().unwrap();
            let Some(next_token_tree) = tokens.next() else {
                break;
            };
            if let proc_macro::TokenTree::Group(group) = next_token_tree {
                expanded.extend(stringify_tokens(eager_stringify_ident_span, group.stream()));
            } else {
                // If we get eager_stringify! but then it doesn't get followed by a group, then add the token back which we've just consumed
                expanded.extend(core::iter::once(token_tree));
                expanded.extend(core::iter::once(exclamation_mark));
                expanded.extend(core::iter::once(next_token_tree));
            }
        } else {
            if let proc_macro::TokenTree::Group(group) = token_tree {
                expanded.extend(core::iter::once(TokenTree::Group(proc_macro::Group::new(
                    group.delimiter(),
                    replace_recursive(group.stream()),
                ))))
            } else {
                expanded.extend(core::iter::once(token_tree));
            }
        }
    }
    return expanded;
}

fn is_eager_stringify_followed_by_exclamation_mark(
    current: &TokenTree,
    tokens: &mut core::iter::Peekable<<TokenStream as IntoIterator>::IntoIter>,
) -> Option<Span> {
    let TokenTree::Ident(ident) = &current else {
        return None;
    };
    if ident.to_string() != "eager_stringify" {
        return None;
    }
    let Some(TokenTree::Punct(punct)) = tokens.peek() else {
        return None;
    };
    if punct.as_char() != '!' {
        return None;
    }
    Some(ident.span())
}

fn stringify_tokens(span: Span, token_stream: TokenStream) -> TokenStream {
    let mut literal = Literal::string(&token_stream.to_string());
    literal.set_span(span);
    TokenTree::Literal(literal).into()
}

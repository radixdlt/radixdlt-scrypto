use proc_macro::*;

pub(crate) fn replace_recursive(token_stream: TokenStream) -> TokenStream {
    let mut tokens = token_stream.into_iter().peekable();
    let mut expanded = TokenStream::new();
    loop {
        let Some(token_tree) = tokens.next() else {
            break;
        };

        if is_eager_stringify_followed_by_exclamation_mark(&token_tree, &mut tokens) {
            let exclamation_mark = tokens.next().unwrap();
            let next_token_tree = tokens.next();
            if let Some(proc_macro::TokenTree::Group(group)) = &next_token_tree {
                expanded.extend(stringify_tokens(group.span(), group.stream()));
            } else {
                // If we get eager_stringify! but then it doesn't get followed by a group, then add the token back which we've just consumed
                expanded.extend(core::iter::once(token_tree));
                expanded.extend(core::iter::once(exclamation_mark));
                if let Some(next_token_tree) = next_token_tree {
                    expanded.extend(core::iter::once(next_token_tree));
                } else {
                    break;
                }
            }
        } else {
            // If it's a group, run replace on its contents recursively.
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
) -> bool {
    let TokenTree::Ident(ident) = &current else {
        return false;
    };
    if ident.to_string() != "eager_stringify" {
        return false;
    }
    let Some(TokenTree::Punct(punct)) = tokens.peek() else {
        return false;
    };
    if punct.as_char() != '!' {
        return false;
    }
    true
}

fn stringify_tokens(span: Span, token_stream: TokenStream) -> TokenStream {
    let mut literal = Literal::string(&token_stream.to_string());
    literal.set_span(span);
    TokenTree::Literal(literal).into()
}

use proc_macro::*;

///impl macros that only accepts serial parameters.
///Use `impl_with_tuples!(macro_name,start,end,generic_name_prefix)`
#[proc_macro]
pub fn impl_with_tuples(input: TokenStream) -> TokenStream {
    let mut token_trees = input.into_iter();
    //Gets macro.
    let macro_ident = match token_trees.next() {
        Some(token_tree) => match token_tree {
            m @ TokenTree::Ident(_) => m,
            _ => panic!(),
        },
        _ => panic!(),
    };

    fn check_comma(token_tree: Option<TokenTree>) {
        match token_tree {
            Some(token_tree) => match token_tree {
                TokenTree::Punct(p) if p == ',' => {}
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    check_comma(token_trees.next());

    fn parse_usize(token_tree: Option<TokenTree>) -> usize {
        match token_tree {
            Some(token_tree) => match token_tree {
                TokenTree::Literal(l) => l.to_string().parse::<usize>().unwrap(),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    let start = parse_usize(token_trees.next());
    check_comma(token_trees.next());

    let end = parse_usize(token_trees.next());
    check_comma(token_trees.next());
    //Gets generic prefix.
    let generic = match token_trees.next() {
        Some(token_tree) => match token_tree {
            TokenTree::Ident(i) => i.to_string(),
            _ => panic!(),
        },
        _ => panic!(),
    };

    //output
    let mut new_stream = Vec::<TokenTree>::with_capacity((end - start) * (end - start + 1) / 2);
    for i in start..=end {
        //Write macro call.
        new_stream.push(macro_ident.clone());
        new_stream.push(TokenTree::Punct(Punct::new('!', Spacing::Alone)));
        //Contents inner macro call parenthesis.
        let mut call_stream = Vec::<TokenTree>::with_capacity(i);
        //Writes generic name from prefix and index.
        for j in start..i {
            call_stream.push(TokenTree::Ident(Ident::new(
                &format!("{generic}{j}"),
                Span::call_site(),
            )));
            if j < i - 1 {
                call_stream.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
            }
        }
        //Wrap up parenthesis.
        new_stream.push(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            call_stream.into_iter().collect(),
        )));
        //Semicolon and next line.
        new_stream.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
    }

    new_stream.into_iter().collect()
}

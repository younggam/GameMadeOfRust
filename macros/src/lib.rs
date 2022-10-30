use proc_macro::*;

#[proc_macro]
pub fn impl_with_tuples(input: TokenStream) -> TokenStream {
    let mut token_trees = input.into_iter();

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

    let generic = match token_trees.next() {
        Some(token_tree) => match token_tree {
            TokenTree::Ident(i) => i.to_string(),
            _ => panic!(),
        },
        _ => panic!(),
    };

    let mut new_stream = Vec::<TokenTree>::with_capacity(64);
    for i in start..=end {
        new_stream.push(macro_ident.clone());
        new_stream.push(TokenTree::Punct(Punct::new('!', Spacing::Alone)));
        let mut call_stream = Vec::<TokenTree>::with_capacity(i);
        for j in start..i {
            call_stream.push(TokenTree::Ident(Ident::new(
                &format!("{generic}{j}"),
                Span::call_site(),
            )));
            if j < i - 1 {
                call_stream.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
            }
        }
        new_stream.push(TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            call_stream.into_iter().collect(),
        )));
        new_stream.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
    }

    new_stream.into_iter().collect()
}

#[proc_macro]
pub fn concat_ident(input: TokenStream) -> TokenStream {
    let mut token_trees = input.into_iter();

    fn get_ident(token_tree: Option<TokenTree>) -> String {
        match token_tree {
            Some(token_tree) => match token_tree {
                TokenTree::Ident(i) => i.to_string(),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    let mut ident = get_ident(token_trees.next());
    ident.push_str(&get_ident(token_trees.next()));

    let ret = [TokenTree::Ident(Ident::new(&ident, Span::call_site()))];
    ret.into_iter().collect()
}

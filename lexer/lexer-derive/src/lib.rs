use std::collections::BTreeSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ext::IdentExt, parse_macro_input, Data, DeriveInput, Lit, LitChar, LitStr
};

#[proc_macro_derive(Keyword)]
pub fn keyword(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.ident.span();
    let data = match input.data {
        Data::Enum(ref data) => data,
        _ => unimplemented!(),
    };
    let variants = &data.variants;
    let str_variants = variants.iter().map(|v| {
        syn::PatLit {
            attrs: vec![],
            lit: Lit::Str(LitStr::new(v.ident.unraw().to_string().as_str(), span)),
        }
    });

    let mut set = BTreeSet::new();
    let char_variants = variants.iter().filter_map(|v| {
        let c = v.ident.unraw().to_string().chars().next().unwrap();
        set.insert(c).then(|| syn::PatLit {
            attrs: vec![],
            lit: Lit::Char(LitChar::new(c, span)),
        })
    });

    let expanded = quote! {
        fn is_keyword(s: &str) -> bool {
            match s {
                #(#str_variants => true,)*
                _ => false,
            }
        }
        fn is_keyword_prefix(c: char) -> bool {
            match c {
                #(#char_variants => true,)*
                _ => false,
            }
        }
    };
    expanded.into()
}

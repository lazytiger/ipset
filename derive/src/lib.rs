use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::parse_macro_input;
use syn::ItemStruct;

/// Derive SetType used ipset crate
#[proc_macro_derive(SetType)]
pub fn derive_set_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = input.ident;
    let mut splits: Vec<String> = Vec::new();
    let mut item = Vec::new();
    for c in name.to_string().chars() {
        if c.is_uppercase() && !item.is_empty() {
            splits.push(item.iter().collect());
            item.clear();
        }
        item.push(c);
    }
    if !item.is_empty() {
        splits.push(item.iter().collect());
    }
    let method = format_ident!("{}Method", splits[0]);
    let mut data_types = Vec::new();
    for (i, item) in splits.iter().enumerate() {
        if i > 0 {
            data_types.push(format_ident!("{}DataType", item));
        }
    }

    let ret: TokenStream = quote!(
        impl SetType for #name {
            type Method = #method;
            type DataType = (#(#data_types),*);
        }
    )
    .into();
    //panic!("{}", ret.to_string());
    ret
}

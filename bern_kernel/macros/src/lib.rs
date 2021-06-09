mod enum_map;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro]
pub fn enum_map(input: TokenStream) -> TokenStream {
    let map = syn::parse_macro_input!(input as enum_map::EnumMap);
    let mut output = proc_macro2::TokenStream::new();
    map.to_tokens(&mut output);
    TokenStream::from(output)
}
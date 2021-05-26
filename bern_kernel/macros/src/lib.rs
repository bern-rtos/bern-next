mod conf;

use proc_macro::{TokenStream};
use quote::ToTokens;

#[proc_macro]
pub fn load_conf(input: TokenStream) -> TokenStream {
    let config = syn::parse_macro_input!(input as conf::Config);
    let mut output = proc_macro2::TokenStream::new();
    config.to_tokens(&mut output);
    TokenStream::from(output)
}
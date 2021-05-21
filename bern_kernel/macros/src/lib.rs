mod conf;

use proc_macro::{TokenStream};
use quote::ToTokens;

#[proc_macro]
pub fn load_conf(input: TokenStream) -> TokenStream {
    // there's nothing to parse
    let _ = input;

    let mut output = proc_macro2::TokenStream::new();
    let config = conf::toml::Config::new();
    config.to_tokens(&mut output);

    TokenStream::from(output)
}
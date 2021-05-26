use super::Config;
use quote::ToTokens;
use proc_macro2::TokenStream;

use std::str::FromStr;

impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for entry in self.consts.iter() {
            let formatted = TokenStream::from_str(
                &format!("pub const {}: {} = {};",
                         entry.ident,
                         entry.ty,
                         entry.expr
                ));
            tokens.extend(formatted);
        }
    }
}
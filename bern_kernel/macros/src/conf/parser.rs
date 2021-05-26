use syn::parse::{Parse, ParseStream};
use super::Config;
use syn::{LitStr, Result, parse};
use std::path::PathBuf;
use crate::conf::Error;

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: LitStr = input.parse()?;
        match Self::from(PathBuf::from(name.value())) {
            Ok(config) => Ok(config),
            Err(e) => Err(match e {
                Error::InvalidToml => parse::Error::new(
                    name.span(),
                    "TOML invalid",
                ),
                Error::FileNotFound => parse::Error::new(
                    name.span(),
                    "File not found in `CARGO_MANIFEST_DIR`",
                ),
            }),
        }
    }
}
use serde_derive::Deserialize;
use syn::{Expr, Ident, Type};
use proc_macro2::Span;
use toml;
use toml::{Value as Toml, Value};

use std::string::{String, ToString};
use std::env;
use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;
use std::vec::Vec;
use std::fmt::format;

pub struct Entry {
    pub ident: String,
    pub ty: String,
    pub expr: String,
}

pub struct Config {
    pub consts: Vec<Entry>,
}

const CONF_NAME: &str = "bern.toml";

impl Config {
    pub fn new() -> Self {
        let input = Self::load();
        match input.parse() {
            Ok(toml) => {
                Config {
                    consts: Self::toml_to_consts(toml),
                }
            },
            Err(e) => panic!("failed to pars TOML: {}", e),
        }
    }

    fn load() -> String {
        let root = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let mut input = String::new();
        File::open(root.join(CONF_NAME))
            .and_then(|mut f| f.read_to_string(&mut input))
            .unwrap();
        input
    }

    fn toml_to_consts(toml: Toml) -> Vec<Entry> {
        let mut consts: Vec<Entry> = Vec::new();

        let mut stack = vec![(String::new(), toml)];
        while stack.len() > 0 {
            match stack.pop().unwrap() {
                (prefix, Toml::Table(table)) => {
                    for entry in table.iter() {
                        let prefix = if prefix.is_empty() {
                            entry.0.clone()
                        } else {
                            format!("{}_{}", prefix, entry.0)
                        };
                        stack.push((prefix, entry.1.clone()))
                    }
                },
                (prefix,Toml::String(string)) => {
                    consts.push(Entry {
                        ident: prefix.to_uppercase(),
                        ty: "&str".to_string(),
                        expr: format!("\"{}\"", string)
                    });
                },
                (prefix,Toml::Integer(integer)) => {
                    consts.push(Entry {
                        ident: prefix.to_uppercase(),
                        ty: "usize".to_string(),
                        expr: integer.to_string()
                    });
                },
                (prefix,Toml::Boolean(boolean)) => {
                    consts.push(Entry {
                        ident: prefix.to_uppercase(),
                        ty: "bool".to_string(),
                        expr: boolean.to_string()
                    });
                },
                (prefix,Toml::Float(float)) => {
                    consts.push(Entry {
                        ident: prefix.to_uppercase(),
                        ty: "f32".to_string(),
                        expr: float.to_string()
                    });
                },
                (prefix,Toml::Datetime(_)) => {}
                (prefix,Toml::Array(_)) => {}
            }
        }

        consts
    }
}
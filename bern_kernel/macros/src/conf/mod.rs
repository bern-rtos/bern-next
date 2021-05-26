use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use toml;
use toml::{Value as Toml};
use std::env;

pub mod generator;
pub mod parser;

pub enum Error {
    InvalidToml,
    FileNotFound,
}

pub struct Config {
    pub consts: Vec<Entry>,
}

pub struct Entry {
    pub ident: String,
    pub ty: String,
    pub expr: String,
}

impl Config {
    pub fn new(raw: String) -> Result<Self, Error> {
        match raw.parse() {
            Ok(toml) => {
                Ok(Config {
                    consts: Self::toml_to_consts(toml),
                })
            },
            Err(_) => Err(Error::InvalidToml),
        }
    }

    pub fn from(file: PathBuf) -> Result<Self, Error> {
        let root = &PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        let mut input = String::new();
        match File::open(root.join(file))
            .and_then(|mut f| f.read_to_string(&mut input)) {
            Ok(_) => Self::new(input),
            Err(_) => Err(Error::FileNotFound),
        }
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
                (_, Toml::Datetime(_)) => {}
                (_, Toml::Array(_)) => {}
            }
        }

        consts
    }
}
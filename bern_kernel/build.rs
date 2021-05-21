use std::env;
use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    kernel: Option<KernelConfig>,
}

#[derive(Debug, Deserialize)]
struct KernelConfig {
    mutex: Option<MutexConfig>,
}

#[derive(Debug, Deserialize)]
struct MutexConfig {
    pool_size: Option<u32>,
}

fn main() {
    //let root = &PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    //let mut input = String::new();
    //File::open(root.join("bern.toml"))
    //    .and_then(|mut f| f.read_to_string(&mut input))
    //    .unwrap();

    //let config: Config = toml::from_str(&input).unwrap();


    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("bern.toml"))
        .unwrap()
        .write_all(include_bytes!("bern.toml"))
        .unwrap();
    println!("cargo:rerun-if-changed=bern.toml");
}
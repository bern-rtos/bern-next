// The kernel can be configured in `bern.toml`. The configuration is loaded at
// during the build.
use bern_kernel_macros::load_conf;

//const CONFIG_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/bern.toml");
load_conf!("bern.toml");
//load_conf!(concat!(env!("CARGO_MANIFEST_DIR"), "/bern.toml"));
//load_conf!("/home/stefan/embedded/bern/demo/kernel_next/bern_kernel/bern.toml");
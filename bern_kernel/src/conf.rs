// The kernel can be configured in `bern.toml`. The configuration is loaded at
// during the build.
//include!(concat!(env!("OUT_DIR"), "/conf.rs"));
use bern_kernel_macros::load_conf;
//load_conf!(concat!(env!("OUT_DIR"), "/bern.toml"));
load_conf!();
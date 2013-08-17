#[link(name = "rust_fuse",
uuid = "d37c5c30-fdcd-459d-bfca-ebb8da04b2a0",
url = "https://github.com/MicahChalmer/rust-fuse",
vers = "dev")];

#[comment = "FUSE bindings"];
#[license = "MIT"];
#[crate_type = "lib"];

pub mod thin;

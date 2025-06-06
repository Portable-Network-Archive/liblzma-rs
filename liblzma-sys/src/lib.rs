#![doc(html_root_url = "https://docs.rs/liblzma-sys/0.4.4")]
#![allow(bad_style)]

#[cfg(feature = "bindgen")]
mod bindgen;
#[cfg(feature = "bindgen")]
mod bindgen_wrap;
#[cfg(not(feature = "bindgen"))]
mod manual;

#[cfg(target_arch = "wasm32")]
mod wasm_shim;

#[cfg(feature = "bindgen")]
pub use bindgen_wrap::*;
#[cfg(not(feature = "bindgen"))]
pub use manual::*;

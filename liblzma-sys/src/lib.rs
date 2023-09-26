#![allow(bad_style)]

#[cfg(feature = "bindgen")]
mod bindgen;
#[cfg(feature = "bindgen")]
mod bindgen_wrap;
#[cfg(not(feature = "bindgen"))]
mod manual;

#[cfg(feature = "bindgen")]
pub use bindgen_wrap::*;
#[cfg(not(feature = "bindgen"))]
pub use manual::*;

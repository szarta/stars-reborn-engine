// engine/src/lib.rs
//
// Library root for the stars-engine crate.
// The standalone server binary (src/main.rs) links against this rlib.

pub mod game;
pub mod http;
pub mod store;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

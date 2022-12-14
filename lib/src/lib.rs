pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

mod controller;
mod loupedeck;

pub use controller::*;
pub use loupedeck::*;

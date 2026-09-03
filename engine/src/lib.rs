pub mod acquire;
pub mod cli;
pub mod diagnostics;
pub mod digest;
pub mod error;
pub mod events;
pub mod games;
pub mod loader;
pub mod lock;
pub mod manifest;
pub mod orchestrator;
pub mod postcondition;
pub mod preflight;
pub mod receipt;
pub mod recipe_view;
pub mod registry;
pub mod resolve;
pub mod session;
pub mod stage;
pub mod validate;
pub mod weidu;

pub use loader::Manifest;

/// Compiled engine package version exposed to native presentation adapters.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

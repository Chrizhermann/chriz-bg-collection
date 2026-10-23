pub mod acquire;
pub mod cli;
pub mod diagnostics;
pub mod digest;
pub mod error;
pub mod events;
pub mod game_log;
pub mod games;
mod held_folder;
pub mod installed_updates;
pub mod loader;
pub mod lock;
pub mod manifest;
pub mod manual_checkpoint;
pub mod mod_upgrades;
pub mod operation_progress;
#[cfg(test)]
mod operation_progress_tests;
pub mod orchestrator;
pub mod patches;
pub mod postcondition;
pub mod preflight;
pub mod radar;
pub mod receipt;
pub mod recipe_envelope;
pub mod recipe_view;
pub mod recovery_receipt;
pub mod registry;
pub mod release_validate;
pub mod removal;
pub mod resolve;
pub mod session;
pub mod stage;
pub mod updates;
pub mod validate;
pub mod weidu;

pub use loader::Manifest;

/// Compiled engine package version exposed to native presentation adapters.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

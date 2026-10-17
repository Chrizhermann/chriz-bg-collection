//! WeiDU invocation, supervision, and verification.

pub(crate) mod config_files;
pub(crate) mod eet_compat;
pub mod invocation;
pub mod log;
mod process_group;
pub mod recovery;
pub mod runner;
pub(crate) mod ui_compat;
pub mod verify;

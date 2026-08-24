//! Shared test-support code for the engine's integration tests.
//!
//! Each integration-test crate that needs it declares `mod support;`, so
//! items unused by a particular crate must not trip dead-code lints.
#![allow(dead_code)]

pub mod fakegame;

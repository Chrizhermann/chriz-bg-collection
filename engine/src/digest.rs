//! Canonical semantic digests used to freeze resumable campaign identity.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{EngineError, Result};
use crate::recipe_view::NormalizedSelection;
use crate::resolve::InstallPlan;

/// Returns the lowercase SHA-256 of exact bytes.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Returns a deterministic digest of a normalized semantic selection.
pub fn selection_digest(selection: &NormalizedSelection) -> Result<String> {
    canonical_json_digest("normalized selection", selection)
}

/// Returns a deterministic digest of the exact resolved installation plan.
pub fn plan_digest(plan: &InstallPlan) -> Result<String> {
    canonical_json_digest("resolved plan", plan)
}

fn canonical_json_digest<T: Serialize>(context: &'static str, value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|source| EngineError::CanonicalDigest { context, source })?;
    Ok(sha256_bytes(&bytes))
}

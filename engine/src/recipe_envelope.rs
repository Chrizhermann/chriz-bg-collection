//! Exact-byte recipe envelope verification and explicit Minisign trust rotation.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use minisign_verify::{PublicKey, Signature};
use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::ZipArchive;

use crate::digest::sha256_bytes;
use crate::updates::{validate_release, RecipeRelease};

/// Signed metadata for one exact immutable recipe payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeEnvelope {
    pub recipe_id: String,
    pub version: String,
    pub payload_sha256: String,
    pub key_id: String,
    pub minimum_app_version: String,
    pub published_at: String,
}

/// Public verification material. Private key types never enter this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedPublicKey {
    pub key_id: String,
    pub minisign_public_key: String,
}

struct TrustEntry {
    key: PublicKey,
    activation_version: Option<Version>,
}

/// Fail-closed recipe trust roots plus rotations authorized by an existing root.
pub struct RecipeTrustStore {
    keys: BTreeMap<String, TrustEntry>,
}

/// Exact signed statement that authorizes a future recipe verification key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRotationStatement {
    pub recipe_id: String,
    pub from_key_id: String,
    pub to_key_id: String,
    pub to_minisign_public_key: String,
    pub activation_version: String,
}

/// A verified immutable snapshot. Callers may copy it into a new build without replacing an
/// already-running campaign's frozen payload/envelope bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRecipe {
    pub envelope: RecipeEnvelope,
    pub payload_bytes: Vec<u8>,
    pub envelope_bytes: Vec<u8>,
}

/// Exact-byte trust failure.
#[derive(Debug, Error)]
pub enum RecipeTrustError {
    #[error("invalid recipe envelope: {0}")]
    Envelope(String),
    #[error("unknown key {0:?}; a rotation signed by a trusted key is required")]
    UnknownKey(String),
    #[error("invalid recipe signature: {0}")]
    Signature(String),
    #[error("recipe payload digest mismatch: expected {expected}, found {found}")]
    PayloadDigest { expected: String, found: String },
    #[error("recipe version must be newer than highest trusted version {0}")]
    Replay(String),
    #[error("minimum app version {required} is newer than running app {running}")]
    MinimumApp { required: String, running: String },
    #[error("invalid recipe ledger: {0}")]
    Ledger(String),
    #[error("invalid key rotation: {0}")]
    Rotation(String),
}

impl RecipeTrustStore {
    /// Build a trust store from embedded or authoring-injected public keys.
    pub fn new(keys: impl IntoIterator<Item = TrustedPublicKey>) -> Result<Self, RecipeTrustError> {
        let mut trusted = BTreeMap::new();
        for entry in keys {
            if entry.key_id.trim().is_empty() || trusted.contains_key(&entry.key_id) {
                return Err(RecipeTrustError::Rotation(format!(
                    "empty or duplicate key id {:?}",
                    entry.key_id
                )));
            }
            let key = PublicKey::decode(&entry.minisign_public_key)
                .map_err(|error| RecipeTrustError::Rotation(error.to_string()))?;
            trusted.insert(
                entry.key_id,
                TrustEntry {
                    key,
                    activation_version: None,
                },
            );
        }
        Ok(Self { keys: trusted })
    }

    /// Add a new public key only after verifying the exact statement bytes with the named
    /// currently trusted key.
    pub fn authorize_rotation(
        &mut self,
        statement_bytes: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), RecipeTrustError> {
        let statement: KeyRotationStatement = serde_json::from_slice(statement_bytes)
            .map_err(|error| RecipeTrustError::Rotation(error.to_string()))?;
        if statement.from_key_id == statement.to_key_id
            || statement.recipe_id.trim().is_empty()
            || self.keys.contains_key(&statement.to_key_id)
        {
            return Err(RecipeTrustError::Rotation(
                "rotation ids or recipe id are invalid".to_owned(),
            ));
        }
        let current = self
            .keys
            .get(&statement.from_key_id)
            .ok_or_else(|| RecipeTrustError::UnknownKey(statement.from_key_id.clone()))?;
        verify_signature(&current.key, statement_bytes, signature_bytes)
            .map_err(|error| RecipeTrustError::Rotation(error.to_string()))?;
        let activation_version = Version::parse(&statement.activation_version)
            .map_err(|error| RecipeTrustError::Rotation(error.to_string()))?;
        let key = PublicKey::decode(&statement.to_minisign_public_key)
            .map_err(|error| RecipeTrustError::Rotation(error.to_string()))?;
        self.keys.insert(
            statement.to_key_id,
            TrustEntry {
                key,
                activation_version: Some(activation_version),
            },
        );
        Ok(())
    }
}

/// Verify signature, monotonic version, app prerequisite, exact payload digest, and the ledger
/// bound inside that payload. The running application version is supplied by the application,
/// never inferred from the engine crate version.
pub fn verify_recipe_package(
    payload_bytes: &[u8],
    envelope_bytes: &[u8],
    signature_bytes: &[u8],
    trust: &RecipeTrustStore,
    running_app_version: &str,
    highest_trusted_version: Option<&str>,
) -> Result<VerifiedRecipe, RecipeTrustError> {
    // Parsing is used only to route to a candidate public key. No envelope claim is accepted
    // until the detached signature over these exact bytes succeeds.
    let envelope: RecipeEnvelope = serde_json::from_slice(envelope_bytes)
        .map_err(|error| RecipeTrustError::Envelope(error.to_string()))?;
    let trusted = trust
        .keys
        .get(&envelope.key_id)
        .ok_or_else(|| RecipeTrustError::UnknownKey(envelope.key_id.clone()))?;
    verify_signature(&trusted.key, envelope_bytes, signature_bytes)?;

    let version = Version::parse(&envelope.version)
        .map_err(|error| RecipeTrustError::Envelope(error.to_string()))?;
    if let Some(activation) = &trusted.activation_version {
        if &version < activation {
            return Err(RecipeTrustError::Envelope(format!(
                "key {:?} is not active before {activation}",
                envelope.key_id
            )));
        }
    }
    if let Some(highest) = highest_trusted_version {
        let highest = Version::parse(highest)
            .map_err(|error| RecipeTrustError::Envelope(error.to_string()))?;
        if version <= highest {
            return Err(RecipeTrustError::Replay(highest.to_string()));
        }
    }
    let running = Version::parse(running_app_version)
        .map_err(|error| RecipeTrustError::Envelope(error.to_string()))?;
    let required = Version::parse(&envelope.minimum_app_version)
        .map_err(|error| RecipeTrustError::Envelope(error.to_string()))?;
    if running < required {
        return Err(RecipeTrustError::MinimumApp {
            required: required.to_string(),
            running: running.to_string(),
        });
    }
    if envelope.recipe_id.trim().is_empty()
        || envelope.published_at.trim().is_empty()
        || !valid_digest(&envelope.payload_sha256)
    {
        return Err(RecipeTrustError::Envelope(
            "recipe id, publication date, or payload digest is invalid".to_owned(),
        ));
    }
    let actual = sha256_bytes(payload_bytes);
    if actual != envelope.payload_sha256 {
        return Err(RecipeTrustError::PayloadDigest {
            expected: envelope.payload_sha256,
            found: actual,
        });
    }

    let ledger = read_bound_ledger(payload_bytes, &envelope.version)?;
    if ledger.recipe_id != envelope.recipe_id
        || ledger.version != envelope.version
        || ledger.minimum_app_version != envelope.minimum_app_version
        || ledger.published_at != envelope.published_at
    {
        return Err(RecipeTrustError::Ledger(
            "envelope and release ledger identities differ".to_owned(),
        ));
    }
    Ok(VerifiedRecipe {
        envelope,
        payload_bytes: payload_bytes.to_vec(),
        envelope_bytes: envelope_bytes.to_vec(),
    })
}

fn verify_signature(
    key: &PublicKey,
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<(), RecipeTrustError> {
    let text = std::str::from_utf8(signature_bytes)
        .map_err(|error| RecipeTrustError::Signature(error.to_string()))?;
    let signature =
        Signature::decode(text).map_err(|error| RecipeTrustError::Signature(error.to_string()))?;
    key.verify(message, &signature, false)
        .map_err(|error| RecipeTrustError::Signature(error.to_string()))
}

fn read_bound_ledger(
    payload_bytes: &[u8],
    version: &str,
) -> Result<RecipeRelease, RecipeTrustError> {
    let mut archive = ZipArchive::new(Cursor::new(payload_bytes))
        .map_err(|error| RecipeTrustError::Ledger(error.to_string()))?;
    let path = format!("releases/v{version}/ledger.toml");
    let mut entry = archive
        .by_name(&path)
        .map_err(|error| RecipeTrustError::Ledger(format!("{path}: {error}")))?;
    if entry.size() > 1024 * 1024 {
        return Err(RecipeTrustError::Ledger("ledger exceeds 1 MiB".to_owned()));
    }
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|error| RecipeTrustError::Ledger(error.to_string()))?;
    let release: RecipeRelease =
        toml::from_str(&text).map_err(|error| RecipeTrustError::Ledger(error.to_string()))?;
    validate_release(&release).map_err(RecipeTrustError::Ledger)?;
    Ok(release)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

//! Separate, exact-byte authenticated patch metadata. Never extends a recipe ledger.
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};

pub const PATCH_ID: &str = "artisan-campaign-description-links-1.0";
pub const ADAPTER: &str = "AKCB_KIT_DESCRIPTIONS";
pub const TOOL_HASH: &str = "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a";
pub const OWNER_COMMIT: &str = "d16d35ba29c68aa18025a1c5323a8973986a97d9";
pub const FILES: [(&str, u64, &str); 2] = [
    (
        "setup-AKCB_KIT_DESCRIPTIONS.tp2",
        3040,
        "980130c5c7a363a3d4a52d741666725a7ec9ede72eea3ad0e0af48afb90cc479",
    ),
    (
        "lib/kit_strref.tpa",
        2297,
        "f83006ac9bc995b059f6221de7af4f06fd7ea0b6adbbf65a8bc3d92f2746d6f7",
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: u32,
    pub domain: String,
    pub version: String,
    pub minimum_app_version: String,
    pub patch_id: String,
    pub title: String,
    pub change_ids: Vec<String>,
    pub supported_versions: Vec<String>,
    pub summary: String,
}

pub fn verify(
    bytes: &[u8],
    signature: &str,
    public_key: &str,
    app_version: &str,
) -> Result<Catalog, String> {
    if bytes.len() > 64 * 1024 || signature.len() > 4096 {
        return Err("Patch catalog exceeds its size limit.".into());
    }
    let key = PublicKey::decode(public_key).map_err(|e| e.to_string())?;
    let signature = Signature::decode(signature).map_err(|e| e.to_string())?;
    key.verify(bytes, &signature, false)
        .map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if catalog.schema != 1
        || catalog.domain != "chriz-easy-bg-existing-install-patches"
        || catalog.patch_id != PATCH_ID
        || catalog.version != "1.0.0"
        || catalog.supported_versions != ["chriz-v1.3.0"]
        || catalog.title.is_empty()
        || catalog.summary.is_empty()
        || catalog.change_ids.is_empty()
    {
        return Err("This patch catalog is not supported by this application.".into());
    }
    let running = semver::Version::parse(app_version).map_err(|e| e.to_string())?;
    let minimum =
        semver::Version::parse(&catalog.minimum_app_version).map_err(|e| e.to_string())?;
    if running < minimum {
        return Err("Update CEBG before applying this fix.".into());
    }
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_catalog_rejects_tampering_wrong_key_and_wrong_domain() {
        let pair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let key = pair.pk.to_box().unwrap().to_string();
        let bytes = include_bytes!("../../../patches/catalog.json");
        let sign = |b: &[u8]| {
            minisign::sign(None, &pair.sk, std::io::Cursor::new(b), None, None)
                .unwrap()
                .into_string()
        };
        assert!(verify(bytes, &sign(bytes), &key, "0.1.0-alpha.18").is_ok());
        let mut tampered = bytes.to_vec();
        tampered.push(b' ');
        assert!(verify(&tampered, &sign(bytes), &key, "0.1.0-alpha.18").is_err());
        let other = minisign::KeyPair::generate_unencrypted_keypair()
            .unwrap()
            .pk
            .to_box()
            .unwrap()
            .to_string();
        assert!(verify(bytes, &sign(bytes), &other, "0.1.0-alpha.18").is_err());
        let changed = String::from_utf8(bytes.to_vec())
            .unwrap()
            .replace("chriz-easy-bg-existing-install-patches", "recipe");
        assert!(verify(
            changed.as_bytes(),
            &sign(changed.as_bytes()),
            &key,
            "0.1.0-alpha.18"
        )
        .is_err());
        assert!(verify(bytes, &sign(bytes), &key, "0.1.0-alpha.17").is_err());
    }
}

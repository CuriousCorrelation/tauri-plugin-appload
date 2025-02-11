use chrono::Utc;
use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey};
use std::io::Cursor;
use std::sync::Arc;
use tauri::Config;
use zip::ZipArchive;

use crate::{
    bundle::VerifiedBundle, cache::CacheManager, storage::StorageManager, vendor::VendorError,
    verification::FileVerifier, BundleMetadata, FileEntry, Manifest,
};

use super::Result;

const VENDOR_SOURCE: &str = "vendor";

#[derive(Debug, Clone)]
pub struct VendorConfig {
    pub(super) content: Option<Vec<u8>>,
}

impl VendorConfig {
    pub(crate) async fn initialize(
        self,
        config: Config,
        cache: Arc<CacheManager>,
        storage: Arc<StorageManager>,
    ) -> Result<()> {
        let Some(content) = self.content else {
            tracing::info!("No vendored bundle provided, skipping initialization");
            return Ok(());
        };

        let max_bundle_size = 100 * 1024 * 1024;
        if content.len() > max_bundle_size {
            return Err(VendorError::InvalidData(format!(
                "Bundle too large: {} bytes (max: {} bytes)",
                content.len(),
                max_bundle_size
            )));
        }

        let name = config.product_name.unwrap_or("unknown".to_string());
        let version = config.version.as_deref().unwrap_or("0.0.0");

        tracing::info!(
            name = %name,
            version = %version,
            size = content.len(),
            "Initializing vendored bundle"
        );

        let mut archive = ZipArchive::new(Cursor::new(&content))?;
        let mut files = Vec::with_capacity(archive.len());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if !file.is_file() {
                continue;
            }

            if file.size() > max_bundle_size.try_into().unwrap() {
                return Err(VendorError::InvalidData(format!(
                    "File too large: {} ({} bytes)",
                    file.name(),
                    file.size()
                )));
            }

            let path = file.name().to_string();
            let mut content = Vec::with_capacity(file.size() as usize);
            std::io::Read::read_to_end(&mut file, &mut content)?;

            let size = content.len() as u64;
            let hash = FileVerifier::hash(&content);
            let mime_type = mime_guess::from_path(&path).first().map(|m| m.to_string());

            files.push(FileEntry {
                path,
                size,
                hash,
                mime_type,
            });
        }

        let mut signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let signature = signing_key.sign(&content);

        let metadata = BundleMetadata {
            version: version.to_string(),
            created_at: Utc::now(),
            signature,
            manifest: Manifest { files },
            properties: Default::default(),
        };

        let verified = VerifiedBundle::new(content, metadata.clone())?;

        storage
            .store_bundle(&name, VENDOR_SOURCE, &metadata.version, &verified)
            .await?;

        cache.cache_bundle(&name, &verified).await?;

        tracing::info!(
            name = %name,
            "Vendored bundle initialized successfully"
        );
        Ok(())
    }
}

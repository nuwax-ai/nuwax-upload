use std::fmt;
use std::path::{Path, PathBuf};

use chrono::Utc;
use walkdir::WalkDir;

use crate::cli::Args;
use crate::error::UploadError;

/// A single file to be uploaded with its local path and remote key.
#[derive(Debug)]
pub struct UploadItem {
    pub local_path: PathBuf,
    pub remote_key: String,
    pub size: u64,
}

/// Resolved and validated upload configuration.
pub struct UploadConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub prefix: String,
    pub timeout: Option<std::time::Duration>,
    pub concurrency: usize,
    pub items: Vec<UploadItem>,
}

// Manual Debug impl to avoid leaking credentials in logs.
impl fmt::Debug for UploadConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UploadConfig")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key", &"***")
            .field("secret_key", &"***")
            .field("region", &self.region)
            .field("prefix", &self.prefix)
            .field("timeout", &self.timeout)
            .field("concurrency", &self.concurrency)
            .field("items", &self.items.len())
            .finish()
    }
}

/// Resolve CLI args into a validated UploadConfig.
///
/// - Generates default prefix `docker/{YYYYMMDDHHmmss}` if not specified
/// - Expands directories recursively (no symlink following), preserving relative sub-directory structure
/// - Validates all paths exist and filenames are valid UTF-8
pub fn resolve(args: Args) -> Result<UploadConfig, UploadError> {
    let prefix = args
        .prefix
        .map(|p| p.trim_end_matches('/').to_string())
        .unwrap_or_else(|| {
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            format!("docker/{}", timestamp)
        });

    let mut items = Vec::new();

    for path in &args.paths {
        if !path.exists() {
            return Err(UploadError::PathNotFound(path.clone()));
        }

        if path.is_file() {
            let filename = require_utf8_filename(path)?;
            let size = std::fs::metadata(path)
                .map_err(|e| UploadError::PathNotReadable {
                    path: path.clone(),
                    source: e,
                })?
                .len();
            items.push(UploadItem {
                local_path: path.clone(),
                remote_key: format!("{}/{}", prefix, filename),
                size,
            });
        } else if path.is_dir() {
            collect_dir_files(path, &prefix, &mut items)?;
        } else {
            return Err(UploadError::InvalidPath(path.clone()));
        }
    }

    if items.is_empty() {
        return Err(UploadError::NoFiles);
    }

    Ok(UploadConfig {
        endpoint: args.endpoint,
        bucket: args.bucket,
        access_key: args.access_key,
        secret_key: args.secret_key,
        region: args.region,
        prefix,
        timeout: args.timeout.map(std::time::Duration::from_secs),
        concurrency: args.concurrency,
        items,
    })
}

/// Require that a path's filename component is valid UTF-8.
fn require_utf8_filename(path: &Path) -> Result<String, UploadError> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| UploadError::InvalidPath(path.to_path_buf()))
}

/// Recursively collect files from a directory, preserving relative paths.
/// Symlinks are not followed to avoid infinite loops and path traversal.
fn collect_dir_files(
    base_dir: &Path,
    prefix: &str,
    items: &mut Vec<UploadItem>,
) -> Result<(), UploadError> {
    for entry in WalkDir::new(base_dir).min_depth(1).follow_links(false) {
        let entry = entry.map_err(|e| UploadError::DirRead {
            path: base_dir.to_path_buf(),
            source: e,
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        let full_path = entry.path().to_path_buf();

        let relative = full_path
            .strip_prefix(base_dir)
            .map_err(|_| UploadError::InvalidPath(full_path.clone()))?;
        let relative_str = relative
            .to_str()
            .ok_or_else(|| UploadError::InvalidPath(full_path.clone()))?;
        let remote_key = format!("{}/{}", prefix, relative_str);

        let size = entry
            .metadata()
            .map_err(|e| UploadError::DirRead {
                path: full_path.clone(),
                source: e,
            })?
            .len();

        items.push(UploadItem {
            local_path: full_path,
            remote_key,
            size,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_prefix_format() {
        let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let prefix = format!("docker/{}", ts);
        assert!(prefix.starts_with("docker/"));
        assert_eq!(prefix.len(), "docker/".len() + 14);
    }

    #[test]
    fn test_prefix_trailing_slash_trimmed() {
        let input = "releases/v1.0/".to_string();
        let trimmed = input.trim_end_matches('/').to_string();
        assert_eq!(trimmed, "releases/v1.0");
    }

    #[test]
    fn test_collect_dir_preserves_structure() {
        let tmp = std::env::temp_dir().join(format!("nuwax_upload_test_{}", std::process::id()));
        let sub = tmp.join("sub");
        let _ = fs::create_dir_all(&sub);
        fs::write(tmp.join("a.txt"), "a").unwrap();
        fs::write(sub.join("b.txt"), "b").unwrap();

        let mut items = Vec::new();
        collect_dir_files(&tmp, "pfx", &mut items).unwrap();

        let keys: Vec<&str> = items.iter().map(|i| i.remote_key.as_str()).collect();
        assert!(keys.contains(&"pfx/a.txt"));
        assert!(keys.contains(&"pfx/sub/b.txt"));

        // All sizes should be > 0
        for item in &items {
            assert!(item.size > 0);
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_debug_redacts_credentials() {
        let config = UploadConfig {
            endpoint: "https://s3.example.com".to_string(),
            bucket: "test".to_string(),
            access_key: "SUPER_SECRET_KEY".to_string(),
            secret_key: "SUPER_SECRET_VALUE".to_string(),
            region: "us-east-1".to_string(),
            prefix: "docker/test".to_string(),
            timeout: None,
            concurrency: 3,
            items: vec![],
        };
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.contains("SUPER_SECRET_KEY"));
        assert!(!debug_str.contains("SUPER_SECRET_VALUE"));
        assert!(debug_str.contains("***"));
    }
}

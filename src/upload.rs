use indicatif::ProgressBar;
use s3::creds::Credentials;
use s3::{Bucket, Region};

use crate::config::{UploadConfig, UploadItem};
use crate::content_type;
use crate::error::UploadError;
use crate::progress::ProgressReader;

const MULTIPART_THRESHOLD: u64 = 5 * 1024 * 1024; // 5MB

/// Create an S3 Bucket handle configured for MinIO (path-style).
pub fn create_bucket(config: &UploadConfig) -> Result<Box<Bucket>, UploadError> {
    let region = Region::Custom {
        region: config.region.clone(),
        endpoint: config.endpoint.clone(),
    };

    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )
    .map_err(|e| UploadError::ClientInit(e.to_string()))?;

    let mut bucket = Bucket::new(&config.bucket, region, credentials)
        .map_err(|e| UploadError::ClientInit(e.to_string()))?
        .with_path_style();

    if let Some(timeout) = config.timeout {
        bucket = bucket
            .with_request_timeout(timeout)
            .map_err(|e| UploadError::ClientInit(e.to_string()))?;
    }

    Ok(bucket)
}

/// Upload a single file to S3/MinIO with progress tracking.
///
/// Files < 5MB use simple PUT; >= 5MB use streaming multipart with content-type.
pub async fn upload_file(
    bucket: &Bucket,
    item: &UploadItem,
    pb: &ProgressBar,
) -> Result<(), UploadError> {
    let ct = content_type::detect(&item.remote_key);

    if item.size < MULTIPART_THRESHOLD {
        let data = tokio::fs::read(&item.local_path)
            .await
            .map_err(|e| UploadError::UploadFailed {
                filename: item.remote_key.clone(),
                source: e.into(),
            })?;

        bucket
            .put_object_with_content_type(&item.remote_key, &data, &ct)
            .await
            .map_err(|e| UploadError::UploadFailed {
                filename: item.remote_key.clone(),
                source: e.into(),
            })?;

        pb.set_position(item.size);
    } else {
        let file = tokio::fs::File::open(&item.local_path)
            .await
            .map_err(|e| UploadError::UploadFailed {
                filename: item.remote_key.clone(),
                source: e.into(),
            })?;

        let mut reader = ProgressReader::new(file, pb.clone());

        bucket
            .put_object_stream_with_content_type(&mut reader, &item.remote_key, &ct)
            .await
            .map_err(|e| UploadError::UploadFailed {
                filename: item.remote_key.clone(),
                source: e.into(),
            })?;
    }

    Ok(())
}

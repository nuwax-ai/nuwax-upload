use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nuwax-upload")]
#[command(about = "Upload files and directories to MinIO (S3-compatible) storage")]
#[command(version)]
pub struct Args {
    /// Files or directories to upload (supports multiple, mixed)
    #[arg(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,

    /// Custom path prefix in the bucket (default: docker/{YYYYMMDDHHmmss})
    #[arg(short, long, value_name = "PREFIX")]
    pub prefix: Option<String>,

    /// MinIO/S3 API endpoint
    #[arg(long, env = "MINIO_ENDPOINT", default_value = "https://s3.nuwax.com:9443")]
    pub endpoint: String,

    /// Target bucket name
    #[arg(long, env = "MINIO_BUCKET", default_value = "nuwax-packages")]
    pub bucket: String,

    /// Access key ID
    #[arg(long, env = "MINIO_ACCESS_KEY")]
    pub access_key: String,

    /// Secret access key
    #[arg(long, env = "MINIO_SECRET_KEY")]
    pub secret_key: String,

    /// S3 region (arbitrary for MinIO)
    #[arg(long, env = "MINIO_REGION", default_value = "us-east-1")]
    pub region: String,

    /// Request timeout in seconds (per HTTP request, not total).
    /// If not specified, no timeout is applied.
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Number of files to upload concurrently.
    #[arg(short, long, default_value = "3")]
    pub concurrency: usize,
}

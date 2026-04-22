use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("cannot read {path}: {source}")]
    PathNotReadable {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("path is not a file or directory: {0}")]
    InvalidPath(PathBuf),

    #[error("no files to upload")]
    NoFiles,

    #[error("failed to read directory {path}: {source}")]
    DirRead {
        path: PathBuf,
        source: walkdir::Error,
    },

    #[error("progress bar template error: {0}")]
    ProgressStyle(#[from] indicatif::style::TemplateError),

    #[error("S3 client initialization failed: {0}")]
    ClientInit(String),

    #[error("upload failed for {filename}: {source}")]
    UploadFailed {
        filename: String,
        source: anyhow::Error,
    },
}

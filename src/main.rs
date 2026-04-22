mod cli;
mod config;
mod content_type;
mod error;
mod progress;
mod upload;

use clap::Parser;
use futures::stream::{self, StreamExt, TryStreamExt};
use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};
use tracing::{error, info};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), error::UploadError> {
    let args = cli::Args::parse();
    let config = config::resolve(args)?;

    info!(
        "uploading {} file(s) to {}/{} (prefix: {})",
        config.items.len(),
        config.endpoint,
        config.bucket,
        config.prefix,
    );

    let bucket = upload::create_bucket(&config)?;
    let bucket_ref = &*bucket;

    let mp = MultiProgress::new();
    let progress_style = ProgressStyle::with_template(
        "{prefix} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA {eta})",
    )?
    .progress_chars("=> ");

    let finish_style = ProgressStyle::with_template(
        "{prefix} [{bar:40.green}] {total_bytes} in {elapsed}",
    )?
    .progress_chars("=> ");

    let total = config.items.len();

    stream::iter(config.items.iter().enumerate().map(|(i, item)| {
        let filename = item.remote_key.rsplit('/').next().unwrap_or(&item.remote_key);
        let label = format!("[{}/{}] {}", i + 1, total, filename);

        let pb = mp.add(ProgressBar::new(item.size));
        pb.set_style(progress_style.clone());
        pb.set_prefix(label.clone());

        let finish_style = finish_style.clone();
        let bucket = bucket_ref;

        async move {
            upload::upload_file(bucket, item, &pb).await?;
            pb.set_style(finish_style);
            pb.finish();
            Ok::<(), error::UploadError>(())
        }
    }))
    .buffer_unordered(config.concurrency)
    .try_for_each(|()| async { Ok(()) })
    .await?;

    let total_bytes: u64 = config.items.iter().map(|i| i.size).sum();
    info!(
        "all {} file(s) uploaded successfully ({})",
        total,
        HumanBytes(total_bytes),
    );

    let base_url = format!(
        "{}/{}/{}",
        config.endpoint.trim_end_matches('/'),
        config.bucket,
        config.prefix,
    );
    info!("download URLs:");
    for item in &config.items {
        let url = format!(
            "{}/{}/{}",
            config.endpoint.trim_end_matches('/'),
            config.bucket,
            item.remote_key,
        );
        info!("  {}", url);
    }
    info!("prefix: {}", base_url);

    Ok(())
}

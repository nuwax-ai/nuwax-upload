# nuwax-upload

Upload files and directories to MinIO (S3-compatible) storage.

## Install

```bash
make install    # cargo install --path . (recommended)
# or
make build      # cargo build --release
# then move target/release/nuwax-upload to your PATH
```

## Usage

```bash
nuwax-upload [OPTIONS] <PATHS>...
```

## Parameters

| Parameter | Type | Required | Env Variable | Default | Description |
|-----------|------|----------|-------------|---------|-------------|
| `<PATHS>` | positional | yes | - | - | Files or directories to upload, supports multiple |
| `--prefix` / `-p` | optional | no | - | `docker/{YYYYMMDDHHmmss}` | Custom path prefix in the bucket |
| `--endpoint` | optional | no | `MINIO_ENDPOINT` | `https://s3.nuwax.com:9443` | MinIO/S3 API endpoint |
| `--bucket` | optional | no | `MINIO_BUCKET` | `nuwax-packages` | Target bucket name |
| `--access-key` | required | yes | `MINIO_ACCESS_KEY` | - | Access key ID |
| `--secret-key` | required | yes | `MINIO_SECRET_KEY` | - | Secret access key |
| `--region` | optional | no | `MINIO_REGION` | `us-east-1` | S3 region (arbitrary for MinIO) |
| `--timeout` | optional | no | - | no limit | Request timeout in seconds (per HTTP request) |
| `--concurrency` / `-c` | optional | no | - | `3` | Number of files to upload concurrently |

Credentials can be provided via CLI arguments or environment variables. CLI arguments take priority.

## Examples

```bash
# Set credentials via environment variables
export MINIO_ACCESS_KEY=<your-access-key>
export MINIO_SECRET_KEY=your-secret-key

# Upload a single file
nuwax-upload service.zip

# Upload multiple files (with per-file progress bar)
nuwax-upload service.zip config.json

# Upload an entire directory (recursive)
nuwax-upload ./release-dir/

# Mixed upload with custom prefix
nuwax-upload --prefix releases/v2.0.0 changelog.txt ./dist/

# Specify credentials via CLI arguments
nuwax-upload --access-key xxx --secret-key xxx service.zip

# Set per-request timeout to 10 minutes
nuwax-upload --timeout 600 large-file.bin

# Adjust concurrent uploads (default: 3)
nuwax-upload --concurrency 5 *.zip
```

## Upload Behavior

- Files < 5MB: simple PUT upload
- Files >= 5MB: automatic S3 multipart upload (8MB chunks, internal concurrent parts)
- Multiple files are uploaded concurrently (default: 3 at a time)
- Directories are traversed recursively, preserving sub-directory structure
- Symlinks are not followed
- Upload fails fast on first error (cancels in-flight uploads)

## Output

After successful upload, download URLs for all uploaded files are printed for easy access.

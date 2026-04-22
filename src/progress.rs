use std::pin::Pin;
use std::task::{Context, Poll};

use indicatif::ProgressBar;
use tokio::io::AsyncRead;

/// Wraps an `AsyncRead` reader and updates an `indicatif::ProgressBar`
/// with the number of bytes read on each `poll_read` call.
pub struct ProgressReader<R> {
    inner: R,
    pb: ProgressBar,
}

impl<R> ProgressReader<R> {
    pub fn new(inner: R, pb: ProgressBar) -> Self {
        Self { inner, pb }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ProgressReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            let bytes_read = (buf.filled().len() - before) as u64;
            self.pb.inc(bytes_read);
        }
        result
    }
}

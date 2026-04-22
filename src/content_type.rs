/// Detect MIME type from a filename using the `mime_guess2` database.
///
/// Falls back to `application/octet-stream` for unknown extensions.
pub fn detect(filename: &str) -> String {
    mime_guess2::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_types() {
        assert_eq!(detect("file.zip"), "application/zip");
        assert_eq!(detect("file.tar"), "application/x-tar");
        assert_eq!(detect("file.tar.gz"), "application/gzip");
        assert_eq!(detect("file.json"), "application/json");
        assert_eq!(detect("file.txt"), "text/plain");
        assert_eq!(detect("file.pdf"), "application/pdf");
        assert_eq!(detect("file.html"), "text/html");
        assert_eq!(detect("file.css"), "text/css");
        assert_eq!(detect("file.png"), "image/png");
        assert_eq!(detect("file.jpg"), "image/jpeg");
        assert_eq!(detect("file.svg"), "image/svg+xml");
        assert_eq!(detect("file.wasm"), "application/wasm");
        assert_eq!(detect("file.xml"), "text/xml");
        assert_eq!(detect("file.unknown"), "application/octet-stream");
        assert_eq!(detect("noext"), "application/octet-stream");
    }
}

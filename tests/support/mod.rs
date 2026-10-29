//! Each test owns a fresh directory; never use a shared fixture working folder.

pub fn temp_dir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("pivra test Türkçe 🦀-")
        .tempdir()
        .expect("create isolated test directory")
}

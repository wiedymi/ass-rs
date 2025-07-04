//! IO convenience helpers for reading/writing ASS scripts.
//!
//! The API is intentionally thin; larger apps can roll their own.  All helpers keep the raw
//! bytes intact –  the parsing happens in `ass-core`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod blocking {
    use std::fs;
    use std::io::{self, Write};
    use std::path::Path;

    /// Read entire file into a byte vector.
    pub fn read_bytes<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    /// Read entire file as UTF-8 string (lossy — invalid bytes replaced by ``).
    pub fn read_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        fs::read(path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Write bytes to file (create + truncate).
    pub fn write_bytes<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
        fs::write(path, data)
    }

    /// Write UTF-8 string to file.
    pub fn write_string<P: AsRef<Path>>(path: P, data: &str) -> io::Result<()> {
        let mut f = fs::File::create(path)?;
        f.write_all(data.as_bytes())
    }
}

#[cfg(all(feature = "std", feature = "async"))]
pub mod r#async {
    //! Async versions powered by tokio.
    use std::path::Path;
    use tokio::fs;

    pub async fn read_bytes<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        fs::read(path).await
    }

    pub async fn write_bytes<P: AsRef<Path>>(path: P, data: &[u8]) -> std::io::Result<()> {
        fs::write(path, data).await
    }
}

// Re-export convenient blocking API at crate root when std is present.
#[cfg(feature = "std")]
pub use blocking::{read_bytes, read_string, write_bytes, write_string};

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rw_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sample.ass");
        let content = "[Script Info]\nTitle: Test\n";
        write_string(&file_path, content).unwrap();
        let read_back = read_string(&file_path).unwrap();
        assert_eq!(content, read_back);
    }
}

#[cfg(feature = "async")]
mod async_helpers {
    // Placeholder for async helpers once we implement them
}

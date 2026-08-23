//! Streaming BLAKE3 hashing utility used as the fallback verification step
//! when metadata alone cannot confirm two files are identical.

use std::fs::File;
use std::path::Path;

/// Computes the BLAKE3 hash of the file at `path` by streaming its
/// contents through a `Hasher`, avoiding loading the whole file into memory.
pub fn hash_file(path: &Path) -> anyhow::Result<blake3::Hash> {
    let file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file)?;
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn hash_matches_reference_implementation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        fs::write(&path, b"hello world").unwrap();

        let hash = hash_file(&path).unwrap();
        assert_eq!(hash, blake3::hash(b"hello world"));
    }

    #[test]
    fn different_content_yields_different_hash() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, b"content a").unwrap();
        fs::write(&b, b"content b").unwrap();

        assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn identical_content_yields_identical_hash() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, b"same bytes").unwrap();
        fs::write(&b, b"same bytes").unwrap();

        assert_eq!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn missing_file_returns_error() {
        let missing = Path::new("/nonexistent/path/should/not/exist.txt");
        assert!(hash_file(missing).is_err());
    }
}

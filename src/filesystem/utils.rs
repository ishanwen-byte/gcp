use std::path::{Path, PathBuf};
use std::fs;
use std::io;

/// Create intermediate directories for the given file path
pub fn create_intermediate_dirs(path: &PathBuf) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Resolve file conflicts by auto-renaming (append number)
pub fn resolve_conflict(path: &PathBuf) -> PathBuf {
    if !path.exists() {
        return path.clone();
    }

    let mut counter = 1;
    let stem = path.file_stem().unwrap_or_default();
    let extension = path.extension().and_then(|s| s.to_str());

    loop {
        let new_name = if let Some(ext) = extension {
            format!("{}_{}.{}", stem.to_string_lossy(), counter, ext)
        } else {
            format!("{}_{}", stem.to_string_lossy(), counter)
        };

        let new_path = path.with_file_name(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;

        // Prevent infinite loops
        if counter > 10000 {
            break;
        }
    }

    path.clone()
}

/// Validate that a path is safe to write to
pub fn validate_safe_path(path: &Path) -> io::Result<()> {
    // Basic safety checks - can be expanded
    if path.to_string_lossy().contains("..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path contains parent directory references",
        ));
    }
    Ok(())
}

/// Ensure the destination directory exists and is writable
pub fn ensure_destination_dir(dest: &PathBuf) -> io::Result<()> {
    validate_safe_path(dest)?;

    if dest.is_file() {
        // If destination is a file, its parent must be a directory
        if let Some(parent) = dest.parent() {
            create_intermediate_dirs(&parent.to_path_buf())?;
        }
    } else {
        // Create directory if it doesn't exist
        create_intermediate_dirs(dest)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_conflict_no_existing_file() {
        let path = PathBuf::from("/nonexistent/test.txt");
        let result = resolve_conflict(&path);
        assert_eq!(result, path);
    }

    #[test]
    fn test_resolve_conflict_with_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        // Create initial file
        File::create(&file_path).unwrap();

        let resolved = resolve_conflict(&file_path);
        assert_ne!(resolved, file_path);
        assert!(resolved.to_string_lossy().contains("test_1.txt"));
    }

    #[test]
    fn test_create_intermediate_dirs() {
        let dir = tempdir().unwrap();
        let deep_path = dir.path().join("a").join("b").join("c").join("file.txt");

        create_intermediate_dirs(&deep_path).unwrap();

        assert!(deep_path.parent().unwrap().exists());
    }
}
use crate::models::error::SError;
use camino::Utf8Path;
use walkdir::WalkDir;

pub fn create_dir_all(path: &Utf8Path) -> Result<(), SError> {
    std::fs::create_dir_all(path).map_err(Into::into)
}

pub fn remove_dir(path: &Utf8Path) -> Result<(), SError> {
    std::fs::remove_dir(path).map_err(Into::into)
}

pub fn remove_dir_all(path: &Utf8Path) -> Result<(), SError> {
    std::fs::remove_dir_all(path).map_err(Into::into)
}

pub fn rename(from: &Utf8Path, to: &Utf8Path) -> Result<(), SError> {
    std::fs::rename(from, to).map_err(Into::into)
}

pub fn copy(from: &Utf8Path, to: &Utf8Path) -> Result<(), SError> {
    std::fs::copy(from, to).map(|_| ()).map_err(Into::into)
}

pub fn write(path: &Utf8Path, contents: impl AsRef<[u8]>) -> Result<(), SError> {
    std::fs::write(path, contents).map_err(Into::into)
}

pub fn read_to_string(path: &Utf8Path) -> Result<String, SError> {
    std::fs::read_to_string(path).map_err(Into::into)
}

pub fn read_dir(path: &Utf8Path) -> Result<std::fs::ReadDir, SError> {
    std::fs::read_dir(path).map_err(Into::into)
}

/// Writes contents to a temp file in the target's directory, then renames it
/// over the target - the target is never observable half-written.
pub fn atomic_write(path: &Utf8Path, contents: impl AsRef<[u8]>) -> Result<(), SError> {
    let parent = path
        .parent()
        .ok_or_else(|| SError::IOError(format!("No parent directory for {path}")))?;
    create_dir_all(parent)?;

    let file_name = path.file_name().unwrap_or("file");
    let tmp = parent.join(format!("{}.tmp-{}", file_name, std::process::id()));

    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(())
}

pub fn is_dir_empty(path: &Utf8Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
}

/// Recursively copies a directory tree from source to destination.
/// Creates all necessary directories and overwrites existing files.
pub fn copy_recursive(src: &Utf8Path, dst: &Utf8Path) -> Result<(), SError> {
    // 1. Ensure the root destination directory exists
    std::fs::create_dir_all(dst)?;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        // 2. Convert standard Path to Camino Utf8Path
        let src_path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| SError::ParseError(format!("Invalid UTF-8 path: {:?}", entry.path())))?;

        // 3. Calculate the relative path from the source root
        let rel_path = src_path.strip_prefix(src)?;

        // 4. Construct the final destination path
        let dst_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            // 5. If it's a directory, create it in the destination
            std::fs::create_dir_all(&dst_path)?;
        } else {
            // 6. If it's a file, ensure the parent directory exists (safety check)
            if let Some(parent) = dst_path.parent()
                && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            // 7. Copy the file (Note: This overwrites existing files at the destination)
            std::fs::copy(src_path, &dst_path)?;
        }
    }

    Ok(())
}

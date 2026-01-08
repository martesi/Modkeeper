use crate::models::error::SError;
use camino::Utf8Path;
use walkdir::WalkDir;

pub struct FileUtils;

impl FileUtils {
    /// Recursively copies a directory tree from source to destination.
    /// Creates all necessary directories and overwrites existing files.
    pub fn copy_recursive(src: &Utf8Path, dst: &Utf8Path) -> Result<(), SError> {
        // 1. Ensure the root destination directory exists
        std::fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            // 2. Convert standard Path to Camino Utf8Path
            let src_path = Utf8Path::from_path(entry.path()).ok_or_else(|| {
                SError::ParseError(format!("Invalid UTF-8 path: {:?}", entry.path()))
            })?;

            // 3. Calculate the relative path from the source root
            let rel_path = src_path.strip_prefix(src)?;

            // 4. Construct the final destination path
            let dst_path = dst.join(rel_path);

            if entry.file_type().is_dir() {
                // 5. If it's a directory, create it in the destination
                std::fs::create_dir_all(&dst_path)?;
            } else {
                // 6. If it's a file, ensure the parent directory exists (safety check)
                if let Some(parent) = dst_path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                // 7. Copy the file (Note: This overwrites existing files at the destination)
                std::fs::copy(src_path, &dst_path)?;
            }
        }

        Ok(())
    }
}

use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use std::io;

/// Reads the target of a Symbolic Link.
pub fn read_link_target(path: &Utf8Path) -> io::Result<Utf8PathBuf> {
    let target = fs::read_link(path)?;
    Ok(Utf8PathBuf::from(target.to_string_lossy().to_string()))
}

/// Creates a symlink from target → source.
/// Uses OS-appropriate symlink APIs.
/// On Windows, requires Developer Mode or Administrator privileges.
pub fn link(source: &Utf8Path, target: &Utf8Path) -> io::Result<()> {
    // 1. Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // 2. Check if target already exists
    if target.exists() || target.is_symlink() {
        // Already a symlink pointing to the right source — idempotent
        if let Ok(existing_target) = read_link_target(target)
            && existing_target == source {
                return Ok(());
            }
        // Collision: target exists and points elsewhere (or isn't a symlink)
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Target exists and is not linked to source: {}", target),
        ));
    }

    // 3. Create symlink
    #[cfg(windows)]
    {
        if source.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)?;
        } else {
            std::os::windows::fs::symlink_file(source, target)?;
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    Ok(())
}

/// Safely removes a symlink or empty directory.
pub fn unlink(target: &Utf8Path) -> io::Result<()> {
    let meta = match fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    if meta.file_type().is_symlink() {
        #[cfg(windows)]
        {
            // On Windows, symlink_metadata().is_dir() does NOT follow the link,
            // so it's always false for a symlink. We can't use it to distinguish
            // file symlinks from directory symlinks. Instead, try remove_dir first
            // (works for directory symlinks), falling back to remove_file (for file symlinks).
            fs::remove_dir(target).or_else(|_| fs::remove_file(target))
        }
        #[cfg(unix)]
        {
            // On Unix, all symlinks (regardless of target type) are removed via unlink/remove_file
            fs::remove_file(target)
        }
    } else if meta.is_dir() {
        fs::remove_dir(target)
    } else {
        fs::remove_file(target)
    }
}

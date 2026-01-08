use camino::{Utf8Path, Utf8PathBuf};
use file_id::{get_file_id, FileId};
use std::fs;
use std::io;

/// Gets the unique physical ID of a file (Volume Serial + File Index / Inode).
/// Used to identify if two paths point to the exact same hard link.
pub fn get_id(path: &Utf8Path) -> io::Result<FileId> {
    get_file_id(path)
}

/// Checks if two paths point to the exact same physical file data.
pub fn is_same_file(path_a: &Utf8Path, path_b: &Utf8Path) -> bool {
    match (get_id(path_a), get_id(path_b)) {
        (Ok(id_a), Ok(id_b)) => id_a == id_b,
        _ => false,
    }
}

/// Reads the target of a Symbolic Link or Windows Junction.
pub fn read_link_target(path: &Utf8Path) -> io::Result<Utf8PathBuf> {
    let target = fs::read_link(path)?;
    Ok(Utf8PathBuf::from(target.to_string_lossy().to_string()))
}

/// Creates a link from source to target.
/// - Windows: Uses Hard Links for files, Junctions for directories.
/// - Unix: Uses Symbolic Links for everything.
pub fn link(source: &Utf8Path, target: &Utf8Path) -> io::Result<()> {
    // 1. Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // 2. Check if target already exists
    if target.exists() || target.is_symlink() {
        // Case A: It's a Directory/Junction
        if target.is_dir() {
            if let Ok(existing_target) = read_link_target(target) {
                // Normalize paths for comparison (optional but recommended)
                if existing_target == source {
                    return Ok(()); // Already linked correctly
                }
            }
        }
        // Case B: It's a File/Hard Link
        else if is_same_file(source, target) {
            return Ok(()); // Already linked correctly
        }

        // Case C: Collision
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Target exists and is not linked to source: {}", target),
        ));
    }

    // 3. Create the Link
    #[cfg(windows)]
    {
        if source.is_dir() {
            // Junctions allow linking directories without Admin rights
            junction::create(source, target)?;
        } else {
            // Hard links allow linking files without Admin rights
            // Note: Source and Target must be on the same Drive volume.
            fs::hard_link(source, target)?;
        }
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    Ok(())
}

/// Safely removes a link, file, or empty directory.
/// Handles platform differences between Junctions, Symlinks, and Files.
pub fn unlink(target: &Utf8Path) -> io::Result<()> {
    // Check if path exists or is a broken symlink
    let meta = match fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    #[cfg(windows)]
    {
        // On Windows, if it's a directory OR any kind of link (Junction/Symlink),
        // we should try remove_dir first.
        if meta.is_dir() || meta.file_type().is_symlink() {
            // If remove_dir fails (e.g. it was actually a file symlink), fall back to remove_file
            fs::remove_dir(target).or_else(|_| fs::remove_file(target))
        } else {
            fs::remove_file(target)
        }
    }

    #[cfg(unix)]
    {
        // On Unix, a symlink is removed via unlink (remove_file),
        // even if it points to a directory.
        if meta.file_type().is_symlink() {
            fs::remove_file(target)
        } else if meta.is_dir() {
            // Only use remove_dir if it's a real directory (not a link)
            fs::remove_dir(target)
        } else {
            fs::remove_file(target)
        }
    }
}


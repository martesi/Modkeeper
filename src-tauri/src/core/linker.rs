use camino::{Utf8Path, Utf8PathBuf};
use file_id::{get_file_id, FileId};
use std::fs;
use std::io;

pub struct Linker;

impl Linker {
    /// Gets the unique physical ID of a file (Volume Serial + File Index / Inode).
    /// Used to identify if two paths point to the exact same hard link.
    pub fn get_id(path: &Utf8Path) -> io::Result<FileId> {
        get_file_id(path)
    }

    /// Checks if two paths point to the exact same physical file data.
    pub fn is_same_file(path_a: &Utf8Path, path_b: &Utf8Path) -> bool {
        match (Self::get_id(path_a), Self::get_id(path_b)) {
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
                if let Ok(existing_target) = Self::read_link_target(target) {
                    // Normalize paths for comparison (optional but recommended)
                    if existing_target == source {
                        return Ok(()); // Already linked correctly
                    }
                }
            }
            // Case B: It's a File/Hard Link
            else if Self::is_same_file(source, target) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_hard_link() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let src = root.join("source.txt");
        let dst = root.join("target.txt");

        fs::write(&src, "hello linker").unwrap();

        // 1. Create Link
        Linker::link(&src, &dst).expect("Failed to create hard link");

        // 2. Verify content is shared
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello linker");

        // 3. Verify they are the same physical file
        assert!(Linker::is_same_file(&src, &dst));

        // 4. Verify modifying one affects the other
        fs::write(&src, "changed").unwrap();
        assert_eq!(fs::read_to_string(&dst).unwrap(), "changed");
    }

    #[test]
    fn test_directory_link() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let src_dir = root.join("source_dir");
        let dst_dir = root.join("target_dir");
        let inner_file = src_dir.join("data.txt");

        fs::create_dir_all(&src_dir).unwrap();
        fs::write(&inner_file, "nested data").unwrap();

        // 1. Create Directory Link (Junction on Windows, Symlink on Unix)
        Linker::link(&src_dir, &dst_dir).expect("Failed to link directory");

        // 2. Verify visibility
        let linked_file = dst_dir.join("data.txt");
        assert!(linked_file.exists());
        assert_eq!(fs::read_to_string(linked_file).unwrap(), "nested data");

        // 3. Verify read_link_target
        let target = Linker::read_link_target(&dst_dir).expect("Failed to read link");
        // Canonicalization differences might occur, but checking ends_with is safe
        assert!(target.ends_with("source_dir"));
    }

    #[test]
    fn test_unlink_logic() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let src = root.join("original.txt");
        let dst = root.join("link.txt");
        fs::write(&src, "keep me").unwrap();

        Linker::link(&src, &dst).unwrap();
        assert!(dst.exists());

        // 1. Unlink the link
        Linker::unlink(&dst).expect("Failed to unlink");

        // 2. Assert link is gone but source remains
        assert!(!dst.exists());
        assert!(src.exists());
    }

    #[test]
    fn test_link_already_exists_correctly() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let src = root.join("src.txt");
        let dst = root.join("dst.txt");
        fs::write(&src, "test").unwrap();

        // Link first time
        Linker::link(&src, &dst).unwrap();

        // Link second time (should be idempotent / return Ok)
        let result = Linker::link(&src, &dst);
        assert!(result.is_ok(), "Subsequent link to same source should succeed");
    }

    #[test]
    fn test_collision_detection() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();

        let src_a = root.join("a.txt");
        let src_b = root.join("b.txt");
        let dst = root.join("collision.txt");

        fs::write(&src_a, "content a").unwrap();
        fs::write(&src_b, "content b").unwrap();

        // 1. Link A to Target
        Linker::link(&src_a, &dst).unwrap();

        // 2. Attempt to Link B to Target (Collision!)
        let result = Linker::link(&src_b, &dst);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn test_unlink_non_existent_path() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let path = root.join("ghost.txt");

        // Should not error if path doesn't exist
        let result = Linker::unlink(&path);
        assert!(result.is_ok());
    }
}
use camino::Utf8Path;
use std::fs;

pub struct Linker;

impl Linker {
    pub fn link(source: &Utf8Path, target: &Utf8Path) -> std::io::Result<()> {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if target already exists
        if target.exists() || target.is_symlink() {
            // If it's a link, verify it points to the correct source
            #[cfg(unix)]
            {
                if let Ok(link_target) = fs::read_link(target) {
                    let link_target_str = link_target.to_string_lossy().to_string();
                    let link_path = Utf8Path::new(&link_target_str);
                    if link_path == source {
                        return Ok(()); // Link is correct, skip
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            format!("Link exists but points to different target: {:?}", link_path),
                        ));
                    }
                }
            }
            #[cfg(windows)]
            {
                // For Windows junctions, check metadata
                if let Ok(meta) = fs::metadata(target) {
                    if meta.is_symlink() {
                        if let Ok(link_target) = fs::read_link(target) {
                            let link_target_str = link_target.to_string_lossy().to_string();
                            let link_path = Utf8Path::new(&link_target_str);
                            if link_path == source {
                                return Ok(()); // Link is correct, skip
                            }
                        }
                    }
                }
            }
            // If we get here, target exists but is not a valid link to source
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Target exists but is not a link to source",
            ));
        }

        // Create link
        #[cfg(windows)]
        {
            if source.is_dir() {
                junction::create(source, target)
            } else {
                fs::hard_link(source, target)
            }
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)
        }
    }

    pub fn unlink(target: &Utf8Path) -> std::io::Result<()> {
        // Only unlink if it's actually a link or symlink; skip if not
        if !target.exists() && !target.is_symlink() {
            return Ok(());
        }

        let meta = fs::symlink_metadata(target)?;

        #[cfg(windows)]
        {
            if meta.is_dir() {
                fs::remove_dir(target)?;
            } else {
                fs::remove_file(target)?;
            }
            Ok(())
        }
        #[cfg(unix)]
        {
            // On Unix, only remove if it's a symlink
            if meta.is_symlink() {
                fs::remove_file(target)?;
                Ok(())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Target is not a symlink",
                ))
            }
        }
    }
}
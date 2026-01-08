use crate::models::error::SError;
use camino::Utf8Path;
use std::fs::{self, File};
use std::io;

pub struct Decompression;

impl Decompression {
    pub fn extract(archive_path: &Utf8Path, destination: &Utf8Path) -> Result<(), SError> {
        // 1. Open the archive file
        let file = File::open(archive_path)?;

        let mut archive = zip::ZipArchive::new(file)?;

        // 2. Iterate through all files in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            // 3. Security: Prevent "Zip Slip"
            // enclosed_name() ensures the path is valid and inside the target directory
            let safe_path = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue, // Skip unsafe paths
            };

            let output_path = destination.as_std_path().join(&safe_path);

            // 4. Handle Directories
            if file.is_dir() {
                fs::create_dir_all(&output_path)?;
            }
            // 5. Handle Files
            else {
                // Ensure parent directory exists
                if let Some(parent) = output_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                let mut outfile = File::create(&output_path)?;

                io::copy(&mut file, &mut outfile)?;
            }

            // 6. (Optional) Preserve Permissions on Unix/Linux/Mac
            // This is important for executables inside mods
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let _ = fs::set_permissions(&output_path, fs::Permissions::from_mode(mode));
                }
            }
        }

        Ok(())
    }
}

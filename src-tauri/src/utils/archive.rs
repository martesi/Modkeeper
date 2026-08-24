use crate::models::error::SError;
use camino::Utf8Path;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArchiveFormat {
    Zip,
    SevenZ,
}

impl ArchiveFormat {
    pub(crate) fn from_path(path: &Utf8Path) -> Option<Self> {
        let extension = path.extension()?.to_ascii_lowercase();
        match extension.as_str() {
            "zip" => Some(Self::Zip),
            "7z" => Some(Self::SevenZ),
            _ => None,
        }
    }
}
pub fn extract(archive_path: &Utf8Path, destination: &Utf8Path) -> Result<(), SError> {
    match ArchiveFormat::from_path(archive_path) {
        Some(ArchiveFormat::Zip) => extract_zip(archive_path, destination),
        Some(ArchiveFormat::SevenZ) => extract_seven_z(archive_path, destination),
        None => Err(SError::UnhandledCompression(archive_path.to_string())),
    }
}

fn extract_zip(archive_path: &Utf8Path, destination: &Utf8Path) -> Result<(), SError> {
    // 1. Open the archive file
    let file = File::open(archive_path)?;

    let mut archive = zip::ZipArchive::new(file)?;

    let mut entries = Vec::new();
    let mut seen = HashMap::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let safe_path = validate_entry_path(&mut seen, file.name(), file.is_dir())?;
        entries.push((safe_path, i));
    }

    for (safe_path, index) in entries {
        let mut file = archive.by_index(index)?;

        let output_path = destination.as_std_path().join(&safe_path);

        // 4. Handle Directories
        if file.is_dir() {
            fs::create_dir_all(&output_path)?;
        }
        // 5. Handle Files
        else {
            // Ensure parent directory exists
            if let Some(parent) = output_path.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
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

fn normalize_entry_path(name: &str) -> Option<PathBuf> {
    let normalized = name.replace('\\', "/");
    let mut path = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) | Component::ParentDir => return None,
        }
    }
    (!path.as_os_str().is_empty()).then_some(path)
}
fn extract_seven_z(archive_path: &Utf8Path, destination: &Utf8Path) -> Result<(), SError> {
    // Validate all names before extraction so a collision cannot leave a partially staged mod.
    let archive = sevenz_rust2::Archive::open(archive_path.as_std_path())?;
    let mut seen = HashMap::new();
    for entry in &archive.files {
        validate_entry_path(&mut seen, entry.name(), entry.is_directory())?;
    }

    // The crate's helper performs a second traversal check while writing entries.
    sevenz_rust2::decompress_file(archive_path.as_std_path(), destination.as_std_path())?;
    Ok(())
}

fn validate_entry_path(
    seen: &mut HashMap<PathBuf, bool>,
    name: &str,
    is_dir: bool,
) -> Result<PathBuf, SError> {
    let Some(safe_path) = normalize_entry_path(name) else {
        return Err(SError::ParseError(format!("Unsafe archive path: {name}")));
    };

    let duplicate = seen.insert(safe_path.clone(), is_dir).is_some();
    let file_parent = seen.iter().any(|(path, existing_is_dir)| {
        !*existing_is_dir && path != &safe_path && safe_path.starts_with(path)
    });
    let directory_parent = !is_dir
        && seen
            .keys()
            .any(|path| path.starts_with(&safe_path) && path != &safe_path);

    if duplicate || file_parent || directory_parent {
        return Err(SError::ParseError(format!(
            "Archive path collision after normalization: {}",
            safe_path.display()
        )));
    }

    Ok(safe_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use std::io::{Cursor, Write};
    use tempfile::tempdir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    #[test]
    fn normalizes_windows_separators() {
        assert_eq!(
            normalize_entry_path(r"BepInEx\plugins\mod.dll"),
            Some(PathBuf::from("BepInEx/plugins/mod.dll"))
        );
    }

    #[test]
    fn recognizes_only_supported_archive_extensions() {
        assert_eq!(
            ArchiveFormat::from_path(Utf8Path::new("mod.ZIP")),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_path(Utf8Path::new("mod.7z")),
            Some(ArchiveFormat::SevenZ)
        );
        assert_eq!(ArchiveFormat::from_path(Utf8Path::new("mod.rar")), None);
    }

    #[test]
    fn rejects_normalized_collisions() {
        let temp = tempdir().unwrap();
        let archive_path = temp.path().join("mods.zip");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        writer
            .start_file(r"BepInEx\plugins\mod.dll", options)
            .unwrap();
        writer.write_all(b"one").unwrap();
        writer
            .start_file("BepInEx/plugins/mod.dll", options)
            .unwrap();
        writer.write_all(b"two").unwrap();
        writer.finish().unwrap();

        let archive = Utf8PathBuf::from_path_buf(archive_path).unwrap();
        let result = extract(
            &archive,
            &Utf8PathBuf::from_path_buf(temp.path().join("out")).unwrap(),
        );
        assert!(
            matches!(result, Err(SError::ParseError(message)) if message.contains("collision"))
        );
    }

    #[test]
    fn extracts_seven_z_archives() {
        let temp = tempdir().unwrap();
        let archive_path = temp.path().join("mods.7z");
        let mut writer = sevenz_rust2::ArchiveWriter::create(&archive_path).unwrap();
        writer
            .push_archive_entry(
                sevenz_rust2::ArchiveEntry::new_file("BepInEx/plugins/example.dll"),
                Some(Cursor::new(b"payload".to_vec())),
            )
            .unwrap();
        writer.finish().unwrap();

        let archive = Utf8PathBuf::from_path_buf(archive_path).unwrap();
        let destination = Utf8PathBuf::from_path_buf(temp.path().join("out")).unwrap();
        extract(&archive, &destination).unwrap();

        assert_eq!(
            fs::read(destination.join("BepInEx/plugins/example.dll")).unwrap(),
            b"payload"
        );
    }
}

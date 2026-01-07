use crate::models::divider::MOD_ID_DIVIDER;
use crate::models::error::SError;
use crate::models::mod_dto::{ModManifest, ModType};
use crate::models::paths::{ModPaths, SPTPathRules};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

// Internal cache representation: includes files but NOT sent to frontend
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModFS {
    pub id: String,
    pub mod_type: ModType,
    pub files: Vec<Utf8PathBuf>,
    pub executables: Vec<Utf8PathBuf>
}

impl ModFS {
    fn read_manifest_guid(manifest_path: &Utf8Path) -> Result<String, SError> {
        Ok(Self::read_manifest(manifest_path)?.guid)
    }

    pub fn read_manifest(path: &Utf8Path) -> Result<ModManifest, SError> {
        Ok(serde_json::from_reader(std::fs::File::open(path)?)?)
    }

    pub fn resolve_id(
        mod_root: &Utf8Path,
        spt_paths: &SPTPathRules,
        files: &[Utf8PathBuf],
    ) -> Result<String, SError> {
        // 1. Priority 1: Manifest check
        let mod_paths = ModPaths::new(mod_root);
        if let Ok(guid) = Self::read_manifest_guid(&mod_paths.file) {
            return Ok(guid);
        }

        // 2. Single-pass collection using BTreeSet for automatic sorting
        let ids: std::collections::BTreeSet<String> = files
            .iter()
            .filter_map(|path| {
                // Server check
                if let Ok(rel) = path.strip_prefix(&spt_paths.server_mods) {
                    return rel.components().next().map(|c| c.as_str().to_string());
                }

                // Client check (DLLs only)
                if path.extension() == Some("dll") {
                    if let Ok(rel) = path.strip_prefix(&spt_paths.client_plugins) {
                        return Some(rel.as_str().to_string());
                    }
                }

                None // Ignore file if it matches neither
            })
            .collect();

        if ids.is_empty() {
            return Err(SError::UnableToDetermineModId);
        }

        // 3. Final join (ids is already sorted because it's a BTreeSet)
        Ok(ids.into_iter().collect::<Vec<_>>().join(MOD_ID_DIVIDER).to_lowercase())
    }

    pub fn infer_mod_type(files: &[Utf8PathBuf], config: &SPTPathRules) -> ModType {
        let has_client = files.iter().any(|p| p.starts_with(&config.client_plugins));
        let has_server = files.iter().any(|p| p.starts_with(&config.server_mods));

        match (has_client, has_server) {
            (true, true) => ModType::Both,
            (true, false) => ModType::Client,
            (false, true) => ModType::Server,
            _ => ModType::Unknown,
        }
    }

    fn collect_files(base: &Utf8Path) -> (Vec<Utf8PathBuf>, Vec<Utf8PathBuf>) {
        let manifest_folder = ModPaths::default().folder;

        WalkDir::new(base)
            .into_iter()
            // 1. Convert Result<DirEntry> to Option<DirEntry>
            .filter_map(Result::ok)
            // 2. Filter for files only
            .filter(|e| e.path().is_file())
            // 3. Transform to Utf8PathBuf and strip prefix using Result/Option combinators
            .filter_map(|entry| {
                Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).ok()
                    .and_then(|path| path.strip_prefix(base).ok().map(|p| p.to_path_buf()))
            })
            // 4. Remove manifest files
            .filter(|rel| !rel.starts_with(&manifest_folder))
            // 5. Fold into a tuple of (AllFiles, Executables)
            .fold((Vec::new(), Vec::new()), |(mut all, mut exes), path| {
                // Use Option::filter to handle the conditional push without an "if"
                path.extension()
                    .filter(|&ext| ext == "exe")
                    .inspect(|_| exes.push(path.clone()));

                all.push(path);
                (all, exes)
            })
    }

    pub fn copy_recursive(src: &Utf8Path, dst: &Utf8Path) -> Result<(), SError> {
        // 1. Ensure the root destination directory exists
        std::fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            // 2. Convert standard Path to Camino Utf8Path
            let src_path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| SError::ParseError(format!("Invalid UTF-8 path: {:?}", entry.path())))?;

            // 3. Calculate the relative path from the source root
            let rel_path = src_path
                .strip_prefix(src)?;

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

    pub fn new(root: &Utf8Path, spt_paths: &SPTPathRules) -> Result<Self, SError> {
        let (files,executables) = Self::collect_files(root); // Call once

        Ok(ModFS {
            id: Self::resolve_id(root, &spt_paths, &files)?,
            mod_type: Self::infer_mod_type(&files, &spt_paths),
            files, // Use the same vector
            executables
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_id_server_only() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Server structure: SPT/user/mods/WeatherMod/mod.dll
        let server_path = root.join(&rules.server_mods).join("WeatherMod/mod.dll");
        fs::create_dir_all(server_path.parent().unwrap()).unwrap();
        fs::write(server_path, "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // Server ID should be just the first folder name: "WeatherMod"
        assert_eq!(mod_fs.id, "WeatherMod");
        assert_eq!(mod_fs.mod_type, ModType::Server);
    }

    #[test]
    fn test_resolve_id_client_only() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Client structure: BepInEx/plugins/AuthorName/Logic.dll
        let client_path = root.join(&rules.client_plugins).join("AuthorName/Logic.dll");
        fs::create_dir_all(client_path.parent().unwrap()).unwrap();
        fs::write(client_path, "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // Client ID logic is rel.as_str().
        // We compare against a PathBuf to handle \ vs / automatically.
        let expected_rel = Utf8Path::new("AuthorName").join("Logic.dll");
        assert_eq!(mod_fs.id, expected_rel.as_str());
        assert_eq!(mod_fs.mod_type, ModType::Client);
    }

    #[test]
    fn test_resolve_id_combined() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Create both
        let s_path = root.join(&rules.server_mods).join("CoreMod/mod.dll");
        let c_path = root.join(&rules.client_plugins).join("Fixes.dll");

        fs::create_dir_all(s_path.parent().unwrap()).unwrap();
        fs::create_dir_all(c_path.parent().unwrap()).unwrap();
        fs::write(s_path, "").unwrap();
        fs::write(c_path, "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // BTreeSet sorts alphabetically:
        // "CoreMod" vs "Fixes.dll"
        // Result: "CoreMod--Fixes.dll"
        let expected = format!("CoreMod{}Fixes.dll", MOD_ID_DIVIDER);
        assert_eq!(mod_fs.id, expected);
        assert_eq!(mod_fs.mod_type, ModType::Both);
    }

    #[test]
    fn test_resolve_id_ignores_non_dll_in_client_plugins() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // .txt files in plugins should be ignored for ID generation
        let txt_path = root.join(&rules.client_plugins).join("readme.txt");
        fs::create_dir_all(txt_path.parent().unwrap()).unwrap();
        fs::write(txt_path, "").unwrap();

        // Valid server mod to ensure the test doesn't fail on "empty"
        let s_path = root.join(&rules.server_mods).join("ValidMod/mod.dll");
        fs::create_dir_all(s_path.parent().unwrap()).unwrap();
        fs::write(s_path, "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // ID should only be "ValidMod", the readme.txt is ignored
        assert_eq!(mod_fs.id, "ValidMod");
    }

    #[test]
    fn test_collect_files_excludes_manifest() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Create manifest folder
        let manifest_dir = root.join(ModPaths::default().folder);
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(manifest_dir.join("info.json"), "").unwrap();

        // Create actual mod file
        let mod_file = root.join(&rules.server_mods).join("Mod/mod.dll");
        fs::create_dir_all(mod_file.parent().unwrap()).unwrap();
        fs::write(&mod_file, "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // Only the dll should be in the files list
        assert_eq!(mod_fs.files.len(), 1);
        assert!(mod_fs.files[0].ends_with("mod.dll"));
    }

    #[test]
    fn test_executables_detection() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Create a server dll (to pass ID resolution)
        let mod_dir = root.join(&rules.server_mods).join("ModWithTools");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("mod.dll"), "").unwrap();

        // Create exes in different locations
        fs::write(root.join("root_tool.exe"), "").unwrap();
        let deep_dir = mod_dir.join("tools/sub");
        fs::create_dir_all(&deep_dir).unwrap();
        fs::write(deep_dir.join("nested_tool.exe"), "").unwrap();

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        assert_eq!(mod_fs.executables.len(), 2);
        assert!(mod_fs.executables.iter().any(|e| e.ends_with("root_tool.exe")));
        assert!(mod_fs.executables.iter().any(|e| e.ends_with("nested_tool.exe")));
    }

    #[test]
    fn test_copy_recursive_overwrite_and_deep_nesting() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let src = root.join("src");
        let dst = root.join("dst");

        // 1. Setup source with deep nesting
        let deep_file = src.join("a/b/c/d.txt");
        fs::create_dir_all(deep_file.parent().unwrap()).unwrap();
        fs::write(&deep_file, "new content").unwrap();

        // 2. Setup destination with an old version of the file
        let old_file = dst.join("a/b/c/d.txt");
        fs::create_dir_all(old_file.parent().unwrap()).unwrap();
        fs::write(&old_file, "old content").unwrap();

        // 3. Act
        ModFS::copy_recursive(&src, &dst).unwrap();

        // 4. Assert
        let content = fs::read_to_string(dst.join("a/b/c/d.txt")).unwrap();
        assert_eq!(content, "new content"); // Verified overwrite
    }

    #[test]
    fn test_new_fails_for_invalid_structure() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Create files that don't match rules (not in SPT/user/mods or BepInEx/plugins)
        fs::create_dir_all(root.join("RandomFolder")).unwrap();
        fs::write(root.join("RandomFolder/something.dll"), "").unwrap();

        let result = ModFS::new(&root, &rules);

        assert!(result.is_err());
        // Match against your specific error variant
        match result.unwrap_err() {
            SError::UnableToDetermineModId => {},
            e => panic!("Expected UnableToDetermineModId, got {:?}", e),
        }
    }

    #[test]
    fn test_deterministic_id_sorting() {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let rules = SPTPathRules::default();

        // Create mods out of alphabetical order
        let mods = ["Z_mod", "A_mod", "M_mod"];
        for m in mods {
            let p = root.join(&rules.server_mods).join(m).join("mod.dll");
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "").unwrap();
        }

        let mod_fs = ModFS::new(&root, &rules).unwrap();

        // BTreeSet should have forced: A_mod--M_mod--Z_mod
        let expected = format!("A_mod{0}M_mod{0}Z_mod", MOD_ID_DIVIDER);
        assert_eq!(mod_fs.id, expected);
    }
}
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::utils::process::ProcessChecker;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use sysinfo::System;
use uuid::Uuid;
use crate::core::decompression::Decompression;

pub struct ModStager;

pub struct StagedMod {
    pub fs: ModFS,
    pub source_path: Utf8PathBuf, // The location in staging (or original folder)
    pub is_staging: bool,         // True if this is a temp folder we need to delete later
}

impl ModStager {
    /// Takes raw user inputs and converts them into validated ModFS objects ready for installation.
    /// Does NOT touch the active Library.
    pub fn resolve(
        inputs: &[Utf8PathBuf],
        rules: &SPTPathRules,
        staging_root: &Utf8Path,
    ) -> Result<Vec<StagedMod>, SError> {
        let mut results = Vec::new();

        // 1. Check if the input list is a "Loose File" mod (Game Root structure)
        if Self::is_game_root_structure(inputs, rules) {
            let staged = Self::stage_loose_files(inputs, rules, staging_root)?;
            results.push(staged);
            return Ok(results);
        }

        // 2. Process individual inputs
        for input in inputs {
            if input.is_dir() {
                // Case A: The folder itself is a Game Root (contains user/ or BepInEx/)
                if Self::folder_matches_game_structure(input, rules)? {
                    // Use directly "On Spot"
                    let fs = ModFS::new(input, rules)?;
                    results.push(StagedMod {
                        fs,
                        source_path: input.clone(),
                        is_staging: false,
                    });
                }
                // Case B: Standard Mod Folder
                else if let Ok(fs) = ModFS::new(input, rules) {
                    results.push(StagedMod {
                        fs,
                        source_path: input.clone(),
                        is_staging: false,
                    });
                }
            }
            // Case C: Archive (Zip/7z)
            else if Self::is_archive(input) {
                let staged = Self::stage_archive(input, rules, staging_root)?;
                results.push(staged);
            }
        }

        Ok(results)
    }

    /// Checks if it is safe to install these mods.
    /// Requires the System lock and lists of currently active/installed executables.
    pub fn any_mod_tool_running(
        sys: &mut System,
        mods_to_install: &[StagedMod],
    ) -> Result<(), SError> {
        // 2. Check if the NEW mods contain executables that are currently running
        // (e.g. user is trying to update a tool that is currently open)
        let mut specific_paths = Vec::new();
        for m in mods_to_install {
            for exe in &m.fs.executables {
                // Check the executable at its source location
                specific_paths.push(m.source_path.join(exe));
            }
        }

        if ProcessChecker::is_running(sys, &specific_paths) {
            return Err(SError::ProcessRunning);
        }

        Ok(())
    }

    // --- Internal Logic (Pure Functions) ---

    fn is_game_root_structure(inputs: &[Utf8PathBuf], rules: &SPTPathRules) -> bool {
        let roots = [
            Self::get_root_component(&rules.server_mods),
            Self::get_root_component(&rules.client_plugins),
        ];

        inputs.iter().any(|path| {
            path.file_name()
                .map(|name| roots.contains(&Some(name)))
                .unwrap_or(false)
        })
    }

    fn folder_matches_game_structure(
        folder: &Utf8Path,
        rules: &SPTPathRules,
    ) -> Result<bool, SError> {
        let roots = [
            Self::get_root_component(&rules.server_mods),
            Self::get_root_component(&rules.client_plugins),
        ];

        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            if let Ok(name) = entry.file_name().into_string() {
                if roots.contains(&Some(name.as_str())) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn stage_loose_files(
        inputs: &[Utf8PathBuf],
        rules: &SPTPathRules,
        staging_root: &Utf8Path,
    ) -> Result<StagedMod, SError> {
        let uuid = Uuid::new_v4().to_string();
        let dest_dir = staging_root.join(uuid);
        fs::create_dir_all(&dest_dir)?;

        for input in inputs {
            let name = input.file_name().ok_or_else(|| {
                SError::ParseError(format!("Unable to get file name for {input}"))
            })?;
            ModFS::copy_recursive(input, &dest_dir.join(name))?;
        }

        let fs = ModFS::new(&dest_dir, rules)?;
        Ok(StagedMod {
            fs,
            source_path: dest_dir,
            is_staging: true,
        })
    }

    fn stage_archive(
        archive: &Utf8Path,
        rules: &SPTPathRules,
        staging_root: &Utf8Path,
    ) -> Result<StagedMod, SError> {
        let uuid = Uuid::new_v4().to_string();
        let dest_dir = staging_root.join(uuid);
        fs::create_dir_all(&dest_dir)?;

        Decompression::extract(archive, &dest_dir)?;

        let fs = ModFS::new(&dest_dir, rules)?;
        Ok(StagedMod {
            fs,
            source_path: dest_dir,
            is_staging: true,
        })
    }

    fn is_archive(path: &Utf8Path) -> bool {
        let ext = path.extension().unwrap_or("").to_lowercase();
        matches!(ext.as_str(), "zip")
    }

    fn get_root_component(path: &Utf8Path) -> Option<&str> {
        path.components().next().map(|c| c.as_str())
    }
}

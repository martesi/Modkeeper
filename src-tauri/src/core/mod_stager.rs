use crate::core::decompression::Decompression;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::utils::process::ProcessChecker;
use camino::{Utf8Path, Utf8PathBuf};
use log::debug;
use std::fs;
use std::fs::remove_dir_all;
use sysinfo::System;
use uuid::Uuid;

pub struct ModStager;

#[derive(Debug)]
pub struct StagedMod {
    pub fs: ModFS,
    pub source_path: Utf8PathBuf, // The location in staging (or original folder)
    pub is_staging: bool,         // True if this is a temp folder we need to delete later
}

#[derive(Debug)]
pub struct StageMaterial {
    pub rules: SPTPathRules,
    pub root: Utf8PathBuf,
}

impl ModStager {
    /// Takes raw user inputs and converts them into validated ModFS objects ready for installation.
    /// Uses a functional pipeline to resolve inputs.
    pub fn resolve(
        inputs: &[Utf8PathBuf],
        StageMaterial { root, rules }: &StageMaterial,
    ) -> Result<Vec<StagedMod>, SError> {
        // 1. Guard Clause: Collective "Loose File" Check
        // If the inputs collectively form a mod root, treat them as one unit immediately.
        if Self::is_game_root_structure(inputs, &rules) {
            return Self::stage_loose_files(inputs, &rules, &root).map(|staged| vec![staged]);
        }

        // 2. Functional Pipeline: Process individual inputs
        inputs
            .iter()
            .map(|input| {
                // Chain strategies: Try Directory -> If None, Try Archive
                Self::process_as_directory(input, &rules)
                    .or_else(|| Self::process_as_archive(input, &rules, &root))
            })
            // Remove inputs that matched no strategy (Option::None)
            .filter_map(|res_opt| res_opt)
            // Collect into Result<Vec<_>>, returning the first Error if any occur
            .collect()
    }

    /// Checks if it is safe to install these mods.
    pub fn any_mod_tool_running(
        sys: &mut System,
        mods_to_install: &[StagedMod],
    ) -> Result<(), SError> {
        let specific_paths: Vec<_> = mods_to_install
            .iter()
            .flat_map(|m| m.fs.executables.iter().map(|exe| m.source_path.join(exe)))
            .collect();

        if ProcessChecker::is_running(sys, &specific_paths) {
            return Err(SError::ProcessRunning);
        }

        Ok(())
    }

    // --- Strategy Functions (Option<Result<...>>) ---

    /// Strategy A: Input is a directory.
    /// Returns:
    /// - Some(Ok): Valid mod found.
    /// - Some(Err): Valid mod structure found but failed to parse (Critical Error).
    /// - None: Not a directory, or not a mod (safe to try next strategy).
    fn process_as_directory(
        input: &Utf8PathBuf,
        rules: &SPTPathRules,
    ) -> Option<Result<StagedMod, SError>> {
        if !input.is_dir() {
            return None;
        }

        // Sub-strategy A1: Folder has strict Game Root structure (user/ or BepInEx/)
        // We use boolean matching to avoid deep nesting.
        let is_game_structure =
            Self::folder_matches_game_structure(input, rules).map_err(SError::from); // Propagate IO errors if they happen

        match is_game_structure {
            Ok(true) => {
                // It IS a game structure, so it MUST be a valid mod. Fail if ModFS::new fails.
                Some(ModFS::new(input, rules).map(|fs| StagedMod {
                    fs,
                    source_path: input.clone(),
                    is_staging: false,
                }))
            }
            Ok(false) => {
                // Sub-strategy A2: Folder is a standard mod folder.
                // We try ModFS::new. If it succeeds, Good. If it fails, we treat it as "Not a mod" (None).
                ModFS::new(input, rules).ok().map(|fs| {
                    Ok(StagedMod {
                        fs,
                        source_path: input.clone(),
                        is_staging: false,
                    })
                })
            }
            Err(e) => Some(Err(e)), // Critical IO error reading dir
        }
    }

    /// Strategy B: Input is an archive.
    fn process_as_archive(
        input: &Utf8PathBuf,
        rules: &SPTPathRules,
        staging_root: &Utf8Path,
    ) -> Option<Result<StagedMod, SError>> {
        Self::is_archive(input).then(|| Self::stage_archive(input, rules, staging_root))
    }

    // --- Internal Helpers ---

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

        // Using iterator to avoid manual loop
        let has_match = fs::read_dir(folder)?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .any(|name| roots.contains(&Some(name.as_str())));

        Ok(has_match)
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
        path.extension()
            .map(|ext| ext.to_lowercase() == "zip")
            .unwrap_or(false)
    }

    fn get_root_component(path: &Utf8Path) -> Option<&str> {
        path.components().next().map(|c| c.as_str())
    }

    pub fn clean_up(
        StagedMod {
            is_staging,
            source_path,
            ..
        }: &StagedMod,
    ) -> Result<(), SError> {
        if !is_staging {
            return Ok(());
        }
        debug!("clean up for {source_path}");
        remove_dir_all(source_path).map_err(Into::into)
    }
}

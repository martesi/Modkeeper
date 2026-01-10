use crate::core::decompression;
use crate::core::mod_fs::ModFS;
use crate::models::error::SError;
use crate::models::paths::{ModPaths, SPTPathRules};
use crate::utils::file::FileUtils;
use crate::utils::process::ProcessChecker;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use std::fs::remove_dir_all;
use sysinfo::System;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug)]
pub struct StagedMod {
    pub fs: ModFS,
    pub source_path: Utf8PathBuf, // The location in staging (or original folder)
    pub is_staging: bool,         // True if this is a temp folder we need to delete later
    pub name: String,             // The resolved name for the mod
}

#[derive(Debug)]
pub struct StageMaterial {
    pub rules: SPTPathRules,
    pub root: Utf8PathBuf,
    pub name: String, // Translated "Unknown mod" string from frontend for loose files
}

/// Takes raw user inputs and converts them into validated ModFS objects ready for installation.
/// Uses a functional pipeline to resolve inputs.
pub fn resolve(
    inputs: &[Utf8PathBuf],
    StageMaterial { root, rules, name }: &StageMaterial,
) -> Result<Vec<StagedMod>, SError> {
    // 1. Guard Clause: Collective "Loose File" Check
    // If the inputs collectively form a mod root, treat them as one unit immediately.
    if is_game_root_structure(inputs, &rules) {
        return stage_loose_files(inputs, &rules, &root, name).map(|staged| vec![staged]);
    }

    // 2. Functional Pipeline: Process individual inputs
    inputs
        .iter()
        .map(|input| {
            // Chain strategies: Try Directory -> If None, Try Archive
            process_as_directory(input, &rules, name)
                .or_else(|| process_as_archive(input, &rules, &root, name))
        })
        // Remove inputs that matched no strategy (Option::None)
        .filter_map(|res_opt| res_opt)
        // Collect into Result<Vec<_>>, returning the first Error if any occur
        .collect()
}

/// Checks if it is safe to install these mods.
pub fn any_mod_tool_running(sys: &mut System, mods_to_install: &[StagedMod]) -> Result<(), SError> {
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
    unknown_mod_name: &str,
) -> Option<Result<StagedMod, SError>> {
    if !input.is_dir() {
        return None;
    }

    // Sub-strategy A1: Folder has strict Game Root structure (user/ or BepInEx/)
    // We use boolean matching to avoid deep nesting.
    let is_game_structure = folder_matches_game_structure(input, rules).map_err(SError::from); // Propagate IO errors if they happen

    match is_game_structure {
        Ok(true) => {
            // It IS a game structure, so it MUST be a valid mod. Fail if ModFS::new fails.
            Some(ModFS::new(input, rules).map(|fs| {
                // Determine name: manifest name (highest priority) or directory name
                let name = read_manifest_name(input)
                    .unwrap_or_else(|| input.file_name().unwrap_or(unknown_mod_name).to_string());
                StagedMod {
                    fs,
                    source_path: input.clone(),
                    is_staging: false,
                    name,
                }
            }))
        }
        Ok(false) => {
            // Sub-strategy A2: Folder is a standard mod folder.
            // We try ModFS::new. If it succeeds, Good. If it fails, we treat it as "Not a mod" (None).
            ModFS::new(input, rules).ok().map(|fs| {
                // Determine name: manifest name (highest priority) or directory name
                let name = read_manifest_name(input)
                    .unwrap_or_else(|| input.file_name().unwrap_or(unknown_mod_name).to_string());
                Ok(StagedMod {
                    fs,
                    source_path: input.clone(),
                    is_staging: false,
                    name,
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
    unknown_mod_name: &str,
) -> Option<Result<StagedMod, SError>> {
    is_archive(input).then(|| stage_archive(input, rules, staging_root, unknown_mod_name))
}

// --- Internal Helpers ---

/// Reads the manifest name if a manifest exists at the mod root, otherwise returns None.
fn read_manifest_name(mod_root: &Utf8Path) -> Option<String> {
    let mod_paths = ModPaths::new(mod_root);
    ModFS::read_manifest(&mod_paths.file)
        .ok()
        .map(|manifest| manifest.name)
}

fn is_game_root_structure(inputs: &[Utf8PathBuf], rules: &SPTPathRules) -> bool {
    let roots = [
        get_root_component(&rules.server_mods),
        get_root_component(&rules.client_plugins),
    ];

    inputs.iter().any(|path| {
        path.file_name()
            .map(|name| roots.contains(&Some(name)))
            .unwrap_or(false)
    })
}

fn folder_matches_game_structure(folder: &Utf8Path, rules: &SPTPathRules) -> Result<bool, SError> {
    let roots = [
        get_root_component(&rules.server_mods),
        get_root_component(&rules.client_plugins),
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
    unknown_mod_name: &str,
) -> Result<StagedMod, SError> {
    let uuid = Uuid::new_v4().to_string();
    let dest_dir = staging_root.join(uuid);
    fs::create_dir_all(&dest_dir)?;

    for input in inputs {
        let name = input
            .file_name()
            .ok_or_else(|| SError::ParseError(format!("Unable to get file name for {input}")))?;
        FileUtils::copy_recursive(input, &dest_dir.join(name))?;
    }

    let fs = ModFS::new(&dest_dir, rules)?;

    // Determine name: manifest name (highest priority) or translated "Unknown mod" for loose files
    let name = read_manifest_name(&dest_dir).unwrap_or_else(|| unknown_mod_name.to_string());

    Ok(StagedMod {
        fs,
        source_path: dest_dir,
        is_staging: true,
        name,
    })
}

fn stage_archive(
    archive: &Utf8Path,
    rules: &SPTPathRules,
    staging_root: &Utf8Path,
    unknown_mod_name: &str,
) -> Result<StagedMod, SError> {
    let uuid = Uuid::new_v4().to_string();
    let dest_dir = staging_root.join(uuid);
    fs::create_dir_all(&dest_dir)?;

    decompression::extract(archive, &dest_dir)?;

    let fs = ModFS::new(&dest_dir, rules)?;

    // Determine name: manifest name (highest priority) or archive name without extension
    let name = read_manifest_name(&dest_dir)
        .unwrap_or_else(|| archive.file_stem().unwrap_or(unknown_mod_name).to_string());

    Ok(StagedMod {
        fs,
        source_path: dest_dir,
        is_staging: true,
        name,
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

pub fn clean_up(is_staging: bool, source_path: &Utf8Path) -> Result<(), SError> {
    if !is_staging {
        return Ok(());
    }
    debug!("clean up for {source_path}");
    remove_dir_all(source_path).map_err(Into::into)
}

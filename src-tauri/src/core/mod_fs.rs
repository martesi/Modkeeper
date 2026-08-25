use crate::models::error::SError;
use crate::models::mod_dto::ModType;
use crate::models::paths::SPTPathRules;
use crate::utils::id::hash_id;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModFS {
    pub id: String,
    pub mod_type: ModType,
    pub files: Vec<Utf8PathBuf>,
    pub executables: Vec<Utf8PathBuf>,
}

pub fn resolve_id(spt_paths: &SPTPathRules, files: &[Utf8PathBuf]) -> Result<String, SError> {
    let ids: std::collections::BTreeSet<String> = files
        .iter()
        .filter_map(|path| {
            if let Ok(rel) = path.strip_prefix(&spt_paths.server_mods) {
                return rel.components().next().map(|c| c.as_str().to_string());
            }

            if path.extension() == Some("dll")
                && let Ok(rel) = path.strip_prefix(&spt_paths.client_plugins)
            {
                return Some(rel.as_str().replace('\\', "/").to_string());
            }

            None
        })
        .collect();

    if ids.is_empty() {
        return Err(SError::UnableToDetermineModId);
    }

    let concatenated = ids.into_iter().collect::<Vec<_>>().join("").to_lowercase();
    Ok(hash_id(&concatenated))
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

fn collect_files(base: &Utf8Path) -> Result<(Vec<Utf8PathBuf>, Vec<Utf8PathBuf>), SError> {
    let mut files = Vec::new();
    let mut executables = Vec::new();

    for entry in WalkDir::new(base) {
        let entry = entry.map_err(|e| SError::IOError(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| SError::ParseError(format!("Invalid UTF-8 path: {:?}", entry.path())))?;
        let path = path.strip_prefix(base)?.to_path_buf();

        if path.extension() == Some("exe") {
            executables.push(path.clone());
        }

        files.push(path);
    }

    Ok((files, executables))
}

pub fn scan(root: &Utf8Path, spt_paths: &SPTPathRules) -> Result<ModFS, SError> {
    let (files, executables) = collect_files(root)?;

    Ok(ModFS {
        id: resolve_id(spt_paths, &files)?,
        mod_type: infer_mod_type(&files, spt_paths),
        files,
        executables,
    })
}

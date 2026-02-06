use crate::core::library::Library;
use crate::models::error::SError;
use crate::models::paths::ModPaths;

/// Reads the documentation file for a mod.
/// The documentation filename is specified in the mod's manifest.
pub fn read_documentation(library: &Library, mod_id: &str) -> Result<Option<String>, SError> {
    // Verify mod exists
    if !library.mods.contains_key(mod_id) {
        return Err(SError::ModNotFound(mod_id.to_string()));
    }

    // Get documentation filename from manifest
    let doc_filename = library
        .cache
        .manifests
        .get(mod_id)
        .and_then(|manifest| manifest.documentation.as_ref());

    if doc_filename.is_none() {
        return Ok(None);
    }

    // Build path to documentation file
    let doc_path = ModPaths::new(&library.lib_paths.mods.join(mod_id))
        .folder
        .join(doc_filename.unwrap());

    // Read and return documentation content
    std::fs::read_to_string(&doc_path)
        .map(|v| Some(v))
        .map_err(|e| SError::IOError(format!("Failed to read documentation: {}", e)))
}

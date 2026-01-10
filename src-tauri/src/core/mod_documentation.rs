use crate::core::library::Library;
use crate::models::error::SError;

/// Reads the documentation file for a mod.
/// The documentation filename is specified in the mod's manifest.
pub fn read_documentation(library: &Library, mod_id: &str) -> Result<String, SError> {
    // Verify mod exists
    if !library.mods.contains_key(mod_id) {
        return Err(SError::ModNotFound(mod_id.to_string()));
    }

    // Get documentation filename from manifest
    let doc_filename = library
        .cache
        .manifests
        .get(mod_id)
        .and_then(|manifest| manifest.documentation.as_ref())
        .ok_or_else(|| SError::ParseError("Documentation not specified in manifest".to_string()))?;

    // Build path to documentation file
    let doc_path = library.lib_paths.mods.join(mod_id).join(doc_filename);

    // Read and return documentation content
    std::fs::read_to_string(&doc_path)
        .map_err(|e| SError::IOError(format!("Failed to read documentation: {}", e)))
}

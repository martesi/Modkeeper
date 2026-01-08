use crate::core::library::Library;
use crate::models::library::LibraryDTO;
use crate::utils::icon::load_icon_as_data_uri;

/// Builds a frontend DTO with enriched data (manifests and icons).
/// This is the DTO sent to the frontend with all necessary display information.
pub fn build_frontend_dto(library: &Library) -> LibraryDTO {
    let mut dto = library.to_dto();

    for (id, m) in &mut dto.mods {
        m.manifest = library.cache.manifests.get(id).cloned();

        // Load icon data if manifest specifies an icon
        m.icon_data = m.manifest.as_ref()
            .and_then(|manifest| manifest.icon.as_ref())
            .and_then(|icon_filename| {
                let icon_path = library.lib_paths.mods.join(id).join(icon_filename);
                load_icon_as_data_uri(&icon_path)
            });
    }

    dto
}

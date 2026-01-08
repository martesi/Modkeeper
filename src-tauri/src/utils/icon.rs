use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use camino::Utf8Path;
use std::fs;

/// Loads an icon file and encodes it as a data URI string.
/// Returns None if the file doesn't exist or if encoding fails.
pub fn load_icon_as_data_uri(icon_path: &Utf8Path) -> Option<String> {
    // Read the file
    let icon_bytes = fs::read(icon_path).ok()?;

    // Detect MIME type from file extension
    let mime_type = match icon_path.extension()? {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => return None, // Unsupported format
    };

    // Encode to base64
    let base64_data = BASE64.encode(&icon_bytes);

    // Return as data URI
    Some(format!("data:{};base64,{}", mime_type, base64_data))
}

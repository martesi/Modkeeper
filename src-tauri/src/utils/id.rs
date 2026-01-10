use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// Hashes a string using Blake3 (16 bytes) and encodes it with base64url (no padding).
/// Returns a stable, compact identifier (~22 characters).
pub fn hash_id(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    let hash_bytes = hash.as_bytes();
    // Take first 16 bytes for compact output
    let truncated = &hash_bytes[..16];
    URL_SAFE_NO_PAD.encode(truncated)
}

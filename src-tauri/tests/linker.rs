use mod_keeper_lib::core::linker::{is_same_file, link, read_link_target, unlink};
use camino::Utf8Path;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_file_hard_link() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    let src = root.join("source.txt");
    let dst = root.join("target.txt");

    fs::write(&src, "hello linker").unwrap();

    // 1. Create Link
    link(&src, &dst).expect("Failed to create hard link");

    // 2. Verify content is shared
    assert_eq!(fs::read_to_string(&dst).unwrap(), "hello linker");

    // 3. Verify they are the same physical file
    assert!(is_same_file(&src, &dst));

    // 4. Verify modifying one affects the other
    fs::write(&src, "changed").unwrap();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "changed");
}

#[test]
fn test_directory_link() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    let src_dir = root.join("source_dir");
    let dst_dir = root.join("target_dir");
    let inner_file = src_dir.join("data.txt");

    fs::create_dir_all(&src_dir).unwrap();
    fs::write(&inner_file, "nested data").unwrap();

    // 1. Create Directory Link (Junction on Windows, Symlink on Unix)
    link(&src_dir, &dst_dir).expect("Failed to link directory");

    // 2. Verify visibility
    let linked_file = dst_dir.join("data.txt");
    assert!(linked_file.exists());
    assert_eq!(fs::read_to_string(linked_file).unwrap(), "nested data");

    // 3. Verify read_link_target
    let target = read_link_target(&dst_dir).expect("Failed to read link");
    // Canonicalization differences might occur, but checking ends_with is safe
    assert!(target.ends_with("source_dir"));
}

#[test]
fn test_unlink_logic() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    let src = root.join("original.txt");
    let dst = root.join("link.txt");
    fs::write(&src, "keep me").unwrap();

    link(&src, &dst).unwrap();
    assert!(dst.exists());

    // 1. Unlink the link
    unlink(&dst).expect("Failed to unlink");

    // 2. Assert link is gone but source remains
    assert!(!dst.exists());
    assert!(src.exists());
}

#[test]
fn test_link_already_exists_correctly() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    let src = root.join("src.txt");
    let dst = root.join("dst.txt");
    fs::write(&src, "test").unwrap();

    // Link first time
    link(&src, &dst).unwrap();

    // Link second time (should be idempotent / return Ok)
    let result = link(&src, &dst);
    assert!(
        result.is_ok(),
        "Subsequent link to same source should succeed"
    );
}

#[test]
fn test_collision_detection() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    let src_a = root.join("a.txt");
    let src_b = root.join("b.txt");
    let dst = root.join("collision.txt");

    fs::write(&src_a, "content a").unwrap();
    fs::write(&src_b, "content b").unwrap();

    // 1. Link A to Target
    link(&src_a, &dst).unwrap();

    // 2. Attempt to Link B to Target (Collision!)
    let result = link(&src_b, &dst);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
}

#[test]
fn test_unlink_non_existent_path() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let path = root.join("ghost.txt");

    // Should not error if path doesn't exist
    let result = unlink(&path);
    assert!(result.is_ok());
}

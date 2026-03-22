//! CLI integration tests for the flat-batch warning (#70 item 5).
//!
//! Verifies that `hunch --batch <dir>` (without `-r`) warns on stderr when
//! subdirectories contain media files being skipped, and stays silent when
//! there's nothing to warn about.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper: create a temp directory tree with media files at various depths.
///
/// ```text
/// root/
///   SubdirA/
///     Nested/
///       episode.mkv
///   SubdirB/
///     movie.mp4
///   leaf_file.avi       ← only present when `include_root_file` is true
/// ```
fn create_test_tree(include_root_file: bool) -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let root = tmp.path();

    // SubdirA with deeply nested media file
    let nested = root.join("SubdirA").join("Nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("episode.mkv"), b"").unwrap();

    // SubdirB with direct media file
    let subdir_b = root.join("SubdirB");
    fs::create_dir_all(&subdir_b).unwrap();
    fs::write(subdir_b.join("movie.mp4"), b"").unwrap();

    if include_root_file {
        fs::write(root.join("leaf_file.avi"), b"").unwrap();
    }

    tmp
}

/// Helper: create a leaf directory with only media files (no subdirs).
fn create_leaf_dir() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    fs::write(tmp.path().join("ep01.mkv"), b"").unwrap();
    fs::write(tmp.path().join("ep02.mkv"), b"").unwrap();
    tmp
}

fn hunch_cmd() -> Command {
    Command::cargo_bin("hunch").expect("binary not found")
}

#[test]
fn flat_batch_warns_when_subdirs_have_media() {
    let tree = create_test_tree(false);

    let output = hunch_cmd()
        .args(["--batch", tree.path().to_str().unwrap(), "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("hint: found media files in"),
        "expected warning on stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("subdirector"),
        "expected 'subdirectory/ies' in warning, got: {stderr}"
    );
    assert!(
        stderr.contains("-r"),
        "expected '-r' suggestion in warning, got: {stderr}"
    );
}

#[test]
fn flat_batch_with_root_files_still_warns_about_subdirs() {
    let tree = create_test_tree(true);

    let output = hunch_cmd()
        .args(["--batch", tree.path().to_str().unwrap(), "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("hint: found media files in"),
        "should still warn even when root has files: {stderr}"
    );
}

#[test]
fn flat_batch_no_warning_on_leaf_dir() {
    let leaf = create_leaf_dir();

    let output = hunch_cmd()
        .args(["--batch", leaf.path().to_str().unwrap(), "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("hint:"),
        "should NOT warn when no subdirs exist, got: {stderr}"
    );
}

#[test]
fn recursive_batch_no_warning() {
    let tree = create_test_tree(false);

    let output = hunch_cmd()
        .args(["--batch", tree.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("hint:"),
        "should NOT warn in recursive mode, got: {stderr}"
    );
}

#[test]
fn warning_includes_example_command() {
    let tree = create_test_tree(false);
    let dir_str = tree.path().to_str().unwrap();

    let output = hunch_cmd()
        .args(["--batch", dir_str, "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Warning should include an actionable example with the actual path.
    assert!(
        stderr.contains(&format!("hunch --batch {dir_str} -r -j")),
        "expected example command with actual path, got: {stderr}"
    );
}

#[test]
fn warning_pluralizes_correctly_for_single_subdir() {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let subdir = tmp.path().join("OnlyChild");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("file.mkv"), b"").unwrap();

    let output = hunch_cmd()
        .args(["--batch", tmp.path().to_str().unwrap(), "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 subdirectory being"),
        "expected singular 'subdirectory', got: {stderr}"
    );
}

#[test]
fn warning_pluralizes_correctly_for_multiple_subdirs() {
    let tree = create_test_tree(false);

    let output = hunch_cmd()
        .args(["--batch", tree.path().to_str().unwrap(), "-j"])
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("2 subdirectories being"),
        "expected plural 'subdirectories', got: {stderr}"
    );
}

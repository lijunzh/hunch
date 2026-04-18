//! CLI safety tests for the recursive directory walker.
//!
//! These tests pin the defensive guarantees added in PR-B of the v1.1.8
//! release-prep wave (security-auditor finding D4):
//!
//! 1. **Depth-bounded** — `walk_dir` stops recursing past `MAX_WALK_DEPTH`
//!    (32) and does not stack-overflow on pathologically deep trees.
//! 2. **Symlink-safe** — symlinked entries are skipped entirely, so the CLI
//!    cannot be tricked into a symlink-loop infinite recursion or into
//!    walking outside the user-supplied root via a symlink to `/`.
//!
//! These guards live in `src/main.rs` (private), so we exercise them
//! through the CLI binary rather than via direct unit tests.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Build a directory tree N levels deep, with a media file at the bottom
/// and a "control" media file at depth 1 that should always be reachable.
///
/// ```text
/// root/
///   control.mkv                             ← always reached
///   d1/d2/d3/.../dN/deep.mkv                ← reached only if N ≤ MAX
/// ```
fn create_deep_tree(depth: usize) -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let root = tmp.path();
    fs::write(root.join("control.mkv"), b"").unwrap();

    let mut path = root.to_path_buf();
    for i in 1..=depth {
        path = path.join(format!("d{i}"));
    }
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("deep.mkv"), b"").unwrap();

    tmp
}

#[test]
fn walk_dir_does_not_crash_on_deep_tree() {
    // 40 levels deep — past the MAX_WALK_DEPTH=32 guard. Must terminate
    // successfully without stack overflow or panic. The deep file may or
    // may not appear in output (it's past the cap), but the control file
    // at depth 1 must always be processed.
    let tmp = create_deep_tree(40);

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Control file must be reachable (proves the walker still works at
    // shallow depths after we added the depth guard).
    assert!(
        stdout.contains("control.mkv"),
        "control.mkv at depth 1 should always be processed; got: {stdout}",
    );
}

#[test]
fn walk_dir_processes_realistic_depth_unaffected() {
    // 6 levels deep — well within MAX_WALK_DEPTH. Both control and deep
    // files must appear. This pins that the guard does not regress
    // realistic media-library traversal.
    let tmp = create_deep_tree(6);

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("control.mkv"), "control.mkv missing");
    assert!(
        stdout.contains("deep.mkv"),
        "deep.mkv at depth 6 should be reachable; got: {stdout}",
    );
}

#[cfg(unix)]
#[test]
fn walk_dir_skips_symlink_loop() {
    // Build a symlink loop: root/loop -> root. Without the symlink guard
    // this would recurse forever and eventually exhaust the depth cap +
    // produce an unbounded number of (the same) files. With the guard,
    // the symlink entry is skipped entirely and the walker terminates
    // quickly.
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("real.mkv"), b"").unwrap();
    symlink(root, root.join("loop")).unwrap();

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", root.to_str().unwrap(), "-r", "-j"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Exactly one occurrence of real.mkv — proves we did NOT follow the
    // symlink (which would have produced loop/real.mkv,
    // loop/loop/real.mkv, etc., up to the depth cap).
    let count = stdout.matches("real.mkv").count();
    assert_eq!(
        count, 1,
        "expected exactly 1 occurrence of real.mkv (symlink not followed); got {count} in: {stdout}",
    );
}

#[cfg(unix)]
#[test]
fn walk_dir_skips_symlinked_media_file() {
    // A symlink that points to a media file outside the batch root must
    // not be followed. Defensive: the user said "scan this dir," not "scan
    // wherever symlinks point."
    use std::os::unix::fs::symlink;

    let outside = TempDir::new().unwrap();
    let target = outside.path().join("escaped.mkv");
    fs::write(&target, b"").unwrap();

    let inside = TempDir::new().unwrap();
    fs::write(inside.path().join("legit.mkv"), b"").unwrap();
    symlink(&target, inside.path().join("symlink_to_outside.mkv")).unwrap();

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", inside.path().to_str().unwrap(), "-r", "-j"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("legit.mkv"),
        "legit.mkv missing from output: {stdout}",
    );
    assert!(
        !stdout.contains("symlink_to_outside.mkv") && !stdout.contains("escaped.mkv"),
        "symlinked file should NOT have been processed; got: {stdout}",
    );
}

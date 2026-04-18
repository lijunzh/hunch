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
//! 3. **Read-error-resilient** — unreadable directories (EACCES, etc.) are
//!    silently skipped instead of crashing the entire walk (#153).
//!
//! These guards live in `src/main.rs` (private), so we exercise them
//! through the CLI binary rather than via direct unit tests.
//!
//! ## Platform coverage gap (#153 gap 4)
//!
//! All symlink tests are `#[cfg(unix)]`. Windows symlinks require the
//! `SeCreateSymbolicLinkPrivilege` (Developer Mode or admin shell), which
//! we cannot assume on the GitHub Actions Windows runner. The Windows CI
//! job therefore exercises only the depth-boundary and EACCES tests.
//!
//! If hunch ever ships Windows-specific path handling (e.g., Windows
//! junction points have different semantics than POSIX symlinks: junctions
//! to directories *do* report `is_symlink() == false` via `Metadata`, but
//! `FileType::is_symlink()` returns true), add a `#[cfg(windows)]`
//! companion test using `std::os::windows::fs::{symlink_dir, symlink_file}`
//! gated on a runtime privilege check (skip-if-no-privilege rather than
//! fail-if-no-privilege).

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

// ── #153 gap 1: off-by-one boundary at MAX_WALK_DEPTH (32) ────────────────

#[test]
fn walk_dir_reaches_exactly_max_depth_minus_one() {
    // Depth 31 = MAX_WALK_DEPTH - 1 = the last reachable level. The deep
    // file MUST appear in output. Pins the comparison as `>=` against a
    // future off-by-one regression that switches it to `>`.
    let tmp = create_deep_tree(31);

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("deep.mkv"),
        "deep.mkv at depth 31 must be reachable (MAX_WALK_DEPTH - 1); got: {stdout}",
    );
}

#[test]
fn walk_dir_stops_at_exactly_max_depth() {
    // Depth 32 = exactly MAX_WALK_DEPTH = the first UNREACHABLE level.
    // Per the `if depth >= MAX_WALK_DEPTH { return }` guard, the deep
    // file at this depth must NOT appear. Pins the off-by-one boundary
    // from the other direction.
    let tmp = create_deep_tree(32);

    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Control file must still be reachable (proves the walker survived
    // the depth-cap branch without bailing too early).
    assert!(
        stdout.contains("control.mkv"),
        "control.mkv at depth 1 should still be processed; got: {stdout}",
    );
    assert!(
        !stdout.contains("deep.mkv"),
        "deep.mkv at depth 32 = MAX_WALK_DEPTH must NOT be reached; got: {stdout}",
    );
}

// ── #153 gap 2: dir_contains_media exercised by symlink + depth ───────────

#[cfg(unix)]
#[test]
fn dir_contains_media_warning_path_skips_symlink_loop() {
    // `--batch <root>` (without `-r`) triggers warn_if_subdirs_have_media,
    // which calls dir_contains_media on every subdir. Without the symlink
    // skip in dir_contains_media_inner, a subdir containing a symlink
    // loop would hang the warning path. With the guard, it terminates.
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("topfile.mkv"), b"").unwrap();

    // Subdir with a self-loop. dir_contains_media must NOT recurse into
    // the loop and must terminate quickly.
    let subdir = root.join("looped_subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("real.mkv"), b"").unwrap();
    symlink(&subdir, subdir.join("loop")).unwrap();

    // Run without `-r` so warn_if_subdirs_have_media (and thus
    // dir_contains_media) is the code path under test.
    let assert = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", root.to_str().unwrap(), "-j"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success();

    // The top-level file must be processed (proves the walk completed).
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("topfile.mkv"),
        "topfile.mkv must be processed; got: {stdout}",
    );
}

// ── #153 gap 3: EACCES / read_dir → Err handled silently ──────────────────

#[cfg(unix)]
#[test]
fn walk_dir_survives_unreadable_subdir() {
    // Pin the `let Ok(entries) = read_dir(dir) else { return }` arm by
    // creating a chmod-000 subdir alongside readable content. The walker
    // must skip the unreadable dir silently and yield the rest. Catches
    // a future change that switches the silent-skip to `unwrap()` /
    // `expect()` / `panic!()`.
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join("readable.mkv"), b"").unwrap();

    let unreadable = root.join("locked");
    fs::create_dir(&unreadable).unwrap();
    fs::write(unreadable.join("hidden.mkv"), b"").unwrap();
    // 0o000 = no perms; read_dir on this dir will return EACCES.
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();

    let result = Command::cargo_bin("hunch")
        .unwrap()
        .args(["--batch", root.to_str().unwrap(), "-r", "-j"])
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success();

    // Restore perms BEFORE assertions so TempDir cleanup works even if
    // the asserts panic.
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o755)).unwrap();

    let stdout = String::from_utf8(result.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("readable.mkv"),
        "readable.mkv must still be processed despite the unreadable sibling; got: {stdout}",
    );
    assert!(
        !stdout.contains("hidden.mkv"),
        "hidden.mkv inside the EACCES dir must NOT be processed; got: {stdout}",
    );
}

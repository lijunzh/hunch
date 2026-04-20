//! Issue #209 regression: `list_media_files` (the non-recursive directory
//! listing used by both `--context` mode AND `hunch --batch <dir>` without
//! `-r`) must skip symlinks for parity with `walk_dir_inner` (used by
//! `--batch -r`).
//!
//! Previously `list_media_files` used `Path::is_file()` which follows
//! symlinks, allowing an attacker who controls files inside the directory
//! the user explicitly chose to scan to inject crafted basenames into the
//! parser via symlinks pointing outside the directory. Hunch only reads
//! basenames (not file contents), so the impact was low — but matching
//! `walk_dir`'s hardening keeps the defense story consistent across both
//! CLI entry points.
//!
//! Symlink creation requires elevated privileges on Windows
//! (`SeCreateSymbolicLinkPrivilege`); this test is therefore `#[cfg(unix)]`,
//! mirroring the convention in `cli_walk_dir_safety.rs`.

#![cfg(unix)]

use assert_cmd::Command;
use std::fs;
use std::os::unix::fs::symlink;
use tempfile::TempDir;

/// We exercise `list_media_files` through `--batch <dir>` (without `-r`)
/// because that path produces a deterministic one-output-line-per-file
/// signal we can count, making "was the symlink processed?" a sharp test.
/// `--context` mode uses the same function but funnels the result through
/// invariance (which needs same-name siblings to surface anything visible)
/// so observable behavior would be muddier there.
#[test]
fn issue_209_batch_flat_skips_symlinks() {
    let tmp = TempDir::new().expect("temp dir");
    let outside = tmp.path().join("outside");
    let scan = tmp.path().join("scan");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&scan).unwrap();

    // Bait file lives outside the scan directory entirely — without the
    // symlink-skip guard, its basename would be injected into the parser
    // input despite the user never choosing it.
    let bait = outside.join("bait.Movie.2024.1080p.mkv");
    fs::write(&bait, b"").unwrap();

    // One real file in the scan directory.
    fs::write(scan.join("Show.S01E01.mkv"), b"").unwrap();

    // Symlink injecting an attacker-controlled basename into the scan dir.
    symlink(&bait, scan.join("linked.Other.S99E99.mkv")).expect("symlink");

    let output = Command::cargo_bin("hunch")
        .expect("hunch binary")
        .args(["--batch", scan.to_str().unwrap(), "-j"])
        .output()
        .expect("hunch ran");
    assert!(
        output.status.success(),
        "hunch failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| l.contains("_filename")).collect();

    // With the fix: exactly 1 result (Show.S01E01), the symlink is skipped.
    // Without the fix: 2 results (Show + the symlink-injected bait basename).
    assert_eq!(
        lines.len(),
        1,
        "expected exactly 1 file (the real one); the symlink should be \
         skipped. Got {} lines:\n{stdout}",
        lines.len(),
    );
    assert!(
        lines[0].contains("Show.S01E01.mkv"),
        "the surviving file should be Show.S01E01.mkv, got: {}",
        lines[0],
    );

    // Belt-and-suspenders: the symlink's basename must not appear anywhere
    // in stdout. Catches any future code path that might surface it
    // through siblings/context even if the file count check passed.
    assert!(
        !stdout.contains("linked.Other.S99E99.mkv"),
        "symlinked entry should be invisible. Got: {stdout}",
    );
}

/// Companion check: real (non-symlink) files in the same directory still
/// ARE seen by `--batch`. Guards against an over-broad fix that
/// accidentally drops legitimate files.
#[test]
fn issue_209_batch_flat_still_sees_real_files() {
    let tmp = TempDir::new().expect("temp dir");
    let scan = tmp.path();

    fs::write(scan.join("Show.S01E01.mkv"), b"").unwrap();
    fs::write(scan.join("Show.S01E02.mkv"), b"").unwrap();
    fs::write(scan.join("Show.S01E03.mkv"), b"").unwrap();

    let output = Command::cargo_bin("hunch")
        .expect("hunch binary")
        .args(["--batch", scan.to_str().unwrap(), "-j"])
        .output()
        .expect("hunch ran");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.lines().filter(|l| l.contains("_filename")).count();
    assert_eq!(count, 3, "expected 3 real files. Got:\n{stdout}");
}

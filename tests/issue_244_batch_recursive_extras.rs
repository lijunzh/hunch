//! CLI regression tests for issue #244 bugs #2 and #3:
//!
//! - **#2**: parent-directory name leaks as `title` in `--batch -r` mode
//!   when the deepest dir is a structural anime-extras grouping
//!   (`PV/`, `menu/`, `NCOP&NCED/`).
//! - **#3**: `.ass` (and other external subtitle) files silently dropped
//!   by `--batch -r` because `MEDIA_EXTENSIONS` was video-only.
//!
//! Both fixes live in low-blast-radius places:
//!   - #2 → `is_generic_dir` allow-list (`src/properties/title/clean.rs`)
//!   - #3 → `MEDIA_EXTENSIONS` constant (`src/main.rs`)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn hunch_cmd() -> Command {
    Command::cargo_bin("hunch").expect("binary not found")
}

fn parse_lines(stdout: &str) -> Vec<serde_json::Value> {
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("each line should be valid JSON"))
        .collect()
}

// ── Bug #2: structural anime-extras dirs no longer leak as title ──────

#[test]
fn batch_recursive_does_not_leak_pv_dir_as_title() {
    let tmp = TempDir::new().unwrap();
    let pv = tmp.path().join("PV");
    fs::create_dir_all(&pv).unwrap();
    fs::write(
        pv.join("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][PV][01][1080P][BDRip].mkv"),
        b"",
    )
    .unwrap();

    let out = hunch_cmd()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let results = parse_lines(&stdout);

    assert_eq!(results.len(), 1, "expected 1 file, got: {stdout}");
    let title = results[0].get("title").and_then(|t| t.as_str());
    // The fix: "PV" is now in is_generic_dir so parent_dir walks past it
    // and finds the title in the filename's bracket. Title MUST NOT be "PV".
    assert_ne!(
        title,
        Some("PV"),
        "PV directory name leaked as title (bug #2)"
    );
    // Bonus: with the bug-#1 fix, we get the actual title from the bracket.
    assert_eq!(title, Some("Re Zero kara Hajimeru Isekai Seikatsu"));
}

#[test]
fn batch_recursive_does_not_leak_menu_dir_as_title() {
    let tmp = TempDir::new().unwrap();
    let menu = tmp.path().join("menu");
    fs::create_dir_all(&menu).unwrap();
    fs::write(
        menu.join(
            "[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][menu][01][1080P][BDRip].mkv",
        ),
        b"",
    )
    .unwrap();

    let out = hunch_cmd()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .unwrap();
    let results = parse_lines(&String::from_utf8_lossy(&out.stdout));

    let title = results[0].get("title").and_then(|t| t.as_str());
    assert_ne!(title, Some("menu"), "menu/ leaked as title (bug #2)");
}

#[test]
fn batch_recursive_does_not_leak_ncop_nced_dir_as_title() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("NCOP&NCED");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][NCED1][1080P][BDRip].mkv"),
        b"",
    )
    .unwrap();

    let out = hunch_cmd()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .unwrap();
    let results = parse_lines(&String::from_utf8_lossy(&out.stdout));

    let title = results[0].get("title").and_then(|t| t.as_str());
    assert_ne!(
        title,
        Some("NCOP&NCED"),
        "NCOP&NCED/ leaked as title (bug #2)"
    );
}

// ── Bug #3: external subtitle files included in --batch -r output ─────

#[test]
fn batch_recursive_includes_ass_subtitle_files() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("Show");
    fs::create_dir_all(&dir).unwrap();

    // Mix of video and subtitle files \u2014 both should show up in --batch -r.
    fs::write(dir.join("[Group][Show S2][01][1080P].mkv"), b"").unwrap();
    fs::write(dir.join("[Group][Show S2][01][1080P].sc.ass"), b"").unwrap();
    fs::write(dir.join("[Group][Show S2][01][1080P].tc.ass"), b"").unwrap();
    fs::write(dir.join("[Group][Show S2][01].srt"), b"").unwrap();

    let out = hunch_cmd()
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .unwrap();
    let results = parse_lines(&String::from_utf8_lossy(&out.stdout));

    // Pre-fix: only the .mkv was emitted (1 result). Post-fix: all 4.
    assert_eq!(
        results.len(),
        4,
        "expected video + 3 subtitle files, got {} entries",
        results.len()
    );

    let containers: Vec<_> = results
        .iter()
        .filter_map(|r| r.get("container").and_then(|c| c.as_str()))
        .collect();
    assert!(containers.contains(&"mkv"));
    assert!(containers.contains(&"ass"));
    assert!(containers.contains(&"srt"));
}

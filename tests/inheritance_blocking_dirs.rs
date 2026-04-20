//! Issue #208 regression: extras/specials/bonus subdirectories should
//! block ancestor-title inheritance, not just sample/subs.
//!
//! The original `is_sample_dir` predicate only suppressed the
//! parent-title fallback for `sample/samples/subs/subtitles/featurettes`.
//! Renamed in this fix to `is_inheritance_blocking_dir` and extended to
//! cover `extras/extra/specials/bonus`.
//!
//! Without the fix, a batch tree like:
//!
//! ```text
//! tv/
//!   Movie.2024.mkv
//!   Show/Extras/Bonus.Featurette.mkv
//! ```
//!
//! would cause `Bonus.Featurette.mkv` to incorrectly inherit "Movie" via
//! the ancestor cache built up at the `tv/` level.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Build the #208 reproducer tree. Returns the temp dir (kept alive
/// until drop).
fn build_extras_inheritance_tree() -> TempDir {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path();

    // Sibling movie file at the batch root \u2014 this is the title that
    // would (wrongly) leak into the Extras subtree without the fix.
    fs::write(root.join("Movie.2024.mkv"), b"").unwrap();

    // The Extras directory under a Show, holding a single bonus file.
    let extras = root.join("Show").join("Extras");
    fs::create_dir_all(&extras).unwrap();
    fs::write(extras.join("Bonus.Featurette.mkv"), b"").unwrap();

    tmp
}

/// The headline regression: `--batch -r` over the tree above must NOT
/// stamp "Movie" onto `Bonus.Featurette.mkv`.
#[test]
fn issue_208_extras_dir_blocks_ancestor_title_inheritance() {
    let tmp = build_extras_inheritance_tree();

    let output = Command::cargo_bin("hunch")
        .expect("hunch binary")
        .args(["--batch", tmp.path().to_str().unwrap(), "-r", "-j"])
        .output()
        .expect("hunch ran");

    assert!(
        output.status.success(),
        "hunch exited with: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find the Bonus.Featurette.mkv line. It must not carry "Movie" as
    // its title via the polluted ancestor cache.
    let bonus_line = stdout
        .lines()
        .find(|l| l.contains("Bonus.Featurette.mkv"))
        .unwrap_or_else(|| panic!("Bonus.Featurette.mkv missing from output:\n{stdout}"));

    assert!(
        !bonus_line.contains("\"title\":\"Movie\""),
        "Bonus.Featurette.mkv must not inherit 'Movie' title from sibling \
         batch entries. Got: {bonus_line}",
    );
}

/// Companion check: make sure all four new vocabulary words actually
/// block inheritance. We use one small tree per word to keep failures
/// pinpointable to the exact directory name that regressed.
#[test]
fn issue_208_all_extras_synonyms_block_inheritance() {
    for synonym in ["Extras", "Extra", "Specials", "Bonus"] {
        let tmp = TempDir::new().expect("temp dir");
        let root = tmp.path();

        fs::write(root.join("Movie.2024.mkv"), b"").unwrap();
        let sub = root.join("Show").join(synonym);
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("Aux.Clip.mkv"), b"").unwrap();

        let output = Command::cargo_bin("hunch")
            .expect("hunch binary")
            .args(["--batch", root.to_str().unwrap(), "-r", "-j"])
            .output()
            .expect("hunch ran");
        assert!(
            output.status.success(),
            "[{synonym}] hunch failed: {}",
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let aux_line = stdout
            .lines()
            .find(|l| l.contains("Aux.Clip.mkv"))
            .unwrap_or_else(|| panic!("[{synonym}] Aux.Clip.mkv missing from:\n{stdout}"));

        assert!(
            !aux_line.contains("\"title\":\"Movie\""),
            "[{synonym}] should block ancestor-title inheritance. Got: {aux_line}",
        );
    }
}

/// Don't regress #97: the original sample/subs vocabulary must still
/// block inheritance. Lowercase + uppercase casing both honored.
#[test]
fn issue_97_sample_synonyms_still_block_inheritance() {
    for synonym in ["Sample", "samples", "Subs", "Subtitles", "Featurettes"] {
        let tmp = TempDir::new().expect("temp dir");
        let root = tmp.path();

        fs::write(root.join("Movie.2024.mkv"), b"").unwrap();
        let sub = root.join("Show").join(synonym);
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("Clip.mkv"), b"").unwrap();

        let output = Command::cargo_bin("hunch")
            .expect("hunch binary")
            .args(["--batch", root.to_str().unwrap(), "-r", "-j"])
            .output()
            .expect("hunch ran");
        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let clip_line = stdout
            .lines()
            .find(|l| l.contains("Clip.mkv"))
            .unwrap_or_else(|| panic!("[{synonym}] Clip.mkv missing"));

        assert!(
            !clip_line.contains("\"title\":\"Movie\""),
            "[{synonym}] should still block inheritance (#97). Got: {clip_line}",
        );
    }
}

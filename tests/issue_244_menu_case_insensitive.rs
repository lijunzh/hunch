//! Regression tests for issue #244 bug #5: `[menu]` not recognized as
//! an episode_details marker due to case-sensitive matching.
//!
//! Symptom (v2.0.1):
//!   `[menu]`  → episode_details: missing  ❌ (no rule fires)
//!   `[Menu]`  → episode_details: missing  ❌
//!   `[MENU]`  → episode_details: "Menu"   ✅ (only this case worked)
//!
//! In real-world anime BD releases, `[menu]` (lowercase) is the convention
//! used by DBD-Raws and others. The case-sensitive `MENU` entry in
//! `[exact_sensitive]` only matched the all-caps form.
//!
//! Fix: move `MENU` → `menu` in `[exact]` (case-insensitive) in
//! `src/rules/anime_bonus.toml`. The justification for case-sensitive
//! matching of OP/ED/SP ("avoid `Op` as a name colliding with `OP` for
//! Opening") doesn't apply to "menu" — it's rare as a title word and
//! the anime-extras semantics dominate.

use hunch::hunch;

fn details(input: &str) -> Option<String> {
    hunch(input)
        .to_flat_map()
        .get("episode_details")
        .and_then(|v| v.as_str())
        .map(String::from)
}

#[test]
fn menu_lowercase() {
    // The real-world DBD-Raws form.
    assert_eq!(
        details("[DBD-Raws][Re Zero S2][menu][01][1080P][BDRip].mkv"),
        Some("Menu".into())
    );
}

#[test]
fn menu_titlecase() {
    assert_eq!(
        details("[Group][Show S2][Menu][01][1080P].mkv"),
        Some("Menu".into())
    );
}

#[test]
fn menu_uppercase() {
    // Pre-existing behavior preserved.
    assert_eq!(
        details("[Group][Show S2][MENU][01][1080P].mkv"),
        Some("Menu".into())
    );
}

//! Fallback title-extraction strategies.
//!
//! When the primary positional extraction in [`super::extract_title`] yields
//! an empty cleaned string, control passes to the **fallback ladder** defined
//! here. Each strategy is a small implementation of [`TitleStrategy`]; they
//! are tried in [`FALLBACK_STRATEGIES`] order and the first non-`None` answer
//! wins.
//!
//! ## Why a trait?
//!
//! Before this module, the ladder was an inline if-let chain inside
//! `extract_title` calling four free functions with bespoke signatures.
//! Each function duplicated the "scan the filename, pick a byte range,
//! clean it, build a [`MatchSpan`]" skeleton. That worked at four; D10
//! (the post-#127 architecture-review tripwire) flagged a 6th strategy as
//! the threshold for refactor-first. Trait + registry hits that threshold
//! preemptively: adding a 5th strategy is now appending one line to
//! [`FALLBACK_STRATEGIES`], not editing the orchestrator.
//!
//! ## How to add a new strategy
//!
//! 1. Create `strategies/your_thing.rs` with a unit struct implementing
//!    [`TitleStrategy`].
//! 2. `mod your_thing;` here.
//! 3. Add `&your_thing::YourThing` to [`FALLBACK_STRATEGIES`] at the
//!    correct ordinal (the ladder is tried in order, so place it where
//!    it should win against existing strategies).
//!
//! That's the entire surface area.

use crate::matcher::span::MatchSpan;
use log::trace;

mod after_bracket_group;
mod cjk_bracket;
mod parent_dir;
mod unclaimed_bracket;

// Strategy structs are re-exported for the rare callers in `super::mod` that
// need to invoke a SPECIFIC strategy in isolation (the parent-dir casing
// fallback in the main path, and the empty-title-zone recovery). Adding
// the strategy to the ladder is the common case; ad-hoc invocation is the
// exception.
pub(super) use after_bracket_group::AfterBracketGroup;
pub(super) use cjk_bracket::CjkBracket;
pub(super) use parent_dir::ParentDir;
pub(super) use unclaimed_bracket::UnclaimedBracket;

/// Inputs every strategy needs. Bundled into a struct so adding a new
/// piece of context (e.g. `zone_map`) is a one-line change to every
/// strategy signature \u2014 not N.
pub(super) struct StrategyContext<'a> {
    pub input: &'a str,
    pub matches: &'a [MatchSpan],
    pub filename_start: usize,
}

/// One fallback title extractor.
///
/// Strategies are stateless unit structs; behavior lives entirely in
/// [`try_extract`](Self::try_extract).
pub(super) trait TitleStrategy: Sync {
    /// Short, debug-friendly identifier (e.g. `"cjk_bracket"`). Used in
    /// trace logs to explain *which* strategy claimed the title.
    fn name(&self) -> &'static str;

    /// Try to produce a title match. Return `None` if the strategy does
    /// not apply to this input (the next strategy in the ladder is then
    /// tried).
    fn try_extract(&self, ctx: &StrategyContext<'_>) -> Option<MatchSpan>;
}

/// The fallback ladder, in priority order.
///
/// Order rationale (do not shuffle without thought):
///
/// 1. **CjkBracket** \u2014 most specific (requires `[Group][Title][Ep]` shape +
///    an Episode match). Cheap to reject when it doesn't apply.
/// 2. **AfterBracketGroup** \u2014 anime `[Group] Title - Ep [tags]`. Runs
///    before the all-bracket fallback because some files satisfy both
///    patterns and this one is more accurate when applicable.
/// 3. **UnclaimedBracket** \u2014 broader all-bracket fallback for files like
///    `[a][b][title][c][d].mkv` where one bracket isn't claimed by any
///    matcher.
/// 4. **ParentDir** \u2014 last resort: walk up the directory tree.
pub(super) static FALLBACK_STRATEGIES: &[&dyn TitleStrategy] = &[
    &CjkBracket,
    &AfterBracketGroup,
    &UnclaimedBracket,
    &ParentDir,
];

/// Run the ladder; return the first hit.
pub(super) fn run_fallback_ladder(ctx: &StrategyContext<'_>) -> Option<MatchSpan> {
    for strategy in FALLBACK_STRATEGIES {
        if let Some(title) = strategy.try_extract(ctx) {
            trace!(
                "title fallback: {} claimed {:?} at {}..{}",
                strategy.name(),
                title.value,
                title.start,
                title.end
            );
            return Some(title);
        }
    }
    None
}

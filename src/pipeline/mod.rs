//! Pipeline v0.2.1: tokenize → zones → match → disambiguate → title → result.
//!
//! The pipeline tokenizes the input, builds zone boundaries, then matches
//! tokens against TOML rules and legacy matchers. Zone-aware disambiguation
//! replaces v0.1 prune_* heuristics.

pub(crate) mod context;
mod invariance;
mod matching;
mod pass2_helpers;
mod proper_count;
mod rule_registry;
pub(crate) mod token_context;
mod zone_rules;

use crate::hunch_result::HunchResult;
use crate::matcher::engine;
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::{self, TokenStream};
use crate::zone_map::{self, TitleYear, ZoneMap};
use matching::MatchContext;
use rule_registry::{LegacyMatcherFn, SegmentScope, TomlRule};

/// Returns true if a `[start, end)` byte range overlaps any of the
/// `title_years` ranges.
///
/// Hoisted out of `Pipeline::pass1` so the boundary semantics can be
/// pinned by unit tests directly. Uses **half-open interval** logic
/// (`m.start < ty.end && m.end > ty.start`):
///
/// - Touching ranges do NOT overlap (`m.end == ty.start` is NOT overlap;
///   `m.start == ty.end` is NOT overlap). This matches Rust's `Range<usize>`
///   convention used everywhere else in the matcher.
/// - The match must have at least one byte inside `[ty.start, ty.end)`
///   for the predicate to return true.
/// - An empty `title_years` slice always returns false (vacuous "any").
///
/// Used by year disambiguation: when the title contains year-like numbers
/// (e.g., "Blade Runner 2049"), those byte ranges are recorded as
/// `title_years` so we don't ALSO extract them as the release year.
pub(super) fn match_overlaps_any_title_year(
    match_start: usize,
    match_end: usize,
    title_years: &[TitleYear],
) -> bool {
    title_years
        .iter()
        .any(|ty| match_start < ty.end && match_end > ty.start)
}

use log::{debug, trace};

use crate::priority;
use crate::properties::part;
use crate::properties::release_group;
use crate::properties::title;
use crate::properties::title::{TitleConfidence, TitleExtraction};

/// A title supplied to [`Pipeline::pass2`] from outside the file itself.
///
/// Two cross-file sources can produce a hint:
///   - **Invariance**: sibling consensus on filename content
///     (`source = "invariance"`, `position = Some(byte_offset)` from the
///     [`InvarianceReport`](invariance::InvarianceReport)).
///   - **Ancestor fallback**: a title cached from a parent directory
///     during batch ‑r traversal (`source = "fallback"`, `position = None`
///     because the text usually doesn't appear verbatim in the input).
///
/// The pipeline disambiguates between the two before constructing the
/// hint (invariance preferred). Pass2 then weighs this hint against the
/// file's own extraction via [`pick_final_title`].
///
/// Replaces the previous `(title_override, last_resort_title)` pair and
/// the `filename_has_bracket` / `is_path_dir_name` ad-hoc heuristics.
struct TitleHint {
    /// The title text.
    value: String,
    /// Byte offset of `value` in `input`, when known.
    position: Option<usize>,
    /// Where the hint came from (debug/trace logs only).
    source: &'static str,
}

/// The two-pass parsing pipeline.
///
/// Orchestrates the full parsing flow: tokenization → zone mapping
/// → TOML + legacy matching → conflict resolution → zone disambiguation
/// → release group / title extraction → result assembly.
///
/// See [`Pipeline::run`] for the main entry point, or use
/// [`hunch`](crate::hunch) / [`hunch_with_context`](crate::hunch_with_context)
/// for convenience.
#[must_use = "a Pipeline is only useful when you call `.run()` / `.run_with_context()` on it"]
pub struct Pipeline {
    /// TOML-driven rule sets registered in [`rule_registry::build_toml_rules`].
    toml_rules: Vec<TomlRule>,
    /// Legacy matchers registered in [`rule_registry::build_legacy_matchers`].
    legacy_matchers: Vec<LegacyMatcherFn>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    /// Create a new pipeline.
    ///
    /// Prefer [`hunch`](crate::hunch) for one-shot parsing.
    /// Construct a `Pipeline` directly when you want to reuse the same
    /// configuration across many inputs.
    pub fn new() -> Self {
        Self {
            toml_rules: rule_registry::build_toml_rules(),
            legacy_matchers: rule_registry::build_legacy_matchers(),
        }
    }

    /// Run the full pipeline on an input string.
    ///
    /// ## Two-pass architecture (v0.3)
    ///
    /// **Pass 1**: Tech property resolution — TOML rules + legacy matchers
    /// (everything except release_group). Conflict resolution + zone
    /// disambiguation produces `resolved_tech_matches`.
    ///
    /// **Pass 2**: Positional property extraction — release_group uses
    /// resolved match positions (no more `is_known_token` exclusion list).
    /// Title, episode_title, alternative_title use all resolved matches.
    pub fn run(&self, input: &str) -> HunchResult {
        let (mut matches, token_stream, zone_map) = self.pass1(input);
        self.pass2(input, &mut matches, &zone_map, &token_stream, None, None)
    }

    /// Parse a filename using sibling filenames for cross-file title detection.
    ///
    /// Both `input` and `siblings` can include directory path components
    /// (e.g., `"Show Name/Season 1/S01E03.720p.mkv"`). When paths are
    /// provided, `extract_title_from_parent` uses them for title fallback
    /// (walking parent directories, skipping generic names like "Season 1").
    ///
    /// Siblings should be files from the **same directory** as the target.
    /// Even 1–2 siblings can dramatically improve title extraction for CJK
    /// and non-standard formats.
    ///
    /// Accepts any slice of string-like types (`&[&str]`, `&[String]`, etc.).
    /// Even 1–2 siblings can dramatically improve title extraction for CJK
    /// and non-standard formats.
    ///
    /// Cross-file analysis produces an `InvarianceReport` that informs:
    /// - **Title**: invariant text across files
    /// - **Year signals**: year-like numbers classified as title vs metadata
    /// - **Episode signals**: sequential variant numbers as episode evidence
    ///
    /// # Example
    ///
    /// ```rust
    /// use hunch::Pipeline;
    ///
    /// let pipeline = Pipeline::new();
    /// let result = pipeline.run_with_context(
    ///     "Show.S01E03.720p.mkv",
    ///     &["Show.S01E01.720p.mkv", "Show.S01E02.720p.mkv"],
    /// );
    /// assert_eq!(result.title(), Some("Show"));
    /// ```
    ///
    /// # Example with paths
    ///
    /// ```rust
    /// use hunch::Pipeline;
    ///
    /// let pipeline = Pipeline::new();
    /// let result = pipeline.run_with_context(
    ///     "Paw Patrol/S01E10 - Pups Save Ryder's Robot.mkv",
    ///     &["Paw Patrol/S01E11 - Pups and the Ghost Pirate.mkv"],
    /// );
    /// assert_eq!(result.title(), Some("Paw Patrol"));
    /// ```
    pub fn run_with_context<S: AsRef<str>>(&self, input: &str, siblings: &[S]) -> HunchResult {
        let sibs: Vec<&str> = siblings.iter().map(|s| s.as_ref()).collect();
        self.run_with_context_inner(input, &sibs)
    }

    /// Inner implementation with concrete `&[&str]` to avoid monomorphization bloat.
    fn run_with_context_inner(&self, input: &str, siblings: &[&str]) -> HunchResult {
        self.run_with_context_and_fallback_inner(input, siblings, None)
    }

    /// Parse with sibling context and an optional fallback title.
    ///
    /// When `fallback_title` is `Some(...)`, it is used as the title hint
    /// if and only if the invariance analysis does not produce one. This
    /// allows parent directory context to propagate to child directories
    /// (e.g., `Extras/`, `SP/`) that have too few files for independent
    /// invariance detection.
    ///
    /// The fallback informs but does not force: if the child directory
    /// has strong invariance of its own, it wins.
    pub fn run_with_context_and_fallback<S: AsRef<str>>(
        &self,
        input: &str,
        siblings: &[S],
        fallback_title: Option<&str>,
    ) -> HunchResult {
        let sibs: Vec<&str> = siblings.iter().map(|s| s.as_ref()).collect();
        self.run_with_context_and_fallback_inner(input, &sibs, fallback_title)
    }

    /// Inner implementation for context + fallback.
    fn run_with_context_and_fallback_inner(
        &self,
        input: &str,
        siblings: &[&str],
        fallback_title: Option<&str>,
    ) -> HunchResult {
        if siblings.is_empty() && fallback_title.is_none() {
            return self.run(input);
        }

        // No siblings but a fallback title: skip invariance, run pass1
        // and let pass2 weigh the fallback against the file's own
        // extraction by confidence.
        if siblings.is_empty() {
            let (mut matches, ts, zm) = self.pass1(input);
            let hint = fallback_title.map(|fb| TitleHint {
                value: fb.to_string(),
                position: None,
                source: "fallback",
            });
            return self.pass2(input, &mut matches, &zm, &ts, hint, None);
        }

        // 1. Run Pass 1 on target + all siblings.
        let (target_matches, target_ts, target_zm) = self.pass1(input);
        let sibling_results: Vec<_> = siblings.iter().map(|s| self.pass1(s)).collect();

        // 2. Unified invariance analysis (title + year + episode signals).
        //    Invariance gaps are filename-scoped (see
        //    `find_unclaimed_gaps`), so any title produced here came
        //    from actual filename content shared across siblings — not
        //    from path-prefix accidents.
        let sibling_analyses: Vec<_> = siblings
            .iter()
            .zip(&sibling_results)
            .map(|(s, (matches, _, _))| invariance::FileAnalysis {
                input: s,
                matches: matches.as_slice(),
            })
            .collect();
        let report = invariance::analyze_invariance(
            &invariance::FileAnalysis {
                input,
                matches: &target_matches,
            },
            &sibling_analyses,
        );

        debug!(
            "cross-file context: {} sibling(s), title={:?}, {} year signal(s), {} episode signal(s)",
            siblings.len(),
            report.title,
            report.year_signals.len(),
            report.episode_signals.len(),
        );
        for ys in &report.year_signals {
            trace!(
                "  [YEAR] {} at {}..{} invariant={}",
                ys.value, ys.start, ys.end, ys.is_invariant
            );
        }
        for es in &report.episode_signals {
            trace!(
                "  [EPISODE] {} at {}..{} sequential={} digits={}",
                es.value, es.start, es.end, es.is_sequential, es.digit_count
            );
        }

        // 3. Pick the title hint to pass to pass2.
        //
        //    Two cross-file signals exist; both can potentially override
        //    a Weak extraction. Invariance (sibling consensus on actual
        //    filename content) is more authoritative than ancestor
        //    fallback (a parent-dir-derived guess), so it takes priority
        //    when both are present.
        //
        //    Pass2 then resolves the final title by comparing this hint
        //    against the file's own extraction confidence (see
        //    [`pick_final_title`]). The pipeline never overrides Strong
        //    extractions — a self-describing file always wins, because
        //    second-guessing explicit author markup is exactly the bug
        //    `filename_has_bracket` was patching around.
        let title_hint = match (report.title.as_ref(), fallback_title) {
            (Some(inv), _) => Some(TitleHint {
                value: inv.clone(),
                position: report.title_start,
                source: "invariance",
            }),
            (None, Some(fb)) => Some(TitleHint {
                value: fb.to_string(),
                position: None,
                source: "fallback",
            }),
            (None, None) => None,
        };

        let mut matches = target_matches;
        self.pass2(
            input,
            &mut matches,
            &target_zm,
            &target_ts,
            title_hint,
            Some(&report),
        )
    }

    /// Run Pass 1: tokenize → zone map → match → conflict resolve → zone disambiguate.
    ///
    /// Returns the resolved tech matches, token stream, and zone map.
    /// This is the reusable core that `run_with_context()` calls on both
    /// the target file and each sibling.
    fn pass1(&self, input: &str) -> (Vec<MatchSpan>, TokenStream, ZoneMap) {
        // Step 1: Tokenize.
        let token_stream = tokenizer::tokenize(input);
        debug!(
            "step 1: tokenized into {} segment(s), {} total token(s)",
            token_stream.segments.len(),
            token_stream
                .segments
                .iter()
                .map(|s| s.tokens.len())
                .sum::<usize>()
        );

        // Step 1b: Build zone map (anchor detection + year disambiguation).
        let zone_map = zone_map::build_zone_map(input, &token_stream);
        debug!(
            "step 1b: zone map — has_anchors={}, title_zone={}..{}, year={:?}",
            zone_map.has_anchors,
            zone_map.title_zone.start,
            zone_map.title_zone.end,
            zone_map.year.as_ref().map(|y| y.value)
        );

        // Step 2: Match — TOML rules against tokens + legacy matchers against raw input.
        // NOTE: release_group is NOT included here — it runs in Pass 2.
        let mut all_matches = self.match_all(input, &token_stream, &zone_map);
        debug!(
            "step 2: matching produced {} raw match(es)",
            all_matches.len()
        );
        for m in &all_matches {
            trace!(
                "  raw match: {:?}={} at {}..{} (pri={})",
                m.property, m.value, m.start, m.end, m.priority
            );
        }

        // Step 2b: Year disambiguation using ZoneMap.
        //
        // The `match_overlaps_any_title_year` helper returns false for an
        // empty `title_years` slice, so we don't need a separate
        // `!is_empty()` guard — the retain becomes a no-op for year
        // matches when there's nothing to compare against. Removing the
        // guard also eliminates a mutation hot spot (no `!` to delete).
        if let Some(ref yi) = zone_map.year {
            all_matches.retain(|m| {
                if m.property != Property::Year {
                    return true;
                }
                !match_overlaps_any_title_year(m.start, m.end, &yi.title_years)
            });
        }

        // Step 3: Resolve overlapping conflicts.
        let pre_resolve_count = all_matches.len();
        engine::resolve_conflicts(&mut all_matches);
        debug!(
            "step 3: conflict resolution — {} → {} match(es)",
            pre_resolve_count,
            all_matches.len()
        );

        // Step 4: Zone-based disambiguation.
        let pre_zone_count = all_matches.len();
        zone_rules::apply_zone_rules(input, &zone_map, &token_stream, &mut all_matches);
        debug!(
            "step 4: zone disambiguation — {} → {} match(es)",
            pre_zone_count,
            all_matches.len()
        );

        // Step 4b: Mark Part reclaimable when an Episode is present so
        // the standard title absorption flow handles anime titles
        // containing "Part N" (see #128 Debt #3, principled replacement
        // for an earlier post-hoc title-absorption corrector).
        part::mark_reclaimable_when_episode_present(&mut all_matches);

        for m in &all_matches {
            trace!(
                "  resolved: {:?}={} at {}..{}",
                m.property, m.value, m.start, m.end
            );
        }

        (all_matches, token_stream, zone_map)
    }

    /// Run Pass 2: positional extraction (release group, title, episode title, etc.).
    ///
    /// When `title_override` is `Some(...)`, the provided title is used directly
    /// instead of running the standard positional title extractor. This is the
    /// hook for cross-file invariance detection (`run_with_context`).
    ///
    /// When `report` is `Some(...)`, year and episode signals from cross-file
    /// analysis are applied to disambiguate year-in-title numbers and confirm
    /// episode evidence.
    fn pass2(
        &self,
        input: &str,
        all_matches: &mut Vec<MatchSpan>,
        zone_map: &ZoneMap,
        token_stream: &TokenStream,
        title_hint: Option<TitleHint>,
        report: Option<&invariance::InvarianceReport>,
    ) -> HunchResult {
        // Step 5a: Release group (post-resolution — can see claimed positions).
        let rg_matches = release_group::find_matches(input, all_matches, zone_map, token_stream);
        // Always log — the `debug!` macro lazily evaluates its args only
        // when debug-level logging is enabled, so the empty-list case
        // costs nothing in release builds. Removing the previous
        // `if !rg_matches.is_empty()` guard eliminates a mutant whose
        // only effect was to gate the log line (equivalent mutation).
        debug!(
            "step 5a: release group — found {:?}",
            rg_matches
                .iter()
                .map(|m| m.value.as_str())
                .collect::<Vec<_>>()
        );
        all_matches.extend(rg_matches);

        // Step 5a.1: Zone rules that depend on release group positions.
        zone_rules::apply_post_release_group_rules(all_matches);

        // Step 5a.2: Cross-file year/episode disambiguation.
        // When an InvarianceReport is available, use its signals to:
        //   - Suppress Year matches for invariant year-like numbers (they're title content)
        //   - Inject episode matches for sequential variant numbers
        if let Some(report) = report {
            pass2_helpers::apply_invariance_signals(all_matches, report);
        }

        // Step 5b: Title extraction.
        //
        // The pipeline orchestrates between three possible title sources:
        //   1. The file's own pass2 extraction (with declared
        //      `TitleConfidence`).
        //   2. A cross-file `title_hint` (invariance OR ancestor
        //      fallback, already disambiguated by the caller).
        //
        // [`pick_final_title`] applies the precedence:
        //   - Strong extraction always wins (file is self-describing).
        //   - Otherwise, hint wins if present.
        //   - Otherwise, weak extraction is used.
        //   - Otherwise, no title.
        let extraction = title::extract_title(input, all_matches, zone_map, token_stream);
        if let Some(final_title) = pick_final_title(input, extraction, title_hint.as_ref()) {
            debug!(
                "step 5b: title → {:?} at {}..{}",
                final_title.value, final_title.start, final_title.end
            );
            // Reclaimables inside the title span (e.g. `Part N` in anime
            // titles, or `3D` in `Pacific.Rim.3D`) get absorbed.
            title::absorb_reclaimable(&final_title, all_matches);
            all_matches.push(final_title);
        }
        // Film title: when -fNN- marker exists, split franchise from movie title.
        if let Some((film_title, adjusted_title)) =
            title::extract_film_title(input, all_matches, token_stream)
        {
            all_matches.retain(|m| m.property != Property::Title);
            all_matches.push(film_title);
            all_matches.push(adjusted_title);
        }

        // Step 5c: Episode title.
        if let Some(ep_title) = title::extract_episode_title(input, all_matches, token_stream) {
            debug!("step 5c: episode title — \"{}\"", ep_title.value);
            // Remove release_group if it overlaps with the episode title.
            // Plex-dash format (`Show - S01E01 - Episode Title.mkv`) triggers
            // last-word fallback release_group extraction on the final word of
            // the episode title (e.g., "Ninja" from "Rising Ninja"). Fixes #38.
            let ep_start = ep_title.start;
            let ep_end = ep_title.end;
            all_matches.retain(|m| {
                if m.property != Property::ReleaseGroup {
                    return true;
                }
                // Drop RG if it's fully inside or substantially overlaps the episode title.
                !pass2_helpers::release_group_overlaps_episode_title(
                    m.start, m.end, ep_start, ep_end,
                )
            });
            all_matches.push(ep_title);
        }

        // Step 5d: Alternative title(s).
        let alt_titles = title::extract_alternative_titles(input, all_matches, token_stream);
        for alt_title in alt_titles {
            all_matches.push(alt_title);
        }

        let media_type = title::infer_media_type(input, all_matches);
        let proper_count = proper_count::compute_proper_count(input, all_matches);

        // Step 5e: When media_type is "movie", drop heuristic-only episode
        // matches — bare numbers like "10" in "Movie.10" are franchise
        // numbers, not episodes. Strong episode signals (SxxExx) are kept.
        if media_type == "movie" {
            all_matches.retain(|m| {
                !(m.property == Property::Episode && m.priority <= priority::HEURISTIC)
            });
        }

        // Step 5f: Strip video/audio tech properties from subtitle containers.
        // Files like .ass, .srt, .sub should not carry video_codec, color_depth, etc.
        pass2_helpers::strip_tech_from_subtitle_containers(all_matches);

        // Step 6: Build result.
        debug!(
            "step 6: building result from {} final match(es), media_type={}",
            all_matches.len(),
            media_type
        );
        let mut result = HunchResult::from_matches(all_matches);
        result.set(Property::MediaType, media_type);
        if proper_count > 0 {
            result.set(Property::ProperCount, proper_count.to_string());
        }

        // Step 7: Compute confidence.
        let confidence =
            pass2_helpers::compute_confidence(&result, title_hint.is_some(), all_matches);
        result.set_confidence(confidence);
        debug!("step 7: confidence = {:?}", confidence);

        result
    }

    /// Run all matchers: TOML token rules + legacy raw-string matchers.
    fn match_all(
        &self,
        input: &str,
        token_stream: &TokenStream,
        zone_map: &ZoneMap,
    ) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // TOML rules: segment-aware matching.
        // Each rule set declares its SegmentScope:
        //   FilenameOnly  → skip directory segments entirely
        //   AllSegments   → match dirs too, but with a priority penalty
        for rule in &self.toml_rules {
            for (seg_idx, segment) in token_stream.segments.iter().enumerate() {
                let is_dir = segment.kind == tokenizer::SegmentKind::Directory;

                // Skip directory segments for filename-only rules.
                if is_dir && rule.scope == SegmentScope::FilenameOnly {
                    continue;
                }

                // Directory matches get a priority penalty so filename wins in conflicts.
                let effective_priority =
                    matching::effective_priority_for_segment(rule.priority, is_dir);

                // Use per-directory zone map for directory segments.
                let dir_zone = if is_dir {
                    matching::find_dir_zone_for_segment(&zone_map.dir_zones, seg_idx)
                } else {
                    None
                };

                let tokens = &segment.tokens;
                matching::match_tokens_in_segment(
                    &MatchContext {
                        input,
                        tokens,
                        rule_set: rule.rules,
                        property: rule.property,
                        priority: effective_priority,
                        zone_map,
                        dir_zone,
                    },
                    &mut matches,
                );
            }
        }

        // Legacy matchers: run against raw input.
        for matcher in &self.legacy_matchers {
            matches.extend(matcher(input));
        }

        // Extension → Container: emit directly from the tokenizer's extension
        // field. This is PATH A for container detection (see container.toml).
        // Priority 10 beats all other container matches.
        if let Some(ext) = &token_stream.extension {
            let ext_start = input.len() - ext.len();
            matches.push(
                MatchSpan::new(ext_start, input.len(), Property::Container, ext.as_str())
                    .with_extension()
                    .with_priority(priority::EXTENSION),
            );
        }

        matches
    }
}

/// Pick the final title from the file's own extraction and an optional
/// cross-file [`TitleHint`].
///
/// Precedence (highest first):
/// 1. **Strong extraction.** The file declared its own title via an
///    explicit structural marker (bracket group, year/episode anchor,
///    structural separator). The author is self-describing; second-
///    guessing them is exactly the bug `filename_has_bracket` was
///    patching around. Hint is ignored.
/// 2. **Hint.** No strong self-description was found. The cross-file
///    signal (invariance OR ancestor fallback, already prioritized by
///    the caller) wins.
/// 3. **Weak extraction.** No hint either. Use whatever residual title
///    the file produced.
/// 4. **Nothing.** Pass2 produced no title; no hint exists.
///
/// When the hint wins, this function locates its byte span in `input`
/// (using `hint.position` if available, then a verbatim `find`, finally
/// falling back to a zero-width span at offset 0 when the text doesn't
/// appear literally — e.g. a normalized fallback like `"Paw Patrol"`
/// against an input like `"Paw Patrol/SP/Special.720p.mkv"` *will* be
/// found, but normalized titles often won't).
fn pick_final_title(
    input: &str,
    extraction: Option<TitleExtraction>,
    hint: Option<&TitleHint>,
) -> Option<MatchSpan> {
    match (extraction, hint) {
        // Strong extraction beats any hint.
        (Some(ex), _) if ex.confidence == TitleConfidence::Strong => {
            trace!(
                "title decision: STRONG extraction wins ({:?}); hint discarded",
                ex.span.value
            );
            Some(ex.span)
        }
        // No strong self-description — use the hint when present.
        (ex, Some(h)) => {
            trace!(
                "title decision: hint wins (source={}, value={:?}); extraction was {:?}",
                h.source,
                h.value,
                ex.as_ref().map(|e| (&e.span.value, e.confidence))
            );
            Some(hint_to_match(input, h))
        }
        // No hint, use weak extraction if any.
        (Some(ex), None) => {
            trace!(
                "title decision: weak extraction wins ({:?}); no hint available",
                ex.span.value
            );
            Some(ex.span)
        }
        (None, None) => {
            trace!("title decision: no extraction, no hint — no title");
            None
        }
    }
}

/// Materialize a [`TitleHint`] into a [`MatchSpan`], locating its byte
/// span in `input` if possible.
fn hint_to_match(input: &str, hint: &TitleHint) -> crate::matcher::span::MatchSpan {
    use crate::matcher::span::{MatchSpan, Property};

    let value = hint.value.as_str();
    let position = hint.position.or_else(|| input.find(value));
    if let Some(start) = position {
        let (start, end) =
            pass2_helpers::compute_override_title_span(start, value.len(), input.len());
        MatchSpan::new(start, end, Property::Title, value)
    } else {
        // Title text not in input verbatim — emit a zero-width span.
        MatchSpan::new(0, 0, Property::Title, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- match_overlaps_any_title_year ----
    //
    // These tests pin the half-open interval boundaries directly.
    // The original code used `m.start < ty.end && m.end > ty.start`
    // and 5 boundary mutants survived because no test exercised the
    // touching/equal cases. Each test below is named for the mutation
    // it kills.

    fn ty(start: usize, end: usize) -> TitleYear {
        TitleYear {
            value: 2049, // arbitrary; not under test here
            start,
            end,
        }
    }

    #[test]
    fn overlap_empty_title_years_returns_false() {
        // Vacuous "any": no ranges to compare against.
        // (The early-return guard at the call site relies on this.)
        assert!(!match_overlaps_any_title_year(0, 100, &[]));
    }

    #[test]
    fn overlap_match_fully_inside_title_year_returns_true() {
        // Sanity: the obvious overlap case.
        // ty=[10,14), match=[11,13) — fully contained.
        assert!(match_overlaps_any_title_year(11, 13, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_fully_contains_title_year_returns_true() {
        // ty=[10,14), match=[5,20) — match strictly larger.
        assert!(match_overlaps_any_title_year(5, 20, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_disjoint_before_returns_false() {
        // ty=[10,14), match=[0,5) — no overlap, gap of 5.
        // Pins `<` against `==`/`<=` and `>` against `<`.
        assert!(!match_overlaps_any_title_year(0, 5, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_disjoint_after_returns_false() {
        // ty=[10,14), match=[20,25) — no overlap, gap of 6.
        // Pins `>` against `==`/`>=`.
        assert!(!match_overlaps_any_title_year(20, 25, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_touching_at_left_returns_false() {
        // ty=[10,14), match=[5,10) — touching but NOT overlapping.
        // m.end (10) == ty.start (10), so `m.end > ty.start` is false.
        // This kills `>` -> `>=` (which would falsely return true).
        // This kills `>` -> `==` (false at 10>0 vs 10==0).
        assert!(!match_overlaps_any_title_year(5, 10, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_touching_at_right_returns_false() {
        // ty=[10,14), match=[14,20) — touching but NOT overlapping.
        // m.start (14) == ty.end (14), so `m.start < ty.end` is false.
        // This kills `<` -> `<=` (which would falsely return true).
        // This kills `<` -> `==` (15<14 false vs 15==14 false; but 14==14 differs).
        assert!(!match_overlaps_any_title_year(14, 20, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_one_byte_inside_at_right_edge_returns_true() {
        // ty=[10,14), match=[13,20) — one byte (index 13) inside.
        // m.start (13) < ty.end (14) → true; m.end (20) > ty.start (10) → true.
        // Pins `<` against `==` (13==14 false vs 13<14 true).
        assert!(match_overlaps_any_title_year(13, 20, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_match_one_byte_inside_at_left_edge_returns_true() {
        // ty=[10,14), match=[5,11) — one byte (index 10) inside.
        // m.end (11) > ty.start (10) → true; m.start (5) < ty.end (14) → true.
        // Pins `>` against `==` (11==10 false vs 11>10 true).
        assert!(match_overlaps_any_title_year(5, 11, &[ty(10, 14)]));
    }

    #[test]
    fn overlap_with_multiple_title_years_returns_true_if_any_match() {
        // Three ranges; only the third overlaps.
        let years = vec![ty(0, 4), ty(10, 14), ty(20, 24)];
        assert!(match_overlaps_any_title_year(22, 23, &years));
    }

    #[test]
    fn overlap_with_multiple_title_years_returns_false_if_none_match() {
        // Three ranges; match sits in the gap between two.
        let years = vec![ty(0, 4), ty(10, 14), ty(20, 24)];
        assert!(!match_overlaps_any_title_year(15, 19, &years));
    }
}

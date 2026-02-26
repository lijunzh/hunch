//! Proper/repack count computation.

use std::sync::LazyLock;

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer;

static REAL_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)^REAL$").unwrap());

static REPACK_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)^(?:REPACK|RERIP)(\d+)?$").unwrap());

pub fn compute_proper_count(input: &str, matches: &[MatchSpan]) -> u32 {
    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let mut has_real = false;
    let mut proper_count_raw: u32 = 0;
    let mut repack_count: u32 = 0;

    let tech_start = matches
        .iter()
        .filter(|m| {
            m.start >= fn_start
                && matches!(
                    m.property,
                    Property::VideoCodec
                        | Property::AudioCodec
                        | Property::Source
                        | Property::ScreenSize
                )
        })
        .map(|m| m.start)
        .min();

    if let Some(ts) = tech_start {
        let tech_tokens = tokenizer::tokenize(&input[ts..]);
        if tech_tokens
            .tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("REAL"))
        {
            has_real = true;
        }
    }

    for m in matches
        .iter()
        .filter(|m| m.property == Property::Other && m.value == "Proper" && m.start >= fn_start)
    {
        let raw = &input[m.start..m.end];
        if REAL_RE.is_match(raw) {
            has_real = true;
            continue;
        }
        if let Some(caps) = REPACK_RE.captures(raw) {
            if let Some(num) = caps.get(1) {
                repack_count += num.as_str().parse::<u32>().unwrap_or(1);
            } else {
                repack_count += 1;
            }
            continue;
        }
        proper_count_raw += 1;
    }

    let base = if has_real { 2 } else { proper_count_raw };
    base + repack_count
}

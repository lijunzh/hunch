//! Date detection.
//!
//! Detects air dates in filenames: 2014.12.25, 25-12-2014, etc.

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static DIGIT_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::Digit),
    right: Some(CharClass::Digit),
};

static ALPHA_DIGIT_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),
    right: Some(CharClass::AlphaDigit),
};

/// YYYY.MM.DD or YYYY-MM-DD format.
static DATE_YMD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<date>(?:19|20)\d{2})[.-](?P<month>0[1-9]|1[0-2])[.-](?P<day>0[1-9]|[12]\d|3[01])",
    )
    .unwrap()
});

/// YYYY with non-standard separator: 2008x12.13
static DATE_YMIXED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<date>(?:19|20)\d{2})[x](?P<month>0[1-9]|1[0-2])[.-](?P<day>0[1-9]|[12]\d|3[01])",
    )
    .unwrap()
});

/// DD.MM.YYYY or DD-MM-YYYY format.
static DATE_DMY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<day>0[1-9]|[12]\d|3[01])[.-](?P<month>0[1-9]|1[0-2])[.-](?P<year>(?:19|20)\d{2})",
    )
    .unwrap()
});

/// MM-DD-YYYY format (US style).
static DATE_MDY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<month>0[1-9]|1[0-2])[-](?P<day>0[1-9]|[12]\d|3[01])[-](?P<year>(?:19|20)\d{2})",
    )
    .unwrap()
});

/// YYYYMMDD compact format (no separators).
static DATE_COMPACT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<year>(?:19|20)\d{2})(?P<month>0[1-9]|1[0-2])(?P<day>0[1-9]|[12]\d|3[01])")
        .unwrap()
});

/// DD.MM.YY or YY.MM.DD (2-digit year).
static DATE_2DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?P<a>\d{2})[.-](?P<b>\d{2})[.-](?P<c>\d{2})").unwrap());

/// Month name date: "2 mar 2013", "March 5 2020", "5th January 2019".
static DATE_MONTH_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?:(?P<day1>\d{1,2})(?:st|nd|rd|th)?\s+(?P<mon1>jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:tember)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\s+(?P<year1>(?:19|20)\d{2})|(?P<mon2>jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:tember)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\s+(?P<day2>\d{1,2})(?:st|nd|rd|th)?[,]?\s+(?P<year2>(?:19|20)\d{2}))"
    ).unwrap()
});

/// Scan for date patterns (e.g., `2024-01-15`, `2024.01.15`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();

    // 1. YYYY.MM.DD
    if let Some(cap) = DATE_YMD.captures(input)
        && let (Some(full), Some(year), Some(month), Some(day)) = (
            cap.get(0),
            cap.name("date"),
            cap.name("month"),
            cap.name("day"),
        )
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Date,
                format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
            )
            .with_priority(2),
        );
    }

    // 2. YYYYxMM.DD (mixed separator)
    if matches.is_empty()
        && let Some(cap) = DATE_YMIXED.captures(input)
        && let (Some(full), Some(year), Some(month), Some(day)) = (
            cap.get(0),
            cap.name("date"),
            cap.name("month"),
            cap.name("day"),
        )
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Date,
                format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
            )
            .with_priority(2),
        );
    }

    // 3. DD.MM.YYYY
    if matches.is_empty()
        && let Some(cap) = DATE_DMY.captures(input)
        && let (Some(full), Some(year), Some(month), Some(day)) = (
            cap.get(0),
            cap.name("year"),
            cap.name("month"),
            cap.name("day"),
        )
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Date,
                format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
            )
            .with_priority(2),
        );
    }

    // 4. MM-DD-YYYY (US style)
    if matches.is_empty()
        && let Some(cap) = DATE_MDY.captures(input)
        && let (Some(full), Some(year), Some(month), Some(day)) = (
            cap.get(0),
            cap.name("year"),
            cap.name("month"),
            cap.name("day"),
        )
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Date,
                format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
            )
            .with_priority(2),
        );
    }

    // 5. YYYYMMDD compact
    if matches.is_empty()
        && let Some(cap) = DATE_COMPACT.captures(input)
        && let (Some(full), Some(year), Some(month), Some(day)) = (
            cap.get(0),
            cap.name("year"),
            cap.name("month"),
            cap.name("day"),
        )
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Date,
                format!("{}-{}-{}", year.as_str(), month.as_str(), day.as_str()),
            )
            .with_priority(1),
        );
    }

    // 6. DD.MM.YY / YY.MM.DD (2-digit year)
    if matches.is_empty()
        && let Some(cap) = DATE_2DIGIT.captures(input)
        && let (Some(full), Some(a), Some(b), Some(c)) =
            (cap.get(0), cap.name("a"), cap.name("b"), cap.name("c"))
        && check_boundary(bytes, full.start(), full.end(), &DIGIT_BOUNDARY)
    {
        let av: u32 = a.as_str().parse().unwrap_or(0);
        let bv: u32 = b.as_str().parse().unwrap_or(0);
        let cv: u32 = c.as_str().parse().unwrap_or(0);

        if let Some((y, m, d)) = resolve_2digit_date(av, bv, cv) {
            let year_full = if y < 100 {
                if y > 29 { 1900 + y } else { 2000 + y }
            } else {
                y
            };
            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Date,
                    format!("{year_full}-{m:02}-{d:02}"),
                )
                .with_priority(0),
            );
        }
    }

    // 7. Month name dates: "2 mar 2013", "March 5, 2020"
    if matches.is_empty()
        && let Some(cap) = DATE_MONTH_NAME.captures(input)
        && let Some(full) = cap.get(0)
        && check_boundary(bytes, full.start(), full.end(), &ALPHA_DIGIT_BOUNDARY)
    {
        let (day_s, mon_s, year_s) = if cap.name("day1").is_some() {
            (
                cap.name("day1").unwrap().as_str(),
                cap.name("mon1").unwrap().as_str(),
                cap.name("year1").unwrap().as_str(),
            )
        } else {
            (
                cap.name("day2").unwrap().as_str(),
                cap.name("mon2").unwrap().as_str(),
                cap.name("year2").unwrap().as_str(),
            )
        };
        if let Some(month_num) = month_name_to_num(mon_s) {
            let day: u32 = day_s.parse().unwrap_or(0);
            if (1..=31).contains(&day) {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Date,
                        format!("{}-{month_num:02}-{day:02}", year_s),
                    )
                    .with_priority(1),
                );
            }
        }
    }

    matches
}

/// Resolve ambiguous 2-digit date: returns (year, month, day) or None.
/// Rules:
/// - If first > 31 → first is year (YY.MM.DD)
/// - If last > 31 → last is year (DD.MM.YY)
/// - If first > 12 → first is day (DD.MM.YY)
/// - If last > 12 → last is day → first=year (YY.MM.DD)
/// - Default: DD.MM.YY (day-first, European convention)
fn resolve_2digit_date(a: u32, b: u32, c: u32) -> Option<(u32, u32, u32)> {
    // b is always month in these 3-field dates.
    if !(1..=12).contains(&b) {
        return None;
    }
    if a > 31 {
        // YY.MM.DD
        if (1..=31).contains(&c) {
            return Some((a, b, c));
        }
        return None;
    }
    if c > 31 {
        // DD.MM.YY — but c is 2-digit so this won't happen (c ≤ 99)
        if (1..=31).contains(&a) {
            return Some((c, b, a));
        }
        return None;
    }
    // Both a and c are ≤ 31.
    if a > 12 {
        // a must be day → DD.MM.YY
        return Some((c, b, a));
    }
    if c > 12 {
        // c must be day → YY.MM.DD... but wait, c > 12 could be day or year.
        // If c ≤ 31, it's ambiguous. Default: assume a=DD, c=YY.
        return Some((c, b, a));
    }
    // Fully ambiguous (all ≤ 12). Default: DD.MM.YY.
    Some((c, b, a))
}

/// Convert month name/abbreviation to number.
fn month_name_to_num(name: &str) -> Option<u32> {
    match name.to_lowercase().as_str() {
        s if s.starts_with("jan") => Some(1),
        s if s.starts_with("feb") => Some(2),
        s if s.starts_with("mar") => Some(3),
        s if s.starts_with("apr") => Some(4),
        "may" => Some(5),
        s if s.starts_with("jun") => Some(6),
        s if s.starts_with("jul") => Some(7),
        s if s.starts_with("aug") => Some(8),
        s if s.starts_with("sep") => Some(9),
        s if s.starts_with("oct") => Some(10),
        s if s.starts_with("nov") => Some(11),
        s if s.starts_with("dec") => Some(12),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ymd() {
        let m = find_matches("Show.2014.12.25.HDTV.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }

    #[test]
    fn test_dmy() {
        let m = find_matches("Show.25-12-2014.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }
}

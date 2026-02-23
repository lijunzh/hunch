//! Date detection.
//!
//! Detects air dates in filenames: 2014.12.25, 25-12-2014, etc.

use lazy_static::lazy_static;
use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// YYYY.MM.DD or YYYY-MM-DD format.
    static ref DATE_YMD: Regex = Regex::new(
        r"(?<![0-9])(?P<date>(?:19|20)\d{2})[.-](?P<month>0[1-9]|1[0-2])[.-](?P<day>0[1-9]|[12]\d|3[01])(?![0-9])"
    ).unwrap();

    /// YYYY with non-standard separator: 2008x12.13
    static ref DATE_YMIXED: Regex = Regex::new(
        r"(?<![0-9])(?P<date>(?:19|20)\d{2})[x](?P<month>0[1-9]|1[0-2])[.-](?P<day>0[1-9]|[12]\d|3[01])(?![0-9])"
    ).unwrap();

    /// DD.MM.YYYY or DD-MM-YYYY format.
    static ref DATE_DMY: Regex = Regex::new(
        r"(?<![0-9])(?P<day>0[1-9]|[12]\d|3[01])[.-](?P<month>0[1-9]|1[0-2])[.-](?P<year>(?:19|20)\d{2})(?![0-9])"
    ).unwrap();

    /// MM-DD-YYYY format (US style).
    static ref DATE_MDY: Regex = Regex::new(
        r"(?<![0-9])(?P<month>0[1-9]|1[0-2])[-](?P<day>0[1-9]|[12]\d|3[01])[-](?P<year>(?:19|20)\d{2})(?![0-9])"
    ).unwrap();

    /// YYYYMMDD compact format (no separators).
    static ref DATE_COMPACT: Regex = Regex::new(
        r"(?<![0-9])(?P<year>(?:19|20)\d{2})(?P<month>0[1-9]|1[0-2])(?P<day>0[1-9]|[12]\d|3[01])(?![0-9])"
    ).unwrap();

    /// DD.MM.YY 2-digit year format.
    static ref DATE_DMY_SHORT: Regex = Regex::new(
        r"(?<![0-9])(?P<day>0[1-9]|[12]\d|3[01])[.-](?P<month>0[1-9]|1[0-2])[.-](?P<yy>\d{2})(?![0-9])"
    ).unwrap();
}

pub struct DateMatcher;

impl PropertyMatcher for DateMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // 1. YYYY.MM.DD
        if let Ok(Some(cap)) = DATE_YMD.captures(input) {
            if let (Some(full), Some(year), Some(month), Some(day)) =
                (cap.get(0), cap.name("date"), cap.name("month"), cap.name("day"))
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
        }

        // 2. YYYYxMM.DD (mixed separator)
        if matches.is_empty() {
            if let Ok(Some(cap)) = DATE_YMIXED.captures(input) {
                if let (Some(full), Some(year), Some(month), Some(day)) =
                    (cap.get(0), cap.name("date"), cap.name("month"), cap.name("day"))
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
            }
        }

        // 3. DD.MM.YYYY
        if matches.is_empty() {
            if let Ok(Some(cap)) = DATE_DMY.captures(input) {
                if let (Some(full), Some(year), Some(month), Some(day)) =
                    (cap.get(0), cap.name("year"), cap.name("month"), cap.name("day"))
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
            }
        }

        // 4. MM-DD-YYYY (US style)
        if matches.is_empty() {
            if let Ok(Some(cap)) = DATE_MDY.captures(input) {
                if let (Some(full), Some(year), Some(month), Some(day)) =
                    (cap.get(0), cap.name("year"), cap.name("month"), cap.name("day"))
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
            }
        }

        // 5. YYYYMMDD compact
        if matches.is_empty() {
            if let Ok(Some(cap)) = DATE_COMPACT.captures(input) {
                if let (Some(full), Some(year), Some(month), Some(day)) =
                    (cap.get(0), cap.name("year"), cap.name("month"), cap.name("day"))
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
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ymd() {
        let m = DateMatcher.find_matches("Show.2014.12.25.HDTV.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }

    #[test]
    fn test_dmy() {
        let m = DateMatcher.find_matches("Show.25-12-2014.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2014-12-25");
    }
}

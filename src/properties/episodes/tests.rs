use super::*;

#[test]
fn test_s01e02() {
    let m = find_matches("Show.S01E02.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
}

#[test]
fn test_multi_episode_e01e02() {
    let m = find_matches("Show.S01E01E02.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
}

#[test]
fn test_multi_episode_e01_dash_02() {
    let m = find_matches("Show.S03E01-02.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
}

#[test]
fn test_multi_episode_e01_dash_e02() {
    let m = find_matches("Show.S03E01-E02.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
}

#[test]
fn test_1x03() {
    let m = find_matches("Show.1x03.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "3")
    );
}

#[test]
fn test_s03_dash_e01() {
    let m = find_matches("Parks_and_Recreation-s03-e01.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "3")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_s06xe01() {
    let m = find_matches("The Office - S06xE01.avi");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "6")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_episode_word() {
    let m = find_matches("Show Season 2 Episode 5");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "2")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "5")
    );
}

#[test]
fn test_standalone_ep() {
    let m = find_matches("Show.E05.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "5")
    );
}

#[test]
fn test_season_dir() {
    let m = find_matches("TV/Show/Season 6/file.avi");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "6")
    );
}

#[test]
fn test_s01_only() {
    let m = find_matches("Show.S01Extras.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "1")
    );
}

#[test]
fn test_s03_dash_x01() {
    let m = find_matches("Parks_and_Recreation-s03-x01.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "3")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_three_digit_501() {
    let m = find_matches("the.mentalist.501.hdtv.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "5")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_three_digit_117() {
    let m = find_matches("new.girl.117.hdtv.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "17")
    );
}

#[test]
fn test_four_digit_2401() {
    let m = find_matches("the.simpsons.2401.hdtv.mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "24")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_anime_dash_episode() {
    let m = find_matches("Show Name - 03 Vostfr HD");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "3")
    );
}

#[test]
fn test_bare_dot_episode() {
    let m = find_matches("Neverwhere.05.Down.Street.avi");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "5")
    );
}

#[test]
fn test_cap_single() {
    let m = find_matches("Show.Name.-.Temporada.1.720p.HDTV.x264[Cap.102]SPANISH.AUDIO");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "1")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
}

#[test]
fn test_cap_range() {
    let m = find_matches("Show.Name.-.Temporada.1.720p.HDTV.x264[Cap.102_104]SPANISH.AUDIO");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "2")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "3")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "4")
    );
}

#[test]
fn test_cap_four_digit() {
    let m = find_matches("Show.Name.-.Temporada.15.720p.HDTV.x264[Cap.1503]SPANISH.AUDIO");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "15")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "3")
    );
}

#[test]
fn test_cap_four_digit_range() {
    let m = find_matches("Show.Name.-.Temporada.15.720p.HDTV.x264[Cap.1503_1506]SPANISH.AUDIO");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "3")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "6")
    );
}

#[test]
fn test_s_range() {
    let m = find_matches("Friends.S01-S10.COMPLETE.720p.BluRay.x264-PtM");
    let seasons: Vec<&str> = m
        .iter()
        .filter(|x| x.property == Property::Season)
        .map(|x| x.value.as_str())
        .collect();
    assert!(
        seasons.len() >= 2,
        "Expected multi-season, got: {:?}",
        seasons
    );
}

#[test]
fn test_s_concat() {
    let m = find_matches("Some Series S01S02S03");
    let seasons: Vec<&str> = m
        .iter()
        .filter(|x| x.property == Property::Season)
        .map(|x| x.value.as_str())
        .collect();
    assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
}

#[test]
fn test_s_multi_num() {
    let m = find_matches("Some Series S01-02-03");
    let seasons: Vec<&str> = m
        .iter()
        .filter(|x| x.property == Property::Season)
        .map(|x| x.value.as_str())
        .collect();
    assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
}

#[test]
fn test_season_range_word() {
    let m = find_matches("Show.Name.-.Season.1.to.3.-.Mp4.1080p");
    let seasons: Vec<&str> = m
        .iter()
        .filter(|x| x.property == Property::Season)
        .map(|x| x.value.as_str())
        .collect();
    assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
}

// ── Issue #212: CJK fansub patterns ──────────────────────────────

#[test]
fn test_nth_dash_episode_basic() {
    // [4th - 01] → season 4, episode 1
    let m = find_matches("[Group][Title][4th - 01][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "4"),
        "Expected season 4, got: {:?}",
        m.iter()
            .filter(|x| x.property == Property::Season)
            .collect::<Vec<_>>()
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1"),
        "Expected episode 1, got: {:?}",
        m.iter()
            .filter(|x| x.property == Property::Episode)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_nth_dash_episode_all_ordinals() {
    // 1st through 9th should all parse.
    for (label, season) in [
        ("1st", "1"),
        ("2nd", "2"),
        ("3rd", "3"),
        ("4th", "4"),
        ("5th", "5"),
        ("9th", "9"),
    ] {
        let filename = format!("[Group][Title][{} - 03][1080P].mkv", label);
        let m = find_matches(&filename);
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == season),
            "Expected season {} for {:?}, got: {:?}",
            season,
            filename,
            m
        );
    }
}

#[test]
fn test_nth_dash_episode_with_version() {
    // [4th - 01v2] — the v2 is a revision suffix; episode is still 1.
    let m = find_matches("[Group][Title][4th - 01v2][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "4")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "1")
    );
}

#[test]
fn test_nth_dash_episode_em_dash() {
    // Some fansubs use em-dash (—) or en-dash (–) instead of hyphen.
    let m = find_matches("[Group][Title][3rd – 12][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::Season && x.value == "3")
    );
    assert!(
        m.iter()
            .any(|x| x.property == Property::Episode && x.value == "12")
    );
}

#[test]
fn test_nth_dash_episode_ignores_two_digit_ordinals() {
    // We deliberately only match single-digit ordinals (1st-9th) to
    // avoid false positives on group names / scene tags.
    let m = find_matches("[Group][Title][10th - 05][1080P].mkv");
    assert!(
        !m.iter()
            .any(|x| x.property == Property::Season && x.value == "10"),
        "Should NOT match two-digit ordinals like '10th', got: {:?}",
        m.iter()
            .filter(|x| x.property == Property::Season)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_cjk_cumulative_episode_basic() {
    // [总第67] → absolute_episode 67
    let m = find_matches("[Group][Title][4th - 01][总第67][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::AbsoluteEpisode && x.value == "67"),
        "Expected absolute_episode 67, got: {:?}",
        m.iter()
            .filter(|x| x.property == Property::AbsoluteEpisode)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_cjk_cumulative_episode_with_whitespace() {
    // [总第 100] with space — should still match.
    let m = find_matches("[Group][Title][总第 100][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::AbsoluteEpisode && x.value == "100")
    );
}

#[test]
fn test_cjk_cumulative_episode_independent_of_episode() {
    // The cumulative pattern emits AbsoluteEpisode even when no
    // regular Episode is present in the filename.
    let m = find_matches("[Group][Title][总第42][1080P].mkv");
    assert!(
        m.iter()
            .any(|x| x.property == Property::AbsoluteEpisode && x.value == "42")
    );
}

#[test]
fn test_212_full_filename_regression() {
    // The exact filenames from issue #212. Pins season + episode +
    // absolute_episode + (implicitly) `type: episode` via the
    // cascading effect on type classification.
    let cases = [
        (
            "[晚街与灯][Re Zero kara Hajimeru Isekai Seikatsu][4th - 01][总第67][WEB-DL Remux][1080P_AVC_AAC][简繁日内封PGS][V2].mkv",
            "4",
            "1",
            "67",
        ),
        (
            "[晚街与灯][Re Zero kara Hajimeru Isekai Seikatsu][4th - 02][总第68][WEB-DL Remux][1080P_AVC_AAC][简繁日内封PGS].mkv",
            "4",
            "2",
            "68",
        ),
    ];
    for (filename, season, episode, abs_ep) in cases {
        let m = find_matches(filename);
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == season),
            "Expected season {} in {:?}",
            season,
            filename
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == episode),
            "Expected episode {} in {:?}",
            episode,
            filename
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::AbsoluteEpisode && x.value == abs_ep),
            "Expected absolute_episode {} in {:?}",
            abs_ep,
            filename
        );
    }
}

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

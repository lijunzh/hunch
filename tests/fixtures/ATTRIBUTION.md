# guessit Test Fixtures

These YAML test files are copied from
[guessit](https://github.com/guessit-io/guessit) (LGPL-3.0) and are used
solely as compatibility / regression test vectors.

Original source: `guessit/test/` in the guessit repository, refreshed from
**guessit v4.5.5** (tag `v4.5.5`, commit `b296b9c`, 2026-11-22). The
monthly *guessit Drift* workflow reports when a newer release adds cases.

hunch-specific cases (not in guessit) sit at the end of `episodes.yml` and
`rules/website.yml` under a `hunch-specific cases` comment, and in
`community.yml`. Keep them when refreshing.

## Files included

- `movies.yml`, `episodes.yml`, `various.yml` — full-filename test suites
- `rules/` — per-property test files: `audio_codec`, `bonus`, `cd`,
  `common_words`, `country`, `date`, `edition`, `episodes`, `film`,
  `language`, `other`, `part`, `release_group`, `screen_size`, `size`,
  `source`, `title`, `video_codec`, `website`

Copyright (c) guessit contributors. Licensed under LGPL-3.0.
See https://github.com/guessit-io/guessit/blob/develop/LICENSE

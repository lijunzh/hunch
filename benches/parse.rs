//! Benchmarks for hunch parsing performance.
//!
//! Run with: `cargo bench`

use criterion::{Criterion, criterion_group, criterion_main};
use hunch::hunch;
use std::hint::black_box;

fn bench_movie(c: &mut Criterion) {
    c.bench_function("movie_basic", |b| {
        b.iter(|| hunch(black_box("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv")))
    });
}

fn bench_movie_complex(c: &mut Criterion) {
    c.bench_function("movie_complex", |b| {
        b.iter(|| {
            hunch(black_box(
                "Blade.Runner.2049.2017.2160p.UHD.BluRay.REMUX.HDR.HEVC.DTS-HD.MA.7.1.Atmos-EPSiLON.mkv",
            ))
        })
    });
}

fn bench_episode(c: &mut Criterion) {
    c.bench_function("episode_sxxexx", |b| {
        b.iter(|| {
            hunch(black_box(
                "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv",
            ))
        })
    });
}

fn bench_episode_with_path(c: &mut Criterion) {
    c.bench_function("episode_with_path", |b| {
        b.iter(|| {
            hunch(black_box(
                "Series/Californication/Season 2/Californication.2x05.Vaginatown.HDTV.XviD-0TV.avi",
            ))
        })
    });
}

fn bench_anime(c: &mut Criterion) {
    c.bench_function("anime_bracket", |b| {
        b.iter(|| {
            hunch(black_box(
                "[SubGroup] Anime Title - 01 [720p] [ABCD1234].mkv",
            ))
        })
    });
}

// `bench_minimal` was removed in #191 — it parsed only "movie.mkv" so it
// primarily measured function-call overhead, not parser logic, and its
// 16-22 µs baseline made it hyper-sensitive to ubuntu-latest runner-
// hardware shifts (a flat ~6 µs offset showed up as a 37% ratio —
// enough to fire the regression gate on innocuous PRs). The other 5
// benches cover the parse paths that actually matter.

criterion_group!(
    benches,
    bench_movie,
    bench_movie_complex,
    bench_episode,
    bench_episode_with_path,
    bench_anime,
);
criterion_main!(benches);

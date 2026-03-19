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

fn bench_minimal(c: &mut Criterion) {
    c.bench_function("minimal", |b| b.iter(|| hunch(black_box("movie.mkv"))));
}

criterion_group!(
    benches,
    bench_movie,
    bench_movie_complex,
    bench_episode,
    bench_episode_with_path,
    bench_anime,
    bench_minimal
);
criterion_main!(benches);

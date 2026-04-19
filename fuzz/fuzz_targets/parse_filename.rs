// Fuzz target #1: hunch::hunch(filename)
//
// The simplest, broadest target: hand the public single-string parse
// function arbitrary UTF-8 input and look for panics.
//
// Why &str (not &[u8]): `hunch::hunch` takes &str, and `Pipeline` does too.
// We let libfuzzer's mutator produce arbitrary bytes, then rejection-sample
// non-UTF-8 inputs. Panics found here are real panics on real string input.
//
// What we're hunting:
//   - integer overflow / underflow in span arithmetic
//   - out-of-bounds slice indexing on multi-byte UTF-8 boundaries
//   - regex catastrophic backtracking (would manifest as fuzzer timeout,
//     not a panic — separate finding category, but worth tracking)
//   - any unwrap()/expect()/panic!() reachable from public API
//
// What we're NOT hunting (out of scope per the epic + SECURITY.md threat model):
//   - filesystem I/O paths (covered by tests/fixtures + walk_dir guards)
//   - YAML fixture parser (third-party deps)
//
// Triage: any crash → minimize with `cargo fuzz tmin` → file an issue
// per docs/fuzzing.md, fix as either a defensive Result return or a
// debug_assert! documenting an intentional invariant.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Reject non-UTF-8 input — the public `hunch()` signature is `&str`,
    // so non-UTF-8 simply can't reach it through normal means. Wasting
    // fuzzer cycles on non-UTF-8 mutants would shrink our useful coverage.
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = hunch::hunch(input);
    }
});

// Fuzz target #2: hunch::hunch_with_context(input, siblings)
//
// Exercises the multi-input path: cross-file invariance analysis treats
// the `siblings` slice as additional context for the primary input. This
// is structurally distinct from target #1 — bugs that only manifest when
// multiple strings interact won't show up there.
//
// We use `arbitrary` to derive a structured input: the primary filename
// plus 0-N sibling strings, all sized by the fuzzer rather than a single
// blob split. This gives libfuzzer better coverage feedback because each
// String is its own object the mutator can grow/shrink/cross-over.
//
// Sibling cap of 16: in practice walk_dir caps at directory size; we
// don't need to fuzz pathological 10K-sibling cases here. The pipeline's
// invariance code is O(n) in siblings so 16 is plenty to exercise the
// merging logic.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
struct Input<'a> {
    primary: &'a str,
    // Bounded to avoid pathological inputs that don't stress the
    // invariance algorithm in interesting ways.
    siblings: Vec<&'a str>,
}

fuzz_target!(|input: Input<'_>| {
    // arbitrary already gives us &str (validated UTF-8) — no manual
    // rejection needed.
    let cap = input.siblings.len().min(16);
    let siblings: Vec<&str> = input.siblings.into_iter().take(cap).collect();
    let _ = hunch::hunch_with_context(input.primary, &siblings);
});

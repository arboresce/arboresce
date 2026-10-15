# Actual target source suffixes

These five original synthetic cases use MIT OR Apache-2.0. Cargo can select a
Rust source file whose suffix is not rs. Every actual production target source
therefore belongs in the guard regardless of its name, and a pure such source
is sufficient for a nonempty inventory. A harmless default src/lib.rs cannot
conceal effects in the actual target. This applies the existing complete-source
and owning-target requirements without adding a file-suffix ban. Independent
expected review precedes consumption. These are metadata/syntax inputs, not
compiled product or service evidence. The prior 187 accepted cases stay intact.

Two source cases reject direct/aliased resume_unwind, which intentionally starts
unwinding, under the same all-production-layer panic rule as panic_any and UFCS.

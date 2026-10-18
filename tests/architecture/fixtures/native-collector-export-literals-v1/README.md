# Lossless coverage export numeric literals

These original public synthetic literals use MIT OR Apache-2.0. They supplement
the existing complete bounded LLVM export package without changing any of its
57 cases or bounds. These twelve positive assembly cases are frozen before
consumer changes and use no customer data, secrets or operational credentials.

The existing assembly contract preserves the ENTIRE first files/totals document
and ENTIRE second function vector, including exact numeric values, nested
arrays, repeated functions and strings. Object key order is immaterial. Passing
through binary64 and serializing a rounded number violates that requirement.
For example, 99.637023593466424 and 99.63702359346644 differ, as do
94.207792625468343 and 94.20779262546833. Counts beyond binary64 integer
precision, unsigned64 maximum, signed zero, exponents, trailing zeroes, escaped
member names, delimiter-like strings, whitespace and duplicate function records
are supplied explicitly. These minimal assembly documents do not independently
qualify downstream coverage counters or source/native conservation.

Only case.input.parts reaches assembly. Each part's utf8 string supplies exact
bytes. Every expected_utf8 is a complete independently authored allowed lossless
rendering. The byte-preserving implementation inserts the second raw functions
array as a new member of the first data object; tests compare COMPLETE output
UTF-8 with that rendering, without parsing either expected or actual numbers
through another numeric representation. This regression check exercises the
chosen lossless rendering without imposing a global object-member ordering
rule on admitted input. Inputs deliberately use different member order and
escaped key spellings. The existing 57 positive/negative typed comparisons,
strict duplicate/nonfinite/trailing-data validation and complete downstream
semantic/conservation checks remain required.

Do not enable a parser feature or dependency to reconstruct approximate numbers.
Retain the existing selected tool, exact two captures, original single180s
budget, unchanged8MiB streams, complete64MiB format ceiling, input/tool identity
checks, profile/raw/module/native comparisons, source census and strict floors.
No truncation, summary-only fallback, reordering, deduplication, lossy numeric
conversion or renewed budget is permitted. An independent complete comparison
against genuine selected LLVM output still precedes qualification. Complete
owning and fresh public-only standalone collections, all original reviews,
retention and both positive owned cleanups remain separate. Fixture admission
does not qualify current or future executable bytes.

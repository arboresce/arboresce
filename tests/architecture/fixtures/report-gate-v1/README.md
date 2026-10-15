# Supplied LLVM report-count gate literals

These original public synthetic literals use MIT OR Apache-2.0. Independent
expected-value review precedes implementation and consumption. They describe a
complete report-count model and native command, not a collection runner. A pass
checks a supplied selected inventory and LLVM 3.1.0 export summary; it does not
attest actual inventory completeness, source/tool/run freshness, declaration-only
N/A, native process receipt completeness or whole verification. Those remain
required at the actual collector and whole-checkpoint gates. No declaration role
is admitted here as an unproved exemption.

The role input version is rust-report-roles-v1. Paths are normalized relative
POSIX paths with no empty, dot, parent, control or backslash component. The source
root is a normalized absolute POSIX prefix (the slash root is permitted). Package
and target names are nonempty ASCII letters/digits/underscore/hyphen. Sources
are unique, nonempty and at most4096; their roles are measured or assertion.
Targets are unique by (kind,name); a library and binary may share a name.
Their names and measured source lists are nonempty, with unique measured
source lists. Every package must contain measured source. Unknown, missing,
duplicate or mismatched report files fail. Empty target/source collections fail.

Exactly one selected export data block is admitted. Raw integer u64 counts must
be nonzero and covered<=count; region notcovered must equal count-covered.
File sums include assertion rows for comparison with the raw overall summary.
Only measured rows enter source/file/package/repository/target gates. Assertions
cannot pad source. Every metric independently requires 10*covered>9*count using
wider exact arithmetic; aggregate u64 overflow fails. Metric literals include
u64 maximum values to prove multiplication itself never wraps. Display percent
fields are informational and never override integer counts. Raw region/function
data is retained by the caller and not reinterpreted by this summary adapter.

Malformed JSON, duplicate object keys, trailing data, unsupported export format,
empty/mixed export data and byte-bound failures are invalid coverage input.
Semantic role/path/target/registration mismatches are coverage inventory mismatch.
Malformed/zero/noninteger/negative/nonfinite/inconsistent/overflowed counts are
invalid coverage counts (nonfinite JSON syntax is invalid input). A well-formed
metric at or below90% is coverage below strict threshold. Both input texts are
limited to67108864bytes before parsing; the native reader uses a bounded read.
The padded input recipes are actual owned otherwise-valid JSON files with the
exact stated length. They append only ASCII space after the complete inventory
or report. Both exact67108864byte files pass; both67108865byte files fail.
Removing the bound would therefore change the negative outcomes.

Each case starts from the independent base literals and replaces only existing
JSON pointers with literal fixture values. Builder failures fail preparation;
they cannot become expected production rejections. The base intentionally has
poor assertion coverage, while both source files and all six required source
scopes pass; its literal expected counts are separate from production output.

The maintained native binary is arboresce-report-gate. Three positional arguments
are the logical original source root and two input files. Help aliases print the
frozen usage on stdout/0; missing/extra arguments print stderr/2. Other owned
input/gate failures emit their fixed prefixed stderr/1. Successful complete output
is the frozen stdout line/0. Owned output write/flush failure exits1 with the
frozen output-failure diagnostic when stderr remains writable. No input paths or
content are echoed. Every native case uses the shared bounded real-process
observer and actual compiled binary, with fresh profiles retained at measurement.
Prepared Cargo/LLVM qualification remains separate from these synthetic values.

Scope identities use disjoint prefixes: file:<relative-path>, package:<name>,
library:<target-name>, binary:<target-name> and repository:Rust. The collision
positive deliberately names source files package:fixture and repository:Rust.
Their file-prefixed scopes remain distinct from aggregate scopes. A second
package's measured helper under tests has its own literal package/target counts;
an assertion-only package is invalid. Directory names do not determine roles.

The otherwise-valid duplicate-key cases contain full report/inventory values.
A last-wins decoder would pass them; a strict decoder must reject them instead.
The u64-maximum near90 pair distinguishes exact wide-integer comparison from
f64 rounding. Slash-root and4096/4097 registration boundaries are explicit.
Generated registration inputs are synthetic model data, never compiled source
or native runtime evidence: paths are source_prefix+decimal index+source_suffix,
with independent literal per-file and aggregate metric oracles. A4096-source
input has4099 scopes (4096file, repository, package, binary);4097is rejected.
The generator reads the literal totals; it does not calculate expected output.

The aggregate-overflow case supplies the wrapped raw line totals199/95 on
purpose: unchecked wrapping sums would otherwise admit the source94/99 scope.
Checked aggregation rejects it. The zero-row and covered>count cases also keep
otherwise-consistent overall line totals, so unrelated summary errors cannot
stand in for the intended guard. Full duplicate values are identical; both
first-wins and last-wins decoding would pass without duplicate rejection.
API oversize inputs use the same otherwise-valid padded data as native files.
Maintained input preparation is separately measured. Its bounded recipes reject
unknown pointers/targets, malformed patches, generated counts above4097, missing
literal totals and padding shorter than the complete JSON. Preparation errors
fail preparation; they are never fed to production as a negative report.

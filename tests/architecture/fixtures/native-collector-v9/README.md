# Registered descendant harness probes

A required harness may declare an optional `descendant_probes` array of at most
16 unique selected test names. Each name is nonempty, NUL-free, at most4096UTF-8
bytes and present in that harness's complete selected test metadata. Absence or
an empty array preserves the previous selection. Incidental harnesses and
ordinary binaries cannot declare this field.

Each declared probe must occur exactly once as a direct child of the same
object's selected top-level run PID. Its exact argument bytes are `--exact`,
the registered test name, `--nocapture`, and `--test-threads=1`. It needs the
same bound executable path/hash, context, source and owned acyclic parent chain,
actual exit0 without a signal, positive counters and its own fresh matching raw
profile. No unknown, duplicate, missing, driver-parent, list-parent, cross-object
or arbitrary descendant probe is admitted. All original raw/profile conservation
and top-level list/run checks remain in force.

The probe is additional measured helper execution. It does not add a selected
command, required suite or test count. The original full run must still execute
all registered tests and case IDs once, with zero skips/filters/failures. Its
parent test owns the probe's exact behavior and captured-output assertions.

These51 independently reviewed pure cases include33 receipt cases and18 Cargo
selection cases. Every prior fixture obligation remains required. This document
does not assert a whole-workspace pass. A real missing-selector probe is
qualified only by the complete owning and public-only standalone runs.

Exact16/17 declarations use complete matching child/profile sets. Exact4096 and
one-over4097 UTF-8-byte names use matching selected metadata and argument bytes;
multibyte names distinguish bytes from scalar counts. The NUL recipe likewise
keeps the declared and child name identical. These are virtual supplied facts.

The legacy empty-declaration control keeps the original valid supplied facts
without test metadata. An empty declaration remains vacuous; actual Cargo
selection still validates its complete test metadata independently.

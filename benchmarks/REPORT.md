# Benchmark report

No SP1 execution or proof-generation benchmark was completed on this Windows host.

Local evidence collected:

```text
cargo test -p ogunedo-core --all-features --locked: passed, 8 tests
cargo clippy -p ogunedo-core --all-targets --all-features --locked -- -D warnings: passed
cargo check -p ogunedo-program --locked: passed
python scripts\validate_bundle.py: passed
powershell -ExecutionPolicy Bypass -File .\scripts\local_sp1_safe.ps1 -Mode guest-build: passed
cargo check -p ogunedo-cli --locked: stopped after timeout; no success claimed
```

Resource evidence from `artifacts/local-resource-report.json`:

```text
total physical RAM before guest-build: 11.756 GiB
free physical RAM before guest-build: 0.716 GiB
total virtual memory before guest-build: 35.547 GiB
free virtual memory before guest-build: 15.88 GiB
pagefile allocated: 23.79 GiB
pagefile current usage: 2.169 GiB
Windows-reported pagefile peak usage: 6.979 GiB
free disk on C: before guest-build: 24.074 GiB
CARGO_BUILD_JOBS: 2
```

Additional resource snapshot before the bounded host CLI compile attempt on 2026-07-25:

```text
total physical RAM: 11.756 GiB
free physical RAM: 1.408 GiB
total virtual memory: 35.443 GiB
free virtual memory: 7.96 GiB
free disk on C: 25.497 GiB
```

The `ogunedo-cli` check exceeded the 180-second local budget while compiling the expanded SP1 network dependency graph. Remaining Cargo/compiler child processes were stopped cleanly. This is recorded as a local resource/time limitation, not a protocol failure.

Final safety snapshot before packaging the network-proving source tree:

```text
total physical RAM: 11.756 GiB
free physical RAM: 0.708 GiB
total virtual memory: 35.443 GiB
free virtual memory: 9.253 GiB
free disk on C: 25.377 GiB
```

No further local host-CLI/SP1-network compile or proof command was launched because physical RAM was below the heavy-command threshold.

The local runner does not report aggregate child-process peak RSS for Cargo/SP1. It reports system RAM/pagefile/disk snapshots, which are the machine-safety inputs used to decide whether proof commands may launch.

Blocked measurements:

- guest cycle count;
- host execution time;
- proof-generation wall time;
- proof-verification wall time;
- peak proving memory;
- guest ELF size;
- proof size;
- public-value size from real SP1 execution.
- network proving units and actual PROVE cost.

These must be produced by the Linux SP1 workflow before any production release candidate is tagged.

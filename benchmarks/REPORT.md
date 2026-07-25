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
WSL network-estimate: passed after installing protoc and the pinned SP1 toolchain
WSL network-prove: stopped with exit 130 after local CPU-heavy setup produced no request ID
WSL cargo check -p ogunedo-cli --locked after receipt hardening: passed in 9m14s
WSL network-estimate after receipt hardening: passed, refreshed maximum possible spend 0.258933000005886633 PROVE
WSL network-prove balance guard: refused before submission because requester balance was 0 atomic PROVE
WSL network-estimate after requester funding: passed, requester balance 10 PROVE, maximum possible spend 0.259605000005886633 PROVE
WSL compressed network-prove after approval: submitted request 0xe574b1b1770f22b015aea357466f539d0d5e899b2452640a430aa544a9398da6 and downloaded a 1,272,673-byte candidate proof
WSL compressed requester verification: interrupted with exit 130 at memory safety boundary before completed receipt/manifest
WSL credential-free verify of downloaded proof: interrupted with exit 130 at memory safety boundary
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

WSL network-path resource evidence collected on 2026-07-25:

```text
protobuf compiler installed: libprotoc 3.21.12
SP1 installer: scripts/install_sp1.sh
cargo-prove: cargo-prove sp1 (150e629 2026-05-23T01:30:54.499183867Z)
succinct toolchain: rustc 1.93.0-dev
network-estimate: completed, wrote artifacts/network-preflight.json
proof mode: compressed
maximum possible spend from preflight: 0.259605000005886633 PROVE
requester balance reported by preflight: 0 atomic PROVE
network-prove runtime before interruption: 2026-07-25T13:04:51+01:00 to 2026-07-25T13:30:40+01:00
network-prove exit: 130
network-prove artifacts: no proof, no manifest, no receipt, no request ID
lowest observed Windows free physical RAM: ~0.564 GiB
lowest observed C: free disk after WSL/SP1 build artifacts: ~19.215 GiB
highest observed single rustc RSS: ~1.72 GiB
observed network-prove requester RSS: ~399 MiB
observed network-prove CPU before interruption: ~770%
WSL swap: unused except for a transient 44 KiB during compilation
latest post-check Windows snapshot: 1.006 GiB free physical RAM, 13.472 GiB free virtual memory, 18.039 GiB free disk on C:
latest post-check WSL snapshot: 5.1 GiB available memory, 2.0 GiB swap free, 19 GiB available on /mnt/c
latest post-balance-gate Windows snapshot: 3.647 GiB free physical RAM, 16.601 GiB free virtual memory, 18.157 GiB free disk on C:
pre-submit compressed request snapshot: 4.103 GiB free Windows physical RAM, 19.144 GiB free virtual memory, 21.159 GiB free disk on C:, WSL 5.1 GiB available memory, WSL swap free
compressed requester verification peak: child verifier RSS ~3,785,148 KiB, WSL free memory ~78 MiB, WSL available memory ~1.5 GiB, Windows free physical RAM ~0.765 GiB
credential-free verification peak: verifier RSS ~3,919,668 KiB, WSL free memory ~78 MiB, WSL available memory ~1.5 GiB
post-stop recovery snapshot: 2.267 GiB free Windows physical RAM, 12.742 GiB free virtual memory, 21.068 GiB free disk on C:, WSL 5.2 GiB available memory
downloaded compressed proof SHA-256: 32971efa5337fcefe0094228a99459cbcf0f227ee1068359e184182645ba8474
```

The first stopped `network-prove` run is recorded as a local requester/setup resource boundary. It is not a protocol failure and not a failed remote proof, because no request ID or receipt was emitted.

The funded compressed request did submit and download a candidate proof, but the local laptop did not complete immediate or credential-free verification before reaching the resource safety boundary. The downloaded proof is therefore not counted as a verified proof benchmark yet.

Blocked measurements:

- guest cycle count;
- host execution time;
- completed proof-generation wall time;
- completed proof-verification wall time;
- peak proving memory;
- guest ELF size;
- verified production proof size;
- public-value size from real SP1 execution.
- network proving units and actual PROVE cost.
- completed verification receipt and manifest.

These must be produced by the Linux SP1 workflow before any production release candidate is tagged.

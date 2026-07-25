# Build status

**Repository:** `CaptSinBad/ogunedo`
**Local path:** `C:\Users\IFEANYI\Downloads\ogunedo`
**Prepared for:** Ifeanyi Joseph Ogunedo
**Version:** `0.1.0-unreleased`
**Backend pin:** SP1 `6.2.2`
**Evidence date:** 2026-07-25

## Completed on this machine

The following commands were actually run in the local repository and completed successfully:

```text
cargo generate-lockfile
cargo metadata --locked --format-version 1
cargo fmt --all -- --check
cargo test -p ogunedo-core --all-features --locked
cargo clippy -p ogunedo-core --all-targets --all-features --locked -- -D warnings
cargo check -p ogunedo-core --no-default-features --locked
cargo doc -p ogunedo-core --all-features --no-deps --locked
cargo check -p ogunedo-program --locked
python scripts\reference_check.py
python scripts\validate_bundle.py
powershell -ExecutionPolicy Bypass -File .\scripts\local_sp1_safe.ps1 -Mode guest-build
```

Observed results:

- `Cargo.lock` was generated and is committed in this source tree.
- `ogunedo-core` native tests passed: 8 tests.
- `ogunedo-core` clippy passed with `-D warnings`.
- `ogunedo-core` also compiles with default features disabled.
- The SP1 guest crate `ogunedo-program` type-checks against the pinned dependencies.
- `ogunedo-program` was rechecked after enabling the SP1 SDK `network` feature and updating the serde pin to `1.0.221`.
- The independent Python reference model regenerated the development fixtures.
- The offline bundle validator passed source structured-file parsing, script syntax checks, deterministic fixture regeneration, dev and draft NTT-vs-schoolbook checks, matrix-expander determinism checks, and SP1 version-pin checks.
- The offline bundle validator generated and checked `fixtures/public-benchmark-instance.json`, a deterministic public benchmark instance marked `safe_for_remote_proving=true`.
- The local SP1-safe runner set `CARGO_BUILD_JOBS=2`, executed guest-build only, and wrote resource evidence to `artifacts/local-resource-report.json`.

Machine-readable offline validation results are in [`VALIDATION_REPORT.json`](VALIDATION_REPORT.json).

`bash scripts/release_check.sh` was also attempted. The wrapper did not complete on this Windows/WSL-mixed host because the Bash environment cannot see the Windows Rust installation. The underlying commands in the wrapper were run directly and are listed above.

## Local resource snapshot

The safe runner observed this laptop envelope before the guest-build check:

- total physical RAM: `11.756 GiB`;
- free physical RAM: `0.716 GiB`;
- total virtual memory: `35.547 GiB`;
- free virtual memory: `15.88 GiB`;
- pagefile allocated: `23.79 GiB`;
- pagefile current usage: `2.169 GiB`;
- pagefile peak usage reported by Windows: `6.979 GiB`;
- free disk on `C:`: `24.074 GiB`;
- `CARGO_BUILD_JOBS`: `2`.

Because free physical RAM was below the configured `3 GiB` heavy-command threshold, local SP1 proof generation should remain stopped until resources are freed or moved to GitHub Actions / a supported SP1 prover network. An out-of-memory termination would be a local resource failure, not a protocol failure.

An additional bounded host CLI compile attempt was made after adding the SP1 network feature:

```text
cargo check -p ogunedo-cli --locked
```

The command was run sequentially with `CARGO_BUILD_JOBS=2`, exceeded the 180-second local budget, and was stopped. Remaining `cargo`/compiler child processes were terminated. No success is claimed for the host CLI compile on this laptop.

Resource snapshot before that attempt:

- total physical RAM: `11.756 GiB`;
- free physical RAM: `1.408 GiB`;
- total virtual memory: `35.443 GiB`;
- free virtual memory: `7.96 GiB`;
- free disk on `C:`: `25.497 GiB`.

Resource snapshot after stopping the timed-out build:

- total physical RAM: `11.756 GiB`;
- free physical RAM: `1.831 GiB`;
- total virtual memory: `35.443 GiB`;
- free virtual memory: `10.375 GiB`;
- free disk on `C:`: `25.038 GiB`.

Final safety snapshot before packaging the network-proving source tree:

- total physical RAM: `11.756 GiB`;
- free physical RAM: `0.708 GiB`;
- total virtual memory: `35.443 GiB`;
- free virtual memory: `9.253 GiB`;
- free disk on `C:`: `25.377 GiB`.

Because physical RAM was again below the heavy-command threshold, no further local SP1/network host compile or proof command was launched before packaging.

## Blocked on this Windows host

The following commands were attempted and failed because the SP1 host-side dependency `sp1-jit` does not compile for this Windows `x86_64-pc-windows-gnullvm` environment:

```text
cargo check --workspace --all-targets --all-features --locked
cargo check -p ogunedo-cli --locked
cargo test --workspace --all-features --locked
```

The failure is in the upstream `sp1-jit` crate, which imports POSIX APIs such as `std::os::fd`, `ftruncate`, `madvise`, POSIX semaphores, and `shm_open`. Those APIs are unavailable on this Windows target. This is recorded as an environment/toolchain support boundary, not as evidence of successful SP1 proving.

After adding the SP1 SDK `network` feature, the host CLI dependency graph also became too large to complete within the local 180-second safety window. The full host CLI check should run on the Linux SP1 CI/prover environment.

## Network proving implementation status

Implemented in source:

- `.env` and `.env.*` are ignored, except `.env.example`;
- `.env.example` contains variable names only;
- `network-estimate` loads only `SP1_PROVER`, `NETWORK_PRIVATE_KEY`, and optional `NETWORK_RPC_URL` into the local requester process;
- `network-estimate` refuses every instance except `fixtures/public-benchmark-instance.json`;
- remote proving requires `safe_for_remote_proving=true` and an explicit no-secret policy block;
- `network-estimate` writes a sanitized preflight report and exact approval phrase;
- `network-prove` refuses payment unless the caller supplies the exact approval phrase from the preflight;
- local `prove` refuses `SP1_PROVER=network`, preventing accidental paid fallback;
- `verify` and `vkey` use local verifier setup and do not read network credentials;
- network proof saving is atomic and followed by a fresh credential-free verifier subprocess.

Not completed:

- no network preflight was run against a funded requester wallet in this environment;
- no paid network request was submitted;
- no request ID, explorer link, PROVE cost, proof, receipt, or network verification report exists yet.

## Not completed locally

These production gates have **not** been claimed as completed:

- local SP1 toolchain installation through `cargo prove`;
- SP1 guest ELF generation through the SP1 build toolchain;
- SP1 execution of the relation;
- compressed proof generation;
- Groth16 proof generation;
- Succinct Prover Network preflight against a live funded requester;
- Succinct Prover Network compressed or Groth16 paid request;
- proof serialization/reload verification;
- verification-key extraction and pinning;
- proof tampering matrix;
- production benchmarks;
- estimator run for the draft module-ISIS profile;
- GitHub Actions execution;
- pull request creation;
- release candidate tag creation.

`cargo prove` and `gh` were not installed in this environment.

## Cryptographic deployment status

Ogunedo remains a production-oriented research implementation, not an audited production cryptographic primitive.

- no registered parameter profile has `ProductionApproved` status;
- the draft profile is `CryptanalysisRequired`;
- the determinant-one Keller compiler is not implemented in this v1 source tree;
- external audit, public cryptanalysis, and parameter review remain outstanding.

The CLI refuses proof generation for current profiles unless the caller supplies `--allow-unreviewed-parameters`.

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
```

Observed results:

- `Cargo.lock` was generated and is committed in this source tree.
- `ogunedo-core` native tests passed: 8 tests.
- `ogunedo-core` clippy passed with `-D warnings`.
- `ogunedo-core` also compiles with default features disabled.
- The SP1 guest crate `ogunedo-program` type-checks against the pinned dependencies.
- The independent Python reference model regenerated the development fixtures.
- The offline bundle validator passed source structured-file parsing, script syntax checks, deterministic fixture regeneration, dev and draft NTT-vs-schoolbook checks, matrix-expander determinism checks, and SP1 version-pin checks.

Machine-readable offline validation results are in [`VALIDATION_REPORT.json`](VALIDATION_REPORT.json).

`bash scripts/release_check.sh` was also attempted. The wrapper did not complete on this Windows/WSL-mixed host because the Bash environment cannot see the Windows Rust installation. The underlying commands in the wrapper were run directly and are listed above.

## Blocked on this Windows host

The following commands were attempted and failed because the SP1 host-side dependency `sp1-jit` does not compile for this Windows `x86_64-pc-windows-gnullvm` environment:

```text
cargo check --workspace --all-targets --all-features --locked
cargo check -p ogunedo-cli --locked
cargo test --workspace --all-features --locked
```

The failure is in the upstream `sp1-jit` crate, which imports POSIX APIs such as `std::os::fd`, `ftruncate`, `madvise`, POSIX semaphores, and `shm_open`. Those APIs are unavailable on this Windows target. This is recorded as an environment/toolchain support boundary, not as evidence of successful SP1 proving.

## Not completed locally

These production gates have **not** been claimed as completed:

- local SP1 toolchain installation through `cargo prove`;
- SP1 guest ELF generation through the SP1 build toolchain;
- SP1 execution of the relation;
- compressed proof generation;
- Groth16 proof generation;
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

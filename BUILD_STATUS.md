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
- `network-prove` now writes a pending sanitized receipt with the request ID and explorer URL immediately after submission, before waiting for proof fulfillment;
- network proof saving is atomic and followed by a fresh credential-free verifier subprocess.

Additional WSL network-path evidence collected on 2026-07-25:

- Ubuntu WSL2 was used because the Windows host target cannot compile upstream `sp1-jit`.
- `protobuf-compiler` was installed in WSL after `sp1-prover-types` failed to find `protoc`; `protoc --version` reported `libprotoc 3.21.12`.
- `bash scripts/install_sp1.sh` completed and installed `cargo-prove sp1 (150e629 2026-05-23T01:30:54.499183867Z)` plus the `succinct` Rust toolchain.
- `RUSTUP_TOOLCHAIN=succinct rustc --version` reported `rustc 1.93.0-dev`.
- The first WSL `network-estimate` compile exposed a transitive dependency mismatch: `alloy-consensus 1.0.30` accepted `alloy-tx-macros 1.8.3`, whose derive output did not match the older `alloy-consensus` trait surface.
- The source now pins `alloy-tx-macros = "=1.0.30"` in `ogunedo-cli`; `Cargo.lock` was regenerated and committed.
- The SP1 guest ELF build completed through `sp1-build`; the build log reported `ogunedo-program built at 2026-07-25 12:51:06`.
- `network-estimate` completed successfully in WSL with `CARGO_BUILD_JOBS=2` and wrote the ignored sanitized preflight report `artifacts/network-preflight.json`.
- The successful preflight used the public benchmark fixture, proof mode `compressed`, SP1 SDK `6.2.2`, circuit version `v6.1.0`, and reported maximum possible spend `0.259605000005886633 PROVE`.
- The preflight reported requester balance `0` atomic PROVE. No private key was printed, recorded, staged, packaged, or committed.
- A guarded `network-prove` attempt was launched with the exact approval phrase from the preflight. It ran from `2026-07-25T13:04:51+01:00` to `2026-07-25T13:30:40+01:00`, consumed high local CPU before emitting a request ID, and was interrupted to protect the laptop.
- The stopped `network-prove` attempt exited `130` and produced no proof file, no manifest, no receipt, and no explorer/request ID. This is recorded as a local requester/setup resource boundary, not as a protocol failure and not as a failed remote proof.
- After hardening the receipt lifecycle, WSL `cargo check -p ogunedo-cli --locked` completed successfully with `CARGO_BUILD_JOBS=2` in `9m14s`. This validates the network receipt changes at compile time on the Linux SP1 host path.
- A fresh `network-estimate` completed successfully after the receipt hardening and wrote an ignored sanitized preflight report. It reported maximum possible spend `0.258933000005886633 PROVE` and requester balance `0` atomic PROVE.
- `network-prove` was run with the exact refreshed preflight phrase only to validate the fail-closed balance guard. It refused before submission with `requester network balance (0 atomic PROVE) is below maximum possible spend (258933000005886633 atomic PROVE); paid request not submitted`. No proof, manifest, receipt, explorer URL, or request ID was created by this guarded refusal.

Peak resource observations from the WSL network-path attempts:

- lowest observed Windows free physical RAM during WSL estimate/prove work: approximately `0.564 GiB`;
- WSL memory remained healthy, with approximately `3.5 GiB` or more available during the CPU-heavy proof attempt;
- WSL swap was unused except for a negligible transient `44 KiB` during compilation;
- lowest observed free disk on `C:` after SP1 installation/build artifacts: approximately `18.039 GiB`;
- highest observed single `rustc` RSS during the host/guest build path: approximately `1.72 GiB`;
- observed `network-prove` requester process RSS: approximately `399 MiB`;
- observed `network-prove` requester CPU before interruption: approximately `770%`, indicating local CPU-heavy setup before request evidence was written.
- latest post-check Windows snapshot: `1.006 GiB` free physical RAM, `13.472 GiB` free virtual memory, `18.039 GiB` free disk on `C:`.
- latest WSL snapshot: `5.1 GiB` available memory, `2.0 GiB` swap free, and `19 GiB` available on `/mnt/c`.
- latest post-balance-gate Windows snapshot: `3.647 GiB` free physical RAM, `16.601 GiB` free virtual memory, `18.157 GiB` free disk on `C:`.
- pre-submit compressed request snapshot: `4.103 GiB` free Windows physical RAM, `19.144 GiB` free virtual memory, `21.159 GiB` free disk on `C:`, WSL `5.1 GiB` available memory, and WSL swap free.
- compressed requester verification peak: child verifier RSS approximately `3,785,148 KiB`, WSL free memory approximately `78 MiB`, WSL available memory approximately `1.5 GiB`, and Windows free physical RAM approximately `0.765 GiB`; the process was interrupted to protect the laptop.
- credential-free verification peak: verifier RSS approximately `3,919,668 KiB`, WSL free memory approximately `78 MiB`, WSL available memory approximately `1.5 GiB`; the process was interrupted to protect the laptop.
- post-stop recovery snapshot: `2.267 GiB` free Windows physical RAM, `12.742 GiB` free virtual memory, `21.068 GiB` free disk on `C:`, WSL `5.2 GiB` available memory, and WSL swap free except for a negligible `72 KiB`.

## VPS compressed proof verification

The local laptop verification boundary above was superseded on 2026-07-26 by an independent Linux VPS verification run on `ogunedo-vps`, with wallet credentials kept off the VPS.

Compressed network request evidence:

- request ID: `0xe574b1b1770f22b015aea357466f539d0d5e899b2452640a430aa544a9398da6`;
- explorer: `https://explorer.succinct.xyz/request/0xe574b1b1770f22b015aea357466f539d0d5e899b2452640a430aa544a9398da6`;
- downloaded proof path: `proofs/development-compressed.bin`;
- downloaded proof size: `1,272,673` bytes;
- downloaded proof SHA-256: `32971efa5337fcefe0094228a99459cbcf0f227ee1068359e184182645ba8474`;
- public statement SHA-256: `e80c9731bd18dd6cf5aef618e05bbdfb2ed48636d893433c83e6db7c32de6210`;
- statement digest: `0x7f72d8f19ef1429a3a9dbd527301815794af4d87460720501835788555535ce1`;
- SP1 vkey: `0x00802c99c8f0a957ff88e97091fea717ca15a6f51390b4a721cbb23cee0d81a1`;
- submitted guest ELF SHA-256: `fd2668813dd45f63e0675d21e1accb60aa928befc7717583e2c3728dcefebd45`.

The initial VPS rebuild exposed a reproducibility gap: the SP1 guest ELF embedded absolute Cargo registry paths, so `/root/.cargo/...` and `/home/ogunedo/.cargo/...` builds produced different ELF hashes and vkeys. `script/build.rs` now passes a `--remap-path-prefix=<CARGO_HOME>=/root/.cargo` rustflag to the SP1 guest build. After that fix, the VPS-built guest ELF matched the submitted network ELF byte-for-byte and derived the expected vkey.

Verified VPS commands and results:

- baseline source validation passed in `artifacts/vps-baseline.log`;
- remapped release build passed in `artifacts/vps-release-build-remap.log`, peak RSS `2,646,700 KiB`, zero swaps;
- final hash-bound release build passed in `artifacts/vps-release-build-hashbind.log`, peak RSS `2,553,176 KiB`, zero swaps;
- first credential-free compressed verification passed in `artifacts/vps-verification-first.log`, peak RSS `7,327,528 KiB`, zero swaps;
- second fresh-process credential-free verification passed in `artifacts/vps-verification-fresh.log`, peak RSS `7,339,880 KiB`, zero swaps;
- fully-bound verification passed in `artifacts/vps-verification-fully-bound.log`, with proof SHA-256, proof size, statement SHA-256, proof mode, and vkey all specified; peak RSS `7,321,424 KiB`, zero swaps;
- adversarial verification passed in `artifacts/vps-adversarial-verification.json`, covering 18 malformed or mismatched cases; peak RSS `7,285,648 KiB`, zero swaps.

The adversarial suite initially found that a truncated-by-one and trailing-byte proof file could still decode to a valid SP1 proof object. The verifier now supports explicit `--expected-proof-sha256`, `--expected-proof-size-bytes`, and `--expected-statement-sha256` bindings and rejects symlink proof/statement inputs. With those release bindings enabled, mutated proofs, truncated proofs, trailing-byte proofs, empty proofs, proof symlinks, oversized sparse proofs, statement mutations, wrong mode, wrong vkey, and manifest mismatches all reject as expected.

Machine-readable summaries are in ignored local evidence files:

- `artifacts/vps-program-identity.json`;
- `artifacts/vps-verification-summary.json`;
- `artifacts/vps-adversarial-verification.json`.

## Not completed locally

These production gates have **not** been claimed as completed:

- SP1 execution of the relation;
- Groth16 proof generation;
- successful Succinct Prover Network Groth16 paid request;
- production benchmarks;
- estimator run for the draft module-ISIS profile;
- GitHub Actions execution;
- pull request creation;
- release candidate tag creation.

`cargo prove` is now installed in WSL through the pinned SP1 installer. `gh` was not used by this local status run.

## Cryptographic deployment status

Ogunedo remains a production-oriented research implementation, not an audited production cryptographic primitive.

- no registered parameter profile has `ProductionApproved` status;
- the draft profile is `CryptanalysisRequired`;
- the determinant-one Keller compiler is not implemented in this v1 source tree;
- external audit, public cryptanalysis, and parameter review remain outstanding.

The CLI refuses proof generation for current profiles unless the caller supplies `--allow-unreviewed-parameters`.

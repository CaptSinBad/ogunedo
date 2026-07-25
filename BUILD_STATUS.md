# Build status

**Repository:** `CaptSinBad/ogunedo`  
**Prepared for:** Ifeanyi Joseph Ogunedo  
**Version:** 0.1.0-unreleased  
**Backend pin:** SP1 6.2.2  

## Completed offline

- All TOML, JSON, and GitHub Actions YAML files parse successfully.
- All shell scripts pass `bash -n`.
- All Python scripts compile successfully.
- The independent Python implementation regenerates the committed development fixtures.
- Development-profile NTT multiplication agrees with schoolbook negacyclic multiplication.
- Draft-profile `N = 256`, `q = 12289` NTT multiplication agrees with schoolbook multiplication.
- Statement, context, target, and relation-domain binding test vectors are fixed.
- Tampered targets, witnesses, and contexts are rejected by the independent model.
- Security-sensitive top-level versions are exactly pinned.

The machine-readable results are in [`VALIDATION_REPORT.json`](VALIDATION_REPORT.json).

## Not executed in the artifact environment

The artifact container had no Rust toolchain, no SP1 toolchain, and no outbound DNS. Consequently, the following have **not** been claimed as executed here:

- `cargo check`, `cargo test`, `cargo clippy`, or `cargo fmt`;
- SP1 guest compilation;
- SP1 execution, proof generation, or proof verification;
- generation of `Cargo.lock`;
- extraction and recording of the final SP1 verification-key commitment.

GitHub Actions in `.github/workflows/` perform these checks after the repository is pushed. A release is blocked by `scripts/release_check.sh` until a generated `Cargo.lock` is committed.

## Cryptographic deployment status

The proof integration is production-oriented and uses the audited SP1 backend, but **Ogunedo 0.1.0 is not approved for protecting real value**:

- no registered profile has `ProductionApproved` status;
- the draft module-ISIS profile still requires independent cryptanalysis;
- the Ogunedo relation and integration require external audit;
- clean-room proof-generation benchmarks and verification-key pinning remain open.

This distinction is enforced by the CLI: proof generation with current profiles requires `--allow-unreviewed-parameters`.

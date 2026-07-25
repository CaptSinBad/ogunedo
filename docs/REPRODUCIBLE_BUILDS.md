# Reproducible builds

## Source-level reproducibility

The source tree pins Rust dependencies in `Cargo.toml` and commits `Cargo.lock`. A release candidate must be built from a clean checkout and must run:

```bash
cargo metadata --locked --format-version 1
cargo fmt --all -- --check
cargo test -p ogunedo-core --all-features --locked
cargo clippy -p ogunedo-core --all-targets --all-features --locked -- -D warnings
python scripts/reference_check.py
python scripts/validate_bundle.py
```

## SP1 build reproducibility

SP1 proof artifacts are not reproducible from this Windows host because the SP1 host CLI dependency stack requires POSIX APIs. The reproducible SP1 build path is the Linux GitHub Actions workflow:

```bash
.github/workflows/sp1.yml
```

A release candidate must record:

- Rust version;
- SP1 version;
- guest ELF SHA-256;
- SP1 verification-key commitment;
- proof mode;
- proof SHA-256 and byte length;
- committed public values;
- source commit.

No file may be manually edited to invent an ELF hash, verification key, proof hash, or benchmark.

Laptop-class machines must use `scripts/local_sp1_safe.ps1` for local SP1 attempts. The runner sets `CARGO_BUILD_JOBS=2`, records memory/pagefile/disk snapshots before heavy commands, and refuses local production Groth16 when the machine is below the configured proving envelope.

## Release archive reproducibility

Source archives should be generated from Git history, not from an ad hoc filesystem copy:

```bash
git archive --format=zip --output dist/ogunedo-source.zip HEAD
git archive --format=tar.gz --output dist/ogunedo-source.tar.gz HEAD
git bundle create dist/ogunedo.git.bundle --all
```

Then compute SHA-256 checksums over the generated artifacts.

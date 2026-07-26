# Repository bootstrap

The intended GitHub repository is:

```text
CaptSinBad/ogunedo
```

From the unpacked source directory, install and authenticate the GitHub CLI, then run:

```bash
./scripts/create_github_repo.sh CaptSinBad ogunedo public
```

The script:

1. checks GitHub authentication;
2. initializes a `main` branch when needed;
3. creates the initial commit;
4. creates the new repository;
5. configures `origin` and pushes `main`.

To create a private repository instead:

```bash
./scripts/create_github_repo.sh CaptSinBad ogunedo private
```

After the first push, wait for the native and SP1 integration workflows. Do not tag a release until:

```bash
cargo generate-lockfile
cargo test -p ogunedo-core --all-features
cargo run --release -p ogunedo-cli -- execute --instance fixtures/dev-instance.json
cargo run --release -p ogunedo-cli -- prove \\
  --instance fixtures/dev-instance.json \\
  --mode compressed \\
  --output proofs/dev-compressed.bin \\
  --allow-unreviewed-parameters
cargo run --release -p ogunedo-cli -- verify \\
  --proof proofs/dev-compressed.bin \\
  --statement fixtures/dev-statement.json
cargo run --release -p ogunedo-cli -- vkey
```

Commit `Cargo.lock` and record the verification-key commitment before producing an immutable release tag.

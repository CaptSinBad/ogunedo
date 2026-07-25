# Contributing

All changes to relation semantics require specification updates and test vectors.

Before submitting changes:

```bash
cargo fmt --all -- --check
cargo clippy -p ogunedo-core --all-targets --all-features -- -D warnings
cargo test -p ogunedo-core --all-features
python3 scripts/reference_check.py
git diff --exit-code fixtures/
```

Security-sensitive dependency upgrades must be isolated in their own pull request and document upstream release notes and audit implications.

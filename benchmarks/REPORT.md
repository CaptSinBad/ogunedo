# Benchmark report

No SP1 execution or proof-generation benchmark was completed on this Windows host.

Local evidence collected:

```text
cargo test -p ogunedo-core --all-features --locked: passed, 8 tests
cargo clippy -p ogunedo-core --all-targets --all-features --locked -- -D warnings: passed
cargo check -p ogunedo-program --locked: passed
python scripts\validate_bundle.py: passed
```

Blocked measurements:

- guest cycle count;
- host execution time;
- proof-generation wall time;
- proof-verification wall time;
- peak proving memory;
- guest ELF size;
- proof size;
- public-value size from real SP1 execution.

These must be produced by the Linux SP1 workflow before any production release candidate is tagged.

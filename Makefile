.PHONY: reference test fmt clippy check execute proof release-check

reference:
	python3 scripts/reference_check.py

test:
	cargo test -p ogunedo-core --all-features

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy -p ogunedo-core --all-targets --all-features -- -D warnings

check: reference fmt clippy test

execute:
	cargo run --release -p ogunedo-cli -- execute --instance fixtures/dev-instance.json

proof:
	cargo run --release -p ogunedo-cli -- prove --instance fixtures/dev-instance.json --mode compressed --output proofs/dev-compressed.bin --allow-unreviewed-parameters

release-check:
	bash scripts/release_check.sh

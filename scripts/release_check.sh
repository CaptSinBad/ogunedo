#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy -p ogunedo-core --all-targets --all-features -- -D warnings
cargo test -p ogunedo-core --all-features
python3 scripts/reference_check.py
git diff --exit-code -- fixtures/

if [ ! -f Cargo.lock ]; then
  echo "Cargo.lock is missing. Generate and commit it in the pinned online build environment." >&2
  exit 1
fi

if grep -R "ProductionApproved" -n crates/ogunedo-core/src/params.rs | grep -v "enum" >/dev/null; then
  echo "Review parameter promotion evidence before release." >&2
fi

echo "Native release checks passed. SP1 execution/proof and vkey recording remain mandatory."

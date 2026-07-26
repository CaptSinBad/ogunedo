#!/usr/bin/env bash
set -euo pipefail

CARGO_BIN="${CARGO:-cargo}"
if ! command -v "$CARGO_BIN" >/dev/null 2>&1; then
  if command -v powershell.exe >/dev/null 2>&1; then
    CARGO_BIN="$(powershell.exe -NoProfile -Command "(Get-Command cargo).Source" | tr -d '\r')"
    if command -v cygpath >/dev/null 2>&1; then
      CARGO_BIN="$(cygpath -u "$CARGO_BIN")"
    fi
  fi
fi
case "$CARGO_BIN" in
  [A-Za-z]:\\*)
    drive="$(printf '%s' "${CARGO_BIN:0:1}" | tr '[:upper:]' '[:lower:]')"
    rest="${CARGO_BIN:2}"
    rest="${rest//\\//}"
    CARGO_BIN="/${drive}${rest}"
    ;;
esac

"$CARGO_BIN" fmt --all -- --check
"$CARGO_BIN" clippy -p ogunedo-core --all-targets --all-features -- -D warnings
"$CARGO_BIN" test -p ogunedo-core --all-features
PYTHON_BIN="${PYTHON:-python3}"
if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  PYTHON_BIN="python"
fi
"$PYTHON_BIN" scripts/reference_check.py
git diff --exit-code -- fixtures/

if [ ! -f Cargo.lock ]; then
  echo "Cargo.lock is missing. Generate and commit it in the pinned online build environment." >&2
  exit 1
fi

if grep -R "ProductionApproved" -n crates/ogunedo-core/src/params.rs | grep -v "enum" >/dev/null; then
  echo "Review parameter promotion evidence before release." >&2
fi

echo "Native release checks passed. SP1 execution/proof and vkey recording remain mandatory."

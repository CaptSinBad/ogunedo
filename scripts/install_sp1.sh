#!/usr/bin/env bash
set -euo pipefail

# Review the upstream installer before running it in a sensitive environment.
curl --proto '=https' --tlsv1.2 -sSfL https://sp1up.succinct.xyz | bash
export PATH="$HOME/.sp1/bin:$PATH"
sp1up --version v6.2.2
cargo prove --version

#!/usr/bin/env bash

# Regenerate all test cases.
#
# CAUTION: you _must_ manually review the output before committing after running this script.

set -e

# execute from the repo root, no matter where the script was invoked from
cd "$(dirname "$0")"
cd "$(git rev-parse --show-toplevel)"

cargo build --release --all-features
for definition in tests/cases/*/definition.yaml; do
    case_dir="$(dirname "$definition")"
    target/release/openapi-gen --no-emit-docs "$definition" > "$case_dir/expect.rs"
done

./scripts/compile_test_cases.sh

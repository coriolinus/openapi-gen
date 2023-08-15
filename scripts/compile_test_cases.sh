#!/usr/bin/env bash

# Compile all test cases.
#
# This only compiles the expected value, but that should be sufficient to validate that the generated code is plausible.

set -euo pipefail

# execute from the repo root, no matter where the script was invoked from
cd "$(dirname "$0")"
cd "$(git rev-parse --show-toplevel)"

repo_path="$(realpath "$PWD")"

tmpdir="$(mktemp -d --tmpdir "compile-tests.XXXXXX")"
cd "$tmpdir"
cargo init --name "compile-tests" --lib >/dev/null 2>&1
cargo add openapi-gen --path "$repo_path" --features api-problem,axum-support,bytes,integer-restrictions,string-pattern,uuid >/dev/null 2>&1

exit_code=0

for case in "$repo_path"/tests/cases/*/expect.rs; do
    file="$(realpath "$case")"
    case_name="$(basename "$(dirname "$file")")"
    echo "$case_name..."
    cp "$file" src/lib.rs
    if cargo check --quiet --lib; then
      echo "  OK!"
    else
        exit_code=1
    fi
done

cd "$repo_path"
rm -rf "$tmpdir"

exit $exit_code

#!/usr/bin/env bash
set -euo pipefail

# Sync canonical cross-language test vectors into the Rust crate.
#
# Canonical vectors live in <repo>/spec/test-vectors. Cargo packages cannot
# include files outside the crate directory, so pulselink-protocol keeps a copy
# for its own tests.

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rust_root="$(cd "${script_dir}/.." && pwd)"
repo_root="$(cd "${rust_root}/.." && pwd)"

src="${repo_root}/spec/test-vectors"
dst="${rust_root}/crates/pulselink-protocol/tests/vectors"

if [[ ! -d "${src}" ]]; then
    echo "error: canonical test vector directory not found: ${src}" >&2
    exit 1
fi

mkdir -p "${dst}"
cp -R "${src}/." "${dst}/"

echo "synced ${src} -> ${dst}"

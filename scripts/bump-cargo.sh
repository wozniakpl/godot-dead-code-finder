#!/usr/bin/env bash
# Update Cargo.toml version. Called by semantic-release @semantic-release/exec with next_release_version set.
set -e
if [ -z "${next_release_version}" ]; then
  echo "next_release_version not set" >&2
  exit 1
fi
# Strip leading 'v' if present (semantic-release may output v1.2.3)
ver="${next_release_version#v}"
sed -i.bak "s/^version = .*/version = \"${ver}\"/" Cargo.toml && rm -f Cargo.toml.bak

#!/usr/bin/env bash
# Update Cargo.toml version. Called by semantic-release @semantic-release/exec.
# Plugin may set next_release_version or NEXT_RELEASE_VERSION (see @semantic-release/exec).
set -e
ver="${next_release_version:-${NEXT_RELEASE_VERSION:-$1}}"
if [ -z "$ver" ]; then
  echo "next_release_version / NEXT_RELEASE_VERSION not set and no argument given" >&2
  exit 1
fi
# Strip leading 'v' if present (e.g. v1.2.3)
ver="${ver#v}"
sed -i.bak "s/^version = .*/version = \"${ver}\"/" Cargo.toml && rm -f Cargo.toml.bak

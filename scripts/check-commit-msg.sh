#!/usr/bin/env bash
# Validate conventional commit message (first line).
# Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci
# Format: type(scope)?: description
# Allow: Merge branch, Revert "...", etc. (for merges/reverts)

set -e
msg_file="${1:?missing commit message file}"
first_line=$(head -n1 "$msg_file")

# Skip merge commits, revert, and empty first line
skip_pattern='^Merge |^Revert |^$'
if [[ "$first_line" =~ $skip_pattern ]]; then
  exit 0
fi

# Conventional commit: type(scope)?: description
# type = feat|fix|docs|style|refactor|perf|test|chore|build|ci
# scope = optional, in parens
# description = non-empty
pattern='^(feat|fix|docs|style|refactor|perf|test|chore|build|ci)(\([a-zA-Z0-9_-]+\))?!?:\ .+'
if [[ "$first_line" =~ $pattern ]]; then
  exit 0
fi

echo "Invalid commit message. Use conventional commits: type(scope)?: description" >&2
echo "Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci" >&2
echo "Example: feat(cli): add --quiet flag" >&2
echo "Got: $first_line" >&2
exit 1

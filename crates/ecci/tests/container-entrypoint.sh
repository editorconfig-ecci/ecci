#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/workspace"
if env GITHUB_WORKSPACE="$tmp/workspace" ECCI_BIN=/bin/true INPUT_PATHS=. \
  INPUT_WORKING_DIRECTORY=. INPUT_FAIL_ON_VIOLATION=TRUE INPUT_ANNOTATIONS=true \
  INPUT_SUMMARY=true INPUT_MAX_ANNOTATIONS=50 INPUT_LOG_LEVEL=summary \
  "$root/entrypoint.sh" >"$tmp/stdout" 2>&1; then
  echo 'entrypoint unexpectedly accepted invalid input' >&2
  exit 1
fi
grep '::error title=ECCI-CONFIG::' "$tmp/stdout" >/dev/null
env GITHUB_WORKSPACE="$tmp/workspace" ECCI_BIN=/bin/true INPUT_PATHS='one
two' INPUT_WORKING_DIRECTORY=. INPUT_FAIL_ON_VIOLATION=false INPUT_ANNOTATIONS=false \
  INPUT_SUMMARY=false INPUT_MAX_ANNOTATIONS=0 INPUT_LOG_LEVEL=quiet "$root/entrypoint.sh"

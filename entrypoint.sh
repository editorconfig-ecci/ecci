#!/bin/sh
set -eu

config_error() {
  escaped=$(printf '%s' "$1" | sed -e 's/%/%25/g' -e 's/\r/%0D/g' -e 's/\n/%0A/g')
  printf '::error title=config.invalid::%s\n' "$escaped"
  if [ -n "${GITHUB_OUTPUT-}" ]; then
    printf 'outcome=configuration-error\nviolations=0\nchecked-files=0\nskipped-files=0\n' >> "$GITHUB_OUTPUT"
  fi
  if [ "${INPUT_SUMMARY-}" != false ] && [ -n "${GITHUB_STEP_SUMMARY-}" ]; then
    printf '## ecci\n\n**Outcome:** configuration-error\n\n%s\n' "$1" >> "$GITHUB_STEP_SUMMARY"
  fi
  exit 2
}

validate_boolean() {
  eval "value=\${$1-}"
  case "$value" in true|false) ;; *) config_error "$2 must be exactly true or false" ;; esac
}

validate_boolean INPUT_FAIL_ON_VIOLATION fail-on-violation
validate_boolean INPUT_ANNOTATIONS annotations
validate_boolean INPUT_SUMMARY summary
case "${INPUT_MAX_ANNOTATIONS-}" in ''|*[!0-9]*) config_error 'max-annotations must be a non-negative decimal integer' ;; esac
case "${INPUT_LOG_LEVEL-}" in quiet|summary|diagnostic|debug) ;; *) config_error 'log-level must be quiet, summary, diagnostic, or debug' ;; esac
[ -n "${INPUT_PATHS-}" ] || config_error 'paths must contain at least one non-empty line'
[ -n "${INPUT_WORKING_DIRECTORY-}" ] || config_error 'working-directory must not be empty'

exec "${ECCI_BIN:-/usr/local/bin/ecci}" --github-action

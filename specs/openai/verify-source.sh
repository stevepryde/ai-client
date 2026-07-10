#!/bin/sh
set -eu

EXPECTED_URL='https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml'
EXPECTED_VERSION='2.3.0'
EXPECTED_BYTES='3520114'
EXPECTED_ETAG='20b52b62cb42863a87ce612936087e89f9ccfd79'
EXPECTED_SHA256='ffb722eb86b2382e760f840d53d5888f4086fd1aa4aaade8ad522ad69e3e6afe'

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
PINNED_FILE="$SCRIPT_DIR/openapi.documented.yml"
TEMP_DIR=''

cleanup() {
  if [ -n "$TEMP_DIR" ]; then
    rm -rf "$TEMP_DIR"
  fi
}
trap cleanup EXIT HUP INT TERM

verify_file() {
  file=$1
  actual_bytes=$(wc -c < "$file" | tr -d '[:space:]')
  if command -v shasum >/dev/null 2>&1; then
    actual_sha256=$(shasum -a 256 "$file" | awk '{print $1}')
  elif command -v sha256sum >/dev/null 2>&1; then
    actual_sha256=$(sha256sum "$file" | awk '{print $1}')
  else
    echo 'cannot verify SHA-256: install shasum or sha256sum' >&2
    exit 1
  fi
  actual_version=$(sed -n '1,20p' "$file" | awk '$1 == "version:" {print $2; exit}')

  [ "$actual_bytes" = "$EXPECTED_BYTES" ] || {
    echo "byte length mismatch: expected $EXPECTED_BYTES, got $actual_bytes" >&2
    exit 1
  }
  [ "$actual_sha256" = "$EXPECTED_SHA256" ] || {
    echo "SHA-256 mismatch: expected $EXPECTED_SHA256, got $actual_sha256" >&2
    exit 1
  }
  [ "$actual_version" = "$EXPECTED_VERSION" ] || {
    echo "API version mismatch: expected $EXPECTED_VERSION, got $actual_version" >&2
    exit 1
  }
}

verify_file "$PINNED_FILE"

if [ "${1:-}" = '--remote' ]; then
  TEMP_DIR=$(mktemp -d)
  curl -fsSL -D "$TEMP_DIR/headers" -o "$TEMP_DIR/openapi.yml" "$EXPECTED_URL"
  actual_etag=$(grep -i '^etag:' "$TEMP_DIR/headers" | tail -1 | cut -d: -f2- | tr -d ' "\r')
  [ "$actual_etag" = "$EXPECTED_ETAG" ] || {
    echo "ETag mismatch: expected $EXPECTED_ETAG, got $actual_etag" >&2
    exit 1
  }
  verify_file "$TEMP_DIR/openapi.yml"
  cmp -s "$PINNED_FILE" "$TEMP_DIR/openapi.yml" || {
    echo 'remote document differs from the pinned document' >&2
    exit 1
  }
fi

echo "verified OpenAI OpenAPI $EXPECTED_VERSION ($EXPECTED_BYTES bytes, sha256 $EXPECTED_SHA256)"

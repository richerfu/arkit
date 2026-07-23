#!/usr/bin/env bash
# Prefetch Ghostty Zig package tarballs into the Zig global cache.
#
# deps.files.ghostty.org rejects bare clients (HTTP 400). Fetching with a
# browser User-Agent and then `zig fetch` on the local file works around that.
#
# Usage:
#   ./scripts/fetch-ghostty-deps.sh
#   GHOSTTY_SRC=/path/to/ghostty ./scripts/fetch-ghostty-deps.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GHOSTTY_SRC="${GHOSTTY_SRC:-${1:-$ROOT/vendor/ghostty}}"
if [[ -z "$GHOSTTY_SRC" || ! -d "$GHOSTTY_SRC" ]]; then
  echo "usage: GHOSTTY_SRC=/path/to/ghostty $0" >&2
  exit 1
fi

ZIG="${ZIG:-$(command -v zig || true)}"
if [[ -z "$ZIG" ]]; then
  echo "zig not found; set ZIG= or add zig to PATH" >&2
  exit 1
fi
ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR:-${HOME}/.cache/zig-ghostty}"
DEPS_DIR="${DEPS_DIR:-${TMPDIR:-/tmp}/ghostty-deps}"
UA="${CURL_UA:-Mozilla/5.0 (compatible; arkit-terminal-build)}"

mkdir -p "$DEPS_DIR" "$ZIG_GLOBAL_CACHE_DIR"
export ZIG_GLOBAL_CACHE_DIR

# Portable (bash 3 / macOS): no mapfile.
URLS="$(grep -rhoE 'https://deps\.files\.ghostty\.org/[^"]+' "$GHOSTTY_SRC" --include='*.zon' 2>/dev/null | sort -u || true)"
count=0
if [[ -n "$URLS" ]]; then
  count="$(printf '%s\n' "$URLS" | grep -c . || true)"
fi
echo "prefetching ${count} ghostty package URLs into $ZIG_GLOBAL_CACHE_DIR"

ok=0
fail=0
# Zig 0.16 `zig fetch` on a bare path needs a package context (build.zig.zon
# nearby); run from the Ghostty tree so hashes land in the global cache.
pushd "$GHOSTTY_SRC" >/dev/null
if [[ -n "$URLS" ]]; then
  while IFS= read -r url; do
    [[ -z "$url" ]] && continue
    name=$(basename "$url")
    out="$DEPS_DIR/$name"
    if [[ ! -s "$out" ]]; then
      code=$(curl -sL -A "$UA" -o "$out" "$url" -w '%{http_code}')
      if [[ "$code" != "200" ]]; then
        echo "FAIL $code $url" >&2
        rm -f "$out"
        fail=$((fail + 1))
        continue
      fi
    fi
    if "$ZIG" fetch --global-cache-dir "$ZIG_GLOBAL_CACHE_DIR" "$out" >/dev/null; then
      ok=$((ok + 1))
    else
      echo "zig fetch failed for $name" >&2
      fail=$((fail + 1))
    fi
  done <<EOF
$URLS
EOF
fi
popd >/dev/null

echo "done ok=$ok fail=$fail"
if [[ $fail -eq 0 ]]; then
  exit 0
fi
exit 1

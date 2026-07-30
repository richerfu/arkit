#!/usr/bin/env bash
# Build libghostty-vt with the OHOS-capable Zig toolchain.
#
# Ghostty sources default to the git submodule / packaged tree at
#   crates/arkit_terminal/vendor/ghostty
#
# Optional env:
#   GHOSTTY_SRC   — override Ghostty source tree
#   ZIG           — zig binary (default: zig on PATH)
#   ZIG_TARGET    — e.g. aarch64-linux-ohos
#   OUT_DIR       — install prefix (default: ./out)
#   ZIG_GLOBAL_CACHE_DIR — package cache (default ~/.cache/zig-ghostty)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEFAULT_SRC="$ROOT/vendor/ghostty"
GHOSTTY_SRC="${GHOSTTY_SRC:-$DEFAULT_SRC}"
ZIG="${ZIG:-$(command -v zig || true)}"
OUT_DIR="${OUT_DIR:-$ROOT/out}"
ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR:-${HOME}/.cache/zig-ghostty}"
export ZIG_GLOBAL_CACHE_DIR

if [[ ! -f "$GHOSTTY_SRC/build.zig" ]]; then
  cat >&2 <<EOF
ghostty sources not found at:
  $GHOSTTY_SRC

Init the submodule (dev checkout):
  git submodule update --init --recursive crates/arkit_terminal/vendor/ghostty

Or set GHOSTTY_SRC=/path/to/ghostty
EOF
  exit 1
fi

if [[ -z "$ZIG" || ! -x "$ZIG" ]]; then
  echo "zig not found; set ZIG= or add zig to PATH" >&2
  exit 1
fi

# Prefetch deps (idempotent). CDN rejects bare clients — script uses a UA.
"$ROOT/scripts/fetch-ghostty-deps.sh" "$GHOSTTY_SRC" || true

# Build from Ghostty itself for a reliable artifact layout.
cd "$GHOSTTY_SRC"
if [[ -n "${ZIG_TARGET:-}" ]]; then
  "$ZIG" build -Demit-lib-vt=true -Doptimize=ReleaseSafe \
    --global-cache-dir "$ZIG_GLOBAL_CACHE_DIR" \
    -Dtarget="$ZIG_TARGET" \
    --prefix "$OUT_DIR"
else
  "$ZIG" build -Demit-lib-vt=true -Doptimize=ReleaseSafe \
    --global-cache-dir "$ZIG_GLOBAL_CACHE_DIR" \
    --prefix "$OUT_DIR"
fi

echo "installed under $OUT_DIR"
find "$OUT_DIR" -name '*ghostty-vt*' 2>/dev/null | head -20

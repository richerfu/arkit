#!/usr/bin/env bash
# 把指定 arkit example 打包成 hap 安装到 OpenHarmony 模拟器并启动。
#
# 用法: ./run.sh <example-dir> [install|build|start|log]
#   example-dir: counter | async_task | animation | camera | chart | complex_cases | i18n | lottie | router | shadcn_showcase | webview
#
# 每个 example 的 .so 名 = lib<crate-name>.so（crate-name 取自 examples/<dir>/Cargo.toml）。
# 切换 example 时同步更新 app 壳的 moduleName / lib 依赖 / cpp/types，保持名字一致。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/app"
HVIGWORW="/Users/ranger/Downloads/command-line-tools/bin/hvigorw"
OHPM="/Users/ranger/Downloads/command-line-tools/bin/ohpm"
BUNDLE="com.arkit.example"
ABILITY="EntryAbility"
HDC_TARGET="${HDC_TARGET:-$(hdc list targets -v 2>/dev/null | sed -n '1s/[[:space:]].*//p')}"
HDC=(hdc)
if [ -n "$HDC_TARGET" ]; then
  HDC+=(-t "$HDC_TARGET")
fi

# hdc can transiently report "Connect server failed" with a zero exit code
# after a longer hvigor build. Retry it and turn that text failure into a real
# shell failure so `all` never claims a deployment that did not happen.
run_hdc() {
  local output=""
  local attempt
  for attempt in 1 2 3; do
    output=$("${HDC[@]}" "$@" 2>&1) || true
    if [[ "$output" != *"Connect server failed"* ]]; then
      printf '%s\n' "$output"
      return 0
    fi
  done
  printf '%s\n' "$output" >&2
  return 1
}

EX="${1:?usage: $0 <example-dir> [build|install|start|log]}"
ACTION="${2:-all}"

# 解析 example 的 crate 名（= .so 名）
CARGO_TOML="$ROOT/examples/$EX/Cargo.toml"
[ -f "$CARGO_TOML" ] || { echo "example not found: $EX"; exit 1; }
CRATE=$(grep '^name' "$CARGO_TOML" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
SO_SRC="$ROOT/examples/$EX/dist/arm64-v8a/lib${CRATE}.so"
DTS_SRC="$ROOT/examples/$EX/dist/index.d.ts"
[ -f "$SO_SRC" ] || { echo ".so not found, build first: ohrs build --arch aarch in examples/$EX ($SO_SRC)"; exit 1; }

echo ">> example=$EX crate=$CRATE"

# 1) 同步 .so + d.ts + moduleName + oh-package 依赖到 app 壳
sync_shell() {
  echo ">> syncing lib${CRATE}.so + types into app shell"
  # 清旧 types，建新
  rm -rf "$APP/entry/src/main/cpp/types"/lib*
  mkdir -p "$APP/entry/src/main/cpp/types/lib${CRATE}"
  cp "$DTS_SRC" "$APP/entry/src/main/cpp/types/lib${CRATE}/Index.d.ts"
  cat > "$APP/entry/src/main/cpp/types/lib${CRATE}/oh-package.json5" <<EOF
{
  "name": "lib${CRATE}.so",
  "types": "./Index.d.ts",
  "version": "1.0.0",
  "description": "arkit ${EX} example napi bindings"
}
EOF
  # .so — 放 entry/libs/arm64-v8a/（hvigor 默认打包此目录的预编译 native 库）
  rm -f "$APP/entry/libs/arm64-v8a"/lib*.so
  mkdir -p "$APP/entry/libs/arm64-v8a"
  cp "$SO_SRC" "$APP/entry/libs/arm64-v8a/lib${CRATE}.so"
  # Native libraries linked against OHOS libc++ cannot resolve the system copy
  # from an application's module namespace. Bundle the SDK's matching shared
  # runtime whenever the example cdylib declares it as a dependency.
  if [ -n "${OHOS_NDK_HOME:-}" ] && \
    "$OHOS_NDK_HOME/native/llvm/bin/llvm-readelf" -d "$SO_SRC" | \
      grep -q '\[libc++_shared\.so\]'; then
    CXX_SHARED="$OHOS_NDK_HOME/native/llvm/lib/aarch64-linux-ohos/libc++_shared.so"
    [ -f "$CXX_SHARED" ] || { echo "OHOS libc++ runtime not found: $CXX_SHARED"; exit 1; }
    cp "$CXX_SHARED" "$APP/entry/libs/arm64-v8a/libc++_shared.so"
  fi
  # entry oh-package.json5 的 lib 依赖
  cat > "$APP/entry/oh-package.json5" <<EOF
{
  "name": "entry",
  "version": "1.0.0",
  "description": "arkit example entry",
  "main": "",
  "author": "",
  "license": "Apache-2.0",
  "dependencies": {
    "lib${CRATE}.so": "file:./src/main/cpp/types/lib${CRATE}",
    "@ohos-rs/ability": "0.4.0-beta.7"
  }
}
EOF
  # EntryAbility moduleName + Index 默认值
  LC_ALL=C LANG=C perl -0pi -e 's/(public moduleName: string = ")[^"]*(")/${1}'"$CRATE"'${2}/' \
    "$APP/entry/src/main/ets/entryability/EntryAbility.ets"
  LC_ALL=C LANG=C perl -0pi -e 's/(@State moduleName: string = ")[^"]*(")/${1}'"$CRATE"'${2}/' \
    "$APP/entry/src/main/ets/pages/Index.ets"
}

do_build() {
  echo ">> ohpm install"
  (cd "$APP" && "$OHPM" install)
  echo ">> hvigorw assembleHap"
  (cd "$APP" && "$HVIGWORW" assembleHap --no-daemon --mode module -p product=default -p buildMode=debug --no-hvigorw-daemon)
}

do_install() {
  HAP=$(find "$APP/entry/build" -name "*.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
  [ -n "$HAP" ] || { echo "no hap found, build first"; exit 1; }
  echo ">> hdc ${HDC_TARGET:+-t $HDC_TARGET }install $HAP"
  run_hdc install -r "$HAP" | tail -3
}

do_start() {
  echo ">> aa start $ABILITY / $BUNDLE"
  run_hdc shell aa start -a "$ABILITY" -b "$BUNDLE" | tail -3
}

case "$ACTION" in
  sync) sync_shell ;;
  build)
    sync_shell
    do_build
    ;;
  install) do_install ;;
  start) do_start ;;
  log) "${HDC[@]}" hilog | grep -iE "arkit|ArkUI|dioxus|error|fatal" ;;
  all)
    sync_shell
    do_build
    do_install
    do_start
    echo ">> deployed $EX ($CRATE). tail logs: $0 $EX log"
    ;;
  *) echo "unknown action: $ACTION"; exit 1 ;;
esac

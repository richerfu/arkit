#!/usr/bin/env bash
# 构建整合 demo（examples/demos,单一 libdemos.so,内含全部示例页面并做
# Rust 侧路由分发）,打包成 hap 安装到 OpenHarmony 模拟器并启动。
#
# 用法: ./run.sh [build|install|start|log|sync]
#   sync     仅同步 .so + d.ts + 类型包到 app 壳
#   build    sync + ohpm install + hvigor assembleHap
#   install  安装最新 hap
#   start    启动 EntryAbility
#   log      过滤查看 arkit 相关日志
#   (默认)   build + install + start
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/app"
HVIGWORW="/Users/ranger/Downloads/command-line-tools/bin/hvigorw"
OHPM="/Users/ranger/Downloads/command-line-tools/bin/ohpm"
BUNDLE="com.arkit.example"
ABILITY="EntryAbility"
HDC_TARGET="${HDC_TARGET:-}"
HDC=(hdc)
HDC_READY=0

# 整合后的单一 native 模块:全部示例编译进 examples/demos 这一个 cdylib。
EX="demos"
CRATE="demos"
SO_SRC="$ROOT/examples/$EX/dist/arm64-v8a/lib${CRATE}.so"
DTS_SRC="$ROOT/examples/$EX/dist/index.d.ts"

hdc_output_is_transient_failure() {
  local output="$1"
  [[ "$output" == *"Connect server failed"* || \
    "$output" == *"[Fail]"* || \
    "$output" == *"Device not found or connected"* ]]
}

hdc_output_is_fatal_failure() {
  local output="$1"
  [[ "$output" == *"error: failed"* || \
    "$output" == *"failed to install bundle"* || \
    "$output" == *"no signature file"* ]]
}

# hdc can transiently report a textual connection failure with exit code 0
# after a longer hvigor build. Normalize both status and output before the
# caller is allowed to report a successful deployment.
retry_hdc() {
  local attempt
  local max_attempts=5
  local output=""
  local status=0
  for ((attempt = 1; attempt <= max_attempts; attempt++)); do
    status=0
    output=$("$@" 2>&1) || status=$?
    if [ "$status" -eq 0 ] && \
      ! hdc_output_is_transient_failure "$output" && \
      ! hdc_output_is_fatal_failure "$output"; then
      [ -z "$output" ] || printf '%s\n' "$output"
      return 0
    fi
    if hdc_output_is_fatal_failure "$output"; then
      printf '%s\n' "$output" >&2
      return 1
    fi
    if [ "$attempt" -lt "$max_attempts" ]; then
      echo ">> hdc attempt $attempt failed; retrying" >&2
      sleep "$attempt"
    fi
  done
  [ -z "$output" ] || printf '%s\n' "$output" >&2
  echo "hdc command failed after $max_attempts attempts (exit $status)" >&2
  return 1
}

ensure_hdc_target() {
  [ "$HDC_READY" -eq 1 ] && return 0

  if [ -z "$HDC_TARGET" ]; then
    local target_output
    local connected_targets
    local target_count
    target_output=$(retry_hdc hdc list targets -v)
    connected_targets=$(printf '%s\n' "$target_output" | awk '$3 == "Connected" { print $1 }')
    target_count=$(printf '%s\n' "$connected_targets" | awk 'NF { count++ } END { print count + 0 }')
    case "$target_count" in
      0)
        echo "no connected hdc target; connect a device or set HDC_TARGET" >&2
        return 1
        ;;
      1)
        HDC_TARGET="$connected_targets"
        ;;
      *)
        echo "multiple connected hdc targets; set HDC_TARGET explicitly:" >&2
        printf '%s\n' "$connected_targets" | sed 's/^/  - /' >&2
        return 1
        ;;
    esac
  fi

  HDC=(hdc -t "$HDC_TARGET")
  HDC_READY=1
}

run_hdc() {
  ensure_hdc_target
  retry_hdc "${HDC[@]}" "$@"
}

ACTION="${1:-all}"

# 1) 构建 Rust 侧(如 .so 缺失),2) 同步 .so + d.ts + 类型包到 app 壳
sync_shell() {
  if [ ! -f "$SO_SRC" ]; then
    echo ">> building lib${CRATE}.so (examples/$EX)"
    (cd "$ROOT/examples/$EX" && ohrs build --arch aarch)
  fi
  [ -f "$SO_SRC" ] || { echo "build failed, .so missing: $SO_SRC"; exit 1; }

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
  # entry oh-package.json5 的 lib 依赖。
  # @ohos-rs/ability / ability-plugin-webview 走 ohpm 注册表版本
  # （1.0.0-beta.2，含 Ability-session bridge 与 render-owner 生命周期）。
  cat > "$APP/entry/oh-package.json5" <<EOF
{
  "name": "entry",
  "version": "1.0.0",
  "description": "arkit demos entry",
  "main": "",
  "author": "",
  "license": "Apache-2.0",
  "dependencies": {
    "lib${CRATE}.so": "file:./src/main/cpp/types/lib${CRATE}",
    "@ohos-rs/ability": "1.0.0-beta.2",
    "@ohos-rs/ability-plugin-webview": "1.0.0-beta.2"
  }
}
EOF
  # EntryAbility / Index 的 moduleName 已固定为 demos,无需再改写。
}

do_build() {
  echo ">> ohpm install"
  (cd "$APP" && "$OHPM" install)
  echo ">> hvigorw assembleHap"
  # hvigor 数据目录默认在 ~/.hvigor；受保护/沙箱 home 环境可用官方
  # HVIGOR_USER_HOME 环境变量重定向（hvigorw 原生支持，脚本无需感知）。
  if ! (cd "$APP" && "$HVIGWORW" assembleHap --no-daemon --mode module -p product=default -p buildMode=debug --no-hvigorw-daemon); then
    # SignHap 依赖 ~/.ohos/config 下的签名材料（build-profile.json5 的
    # signingConfigs）。材料缺失时（如被 DevEco 清理）容忍未签名产物,
    # 由 do_sign 用自签材料补签。
    UNSIGNED=$(find "$APP/entry/build" -name "*-unsigned.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
    if [ -n "$UNSIGNED" ]; then
      echo ">> SignHap failed (missing signing material); continuing with unsigned hap" >&2
      return 0
    fi
    echo ">> hvigor build failed and no unsigned hap produced" >&2
    return 1
  fi
}

# 用 OpenHarmony 自签材料补签 unsigned hap(DevEco 自动签名材料缺失时的兜底)。
do_sign() {
  UNSIGNED=$(find "$APP/entry/build" -name "*-unsigned.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
  SIGNED=$(find "$APP/entry/build" -name "*-signed.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
  if [ -n "$SIGNED" ]; then
    return 0
  fi
  [ -n "$UNSIGNED" ] || return 0
  "$ROOT/app/sign.sh" "$UNSIGNED" "${UNSIGNED%-unsigned.hap}-signed.hap"
}

do_install() {
  HAP=$(find "$APP/entry/build" -name "*-signed.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
  if [ -z "$HAP" ]; then
    HAP=$(find "$APP/entry/build" -name "*.hap" -path "*outputs*" 2>/dev/null | head -1 || true)
  fi
  [ -n "$HAP" ] || { echo "no hap found, build first"; exit 1; }
  ensure_hdc_target
  echo ">> hdc -t $HDC_TARGET install $HAP"
  run_hdc install -r "$HAP"
}

do_start() {
  echo ">> aa start $ABILITY / $BUNDLE"
  run_hdc shell aa start -a "$ABILITY" -b "$BUNDLE"
}

case "$ACTION" in
  sync) sync_shell ;;
  build)
    sync_shell
    do_build
    do_sign
    ;;
  install) do_install ;;
  start) do_start ;;
  log)
    ensure_hdc_target
    "${HDC[@]}" hilog | grep -iE "arkit|ArkUI|dioxus|error|fatal"
    ;;
  all)
    sync_shell
    do_build
    do_install
    do_start
    echo ">> deployed demos (libdemos.so). tail logs: $0 log"
    ;;
  *) echo "unknown action: $ACTION"; exit 1 ;;
esac

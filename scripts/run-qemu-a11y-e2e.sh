#!/usr/bin/env bash
# Build the demo, boot an unmodified a11y-capable ohos-qemu image, and verify
# ArkUI/shadcn semantics plus screen-reader-routed actions.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLE="com.arkit.example"
EXTENSION="AccessibilityE2ETestExtension"
HDC_PORT="${ARKIT_A11Y_HDC_PORT:-5555}"
HDC_TARGET="127.0.0.1:${HDC_PORT}"
QMP_SOCKET="${ARKIT_A11Y_QMP_SOCKET:-/tmp/arkit-qemu-a11y-${HDC_PORT}.sock}"
DIAGNOSTICS="${ARKIT_A11Y_DIAGNOSTICS:-$ROOT/.tools/qemu-a11y}"
DEVICE_LAYOUT="/data/local/tmp/arkit-a11y-layout.json"
HOST_LAYOUT="$DIAGNOSTICS/layout.json"
QEMU_LOG="$DIAGNOSTICS/qemu.log"
QEMU_PID=""
HDC=(hdc -t "$HDC_TARGET")

fail() {
  echo "a11y QEMU E2E failed: $*" >&2
  exit 1
}

cleanup() {
  if [ -n "$QEMU_PID" ] && kill -0 "$QEMU_PID" >/dev/null 2>&1; then
    kill "$QEMU_PID" >/dev/null 2>&1 || true
    wait "$QEMU_PID" 2>/dev/null || true
  fi
}

find_qemu_package() {
  local candidate manifest
  if [ -n "${OHOS_QEMU_PACKAGE:-}" ]; then
    candidate="$OHOS_QEMU_PACKAGE"
    manifest="$candidate/manifest.json"
    [ -f "$manifest" ] || fail "manifest missing from OHOS_QEMU_PACKAGE=$candidate"
    jq -e '.capabilities.accessibility_test == true and
      .capabilities.accessibility_cli == true and
      .capabilities.virtio_multitouch == true' "$manifest" >/dev/null \
      || fail "the selected QEMU package does not advertise complete a11y support"
    printf '%s\n' "$candidate"
    return
  fi

  while IFS= read -r manifest; do
    if jq -e '.capabilities.accessibility_test == true and
      .capabilities.accessibility_cli == true and
      .capabilities.virtio_multitouch == true' "$manifest" >/dev/null 2>&1; then
      dirname "$manifest"
      return
    fi
  done < <(find "$HOME/.ohos-qemu" -type f -name manifest.json 2>/dev/null | sort -r)
  fail "no installed ohos-qemu image advertises accessibility_test/accessibility_cli/virtio_multitouch"
}

wait_for_device() {
  local attempt output boot_completed
  for attempt in $(seq 1 180); do
    hdc tconn "$HDC_TARGET" >/dev/null 2>&1 || true
    output="$("${HDC[@]}" shell param get const.product.cpu.abilist 2>&1 || true)"
    boot_completed="$("${HDC[@]}" shell param get bootevent.boot.completed 2>&1 || true)"
    if [[ "$output" == *arm64* || "$output" == *aarch64* ]] \
      && [[ "$boot_completed" == *true* ]]; then
      return
    fi
    sleep 1
  done
  fail "QEMU did not expose hdc at $HDC_TARGET; see $QEMU_LOG"
}

dump_layout() {
  "${HDC[@]}" shell uitest dumpLayout -b "$BUNDLE" -p "$DEVICE_LAYOUT" >/dev/null
  "${HDC[@]}" file recv "$DEVICE_LAYOUT" "$HOST_LAYOUT" >/dev/null
  jq -e . "$HOST_LAYOUT" >/dev/null
}

wait_for_label() {
  local label="$1"
  local attempt
  for attempt in $(seq 1 40); do
    dump_layout 2>/dev/null || true
    if visible_node_exists "$label"; then
      return
    fi
    sleep 1
  done
  fail "accessibility label did not appear: $label"
}

scroll_until_label() {
  local label="$1"
  local direction="${2:-up}"
  local attempt
  for attempt in $(seq 1 12); do
    dump_layout 2>/dev/null || true
    if visible_node_exists "$label"; then
      return
    fi
    if [ "$direction" = "up" ]; then
      qemu_touch swipe 400 400 400 100 --duration-ms 400
    else
      qemu_touch swipe 400 100 400 400 --duration-ms 400
    fi
    sleep 1
  done
  fail "could not scroll to semantic node: $label"
}

node_exists() {
  local label="$1"
  jq -e --arg label "$label" '
    any(.. | objects | select(.attributes?);
      ([.attributes[] | tostring] | index($label)) != null)
  ' "$HOST_LAYOUT" >/dev/null
}

visible_node_exists() {
  local label="$1"
  jq -e --arg label "$label" '
    any(.. | objects | select(.attributes?);
      .attributes.visible == "true" and
      .attributes.bounds != "[0,0][0,0]" and
      (([.attributes[] | tostring] | index($label)) != null))
  ' "$HOST_LAYOUT" >/dev/null
}

node_attribute() {
  local label="$1"
  local attribute="$2"
  jq -r --arg label "$label" --arg attribute "$attribute" '
    first(.. | objects | select(.attributes? and
      .attributes.visible == "true" and
      .attributes.bounds != "[0,0][0,0]" and
      (([.attributes[] | tostring] | index($label)) != null))) |
    .attributes[$attribute] // empty
  ' "$HOST_LAYOUT"
}

assert_node() {
  local label="$1"
  node_exists "$label" || fail "missing semantic node: $label"
}

assert_attribute() {
  local label="$1"
  local attribute="$2"
  local expected="$3"
  local actual
  actual="$(node_attribute "$label" "$attribute")"
  [ "$actual" = "$expected" ] \
    || fail "$label expected $attribute=$expected, got ${actual:-<empty>}"
}

assert_type_attribute() {
  local type="$1"
  local attribute="$2"
  local expected="$3"
  jq -e --arg type "$type" --arg attribute "$attribute" --arg expected "$expected" '
    any(.. | objects | select(.attributes?);
      .attributes.type == $type and .attributes[$attribute] == $expected)
  ' "$HOST_LAYOUT" >/dev/null \
    || fail "missing $type node with $attribute=$expected"
}

node_center() {
  local label="$1"
  local bounds left top right bottom
  bounds="$(node_attribute "$label" bounds)"
  read -r left top right bottom < <(
    sed -n 's/^\[\([0-9][0-9]*\),\([0-9][0-9]*\)\]\[\([0-9][0-9]*\),\([0-9][0-9]*\)\]$/\1 \2 \3 \4/p' <<<"$bounds"
  )
  [ -n "${bottom:-}" ] || fail "invalid bounds for $label: $bounds"
  printf '%s %s\n' "$(((left + right) / 2))" "$(((top + bottom) / 2))"
}

qemu_touch() {
  python3 "$ROOT/scripts/qemu-multitouch.py" \
    --socket "$QMP_SOCKET" --width 800 --height 500 "$@"
}

tap_node() {
  local label="$1"
  local x y
  read -r x y < <(node_center "$label")
  qemu_touch tap "$x" "$y"
}

command -v jq >/dev/null 2>&1 || fail "jq is required"
command -v hdc >/dev/null 2>&1 || fail "hdc is required"
command -v python3 >/dev/null 2>&1 || fail "python3 is required"
mkdir -p "$DIAGNOSTICS"
trap cleanup EXIT

QEMU_PACKAGE="$(find_qemu_package)"
QEMU_LAUNCHER="$QEMU_PACKAGE/launch/macos.command"
[ -x "$QEMU_LAUNCHER" ] || fail "QEMU launcher missing: $QEMU_LAUNCHER"
echo "==> QEMU package: $QEMU_PACKAGE"
echo "==> image capabilities verified; no image patching or rebuilding"

if [ "${ARKIT_A11Y_SKIP_BUILD:-0}" != "1" ]; then
  echo "==> building arm64 Rust demo"
  (cd "$ROOT/examples/demos" && ohrs build --arch aarch)
  echo "==> assembling signed HAP"
  "$ROOT/app/run.sh" build
fi

echo "==> launching QEMU with packaged --a11y support"
"$QEMU_LAUNCHER" --headless --a11y --hdc-port "$HDC_PORT" \
  --qmp-socket "$QMP_SOCKET" >"$QEMU_LOG" 2>&1 &
QEMU_PID=$!
wait_for_device

echo "==> installing and starting demo"
HDC_TARGET="$HDC_TARGET" "$ROOT/app/run.sh" install
"${HDC[@]}" shell aa force-stop "$BUNDLE" >/dev/null 2>&1 || true
HDC_TARGET="$HDC_TARGET" "$ROOT/app/run.sh" start
scroll_until_label "无障碍验收" up
tap_node "无障碍验收"
wait_for_label "Accessibility acceptance"

dump_layout
echo "==> checking name, role, state, range, form, region, list, and decorative semantics"
assert_attribute "Native button" type Button
assert_attribute "Custom row exposed as a button" type Button
assert_attribute "Custom row exposed as a button" clickable true
assert_attribute "shadcn Button" type Button
assert_attribute "A11Y shadcn checkbox" type Checkbox
assert_type_attribute Toggle checked true
assert_type_attribute Progress type Progress
assert_attribute "Long press to open context menu" longClickable true
cp "$HOST_LAYOUT" "$DIAGNOSTICS/top-tree.json"

scroll_until_label "A11Y account error" up
dump_layout
assert_type_attribute TextInput enabled true
assert_attribute "A11Y account name *" description "Required. Invalid"
assert_node "A11Y account error"
cp "$HOST_LAYOUT" "$DIAGNOSTICS/form-tree.json"

scroll_until_label "A11Y card title" up
dump_layout
assert_node "Card descendants remain reachable"
assert_node "A11Y card title"
cp "$HOST_LAYOUT" "$DIAGNOSTICS/content-tree.json"

scroll_until_label "A11Y scroll content" up
dump_layout
assert_node "rust code"
assert_attribute "A11Y canvas surface" type Image
assert_attribute "A11Y barcode preview" type Image
assert_attribute "A11Y native chart" type Image
assert_node "Scrollable component results"
assert_node "A11Y scroll content"
cp "$HOST_LAYOUT" "$DIAGNOSTICS/region-tree.json"

scroll_until_label "A11Y semantics complete" up
dump_layout
assert_type_attribute Button selected true
assert_node "A11Y semantics complete"
assert_node "A11Y QEMU verified"
cp "$HOST_LAYOUT" "$DIAGNOSTICS/list-tree.json"

scroll_until_label "Native button" down

echo "==> enabling the app-local accessibility test service"
"${HDC[@]}" shell /system/bin/cli_tool/executable/ohos-a11yManager ability-enable \
  --name "$BUNDLE/$EXTENSION" --capabilities 7 >"$DIAGNOSTICS/enable.log"
for _ in $(seq 1 20); do
  if "${HDC[@]}" shell hidumper -s AccessibilityManagerService -a -u \
    >"$DIAGNOSTICS/manager-state.log" 2>&1 \
    && grep -Eq 'accessible:[[:space:]]+1' "$DIAGNOSTICS/manager-state.log"; then
    break
  fi
  sleep 1
done
grep -Eq 'accessible:[[:space:]]+1' "$DIAGNOSTICS/manager-state.log" \
  || fail "AccessibilityManagerService did not activate the test service"
grep -Eq 'touchGuide:[[:space:]]+1' "$DIAGNOSTICS/manager-state.log" \
  || fail "AccessibilityManagerService did not activate touch exploration"
grep -Eq 'gesture:[[:space:]]+1' "$DIAGNOSTICS/manager-state.log" \
  || fail "AccessibilityManagerService did not activate gestures"

dump_layout
tap_node "Native button"
sleep 1
tap_node "shadcn Button"
sleep 1
tap_node "Custom row exposed as a button"
sleep 1
tap_node "Long press to open context menu"
sleep 1
"${HDC[@]}" shell hilog -x >"$DIAGNOSTICS/focus.log"
for label in "A11Y native button" "A11Y shadcn button" "A11Y custom row button"; do
  grep -F "focused a11y=$label actions=accessibilityFocus,click" \
    "$DIAGNOSTICS/focus.log" >/dev/null \
    || fail "screen-reader service did not receive the click contract for $label"
done
grep -F "focused a11y=A11Y context menu trigger actions=accessibilityFocus,longClick" \
  "$DIAGNOSTICS/focus.log" >/dev/null \
  || fail "screen-reader service did not receive the ContextMenu LongClick contract"

echo "==> invoking Click through accessibility focus + performAction"
tap_node "Custom row exposed as a button"
sleep 1
read -r click_x click_y < <(node_center "Custom row exposed as a button")
click_end_y=$((click_y + 140))
if [ "$click_end_y" -gt 480 ]; then
  click_end_y=480
fi
qemu_touch swipe "$click_x" "$click_y" "$click_x" "$click_end_y"
sleep 2
dump_layout
jq -e '
  any(.. | objects | select(.attributes?);
    ((.attributes.text // "") | test("A11Y clicks=[1-9][0-9]* events=[1-9][0-9]*")))
' "$HOST_LAYOUT" >/dev/null || fail "accessibility Click did not update both action counters"

echo "==> invoking LongClick through the accessibility service"
tap_node "Long press to open context menu"
sleep 1
read -r context_x context_y < <(node_center "Long press to open context menu")
gesture_end_y=$((context_y > 160 ? context_y - 140 : context_y + 140))
qemu_touch swipe "$context_x" "$context_y" "$context_x" "$gesture_end_y"
sleep 2
"${HDC[@]}" shell hilog -x >"$DIAGNOSTICS/action.log"
grep -F "performed=click" "$DIAGNOSTICS/action.log" >/dev/null \
  || fail "accessibility service did not perform Click"
grep -F "performed=longClick" "$DIAGNOSTICS/action.log" >/dev/null \
  || fail "accessibility service did not perform LongClick"
wait_for_label "A11Y menu action"

cp "$HOST_LAYOUT" "$DIAGNOSTICS/passed-tree.json"
"${HDC[@]}" shell hilog -x | grep -E 'ArkitA11yE2E|AccessibilityManagerService' \
  >"$DIAGNOSTICS/accessibility.log" || true
echo "==== QEMU accessibility E2E passed ===="
echo "tree: $DIAGNOSTICS/passed-tree.json"

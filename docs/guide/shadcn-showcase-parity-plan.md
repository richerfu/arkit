# Shadcn Showcase Parity Plan

## Goal

`examples/shadcn_showcase` must match the pre-Dioxus showcase and the shadcn / React Native Reusables interaction model component by component. The source of truth is:

1. The old showcase files under `examples/shadcn_showcase/src/showcase/examples/*.rs` from `HEAD`.
2. Shared old component behavior in `crates/arkit_shadcn/src/components/*` from `HEAD`.
3. Current ArkUI / Dioxus constraints only where the native API has no equivalent yet.

This document is a parity worklist, not a completion report. A component is not
done unless it has passed source comparison, device visual verification, and
interaction verification.

## Rules

- Do not patch individual demos when a shared component owns the behavior.
- Do not introduce item-level `stack` layout to hide row/column sizing bugs.
- Preserve old helper semantics such as `inset`, leading slots, trailing shortcuts, row height, panel padding, and popover placement.
- Every popup component must validate both closed and expanded states with `uitest dumpLayout`.
- Interactions that cannot be represented by current Dioxus events must become framework work, not demo workarounds.
- Do not mark any component `done` from static code review alone. Required evidence is: RN/source diff notes, Harmony device screenshot or layout dump, and a successful interaction path where the component is interactive.

## Current Findings

### 2026-07-09 iOS Parity Pass

Completed against the React Native Reusables iOS showcase behavior:

- Home list now matches the iOS structure: no outer card wrapper, 48vp search input, 56vp rows, light separators, only first/last row corners, chevrons pinned to the right edge.
- Button shared sizing now follows native RN Reusables defaults: default 48vp, large 56vp, small 36vp, icon button 40vp, normal button skin with shadcn radius/colors.
- Card shell/header/content/footer now matches RN Reusables padding and left alignment. Header/content no longer depend on ArkUI Column child alignment because current projection keeps Column children centered unless they are carried by full-width rows.
- Dialog close button is overlayed instead of participating in content layout.
- Menu panels use native-width defaults and explicit leading/trailing slots, so submenu rows align with non-submenu rows.

This is not a full showcase pass. Verified `done` scope is currently limited to
Home list, Button, and Card. Popup, form, data-display, typography, and gesture
components still need component-by-component audit and interaction verification.

Renderer fixes landed in this pass:

- Native Text is initialized with `TextAlign.START`, matching Dioxus/RN default text semantics. Explicit `text_align` still overrides it.
- Mounted ArkUI wrappers now replay desired attributes after `insert_child` rebinds the native wrapper. This prevents first-render-only style loss when ArkUI returns a different mounted wrapper than the detached node originally styled by Dioxus mutations.
- Enum token parsing now tolerates static string tokens with quote wrappers, keeping static and dynamic `align_items` / `justify_content` / flex attrs consistent.

### 2026-07-10 Runtime, Overlay, and Gesture Pass

Implemented:

- Native ArkUI callbacks now enqueue owned event payloads and wake the OpenHarmony UI loop. The UI loop calls Dioxus `Runtime::handle_event` before rendering ready scheduler work, so callbacks fired during a native tree patch cannot re-enter `RuntimeInner`.
- Controlled menu entries no longer remain stale while an overlay is open. `MenuOverlaySession` republishes the same overlay subtree with current props and preserves overlay-local submenu path state.
- `arkit_elements` exposes `onlongpress` / `on_long_press`. `arkit_arkui` owns a single-finger, non-repeating 500ms ArkUI LongPress recognizer and dispatches once at gesture `Accept`; ContextMenu no longer opens on click.
- HoverCard demo content restores the old start-aligned `v_stack` behavior while retaining center anchoring relative to its trigger.

Device packages for each change were built with `ohrs build --arch aarch`, packaged, installed, and launched individually. Component status below still follows the stricter rule that `done` requires an explicit visual and interaction acceptance pass.

### Menu Family

Components: `ContextMenu`, `DropdownMenu`, `Menubar`.

Completed:

- Submenu state is owned by shared `MenuContentPanel` through an open path.
- Shortcut and chevron alignment no longer depends on `space_between`; rows now use an explicit weighted spacer.
- Floating menu placement estimates expanded submenu height and flips/clamps inside the viewport.
- Context menu `More Tools` restores `SubTrigger inset` demo semantics.
- Open menu checkbox/radio entries refresh immediately after controlled state changes without closing the popup.
- ContextMenu uses a native long-press gesture bridge; short tap is no longer an activation fallback.
- Menubar root popup placement now uses an element-bound Dioxus `onarea`
  layout event for the exact trigger node, closed-state menu height for root
  placement, and a top-start overlay portal. Verified on device:
  `File [101,1454][181,1524]`, `New Tab [118,1611][365,1688]`.

Remaining:

- Desktop side-submenu placement is not implemented because the current mobile viewport cannot fit two 208vp panels side by side. The current mobile behavior is inline expansion with precomputed placement. A future adaptive submenu placement must choose side panel only when viewport width allows it.
- ContextMenu short-tap/500ms-long-press behavior and Menubar active-menu switching still need explicit acceptance before either component is marked `done`.

### 2026-07-09 AlertDialog Verification

Completed for `AlertDialog`:

- Demo trigger is centered by the shared `DemoCanvas` Stack-alignment path, matching the RN `justify-center items-center` screen.
- Modal presentation is centered by `arkit_hooks::modal_overlay_layer` using Stack native alignment instead of Column `justify_content`, which is unreliable in the current ArkUI projection.
- Footer action semantics use explicit `action` / `cancel` slots. The visual order is RN native `Continue` above `Cancel` without reversing opaque Dioxus `children`.
- Verified on device:
  - `/private/tmp/shadcn_alert_page_centered_trigger.jpeg`
  - `/private/tmp/shadcn_alert_dialog_centered.jpeg`
  - `/private/tmp/shadcn_alert_dialog_after_cancel.jpeg`

Remaining modal work:

- `Dialog` still needs a separate screenshot/dump pass after the shared overlay-centering fix.
- Sheet/Drawer use other modal presentations and must be rechecked because their placement still depends on Row/Column positioning.

### 2026-07-14 Bottom Sheet Verification

Completed for `BottomSheet`:

- Added the RNR native profile editor demo, controlled/uncontrolled open state,
  backdrop and close-button dismissal, safe-area padding, and pan-down dismiss
  threshold on the handle.
- Bottom presentation uses an in-flow full-width panel plus a weighted spacer.
  This avoids zero-size `Stack`, transition, and absolute-position wrappers
  that caused ArkUI API 24 to paint the panel from the viewport top.
- Verified on device at 1320×2856: the expanded panel root is
  `[0,1437][1320,2856]`; trigger open, close-button dismiss, and backdrop
  dismiss all complete successfully.

### 2026-07-14 Calendar Verification

Completed for `Calendar`:

- Replaced the compact ArkUI `CalendarPicker` wrapper with a full month grid
  matching the iOS RNR `react-native-calendars` showcase: month navigation,
  weekday headings, six stable week rows, outside-month dates, and today
  highlighting.
- Added controlled single- and multi-selection APIs. The Showcase now contains
  the RNR blue single-select calendar and orange multi-select calendar.
- Verified on a 1320×2856 device: calendar panels are
  `[56,360][1264,1542]` and `[56,1626][1264,2808]`; selecting day 20 produces
  `#0284C7`, navigating advances July 2026 to August 2026, selecting days 20
  and 21 produces two `#F97316` cells, and pressing day 20 again deselects only
  that cell.

### 2026-07-14 Date Picker Verification

Completed for `DatePicker`:

- Replaced the inline ArkUI wheel wrapper with the iOS RNR mobile flow: a
  calendar-icon outline trigger opens the shared Calendar in a headerless
  Bottom Sheet with a full-width Close action.
- The Calendar uses its embedded surface mode in Date Picker, removing the
  standalone card border, radius, and shadow to match the borderless RNR sheet.
- Bottom Sheet now republishes open content when controlled child props change,
  so a selected date updates both the calendar cell and the trigger label
  without dismissing the sheet.
- Verified on a 1320×2856 device: the sheet is `[0,1223][1320,2856]`; pressing
  day 20 changes its fill to `#0284C7` and the trigger to `2026-07-20`, pressing
  it again clears both states, Close removes the overlay, and a selected value
  remains visible after closing.

### 2026-07-14 Bottom Navigation Verification

Completed for `BottomNavigation`:

- Added an Arkit mobile extension with icon-and-label destinations, theme-aware
  active/inactive colors, equal-width touch targets, and controlled or
  uncontrolled selection.
- The showcase uses a production-shaped full-page layout: page content fills
  the available viewport and the navigation bar sits directly above the
  system bottom area without a centered mock-device wrapper.
- Verified on a 1320×2856 device: the four targets are
  `[0,2537][330,2758]`, `[330,2537][660,2758]`,
  `[660,2537][990,2758]`, and `[990,2537][1320,2758]`. Home, Explore, Alerts,
  and Profile each update the page icon, title, description, and active item.

## Component Checklist

Status values:

- `done`: code and device dump/interaction were verified.
- `in_progress`: code changed but needs full visual pass.
- `pending`: not yet audited against old demo.
- `blocked`: needs renderer/native event work.

| Component | Demo Source | Status | Required Work |
| --- | --- | --- | --- |
| Accordion | `accordion.rs` | pending | Compare item titles, body spacing, chevron alignment, collapsible state. |
| Alert | `alert.rs` | pending | Compare icon slot, destructive colors, list indentation. |
| Alert Dialog | `alert_dialog.rs` | done | Verified trigger centering, modal centering, footer order, and Cancel dismiss on device. |
| Aspect Ratio | `aspect_ratio.rs` | in_progress | Recheck image fill, radius, full-width canvas. |
| Avatar | `avatar.rs` | pending | Compare image size, fallback text, overlap/ring. |
| Badge | `badge.rs` | in_progress | Recheck icon slot, pill min size, variant colors. |
| Bottom Navigation | `bottom_navigation.rs` | done | Full-page mobile layout verified with four equal-width targets, theme-aware active state, and Home/Explore/Alerts/Profile switching. |
| Bottom Sheet | `bottom_sheet.rs` | done | Verified RNR profile form, `[0,1437][1320,2856]` bottom placement, trigger open, close button, and backdrop dismiss on device. |
| Button | `button.rs` | done | Verified on device: iOS demo sizing, variants, disabled state, and icon/text overlap fixed. |
| Calendar | `calendar.tsx` | done | Verified full month layout, previous/next navigation, today accent, blue single selection, orange multi-selection, and deselection on device. |
| Card | `card.rs` | done | Verified on device: shell radius/border, p-6 header/content/footer, left-aligned title/description/content/footer. |
| Checkbox | `checkbox.rs` | in_progress | Recheck label alignment, card checkbox layout, checked state. |
| Collapsible | `collapsible.rs` | pending | Compare fixed width, row spacing, chevron and content indentation. |
| Context Menu | `context_menu.rs` | in_progress | Native long-press bridge implemented; verify short tap does nothing and 500ms hold opens exactly once. |
| Date Picker | `date-picker.tsx` | done | Verified outline trigger, headerless bottom sheet, calendar selection/deselection, live label refresh, Close dismissal, and selected-value persistence on device. |
| Dialog | `dialog.rs` | in_progress | Header size/alignment, overlay inset, and flex-col-reverse footer updated; needs device overlay screenshot and dismiss verification. |
| Dropdown Menu | `dropdown_menu.rs` | in_progress | Controlled entry refresh implemented; recheck popup placement, submenu expansion, shortcut alignment. |
| Hover Card | `hover_card.rs` | in_progress | Center anchor and start-aligned content restored; needs final placement/hover interaction acceptance. |
| Icon | `icon.rs` | in_progress | Recheck image clarity, tile sizing, star sizing. |
| Input | `input.rs` | in_progress | Recheck height, border, placeholder/value alignment. |
| Label | `label.rs` | pending | Compare text size, disabled/required examples if present. |
| Menubar | `menubar.rs` | in_progress | Root placement and live controlled selection refresh implemented; recheck active-menu switching and submenu expansion. |
| Popover | `popover.rs` | in_progress | Default center align updated; needs trigger anchoring screenshot and outside-dismiss verification. |
| Progress | `progress.rs` | pending | Compare track height, fill color, radius. |
| Radio Group | `radio_group.rs` | in_progress | Recheck option spacing, dot size, label alignment. |
| Select | `select.rs` | pending | Compare trigger, menu width, selected check alignment. |
| Separator | `separator.rs` | done | Verified horizontal/vertical thickness and spacing on device; demo content now shares the RNR left edge while the fixed-width example remains centered. |
| Skeleton | `skeleton.rs` | pending | Compare dimensions and radius. |
| Switch | `switch.rs` | pending | Compare track/thumb size, checked colors, disabled state. |
| Table | `table.rs` | pending | Compare row height, separators, header weight. |
| Tabs | `tabs.rs` | in_progress | Runtime event reentrancy crash fixed; compare tab list width, active indicator, and content padding. |
| Text | `text.rs` | pending | Compare typography variants and canvas layout. |
| Textarea | `textarea.rs` | pending | Compare height, text alignment, placeholder. |
| Toggle | `toggle.rs` | pending | Compare variants, pressed state, icon/text spacing. |
| Toggle Group | `toggle_group.rs` | in_progress | Recheck grouped radius, borders, selected state. |
| Tooltip | `tooltip.rs` | in_progress | RN native panel style and hover-leave close updated; needs placement dump and hover/tap verification. |

## Verification Commands

```bash
cargo fmt -p shadcn_showcase -p arkit_shadcn
cd examples/shadcn_showcase && ohrs build --arch aarch

cd ../..
app/run.sh shadcn_showcase all
hdc shell uitest dumpLayout
```

`cargo check` may be used for host diagnostics, but it is not a validation result. Device validation begins only after the example succeeds with `ohrs build --arch aarch`, and examples are packaged/installed one at a time.

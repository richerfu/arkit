# Popup Overlay Redesign

## Problem

The previous popup implementation in `arkit_shadcn` depended on:

- `portal_scope`
- manual window-coordinate measurement
- extra wrapper nodes around the trigger

This caused three structural issues:

1. The wrapper node could change the trigger's original layout basis.
   This was especially bad for triggers using built-in `Button` skin or percentage sizing.
2. Popup positioning depended on manual measurement and delayed reconciliation.
   This made `tooltip`, `popover`, `hover-card`, `dropdown-menu`, and `context-menu` drift away from the trigger.
3. Popup logic lived in `arkit_shadcn`, so every component carried its own mounting and positioning concerns.

## New Design

The popup anchor primitive is now moved down into `arkit`.

Core idea:

- Keep the existing runtime portal host, but move the attachment primitive into `arkit`.
- Keep the trigger node as the actual mounted root.
- Measure the real trigger node instead of an extra wrapper node.
- Mount popup content into the shared portal host as a full-screen layer, then place the panel inside that layer.
- Let `arkit_shadcn` only describe placement and styling.

## New Runtime Primitives

File: `crates/arkit/src/overlay.rs`

Added:

- `anchored_overlay(trigger, panel)`
- `native_overlay(trigger, panel, placement)`
- `observe_layout_frame(element, signal)`
- `observe_layout_size(element, signal)`
- `LayoutFrame`
- `LayoutSize`
- `NativeOverlayPlacement`

Notes:

- `anchored_overlay` is a passthrough view node. It does not insert an extra layout wrapper around the trigger.
- `anchored_overlay` owns only portal attachment. It does not hardcode popup placement math.
- `native_overlay` is a detached native overlay attachment primitive built on ArkUI `NODE_OVERLAY`.
- `observe_layout_frame` observes the actual trigger root node, so built-in button layout and percentage sizing are not perturbed.
- `observe_layout_size` is also passthrough. It exists only for width-dependent panels such as `select`.

## Shadcn Migration

File: `crates/arkit_shadcn/src/components/floating_layer.rs`

Changed from:

- wrapper measurement
- popup logic mixed with trigger layout

Changed to:

- thin adapter over `arkit::anchored_overlay`
- `observe_layout_frame` for the trigger
- `observe_layout_size` for the popup panel
- full-screen portal layer + ArkUI `Position` attribute

This means popup components now share one mounting model instead of each carrying custom positioning logic.

## Expected Impact

This should directly address the previously reported issues:

- trigger button shape being altered after popup integration
- tooltip covering the trigger
- popover / hover-card / menu panels drifting too far away
- popup behavior relying on demo-specific layout hacks

## Verification

Verified locally:

- `cargo fmt --all`
- `cargo check -p arkit`
- `cargo check -p arkit_shadcn`
- `cargo check -p counter`
- `cd examples/counter && ohrs build --arch aarch`

## Remaining Risk

The new portal-attached path is structurally safer than the previous wrapper-measurement path, but final visual confirmation still needs device-side validation for:

- exact vertical gap
- top/bottom alignment semantics on real devices
- select panel width behavior

## Native vs Calculation

The current popup stack is intentionally hybrid instead of fully manual:

- ArkUI native capabilities handle popup layering and final node placement through normal node attributes.
- `NodeUtils` is used only to read the trigger's measured window frame.
- Popup math is limited to anchor derivation such as `top/bottom` and `start/center`.

This keeps popup behavior generic across `tooltip`, `popover`, `hover-card`, `dropdown-menu`,
`context-menu`, and `select` without relying on demo-specific wrappers or low-level
`setLayoutPosition` mutations.

## Binding Follow-up

The local `ohos-native-bindings` dependency now also exposes the missing pieces needed for
native overlay experiments:

- composite `ArkUI_AttributeItem` payload support in `common::attribute`
- `types::direction::Direction`
- `types::overlay::OverlayOptions`
- `reset_overlay()` on the common attribute trait

This unblocks a typed `NODE_OVERLAY` path for components that truly map to ArkUI's native overlay
semantics.

It is still not enabled for every popup component by default yet, because the exact runtime
semantics of `NODE_OVERLAY` for external anchored panels still need device-side confirmation.
The current portal path remains the stable fallback until that audit is complete.

Device-side validation later showed that ArkUI `NODE_OVERLAY` aligns from the trigger's own
top-left basis and behaves like an in-component overlay instead of a detached popup layer for
these shadcn-style popups. Because of that, `tooltip`, `popover`, and `hover-card` stay on the
portal-backed path by default, while the native overlay primitive remains available for cases that
truly map to component-local overlay semantics.

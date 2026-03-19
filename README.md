# arkit

`arkit` is a workspace-based ArkUI framework built on top of local `ohos-native-bindings` and integrated with `openharmony-ability`.

## Workspace Layout

- `crates/arkit`: core runtime, component DSL, `signal` state model
- `crates/arkit_derive`: `#[entry]` and `#[component]` macros
- `crates/arkit_shadcn`: shadcn-style component crate (built on `arkit`)
- `examples/counter`: sample entry crate for OpenHarmony integration

## Key APIs

- `#[entry]`: defines the OpenHarmony entry function and generates `init/render/destroy`
- `#[component]`: marks reusable component functions
- `use_signal(...)`: Dioxus-style state declaration
- `use_route()`: read current route in component render
- `use_router()`: get router handle for push/replace/back navigation
- `use_lifecycle(...)`: subscribe app lifecycle events (`WindowCreate`, `Resume`, ...)
- `use_component_lifecycle(...)`: subscribe component mount/unmount

## Router Crate

- `crates/arkit_router`: route registration + stack navigation + params/query extraction
- registration: `register("/home")`, `register_named("detail", "/detail/:id")`
- navigation: `push("/detail/42?tab=profile")`, `replace(...)`, `back()`, `reset(...)`
- route read: `use_route()` / `use_route_param("id")` / `use_route_query("tab")`

## Shadcn Component Crate

`arkit_shadcn` provides the full shadcn component list with ArkUI-native rendering:

- accordion, alert, alert_dialog, aspect_ratio, avatar, badge, breadcrumb
- button, calendar, card, carousel, chart, checkbox, collapsible
- combobox, command, context_menu, data_table, date_picker, dialog
- drawer, dropdown_menu, form, hover_card, input, input_otp, label
- menubar, navigation_menu, pagination, popover, progress, radio_group
- resizable, scroll_area, select, separator, sheet, sidebar, skeleton
- slider, sonner, switch, table, tabs, textarea, toast, toggle
- toggle_group, tooltip

Use `arkit_shadcn::prelude::*` to import the full set.

## View Layer (All Components)

`arkit::prelude::*` now exposes all ArkUI API-22 component constructors in view style:

- layout/list: `column_component`, `row_component`, `stack_component`, `list_component`, `grid_component`, ...
- input/content: `text_component`, `text_input_component`, `button_component`, `slider_component`, `checkbox_component`, ...
- media/others: `image_component`, `xcomponent_component`, `embedded_component_component`, ...

For full style/attribute coverage, use:

- `ComponentElement::attr(ArkUINodeAttributeType, ArkUINodeAttributeItem)`
- `ComponentElement::style(...)` (alias of `attr`)
- `ComponentElement::on_event(NodeEventType, ...)` for all node events
- `ComponentElement::on_custom_event(NodeCustomEventType, ...)` for all custom events
- `ComponentElement::native(|native| { ... })` for any native ArkUI API call

## Example

```rust
use arkit::prelude::*;

#[component]
fn app_view() -> Element {
    let count = use_signal(|| 0);
    let inc = count.clone();

    column(vec![
        text(format!("count = {}", count.get())).into(),
        button("+1")
            .on_click(move || inc.update(|value| *value += 1))
            .into(),
    ])
}

#[entry]
fn app() -> Element {
    app_view()
}
```

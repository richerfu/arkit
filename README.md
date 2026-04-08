# arkit

`arkit` is an ArkUI framework for OpenHarmony built on local `ohos-native-bindings`
and integrated with `openharmony-ability`.

The public model now follows `iced`:

- application state lives in `State`
- all side effects flow through `Task<Message>` and `Subscription<Message>`
- `view(&State) -> Element` is a pure render function
- widgets use builder-style APIs and theme/style catalogs

`arkit` itself is the facade crate. Runtime and widget implementation live in
dedicated crates and are re-exported here.

## Workspace Layout

- `crates/arkit`: facade / re-export crate
- `crates/arkit_widget`: ArkUI widget tree, renderer, node diff, and widget builders
- `crates/arkit_runtime`: renderer-agnostic application/runtime shell, task/subscription wiring
- `crates/arkit_derive`: `#[entry]` and `#[component]` macros
- `crates/arkit_shadcn`: shadcn-style component crate (built on `arkit`)
- `examples/counter`: minimal smoke example for OpenHarmony integration
- `examples/shadcn_showcase`: shadcn / react-native-reusables showcase example

## Key APIs

- `#[entry]`: defines the OpenHarmony entry function and generates `init/render/destroy`
- `application(boot, update, view)`: iced-style application builder
- `Task<Message>` / `Subscription<Message>`: message-driven side effects
- `NavigationStack<T>`: explicit application-state navigation for example apps
- `#[component]`: a no-op marker for reusable view helpers; it does not create hidden state

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

`arkit::prelude::*` exposes the ArkUI-backed widget constructors in builder style:

- layout: `column_component`, `row_component`, `stack_component`, `scroll_component`
- input/content: `text_component`, `text_input_component`, `button_component`, `slider_component`, `checkbox_component`
- media/others: `image_component`, `calendar_picker_component`, `date_picker_component`, `swiper_component`

For full style/attribute coverage, use the `Node` builder methods:

- `Node::attr(ArkUINodeAttributeType, ArkUINodeAttributeItem)`
- `Node::style(...)` (alias of `attr`)
- `Node::on_event(NodeEventType, ...)` for all node events
- `Node::on_custom_event(NodeCustomEventType, ...)` for all custom events
- `Node::native(|native| { ... })` for direct ArkUI node access

## Example

```rust
use arkit::prelude::*;
use arkit::{application, Task};

#[derive(Clone, Debug)]
enum Message {
    Increment,
}

#[derive(Default)]
struct State {
    count: i32,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.count += 1,
    }

    Task::none()
}

fn view(state: &State) -> Element {
    column(vec![
        text(format!("count = {}", state.count)).into(),
        button("+1").on_press(Message::Increment).into(),
    ])
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(State::default, update, view)
}
```

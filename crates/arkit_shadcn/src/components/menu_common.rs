use super::floating_layer::{
    floating_panel_aligned_with_builder, FloatingAlign, FloatingSide, FloatingSurfaceRegistry,
};
use super::*;
use arkit::advanced;
use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use arkit::TextAlignment;
use arkit_icon as lucide;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) const TRANSPARENT: u32 = 0x00000000;
const MENU_PANEL_HORIZONTAL_PADDING: f32 = spacing::XXS * 2.0;
const MENU_PANEL_MAX_SIZE: f32 = 100_000.0;
const FIX_AT_IDEAL_SIZE_POLICY: i32 = 2;
const MENU_TEXT_MAX_LINES: i32 = 1;
const MENU_TRAILING_GAP: f32 = spacing::SM;

type ActionCallback = Rc<dyn Fn()>;
type ToggleCallback = Rc<dyn Fn(bool)>;
type SelectCallback = Rc<dyn Fn(String)>;

#[derive(Clone)]
pub(crate) struct MenuStyle {
    pub(crate) width: f32,
    pub(crate) submenu_width: f32,
    pub(crate) side_offset_vp: f32,
}

#[derive(Clone)]
pub enum MenuEntry {
    Action(MenuActionEntry),
    Submenu(MenuSubmenuEntry),
    Checkbox(MenuCheckboxEntry),
    Radio(MenuRadioEntry),
    Label(MenuLabelEntry),
    Separator,
}

#[derive(Clone)]
pub struct MenuActionEntry {
    pub(crate) title: String,
    pub(crate) shortcut: Option<String>,
    pub(crate) destructive: bool,
    pub(crate) disabled: bool,
    pub(crate) inset: bool,
    pub(crate) on_select: Option<ActionCallback>,
}

#[derive(Clone)]
pub struct MenuSubmenuEntry {
    pub(crate) title: String,
    pub(crate) inset: bool,
    pub(crate) items: Vec<MenuEntry>,
}

#[derive(Clone)]
pub struct MenuCheckboxEntry {
    pub(crate) title: String,
    pub(crate) checked: bool,
    pub(crate) on_toggle: ToggleCallback,
}

#[derive(Clone)]
pub struct MenuRadioEntry {
    pub(crate) title: String,
    pub(crate) value: String,
    pub(crate) selected: String,
    pub(crate) on_select: SelectCallback,
}

#[derive(Clone)]
pub struct MenuLabelEntry {
    pub(crate) title: String,
    pub(crate) inset: bool,
}

impl MenuEntry {
    pub fn action(title: impl Into<String>) -> Self {
        menu_action_entry(title, None, false, false, false, None)
    }

    pub fn submenu(title: impl Into<String>, items: Vec<MenuEntry>) -> Self {
        menu_submenu_entry(title, false, items)
    }

    pub fn checkbox<Message>(
        title: impl Into<String>,
        checked: bool,
        on_toggle: impl Fn(bool) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        menu_checkbox_entry(
            title,
            checked,
            Rc::new(move |value| super::dispatch_message(on_toggle(value))),
        )
    }

    pub fn radio<Message>(
        title: impl Into<String>,
        value: impl Into<String>,
        selected: impl Into<String>,
        on_select: impl Fn(String) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        menu_radio_entry(
            title,
            value,
            selected,
            Rc::new(move |value| super::dispatch_message(on_select(value))),
        )
    }

    pub fn label(title: impl Into<String>) -> Self {
        menu_label_entry(title, false)
    }

    pub fn separator() -> Self {
        menu_separator_entry()
    }

    pub fn destructive(mut self) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.destructive = true;
        }
        self
    }

    pub fn disabled(mut self) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.disabled = true;
        }
        self
    }

    pub fn inset(mut self) -> Self {
        match &mut self {
            Self::Action(entry) => entry.inset = true,
            Self::Submenu(entry) => entry.inset = true,
            Self::Label(entry) => entry.inset = true,
            _ => {}
        }
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.shortcut = Some(shortcut.into());
        }
        self
    }

    pub fn on_select(mut self, callback: impl Fn() + 'static) -> Self {
        if let Self::Action(entry) = &mut self {
            entry.on_select = Some(Rc::new(callback));
        }
        self
    }

    pub fn on_select_message<Message>(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.on_select(move || super::dispatch_message(message.clone()))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MenuInteractionVariant {
    Default,
    Destructive,
}

#[derive(Clone)]
struct MenuRenderContext {
    dismiss: Rc<dyn Fn()>,
    root_open: bool,
    root_surfaces: FloatingSurfaceRegistry,
    current_surfaces: FloatingSurfaceRegistry,
    interaction: Rc<RefCell<MenuInteractionState>>,
    path: Vec<usize>,
    style: MenuStyle,
}

#[derive(Default)]
struct MenuInteractionState {
    open_path: Vec<usize>,
}

struct MenuPopupTreeState {
    root_surfaces: FloatingSurfaceRegistry,
    interaction: Rc<RefCell<MenuInteractionState>>,
}

struct MenuSubmenuTreeState {}

struct MenuPopupWidget<Message> {
    trigger: RefCell<Option<Element<Message>>>,
    items: Vec<MenuEntry>,
    open: bool,
    align: FloatingAlign,
    on_open_change: Rc<dyn Fn(bool)>,
    style: MenuStyle,
}

struct MenuSubmenuWidget<Message> {
    title: String,
    inset: bool,
    items: Vec<MenuEntry>,
    path: Vec<usize>,
    context: MenuRenderContext,
    _marker: std::marker::PhantomData<Message>,
}

struct RuntimeMenuRowNode(ArkUINode);

impl ArkUIAttributeBasic for RuntimeMenuRowNode {
    fn raw(&self) -> &ArkUINode {
        &self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        &mut self.0
    }
}

impl ArkUICommonAttribute for RuntimeMenuRowNode {}

fn request_widget_rerender() {
    arkit_widget::queue_ui_loop(|| {
        if let Some(runtime) = arkit_widget::current_runtime() {
            let _ = runtime.request_rerender();
        }
    });
}

fn ensure_tree_children(tree: &mut advanced::widget::Tree, len: usize) {
    let mut children = std::mem::take(tree.children_mut());
    children.truncate(len);
    while children.len() < len {
        children.push(advanced::widget::Tree::empty());
    }
    tree.replace_children(children);
}

fn menu_branch_path(parent_path: &[usize], index: usize) -> Vec<usize> {
    let mut path = parent_path.to_vec();
    path.push(index);
    path
}

fn menu_branch_is_open(open_path: &[usize], branch_path: &[usize]) -> bool {
    !branch_path.is_empty() && open_path.starts_with(branch_path)
}

fn toggle_menu_branch(state: &Rc<RefCell<MenuInteractionState>>, branch_path: &[usize]) {
    let mut state = state.borrow_mut();
    if menu_branch_is_open(&state.open_path, branch_path) {
        state
            .open_path
            .truncate(branch_path.len().saturating_sub(1));
    } else {
        state.open_path.clear();
        state.open_path.extend_from_slice(branch_path);
    }
}

fn reset_menu_branches(state: &Rc<RefCell<MenuInteractionState>>) {
    state.borrow_mut().open_path.clear();
}

impl<Message: 'static> advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for MenuPopupWidget<Message>
{
    fn state(&self) -> advanced::widget::State {
        advanced::widget::State::new(Box::new(MenuPopupTreeState {
            root_surfaces: FloatingSurfaceRegistry::new(),
            interaction: Rc::new(RefCell::new(MenuInteractionState::default())),
        }))
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![advanced::widget::Tree::empty()]
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        ensure_tree_children(tree, 1);
    }

    fn body(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = tree
            .state()
            .downcast_mut::<MenuPopupTreeState>()
            .expect("menu popup tree state type mismatch");
        let root_surfaces = state.root_surfaces.clone();
        let interaction = state.interaction.clone();
        if !self.open {
            reset_menu_branches(&interaction);
        }
        let trigger = self
            .trigger
            .borrow_mut()
            .take()
            .expect("menu popup trigger was already consumed");
        let dismiss_interaction = interaction.clone();
        let on_open_change = self.on_open_change.clone();
        let dismiss = Rc::new(move || {
            reset_menu_branches(&dismiss_interaction);
            on_open_change(false);
        });
        let context = MenuRenderContext {
            dismiss: dismiss.clone(),
            root_open: self.open,
            root_surfaces: root_surfaces.clone(),
            current_surfaces: root_surfaces.clone(),
            interaction: interaction.clone(),
            path: Vec::new(),
            style: self.style.clone(),
        };
        let panel_items = self.items.clone();
        let panel_style = self.style.clone();
        let panel_side_offset_vp = self.style.side_offset_vp;
        let panel_builder = Rc::new(move |_trigger_width: Option<f32>| {
            menu_content_with_width(
                panel_style.width,
                render_menu_entries::<Message>(panel_items.clone(), context.clone()),
            )
        });

        Some(floating_panel_aligned_with_builder(
            trigger,
            self.open,
            FloatingSide::Bottom,
            self.align,
            panel_side_offset_vp,
            panel_builder,
            Some(dismiss),
            false,
            vec![root_surfaces.clone()],
            Some(root_surfaces),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: 'static> advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for MenuSubmenuWidget<Message>
{
    fn state(&self) -> advanced::widget::State {
        advanced::widget::State::new(Box::new(MenuSubmenuTreeState {}))
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![advanced::widget::Tree::empty()]
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        ensure_tree_children(tree, 1);
    }

    fn body(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let _state = tree
            .state()
            .downcast_mut::<MenuSubmenuTreeState>()
            .expect("menu submenu tree state type mismatch");
        let title = self.title.clone();
        let submenu_path = self.path.clone();
        let submenu_open = self.context.root_open
            && menu_branch_is_open(&self.context.interaction.borrow().open_path, &submenu_path);
        let toggle_state = self.context.interaction.clone();
        let toggle_path = submenu_path.clone();

        let trigger = submenu_trigger_row(
            title,
            self.inset,
            submenu_open,
            menu_row_min_width(&self.context.style),
            Rc::new(move || {
                toggle_menu_branch(&toggle_state, &toggle_path);
                request_widget_rerender();
            }),
        );

        let mut column_children: Vec<Element<Message>> = vec![trigger];

        if submenu_open {
            let submenu_context = MenuRenderContext {
                dismiss: self.context.dismiss.clone(),
                root_open: self.context.root_open,
                root_surfaces: self.context.root_surfaces.clone(),
                current_surfaces: self.context.current_surfaces.clone(),
                interaction: self.context.interaction.clone(),
                path: submenu_path,
                style: self.context.style.clone(),
            };
            let sub_items = render_menu_entries::<Message>(self.items.clone(), submenu_context);
            let min_width = menu_submenu_min_width(&self.context.style);

            let sub_content = arkit::column_component::<Message, arkit::Theme>()
                .attr(
                    ArkUINodeAttributeType::WidthLayoutpolicy,
                    FIX_AT_IDEAL_SIZE_POLICY,
                )
                .constraint_size(min_width, MENU_PANEL_MAX_SIZE, 0.0, MENU_PANEL_MAX_SIZE)
                .align_items_start()
                .padding([spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS])
                .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
                .background_color(colors().accent)
                .children(sub_items);

            column_children.push(sub_content.into());
        }

        Some(
            arkit::column_component::<Message, arkit::Theme>()
                .attr(
                    ArkUINodeAttributeType::WidthLayoutpolicy,
                    FIX_AT_IDEAL_SIZE_POLICY,
                )
                .constraint_size(
                    menu_subtree_min_width(&self.context.style),
                    MENU_PANEL_MAX_SIZE,
                    0.0,
                    MENU_PANEL_MAX_SIZE,
                )
                .align_items_start()
                .children(column_children)
                .into(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

pub(crate) fn menu_popup<Message: 'static>(
    trigger: Element<Message>,
    items: Vec<MenuEntry>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    align: FloatingAlign,
    style: MenuStyle,
) -> Element<Message> {
    Element::new(MenuPopupWidget {
        trigger: RefCell::new(Some(trigger)),
        items,
        open,
        align,
        on_open_change: Rc::new(on_open_change),
        style,
    })
}

fn render_menu_entries<Message: 'static>(
    entries: Vec<MenuEntry>,
    context: MenuRenderContext,
) -> Vec<Element<Message>> {
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| render_menu_entry(entry, index, context.clone()))
        .collect()
}

fn render_menu_entry<Message: 'static>(
    entry: MenuEntry,
    index: usize,
    context: MenuRenderContext,
) -> Element<Message> {
    match entry {
        MenuEntry::Action(entry) => render_action_entry(entry, &context),
        MenuEntry::Submenu(entry) => Element::new(MenuSubmenuWidget {
            title: entry.title,
            inset: entry.inset,
            items: entry.items,
            path: menu_branch_path(&context.path, index),
            context,
            _marker: std::marker::PhantomData,
        }),
        MenuEntry::Checkbox(entry) => render_checkbox_entry(entry, &context),
        MenuEntry::Radio(entry) => render_radio_entry(entry, &context),
        MenuEntry::Label(entry) => render_label_entry(entry, &context),
        MenuEntry::Separator => menu_separator(menu_row_min_width(&context.style)),
    }
}

fn render_action_entry<Message: 'static>(
    entry: MenuActionEntry,
    context: &MenuRenderContext,
) -> Element<Message> {
    let leading = entry.inset.then(|| leading_slot(None));
    let children = menu_row_children(
        leading,
        item_text(
            entry.title,
            if entry.destructive {
                colors().destructive
            } else {
                colors().popover_foreground
            },
            FontWeight::W400,
        ),
        entry.shortcut.map(shortcut_text),
    );

    let on_select = entry.on_select.clone();
    let dismiss = context.dismiss.clone();
    let row = interactive_menu_row(
        children,
        menu_row_min_width(&context.style),
        entry.disabled,
        if entry.destructive {
            MenuInteractionVariant::Destructive
        } else {
            MenuInteractionVariant::Default
        },
        None,
        Some(Rc::new(move || {
            if let Some(on_select) = on_select.as_ref() {
                on_select();
            }
            dismiss();
        })),
    );

    if entry.disabled {
        return row.into();
    }
    row.into()
}

fn render_checkbox_entry<Message: 'static>(
    entry: MenuCheckboxEntry,
    context: &MenuRenderContext,
) -> Element<Message> {
    let on_toggle = entry.on_toggle.clone();
    let dismiss = context.dismiss.clone();
    interactive_menu_row(
        menu_row_children(
            Some(leading_slot(if entry.checked {
                Some(
                    lucide::icon("check")
                        .size(16.0)
                        .stroke_width(3.0)
                        .color(colors().foreground)
                        .render::<Message, arkit::Theme>(),
                )
            } else {
                None
            })),
            item_text(entry.title, colors().popover_foreground, FontWeight::W400),
            None,
        ),
        menu_row_min_width(&context.style),
        false,
        MenuInteractionVariant::Default,
        None,
        Some(Rc::new(move || {
            on_toggle(!entry.checked);
            dismiss();
        })),
    )
    .into()
}

fn render_radio_entry<Message: 'static>(
    entry: MenuRadioEntry,
    context: &MenuRenderContext,
) -> Element<Message> {
    let on_select = entry.on_select.clone();
    let dismiss = context.dismiss.clone();
    let selected = entry.selected == entry.value;
    let value = entry.value.clone();
    interactive_menu_row(
        menu_row_children(
            Some(leading_slot(if selected {
                Some(
                    arkit::row_component()
                        .width(8.0)
                        .height(8.0)
                        .border_radius([radii().full, radii().full, radii().full, radii().full])
                        .background_color(colors().foreground)
                        .into(),
                )
            } else {
                None
            })),
            item_text(entry.title, colors().popover_foreground, FontWeight::W400),
            None,
        ),
        menu_row_min_width(&context.style),
        false,
        MenuInteractionVariant::Default,
        None,
        Some(Rc::new(move || {
            on_select(value.clone());
            dismiss();
        })),
    )
    .into()
}

fn render_label_entry<Message: 'static>(
    entry: MenuLabelEntry,
    context: &MenuRenderContext,
) -> Element<Message> {
    let leading = entry.inset.then(|| leading_slot(None));
    let children = menu_row_children(
        leading,
        item_text(entry.title, colors().foreground, FontWeight::W500),
        None,
    );
    menu_row(children, menu_row_min_width(&context.style), false).into()
}

fn submenu_trigger_row<Message: 'static>(
    title: String,
    inset: bool,
    active: bool,
    min_width: f32,
    on_click: Rc<dyn Fn()>,
) -> Element<Message> {
    let leading = inset.then(|| leading_slot(None));
    let children = menu_row_children(
        leading,
        item_text(title, colors().popover_foreground, FontWeight::W400),
        Some(
            lucide::icon(if active { "chevron-up" } else { "chevron-down" })
                .size(16.0)
                .color(colors().foreground)
                .render::<Message, arkit::Theme>(),
        ),
    );
    interactive_menu_row(
        children,
        min_width,
        false,
        MenuInteractionVariant::Default,
        Some(active),
        Some(on_click),
    )
    .into()
}

pub(crate) fn menu_action_entry(
    title: impl Into<String>,
    shortcut: Option<String>,
    destructive: bool,
    disabled: bool,
    inset: bool,
    on_select: Option<ActionCallback>,
) -> MenuEntry {
    MenuEntry::Action(MenuActionEntry {
        title: title.into(),
        shortcut,
        destructive,
        disabled,
        inset,
        on_select,
    })
}

pub(crate) fn menu_submenu_entry(
    title: impl Into<String>,
    inset: bool,
    items: Vec<MenuEntry>,
) -> MenuEntry {
    MenuEntry::Submenu(MenuSubmenuEntry {
        title: title.into(),
        inset,
        items,
    })
}

pub(crate) fn menu_checkbox_entry(
    title: impl Into<String>,
    checked: bool,
    on_toggle: ToggleCallback,
) -> MenuEntry {
    MenuEntry::Checkbox(MenuCheckboxEntry {
        title: title.into(),
        checked,
        on_toggle,
    })
}

pub(crate) fn menu_radio_entry(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: SelectCallback,
) -> MenuEntry {
    MenuEntry::Radio(MenuRadioEntry {
        title: title.into(),
        value: value.into(),
        selected: selected.into(),
        on_select,
    })
}

pub(crate) fn menu_label_entry(title: impl Into<String>, inset: bool) -> MenuEntry {
    MenuEntry::Label(MenuLabelEntry {
        title: title.into(),
        inset,
    })
}

pub(crate) fn menu_separator_entry() -> MenuEntry {
    MenuEntry::Separator
}

pub(crate) fn menu_content_with_width<Message: 'static>(
    width: f32,
    items: Vec<Element<Message>>,
) -> Element<Message> {
    shadow_sm(
        arkit::column_component::<Message, arkit::Theme>()
            .attr(
                ArkUINodeAttributeType::WidthLayoutpolicy,
                FIX_AT_IDEAL_SIZE_POLICY,
            )
            .constraint_size(width, MENU_PANEL_MAX_SIZE, 0.0, MENU_PANEL_MAX_SIZE)
            .align_items_start()
            .padding([spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS])
            .border_radius([radii().lg, radii().lg, radii().lg, radii().lg])
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(colors().border)
            .clip(true)
            .background_color(colors().popover)
            .children(items),
    )
    .into()
}

pub(crate) fn item_text<Message: 'static>(
    content: impl Into<String>,
    color_value: u32,
    weight: FontWeight,
) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(color_value)
        .font_weight(weight)
        .line_height(20.0)
        .attr(ArkUINodeAttributeType::TextMaxLines, MENU_TEXT_MAX_LINES)
        .text_align(TextAlignment::Start)
        .into()
}

fn shortcut_text<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::XS)
        .font_color(colors().muted_foreground)
        .line_height(16.0)
        .text_letter_spacing(1.2_f32)
        .attr(ArkUINodeAttributeType::TextMaxLines, MENU_TEXT_MAX_LINES)
        .text_align(TextAlignment::Start)
        .into()
}

fn leading_slot<Message: 'static>(child: Option<Element<Message>>) -> Element<Message> {
    let mut slot = arkit::row_component::<Message, arkit::Theme>()
        .width(16.0)
        .height(16.0)
        .align_items_center()
        .justify_content_center();

    if let Some(child) = child {
        slot = slot.children(vec![child]);
    }

    arkit::row_component::<Message, arkit::Theme>()
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .margin([0.0, 8.0, 0.0, 0.0])
        .children(vec![slot.into()])
        .into()
}

fn menu_row_children<Message: 'static>(
    leading: Option<Element<Message>>,
    label: Element<Message>,
    trailing: Option<Element<Message>>,
) -> Vec<Element<Message>> {
    let mut leading_children = Vec::new();
    if let Some(leading) = leading {
        leading_children.push(leading);
    }
    leading_children.push(label);

    let mut children = vec![menu_row_leading_group(leading_children)];
    if let Some(trailing) = trailing {
        children.push(menu_row_trailing_group(trailing));
    }
    children
}

fn menu_row_leading_group<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .align_items_center()
        .children(children)
        .into()
}

fn menu_row_trailing_group<Message: 'static>(child: Element<Message>) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .align_items_center()
        .margin([0.0, 0.0, 0.0, MENU_TRAILING_GAP])
        .children(vec![child])
        .into()
}

pub(crate) fn menu_row<Message: 'static>(
    children: Vec<Element<Message>>,
    min_width: f32,
    disabled: bool,
) -> RowElement<Message> {
    let mut row = arkit::row_component::<Message, arkit::Theme>()
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .constraint_size(min_width, MENU_PANEL_MAX_SIZE, 0.0, MENU_PANEL_MAX_SIZE)
        .height(32.0)
        .align_items_center()
        .justify_content(arkit::JustifyContent::SpaceBetween)
        .padding([6.0, 8.0, 6.0, 8.0])
        .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
        .clip(true)
        .background_color(TRANSPARENT)
        .children(children);

    if disabled {
        row = row.opacity(0.5_f32);
    }

    row
}

fn menu_separator<Message: 'static>(min_width: f32) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .attr(
            ArkUINodeAttributeType::WidthLayoutpolicy,
            FIX_AT_IDEAL_SIZE_POLICY,
        )
        .constraint_size(min_width, MENU_PANEL_MAX_SIZE, 0.0, MENU_PANEL_MAX_SIZE)
        .height(1.0)
        .margin([4.0, 0.0, 4.0, 0.0])
        .background_color(colors().border)
        .into()
}

fn menu_row_min_width(style: &MenuStyle) -> f32 {
    (style.width - MENU_PANEL_HORIZONTAL_PADDING).max(0.0)
}

fn menu_submenu_min_width(style: &MenuStyle) -> f32 {
    style.submenu_width.max(0.0)
}

fn menu_subtree_min_width(style: &MenuStyle) -> f32 {
    menu_row_min_width(style).max(menu_submenu_min_width(style))
}

fn menu_row_pressed_background(variant: MenuInteractionVariant) -> u32 {
    match variant {
        MenuInteractionVariant::Default => colors().accent,
        MenuInteractionVariant::Destructive => with_alpha(colors().destructive, 0x1A),
    }
}

pub(crate) fn interactive_menu_row<Message: 'static>(
    children: Vec<Element<Message>>,
    min_width: f32,
    disabled: bool,
    variant: MenuInteractionVariant,
    active: Option<bool>,
    on_activate: Option<Rc<dyn Fn()>>,
) -> RowElement<Message> {
    let runtime_node = Rc::new(RefCell::new(None::<RuntimeMenuRowNode>));
    let capture_node = runtime_node.clone();
    let row = menu_row(children, min_width, disabled)
        .background_color(if active.unwrap_or(false) {
            menu_row_pressed_background(variant)
        } else {
            TRANSPARENT
        })
        .with_patch(move |node| {
            capture_node.replace(Some(RuntimeMenuRowNode(node.clone())));
            Ok(())
        });

    if disabled {
        return row;
    }

    row.on_event(arkit::prelude::NodeEventType::TouchEvent, move |event| {
        let Some(input_event) = event.input_event() else {
            return;
        };
        let _ = input_event.pointer_set_stop_propagation(true);
        let row_binding = runtime_node.borrow();
        let Some(node) = row_binding.as_ref() else {
            return;
        };

        match input_event.action {
            UIInputAction::Down => {
                let _ = node.background_color(menu_row_pressed_background(variant));
            }
            UIInputAction::Up => {
                if let Some(on_activate) = on_activate.as_ref() {
                    on_activate();
                }
                let keep_active = active.unwrap_or(false);
                let _ = node.background_color(if keep_active {
                    menu_row_pressed_background(variant)
                } else {
                    TRANSPARENT
                });
            }
            UIInputAction::Cancel => {
                let keep_active = active.unwrap_or(false);
                let _ = node.background_color(if keep_active {
                    menu_row_pressed_background(variant)
                } else {
                    TRANSPARENT
                });
            }
            UIInputAction::Move => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_closed_branch_opens_that_branch() {
        let state = Rc::new(RefCell::new(MenuInteractionState::default()));
        toggle_menu_branch(&state, &[3]);
        assert_eq!(state.borrow().open_path, vec![3]);
    }

    #[test]
    fn toggling_open_branch_closes_only_that_branch() {
        let state = Rc::new(RefCell::new(MenuInteractionState {
            open_path: vec![2, 4],
        }));
        toggle_menu_branch(&state, &[2, 4]);
        assert_eq!(state.borrow().open_path, vec![2]);
    }

    #[test]
    fn switching_branches_replaces_open_path() {
        let state = Rc::new(RefCell::new(MenuInteractionState {
            open_path: vec![1, 0],
        }));
        toggle_menu_branch(&state, &[3]);
        assert_eq!(state.borrow().open_path, vec![3]);
    }

    #[test]
    fn branch_open_check_uses_prefix_matching() {
        assert!(menu_branch_is_open(&[1, 2], &[1]));
        assert!(menu_branch_is_open(&[1, 2], &[1, 2]));
        assert!(!menu_branch_is_open(&[1, 2], &[2]));
        assert!(!menu_branch_is_open(&[], &[1]));
    }
}

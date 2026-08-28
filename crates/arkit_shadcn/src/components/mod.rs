//! Shadcn component registry — migrated components plus Arkit mobile
//! extensions, implemented with dioxus 0.7 `#[component]` + `rsx!`.
//!
//! Each component preserves its original styling logic (colors/radii/spacing/
//! typography/shadow) but renders via dioxus elements instead of the old
//! `Component`/`Element` builder.

pub(crate) const ARKUI_BORDER_STYLE_SOLID: &str = "solid";

pub use arkit_prelude::{Col, ColProps, Row, RowProps};

mod accordion;
mod alert;
mod alert_dialog;
mod anchor;
mod aspect_ratio;
mod avatar;
mod badge;
mod bottom_navigation;
mod bottom_sheet;
mod breadcrumb;
mod button;
mod calendar;
mod calendar_plugin;
mod card;
mod carousel;
mod chart;
mod checkbox;
/// Standalone syntax-highlighted code block (`code` feature).
#[cfg(feature = "code")]
mod code;
/// Tree-sitter highlight engine and language registry (`code` feature).
///
/// Independent of Markdown. Prefer the flat re-exports below, or
/// `arkit::shadcn::components::code_highlight::register_language`.
#[cfg(feature = "code")]
pub mod code_highlight;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod date_picker;
mod dialog;
mod drawer;
mod dropdown_menu;
mod floating_layer;
mod form;
mod guide;
mod hover_card;
mod input;
mod input_otp;
mod label;
#[cfg(feature = "markdown")]
mod markdown;
mod menu_common;
mod menubar;
mod motion;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod radio_group;
mod refresh;
mod resizable;
mod scroll_area;
mod secure_keyboard;
mod select;
mod separator;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod spinner;
mod surfaces;
mod switch;
mod table;
mod tabs;
mod text;
mod textarea;
mod time_picker;
mod timeline;
mod toggle;
mod toggle_group;
mod tooltip;
mod watermark;

pub use accordion::{Accordion, AccordionItemSpec, AccordionProps};
pub use alert::{
    Alert, AlertDescription, AlertDescriptionProps, AlertList, AlertListProps, AlertProps,
    AlertTitle, AlertTitleProps, AlertVariant,
};
pub use alert_dialog::{AlertDialog, AlertDialogAction, AlertDialogActionProps, AlertDialogProps};
pub use anchor::{
    use_anchor, Anchor, AnchorContext, AnchorItem, AnchorItemProps, AnchorProps, AnchorSection,
    AnchorSectionProps,
};
pub use aspect_ratio::{AspectRatio, AspectRatioProps};
pub use avatar::{Avatar, AvatarFallback, AvatarFallbackProps, AvatarProps};
pub use badge::{Badge, BadgeProps, BadgeVariant};
pub use bottom_navigation::{BottomNavigation, BottomNavigationItem, BottomNavigationProps};
pub use bottom_sheet::{
    BottomSheet, BottomSheetProps, BottomSheetTextInput, BottomSheetTextInputProps,
};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbItemProps, BreadcrumbProps};
pub use button::{Button, ButtonProps, ButtonSize, ButtonVariant};
pub use calendar::{Calendar, CalendarDate, CalendarLabels, CalendarProps, CalendarYearRange};
pub use calendar_plugin::{
    CalendarDayContext, CalendarDayDecoration, CalendarDayEvent, CalendarDayEventKind,
    CalendarDayEventResponse, CalendarDayStyle, CalendarMonthContext, CalendarMonthDecoration,
    CalendarPlugin, CalendarPluginLayout,
};
pub use card::{
    Card, CardContent, CardContentProps, CardDescription, CardDescriptionProps, CardFooter,
    CardFooterProps, CardHeader, CardHeaderProps, CardProps, CardTitle, CardTitleProps,
};
pub use carousel::{
    Carousel, CarouselControlsPlacement, CarouselIndicatorVariant, CarouselProps, CarouselStyle,
    CarouselTransitionCurve,
};
pub use chart::{Chart, ChartCard, ChartCardProps, ChartProps};
pub use checkbox::{Checkbox, CheckboxProps};
#[cfg(feature = "code")]
pub use code::{Code, CodeProps, CodeStyle};
#[cfg(feature = "code")]
pub use code_highlight::{
    highlight_code, is_language_registered, register_highlight_configuration, register_language,
    reset_language_registry, supported_languages, unregister_language, CodeHighlightPalette,
    HighlightConfiguration, HighlightLine, HighlightSpan, Language, RegisterLanguageError,
    HIGHLIGHT_NAMES,
};
pub use collapsible::{Collapsible, CollapsibleProps};
pub use combobox::{Combobox, ComboboxProps};
pub use command::{Command, CommandProps};
pub use context_menu::{ContextMenu, ContextMenuEntry, ContextMenuProps};
pub use date_picker::{DatePicker, DatePickerProps};
pub use dialog::{
    use_dialog_close, Dialog, DialogClose, DialogFooter, DialogFooterProps, DialogHeader,
    DialogHeaderProps, DialogProps,
};
pub use drawer::{Drawer, DrawerProps};
pub use dropdown_menu::{DropdownMenu, DropdownMenuEntry, DropdownMenuProps};
pub use floating_layer::{FloatingAlign, FloatingLayer, FloatingLayerProps, FloatingSide};
pub use form::{
    Field, FieldContent, FieldContentProps, FieldDescription, FieldDescriptionProps, FieldError,
    FieldErrorProps, FieldGroup, FieldGroupProps, FieldLabel, FieldLabelProps, FieldLegend,
    FieldLegendProps, FieldLegendVariant, FieldOrientation, FieldProps, FieldSeparator,
    FieldSeparatorProps, FieldSet, FieldSetProps, FieldTitle, FieldTitleProps, Form, FormItem,
    FormItemProps, FormProps,
};
pub use guide::{
    Guide, GuideLabels, GuideProps, GuideSide, GuideStep, GuideStyle, GuideTarget, GuideTargetProps,
};
pub use hover_card::{HoverCard, HoverCardProps};
pub use input::{Input, InputMode, InputProps};
pub use input_otp::{InputOtp, InputOtpMode, InputOtpProps, InputOtpSeparator, InputOtpStyle};
pub use label::{Label, LabelProps};
#[cfg(feature = "markdown")]
pub use markdown::{
    Markdown, MarkdownAdmonitionLabels, MarkdownOptions, MarkdownProps, MarkdownStyle,
};
pub use menu_common::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, MenuActionEntry, MenuCheckboxEntry, MenuEntry,
    MenuLabelEntry, MenuRadioEntry, MenuStyle, MenuSubmenuEntry,
};
pub use menubar::{Menubar, MenubarEntry, MenubarMenuSpec, MenubarProps};
pub use navigation_menu::{
    NavigationItem, NavigationItemProps, NavigationMenu, NavigationMenuProps,
};
pub use pagination::{Pagination, PaginationProps};
pub use popover::{Popover, PopoverProps};
pub use progress::{Progress, ProgressProps};
pub use radio_group::{RadioGroup, RadioGroupProps};
pub use refresh::{
    InfiniteScroll, InfiniteScrollProps, LoadMoreIndicator, LoadMoreIndicatorProps, LoadMoreLabels,
    LoadMoreState, PullToRefresh, PullToRefreshProps,
};
pub use resizable::{Resizable, ResizableProps};
pub use scroll_area::{ScrollArea, ScrollAreaProps};
pub use secure_keyboard::{
    SecureKeyboard, SecureKeyboardLabels, SecureKeyboardMode, SecureKeyboardProps,
    SecureKeyboardSheet, SecureKeyboardSheetProps, SecureKeyboardStyle,
};
pub use select::{Select, SelectProps};
pub use separator::{Separator, SeparatorProps};
pub use sheet::{Sheet, SheetProps};
pub use sidebar::{Sidebar, SidebarItem, SidebarItemProps, SidebarProps};
pub use skeleton::{Skeleton, SkeletonProps};
pub use slider::{
    MultiSlider, MultiSliderProps, RangeSlider, RangeSliderProps, Slider, SliderOrientation,
    SliderProps, SliderStyle,
};
pub use spinner::{Spinner, SpinnerProps};
pub use surfaces::{
    Sonner, SonnerPosition, SonnerProps, SonnerStyle, SonnerToast, Toast, ToastAppearance,
    ToastDestructive, ToastDestructiveProps, ToastProps, ToastStyle, ToastSwipeDirection,
    ToastVariant,
};
pub use switch::{Switch, SwitchProps};
pub use table::{Table, TableProps};
pub use tabs::{
    Tabs, TabsContent, TabsContentProps, TabsList, TabsListProps, TabsProps, TabsTrigger,
    TabsTriggerProps,
};
pub use text::{Text, TextProps, TextVariant};
pub use textarea::{Textarea, TextareaProps};
pub use time_picker::{TimePicker, TimePickerFormat, TimePickerLabels, TimePickerProps, TimeValue};
pub use timeline::{
    Timeline, TimelineAlign, TimelineContent, TimelineContentProps, TimelineDate,
    TimelineDateProps, TimelineHeader, TimelineHeaderProps, TimelineIndicator,
    TimelineIndicatorProps, TimelineItem, TimelineItemProps, TimelineOrientation, TimelineProps,
    TimelineSeparator, TimelineSeparatorProps, TimelineTitle, TimelineTitleProps,
};
pub use toggle::{Toggle, ToggleProps, ToggleVariant};
pub use toggle_group::{ToggleGroup, ToggleGroupProps};
pub use tooltip::{Tooltip, TooltipProps};
pub use watermark::{
    Watermark, WatermarkBlendMode, WatermarkFontStyle, WatermarkImage, WatermarkProps,
    WatermarkShadow, WatermarkSource, WatermarkStroke, WatermarkStyle,
};

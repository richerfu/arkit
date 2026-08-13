//! ColorUI-styled components. Same capability surface as `arkit_shadcn::components`;
//! only paint follows ColorUI. Extra ColorUI composites live alongside.

mod accordion;
mod alert_dialog;
mod avatar_group;
mod bar;
mod bottom_nav;
mod breadcrumb;
mod calendar;
mod carousel;
mod chat;
mod chrome;
mod collapsible;
mod combobox;
mod command;
mod date_picker;
mod dialog;
mod float;
mod form;
mod form_group;
mod indexes;
mod list;
mod load;
mod menu;
mod nav;
mod otp;
mod overlays;
mod pagination;
mod paint;
mod primitives;
mod radio;
mod select;
mod sidebar;
mod slider;
mod spinner;
mod steps;
mod table;
mod tabs;
mod tag;
mod text;
mod textarea;
mod time_picker;
mod timeline;
mod toast;
mod toggle;
mod toggle_group;

pub use primitives::*;

pub use accordion::Accordion;
pub use alert_dialog::{AlertDialog, AlertDialogAction};
pub use avatar_group::{AvatarGroup, AvatarGroupProps};
pub use bar::{Bar, BarKind, BarProps};
pub use bottom_nav::BottomNavigation;
pub use breadcrumb::Breadcrumb;
pub use calendar::Calendar;
pub use carousel::Carousel;
pub use chat::{Chat, ChatInfo, ChatInfoProps, ChatItem, ChatItemProps, ChatProps};
pub use collapsible::Collapsible;
pub use combobox::Combobox;
pub use command::Command;
pub use date_picker::DatePicker;
pub use dialog::{Dialog, DialogFooter, DialogHeader};
pub use float::{HoverCard, Popover, Tooltip};
pub use form::{Field, FieldLabel, Form};
pub use form_group::{FormGroup, FormGroupProps};
pub use indexes::{Indexes, IndexesProps};
pub use list::{
    GridItem, GridItemProps, GridList, GridListProps, List, ListItem, ListItemProps, ListKind,
    ListProps,
};
pub use load::{Load, LoadModal, LoadModalProps, LoadProps, LoadState};
pub use menu::{ContextMenu, DropdownMenu, Menubar};
pub use nav::{Nav, NavItem, NavItemProps, NavProps};
pub use otp::InputOtp;
pub use overlays::{BottomSheet, Drawer, Sheet};
pub use pagination::Pagination;
pub use radio::RadioGroup;
pub use select::Select;
pub use sidebar::{Sidebar, SidebarItem};
pub use slider::{MultiSlider, RangeSlider, Slider};
pub use spinner::Spinner;
pub use steps::{StepItem, StepItemProps, StepState, Steps, StepsProps};
pub use table::Table;
pub use tabs::{Tabs, TabsContent, TabsList, TabsListProps, TabsTrigger, TabsTriggerProps};
pub use tag::{Capsule, CapsuleProps, Tag, TagProps, TagSize};
pub use text::Text;
pub use textarea::Textarea;
pub use time_picker::TimePicker;
pub use timeline::{Timeline, TimelineItem, TimelineItemProps, TimelineProps};
pub use toast::{Sonner, Toast};
pub use toggle::Toggle;
pub use toggle_group::ToggleGroup;

pub use arkit_component::components::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, use_anchor, use_dialog_close, AccordionItemSpec,
    AccordionProps, AlertDialogActionProps, AlertDialogProps, Anchor, AnchorContext, AnchorItem,
    AnchorItemProps, AnchorProps, AnchorSection, AnchorSectionProps, AspectRatio, AspectRatioProps,
    BottomNavigationItem, BottomNavigationProps, BottomSheetProps, BottomSheetTextInput,
    BottomSheetTextInputProps, BreadcrumbItem, BreadcrumbItemProps, BreadcrumbProps, CalendarDate,
    CalendarDayContext, CalendarDayPlugin, CalendarLabels, CalendarProps, CalendarYearRange,
    CarouselControlsPlacement, CarouselIndicatorVariant, CarouselProps, CarouselStyle,
    CarouselTransitionCurve, Chart, ChartCard, ChartCardProps, ChartProps, CollapsibleProps,
    ComboboxProps, CommandProps, ContextMenuEntry, ContextMenuProps, DatePickerProps, DialogClose,
    DialogFooterProps, DialogHeaderProps, DialogProps, DrawerProps, DropdownMenuEntry,
    DropdownMenuProps, FieldContent, FieldContentProps, FieldDescription, FieldDescriptionProps,
    FieldError, FieldErrorProps, FieldGroup, FieldGroupProps, FieldLabelProps, FieldLegend,
    FieldLegendProps, FieldLegendVariant, FieldOrientation, FieldProps, FieldSeparator,
    FieldSeparatorProps, FieldSet, FieldSetProps, FieldTitle, FieldTitleProps, FloatingAlign,
    FloatingLayer, FloatingLayerProps, FloatingSide, FormItem, FormItemProps, FormProps, Guide,
    GuideLabels, GuideProps, GuideSide, GuideStep, GuideStyle, GuideTarget, GuideTargetProps,
    HoverCardProps, InfiniteScroll, InfiniteScrollProps, InputOtpMode, InputOtpProps,
    InputOtpSeparator, InputOtpStyle, LoadMoreIndicator, LoadMoreIndicatorProps, LoadMoreLabels,
    LoadMoreState, MenuActionEntry, MenuCheckboxEntry, MenuEntry, MenuLabelEntry, MenuRadioEntry,
    MenuStyle, MenuSubmenuEntry, MenubarEntry, MenubarMenuSpec, MenubarProps, MultiSliderProps,
    NavigationItem, NavigationItemProps, NavigationMenu, NavigationMenuProps, PaginationProps,
    PopoverProps, PullToRefresh, PullToRefreshProps, RadioGroupProps, RangeSliderProps, Resizable,
    ResizableProps, ScrollArea, ScrollAreaProps, SecureKeyboard, SecureKeyboardLabels,
    SecureKeyboardMode, SecureKeyboardProps, SecureKeyboardSheet, SecureKeyboardSheetProps,
    SecureKeyboardStyle, SelectProps, SheetProps, SidebarItemProps, SidebarProps,
    SliderOrientation, SliderProps, SliderStyle, SonnerPosition, SonnerProps, SonnerStyle,
    SonnerToast, SpinnerProps, TableProps, TabsContentProps, TabsProps, TextProps, TextVariant,
    TextareaProps, TimePickerFormat, TimePickerLabels, TimePickerProps, TimeValue, ToastAppearance,
    ToastDestructive, ToastDestructiveProps, ToastProps, ToastStyle, ToastSwipeDirection,
    ToastVariant, ToggleGroupProps, ToggleProps, ToggleVariant, TooltipProps, Watermark,
    WatermarkBlendMode, WatermarkFontStyle, WatermarkImage, WatermarkProps, WatermarkShadow,
    WatermarkSource, WatermarkStroke, WatermarkStyle,
};

#[cfg(feature = "code")]
pub use arkit_component::components::code_highlight;
#[cfg(feature = "code")]
pub use arkit_component::components::{
    highlight_code, is_language_registered, register_highlight_configuration, register_language,
    reset_language_registry, supported_languages, unregister_language, Code, CodeHighlightPalette,
    CodeProps, CodeStyle, HighlightConfiguration, HighlightLine, HighlightSpan, Language,
    RegisterLanguageError, HIGHLIGHT_NAMES,
};

#[cfg(feature = "markdown")]
pub use arkit_component::components::{
    Markdown, MarkdownAdmonitionLabels, MarkdownOptions, MarkdownProps, MarkdownStyle,
};

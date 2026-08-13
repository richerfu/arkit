//! shadcn-styled components. Same capability surface as `arkit_colorui`;
//! paint follows official shadcn/ui (new-york-v4) with mobile touch mapping.

mod accordion;
mod alert_dialog;
mod chrome;
mod combobox;
mod date_picker;
mod dialog;
mod float;
mod form;
mod inject;
mod misc;
mod overlays;
mod pagination;
mod paint;
mod primitives;
mod radio;
mod select;
mod table;
mod tabs;
mod text;
mod textarea;
mod toggle;

pub use primitives::*;

pub use accordion::Accordion;
pub use alert_dialog::{AlertDialog, AlertDialogAction};
pub use combobox::Combobox;
pub use date_picker::DatePicker;
pub use dialog::{Dialog, DialogFooter, DialogHeader};
pub use float::{HoverCard, Popover, Tooltip};
pub use form::{Field, FieldLabel, Form};
pub use inject::{Calendar, Carousel, InputOtp, MultiSlider, RangeSlider, Slider, Sonner, Toast};
pub use misc::{
    BottomNavigation, Breadcrumb, Collapsible, Sidebar, SidebarItem, Spinner, ToggleGroup,
};
pub use overlays::{BottomSheet, Drawer, Sheet};
pub use pagination::Pagination;
pub use radio::RadioGroup;
pub use select::Select;
pub use table::Table;
pub use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub use text::Text;
pub use textarea::Textarea;
pub use toggle::Toggle;

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
    ComboboxProps, Command, CommandProps, ContextMenu, ContextMenuEntry, ContextMenuProps,
    DatePickerProps, DialogClose, DialogFooterProps, DialogHeaderProps, DialogProps, DrawerProps,
    DropdownMenu, DropdownMenuEntry, DropdownMenuProps, FieldContent, FieldContentProps,
    FieldDescription, FieldDescriptionProps, FieldError, FieldErrorProps, FieldGroup,
    FieldGroupProps, FieldLabelProps, FieldLegend, FieldLegendProps, FieldLegendVariant,
    FieldOrientation, FieldProps, FieldSeparator, FieldSeparatorProps, FieldSet, FieldSetProps,
    FieldTitle, FieldTitleProps, FloatingAlign, FloatingLayer, FloatingLayerProps, FloatingSide,
    FormItem, FormItemProps, FormProps, Guide, GuideLabels, GuideProps, GuideSide, GuideStep,
    GuideStyle, GuideTarget, GuideTargetProps, HoverCardProps, InfiniteScroll, InfiniteScrollProps,
    InputOtpMode, InputOtpProps, InputOtpSeparator, InputOtpStyle, LoadMoreIndicator,
    LoadMoreIndicatorProps, LoadMoreLabels, LoadMoreState, MenuActionEntry, MenuCheckboxEntry,
    MenuEntry, MenuLabelEntry, MenuRadioEntry, MenuStyle, MenuSubmenuEntry, Menubar, MenubarEntry,
    MenubarMenuSpec, MenubarProps, MultiSliderProps, NavigationItem, NavigationItemProps,
    NavigationMenu, NavigationMenuProps, PaginationProps, PopoverProps, PullToRefresh,
    PullToRefreshProps, RadioGroupProps, RangeSliderProps, Resizable, ResizableProps, ScrollArea,
    ScrollAreaProps, SecureKeyboard, SecureKeyboardLabels, SecureKeyboardMode, SecureKeyboardProps,
    SecureKeyboardSheet, SecureKeyboardSheetProps, SecureKeyboardStyle, SelectProps, SheetProps,
    SidebarItemProps, SidebarProps, SliderOrientation, SliderProps, SliderStyle, SonnerPosition,
    SonnerProps, SonnerStyle, SonnerToast, SpinnerProps, TableProps, TabsContentProps,
    TabsListProps, TabsProps, TabsTriggerProps, TextProps, TextVariant, TextareaProps, TimePicker,
    TimePickerFormat, TimePickerLabels, TimePickerProps, TimeValue, ToastAppearance,
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

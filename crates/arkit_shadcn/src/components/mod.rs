//! Shadcn component registry — 50 components migrated from the original Elm
//! builder API to dioxus 0.7 `#[component]` + `rsx!`.
//!
//! Each component preserves its original styling logic (colors/radii/spacing/
//! typography/shadow) but renders via dioxus elements instead of the old
//! `Component`/`Element` builder.

pub(crate) const ARKUI_BUTTON_TYPE_NORMAL: i32 = 0;
pub(crate) const ARKUI_BORDER_STYLE_SOLID: i32 = 0;

mod accordion;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod avatar;
mod badge;
mod bottom_sheet;
mod breadcrumb;
mod button;
mod calendar;
mod card;
mod carousel;
mod chart;
mod checkbox;
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
mod hover_card;
mod input;
mod input_otp;
mod label;
mod menu_common;
mod menubar;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod radio_group;
mod resizable;
mod scroll_area;
mod select;
mod separator;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod surfaces;
mod switch;
mod table;
mod tabs;
mod text;
mod textarea;
mod toggle;
mod toggle_group;
mod tooltip;

pub use accordion::{Accordion, AccordionItemSpec, AccordionProps};
pub use alert::{
    Alert, AlertDescription, AlertDescriptionProps, AlertList, AlertListProps, AlertProps,
    AlertTitle, AlertTitleProps, AlertVariant,
};
pub use alert_dialog::{AlertDialog, AlertDialogProps};
pub use aspect_ratio::{AspectRatio, AspectRatioProps};
pub use avatar::{Avatar, AvatarFallback, AvatarFallbackProps, AvatarProps};
pub use badge::{Badge, BadgeProps, BadgeVariant};
pub use bottom_sheet::{
    BottomSheet, BottomSheetProps, BottomSheetTextInput, BottomSheetTextInputProps,
};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbItemProps, BreadcrumbProps};
pub use button::{Button, ButtonProps, ButtonSize, ButtonVariant};
pub use calendar::{Calendar, CalendarProps};
pub use card::{
    Card, CardContent, CardContentProps, CardDescription, CardDescriptionProps, CardFooter,
    CardFooterProps, CardHeader, CardHeaderProps, CardProps, CardTitle, CardTitleProps,
};
pub use carousel::{Carousel, CarouselProps};
pub use chart::{Chart, ChartCard, ChartCardProps, ChartProps};
pub use checkbox::{Checkbox, CheckboxProps};
pub use collapsible::{Collapsible, CollapsibleProps};
pub use combobox::{Combobox, ComboboxProps};
pub use command::{Command, CommandProps};
pub use context_menu::{ContextMenu, ContextMenuEntry, ContextMenuProps};
pub use date_picker::{DatePicker, DatePickerProps};
pub use dialog::{
    Dialog, DialogFooter, DialogFooterProps, DialogHeader, DialogHeaderProps, DialogProps,
};
pub use drawer::{Drawer, DrawerProps};
pub use dropdown_menu::{DropdownMenu, DropdownMenuEntry, DropdownMenuProps};
pub use floating_layer::{FloatingAlign, FloatingLayer, FloatingLayerProps, FloatingSide};
pub use form::{Form, FormItem, FormItemProps, FormProps};
pub use hover_card::{HoverCard, HoverCardProps};
pub use input::{Input, InputProps};
pub use input_otp::{InputOtp, InputOtpProps};
pub use label::{Label, LabelProps};
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
pub use resizable::{Resizable, ResizableProps};
pub use scroll_area::{ScrollArea, ScrollAreaProps};
pub use select::{Select, SelectProps};
pub use separator::{Separator, SeparatorProps};
pub use sheet::{Sheet, SheetProps};
pub use sidebar::{Sidebar, SidebarItem, SidebarItemProps, SidebarProps};
pub use skeleton::{Skeleton, SkeletonProps};
pub use slider::{Slider, SliderProps};
pub use surfaces::{
    Sonner, SonnerProps, Toast, ToastDestructive, ToastDestructiveProps, ToastProps,
};
pub use switch::{Switch, SwitchProps};
pub use table::{Table, TableProps};
pub use tabs::{
    Tabs, TabsContent, TabsContentProps, TabsList, TabsListProps, TabsProps, TabsTrigger,
    TabsTriggerProps,
};
pub use text::{Text, TextProps, TextVariant};
pub use textarea::{Textarea, TextareaProps};
pub use toggle::{Toggle, ToggleProps, ToggleVariant};
pub use toggle_group::{ToggleGroup, ToggleGroupProps};
pub use tooltip::{Tooltip, TooltipProps};

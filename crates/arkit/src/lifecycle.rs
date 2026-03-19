use openharmony_ability::{
    AvoidAreaInfo, Configuration, ContentRect, Event, InputEvent, IntervalInfo, Size,
};

#[derive(Clone)]
pub enum LifecycleEvent {
    WindowCreate,
    WindowDestroy,
    WindowRedraw(IntervalInfo),
    WindowResize(Size),
    ContentRectChange(ContentRect),
    AvoidAreaChange(AvoidAreaInfo),
    ConfigChanged(Configuration),
    LowMemory,
    Start,
    GainedFocus,
    LostFocus,
    Resume,
    Pause,
    Stop,
    SaveState,
    Create,
    Destroy,
    SurfaceCreate,
    SurfaceDestroy,
    Input(InputEvent),
    KeyboardEvent(i32),
    UserEvent,
}

impl LifecycleEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleEvent::WindowCreate => "WindowCreate",
            LifecycleEvent::WindowDestroy => "WindowDestroy",
            LifecycleEvent::WindowRedraw(_) => "WindowRedraw",
            LifecycleEvent::WindowResize(_) => "WindowResize",
            LifecycleEvent::ContentRectChange(_) => "ContentRectChange",
            LifecycleEvent::AvoidAreaChange(_) => "AvoidAreaChange",
            LifecycleEvent::ConfigChanged(_) => "ConfigChanged",
            LifecycleEvent::LowMemory => "LowMemory",
            LifecycleEvent::Start => "Start",
            LifecycleEvent::GainedFocus => "GainedFocus",
            LifecycleEvent::LostFocus => "LostFocus",
            LifecycleEvent::Resume => "Resume",
            LifecycleEvent::Pause => "Pause",
            LifecycleEvent::Stop => "Stop",
            LifecycleEvent::SaveState => "SaveState",
            LifecycleEvent::Create => "Create",
            LifecycleEvent::Destroy => "Destroy",
            LifecycleEvent::SurfaceCreate => "SurfaceCreate",
            LifecycleEvent::SurfaceDestroy => "SurfaceDestroy",
            LifecycleEvent::Input(_) => "Input",
            LifecycleEvent::KeyboardEvent(_) => "KeyboardEvent",
            LifecycleEvent::UserEvent => "UserEvent",
        }
    }
}

pub(crate) fn from_ability_event(event: Event<'_>) -> LifecycleEvent {
    match event {
        Event::WindowCreate => LifecycleEvent::WindowCreate,
        Event::WindowDestroy => LifecycleEvent::WindowDestroy,
        Event::WindowRedraw(interval) => LifecycleEvent::WindowRedraw(interval),
        Event::WindowResize(size) => LifecycleEvent::WindowResize(size),
        Event::ContentRectChange(content_rect) => LifecycleEvent::ContentRectChange(content_rect),
        Event::AvoidAreaChange(avoid_area) => LifecycleEvent::AvoidAreaChange(avoid_area),
        Event::ConfigChanged(config) => LifecycleEvent::ConfigChanged(config),
        Event::LowMemory => LifecycleEvent::LowMemory,
        Event::Start => LifecycleEvent::Start,
        Event::GainedFocus => LifecycleEvent::GainedFocus,
        Event::LostFocus => LifecycleEvent::LostFocus,
        Event::Resume(_) => LifecycleEvent::Resume,
        Event::Pause => LifecycleEvent::Pause,
        Event::Stop => LifecycleEvent::Stop,
        Event::SaveState(_) => LifecycleEvent::SaveState,
        Event::Create => LifecycleEvent::Create,
        Event::Destroy => LifecycleEvent::Destroy,
        Event::SurfaceCreate => LifecycleEvent::SurfaceCreate,
        Event::SurfaceDestroy => LifecycleEvent::SurfaceDestroy,
        Event::Input(input) => LifecycleEvent::Input(input),
        Event::KeyboardEvent(height) => LifecycleEvent::KeyboardEvent(height),
        Event::UserEvent => LifecycleEvent::UserEvent,
    }
}

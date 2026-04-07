use napi_ohos::Result;
use ohos_arkui_binding::common::handle::ArkUIHandle;
use openharmony_ability::OpenHarmonyApp;

use crate::{mount_application, Application, ApplicationRuntime, Element, Runtime, Task};

pub trait EntryHandle {
    fn unmount(&self) -> Result<()>;
}

impl EntryHandle for Runtime {
    fn unmount(&self) -> Result<()> {
        Runtime::unmount(self)
    }
}

impl<P> EntryHandle for ApplicationRuntime<P>
where
    P: crate::Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    fn unmount(&self) -> Result<()> {
        ApplicationRuntime::unmount(self)
    }
}

pub type MountedEntry = Box<dyn EntryHandle>;

pub trait EntryPoint {
    fn mount(self, slot: ArkUIHandle, app: OpenHarmonyApp) -> Result<MountedEntry>;
}

impl EntryPoint for Element {
    fn mount(self, slot: ArkUIHandle, app: OpenHarmonyApp) -> Result<MountedEntry> {
        Ok(Box::new(Runtime::new_static(slot, app, self)?))
    }
}

impl<State, Message, View, Update, Boot> EntryPoint
    for Application<State, Message, View, Update, Boot>
where
    State: 'static,
    Message: Clone + 'static,
    Boot: Fn() -> (State, Task<Message>) + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element + 'static,
{
    fn mount(self, slot: ArkUIHandle, app: OpenHarmonyApp) -> Result<MountedEntry> {
        Ok(Box::new(mount_application(slot, app, self)?))
    }
}

pub fn mount_entry(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    entry: impl EntryPoint,
) -> Result<MountedEntry> {
    entry.mount(slot, app)
}

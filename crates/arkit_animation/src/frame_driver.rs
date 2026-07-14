use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arkit_animation_core::TimePoint;
use arkit_hooks::ArkNodeRef;

use crate::AnimationHost;

type FrameSource = Rc<dyn Fn(TimePoint)>;

pub struct FrameDriver {
    host: Rc<AnimationHost>,
    node: ArkNodeRef,
    requested: Cell<bool>,
    sources: RefCell<Vec<Option<FrameSource>>>,
}

pub(crate) struct FrameSourceSubscription {
    driver: Weak<FrameDriver>,
    id: usize,
}

impl FrameDriver {
    pub fn new(host: Rc<AnimationHost>, node: ArkNodeRef) -> Rc<Self> {
        host.set_context_node_provider(Rc::new(move || node.peek()));
        Rc::new(Self {
            host,
            node,
            requested: Cell::new(false),
            sources: RefCell::new(Vec::new()),
        })
    }

    pub(crate) fn subscribe(self: &Rc<Self>, source: FrameSource) -> FrameSourceSubscription {
        let mut sources = self.sources.borrow_mut();
        let id = if let Some((id, slot)) = sources
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(source);
            id
        } else {
            sources.push(Some(source));
            sources.len() - 1
        };
        FrameSourceSubscription {
            driver: Rc::downgrade(self),
            id,
        }
    }

    pub fn request(self: &Rc<Self>) {
        let Some(node) = self.node.peek() else {
            return;
        };
        if self.requested.replace(true) {
            return;
        }
        self.host.record_frame_callback_requested();
        let driver = self.clone();
        let result = node.borrow().post_frame_callback(move |timestamp, _| {
            driver.requested.set(false);
            let frame_time = TimePoint::from_nanos(timestamp);
            driver.flush_sources(frame_time);
            if let Err(error) = driver.host.tick(frame_time) {
                ohos_hilog_binding::error(format!("arkit_animation frame failed: {error}"));
                return;
            }
            if driver.host.has_work() {
                driver.request();
            }
        });
        if let Err(error) = result {
            self.requested.set(false);
            ohos_hilog_binding::error(format!(
                "arkit_animation post frame callback failed: {error:?}"
            ));
        }
    }

    fn flush_sources(&self, frame_time: TimePoint) {
        let source_count = self.sources.borrow().len();
        for index in 0..source_count {
            let source = self.sources.borrow().get(index).and_then(Clone::clone);
            if let Some(source) = source {
                source(frame_time);
            }
        }
    }
}

impl Drop for FrameSourceSubscription {
    fn drop(&mut self) {
        if let Some(driver) = self.driver.upgrade() {
            if let Some(slot) = driver.sources.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

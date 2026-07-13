use std::cell::Cell;
use std::rc::Rc;

use arkit_animation_core::TimePoint;
use arkit_hooks::HostNode;

use crate::AnimationHost;

pub struct FrameDriver {
    host: Rc<AnimationHost>,
    requested: Cell<bool>,
}

impl FrameDriver {
    pub fn new(host: Rc<AnimationHost>) -> Rc<Self> {
        Rc::new(Self {
            host,
            requested: Cell::new(false),
        })
    }

    pub fn request(self: &Rc<Self>, node: &HostNode) {
        if self.requested.replace(true) {
            return;
        }
        self.host.record_frame_callback_requested();
        let driver = self.clone();
        let next_node = node.clone();
        let result = node.borrow().post_frame_callback(move |timestamp, _| {
            driver.requested.set(false);
            if let Err(error) = driver.host.tick(TimePoint::from_nanos(timestamp)) {
                ohos_hilog_binding::error(format!("arkit_animation frame failed: {error}"));
                return;
            }
            if driver.host.has_work() {
                driver.request(&next_node);
            }
        });
        if let Err(error) = result {
            self.requested.set(false);
            ohos_hilog_binding::error(format!(
                "arkit_animation post frame callback failed: {error:?}"
            ));
        }
    }
}

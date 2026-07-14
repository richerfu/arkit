use std::rc::Rc;

use arkit_animation_core::TimePoint;

#[derive(Clone, Default)]
pub(crate) struct AnimationCallbacks {
    pub begin: Option<Rc<dyn Fn()>>,
    pub before_update: Option<Rc<dyn Fn(TimePoint)>>,
    pub update: Option<Rc<dyn Fn(f32)>>,
    pub render: Option<Rc<dyn Fn()>>,
    pub looped: Option<Rc<dyn Fn(u32)>>,
    pub complete: Option<Rc<dyn Fn()>>,
    pub cancel: Option<Rc<dyn Fn()>>,
    pub pause: Option<Rc<dyn Fn()>>,
}

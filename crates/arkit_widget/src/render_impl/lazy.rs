use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use super::*;

pub struct Lazy<Message, AppTheme, Dependency, View, Content> {
    dependency: Dependency,
    view: View,
    _marker: PhantomData<fn() -> (Message, AppTheme, Content)>,
}

#[derive(Debug, Clone)]
struct LazyState {
    dependency_hash: u64,
    rendered: bool,
    rebuild: bool,
    overlay_count: usize,
}

impl LazyState {
    fn new(dependency_hash: u64) -> Self {
        Self {
            dependency_hash,
            rendered: false,
            rebuild: true,
            overlay_count: 0,
        }
    }
}

pub fn lazy<Message, AppTheme, Dependency, View, Content>(
    dependency: Dependency,
    view: View,
) -> Lazy<Message, AppTheme, Dependency, View, Content>
where
    Dependency: Hash + 'static,
    View: Fn(&Dependency) -> Content + 'static,
    Content: Into<Element<Message, AppTheme>> + 'static,
{
    Lazy {
        dependency,
        view,
        _marker: PhantomData,
    }
}

fn dependency_hash<Dependency: Hash>(dependency: &Dependency) -> u64 {
    let mut hasher = DefaultHasher::new();
    dependency.hash(&mut hasher);
    hasher.finish()
}

fn lazy_state<'a>(tree: &'a mut advanced::widget::Tree, dependency_hash: u64) -> &'a mut LazyState {
    tree.state()
        .get_or_insert_with(|| LazyState::new(dependency_hash))
}

impl<Message, AppTheme, Dependency, View, Content> advanced::Widget<Message, AppTheme, Renderer>
    for Lazy<Message, AppTheme, Dependency, View, Content>
where
    Message: 'static,
    AppTheme: 'static,
    Dependency: Hash + 'static,
    View: Fn(&Dependency) -> Content + 'static,
    Content: Into<Element<Message, AppTheme>> + 'static,
{
    fn state(&self) -> advanced::widget::State {
        advanced::widget::State::new(Box::new(LazyState::new(dependency_hash(&self.dependency))))
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        tree.set_persistent_key(None);

        let hash = dependency_hash(&self.dependency);
        let state = lazy_state(tree, hash);
        if !state.rendered || state.dependency_hash != hash {
            state.dependency_hash = hash;
            state.rebuild = true;
        }
    }

    fn cached_body(
        &self,
        tree: &mut advanced::widget::Tree,
        _renderer: &Renderer,
    ) -> Option<advanced::Body<Message, AppTheme, Renderer>> {
        let hash = dependency_hash(&self.dependency);
        let should_rebuild = {
            let state = lazy_state(tree, hash);
            state.rebuild || !state.rendered
        };

        if !should_rebuild {
            let overlays = lazy_state(tree, hash).overlay_count;
            return Some(advanced::Body::Retain { overlays });
        }

        let body = (self.view)(&self.dependency).into();
        let state = lazy_state(tree, hash);
        state.dependency_hash = hash;
        state.rendered = true;
        state.rebuild = false;
        Some(advanced::Body::Rebuild(body))
    }

    fn cache_overlay_count(&self, tree: &mut advanced::widget::Tree, count: usize) {
        let hash = dependency_hash(&self.dependency);
        lazy_state(tree, hash).overlay_count = count;
    }
}

impl<Message, AppTheme, Dependency, View, Content>
    From<Lazy<Message, AppTheme, Dependency, View, Content>> for Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
    Dependency: Hash + 'static,
    View: Fn(&Dependency) -> Content + 'static,
    Content: Into<Element<Message, AppTheme>> + 'static,
{
    fn from(value: Lazy<Message, AppTheme, Dependency, View, Content>) -> Self {
        arkit_core::Element::new(value)
    }
}

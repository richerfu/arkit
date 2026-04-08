use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub struct SubscriptionHandle {
    cleanup: Option<Box<dyn FnOnce()>>,
}

impl SubscriptionHandle {
    pub fn none() -> Self {
        Self { cleanup: None }
    }

    pub fn from_cleanup(cleanup: impl FnOnce() + 'static) -> Self {
        Self {
            cleanup: Some(Box::new(cleanup)),
        }
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

pub struct SubscriptionRecipe<Message> {
    pub id: String,
    pub start: Box<dyn FnOnce(Rc<dyn Fn(Message)>) -> SubscriptionHandle>,
}

pub struct Subscription<Message> {
    recipes: Vec<SubscriptionRecipe<Message>>,
}

impl<Message: Send + 'static> Subscription<Message> {
    pub fn none() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }

    pub fn run<I>(builder: fn() -> I) -> Self
    where
        I: IntoIterator<Item = Message> + 'static,
    {
        let id = format!("run:{:p}", builder as *const ());
        Self::run_recipe(id, move |emit| {
            for message in builder() {
                emit(message);
            }
            SubscriptionHandle::none()
        })
    }

    pub fn run_with<D, I>(data: D, builder: fn(&D) -> I) -> Self
    where
        D: Hash + 'static,
        I: IntoIterator<Item = Message> + 'static,
    {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let id = format!("run_with:{:p}:{}", builder as *const (), hasher.finish());
        Self::run_recipe(id, move |emit| {
            for message in builder(&data) {
                emit(message);
            }
            SubscriptionHandle::none()
        })
    }

    pub fn batch(subscriptions: impl IntoIterator<Item = Self>) -> Self {
        let mut recipes = Vec::new();
        for subscription in subscriptions {
            recipes.extend(subscription.recipes);
        }
        Self { recipes }
    }

    pub fn with<A>(self, value: A) -> Subscription<(A, Message)>
    where
        A: Clone + Hash + 'static,
    {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let suffix = hasher.finish();
        let mut recipes = Vec::with_capacity(self.recipes.len());

        for recipe in self.recipes {
            let SubscriptionRecipe { id, start } = recipe;
            let value = value.clone();
            recipes.push(SubscriptionRecipe {
                id: format!("{id}:{suffix}"),
                start: Box::new(move |emit| {
                    start(Rc::new(move |message| emit((value.clone(), message))))
                }),
            });
        }

        Subscription { recipes }
    }

    pub fn map<B: Send + 'static>(
        self,
        map: impl Fn(Message) -> B + Clone + 'static,
    ) -> Subscription<B> {
        let mut recipes = Vec::with_capacity(self.recipes.len());

        for recipe in self.recipes {
            let SubscriptionRecipe { id, start } = recipe;
            let map = map.clone();
            recipes.push(SubscriptionRecipe {
                id,
                start: Box::new(move |emit| start(Rc::new(move |message| emit(map(message))))),
            });
        }

        Subscription { recipes }
    }

    pub fn filter_map<B: Send + 'static>(
        self,
        map: impl Fn(Message) -> Option<B> + Clone + 'static,
    ) -> Subscription<B> {
        let mut recipes = Vec::with_capacity(self.recipes.len());

        for recipe in self.recipes {
            let SubscriptionRecipe { id, start } = recipe;
            let map = map.clone();
            recipes.push(SubscriptionRecipe {
                id,
                start: Box::new(move |emit| {
                    start(Rc::new(move |message| {
                        if let Some(mapped) = map(message) {
                            emit(mapped);
                        }
                    }))
                }),
            });
        }

        Subscription { recipes }
    }

    pub fn units(&self) -> usize {
        self.recipes.len()
    }

    pub fn into_recipes(self) -> Vec<SubscriptionRecipe<Message>> {
        self.recipes
    }

    fn run_recipe(
        id: impl Into<String>,
        start: impl FnOnce(Rc<dyn Fn(Message)>) -> SubscriptionHandle + 'static,
    ) -> Self {
        Self {
            recipes: vec![SubscriptionRecipe {
                id: id.into(),
                start: Box::new(start),
            }],
        }
    }
}

impl<Message: Send + 'static> Default for Subscription<Message> {
    fn default() -> Self {
        Self::none()
    }
}

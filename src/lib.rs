use std::ops::Deref;
use std::sync::Arc;

pub use async_trait::async_trait;
pub use futures::future::BoxFuture;
pub use futures::FutureExt;
pub use twilight::gateway::Event;

pub mod middlewares;

use tokio::sync::RwLock;
use typemap::{ShareMap, TypeMap};

use twilight::cache::InMemoryCache;
use typemap::Key;

#[derive(Clone)]
pub struct Cache(InMemoryCache);

impl Key for Cache {
    type Value = Self;
}

impl Deref for Cache {
    type Target = InMemoryCache;

    fn deref(&self) -> &InMemoryCache {
        &self.0
    }
}

/// Context-wrapped event that provides some niceties for storing
/// additional data for the duration of the event as it moves through
/// the middleware stack.
pub struct Context<Event> {
    /// Wrapped event.
    pub event: Event,
    local: Arc<RwLock<ShareMap>>,
}

impl<Event> Context<Event> {
    pub fn new(event: Event) -> Context<Event> {
        Self {
            event,
            local: Arc::new(RwLock::new(TypeMap::custom())),
        }
    }

    pub fn local(&self) -> Arc<RwLock<ShareMap>> {
        self.local.clone()
    }
}

#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
    async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>);
}

pub struct Next<'a, State> {
    pub next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<'a, State: 'static> Next<'a, State> {
    pub async fn run(mut self, state: Arc<State>, event: Context<Event>) {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(state, event, self).await;
        }
    }
}

type Middlewares<State> = Arc<Vec<Arc<dyn Middleware<State>>>>;

pub struct MiddlewareStack<State: Send + Sync + 'static> {
    state: Arc<State>,
    middlewares: Middlewares<State>,
}

impl<State: Send + Sync + 'static> MiddlewareStack<State> {
    pub fn new(state: State) -> Self {
        Self {
            state: Arc::new(state),
            middlewares: Arc::new(vec![]),
        }
    }

    pub fn push<M>(mut self, middleware: M) -> Self
    where
        M: Middleware<State>,
    {
        let middlewares =
            Arc::get_mut(&mut self.middlewares).expect("Unable to register new middleware.");
        middlewares.push(Arc::new(middleware));
        self
    }

    pub fn handle(&self, event: Event) {
        let state = self.state.clone();
        let middlewares = self.middlewares.clone();

        tokio::spawn(async move {
            let next = Next {
                next_middleware: middlewares.as_slice(),
            };

            next.run(state, Context::new(event)).await;
        });
    }
}

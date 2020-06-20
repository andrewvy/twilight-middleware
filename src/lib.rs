use std::sync::Arc;

mod middleware;

pub use async_trait::async_trait;
pub use futures::future::BoxFuture;
pub use middleware::{Command, Context, Middleware, Next};

use twilight::gateway::Event;

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

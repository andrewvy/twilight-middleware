use std::sync::Arc;

use async_trait::async_trait;
use twilight::cache::InMemoryCache;

use crate::{Cache, Context, Event, Middleware, Next};

pub struct CacheMiddleware {
  cache: Cache,
}

impl CacheMiddleware {
  pub fn new(cache: InMemoryCache) -> Self {
    Self {
      cache: Cache(cache),
    }
  }
}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for CacheMiddleware {
  async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>) {
    self
      .cache
      .update(&ctx.event)
      .await
      .expect("Could not cache event");

    let local = ctx.local();

    local.write().await.insert::<Cache>(self.cache.clone());

    next.run(state, ctx).await;
  }
}

use std::ops::Deref;
use std::sync::Arc;

use twilight::cache::InMemoryCache;
use typemap::Key;

use crate::{async_trait, Context, Event, Middleware, Next};

#[derive(Clone)]
pub struct Cache(InMemoryCache);

impl Key for Cache {
  type Value = Self;
}

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

impl Deref for Cache {
  type Target = InMemoryCache;

  fn deref(&self) -> &InMemoryCache {
    &self.0
  }
}

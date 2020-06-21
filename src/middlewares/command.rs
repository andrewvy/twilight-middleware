use std::sync::Arc;

use futures::future::BoxFuture;
use twilight::gateway::Event;

use crate::{async_trait, Context, Middleware, Next};

type BoxCommandHook<State> = Box<
  dyn for<'fut> Fn(String, Arc<State>, Context<Event>, Next<'fut, State>) -> BoxFuture<'fut, ()>
    + Send
    + Sync,
>;

pub struct Command<State: Send + Sync + 'static> {
  prefix: String,
  handler: BoxCommandHook<State>,
}

impl<State: Send + Sync + 'static> Command<State> {
  pub fn new(prefix: &str, handler: BoxCommandHook<State>) -> Self {
    Self {
      prefix: prefix.to_owned(),
      handler,
    }
  }
}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for Command<State> {
  async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>) {
    match ctx.event {
      Event::MessageCreate(ref msg) => {
        if msg.content.starts_with(&self.prefix) {
          let idx = self.prefix.len();
          let command = msg.content.get(idx..).unwrap_or("");
          (self.handler)(command.to_owned(), state, ctx, next).await;
        } else {
          next.run(state, ctx).await;
        }
      }
      _ => {
        next.run(state, ctx).await;
      }
    }
  }
}

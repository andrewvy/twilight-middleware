use std::sync::Arc;

use twilight::{gateway::Event, model::id::UserId};

use crate::{Cache, Context, Middleware, Next};
use async_trait::async_trait;

pub struct IgnoreSelf {}

#[async_trait]
impl<State: Send + Sync + 'static> Middleware<State> for IgnoreSelf {
  async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>) {
    let local = ctx.local();
    let reader = local.read().await;
    let cache = reader.get::<Cache>().expect("No cache provided.");
    let me = cache.current_user().await.expect("No current user");
    let user_id = me.as_ref().map(|user| user.id).unwrap_or(UserId(0)).clone();

    match ctx.event {
      Event::MessageCreate(ref msg) => {
        if msg.author.id != user_id {
          next.run(state, ctx).await;
        }
      }
      _ => next.run(state, ctx).await,
    }
  }
}

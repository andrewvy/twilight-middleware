use futures::FutureExt;
use std::{env, error::Error, sync::Arc};
use tokio::stream::StreamExt;
use twilight::{
  cache::{
    twilight_cache_inmemory::config::{EventType, InMemoryConfigBuilder},
    InMemoryCache,
  },
  gateway::{
    cluster::{Cluster, ClusterConfig},
    Event,
  },
  http::Client as HttpClient,
  model::gateway::GatewayIntents,
};

use twilight_middleware::{
  middlewares::{CacheMiddleware, Command},
  BoxFuture, Context, MiddlewareStack, Next,
};

pub struct State {
  http: HttpClient,
}

fn ping<'a>(
  _args: String,
  state: Arc<State>,
  ctx: Context<Event>,
  next: Next<'a, State>,
) -> BoxFuture<'a, ()> {
  async move {
    match ctx.event {
      Event::MessageCreate(ref msg) => {
        let _ = state
          .http
          .create_message(msg.channel_id)
          .content("Pong!")
          .unwrap()
          .await;
      }
      _ => {}
    }

    next.run(state, ctx).await;
  }
  .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  let token = env::var("DISCORD_TOKEN")?;

  let config = ClusterConfig::builder(&token)
    // Use intents to only listen to GUILD_MESSAGES events
    .intents(Some(
      GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES,
    ))
    .build();

  // Start up the cluster
  let cluster = Cluster::new(config).await?;

  let cluster_spawn = cluster.clone();

  tokio::spawn(async move {
    cluster_spawn.up().await;
  });

  // The http client is seperate from the gateway,
  // so startup a new one
  let http = HttpClient::new(&token);

  let mut events = cluster.events().await;

  let cache_config = InMemoryConfigBuilder::new()
    .event_types(
      EventType::MESSAGE_CREATE
        | EventType::MESSAGE_DELETE
        | EventType::MESSAGE_DELETE_BULK
        | EventType::MESSAGE_UPDATE,
    )
    .build();

  let cache = InMemoryCache::from(cache_config);

  // Setup middleware stack
  let middleware_stack = MiddlewareStack::new(State { http: http.clone() })
    .push(CacheMiddleware::new(cache))
    .push(Command::new("!ping", Box::new(ping)));

  // Startup an event loop for each event in the event stream
  while let Some((_, event)) = events.next().await {
    middleware_stack.handle(event);
  }

  Ok(())
}

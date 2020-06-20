use async_trait::async_trait;
use std::{env, error::Error, sync::Arc};
use tokio::stream::StreamExt;
use twilight::{
  gateway::{
    cluster::{config::ShardScheme, Cluster, ClusterConfig},
    Event,
  },
  http::Client as HttpClient,
  model::gateway::GatewayIntents,
};

use twilight_middleware::{Context, Middleware, MiddlewareStack, Next};

pub struct State {
  http: HttpClient,
}

pub struct PrefixMiddleware {
  prefix: String,
}

impl PrefixMiddleware {
  pub fn new(prefix: &str) -> Self {
    PrefixMiddleware {
      prefix: prefix.to_owned(),
    }
  }
}

#[async_trait]
impl Middleware<State> for PrefixMiddleware {
  async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>) {
    match ctx.event {
      Event::MessageCreate(ref msg) => {
        if msg.content.starts_with(&self.prefix) {
          next.run(state, ctx).await;
        }
      }
      _ => {}
    }
  }
}

pub struct PingCommand {}

#[async_trait]
impl Middleware<State> for PingCommand {
  async fn handle<'a>(&'a self, state: Arc<State>, ctx: Context<Event>, next: Next<'a, State>) {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  let token = env::var("DISCORD_TOKEN")?;

  // This is also the default.
  let scheme = ShardScheme::Auto;

  let config = ClusterConfig::builder(&token)
    .shard_scheme(scheme)
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

  // Setup middleware stack
  let middleware_stack = MiddlewareStack::new(State { http: http.clone() })
    .push(PrefixMiddleware::new("!ping"))
    .push(PingCommand {});

  // Startup an event loop for each event in the event stream
  while let Some((_, event)) = events.next().await {
    middleware_stack.handle(event);
  }

  Ok(())
}

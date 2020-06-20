use std::sync::Arc;

use tokio::sync::RwLock;
use typemap::{ShareMap, TypeMap};

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

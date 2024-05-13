pub mod dns;
mod resolve_event;
mod resolved_status;
pub mod server;

pub use resolve_event::{DefaultResolveEvent, ResolveEvent};
pub use server::{Config, Server, ServerConfigBuilder};

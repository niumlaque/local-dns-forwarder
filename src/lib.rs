mod allowlist;
pub mod dns;
pub mod error;
pub mod logger;
mod resolve_event;
mod resolved_data;
mod resolved_status;
pub mod server;

pub use allowlist::AllowList;
pub use error::{Error, Result};
pub use resolve_event::{DefaultResolveEvent, ResolveEvent, TracingResolveEvent};
pub use resolved_data::ResolvedData;
pub use resolved_status::ResolvedStatus;
pub use server::{Config, Server, ServerConfigBuilder};

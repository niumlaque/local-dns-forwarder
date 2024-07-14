pub mod dns;
pub mod error;
mod filters;
pub mod logger;
mod resolve_event;
mod resolved_data;
mod resolved_status;
pub mod server;

pub use error::{Error, Result};
pub use filters::{CheckList, CompositeCheckList};
pub use resolve_event::{DefaultResolveEvent, ResolveEvent, TracingResolveEvent};
pub use resolved_data::ResolvedData;
pub use resolved_status::ResolvedStatus;
pub use server::{Config, Server, ServerConfigBuilder};

pub fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    if let Some(git_hash) = option_env!("GIT_HASH") {
        format!("{version} ({git_hash})")
    } else {
        version.into()
    }
}

pub fn get_build_mode() -> &'static str {
    if cfg!(debug_assertions) {
        "Debug"
    } else {
        "Release"
    }
}

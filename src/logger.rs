use std::path::Path;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter, fmt, reload};
use tracing_subscriber::{prelude::*, Registry};

pub type ReloadHandle = reload::Handle<LevelFilter, Registry>;

pub struct LogContext {
    pub reload_handle: ReloadHandle,
    pub file_guard: Option<non_blocking::WorkerGuard>,
}

impl LogContext {
    fn new(reload_handle: ReloadHandle, file_guard: Option<non_blocking::WorkerGuard>) -> Self {
        Self {
            reload_handle,
            file_guard,
        }
    }
}

pub fn init(level: tracing::Level, log_dir: Option<impl AsRef<Path>>) -> LogContext {
    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(true)
        .with_ansi(true)
        .compact();
    let filter = filter::LevelFilter::from_level(level);
    let (filter_layer, reload_handle) = reload::Layer::new(filter);
    let subscriber = tracing_subscriber::registry().with(filter_layer);
    let stdout_layer = fmt::Layer::default().event_format(format.clone());
    let subscriber = subscriber.with(stdout_layer);
    if let Some(log_dir) = log_dir {
        let file_appender =
            RollingFileAppender::new(Rotation::DAILY, log_dir, "local-fqdn-filter.log");
        let (non_blocking_file_appender, guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = fmt::Layer::default()
            .event_format(format)
            .with_writer(non_blocking_file_appender);
        subscriber.with(file_layer).init();
        LogContext::new(reload_handle, Some(guard))
    } else {
        subscriber.init();
        LogContext::new(reload_handle, None)
    }
}

use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter, fmt, reload};
use tracing_subscriber::{prelude::*, Registry};

const LOGFILE_PREFIX: &str = "local-fqdn-filter.log";

pub type ReloadHandle = reload::Handle<LevelFilter, Registry>;

pub struct LogContext {
    pub reload_handle: ReloadHandle,
    pub file_guard: Option<non_blocking::WorkerGuard>,
    log_dir: Option<PathBuf>,
}

impl LogContext {
    fn new(
        reload_handle: ReloadHandle,
        file_guard: Option<non_blocking::WorkerGuard>,
        log_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            reload_handle,
            file_guard,
            log_dir,
        }
    }

    pub fn remove_old_logs(&self) -> Result<()> {
        if let Some(log_dir) = self.log_dir.as_ref() {
            let threshold = chrono::Local::now()
                .checked_sub_months(chrono::Months::new(1))
                .ok_or_else(|| Error::DeleteLogFiles)?;
            let threshold = threshold.date_naive();
            tracing::info!(
                "Deleting old log files from {} (threshold={})",
                log_dir.display(),
                threshold
            );
            let entries = fs::read_dir(log_dir)?;
            let mut targets = Vec::default();

            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with(LOGFILE_PREFIX) {
                        targets.push(name);
                    }
                }
            }

            targets.sort();
            let mut deleted = 0;
            for v in targets
                .iter()
                .filter(|x| x.len() > LOGFILE_PREFIX.len() + 1)
            {
                let file_date = &v[LOGFILE_PREFIX.len() + 1..];
                match chrono::NaiveDate::parse_from_str(file_date, "%Y-%m-%d") {
                    Ok(file_date) => {
                        if file_date < threshold {
                            let filepath = Path::join(log_dir, v);
                            match std::fs::remove_file(&filepath) {
                                Ok(_) => {
                                    tracing::info!("Deleted {}", filepath.display());
                                    deleted += 1;
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to delete {} ({e})", filepath.display());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get date from file name ({v}: {e})")
                    }
                }
            }

            tracing::info!("Deleted {deleted} log files");
        }

        Ok(())
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
        let log_dir = log_dir.as_ref();
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, LOGFILE_PREFIX);
        let (non_blocking_file_appender, guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = fmt::Layer::default()
            .event_format(format)
            .with_writer(non_blocking_file_appender);
        subscriber.with(file_layer).init();
        LogContext::new(reload_handle, Some(guard), Some(log_dir.to_path_buf()))
    } else {
        subscriber.init();
        LogContext::new(reload_handle, None, None)
    }
}

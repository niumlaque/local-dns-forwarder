use anyhow::Result;
use clap::Parser;
use local_fqdn_filter::logger::{self, LogContext};
use local_fqdn_filter::{get_build_mode, get_version, CheckList, CompositeCheckList, Server};
use local_fqdn_filter::{ResolveEvent, ResolvedData, ResolvedStatus};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Parser)]
struct Cli {
    /// Path to config file
    #[arg(short = 'f', long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct GeneralConfig {
    loglevel: Option<String>,
    log_dir: Option<PathBuf>,
    output_allowed_log: Option<bool>,
    output_nochecked_log: Option<bool>,
    allowlist: Option<PathBuf>,
    denylist: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            loglevel: Some("info".into()),
            log_dir: None,
            output_allowed_log: Some(false),
            output_nochecked_log: Some(false),
            allowlist: None,
            denylist: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    general: Option<GeneralConfig>,
    server: local_fqdn_filter::Config,
}

impl Config {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        use std::fs;
        let text = fs::read_to_string(path)?;
        let mut config = toml::from_str::<Config>(&text)?;
        if config.general.is_none() {
            config.general = Some(GeneralConfig::default());
        }
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: Some(GeneralConfig::default()),
            server: local_fqdn_filter::Config::default(),
        }
    }
}

struct InnerConfig {
    loglevel: tracing::Level,
    log_dir: Option<PathBuf>,
    output_allowed_log: bool,
    output_nochecked_log: bool,
    allowlist: Option<PathBuf>,
    denylist: Option<PathBuf>,
    server: local_fqdn_filter::Config,
}

impl InnerConfig {
    fn new(config: Config) -> Result<Self> {
        use std::str::FromStr;
        let general = config.general.unwrap_or_default();
        let loglevel = if let Some(level) = general.loglevel.as_ref() {
            tracing::Level::from_str(level)?
        } else {
            tracing::Level::INFO
        };
        let log_dir = if let Some(log_dir) = general.log_dir {
            Some(absolute_path(log_dir)?)
        } else {
            None
        };
        let allowlist = if let Some(allowlist) = general.allowlist {
            Some(absolute_path(allowlist)?)
        } else {
            None
        };
        let denylist = if let Some(denylist) = general.denylist {
            Some(absolute_path(denylist)?)
        } else {
            None
        };
        Ok(Self {
            loglevel,
            log_dir,
            output_allowed_log: general.output_allowed_log.unwrap_or(false),
            output_nochecked_log: general.output_nochecked_log.unwrap_or(false),
            allowlist,
            denylist,
            server: config.server,
        })
    }
}

pub struct LFFResolveEvent {
    threshold: usize,
    count_map: Arc<RwLock<std::collections::HashMap<u64, usize>>>,
    output_allowed_log: bool,
    output_nochecked_log: bool,
}

impl LFFResolveEvent {
    fn new(threshold: usize, output_allowed_log: bool, output_nochecked_log: bool) -> Self {
        Self {
            threshold,
            count_map: Default::default(),
            output_allowed_log,
            output_nochecked_log,
        }
    }

    fn code(d: &ResolvedData) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        d.req_qtype.hash(&mut hasher);
        d.req_name.hash(&mut hasher);
        hasher.finish()
    }
}

impl ResolveEvent for LFFResolveEvent {
    fn resolving(&self, _name: &str) {}

    fn resolved(&self, status: ResolvedStatus) {
        let mut ignore = false;
        let code = match &status {
            ResolvedStatus::Allow(v) => {
                ignore = !self.output_allowed_log;
                Self::code(v)
            }
            ResolvedStatus::AllowButError(v, _) => {
                ignore = !self.output_allowed_log;
                Self::code(v)
            }
            ResolvedStatus::Deny(v, _) => Self::code(v),
            ResolvedStatus::NoCheck(v) => {
                ignore = !self.output_nochecked_log;
                Self::code(v)
            }
            ResolvedStatus::NoCheckButError(v, _) => {
                ignore = !self.output_nochecked_log;
                Self::code(v)
            }
        };

        if ignore {
            return;
        }

        if let Ok(mut count_map) = self.count_map.write() {
            let count = count_map.entry(code).or_insert(0);
            if *count < self.threshold {
                tracing::info!("{status}");
            }
            if *count + 1 == self.threshold {
                tracing::warn!("Since the number of requests has exceeded the threshold, log output will be suppressed from now on")
            }
            *count = count.saturating_add(1);
        } else {
            tracing::info!("{status}");
        }
    }

    fn error(&self, message: impl AsRef<str>) {
        tracing::error!("{}", message.as_ref());
    }
}

fn get_config_path(cli: &Cli) -> Result<PathBuf> {
    if let Some(config_path) = cli.config.as_ref() {
        absolute_path(config_path)
    } else {
        Ok(Path::new("/etc/lff/config.toml").to_path_buf())
    }
}

fn get_checklist(config: &InnerConfig) -> Result<CompositeCheckList> {
    if let Some(allowlist_path) = config.allowlist.as_ref() {
        tracing::info!("[Config] AllowList: {}", allowlist_path.display());
    } else {
        tracing::info!("[Config] AllowList: None");
    }
    if let Some(denylist_path) = config.denylist.as_ref() {
        tracing::info!("[Config] DenyList: {}", denylist_path.display());
    } else {
        tracing::info!("[Config] DenyList: None");
    }
    let allowlist = if let Some(path) = config.allowlist.as_ref() {
        CheckList::text(path.to_path_buf())?
    } else {
        CheckList::in_memory()
    };

    let denylist = if let Some(path) = config.denylist.as_ref() {
        CheckList::text(path.to_path_buf())?
    } else {
        CheckList::in_memory()
    };
    tracing::info!("[Config] Allowing {} FQDN(s)", allowlist.count());
    tracing::info!("[Config] Denying {} FQDN(s)", denylist.count());

    Ok(CompositeCheckList::new(allowlist, denylist))
}

fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let ret = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(ret)
}

fn on_ipctl(
    command: &str,
    reload_handle: &logger::ReloadHandle,
    checklist: Arc<RwLock<CompositeCheckList>>,
) -> String {
    use std::str::FromStr;
    let inv = || {
        let msg = format!("Invalid command: {command}");
        tracing::error!("{msg}");
        msg
    };

    let splitted = command.split(' ').collect::<Vec<_>>();
    if splitted.is_empty() {
        return inv();
    }

    match splitted[0].to_lowercase().as_ref() {
        "log" => {
            if splitted.len() < 2 {
                return inv();
            }

            if let Ok(level) = tracing::Level::from_str(splitted[1]) {
                match reload_handle.modify(|y| *y = level.into()) {
                    Ok(_) => {
                        let msg = format!("Log level is changed to {level}");
                        tracing::info!("{msg}");
                        msg
                    }
                    Err(e) => {
                        let msg = format!("Failed to change log lebel to {level}");
                        tracing::error!("{msg} ({e})");
                        msg
                    }
                }
            } else {
                let msg = format!("Failed to convert {} to log level", splitted[1]);
                tracing::error!("{msg}");
                msg
            }
        }
        "allow" => {
            if splitted.len() < 2 {
                return inv();
            }

            let fqdn = splitted[1];
            let msg = if let Ok(mut checklist) = checklist.write() {
                let msg = if checklist.allowlist.add(fqdn) > 0 {
                    format!("Add {fqdn} to AllowList")
                } else {
                    format!("{fqdn} is already in AllowList")
                };

                tracing::info!("{msg}");
                msg
            } else {
                let msg = format!("Failed to add {fqdn} to AllowList");
                tracing::error!("{msg}");
                msg
            };

            msg
        }
        "deny" => {
            if splitted.len() < 2 {
                return inv();
            }

            let fqdn = splitted[1];
            let msg = if let Ok(mut checklist) = checklist.write() {
                let msg = if checklist.allowlist.delete(fqdn) > 0 {
                    format!("Remove {fqdn} from AllowList")
                } else {
                    format!("{fqdn} is not in AllowList")
                };

                tracing::info!("{msg}");
                msg
            } else {
                let msg = format!("Failed to add {fqdn} to AllowList");
                tracing::error!("{msg}");
                msg
            };

            msg
        }
        "save" => {
            let msg = if let Ok(checklist) = checklist.read() {
                match checklist.allowlist.save() {
                    Ok(()) => {
                        let msg = "AllowList is saved";
                        tracing::info!("{msg}");
                        msg.into()
                    }
                    Err(e) => {
                        let msg = "Failed to save allowlist";
                        tracing::error!("{msg}: {e}");
                        msg.into()
                    }
                }
            } else {
                let msg = "Failed to save allowlist";
                tracing::error!("{msg}: Could not get read lock");
                msg.into()
            };
            msg
        }
        "list" => {
            let msg = if let Ok(checklist) = checklist.read() {
                let mut names = Vec::with_capacity(checklist.allowlist.count());
                for name in checklist.allowlist.iter() {
                    names.push(name);
                }

                tracing::info!("Returned the list of FQDN(s)");
                names.join("\n").to_string()
            } else {
                let msg = "Failed to get allowlist";
                tracing::error!("{msg}: Could not get read lock");
                msg.into()
            };
            msg
        }
        _ => inv(),
    }
}

async fn exec(
    config: InnerConfig,
    reload_handle: local_fqdn_filter::logger::ReloadHandle,
) -> Result<()> {
    tracing::info!("[Config] Output Allowed Log: {}", config.output_allowed_log);
    tracing::info!(
        "[Config] Output NoChecked Log: {}",
        config.output_nochecked_log
    );
    tracing::info!("[Config] Server: {}", config.server);

    let checklist = get_checklist(&config)?;
    let addr = "127.0.0.1:60001"
        .parse()
        .expect("Failed to parse endpoint for ipctl Server");

    let server = Server::from_config(config.server)
        .checklist(checklist)
        .event(LFFResolveEvent::new(
            3,
            config.output_allowed_log,
            config.output_nochecked_log,
        ))
        .build();

    let checklist = Arc::clone(&server.checklist);
    let handler =
        ipctl::Server::new(move |x: &str| on_ipctl(x, &reload_handle, Arc::clone(&checklist)))
            .spawn_and_serve(addr);
    tracing::info!("Start Local FQDN Filter");
    server.serve()?;

    handler.join().await?;
    Ok(())
}

fn exit<R>(e: anyhow::Error) -> R {
    eprintln!("{e}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let version = format!("llf ({}) - {}", get_build_mode(), get_version());
    println!("{version}");
    let cli = Cli::parse();
    let config_path = get_config_path(&cli).unwrap_or_else(exit);
    println!("[Config] Config path: {}", config_path.display());
    let config = Config::load(config_path).unwrap_or_else(exit);
    let config = InnerConfig::new(config).unwrap_or_else(exit);
    let log = logger::init(config.loglevel, config.log_dir.as_ref());
    println!("[Config] Log Level: {}", config.loglevel);

    let code = {
        let LogContext {
            reload_handle,
            file_guard: _file_guard,
        } = log;

        tracing::info!("{version}");
        match exec(config, reload_handle).await {
            Ok(_) => 0,
            Err(e) => {
                tracing::error!(
                    "The application has encountered a critical error and will now terminate"
                );
                tracing::error!("{e}");
                1
            }
        }
    };

    std::process::exit(code);
}

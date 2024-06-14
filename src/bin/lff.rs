use anyhow::Result;
use clap::Parser;
use local_fqdn_filter::logger::{self, LogContext};
use local_fqdn_filter::{AllowList, Server, TracingResolveEvent};
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
    allowlist: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            loglevel: Some("info".into()),
            log_dir: None,
            allowlist: None,
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
    allowlist: Option<PathBuf>,
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
        Ok(Self {
            loglevel,
            log_dir,
            allowlist,
            server: config.server,
        })
    }
}

fn get_config_path(cli: &Cli) -> Result<PathBuf> {
    if let Some(config_path) = cli.config.as_ref() {
        absolute_path(config_path)
    } else {
        Ok(Path::new("/etc/lff/config.toml").to_path_buf())
    }
}

fn get_allowlist(config: &InnerConfig) -> Result<AllowList> {
    let allowlist = if let Some(path) = config.allowlist.as_ref() {
        AllowList::text(path.to_path_buf())?
    } else {
        AllowList::in_memory()
    };

    Ok(allowlist)
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
    allowlist: Arc<RwLock<AllowList>>,
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
            let msg = if let Ok(mut allowlist) = allowlist.write() {
                let msg = if allowlist.add(fqdn) > 0 {
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
            let msg = if let Ok(mut allowlist) = allowlist.write() {
                let msg = if allowlist.delete(fqdn) > 0 {
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
            let msg = if let Ok(allowlist) = allowlist.read() {
                match allowlist.save() {
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
            let msg = if let Ok(allowlist) = allowlist.read() {
                let mut names = Vec::with_capacity(allowlist.count());
                for name in allowlist.iter() {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = get_config_path(&cli)?;
    println!("[Config] Config path: {}", config_path.display());
    let config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        println!("[Config] Config file not found");
        println!("[Config] Load default config");
        Config::default()
    };
    let config = InnerConfig::new(config)?;
    let log = logger::init(config.loglevel, config.log_dir.as_ref());
    println!("[Config] Log Level: {}", config.loglevel);

    tracing::info!("[Config] Server: {}", config.server);
    if let Some(allowlist_path) = config.allowlist.as_ref() {
        tracing::info!("[Config] AllowList: {}", allowlist_path.display());
    } else {
        tracing::info!("[Config] AllowList: None");
    }
    let allowlist = get_allowlist(&config)?;
    tracing::info!("[Config] Allowing {} FQDN(s)", allowlist.count());

    let LogContext {
        reload_handle,
        file_guard: _file_guard,
    } = log;

    let addr = "127.0.0.1:60001"
        .parse()
        .expect("Failed to parse endpoint for ipctl Server");

    tracing::info!("Start Local FQDN Filter");
    let server = Server::from_config(config.server)
        .allowlist(allowlist)
        .event(TracingResolveEvent)
        .build();

    let allowlist = Arc::clone(&server.allowlist);
    let handler =
        ipctl::Server::new(move |x: &str| on_ipctl(x, &reload_handle, Arc::clone(&allowlist)))
            .spawn_and_serve(addr);
    server.serve()?;

    handler.join().await?;
    Ok(())
}

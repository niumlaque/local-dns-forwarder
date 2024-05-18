use anyhow::Result;
use clap::Parser;
use local_fqdn_filter::{logger, Server, TracingResolveEvent};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
    use std::env;
    if let Some(config_path) = cli.config.as_ref() {
        absolute_path(config_path)
    } else {
        let mut path = env::current_exe()?;
        path.pop();
        path.push("local-fqdn-filter.toml");
        Ok(path)
    }
}

fn get_allowlist(config: &InnerConfig) -> Result<HashMap<String, ()>> {
    let allowlist = if let Some(path) = config.allowlist.as_ref() {
        let mut allowlist = HashMap::new();
        for line in BufReader::new(File::open(path)?).lines() {
            allowlist.insert(line?, ());
        }
        allowlist
    } else {
        HashMap::new()
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

fn main() -> Result<()> {
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
    let _log = logger::init(config.loglevel, config.log_dir.as_ref());
    println!("[Config] Log Level: {}", config.loglevel);

    tracing::info!("[Config] Server: {}", config.server);
    if let Some(allowlist_path) = config.allowlist.as_ref() {
        tracing::info!("[Config] AllowList: {}", allowlist_path.display());
    } else {
        tracing::info!("[Config] AllowList: None");
    }
    let allowlist = get_allowlist(&config)?;
    tracing::info!("[Config] Allowing {} FQDN(s)", allowlist.len());

    tracing::info!("Start Local FQDN Filter");
    Server::from_config(config.server)
        .allowlist(allowlist)
        .event(TracingResolveEvent)
        .build()
        .serve()?;

    Ok(())
}

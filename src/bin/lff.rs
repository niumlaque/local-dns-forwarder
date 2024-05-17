use anyhow::Result;
use clap::Parser;
use local_fqdn_filter::Server;
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

#[derive(Debug, Deserialize, Default)]
struct Config {
    server: local_fqdn_filter::Config,
    allowlist: Option<PathBuf>,
}

impl Config {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        use std::fs;
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }
}

fn get_config(cli: &Cli) -> Result<Config> {
    use std::env;
    let config_path = if let Some(config_path) = cli.config.as_ref() {
        config_path.clone()
    } else {
        let mut path = env::current_exe()?;
        path.pop();
        path.push("local-fqdn-filter.toml");
        path
    };

    if config_path.exists() {
        println!("[Config] Loading Config File ({})", config_path.display());
        Config::load(config_path)
    } else {
        println!("[Config] Loading Default Config");
        Ok(Config::default())
    }
}

fn get_allowlist(config: &Config) -> Result<HashMap<String, ()>> {
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = get_config(&cli)?;
    println!("[Config] Server: {}", config.server);
    if let Some(allowlist_path) = config.allowlist.as_ref() {
        println!("[Config] AllowList: {}", allowlist_path.display());
    } else {
        println!("[Config] AllowList: None");
    }
    let allowlist = get_allowlist(&config)?;
    println!("[Config] Allowing {} FQDN(s)", allowlist.len());

    println!("Start Local FQDN Filter");
    Server::from_config(config.server)
        .allowlist(allowlist)
        .build()
        .serve()?;

    Ok(())
}

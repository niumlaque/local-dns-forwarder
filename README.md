# Local FQDN Filter
This application is a local DNS server written in Rust, created as a project to help me learn the Rust programming language.
The server operates by resolving domain names based on a predefined allowlist of Fully Qualified Domain Names (FQDNs).

## Features

- **Allowlist-Based Resolution**: The server reads a list of allowed FQDNs at startup.
- **Selective Resolution**: Only resolves domain names that are on the allowlist.
- **Configurable Upstream DNS**: For allowed domains, the server forwards the request to a specified upstream DNS server.
- **Ignore Non-Allowlisted Domains**: Any request for a domain not on the allowlist is ignored.

## Installation

To run this application, ensure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

Clone this repository and navigate to the project directory, and build the project using Cargo:
```sh
glt clone https://github.com/niumlaque/local-fqdn-filter.git
cd local-fqdn-filter.git
cargo build --release
```

## Configuration

The application reads configuration settings from a `local-fqdn-filter.toml` file, which can be placed in the same directory as the executable or specified at runtime using the `-f` flag like so: `local-fqdn-filter -f /path/to/config.toml`.

The `config.toml` file should have the following structure:
```toml
# Path to the allowlist file containing allowed FQDNs, one per line
allowlist = "allowlist.txt"

[server]
# The address the application will bind to
address = "127.0.0.1"
# The port the application will listen on
port = 53
# The default upstream DNS server for resolving allowed domains
default_dns_server = "8.8.8.8"
```

The allowlist.txt file should contain a list of allowed FQDNs, one per line:
```txt
www.debian.org
www.rust-lang.org
```

## Usage
Run the application using Cargo:
```sh
cargo run --release -- -f /path/to/config.toml
```
The server will start and begin listening for DNS queries. It will only process requests for domains listed in allowlist.txt and forward them to the specified upstream DNS server. All other requests will be ignored.

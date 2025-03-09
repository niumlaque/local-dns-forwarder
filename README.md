# Local DNS Forwarder
This application is created as a project to help me learn the Rust programming language.
The server filters requests based on a predefined allowlist of Fully Qualified Domain Names (FQDNs).

## Features
- FQDNs listed in the denylist are unconditionally not resolved
- Only FQDNs listed in the allowlist are resolved

## Installation
To install this application, ensure you have Rust installed.

Clone this repository, navigate to the project directory, and build the project using the Makefile:
```sh
$ git clone https://github.com/niumlaque/local-dns-forwarder.git
$ cd local-dns-forwarder
$ make
$ sudo make install
$ sudo make install-config
```

## Configuration
The application reads configuration settings from a `/etc/ldf/config.toml` file, which can be placed in the same directory as the executable or specified at runtime using the `-f` flag like so: `ldf -f /path/to/config.toml`.

The `config.toml` file should have the following structure:
```toml
[general]
# Path to the allowlist file containing allowed FQDNs, one per line (Option)
allowlist = "allowlist.txt"
# Path to the denylist file containing FQDNs to deny, one per line (Option)
denylist = "denylist.txt"
# Log level: options are "trace", "debug", "info", "warn", "error" (Option)
loglevel = "info"
# Directory where log files will be stored (Option)
log_dir = "/path/to/log/directory"
# Indicates whether to log allowed FQDNs (Option)
output_allowed_log = false
# Indicates whether to log FQDNs that are not checked (Option)
output_nochecked_log = false

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
Run the application:
```sh
# $ make
# $ sudo make install
# $ sudo make install-config
# $ sudo cp /etc/resolv.conf /etc/resolv.conf.backup
# $ echo "nameserver 127.0.0.1" | sudo tee /etc/resolv.conf
$ sudo ldf
```
The server will start and begin listening for DNS queries. It will only process requests for domains listed in allowlist.txt and forward them to the specified upstream DNS server. All other requests will be ignored.

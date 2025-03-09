INSTALL_DIR = /usr/local/bin
BINARY = ldf
CONFIG_DIR = /etc/ldf
SYSD_DIR = /etc/systemd/system

all: build

build:
	cargo build --release

install:
	install -d $(INSTALL_DIR)
	install -m 0755 target/release/$(BINARY) $(INSTALL_DIR)/

install-config:
	install -d $(CONFIG_DIR)
	install -m 0644 misc/config.toml.template  $(CONFIG_DIR)/config.toml

install-sysd:
	install -m 0644 misc/systemd/ldf.service $(SYSD_DIR)/ldf.service

clean:
	cargo clean

PHONY: all build install install-config install-sysd clean


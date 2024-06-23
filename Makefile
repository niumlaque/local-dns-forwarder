INSTALL_DIR = /usr/local/bin
BINARY = lff
CONFIG_DIR = /etc/lff
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
	install -m 0644 misc/systemd/lff.service $(SYSD_DIR)/lff.service

clean:
	cargo clean

PHONY: all build install install-config install-sysd clean


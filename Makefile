.PHONY: all build deps install uninstall reload-module status test clean

CARGO ?= cargo
SUDO ?= sudo
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
GUI_DIR := $(REPO_ROOT)predator-sense-gui
INSTALLER_BIN := $(GUI_DIR)/installer/target/release/predator-sense-installer
APP_BIN := $(GUI_DIR)/target/release/predator-sense

all: build

deps:
	@./scripts/install.sh

build:
	@echo "Building installer multicall binary..."
	$(CARGO) build --release --manifest-path $(GUI_DIR)/installer/Cargo.toml
	@echo "Building desktop GUI application..."
	$(CARGO) build --release --manifest-path $(GUI_DIR)/Cargo.toml

install: build
	@echo "Running Predator Sense installation..."
	$(SUDO) $(INSTALLER_BIN) --install

uninstall: build
	$(SUDO) $(INSTALLER_BIN) --uninstall

reload-module: build
	$(SUDO) $(INSTALLER_BIN) --reload-module

status:
	@if [ -x "/opt/predator-sense/predator-sense-installer" ]; then \
		/opt/predator-sense/predator-sense-installer --status; \
	elif [ -x "$(INSTALLER_BIN)" ]; then \
		$(INSTALLER_BIN) --status; \
	else \
		echo "Installer binary not found. Run 'make build' first."; \
	fi

test:
	$(CARGO) test --manifest-path $(GUI_DIR)/Cargo.toml
	$(CARGO) test --manifest-path $(GUI_DIR)/installer/Cargo.toml

clean:
	$(CARGO) clean --manifest-path $(GUI_DIR)/Cargo.toml
	$(CARGO) clean --manifest-path $(GUI_DIR)/installer/Cargo.toml

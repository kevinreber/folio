# Folio Makefile
#
# Common development tasks for the Folio project.
#
# Usage:
#   make install    - Build and install folio locally
#   make test       - Run all tests
#   make clean      - Clean build artifacts

BINARY_NAME := folio
INSTALL_DIR := $(HOME)/.local/bin
CARGO := cargo

.PHONY: all build release install uninstall test test-install test-quick clean fmt lint check help

# Default target
all: build

# Build web UI (requires Node.js)
web-ui:
	@cd web-ui && npm install && npm run build

# Build debug binary (builds web UI first)
build: web-ui
	$(CARGO) build

# Build release binary (builds web UI first)
release: web-ui
	$(CARGO) build --release

# Install to ~/.local/bin
install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"
	@echo "Make sure $(INSTALL_DIR) is in your PATH"

# Install to /usr/local/bin (requires sudo)
install-global: release
	@sudo cp target/release/$(BINARY_NAME) /usr/local/bin/$(BINARY_NAME)
	@sudo chmod +x /usr/local/bin/$(BINARY_NAME)
	@echo "Installed $(BINARY_NAME) to /usr/local/bin"

# Uninstall from ~/.local/bin
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(INSTALL_DIR)"

# Run unit tests
test:
	$(CARGO) test

# Run installation tests
test-install: release
	@chmod +x scripts/test-install.sh
	@./scripts/test-install.sh

# Run quick installation tests (skip build)
test-quick:
	@chmod +x scripts/test-install.sh
	@./scripts/test-install.sh --quick

# Run all tests
test-all: test test-install

# Clean build artifacts
clean:
	$(CARGO) clean
	@rm -rf $(HOME)/.folio-install

# Format code
fmt:
	$(CARGO) fmt

# Check formatting
fmt-check:
	$(CARGO) fmt --check

# Run clippy lints
lint:
	$(CARGO) clippy -- -D warnings

# Run all checks (format + lint + test)
check: fmt-check lint test

# Development build and run
run:
	$(CARGO) run -- $(ARGS)

# Show help
help:
	@echo "Folio Makefile Targets"
	@echo ""
	@echo "  make build        - Build debug binary"
	@echo "  make release      - Build release binary"
	@echo "  make install      - Build and install to ~/.local/bin"
	@echo "  make install-global - Install to /usr/local/bin (sudo)"
	@echo "  make uninstall    - Remove from ~/.local/bin"
	@echo ""
	@echo "  make test         - Run unit tests"
	@echo "  make test-install - Run installation tests"
	@echo "  make test-quick   - Run quick install tests (no build)"
	@echo "  make test-all     - Run all tests"
	@echo ""
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Run clippy lints"
	@echo "  make check        - Run all checks (fmt + lint + test)"
	@echo ""
	@echo "  make clean        - Clean build artifacts"
	@echo "  make help         - Show this help"

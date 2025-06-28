# Rust Project Makefile

# Default target
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
RUSTUP := rustup

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Help target
.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}'

# Build targets
.PHONY: build
build: ## Build the main TUI project
	@echo "$(GREEN)Building main TUI project...$(NC)"
	$(CARGO) build

.PHONY: build-all
build-all: ## Build all workspace members
	@echo "$(GREEN)Building all workspace members...$(NC)"
	$(CARGO) build --workspace

# Development targets
.PHONY: run-tui
run-tui: ## Run the TUI version
	@echo "$(GREEN)Running TUI version...$(NC)"
	$(CARGO) run

.PHONY: run-native
run-native: ## Run the native egui GUI version
	@echo "$(GREEN)Running native egui version...$(NC)"
	cd web-poc && $(CARGO) run --bin mathypad-web-poc

.PHONY: run-web
run-web: ## Build and serve the web WASM version
	@echo "$(GREEN)Building and serving web version...$(NC)"
	cd web-poc && ./run-web.sh

.PHONY: build-web
build-web: ## Build the web WASM version (without serving)
	@echo "$(GREEN)Building web WASM version...$(NC)"
	cd web-poc && ./build-web.sh

.PHONY: serve-web
serve-web: ## Serve the web version (assumes already built)
	@echo "$(GREEN)Serving web version on http://localhost:8080...$(NC)"
	cd web-poc && python3 -m http.server 8080

.PHONY: check
check: ## Check the main project for errors
	@echo "$(GREEN)Checking main project...$(NC)"
	$(CARGO) check

.PHONY: check-all
check-all: ## Check all workspace members for errors
	@echo "$(GREEN)Checking all workspace members...$(NC)"
	$(CARGO) check --workspace

.PHONY: test
test: ## Run tests for main project
	@echo "$(GREEN)Running tests for main project...$(NC)"
	$(CARGO) test

.PHONY: test-all
test-all: ## Run tests for all workspace members
	@echo "$(GREEN)Running tests for all workspace members...$(NC)"
	$(CARGO) test --workspace

# Code quality targets
.PHONY: fmt
fmt: ## Format code using cargo fmt (all workspace members)
	@echo "$(GREEN)Formatting code for all workspace members...$(NC)"
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Check if code is formatted (all workspace members)
	@echo "$(GREEN)Checking code formatting for all workspace members...$(NC)"
	$(CARGO) fmt --all -- --check

.PHONY: clippy
clippy: ## Run clippy linter on main project
	@echo "$(GREEN)Running clippy on main project...$(NC)"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: clippy-all
clippy-all: ## Run clippy linter on all workspace members
	@echo "$(GREEN)Running clippy on all workspace members...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: clippy-pedantic
clippy-pedantic: ## Run clippy with pedantic lints
	@echo "$(GREEN)Running clippy with pedantic lints...$(NC)"
	$(CARGO) clippy -- -W clippy::pedantic -D warnings

# Documentation targets
.PHONY: doc
doc: ## Generate documentation
	@echo "$(GREEN)Generating documentation...$(NC)"
	$(CARGO) doc

.PHONY: doc-open
doc-open: ## Generate and open documentation
	@echo "$(GREEN)Generating and opening documentation...$(NC)"
	$(CARGO) doc --open

# Dependency management
.PHONY: update
update: ## Update dependencies
	@echo "$(GREEN)Updating dependencies...$(NC)"
	$(CARGO) update

.PHONY: all
all: build-all test-all clippy-all fmt doc ## Run all main targets for all workspace members

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean

.PHONY: help build test clean run run-release clippy lint audit fmt fmt-check doc docker-build docker-run docker-stop docker-clean docker-run-interactive docker-logs docker-all all check ci watch install uninstall

# Configuration
DOCKER_REGISTRY := terminal.stream
IMAGE_NAME := last-mile
IMAGE_TAG := 0.1
DOCKER_IMAGE := $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
CONFIG_DIR := ./config

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development targets
build: ## Build the project in debug mode
	cargo build --workspace

build-release: ## Build the project in release mode
	cargo build --workspace --release

test: ## Run all tests
	cargo test --workspace

check: ## Run all checks (clippy, fmt, test, audit)
	cargo clippy --workspace --all-targets
	cargo fmt --all -- --check
	cargo test --workspace
	cargo audit

clippy: ## Run clippy linter
	cargo clippy --workspace --all-targets

lint: ## Run all linters (clippy + fmt check)
	cargo clippy --workspace --all-targets -- -D warnings
	cargo fmt --all -- --check

audit: ## Check dependencies for security vulnerabilities
	cargo audit

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

doc: ## Generate and open documentation
	cargo doc --workspace --open

clean: ## Clean build artifacts
	cargo clean

run: build ## Build and run the server in debug mode
	./target/debug/server --config-dir $(CONFIG_DIR)

run-release: build-release ## Build and run the server in release mode
	./target/release/server --config-dir $(CONFIG_DIR)

# Docker targets
docker-build: ## Build the Docker image (unified multi-stage build)
	docker build --tag $(DOCKER_IMAGE) -f Dockerfile .

docker-run: ## Run the Docker container with mounted config
	docker run --rm --detach \
		--name $(IMAGE_NAME) \
		-P \
		-v $(PWD)/config:/usr/local/share/tslm-config \
		$(DOCKER_IMAGE) \
		--config-dir=/usr/local/share/tslm-config

docker-run-interactive: ## Run the Docker container interactively
	docker run --rm -it \
		--name $(IMAGE_NAME) \
		-P \
		-v $(PWD)/config:/usr/local/share/tslm-config \
		$(DOCKER_IMAGE) \
		--config-dir=/usr/local/share/tslm-config

docker-stop: ## Stop the running Docker container
	docker stop $(IMAGE_NAME) || true

docker-logs: ## Show Docker container logs
	docker logs -f $(IMAGE_NAME)

docker-clean: ## Remove Docker images
	docker rmi $(DOCKER_IMAGE) || true

docker-all: docker-build ## Build Docker image (alias for docker-build)

# CI targets
ci: check build-release ## Run all CI checks

# Convenience targets
all: build test ## Build and test everything

watch: ## Watch for changes and run tests
	cargo watch -x 'test --workspace'

install: build-release ## Install the binary to cargo bin
	cargo install --path server

uninstall: ## Uninstall the binary
	cargo uninstall server

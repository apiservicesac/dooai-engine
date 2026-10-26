.PHONY: help build image run test fmt fmt-check lint models ci clean

.DEFAULT_GOAL := help

# Everything builds inside Docker through ./dev.sh: Rust does not have to be installed.

# rustfmt and clippy are added to the image as root; the files are handed back to the user afterwards.
UID_GID   := $(shell id -u):$(shell id -g)
RUST_ROOT := docker run --rm -v "$(CURDIR)":/p -w /p rust:1-trixie sh -c

GREEN  := \033[0;32m
YELLOW := \033[1;33m
NC     := \033[0m

help:
	@echo "$(GREEN)dooai-engine — Development Commands$(NC)\n"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / \
	    {printf "  $(YELLOW)%-12s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the release server
	./dev.sh cargo build --release -p engine-server

image: ## Build the server Docker image (dooai-engine)
	docker build -t dooai-engine -f docker/server.Dockerfile .

run: models ## Run the server on http://localhost:8080 (ENGINE_PORT=... to change)
	ENGINE_MODELS_DIR=/project/models ./dev.sh cargo run --release -p engine-server

models: ## Download the OCR models into models/ (PACKS="cyrillic korean" for only some language packs)
	./scripts/download-models.sh $(PACKS)

test: ## Run the unit tests (the OCR test also needs `make models`)
	ENGINE_MODELS_DIR=/project/models ./dev.sh cargo test --release --workspace

fmt: ## Format the code
	$(RUST_ROOT) "rustup component add rustfmt >/dev/null 2>&1 && cargo fmt --all && chown -R $(UID_GID) crates"

fmt-check: ## Check formatting without changes
	$(RUST_ROOT) "rustup component add rustfmt >/dev/null 2>&1 && cargo fmt --all --check"

lint: ## Run clippy; any warning fails
	$(RUST_ROOT) "rustup component add clippy >/dev/null 2>&1 && CARGO_HOME=/p/.cache/cargo cargo clippy --release --workspace --all-targets -- -D warnings; status=\$$?; chown -R $(UID_GID) target .cache; exit \$$status"

ci: fmt-check lint test ## Format check, clippy and tests, as CI runs them

clean: ## Remove build artifacts
	rm -rf target

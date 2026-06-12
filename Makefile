.DEFAULT_GOAL := help

# ——— Commands ———

.PHONY: help
help: ## Show available commands
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build all workspace crates
	cargo build --workspace

.PHONY: test
test: ## Run all tests
	cargo test --workspace

.PHONY: check
check: ## Check compilation (workspace + examples)
	cargo check --workspace --examples

.PHONY: lint
lint: ## Run clippy linter
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: fmt
fmt: ## Auto-format code
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check formatting without modifying
	cargo fmt --all -- --check

.PHONY: bench
bench: ## Run benchmarks
	cargo bench

.PHONY: run
run: ## Run main showcase app
	cargo run -p showcase

.PHONY: snake
snake: ## Run snake game demo
	cargo run --example snake

.PHONY: wasm
wasm: ## Serve WASM build (port 8080)
	trunk serve

.PHONY: doc
doc: ## Generate and open documentation
	cargo doc --no-deps --open

.PHONY: clean
clean: ## Remove build artifacts
	cargo clean

.PHONY: ci
ci: fmt-check lint test ## Run full CI pipeline locally

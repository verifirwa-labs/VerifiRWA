.PHONY: build test lint fmt clean deploy check

# Build all contracts for wasm32
build:
	cargo build --target wasm32-unknown-unknown --release --workspace

# Run all unit tests
test:
	cargo test --workspace

# Run clippy on all contracts
lint:
	cargo clippy --workspace -- -D warnings

# Format all contracts
fmt:
	cargo fmt --all

# Check formatting without modifying files
fmt-check:
	cargo fmt --check

# Run all checks (CI equivalent)
check: test lint fmt-check

# Build a single contract — usage: make build-contract CONTRACT=rwa-registry
build-contract:
	cargo build --target wasm32-unknown-unknown --release --package $(CONTRACT)

# Deploy to testnet (requires .env with ADMIN_SECRET_KEY)
deploy:
	bash scripts/deploy.sh

# Remove build artifacts
clean:
	cargo clean

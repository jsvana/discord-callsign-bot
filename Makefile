.PHONY: build run test clean fmt clippy install docker-build docker-run

# Build the project in release mode
build:
	cargo build --release

# Run the bot
run: build
	./target/release/discord-callsign-bot

# Run tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Clean build artifacts
clean:
	cargo clean
	rm -f members.txt

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy linter
clippy:
	cargo clippy -- -D warnings

# Check everything before commit
check: fmt-check clippy test

# Install to /usr/local/bin
install: build
	sudo cp target/release/discord-callsign-bot /usr/local/bin/

# Build Docker image
docker-build:
	docker build -t discord-callsign-bot .

# Run Docker container
docker-run: docker-build
	docker run -v $(PWD)/config.toml:/app/config.toml -v $(PWD):/output discord-callsign-bot

# Development build (faster, with debug symbols)
dev:
	cargo build

# Run in development mode with debug logging
dev-run: dev
	RUST_LOG=debug ./target/debug/discord-callsign-bot

# Show help
help:
	@echo "Available targets:"
	@echo "  build         - Build release binary"
	@echo "  run           - Build and run the bot"
	@echo "  test          - Run tests"
	@echo "  test-verbose  - Run tests with output"
	@echo "  clean         - Remove build artifacts"
	@echo "  fmt           - Format code"
	@echo "  fmt-check     - Check code formatting"
	@echo "  clippy        - Run linter"
	@echo "  check         - Run all checks (fmt, clippy, test)"
	@echo "  install       - Install to /usr/local/bin"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-run    - Run in Docker container"
	@echo "  dev           - Build debug binary"
	@echo "  dev-run       - Run with debug logging"

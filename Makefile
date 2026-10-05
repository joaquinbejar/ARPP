# Makefile for common tasks in a Rust project

# Default target
.PHONY: all
all: test fmt lint build

# Build the project
.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

# Run tests
.PHONY: test
test:
	cargo test

# Format the code
.PHONY: fmt
fmt:
	cargo +nightly fmt --all

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo +nightly fmt --check

# Run Clippy for linting
.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: lint-fix
lint-fix:
	cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged -- -D warnings

# Clean the project
.PHONY: clean
clean:
	cargo clean

# Pre-push checks
.PHONY: check
check: test fmt-check lint

# Run the project
.PHONY: run
run:
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty

.PHONY: pre-push
pre-push: fix fmt lint-fix test

.PHONY: doc
doc:
	cargo doc --open

.PHONY: publish
publish:
	cargo login ${CARGO_REGISTRY_TOKEN}
	cargo package
	cargo publish

# Minimum cargo-tarpaulin: older releases cannot read the coverage data written
# by Rust 1.99+. Reinstall when the local binary is missing or older than this,
# instead of only checking that it exists.
TARPAULIN_VERSION := 0.37.5

.PHONY: check-cargo-tarpaulin
check-cargo-tarpaulin:
	@v=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ -z "$$v" ] || [ "$$(printf '%s\n%s\n' '$(TARPAULIN_VERSION)' "$$v" | sort -V | head -n1)" != '$(TARPAULIN_VERSION)' ]; then \
		echo "Installing cargo-tarpaulin >= $(TARPAULIN_VERSION) (found: $${v:-none})"; \
		cargo install cargo-tarpaulin --locked --version '>=$(TARPAULIN_VERSION)'; \
	fi

.PHONY: coverage
coverage: check-cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --all-features --workspace --timeout 120 --out Xml

.PHONY: coverage-html
coverage-html: check-cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --all-features --workspace --timeout 120 --out Html

.PHONY: bench
bench:
	cargo bench -- --quiet
.PHONY: help test check clippy doc fmt mock clean example

help:
	@echo "papermint — The GOAT Thermal Printing Library"
	@echo ""
	@echo "Available make commands:"
	@echo "  make test     - Run all unit and integration tests (cargo test --tests)"
	@echo "  make check    - Fast compile & typecheck with all features"
	@echo "  make clippy   - Run strict Clippy linter with -D warnings"
	@echo "  make doc      - Build documentation (cargo doc --no-deps)"
	@echo "  make fmt      - Format Rust source code (cargo fmt)"
	@echo "  make mock     - Start virtual ESC/POS thermal printer server on port 9101"
	@echo "  make example  - Run the demo receipt printer against virtual mock printer"
	@echo "  make clean    - Clean target build artifacts"

test:
	cargo test --tests

check:
	cargo check --all-features

clippy:
	cargo clippy --tests -- -D warnings

doc:
	cargo doc --no-deps

fmt:
	cargo fmt

mock:
	node ../../scripts/mock-thermal-printer.mjs

example:
	cargo run --example print_demo

clean:
	cargo clean

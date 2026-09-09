.PHONY: help test check clippy doc fmt bench bench-check clean example

help:
	@echo "papermint — The GOAT Thermal Printing Library"
	@echo ""
	@echo "Available make commands:"
	@echo "  make test        - Run all unit and integration tests across workspace"
	@echo "  make check       - Fast compile & typecheck with all features across workspace"
	@echo "  make clippy      - Run strict Clippy linter with -D warnings on all targets"
	@echo "  make doc         - Build documentation (cargo doc --no-deps)"
	@echo "  make fmt         - Format all Rust source code (cargo fmt)"
	@echo "  make bench       - Run Criterion performance microbenchmarks"
	@echo "  make bench-check - Fast sanity check of benchmark harness"
	@echo "  make example     - Run the restaurant bill example"
	@echo "  make clean       - Clean target build artifacts"

test:
	cargo test --workspace --all-features

check:
	cargo check --workspace --all-features

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

doc:
	cargo doc --workspace --all-features --no-deps

fmt:
	cargo fmt

bench:
	cargo bench --bench receipt_benchmarks

bench-check:
	cargo bench --bench receipt_benchmarks -- --test

example:
	cargo run --example restaurant_bill

clean:
	cargo clean
